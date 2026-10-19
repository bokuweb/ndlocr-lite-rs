//! 複数ページを 1 回の API でまとめて OCR する高レベル API。
//!
//! `DeimPool` (行検出) と `ParseqPool` (行認識) を 1 つの `PageRecognizer` に
//! 抱えて、`recognize_pages_rgb_u8` で N ページを並列に処理する。狙いは
//! 2 つ:
//!
//! 1. **ページ並列**: ワーカースレッドが atomic counter でページを取り合う
//!    形で `std::thread::scope` 上にスポーン。DEIM Session N 個 + Parseq
//!    Session N 個と組み合わせれば、N ワーカーが互いを待たずに進める。
//! 2. **ステージ間オーバーラップ**: 各ワーカーは `detect → crop →
//!    recognize → reading_order` を直列に回すが、別ワーカーは別フェーズに
//!    居るので、DEIM プール / Parseq プールの稼働が重なる。
//!    クロスステージのパイプライン化が "勝手に" 起きる。
//!
//! 下流の置き換え対象は ellisii-ocr の `ocr_image_blocking` のようなページ
//! ごとの手書きグルーコード。pool の lifecycle (load 1 回 / 多ページ呼び出し)
//! も `PageRecognizer` 側に閉じ込めるので、consumer は `OnceLock` 等で 1 個
//! 持つだけでよくなる。
//!
//! 有効なDEIM行がない場合は、色背景を整えた横書き帯検出を試す。
//! 単純な塗り図形だけの帯は除き、供給された同じモデルへHorizontalで渡す。
//! 追加する認識はconfidence 0.9〜1を要求し、呼出側optionsでも絞り込む。
//! 検出/元認識のエラーはErrのまま返す。大きい空白による情報圧縮は、
//! 元画素を保持する区間の再認識と、必要なら同モデルの全体文脈確認で補う。
//! native入力比率を超える帯は、この確認に成功した場合のみ追加する。
//!
//! 元枠が覆わない画線を持つ単一横書き帯は、同じpoolで全体文脈を確認する。
//! native比率内の原画像/全文被覆帯だけを認識し、元結果の文字数加重平均confidenceを
//! 厳密に上回る候補を選ぶ。空出力から追加せず、追加確認の失敗時は元結果を保つ。
//!
//! 文書依存の構造ルール、辞書、複数モデルのcascadeはconsumer側で適用する。
//! 極端に横長で文字密度が高い行の短すぎる結果は、
//! 同じ供給モデルで安全な部分画像を追加認識する。全ての部分が高信頼度で
//! 十分な文字量を説明できた場合に採用する。consumer 側で必要に応じて
//! [`crate::postprocess::page_rules::apply_structural_rules`] を呼ぶ。

#![cfg(feature = "onnx")]

use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::cached::ParseqPool;
use super::deim_cached::DeimPool;
use crate::pipeline::crop::{BBox, crop_rgb_u8};
use crate::pipeline::reading_order::sort_bboxes_in_reading_order;

/// 1 行ぶんの認識結果。
#[derive(Clone, Debug, PartialEq)]
pub struct PageLine {
    pub bbox_xyxy: [i32; 4],
    pub text: String,
    pub confidence: f32,
}

/// 1 ページぶんの認識結果。`lines` は読み順 (横書き: 上→下→左→右、
/// 縦書き: 右→左→上→下) に並ぶ。
#[derive(Clone, Debug)]
pub struct PageResult {
    pub page_index: usize,
    pub lines: Vec<PageLine>,
}

/// `recognize_pages_rgb_u8` の挙動を調整するオプション。
#[derive(Clone, Copy, Debug)]
pub struct PageRecognizerOptions {
    /// DEIM 行検出の信頼度しきい値。
    pub det_conf_threshold: f32,
    /// 行認識の信頼度しきい値。これ未満は捨てる。
    /// DEIM未検出時に追加する帯は、別途0.9以上の認識支持も必要。
    pub min_line_confidence: f32,
    /// 認識テキストが空白のみの行を捨てるかどうか。
    pub drop_empty_text: bool,
}

