use crate::io;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RecognizeResult {
    pub text: String,
    pub mean_confidence: f32,
}

/// PARSEQ 入力テンソルを作る（`ndlocr/src/parseq.py` と同じ方針）。
///
/// - 縦長なら 90° 反時計回りに回転（`h > w` のとき `cv2.ROTATE_90_COUNTERCLOCKWISE` に相当）
/// - `cv2.resize(..., INTER_LINEAR)` に相当する **バイリニア**で `(input_w, input_h)` へ **全面ストレッチ**
/// - PIL RGB を `[:, :, ::-1]` したのと同じ **BGR 順**で NCHW（学習・Python 推論と整合）
pub fn preprocess_rgb_u8(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> Result<Vec<f32>> {
    let expected = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("image size overflow"))?;
    if rgb.len() != expected {
        bail!("invalid RGB buffer length");
    }
    let mut out = vec![0.0_f32; 3 * input_width * input_height];
    let rotated = height > width;
    let (source_w, source_h) = if rotated {
        (height, width)
    } else {
        (width, height)
    };
    if input_width == 0 || input_height == 0 {
        bail!("invalid resize dimension");
    }
    let plane = input_width * input_height;
    for y in 0..input_height {
        let sy = resize_source_coord(y, input_height, source_h);
        let y0 = sy.floor() as usize;
        let y1 = (y0 + 1).min(source_h - 1);
        let wy = sy - y0 as f32;
        for x in 0..input_width {
            let sx = resize_source_coord(x, input_width, source_w);
            let x0 = sx.floor() as usize;
            let x1 = (x0 + 1).min(source_w - 1);
            let wx = sx - x0 as f32;
            let rgb_px = bilinear_rgb(rgb, width, height, rotated, x0, y0, x1, y1, wx, wy);
            let i = y * input_width + x;
            out[i] = rgb_px[2] / 127.5 - 1.0;
            out[plane + i] = rgb_px[1] / 127.5 - 1.0;
            out[plane * 2 + i] = rgb_px[0] / 127.5 - 1.0;
        }
    }
    Ok(out)
}

fn resize_source_coord(dst: usize, dst_len: usize, src_len: usize) -> f32 {
    if dst_len <= 1 {
        return 0.0;
    }
    (((dst as f32 + 0.5) * src_len as f32 / dst_len as f32) - 0.5).clamp(0.0, (src_len - 1) as f32)
}

fn bilinear_rgb(
    rgb: &[u8],
    width: usize,
    height: usize,
    rotated: bool,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    wx: f32,
    wy: f32,
) -> [f32; 3] {
    let p00 = source_rgb(rgb, width, height, rotated, x0, y0);
    let p10 = source_rgb(rgb, width, height, rotated, x1, y0);
    let p01 = source_rgb(rgb, width, height, rotated, x0, y1);
    let p11 = source_rgb(rgb, width, height, rotated, x1, y1);
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let top = p00[c] as f32 * (1.0 - wx) + p10[c] as f32 * wx;
        let bottom = p01[c] as f32 * (1.0 - wx) + p11[c] as f32 * wx;
        out[c] = top * (1.0 - wy) + bottom * wy;
    }
    out
}

fn source_rgb(
    rgb: &[u8],
    width: usize,
    height: usize,
    rotated: bool,
    x: usize,
    y: usize,
) -> [u8; 3] {
    let (ox, oy) = if rotated { (width - 1 - y, x) } else { (x, y) };
    debug_assert!(ox < width);
    debug_assert!(oy < height);
    let i = (oy * width + ox) * 3;
    [rgb[i], rgb[i + 1], rgb[i + 2]]
}

pub fn decode_indices(indices: &[i64], charset: &[char]) -> String {
    let mut out = String::new();
    for &idx in indices {
        if idx == 0 {
            break;
        }
        if idx <= 0 {
            continue;
        }
        if let Some(ch) = charset.get((idx - 1) as usize) {
            out.push(*ch);
        }
    }
    out
}

