//! セッションをモデルロード後に保持する高速化版 API (`ort` v2 backend)。
//!
//! 旧実装 (`onnxruntime` 0.0.14) では `Environment` 構築・`Session` ロードが
//! 1 行 OCR ごとに走り、ページあたり数十秒掛かっていた。本モジュールは
//! [`ParseqSession`] / [`ParseqPool`] / [`ParseqCascadePool`] でロード結果を
//! `ort::session::Session` として保持し、`run` だけを繰り返し叩けるようにする。
//!
//! `coreml` feature 有効時は [`super::ort_init`] が CoreML EP を登録するので、
//! macOS (Apple Silicon) では ANE/GPU に乗って CPU 比 ~3-5x 速い。

#![cfg(feature = "onnx")]

use anyhow::{Context, Result, anyhow, bail};
use ndarray::Array;
use ort::inputs;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::TensorRef;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::ort_init::{OrtAnyhow, ensure_init};
use super::parseq::{
    PreprocessScratch, RecognizeResult, load_charset_from_yaml_str,
    predict_text_from_flat_logits_with_confidence, preprocess_rgb_u8,
    preprocess_rgb_u8_into_with_scratch, sanitize_recognized_text,
};

pub fn default_parseq_parallelism() -> usize {
    std::env::var("NDLOCR_PARSEQ_PARALLELISM")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .map(|n| n.clamp(1, 4))
        .unwrap_or(1)
}

/// `ort::session::Session` は内部に `*mut OrtSession` を持つだけで、`Send` だが
/// `Run` 自体は thread-safe ではない。Mutex で 1 スレッドずつ叩く。
struct SessionCell(Session);

type IndexedImage<'a> = (usize, &'a [u8], usize, usize);

pub struct ParseqSession {
    session: Mutex<SessionCell>,
    input_name: String,
    output_name: String,
    input_w: usize,
    input_h: usize,
    /// バッチ次元 (= dim[0]) が動的かどうか。`true` なら 1 度の `run` に
    /// 複数枚を詰めてバッチ推論できる。
    batch_dim_dynamic: bool,
    charset: Vec<char>,
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

fn extract_input_meta(session: &Session) -> Result<(String, String, usize, usize, bool)> {
    let inputs = session.inputs();
    let input = inputs
        .first()
        .ok_or_else(|| anyhow!("model has no input"))?;
    let shape = input
        .dtype()
        .tensor_shape()
        .ok_or_else(|| anyhow!("input is not a tensor"))?;
    let dims: &[i64] = shape;
    if dims.len() < 4 {
        bail!("unexpected model input rank");
    }
    if dims[2] <= 0 || dims[3] <= 0 {
        bail!("dynamic input H/W is unsupported");
    }
    let input_h = dims[2] as usize;
    let input_w = dims[3] as usize;
    let batch_dim_dynamic = dims[0] <= 0;
    let input_name = input.name().to_string();
    let outputs = session.outputs();
    let output_name = outputs
        .first()
        .ok_or_else(|| anyhow!("model has no output"))?
        .name()
        .to_string();
    Ok((input_name, output_name, input_w, input_h, batch_dim_dynamic))
}

impl ParseqSession {
    pub fn load(model_path: &Path, charset_path: &Path) -> Result<Self> {
        if !model_path.is_file() {
            bail!("parseq model not found: {}", model_path.display());
        }
        if !charset_path.is_file() {
            bail!("charset not found: {}", charset_path.display());
        }
        let yaml_body = std::fs::read_to_string(charset_path)
            .with_context(|| format!("failed to read {}", charset_path.display()))?;
        let charset = load_charset_from_yaml_str(&yaml_body)?;
        let session = build_session(model_path)?;
        let (input_name, output_name, input_w, input_h, batch_dim_dynamic) =
            extract_input_meta(&session)?;
        Ok(Self {
            session: Mutex::new(SessionCell(session)),
            input_name,
            output_name,
            input_w,
            input_h,
            batch_dim_dynamic,
            charset,
        })
    }

    pub fn supports_batch(&self) -> bool {
        self.batch_dim_dynamic
    }