impl Default for PageRecognizerOptions {
    fn default() -> Self {
        Self {
            det_conf_threshold: 0.3,
            min_line_confidence: 0.0,
            drop_empty_text: true,
        }
    }
}

/// DEIM + Parseq セッションプールを抱えたページ単位 OCR ランタイム。
pub struct PageRecognizer {
    deim: DeimPool,
    parseq: ParseqPool,
}

impl PageRecognizer {
    /// 行検出と行認識のモデルをまとめてロードする。`parallelism` は両プール
    /// 共通の Session 個数 (`DeimPool`/`ParseqPool` 各 N 個 = 計 2N 個)。
    pub fn load(
        det_model: &Path,
        parseq_model: &Path,
        charset_path: &Path,
        parallelism: usize,
    ) -> Result<Self> {
        let deim = DeimPool::load(det_model, parallelism)?;
        let parseq = ParseqPool::load(parseq_model, charset_path, parallelism)?;
        Ok(Self::from_pools(deim, parseq))
    }

    /// 既にロード済みの pool を所有してまとめる。consumer が cascade なら
    /// `from_pools` を使って独自構築 (例: 30/50/100 を別々に管理) もできる
    /// 余地を残しておく。
    /// 行は文の断片として扱い、別行で開かれた閉じ括弧も保持する。
    pub fn from_pools(deim: DeimPool, parseq: ParseqPool) -> Self {
        Self {
            deim,
            parseq: parseq.for_fragments(),
        }
    }

    /// 各プールの並列度。DeimPool と ParseqPool の小さい方を返す
    /// (どちらか片方が詰まると pipeline は遅い方で律速されるため)。
    pub fn parallelism(&self) -> usize {
        self.deim.parallelism().min(self.parseq.parallelism())
    }

    /// 複数ページをまとめて OCR する。並列度はプールの parallelism 上限。
    /// 結果は入力順 (`page_index = items の index`) に並ぶ。
    ///
    /// 1 ページ失敗時は全体が `Err`。部分失敗を許容したい呼び出し側は
    /// 1 ページずつ `recognize_page_rgb_u8` を回すか、上位で個別に
    /// `DeimPool` / `ParseqPool` を直接叩くこと。
    pub fn recognize_pages_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize)],
        opts: PageRecognizerOptions,
    ) -> Result<Vec<PageResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let workers = self.parallelism().max(1);
        let cursor = AtomicUsize::new(0);
        // 各 index に Mutex<Option<_>> を持たせる。worker は atomic で取った
        // index にしか触らないので Mutex は事実上 contention 0 だが、Vec を
        // 安全に共有可変するための借用検査の都合で 1 個ずつ Mutex を介す。
        let buf: Vec<Mutex<Option<PageResult>>> =
            (0..items.len()).map(|_| Mutex::new(None)).collect();

        let result: Result<()> = std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(workers);
            for _ in 0..workers {
                let cursor = &cursor;
                let buf = &buf;
                let deim = &self.deim;
                let parseq = &self.parseq;
                let h = scope.spawn(move || -> Result<()> {
                    loop {
                        let i = cursor.fetch_add(1, Ordering::Relaxed);
                        if i >= items.len() {
                            return Ok(());
                        }
                        let (rgb, w, h) = items[i];
                        let result = recognize_one_page(deim, parseq, rgb, w, h, i, opts)?;
                        let mut slot = buf[i]
                            .lock()
                            .map_err(|_| anyhow!("page slot mutex poisoned"))?;
                        *slot = Some(result);
                    }
                });
                handles.push(h);
            }
            for h in handles {
                h.join()
                    .unwrap_or_else(|_| Err(anyhow!("page worker panicked")))?;
            }
            Ok(())
        });
        result?;

        let mut out = Vec::with_capacity(items.len());
        for (i, slot) in buf.into_iter().enumerate() {
            let inner = slot
                .into_inner()
                .map_err(|_| anyhow!("page slot mutex poisoned"))?;
            out.push(inner.ok_or_else(|| anyhow!("page {i} produced no result"))?);
        }
        Ok(out)
    }

    /// 1 ページぶんの便利ラッパー。`recognize_pages_rgb_u8` を 1 要素で呼ぶ。
    pub fn recognize_page_rgb_u8(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
        opts: PageRecognizerOptions,
    ) -> Result<PageResult> {
        let mut out = self.recognize_pages_rgb_u8(&[(rgb, width, height)], opts)?;
        out.pop().ok_or_else(|| anyhow!("page produced no result"))
    }
}

