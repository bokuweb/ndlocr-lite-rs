//! DEIM 行検出 Session のキャッシュ版 (`ort` v2 backend)。
//!
//! `infer::deim::detect_rgb_u8` は呼び出しごとに `Session` を作っており、
//! 1 回あたり ~2s のオーバーヘッドが乗る。`DeimSession::load` でロードした
//! Session を保持しておけば `detect_rgb_u8` は Run のみになる。
//!
//! 複数ページを並列に処理したい場合は [`DeimPool`] を使う。`ParseqPool` と
//! 同じ思想で N 個の Session を抱え、ページ単位の `detect_batch_rgb_u8` で
//! `std::thread::scope` により分散する。
//!
//! `coreml` feature 有効時は [`super::ort_init`] が CoreML EP を登録するので、
//! M-series Mac では DEIM 推論が ANE/GPU に乗って CPU 比 ~3-5x 速い。
//!
//! API は [`infer::deim::detect_rgb_u8`] と同等の `Detection` を返す。

#![cfg(feature = "onnx")]

use anyhow::{Result, anyhow, bail};
use ndarray::{Array2, Array4};
use ort::inputs;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::TensorRef;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::deim::{Detection, ScaleContext, build_detections, preprocess_rgb_u8};
use super::ort_init::{OrtAnyhow, ensure_init};

pub struct DeimSession {
    session: Mutex<Session>,
    input_w: usize,
    input_h: usize,
}

fn build_session(model_path: &Path) -> Result<Session> {
    ensure_init();
    let session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(model_path)
        .anyort()?;
    Ok(session)
}

fn extract_input_size(session: &Session) -> Result<(usize, usize)> {
    let inputs = session.inputs();
    let m = inputs
        .first()
        .ok_or_else(|| anyhow!("model has no input"))?;
    let shape = m
        .dtype()
        .tensor_shape()
        .ok_or_else(|| anyhow!("input is not tensor"))?;
    let dims: &[i64] = shape;
    if dims.len() < 4 {
        bail!("unexpected model input rank");
    }
    let w = if dims[3] > 0 { dims[3] as usize } else { 800 };
    let h = if dims[2] > 0 { dims[2] as usize } else { 800 };
    Ok((w, h))
}

impl DeimSession {
    pub fn load(model_path: &Path) -> Result<Self> {
        if !model_path.is_file() {
            bail!("deim model not found: {}", model_path.display());
        }
        let session = build_session(model_path)?;
        let (input_w, input_h) = extract_input_size(&session)?;
        Ok(Self {
            session: Mutex::new(session),
            input_w,
            input_h,
        })
    }

    pub fn input_size(&self) -> (usize, usize) {
        (self.input_w, self.input_h)
    }

    pub fn detect_rgb_u8(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
        conf_threshold: f32,
    ) -> Result<Vec<Detection>> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("deim session mutex poisoned"))?;
        run_detect_on_session(
            &mut guard,
            self.input_w,
            self.input_h,
            rgb,
            width,
            height,
            conf_threshold,
        )
    }
}

