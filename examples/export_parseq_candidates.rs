//! 固定した行座標から、通常の水平区間分割と生候補を評価用JSONに出力する。
#[cfg(feature = "onnx")]
#[path = "support/candidate_export.rs"]
mod candidate_export;

/// onnxなしでは推論を始めず、必要なfeatureを示す。
#[cfg(not(feature = "onnx"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("rebuild with --features onnx")
}

/// 文脈補正なしで一回ずつ推論し、候補を保存する。
#[cfg(feature = "onnx")]
fn main() -> anyhow::Result<()> {
    use anyhow::ensure;
    use candidate_export::ExportLine;
    use clap::Parser;
    use ndlocr_lite_rs::{
        infer::{cached::ParseqSession, parseq::ParseqOrientation},
        io::load_rgb_u8,
        pipeline::{
            background::normalize_chromatic_panels,
            crop::{BBox, crop_rgb_u8},
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
        },
    };
    use serde::Deserialize;
    use std::{collections::HashSet, path::PathBuf};

    /// 評価用入力。truthフィールドは読み取らない。
    #[derive(Deserialize)]
    struct Line {
        /// 固定した行ID。
        id: String,
        /// 元画像の行座標。
        bounds: Option<[usize; 4]>,
    }

    /// 候補の抽出条件。
    #[derive(Parser)]
    struct Args {
        /// 入力ページ画像。
        #[arg(long)]
        image: PathBuf,
        /// PARSeqモデル。
        #[arg(long)]
        model: PathBuf,
        /// モデルの文字集合。
        #[arg(long)]
        charset: PathBuf,
        /// 行ID・座標のJSON配列。
        #[arg(long)]
        manifest: PathBuf,
        /// exportするJSONファイル。
        #[arg(long)]
        output: PathBuf,
        /// EOSを含む取得候補数。
        #[arg(long, default_value_t = 10)]
        top_k: usize,
    }

    let args = Args::parse();
    ensure!(args.top_k > 0, "top-k must be positive");
    let lines: Vec<Line> = serde_json::from_slice(&std::fs::read(args.manifest)?)?;
    let mut ids = HashSet::new();
    ensure!(
        !lines.is_empty()
            && lines
                .iter()
                .all(|line| !line.id.is_empty() && ids.insert(&line.id)),
        "manifest ids must be nonempty and unique"
    );
    let mut image = load_rgb_u8(&args.image)?;
    if let std::borrow::Cow::Owned(prepared) =
        normalize_chromatic_panels(&image.data, image.width, image.height)
            .map_err(anyhow::Error::msg)?
    {
        image.data = prepared;
    }
    let session = ParseqSession::load(&args.model, &args.charset)?;
    let (input_width, input_height) = session.input_dimensions();
    let mut exported = Vec::new();
    for line in lines {
        let Some(bounds) = line.bounds else {
            exported.push(ExportLine::missing(line.id));
            continue;
        };
        let [x0, y0, x1, y1] = bounds;
        let band = crop_rgb_u8(
            &image.data,
            image.width,
            image.height,
            BBox::new(x0, y0, x1, y1),
        )?;
        let chunks = split_line_for_input(
            &band.data,
            band.width,
            band.height,
            220,
            input_width,
            input_height,
        )
        .map_err(anyhow::Error::msg)?;
        let mut output = ExportLine::new(line.id, bounds);
        for (index, chunk) in chunks.iter().enumerate() {
            let crop = crop_line_chunk_rgb_u8(&band.data, band.width, band.height, &chunks, index)?;
            let result = session.recognize_rgb_u8_candidates_with_orientation(
                &crop.data,
                crop.width,
                crop.height,
                ParseqOrientation::Horizontal,
                args.top_k,
            )?;
            let space =
                chunk.insert_space_before(&output.text, &result.recognition.text, band.height);
            output.append(result, space, [chunk.x0, chunk.x1])?;
        }
        exported.push(output);
    }
    std::fs::write(args.output, serde_json::to_vec_pretty(&exported)?)?;
    Ok(())
}
