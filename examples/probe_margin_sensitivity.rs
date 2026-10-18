//! 固定chunkの純白余白だけを変え、前処理tensorと生認識を比較する。
use anyhow::{Result, ensure};
use ndlocr_lite_rs::{
    infer::{
        cached::ParseqSession,
        parseq::{ParseqOrientation, preprocess_rgb_u8_with_orientation},
    },
    io::load_rgb_u8,
    pipeline::horizontal_margins::compact_horizontal_margins,
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Instant};

/// 全高の右端へ純白列だけを追加する。
fn pad_right(rgb: &[u8], width: usize, height: usize, extra: usize) -> Result<(Vec<u8>, usize)> {
    ensure!(width > 0 && height > 0, "empty image");
    let row = width
        .checked_mul(3)
        .ok_or_else(|| anyhow::anyhow!("row overflow"))?;
    ensure!(
        row.checked_mul(height) == Some(rgb.len()),
        "invalid RGB length"
    );
    let output_width = width
        .checked_add(extra)
        .ok_or_else(|| anyhow::anyhow!("width overflow"))?;
    let output_row = output_width
        .checked_mul(3)
        .ok_or_else(|| anyhow::anyhow!("row overflow"))?;
    let size = output_row
        .checked_mul(height)
        .ok_or_else(|| anyhow::anyhow!("size overflow"))?;
    let mut output = Vec::new();
    output.try_reserve_exact(size)?;
    output.resize(size, 255);
    for (source, target) in rgb
        .chunks_exact(row)
        .zip(output.chunks_exact_mut(output_row))
    {
        target[..row].copy_from_slice(source);
    }
    Ok((output, output_width))
}

/// 元tensorとの差を全要素で集計する。補間後の字形保持とは区別する。
fn tensor_difference(original: &[f32], changed: &[f32]) -> Value {
    assert_eq!(original.len(), changed.len());
    let mut count = 0;
    let mut max = 0.0_f64;
    let mut squares = 0.0_f64;
    for (&left, &right) in original.iter().zip(changed) {
        let difference = f64::from(left) - f64::from(right);
        count += usize::from(left != right);
        max = max.max(difference.abs());
        squares += difference * difference;
    }
    json!({"different_elements": count, "elements": original.len(), "max_absolute": max,
        "rms": (squares / original.len() as f64).sqrt()})
}

/// source manifestで固定した6入力×10変種を2回認識し、全結果を保存する。
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    ensure!(
        args.len() == 5,
        "usage: probe_margin_sensitivity MODEL CHARSET FIXTURES OUTPUT"
    );
    let (model, charset, fixtures, output) = (
        PathBuf::from(&args[1]),
        PathBuf::from(&args[2]),
        PathBuf::from(&args[3]),
        PathBuf::from(&args[4]),
    );
    let sources: Value = serde_json::from_slice(&std::fs::read(fixtures.join("source.json"))?)?;
    let load_start = Instant::now();
    let session = ParseqSession::load(&model, &charset)?;
    let load_seconds = load_start.elapsed().as_secs_f64();
    let (input_width, input_height) = session.input_dimensions();
    let mut rows = Vec::new();
    for case in sources["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let original = load_rgb_u8(&fixtures.join(format!("{name}.png")))?;
        let compact = compact_horizontal_margins(
            &original.data,
            original.width,
            original.height,
            original.height / 4,
        )?;
        let tensor = |rgb: &[u8], width| {
            preprocess_rgb_u8_with_orientation(
                rgb,
                width,
                original.height,
                input_width,
                input_height,
                ParseqOrientation::Horizontal,
            )
        };
        let original_tensor = tensor(&original.data, original.width)?;
        let mut variants = Vec::new();
        for (kind, rgb, width) in [
            ("original", &original.data, original.width),
            ("compact", &compact.image.data, compact.image.width),
        ] {
            for extra in 0..=4 {
                let (padded, padded_width) = pad_right(rgb, width, original.height, extra)?;
                let prepared = tensor(&padded, padded_width)?;
                let mut repetitions = Vec::new();
                for _ in 0..2 {
                    let start = Instant::now();
                    let result = session.recognize_rgb_u8_unprocessed_with_orientation(
                        &padded,
                        padded_width,
                        original.height,
                        ParseqOrientation::Horizontal,
                    )?;
                    repetitions.push(json!({"text": result.text, "confidence": result.mean_confidence, "seconds": start.elapsed().as_secs_f64()}));
                }
                ensure!(
                    repetitions[0]["text"] == repetitions[1]["text"]
                        && repetitions[0]["confidence"] == repetitions[1]["confidence"],
                    "nondeterministic output: {name}/{kind}/{extra}"
                );
                variants.push(json!({"kind": kind, "extra_white_columns": extra, "dimensions": [padded_width, original.height], "tensor_difference_from_original": tensor_difference(&original_tensor, &prepared), "repetitions": repetitions}));
            }
        }
        rows.push(json!({"name": name, "truth": case["truth"], "variants": variants}));
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"model_dimensions": [input_width, input_height], "load_seconds": load_seconds, "cases": rows}),
        )?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 有彩色・非白近白も全高で保持し、右端だけ白列を追加する。
    #[test]
    fn padding_preserves_every_source_pixel() {
        let rgb = [1, 2, 3, 254, 255, 255, 7, 8, 9, 10, 11, 12];
        let (out, width) = pad_right(&rgb, 2, 2, 2).unwrap();
        assert_eq!(width, 4);
        assert_eq!(&out[..6], &rgb[..6]);
        assert_eq!(&out[12..18], &rgb[6..]);
        assert_eq!(&out[6..12], &[255; 6]);
        assert_eq!(&out[18..], &[255; 6]);
        assert_eq!(pad_right(&rgb, 2, 2, 0).unwrap(), (rgb.to_vec(), 2));
    }
    /// 不正寸法・長さ・加算/積overflowを確保前に拒否する。
    #[test]
    fn padding_rejects_invalid_sizes() {
        for (rgb, w, h, extra) in [
            (&[][..], 0, 1, 1),
            (&[][..], 1, 0, 1),
            (&[0][..], 1, 1, 1),
            (&[1; 3][..], 1, 1, usize::MAX),
            (&[][..], usize::MAX, 2, 0),
        ] {
            assert!(pad_right(rgb, w, h, extra).is_err());
        }
    }
    /// 出力tensorの差を既知の独立計算と照合する。
    #[test]
    fn tensor_delta_counts_changed_elements() {
        let result = tensor_difference(&[-1.0, 0.0, 1.0, 0.0], &[-1.0, 1.0, -1.0, 0.0]);
        assert_eq!(result["different_elements"], 2);
        assert_eq!(result["max_absolute"], 2.0);
        assert_eq!(result["rms"], (5.0_f64 / 4.0).sqrt());
    }
}
