use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use ndlocr_lite_rs::app::run_cli;
use ndlocr_lite_rs::cli::{Cli, Command, LineOrientation, MockPageArgs};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

#[derive(Debug, Clone, ValueEnum)]
enum OrientationArg {
    Vertical,
    Horizontal,
}

#[derive(Debug, Parser)]
#[command(name = "pdf_to_image_mock_page")]
struct Args {
    #[arg(long)]
    pdf: PathBuf,
    #[arg(long)]
    output_dir: PathBuf,
    #[arg(long)]
    output_stem: Option<String>,
    #[arg(long, default_value = "mock100")]
    line_text: String,
    #[arg(long, default_value_t = 1)]
    line_count: usize,
    #[arg(long, default_value_t = 1.0)]
    line_confidence: f32,
    #[arg(long, value_enum)]
    line_orientation: Option<OrientationArg>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.pdf.is_file() {
        bail!("pdf file not found: {}", args.pdf.display());
    }

    std::fs::create_dir_all(&args.output_dir)?;
    let stem = args.output_stem.clone().unwrap_or_else(|| {
        args.pdf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string()
    });
    let image_path = args.output_dir.join(format!("{stem}.png"));
    convert_pdf_to_png(&args.pdf, &image_path)?;

    let cli = Cli {
        command: Command::MockPage(MockPageArgs {
            image: image_path,
            output_dir: args.output_dir,
            output_stem: Some(stem),
            line_text: Some(args.line_text),
            line_count: Some(args.line_count),
            line_confidence: Some(args.line_confidence),
            line_orientation: args.line_orientation.map(|o| match o {
                OrientationArg::Vertical => LineOrientation::Vertical,
                OrientationArg::Horizontal => LineOrientation::Horizontal,
            }),
        }),
    };
    run_cli(cli)
}

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
