use crate::io;
use anyhow::{Result, bail};
use std::path::Path;

pub struct DeimPreprocessOutput {
    pub tensor: Vec<f32>,
    pub padded_wh: usize,
}

pub struct ScaleContext {
    pub image_width: u32,
    pub image_height: u32,
    pub input_width: u32,
    pub input_height: u32,
}

#[derive(Clone, Debug)]
pub struct Detection {
    pub class_index: usize,
    pub confidence: f32,
    pub box_xyxy: [i32; 4],
    pub pred_char_count: f32,
    pub class_name: String,
}

pub fn preprocess_rgb_u8(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> Result<DeimPreprocessOutput> {
    let expected = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("image size overflow"))?;
    if rgb.len() != expected || input_width == 0 || input_height == 0 {
        bail!("invalid input");
    }
    let max_wh = width.max(height);
    // pad_to_square 後 resize_nearest という 2 段構成は中間バッファ
    // (max_wh^2 * 3 byte; 2048x2048 で ~12MB) を取るのでヒープ圧が重い。
    // pad の領域 (右下) は 0 のままで output からはサンプリングされる側
    // ではなく、resize の補完計算上は元画像の範囲外を踏まないので、
    // 直接 padded coord → 元画像 → output に nearest で詰めれば中間バッファ
    // を完全に省ける。
    let mut out = vec![0.0_f32; 3 * input_width * input_height];
    let plane = input_width * input_height;
    let mean = [0.485_f32, 0.456_f32, 0.406_f32];
    let std = [0.229_f32, 0.224_f32, 0.225_f32];
    for y in 0..input_height {
        let sy = y * max_wh / input_height;
        let in_h = sy < height;
        for x in 0..input_width {
            let sx = x * max_wh / input_width;
            let i = y * input_width + x;
            if in_h && sx < width {
                let s = (sy * width + sx) * 3;
                out[i] = (rgb[s] as f32 / 255.0 - mean[0]) / std[0];
                out[plane + i] = (rgb[s + 1] as f32 / 255.0 - mean[1]) / std[1];
                out[plane * 2 + i] = (rgb[s + 2] as f32 / 255.0 - mean[2]) / std[2];
            } else {
                // padding 領域: 元の pad_to_square は 0 で埋め、その後正規化を
                // 通すので最終的には -mean/std になる (= padding が黒の正規化)。
                out[i] = -mean[0] / std[0];
                out[plane + i] = -mean[1] / std[1];
                out[plane * 2 + i] = -mean[2] / std[2];
            }
        }
    }
    Ok(DeimPreprocessOutput {
        tensor: out,
        padded_wh: max_wh,
    })
}

pub fn scale_boxes_to_image_space(
    boxes_xyxy: &[[f32; 4]],
    image_width: u32,
    image_height: u32,
    input_width: u32,
    input_height: u32,
) -> Vec<[i32; 4]> {
    let xs = image_width as f32 / input_width as f32;
    let ys = image_height as f32 / input_height as f32;
    boxes_xyxy
        .iter()
        .map(|b| {
            [
                (b[0] * xs) as i32,
                (b[1] * ys) as i32,
                (b[2] * xs) as i32,
                (b[3] * ys) as i32,
            ]
        })
        .collect()
}

pub fn build_detections(
    class_ids: &[i64],
    boxes_xyxy: &[[f32; 4]],
    scores: &[f32],
    char_counts: Option<&[f32]>,
    classes: &[String],
    conf_threshold: f32,
    scale: ScaleContext,
) -> Vec<Detection> {
    let scaled = scale_boxes_to_image_space(
        boxes_xyxy,
        scale.image_width,
        scale.image_height,
        scale.input_width,
        scale.input_height,
    );
    let n = class_ids.len().min(scaled.len()).min(scores.len());
    let mut out = Vec::new();
    for i in 0..n {
        if scores[i] <= conf_threshold || class_ids[i] <= 0 {
            continue;
        }
        let ci = (class_ids[i] - 1) as usize;
        let Some(name) = classes.get(ci) else {
            continue;
        };
        out.push(Detection {
            class_index: ci,
            confidence: scores[i],
            box_xyxy: scaled[i],
            pred_char_count: char_counts.and_then(|v| v.get(i).copied()).unwrap_or(100.0),
            class_name: name.clone(),
        });
    }
    out
}

pub fn smoke_detect(model_path: &Path, image_path: &Path) -> Result<()> {
    if !image_path.is_file() || !model_path.is_file() {
        bail!("file not found");
    }
    let img = io::load_rgb_u8(image_path)?;
    let _ = preprocess_rgb_u8(&img.data, img.width, img.height, 800, 800)?;
    smoke_detect_impl(model_path)
}

pub fn detect_rgb_u8(
    model_path: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    conf_threshold: f32,
) -> Result<Vec<Detection>> {
    detect_rgb_u8_impl(model_path, rgb, width, height, conf_threshold)
}

#[cfg(feature = "onnx")]
fn smoke_detect_impl(model_path: &Path) -> Result<()> {
    use crate::infer::ort_init::OrtAnyhow;
    use ort::session::Session;
    use ort::session::builder::GraphOptimizationLevel;
    crate::infer::ort_init::ensure_init();
    let _session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(model_path)
        .anyort()?;
    Ok(())
}

#[cfg(feature = "onnx")]
fn detect_rgb_u8_impl(
    model_path: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    conf_threshold: f32,
) -> Result<Vec<Detection>> {
    use crate::infer::ort_init::OrtAnyhow;
    use anyhow::anyhow;
    use ndarray::{Array2, Array4};
    use ort::inputs;
    use ort::session::Session;
    use ort::session::builder::GraphOptimizationLevel;
    use ort::value::TensorRef;

    if !model_path.is_file() {
        bail!("file not found");
    }
    crate::infer::ort_init::ensure_init();
    let mut session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(model_path)
        .anyort()?;

    // モデル入力 H/W をクエリ。固定 (例: 800x800) ならその値、動的なら 800 を使う。
    let (input_w, input_h) = {
        let inputs_meta = session.inputs();
        let m = inputs_meta
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

    let prep = preprocess_rgb_u8(rgb, width, height, input_w, input_h)?;
    let image_arr: Array4<f32> = Array4::from_shape_vec((1, 3, input_h, input_w), prep.tensor)?;
    let size_arr: Array2<i64> =
        Array2::from_shape_vec((1, 2), vec![input_h as i64, input_w as i64])?;

    let img_t = TensorRef::from_array_view(image_arr.view()).anyort()?;
    let size_t = TensorRef::from_array_view(size_arr.view()).anyort()?;
    let outputs = session
        .run(inputs![
            "images" => img_t,
            "orig_target_sizes" => size_t,
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
#[cfg(feature = "onnx")]
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

#[cfg(not(feature = "onnx"))]
fn smoke_detect_impl(_model_path: &Path) -> Result<()> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

#[cfg(not(feature = "onnx"))]
fn detect_rgb_u8_impl(
    _model_path: &Path,
    _rgb: &[u8],
    _width: usize,
    _height: usize,
    _conf_threshold: f32,
) -> Result<Vec<Detection>> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}