    pub fn recognize_rgb_u8(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
    ) -> Result<RecognizeResult> {
        let single = [(rgb, width, height)];
        let mut out = {
            let mut guard = self
                .session
                .lock()
                .map_err(|_| anyhow!("parseq session mutex poisoned"))?;
            run_batch_on_session(
                &mut guard.0,
                &self.input_name,
                &self.output_name,
                self.input_w,
                self.input_h,
                &self.charset,
                &single,
            )?
        };
        let result = out.pop().ok_or_else(|| anyhow!("empty batch result"))?;
        Ok(result)
    }

    pub fn recognize_batch_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize)],
    ) -> Result<Vec<RecognizeResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        if !self.batch_dim_dynamic {
            let mut out = Vec::with_capacity(items.len());
            for (rgb, w, h) in items {
                out.push(self.recognize_rgb_u8(rgb, *w, *h)?);
            }
            return Ok(out);
        }
        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("parseq session mutex poisoned"))?;
        run_batch_on_session(
            &mut guard.0,
            &self.input_name,
            &self.output_name,
            self.input_w,
            self.input_h,
            &self.charset,
            items,
        )
    }
}

/// 1 つの `Session` で `[N, 3, H, W]` 入力を組み立てて推論する内部関数。
fn run_batch_on_session(
    session: &mut Session,
    input_name: &str,
    output_name: &str,
    input_w: usize,
    input_h: usize,
    charset: &[char],
    items: &[(&[u8], usize, usize)],
) -> Result<Vec<RecognizeResult>> {
    use rayon::prelude::*;
    let n = items.len();
    let plane = 3 * input_h * input_w;
    let mut buf = vec![0.0f32; n * plane];
    // 各行の preprocess を rayon で並列化。小さい batch や縦長 crop は direct
    // write が有利だが、大きい横書き batch は local Vec -> memcpy の方が速い
    // ケースがあるため、bench に基づいて従来 path を残す。
    let plane_local = plane;
    let use_direct_slots = items.len() < 16 || items.iter().any(|(_, w, h)| h > w);
    let preprocess_results: Result<()> = if use_direct_slots {
        buf.par_chunks_mut(plane_local)
            .zip(items.par_iter())
            .try_for_each_init(
                PreprocessScratch::new,
                |scratch, (slot, (rgb, w, h))| -> Result<()> {
                    preprocess_rgb_u8_into_with_scratch(
                        slot, rgb, *w, *h, input_w, input_h, scratch,
                    )
                },
            )
    } else {
        buf.par_chunks_mut(plane_local)
            .zip(items.par_iter())
            .try_for_each(|(slot, (rgb, w, h))| -> Result<()> {
                let tensor = preprocess_rgb_u8(rgb, *w, *h, input_w, input_h)?;
                slot.copy_from_slice(&tensor);
                Ok(())
            })
    };
    preprocess_results?;
    let arr = Array::from_shape_vec((n, 3, input_h, input_w), buf)?;
    let tensor = TensorRef::from_array_view(arr.view()).anyort()?;
    let outputs = session.run(inputs![input_name => tensor]).anyort()?;
    let (shape, data) = outputs[output_name].try_extract_tensor::<f32>().anyort()?;
    let shape: Vec<i64> = shape.to_vec();
    // 期待出力形状: [N, T, C]。古い ONNX で [T, C] が返る場合は N=1 のみ許容。
    let (t, c) = match shape.as_slice() {
        [bn, t, c] if *bn as usize == n => (*t as usize, *c as usize),
        [t, c] if n == 1 => (*t as usize, *c as usize),
        _ => bail!("unexpected batch output shape {:?} for n={n}", shape),
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let off = i * t * c;
        let mut r =
            predict_text_from_flat_logits_with_confidence(&data[off..off + t * c], t, c, charset)?;
        r.text = sanitize_recognized_text(&r.text);
        out.push(r);
    }
    Ok(out)
}

/// 同じモデルから複数の `Session` を作って round-robin に貸し出すプール。
pub struct ParseqPool {
    sessions: Vec<Mutex<SessionCell>>,
    next: AtomicUsize,
    input_name: String,
    output_name: String,
    input_w: usize,
    input_h: usize,
    batch_dim_dynamic: bool,
    charset: Vec<char>,
}