fn supported_fallback_confidence(confidence: f32) -> bool {
    (0.9..=1.0).contains(&confidence)
}

fn recognize_one_page(
    deim: &DeimPool,
    parseq: &ParseqPool,
    rgb: &[u8],
    width: usize,
    height: usize,
    page_index: usize,
    opts: PageRecognizerOptions,
) -> Result<PageResult> {
    // 1) DEIM で行検出。pool 内で空 Session を取って 1 ページ分だけ走らせる。
    let dets = deim.detect_rgb_u8(rgb, width, height, opts.det_conf_threshold)?;

    // 2) line_* クラスのみ拾って bbox 化、画像内に収める。
    let mut bboxes: Vec<[i32; 4]> =
        crate::pipeline::line_detections::line_recognition_regions(&dets, rgb, width, height)
            .into_iter()
            .filter_map(|(_, bounds)| {
                let [x0, y0, x1, y1] = bounds;
                if x0 < 0 || y0 < 0 || x0 >= x1 || y0 >= y1 {
                    return None;
                }
                if (x1 as usize) > width || (y1 as usize) > height {
                    return None;
                }
                Some([x0, y0, x1, y1])
            })
            .collect();
    let using_bands = bboxes.is_empty();
    let fallback_rgb = if using_bands {
        crate::pipeline::background::normalize_chromatic_panels(rgb, width, height)
            .map_err(anyhow::Error::msg)?
    } else {
        std::borrow::Cow::Borrowed(rgb)
    };
    let rgb = fallback_rgb.as_ref();
    if bboxes.is_empty() {
        let bands = if width.saturating_mul(height) >= 1_000_000 {
            crate::pipeline::line_segment::detect_textline_bands_fast(rgb, width, height, 220)
        } else {
            crate::pipeline::line_segment::detect_textline_bands_naive(rgb, width, height, 220)
        };
        bboxes = bands
            .into_iter()
            .filter_map(|[x0, y0, x1, y1]| {
                Some([
                    i32::try_from(x0).ok()?,
                    i32::try_from(y0).ok()?,
                    i32::try_from(x1).ok()?,
                    i32::try_from(y1).ok()?,
                ])
            })
            .filter(|bounds| {
                crate::pipeline::band_text_support::has_text_like_ink(
                    rgb, width, height, *bounds, 220,
                )
            })
            .collect();
    }
    sort_bboxes_in_reading_order(&mut bboxes);

    if bboxes.is_empty() {
        return Ok(PageResult {
            page_index,
            lines: vec![],
        });
    }

    // 3) 各行を crop。ParseqPool::recognize_batch_rgb_u8 にまとめて投げる。
    let crops: Vec<([i32; 4], crate::pipeline::crop::CroppedImage)> = bboxes
        .iter()
        .map(|bbox| {
            let bb = BBox::new(
                bbox[0] as usize,
                bbox[1] as usize,
                bbox[2] as usize,
                bbox[3] as usize,
            );
            let crop = crop_rgb_u8(rgb, width, height, bb)
                .map_err(|e| anyhow!("crop bbox={:?}: {e}", bbox))?;
            Ok::<_, anyhow::Error>((*bbox, crop))
        })
        .collect::<Result<Vec<_>>>()?;

    let inputs: Vec<(&[u8], usize, usize)> = crops
        .iter()
        .map(|(_, c)| (c.data.as_slice(), c.width, c.height))
        .collect();
    // 各ページのバッチは「1 Session 専有」で投げる。複数ワーカーが同時に
    // ここに来ても、それぞれ別 Session を掴むので Parseq プールが詰まらない。
    let recs = if using_bands {
        parseq.recognize_batch_single_session_horizontal_rgb_u8(&inputs)?
    } else {
        parseq.recognize_batch_single_session_rgb_u8(&inputs)?
    };
    if recs.len() != crops.len() {
        return Err(anyhow!(
            "parseq batch returned {} for {} crops",
            recs.len(),
            crops.len()
        ));
    }

    // 4) 信頼度フィルタ + 空文字フィルタ。
    let mut lines: Vec<PageLine> = Vec::with_capacity(recs.len());
    let bounds: Vec<_> = crops.iter().map(|(bbox, _)| *bbox).collect();
    let isolated = super::dense_line_recovery::isolated_recovery_regions(&bounds);
    for (((bbox, crop), rec), isolated) in crops.into_iter().zip(recs).zip(isolated) {
        let (input_w, input_h) = parseq.input_dimensions();
        let overwide = crop.width as u128 * input_h as u128 > crop.height as u128 * input_w as u128;
        let rec = if using_bands && overwide {
            let Some(recovered) = super::band_line_recovery::recover_overwide_band(
                &crop.data,
                crop.width,
                crop.height,
                parseq.input_dimensions(),
                |data, w, h| parseq.recognize_horizontal_rgb_u8(data, w, h),
            ) else {
                continue;
            };
            recovered
        } else {
            rec
        };
        let rec = if isolated && !using_bands {
            super::dense_line_recovery::recover_dense_horizontal_line(
                &rec,
                &crop.data,
                crop.width,
                crop.height,
                parseq.input_dimensions(),
                |data, w, h| parseq.recognize_horizontal_rgb_u8(data, w, h),
            )
            .unwrap_or(rec)
        } else {
            rec
        };
        // Fallback adds lines without a detector score; require model support first.
        if using_bands && !supported_fallback_confidence(rec.mean_confidence) {
            continue;
        }
        if rec.mean_confidence < opts.min_line_confidence {
            continue;
        }
        if opts.drop_empty_text && rec.text.trim().is_empty() {
            continue;
        }
        lines.push(PageLine {
            bbox_xyxy: bbox,
            text: rec.text,
            confidence: rec.mean_confidence,
        });
    }
    let scored: Vec<_> = lines
        .iter()
        .map(|line| crate::pipeline::line_context::ScoredRegion {
            bounds: line.bbox_xyxy,
            text: &line.text,
            mean_confidence: line.confidence,
        })
        .collect();
    let keep = crate::pipeline::line_context::select_contexts(&scored, parseq.input_dimensions());
    let mut keep = keep.into_iter().peekable();
    let mut lines: Vec<PageLine> = lines
        .into_iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if keep.peek() == Some(&i) {
                keep.next();
                Some(line)
            } else {
                None
            }
        })
        .collect();
    let scored: Vec<_> = lines
        .iter()
        .map(|line| crate::pipeline::line_context::ScoredRegion {
            bounds: line.bbox_xyxy,
            text: &line.text,
            mean_confidence: line.confidence,
        })
        .collect();
    if let Some(context) = super::missing_line_context::recover_missing_line_context(
        &scored,
        rgb,
        width,
        height,
        parseq.input_dimensions(),
        |data, w, h| parseq.recognize_horizontal_rgb_u8(data, w, h),
    ) {
        lines = vec![PageLine {
            bbox_xyxy: context.bounds,
            text: context.recognized.text,
            confidence: context.recognized.mean_confidence,
        }];
    }
    Ok(PageResult { page_index, lines })
}

#[cfg(test)]
#[path = "../../tests/support/page_band_fallback_model.rs"]
mod band_fallback_tests;

#[cfg(test)]
#[path = "../../tests/support/contained_line_context_model.rs"]
mod contained_context_tests;

#[cfg(test)]
#[path = "../../tests/support/missing_line_context_model.rs"]
mod missing_context_tests;
