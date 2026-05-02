use anyhow::{Result, bail};
use clap::Parser;
use ndlocr_lite_rs::cli::{DEFAULT_CHARSET_PATH, DEFAULT_RECOGNIZE_MODEL_PATH};
use std::path::PathBuf;

#[cfg(feature = "onnx")]
use anyhow::Context;
#[cfg(feature = "onnx")]
use std::path::Path;
#[cfg(feature = "onnx")]
use std::process::Command as ProcessCommand;

#[derive(Debug, Parser)]
#[command(name = "pdf_to_image_real_ocr")]
struct Args {
    #[arg(long)]
    pdf: PathBuf,
    #[arg(long)]
    output_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_RECOGNIZE_MODEL_PATH)]
    model: PathBuf,
    #[arg(long, default_value = DEFAULT_CHARSET_PATH)]
    charset: PathBuf,
    #[arg(long, default_value_t = 220)]
    binarize_threshold: u8,
}

#[cfg(not(feature = "onnx"))]
fn main() -> Result<()> {
    bail!("onnx feature is disabled. Rebuild with `--features onnx`.");
}

#[cfg(feature = "onnx")]
fn main() -> Result<()> {
    use ndarray::Array4;
    use ndlocr_lite_rs::infer::ort_init::OrtAnyhow;
    use ndlocr_lite_rs::infer::parseq;
    use ndlocr_lite_rs::io;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive;
    use ort::inputs;
    use ort::session::Session;
    use ort::session::builder::GraphOptimizationLevel;
    use ort::value::TensorRef;

    let args = Args::parse();
    if !args.pdf.is_file() {
        bail!("pdf file not found: {}", args.pdf.display());
    }

    std::fs::create_dir_all(&args.output_dir)?;
    let stem = args
        .pdf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let image_path = args.output_dir.join(format!("{stem}.png"));
    let out_txt = args.output_dir.join(format!("{stem}.real_ocr.txt"));
    convert_pdf_to_png(&args.pdf, &image_path)?;
    let img = io::load_rgb_u8(&image_path)?;

    let charset_yaml = std::fs::read_to_string(&args.charset)
        .with_context(|| format!("failed to read {}", args.charset.display()))?;
    let charset = parseq::load_charset_from_yaml_str(&charset_yaml)?;

    ndlocr_lite_rs::infer::ort_init::ensure_init();
    let mut session = Session::builder()
        .anyort()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .anyort()?
        .with_intra_threads(1)
        .anyort()?
        .commit_from_file(&args.model)
        .anyort()?;
    let inputs_meta = session.inputs();
    let input_meta = inputs_meta
        .first()
        .ok_or_else(|| anyhow::anyhow!("model has no input"))?;
    let shape_meta = input_meta
        .dtype()
        .tensor_shape()
        .ok_or_else(|| anyhow::anyhow!("input is not tensor"))?;
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
        .ok_or_else(|| anyhow::anyhow!("model has no output"))?
        .name()
        .to_string();
    let _ = outputs_meta;
    let _ = inputs_meta;

    let boxes =
        detect_textline_bands_naive(&img.data, img.width, img.height, args.binarize_threshold);
    let mut lines = Vec::new();
    for [x0, y0, x1, y1] in boxes {
        let crop = crop_rgb_u8(&img.data, img.width, img.height, BBox::new(x0, y0, x1, y1))?;
        let tensor =
            parseq::preprocess_rgb_u8(&crop.data, crop.width, crop.height, input_w, input_h)?;
        let input_array: Array4<f32> = Array4::from_shape_vec((1, 3, input_h, input_w), tensor)?;
        let tref = TensorRef::from_array_view(input_array.view()).anyort()?;
        let outputs = session.run(inputs![input_name.as_str() => tref]).anyort()?;
        let (shape, data) = outputs[output_name.as_str()]
            .try_extract_tensor::<f32>()
            .anyort()?;
        let shape: Vec<i64> = shape.to_vec();
        let text = match shape.as_slice() {
            [1, t, c] => {
                parseq::predict_text_from_flat_logits(data, *t as usize, *c as usize, &charset)?
            }
            [t, c] => {
                parseq::predict_text_from_flat_logits(data, *t as usize, *c as usize, &charset)?
            }
            _ => bail!("unsupported output shape: {:?}", shape),
        };
        let text = parseq::sanitize_recognized_text(&text);
        if !text.trim().is_empty() {
            lines.push(text);
        }
    }

    std::fs::write(&out_txt, lines.join("\n"))?;
    println!("{}", out_txt.display());
    Ok(())
}

#[cfg(feature = "onnx")]
fn convert_pdf_to_png(pdf: &Path, out_png: &Path) -> Result<()> {
    if cfg!(target_os = "macos") {
        let status = ProcessCommand::new("sips")
            .arg("-s")
            .arg("format")
            .arg("png")
            .arg(pdf)
            .arg("--out")
            .arg(out_png)
            .status()
            .context("failed to start sips")?;
        if !status.success() {
            bail!("sips failed while converting PDF to PNG");
        }
        return Ok(());
    }

    let prefix = out_png.with_extension("");
    let status = ProcessCommand::new("pdftoppm")
        .arg("-png")
        .arg("-singlefile")
        .arg(pdf)
        .arg(&prefix)
        .status()
        .context("failed to start pdftoppm")?;
    if !status.success() {
        bail!("pdftoppm failed while converting PDF to PNG");
    }
    Ok(())
}