/// 1 つの `Session` を借りて 1 ページの行検出を行う内部関数。
/// `DeimSession` / `DeimPool` の両方から呼ばれる。
fn run_detect_on_session(
    session: &mut Session,
    input_w: usize,
    input_h: usize,
    rgb: &[u8],
    width: usize,
    height: usize,
    conf_threshold: f32,
) -> Result<Vec<Detection>> {
    let prep = preprocess_rgb_u8(rgb, width, height, input_w, input_h)?;
    let image_arr: Array4<f32> = Array4::from_shape_vec((1, 3, input_h, input_w), prep.tensor)?;
    let size_arr: Array2<i64> =
        Array2::from_shape_vec((1, 2), vec![input_h as i64, input_w as i64])?;

    let img_tensor = TensorRef::from_array_view(image_arr.view()).anyort()?;
    let size_tensor = TensorRef::from_array_view(size_arr.view()).anyort()?;
    let outputs = session
        .run(inputs![
            "images" => img_tensor,
            "orig_target_sizes" => size_tensor,
        ])
        .anyort()?;

    let (_, labels) = outputs["labels"].try_extract_tensor::<i64>().anyort()?;
    let (_, boxes_flat) = outputs["boxes"].try_extract_tensor::<f32>().anyort()?;
    let (_, scores) = outputs["scores"].try_extract_tensor::<f32>().anyort()?;
    let char_counts: Vec<f32> = if let Some(v) = outputs.get("char_count") {
        let (_, raw) = v.try_extract_tensor::<i64>().anyort()?;
        raw.iter().map(|&v| v as f32).collect()
    } else {
        vec![100.0; scores.len()]
    };

    let mut boxes = Vec::new();
    for ch in boxes_flat.chunks_exact(4) {
        boxes.push([ch[0], ch[1], ch[2], ch[3]]);
    }

    let classes = default_ndl_classes();
    let mut dets = build_detections(
        labels,
        &boxes,
        scores,
        Some(&char_counts),
        &classes,
        conf_threshold,
        ScaleContext {
            image_width: prep.padded_wh as u32,
            image_height: prep.padded_wh as u32,
            input_width: input_w as u32,
            input_height: input_h as u32,
        },
    );
    for d in &mut dets {
        d.box_xyxy[0] = d.box_xyxy[0].clamp(0, width as i32);
        d.box_xyxy[1] = d.box_xyxy[1].clamp(0, height as i32);
        d.box_xyxy[2] = d.box_xyxy[2].clamp(0, width as i32);
        d.box_xyxy[3] = d.box_xyxy[3].clamp(0, height as i32);
    }
    dets.retain(|d| d.box_xyxy[2] > d.box_xyxy[0] && d.box_xyxy[3] > d.box_xyxy[1]);
    Ok(dets)
}

/// 同じモデルから複数の `Session` を作って round-robin に貸し出すプール。
///
/// ねらいは 2 つ:
///   1. **Session 構築コストの償却**: `infer::deim::detect_rgb_u8` を毎ページ
///      呼ぶと 1 回あたり ~2s 掛かる Session 構築が N 回繰り返される
///      (300 ページで 10 分以上)。
///   2. **ページ間並列**: 複数ページの行検出を `detect_batch_rgb_u8` で渡せば
///      `std::thread::scope` で各 Session に分散実行する。`ParseqPool` と
///      組み合わせれば、検出/認識の双方をページ並列に流せる。
pub struct DeimPool {
    sessions: Vec<Mutex<Session>>,
    next: AtomicUsize,
    input_w: usize,
    input_h: usize,
}

type IndexedImage<'a> = (usize, &'a [u8], usize, usize);
type IndexedDetections = (usize, Vec<Detection>);

impl DeimPool {
    pub fn load(model_path: &Path, parallelism: usize) -> Result<Self> {
        if !model_path.is_file() {
            bail!("deim model not found: {}", model_path.display());
        }
        let parallelism = parallelism.max(1);
        let mut sessions = Vec::with_capacity(parallelism);
        let mut input_w = 0usize;
        let mut input_h = 0usize;
        for _ in 0..parallelism {
            let session = build_session(model_path)?;
            let (w, h) = extract_input_size(&session)?;
            input_w = w;
            input_h = h;
            sessions.push(Mutex::new(session));
        }
        Ok(Self {
            sessions,
            next: AtomicUsize::new(0),
            input_w,
            input_h,
        })
    }

    pub fn parallelism(&self) -> usize {
        self.sessions.len()
    }

    pub fn input_size(&self) -> (usize, usize) {
        (self.input_w, self.input_h)
    }