impl ParseqPool {
    pub fn load(model_path: &Path, charset_path: &Path, parallelism: usize) -> Result<Self> {
        if !model_path.is_file() {
            bail!("parseq model not found: {}", model_path.display());
        }
        if !charset_path.is_file() {
            bail!("charset not found: {}", charset_path.display());
        }
        let parallelism = parallelism.max(1);
        let yaml_body = std::fs::read_to_string(charset_path)
            .with_context(|| format!("failed to read {}", charset_path.display()))?;
        let charset = load_charset_from_yaml_str(&yaml_body)?;

        let mut sessions = Vec::with_capacity(parallelism);
        let mut input_name = String::new();
        let mut output_name = String::new();
        let mut input_w = 0usize;
        let mut input_h = 0usize;
        let mut batch_dim_dynamic = false;
        for _ in 0..parallelism {
            let session = build_session(model_path)?;
            let (i_name, o_name, iw, ih, dyn_b) = extract_input_meta(&session)?;
            input_name = i_name;
            output_name = o_name;
            input_w = iw;
            input_h = ih;
            batch_dim_dynamic = dyn_b;
            sessions.push(Mutex::new(SessionCell(session)));
        }
        Ok(Self {
            sessions,
            next: AtomicUsize::new(0),
            input_name,
            output_name,
            input_w,
            input_h,
            batch_dim_dynamic,
            charset,
        })
    }

    pub fn parallelism(&self) -> usize {
        self.sessions.len()
    }

    pub fn supports_batch(&self) -> bool {
        self.batch_dim_dynamic
    }

    pub fn recognize_rgb_u8(
        &self,
        rgb: &[u8],
        width: usize,
        height: usize,
    ) -> Result<RecognizeResult> {
        let n = self.sessions.len();
        let start = self.next.fetch_add(1, Ordering::Relaxed) % n;
        let mut acquired: Option<std::sync::MutexGuard<'_, SessionCell>> = None;
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
                .map_err(|_| anyhow!("parseq pool mutex poisoned"))?,
        };
        let single = [(rgb, width, height)];
        let mut out = run_batch_on_session(
            &mut guard.0,
            &self.input_name,
            &self.output_name,
            self.input_w,
            self.input_h,
            &self.charset,
            &single,
        )?;
        out.pop().ok_or_else(|| anyhow!("empty batch result"))
    }

    /// 並列度を使い切らずに **1 つの Session** だけを借りてバッチ推論する。
    ///
    /// `recognize_batch_rgb_u8` は内部で N 個の Session を全部使ってチャンク
    /// 並列実行する。このとき呼び出し元自身が複数スレッドから並列に呼ぶと、
    /// 全 Session が常に locked になって他スレッドが進めなくなる
    /// (worker > pool size でも実質的に直列化)。
    ///
    /// `PageRecognizer` のようにページ単位で並列ワーカーを走らせていて、
    /// 各ワーカーが「自分の 1 ページ分の行」だけをまとめて推論したい場合は
    /// こちらを使う。1 Session は専有するが、他の Session は他ワーカーが
    /// 同時に使えるので、ページ間並列が崩れない。
    pub fn recognize_batch_single_session_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize)],
    ) -> Result<Vec<RecognizeResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let n = self.sessions.len();
        let start = self.next.fetch_add(1, Ordering::Relaxed) % n;
        let mut acquired: Option<std::sync::MutexGuard<'_, SessionCell>> = None;
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
                .map_err(|_| anyhow!("parseq pool mutex poisoned"))?,
        };
        if self.batch_dim_dynamic {
            run_batch_on_session(
                &mut guard.0,
                &self.input_name,
                &self.output_name,
                self.input_w,
                self.input_h,
                &self.charset,
                items,
            )
        } else {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                let single = [*it];
                out.extend(run_batch_on_session(
                    &mut guard.0,
                    &self.input_name,
                    &self.output_name,
                    self.input_w,
                    self.input_h,
                    &self.charset,
                    &single,
                )?);
            }
            Ok(out)
        }
    }

    fn recognize_batch_indexed(
        &self,
        items: &[IndexedImage<'_>],
    ) -> Result<Vec<(usize, RecognizeResult)>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let n_sessions = self.sessions.len();
        let chunk_size = items.len().div_ceil(n_sessions).max(1);
        let chunks: Vec<&[IndexedImage<'_>]> = items.chunks(chunk_size).collect();
        let n_chunks = chunks.len();
        let mut chunk_results: Vec<Result<Vec<(usize, RecognizeResult)>>> =
            (0..n_chunks).map(|_| Ok(Vec::new())).collect();

        std::thread::scope(|scope| {
            let mut handles: Vec<(usize, std::thread::ScopedJoinHandle<'_, _>)> =
                Vec::with_capacity(n_chunks);
            for (ci, chunk) in chunks.iter().enumerate() {
                if chunk.is_empty() {
                    continue;
                }
                let session_mu = &self.sessions[ci % n_sessions];
                let input_name = &self.input_name;
                let output_name = &self.output_name;
                let input_w = self.input_w;
                let input_h = self.input_h;
                let charset = &self.charset;
                let dyn_batch = self.batch_dim_dynamic;
                let chunk_ref: &[IndexedImage<'_>] = chunk;
                let h = scope.spawn(move || -> Result<Vec<(usize, RecognizeResult)>> {
                    let mut guard = session_mu
                        .lock()
                        .map_err(|_| anyhow!("parseq pool mutex poisoned"))?;
                    let session = &mut guard.0;
                    let payload: Vec<(&[u8], usize, usize)> = chunk_ref
                        .iter()
                        .map(|(_, r, w, hh)| (*r, *w, *hh))
                        .collect();
                    let recs = if dyn_batch {
                        run_batch_on_session(
                            session,
                            input_name,
                            output_name,
                            input_w,
                            input_h,
                            charset,
                            &payload,
                        )?
                    } else {
                        let mut out = Vec::with_capacity(payload.len());
                        for it in &payload {
                            let single = [*it];
                            out.extend(run_batch_on_session(
                                session,
                                input_name,
                                output_name,
                                input_w,
                                input_h,
                                charset,
                                &single,
                            )?);
                        }
                        out
                    };
                    Ok(chunk_ref
                        .iter()
                        .zip(recs)
                        .map(|((idx, _, _, _), r)| (*idx, r))
                        .collect())
                });
                handles.push((ci, h));
            }
            for (ci, h) in handles {
                chunk_results[ci] = h
                    .join()
                    .unwrap_or_else(|_| Err(anyhow!("parseq batch worker panicked")));
            }
        });
        let mut out = Vec::with_capacity(items.len());
        for r in chunk_results {
            out.extend(r?);
        }
        Ok(out)
    }

    pub fn recognize_batch_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize)],
    ) -> Result<Vec<RecognizeResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let indexed: Vec<(usize, &[u8], usize, usize)> = items
            .iter()
            .enumerate()
            .map(|(i, (r, w, h))| (i, *r, *w, *h))
            .collect();
        let mut by_idx = self.recognize_batch_indexed(&indexed)?;
        by_idx.sort_by_key(|(i, _)| *i);
        Ok(by_idx.into_iter().map(|(_, r)| r).collect())
    }
}