pub fn argmax_token_ids(logits: &[Vec<f32>]) -> Result<Vec<i64>> {
    let mut ids = Vec::with_capacity(logits.len());
    for row in logits {
        if row.is_empty() {
            bail!("logits row must not be empty");
        }
        let mut bi = 0usize;
        let mut bv = row[0];
        for (i, &v) in row.iter().enumerate().skip(1) {
            if v > bv {
                bv = v;
                bi = i;
            }
        }
        ids.push(bi as i64);
    }
    Ok(ids)
}

pub fn predict_text_from_logits(logits: &[Vec<f32>], charset: &[char]) -> Result<String> {
    Ok(decode_indices(&argmax_token_ids(logits)?, charset))
}

pub fn sanitize_recognized_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev: Option<char> = None;
    let mut run = 0usize;
    let mut last_was_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !last_was_space && !out.is_empty() {
                out.push(' ');
                last_was_space = true;
            }
            prev = None;
            run = 0;
            continue;
        }
        last_was_space = false;
        if Some(ch) == prev {
            run += 1;
        } else {
            prev = Some(ch);
            run = 1;
        }

        let too_many_digits = ch.is_ascii_digit() && run > 4;
        let too_many_symbols = !ch.is_alphanumeric() && run > 3;
        let too_many_same_char = run > 8;
        if too_many_digits || too_many_symbols || too_many_same_char {
            continue;
        }
        out.push(ch);
    }
    let out = out.trim().to_string();
    let out = collapse_repeated_chunks(&out, 2, 2, 10);
    let out = collapse_repeated_phrases(&out, &["という。", "このように, "], 1);
    let out = remove_unmatched_closing_brackets(&out);
    apply_common_japanese_ocr_replacements(&out)
}

pub fn predict_text_from_flat_logits(
    logits: &[f32],
    timesteps: usize,
    classes: usize,
    charset: &[char],
) -> Result<String> {
    Ok(predict_text_from_flat_logits_with_confidence(logits, timesteps, classes, charset)?.text)
}

pub fn predict_text_from_flat_logits_with_confidence(
    logits: &[f32],
    timesteps: usize,
    classes: usize,
    charset: &[char],
) -> Result<RecognizeResult> {
    if timesteps == 0 || classes == 0 {
        bail!("timesteps/classes must be > 0");
    }
    let expected = timesteps
        .checked_mul(classes)
        .ok_or_else(|| anyhow::anyhow!("logits shape overflow"))?;
    if logits.len() < expected {
        bail!("flat logits length is shorter than shape");
    }
    let mut text = String::new();
    let mut conf_sum = 0.0f32;
    let mut conf_count = 0usize;
    let min_tokens_before_low_conf_stop = 2usize;
    let low_confidence_threshold = 0.35f32;
    for t in 0..timesteps {
        let row = &logits[(t * classes)..((t + 1) * classes)];
        let mut bi = 0usize;
        let mut bv = row[0];
        for (i, &v) in row.iter().enumerate().skip(1) {
            if v > bv {
                bv = v;
                bi = i;
            }
        }
        let mut exp_sum = 0.0f32;
        for &v in row {
            exp_sum += (v - bv).exp();
        }
        let pmax = if exp_sum > 0.0 { 1.0 / exp_sum } else { 0.0 };

        if bi == 0 {
            break;
        }
        if conf_count >= min_tokens_before_low_conf_stop && pmax < low_confidence_threshold {
            break;
        }
        if let Some(ch) = charset.get(bi - 1) {
            text.push(*ch);
            conf_sum += pmax;
            conf_count += 1;
        }
    }
    let mean_confidence = if conf_count > 0 {
        conf_sum / conf_count as f32
    } else {
        0.0
    };
    Ok(RecognizeResult {
        text,
        mean_confidence,
    })
}

pub fn load_charset_from_yaml_str(yaml_body: &str) -> Result<Vec<char>> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml_body)?;
    let charset_value = value
        .get("model")
        .and_then(|v| v.get("charset_train"))
        .ok_or_else(|| anyhow::anyhow!("model.charset_train is missing"))?;
    if let Some(s) = charset_value.as_str() {
        return Ok(s.chars().collect());
    }
    if let Some(seq) = charset_value.as_sequence() {
        let mut out = Vec::new();
        for item in seq {
            let Some(s) = item.as_str() else {
                bail!("model.charset_train sequence item must be string")
            };
            out.extend(s.chars());
        }
        return Ok(out);
    }
    bail!("model.charset_train must be string or sequence of strings")
}

