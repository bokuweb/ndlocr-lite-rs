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
//! 認識精度に関わる post-processing (構造ルール、辞書、cascade fallback)
//! は本モジュールには含めない。consumer 側で必要に応じて
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
        Ok(Self { deim, parseq })
    }

    /// 既にロード済みの pool を所有してまとめる。consumer が cascade なら
    /// `from_pools` を使って独自構築 (例: 30/50/100 を別々に管理) もできる
    /// 余地を残しておく。
    pub fn from_pools(deim: DeimPool, parseq: ParseqPool) -> Self {
        Self { deim, parseq }
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
    let mut bboxes: Vec<[i32; 4]> = dets
        .into_iter()
        .filter(|d| d.class_name.starts_with("line_"))
        .filter_map(|d| {
            let [x0, y0, x1, y1] = d.box_xyxy;
            if x0 < 0 || y0 < 0 || x0 >= x1 || y0 >= y1 {
                return None;
            }
            if (x1 as usize) > width || (y1 as usize) > height {
                return None;
            }
            Some([x0, y0, x1, y1])
        })
        .collect();
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
    let recs = parseq.recognize_batch_single_session_rgb_u8(&inputs)?;
    if recs.len() != crops.len() {
        return Err(anyhow!(
            "parseq batch returned {} for {} crops",
            recs.len(),
            crops.len()
        ));
    }

    // 4) 信頼度フィルタ + 空文字フィルタ。
    let mut lines: Vec<PageLine> = Vec::with_capacity(recs.len());
    for ((bbox, _), rec) in crops.into_iter().zip(recs) {
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
    Ok(PageResult { page_index, lines })
}