/// 3 つの parseq モデル (16x256/16x384/16x768) を保持し、行ごとに最適な
/// モデルを選んで推論するカスケード。
pub struct ParseqCascadePool {
    pool30: ParseqPool,
    pool50: ParseqPool,
    pool100: ParseqPool,
}

impl ParseqCascadePool {
    pub fn load(
        model30: &Path,
        model50: &Path,
        model100: &Path,
        charset: &Path,
        parallelism: usize,
    ) -> Result<Self> {
        // 3 モデルを `thread::scope` で並列ロード (cold start ~3x 短縮)。
        let mut r30: Result<ParseqPool> = Err(anyhow!("not run"));
        let mut r50: Result<ParseqPool> = Err(anyhow!("not run"));
        let mut r100: Result<ParseqPool> = Err(anyhow!("not run"));
        std::thread::scope(|s| {
            let h30 = s.spawn(|| ParseqPool::load(model30, charset, parallelism));
            let h50 = s.spawn(|| ParseqPool::load(model50, charset, parallelism));
            let h100 = s.spawn(|| ParseqPool::load(model100, charset, parallelism));
            r30 = h30
                .join()
                .unwrap_or_else(|_| Err(anyhow!("pool30 load panicked")));
            r50 = h50
                .join()
                .unwrap_or_else(|_| Err(anyhow!("pool50 load panicked")));
            r100 = h100
                .join()
                .unwrap_or_else(|_| Err(anyhow!("pool100 load panicked")));
        });
        Ok(Self {
            pool30: r30?,
            pool50: r50?,
            pool100: r100?,
        })
    }