pub fn smoke_recognize(model_path: &Path, image_path: &Path, charset_path: &Path) -> Result<()> {
    if !image_path.is_file() || !model_path.is_file() || !charset_path.is_file() {
        bail!("file not found");
    }
    let yaml_body = fs::read_to_string(charset_path)
        .with_context(|| format!("failed to read {}", charset_path.display()))?;
    let _charset = load_charset_from_yaml_str(&yaml_body)?;
    let img = io::load_rgb_u8(image_path)?;
    let _ = preprocess_rgb_u8(&img.data, img.width, img.height, 384, 32)?;
    smoke_recognize_impl(model_path)
}

pub fn recognize_image(
    model_path: &Path,
    image_path: &Path,
    charset_path: &Path,
) -> Result<String> {
    if !image_path.is_file() || !model_path.is_file() || !charset_path.is_file() {
        bail!("file not found");
    }
    let yaml_body = fs::read_to_string(charset_path)
        .with_context(|| format!("failed to read {}", charset_path.display()))?;
    let charset = load_charset_from_yaml_str(&yaml_body)?;
    let img = io::load_rgb_u8(image_path)?;
    Ok(recognize_image_impl(model_path, &img.data, img.width, img.height, &charset)?.text)
}

pub fn recognize_rgb_u8(
    model_path: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset_path: &Path,
) -> Result<String> {
    if !model_path.is_file() || !charset_path.is_file() {
        bail!("file not found");
    }
    let yaml_body = fs::read_to_string(charset_path)
        .with_context(|| format!("failed to read {}", charset_path.display()))?;
    let charset = load_charset_from_yaml_str(&yaml_body)?;
    Ok(recognize_image_impl(model_path, rgb, width, height, &charset)?.text)
}

pub fn recognize_rgb_u8_with_score(
    model_path: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset_path: &Path,
) -> Result<RecognizeResult> {
    if !model_path.is_file() || !charset_path.is_file() {
        bail!("file not found");
    }
    let yaml_body = fs::read_to_string(charset_path)
        .with_context(|| format!("failed to read {}", charset_path.display()))?;
    let charset = load_charset_from_yaml_str(&yaml_body)?;
    recognize_image_impl(model_path, rgb, width, height, &charset)
}

