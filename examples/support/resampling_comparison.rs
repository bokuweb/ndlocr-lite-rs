//! 固定座標から取り出した同一RGBを同一sessionで比較する。

use anyhow::{Context, Result, ensure};
use clap::Parser;
use ndlocr_lite_rs::{
    infer::{
        ort_init::{OrtAnyhow, ensure_init},
        parseq::{
            ParseqOrientation, load_charset_from_yaml_str,
            predict_text_from_flat_logits_with_confidence, preprocess_rgb_u8_with_orientation,
        },
    },
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    },
};
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashSet, io::Write, path::PathBuf, time::Instant};

/// 比較する固定ページ。正解などの未知フィールドを受け付けない。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Page {
    /// ページの一意なID。
    id: String,
    /// 元ページの画像パス。
    image: PathBuf,
    /// 全評価帯の固定座標。
    lines: Vec<Line>,
}

/// 正解を含まない物理帯の指定。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Line {
    /// ページ内で一意な帯ID。
    id: String,
    /// 左上と右下の半開区間座標。
    bounds: [usize; 4],
}

/// 空・曖昧なID・不正な座標をモデルロード前に拒否する。
fn parse_manifest(bytes: &[u8]) -> Result<Vec<Page>> {
    let pages: Vec<Page> = serde_json::from_slice(bytes)?;
    ensure!(!pages.is_empty(), "empty manifest");
    let mut ids = HashSet::new();
    for page in &pages {
        ensure!(
            !page.id.is_empty() && ids.insert(&page.id),
            "duplicate or empty page id"
        );
        ensure!(!page.lines.is_empty(), "empty page lines");
        let mut lines = HashSet::new();
        for line in &page.lines {
            ensure!(
                !line.id.is_empty() && lines.insert(&line.id),
                "duplicate or empty line id"
            );
            let [x0, y0, x1, y1] = line.bounds;
            ensure!(x0 < x1 && y0 < y1, "invalid bounds");
        }
    }
    Ok(pages)
}

/// 同じモデルと文字集合での比較条件。
#[derive(Parser)]
struct Args {
    /// truthを含まないページ・帯のJSON配列。
    #[arg(long)]
    manifest: PathBuf,
    /// 固定したPARSeqモデル。
    #[arg(long)]
    model: PathBuf,
    /// モデルに対応する文字集合。
    #[arg(long)]
    charset: PathBuf,
    /// 新規作成する全結果JSON。
    #[arg(long)]
    output: PathBuf,
}

