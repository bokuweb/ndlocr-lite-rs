//! 固定した入力のPageRecognizer出力を、供給モデルと二つの設定で記録する診断。
#[cfg(any(test, feature = "onnx"))]
#[path = "support/page_matrix_manifest.rs"]
mod page_matrix_manifest;

#[cfg(not(feature = "onnx"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("onnx feature is disabled. Rebuild with --features onnx.")
}

#[cfg(feature = "onnx")]
fn main() -> anyhow::Result<()> {
    use anyhow::ensure;
    use clap::Parser;
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    use std::path::PathBuf;

    #[derive(Parser)]
    struct Args {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        det_model: PathBuf,
        #[arg(long)]
        parseq_model: PathBuf,
        #[arg(long)]
        charset: PathBuf,
        #[arg(long)]
        label: String,
        /// 新規ディレクトリ。過去の推論結果は上書きしない。
        #[arg(long)]
        output_dir: PathBuf,
    }

    let args = Args::parse();
    let inputs = page_matrix_manifest::parse_manifest(&std::fs::read(&args.manifest)?)?;
    ensure!(
        page_matrix_manifest::safe_id(&args.label),
        "unsafe model label"
    );
    if let Some(parent) = args
        .output_dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir(&args.output_dir)?;
    let runtime = PageRecognizer::load(&args.det_model, &args.parseq_model, &args.charset, 1)?;
    for (suffix, options) in [
        (
            "eval",
            PageRecognizerOptions {
                det_conf_threshold: 0.25,
                min_line_confidence: 0.3,
                drop_empty_text: true,
            },
        ),
        ("default", PageRecognizerOptions::default()),
    ] {
        let label = format!("{}_{suffix}", args.label);
        let dir = args.output_dir.join(&label);
        std::fs::create_dir(&dir)?;
        let mut records = Vec::new();
        for input in &inputs {
            let image = load_rgb_u8(&input.image)?;
            let page =
                runtime.recognize_page_rgb_u8(&image.data, image.width, image.height, options)?;
            let text = page
                .lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(dir.join(format!("{}.txt", input.id)), &text)?;
            records.push(serde_json::json!({
                "id":input.id,"text":text,"page_index":page.page_index,
                "lines":page.lines.iter().map(|line|serde_json::json!({
                    "bbox":line.bbox_xyxy,"text":line.text,"confidence":line.confidence,
                })).collect::<Vec<_>>(),
            }));
        }
        std::fs::write(
            dir.join("results.json"),
            serde_json::to_vec_pretty(&records)?,
        )?;
        println!("{label}: {} pages", records.len());
    }
    Ok(())
}
