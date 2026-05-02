//! DEIM 行検出 Session のキャッシュ版 (`ort` v2 backend)。
//!
//! `infer::deim::detect_rgb_u8` は呼び出しごとに `Session` を作っており、
//! 1 回あたり ~2s のオーバーヘッドが乗る。`DeimSession::load` でロードした
//! Session を保持しておけば `detect_rgb_u8` は Run のみになる。
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

use super::deim::{Detection, ScaleContext, build_detections, preprocess_rgb_u8};
use super::ort_init::{OrtAnyhow, ensure_init};

pub struct DeimSession {
    session: Mutex<Session>,
    input_w: usize,
    input_h: usize,
}

impl DeimSession {
    pub fn load(model_path: &Path) -> Result<Self> {
        if !model_path.is_file() {
            bail!("deim model not found: {}", model_path.display());
        }
        ensure_init();
        let session = Session::builder()
            .anyort()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .anyort()?
            .with_intra_threads(1)
            .anyort()?
            .commit_from_file(model_path)
            .anyort()?;

        // モデル入力 H/W をクエリ。動的なら 800 にフォールバック。
        let (input_w, input_h) = {
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
            (w, h)
        };

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
        let prep = preprocess_rgb_u8(rgb, width, height, self.input_w, self.input_h)?;
        let image_arr: Array4<f32> =
            Array4::from_shape_vec((1, 3, self.input_h, self.input_w), prep.tensor)?;
        let size_arr: Array2<i64> =
            Array2::from_shape_vec((1, 2), vec![self.input_h as i64, self.input_w as i64])?;

        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("deim session mutex poisoned"))?;
        let img_tensor = TensorRef::from_array_view(image_arr.view()).anyort()?;
        let size_tensor = TensorRef::from_array_view(size_arr.view()).anyort()?;
        let outputs = guard
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
                input_width: self.input_w as u32,
                input_height: self.input_h as u32,
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