#[cfg(feature = "onnx")]
fn smoke_recognize_impl(model_path: &Path) -> Result<()> {
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
fn recognize_image_impl(
    model_path: &Path,
    rgb: &[u8],
    width: usize,
    height: usize,
    charset: &[char],
) -> Result<RecognizeResult> {
    use crate::infer::ort_init::OrtAnyhow;
    use anyhow::anyhow;
    use ndarray::Array4;
    use ort::inputs;
    use ort::session::Session;
    use ort::session::builder::GraphOptimizationLevel;
    use ort::value::TensorRef;

    crate::infer::ort_init::ensure_init();
    let mut session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(model_path)
        .anyort()?;

    let inputs_meta = session.inputs();
    let input_meta = inputs_meta
        .first()
        .ok_or_else(|| anyhow!("model has no input"))?;
    let shape_meta = input_meta
        .dtype()
        .tensor_shape()
        .ok_or_else(|| anyhow!("input is not a tensor"))?;
    let dims: &[i64] = shape_meta;
    if dims.len() < 4 {
        bail!("unexpected model input rank");
    }
    if dims[2] <= 0 || dims[3] <= 0 {
        bail!("dynamic input H/W is unsupported");
    }
    let input_h = dims[2] as usize;
    let input_w = dims[3] as usize;
    let input_name = input_meta.name().to_string();
    let outputs_meta = session.outputs();
    let output_name = outputs_meta
        .first()
        .ok_or_else(|| anyhow!("model has no output"))?
        .name()
        .to_string();
    let _ = outputs_meta;
    let _ = inputs_meta;

    let tensor = preprocess_rgb_u8(rgb, width, height, input_w, input_h)?;
    let input_array: Array4<f32> = Array4::from_shape_vec((1, 3, input_h, input_w), tensor)?;
    let tref = TensorRef::from_array_view(input_array.view()).anyort()?;
    let outputs = session.run(inputs![input_name.as_str() => tref]).anyort()?;
    let (shape, data) = outputs[output_name.as_str()]
        .try_extract_tensor::<f32>()
        .anyort()?;
    let shape: Vec<i64> = shape.to_vec();
    let mut result = match shape.as_slice() {
        [1, t, c] => {
            predict_text_from_flat_logits_with_confidence(data, *t as usize, *c as usize, charset)
        }
        [t, c] => {
            predict_text_from_flat_logits_with_confidence(data, *t as usize, *c as usize, charset)
        }
        _ => bail!("unsupported output shape: {:?}", shape),
    }?;
    result.text = sanitize_recognized_text(&result.text);
    Ok(result)
}

#[cfg(not(feature = "onnx"))]
fn smoke_recognize_impl(_model_path: &Path) -> Result<()> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

#[cfg(not(feature = "onnx"))]
fn recognize_image_impl(
    _model_path: &Path,
    _rgb: &[u8],
    _width: usize,
    _height: usize,
    _charset: &[char],
) -> Result<RecognizeResult> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

fn collapse_repeated_chunks(
    input: &str,
    max_repeat: usize,
    min_chunk_len: usize,
    max_chunk_len: usize,
) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        let mut collapsed = false;
        for chunk_len in (min_chunk_len..=max_chunk_len).rev() {
            if i + chunk_len * (max_repeat + 1) > chars.len() {
                continue;
            }
            let first = &chars[i..i + chunk_len];
            let mut repeat_count = 1usize;
            while i + chunk_len * (repeat_count + 1) <= chars.len() {
                let start = i + chunk_len * repeat_count;
                let end = start + chunk_len;
                if &chars[start..end] == first {
                    repeat_count += 1;
                } else {
                    break;
                }
            }
            if repeat_count > max_repeat {
                for _ in 0..max_repeat {
                    for ch in first {
                        out.push(*ch);
                    }
                }
                i += chunk_len * repeat_count;
                collapsed = true;
                break;
            }
        }
        if !collapsed {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn remove_unmatched_closing_brackets(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut stack: Vec<char> = Vec::new();

    for ch in input.chars() {
        match ch {
            '(' | '（' | '「' | '『' | '【' => {
                stack.push(ch);
                out.push(ch);
            }
            ')' => {
                if pop_if_matches(&mut stack, '(') {
                    out.push(ch);
                }
            }
            '）' => {
                if pop_if_matches(&mut stack, '（') {
                    out.push(ch);
                }
            }
            '」' => {
                if pop_if_matches(&mut stack, '「') {
                    out.push(ch);
                }
            }
            '』' => {
                if pop_if_matches(&mut stack, '『') {
                    out.push(ch);
                }
            }
            '】' => {
                if pop_if_matches(&mut stack, '【') {
                    out.push(ch);
                }
            }
            _ => out.push(ch),
        }
    }
    out
}

fn pop_if_matches(stack: &mut Vec<char>, open: char) -> bool {
    if let Some(&last) = stack.last()
        && last == open
    {
        stack.pop();
        return true;
    }
    false
}

fn collapse_repeated_phrases(input: &str, phrases: &[&str], max_repeat: usize) -> String {
    let mut out = input.to_string();
    for &phrase in phrases {
        if phrase.is_empty() {
            continue;
        }
        let repeated = phrase.repeat(max_repeat + 1);
        let kept = phrase.repeat(max_repeat);
        while out.contains(&repeated) {
            out = out.replace(&repeated, &kept);
        }
    }
    out
}

fn apply_common_japanese_ocr_replacements(input: &str) -> String {
    let mut out = input.to_string();
    let replacements = [("調示", "開示"), ("圖示", "開示"), ("単又は乙", "甲又は乙")];
    for (from, to) in replacements {
        out = out.replace(from, to);
    }
    out
}