    pub fn parallelism(&self) -> usize {
        self.pool100.parallelism()
    }

    pub fn recognize_batch_with_buckets_rgb_u8(
        &self,
        items: &[(&[u8], usize, usize, Option<f32>)],
    ) -> Result<Vec<RecognizeResult>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let mut idx30: Vec<(usize, &[u8], usize, usize)> = Vec::new();
        let mut idx50: Vec<(usize, &[u8], usize, usize)> = Vec::new();
        let mut idx100: Vec<(usize, &[u8], usize, usize)> = Vec::new();
        for (i, (rgb, w, h, pcc)) in items.iter().enumerate() {
            let bucket = bucket_from_pred_char_count(*pcc);
            match bucket {
                3 => idx30.push((i, rgb, *w, *h)),
                2 => idx50.push((i, rgb, *w, *h)),
                _ => idx100.push((i, rgb, *w, *h)),
            }
        }

        let mut r30: Result<Vec<(usize, RecognizeResult)>> = Ok(Vec::new());
        let mut r50: Result<Vec<(usize, RecognizeResult)>> = Ok(Vec::new());
        let mut r100: Result<Vec<(usize, RecognizeResult)>> = Ok(Vec::new());
        std::thread::scope(|s| {
            let h30 = s.spawn(|| self.pool30.recognize_batch_indexed(&idx30));
            let h50 = s.spawn(|| self.pool50.recognize_batch_indexed(&idx50));
            let h100 = s.spawn(|| self.pool100.recognize_batch_indexed(&idx100));
            r30 = h30
                .join()
                .unwrap_or_else(|_| Err(anyhow!("cascade 30 worker panicked")));
            r50 = h50
                .join()
                .unwrap_or_else(|_| Err(anyhow!("cascade 50 worker panicked")));
            r100 = h100
                .join()
                .unwrap_or_else(|_| Err(anyhow!("cascade 100 worker panicked")));
        });

        let mut all: Vec<(usize, RecognizeResult)> = Vec::with_capacity(items.len());
        all.extend(r30?);
        all.extend(r50?);
        all.extend(r100?);
        all.sort_by_key(|(i, _)| *i);

        // フォールバック: 30 で >=25 字 → 50 へ、50 で >=45 字 → 100 へ。
        let mut redo50: Vec<(usize, &[u8], usize, usize)> = Vec::new();
        let mut redo100: Vec<(usize, &[u8], usize, usize)> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let bucket = bucket_from_pred_char_count(item.3);
            let len = all[i].1.text.chars().count();
            if bucket == 3 && len >= 25 {
                redo50.push((i, item.0, item.1, item.2));
            } else if bucket == 2 && len >= 45 {
                redo100.push((i, item.0, item.1, item.2));
            }
        }
        if !redo50.is_empty() || !redo100.is_empty() {
            let mut rd50: Result<Vec<(usize, RecognizeResult)>> = Ok(Vec::new());
            let mut rd100: Result<Vec<(usize, RecognizeResult)>> = Ok(Vec::new());
            std::thread::scope(|s| {
                let h1 = s.spawn(|| self.pool50.recognize_batch_indexed(&redo50));
                let h2 = s.spawn(|| self.pool100.recognize_batch_indexed(&redo100));
                rd50 = h1
                    .join()
                    .unwrap_or_else(|_| Err(anyhow!("cascade redo50 panicked")));
                rd100 = h2
                    .join()
                    .unwrap_or_else(|_| Err(anyhow!("cascade redo100 panicked")));
            });
            for (i, r) in rd50? {
                if r.text.chars().count() >= 45 {
                    let it = items[i];
                    let single = [(i, it.0, it.1, it.2)];
                    let r2 = self.pool100.recognize_batch_indexed(&single)?;
                    if let Some((_, rr)) = r2.into_iter().next() {
                        all[i].1 = rr;
                        continue;
                    }
                }
                all[i].1 = r;
            }
            for (i, r) in rd100? {
                all[i].1 = r;
            }
        }

        Ok(all.into_iter().map(|(_, r)| r).collect())
    }
}

fn bucket_from_pred_char_count(pcc: Option<f32>) -> u8 {
    match pcc {
        Some(v) if (v - 3.0).abs() < 0.2 => 3,
        Some(v) if (v - 2.0).abs() < 0.2 => 2,
        _ => 100,
    }
}