    /// 1 ページの行検出。空いている Session を try_lock で拾い、無ければ
    /// round-robin の起点で待つ。`ParseqPool::recognize_rgb_u8` と同じ戦略。
    pub fn detect_rgb_u8(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
        conf_threshold: f32,
    ) -> Result<Vec<Detection>> {
        let n = self.sessions.len();
        let start = self.next.fetch_add(1, Ordering::Relaxed) % n;
        let mut acquired: Option<std::sync::MutexGuard<'_, Session>> = None;
        for off in 0..n {
            let i = (start + off) % n;
            if let Ok(g) = self.sessions[i].try_lock() {
                acquired = Some(g);
                break;
            }
        }
        let mut guard = match acquired {
            Some(g) => g,
            None => self.sessions[start]
                .lock()
                .map_err(|_| anyhow!("deim pool mutex poisoned"))?,
        };
        run_detect_on_session(
            &mut guard,
            self.input_w,
            self.input_h,
            rgb,
            width,
            height,
            conf_threshold,
        )
    }

    /// 複数ページの行検出をページ単位で並列実行する。返り値は入力順に揃える。
    ///
    /// 1 ページ 1 推論で、各 Session に均等にチャンクを割り当てる
    /// (DEIM は固定入力 H/W のため `parseq` のようなバッチ次元連結は使えない)。
    /// 1 ページが失敗すると全体が `Err` になる。部分失敗を許容したい呼び出し側
    /// は 1 ページずつ `detect_rgb_u8` を回すこと。
    pub fn detect_batch_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize)],
        conf_threshold: f32,
    ) -> Result<Vec<Vec<Detection>>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let indexed: Vec<IndexedImage<'_>> = items
            .iter()
            .enumerate()
            .map(|(i, (r, w, h))| (i, *r, *w, *h))
            .collect();
        let mut by_idx = self.detect_batch_indexed(&indexed, conf_threshold)?;
        by_idx.sort_by_key(|(i, _)| *i);
        Ok(by_idx.into_iter().map(|(_, d)| d).collect())
    }

    fn detect_batch_indexed(
        &self,
        items: &[IndexedImage<'_>],
        conf_threshold: f32,
    ) -> Result<Vec<IndexedDetections>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let n_sessions = self.sessions.len();
        let chunk_size = items.len().div_ceil(n_sessions).max(1);
        let chunks: Vec<&[IndexedImage<'_>]> = items.chunks(chunk_size).collect();
        let n_chunks = chunks.len();
        let mut chunk_results: Vec<Result<Vec<IndexedDetections>>> =
            (0..n_chunks).map(|_| Ok(Vec::new())).collect();

        std::thread::scope(|scope| {
            let mut handles: Vec<(usize, std::thread::ScopedJoinHandle<'_, _>)> =
                Vec::with_capacity(n_chunks);
            for (ci, chunk) in chunks.iter().enumerate() {
                if chunk.is_empty() {
                    continue;
                }
                let session_mu = &self.sessions[ci % n_sessions];
                let input_w = self.input_w;
                let input_h = self.input_h;
                let chunk_ref: &[IndexedImage<'_>] = chunk;
                let h = scope.spawn(move || -> Result<Vec<IndexedDetections>> {
                    let mut guard = session_mu
                        .lock()
                        .map_err(|_| anyhow!("deim pool mutex poisoned"))?;
                    let mut out = Vec::with_capacity(chunk_ref.len());
                    for (idx, rgb, w, h) in chunk_ref {
                        let dets = run_detect_on_session(
                            &mut guard,
                            input_w,
                            input_h,
                            rgb,
                            *w,
                            *h,
                            conf_threshold,
                        )?;
                        out.push((*idx, dets));
                    }
                    Ok(out)
                });
                handles.push((ci, h));
            }
            for (ci, h) in handles {
                chunk_results[ci] = h
                    .join()
                    .unwrap_or_else(|_| Err(anyhow!("deim batch worker panicked")));
            }
        });
        let mut out = Vec::with_capacity(items.len());
        for r in chunk_results {
            out.extend(r?);
        }
        Ok(out)
    }
}

fn default_ndl_classes() -> Vec<String> {
    [
        "text_block",
        "line_main",
        "line_caption",
        "line_ad",
        "line_note",
        "line_note_tochu",
        "block_fig",
        "block_ad",
        "block_pillar",
        "block_folio",
        "block_rubi",
        "block_chart",
        "block_eqn",
        "block_cfm",
        "block_eng",
        "block_table",
        "line_title",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}