/// CPU単一threadの同一sessionで順序を交互にして全chunkを推論する。
pub fn run_with(
    candidate_name: &str,
    mut candidate: impl FnMut(&[u8], usize, usize, usize, usize) -> Result<Vec<f32>>,
) -> Result<()> {
    ensure!(
        candidate_name != "bilinear" && !candidate_name.is_empty(),
        "invalid candidate name"
    );
    let args = Args::parse();
    let pages = parse_manifest(&std::fs::read(&args.manifest)?)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args.output)?;
    let charset = load_charset_from_yaml_str(&std::fs::read_to_string(&args.charset)?)?;
    ensure_init();
    let mut session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(&args.model)
        .anyort()?;
    let input = session.inputs().first().context("no input")?;
    let shape = input.dtype().tensor_shape().context("not a tensor")?;
    ensure!(
        shape.len() == 4 && shape[1] == 3 && shape[2] > 0 && shape[3] > 0,
        "unsupported shape"
    );
    let (iw, ih) = (usize::try_from(shape[3])?, usize::try_from(shape[2])?);
    let input_name = input.name().to_owned();
    let output_name = session
        .outputs()
        .first()
        .context("no output")?
        .name()
        .to_owned();
    let mut results = Vec::new();
    let mut counter = 0;
    for page in pages {
        let mut image = load_rgb_u8(&page.image)?;
        if let std::borrow::Cow::Owned(rgb) =
            normalize_chromatic_panels(&image.data, image.width, image.height)
                .map_err(anyhow::Error::msg)?
        {
            image.data = rgb;
        }
        let mut lines = Vec::new();
        for line in page.lines {
            let [x0, y0, x1, y1] = line.bounds;
            ensure!(
                x1 <= image.width && y1 <= image.height,
                "out of image bounds"
            );
            let band = crop_rgb_u8(
                &image.data,
                image.width,
                image.height,
                BBox::new(x0, y0, x1, y1),
            )?;
            let chunks = split_line_for_input(&band.data, band.width, band.height, 220, iw, ih)
                .map_err(anyhow::Error::msg)?;
            let mut text = [String::new(), String::new()];
            let mut records = Vec::new();
            for (index, chunk) in chunks.iter().enumerate() {
                let crop =
                    crop_line_chunk_rgb_u8(&band.data, band.width, band.height, &chunks, index)?;
                let mut outputs = [Value::Null, Value::Null];
                for method in [counter % 2, (counter + 1) % 2] {
                    let started = Instant::now();
                    let tensor = if method == 0 {
                        preprocess_rgb_u8_with_orientation(
                            &crop.data,
                            crop.width,
                            crop.height,
                            iw,
                            ih,
                            ParseqOrientation::Horizontal,
                        )
                    } else {
                        candidate(&crop.data, crop.width, crop.height, iw, ih)
                    };
                    let prep_seconds = started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    let result = tensor.and_then(|data| {
                        let array = ndarray::Array::from_shape_vec((1, 3, ih, iw), data)?;
                        let tensor = TensorRef::from_array_view(array.view()).anyort()?;
                        let outputs = session
                            .run(ort::inputs![input_name.as_str() => tensor])
                            .anyort()?;
                        let (shape, logits) = outputs[output_name.as_str()]
                            .try_extract_tensor::<f32>()
                            .anyort()?;
                        ensure!(
                            shape.len() == 3 && shape[0] == 1 && shape[1] > 0 && shape[2] > 0,
                            "unexpected output shape"
                        );
                        predict_text_from_flat_logits_with_confidence(
                            logits,
                            usize::try_from(shape[1])?,
                            usize::try_from(shape[2])?,
                            &charset,
                        )
                    });
                    let seconds = started.elapsed().as_secs_f64();
                    let mut inserted_space = false;
                    if let Ok(result) = &result {
                        inserted_space =
                            chunk.insert_space_before(&text[method], &result.text, band.height);
                        if inserted_space {
                            text[method].push(' ');
                        }
                        text[method].push_str(&result.text);
                    }
                    outputs[method] = json!({"text":result.as_ref().ok().map(|r| &r.text),
                        "confidence":result.as_ref().ok().map(|r| r.mean_confidence),
                        "error":result.err().map(|e| e.to_string()), "inserted_space_before":inserted_space,
                        "preprocess_seconds":prep_seconds, "inference_decode_seconds":seconds});
                }
                records.push(json!({"x_range":[chunk.x0,chunk.x1], "dimensions":[crop.width,crop.height],
                    "bilinear":outputs[0], (candidate_name):outputs[1], "first":if counter % 2 == 0 {"bilinear"} else {candidate_name}}));
                counter += 1;
            }
            lines.push(json!({"id":line.id,"bounds":line.bounds,"bilinear":text[0],(candidate_name):text[1],"chunks":records}));
        }
        eprintln!("completed {}", page.id);
        results.push(json!({"id":page.id,"lines":lines}));
    }
    file.write_all(&serde_json::to_vec_pretty(
        &json!({"model_dimensions":[iw,ih],"pages":results}),
    )?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// truth付き入力と空・重複・逆転座標を推論より先に拒否する。
    #[test]
    fn manifest_rejects_ambiguous_or_oracle_inputs() {
        for value in [
            json!([]),
            json!([{"id":"p", "image":"x", "lines":[], "truth":"answer"}]),
            json!([{"id":"p", "image":"x", "lines":[{"id":"a","bounds":[1,0,0,1]}]}]),
            json!([{"id":"p", "image":"x", "lines":[{"id":"a","bounds":[0,0,1,1]}, {"id":"a","bounds":[0,1,1,2]}]}]),
            json!([{"id":"p", "image":"x", "lines":[]}, {"id":"p", "image":"y", "lines":[]}]),
        ] {
            assert!(super::parse_manifest(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        assert!(
            super::parse_manifest(
                br#"[{"id":"p","image":"x","lines":[{"id":"a","bounds":[0,0,3,2]}]}]"#
            )
            .is_ok()
        );
    }
}
