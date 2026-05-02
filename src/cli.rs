use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub const DEFAULT_DETECT_MODEL_PATH: &str = "models/deim-s-1024x1024.onnx";
pub const DEFAULT_RECOGNIZE_MODEL_PATH: &str =
    "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx";
pub const DEFAULT_RECOGNIZE_MODEL30_PATH: &str =
    "models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx";
pub const DEFAULT_RECOGNIZE_MODEL50_PATH: &str =
    "models/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx";
pub const DEFAULT_CHARSET_PATH: &str = "ndlocr/src/config/NDLmoji.yaml";

#[derive(Debug, Parser)]
#[command(name = "ndlocr-lite-rs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Detect(DetectArgs),
    Recognize(RecognizeArgs),
    RecognizePage(RecognizePageArgs),
    MockPage(MockPageArgs),
}

#[derive(Debug, Args)]
pub struct DetectArgs {
    #[arg(long, default_value = DEFAULT_DETECT_MODEL_PATH)]
    pub model: PathBuf,
    #[arg(long)]
    pub image: PathBuf,
}

#[derive(Debug, Args)]
pub struct RecognizeArgs {
    #[arg(long, default_value = DEFAULT_RECOGNIZE_MODEL_PATH)]
    pub model: PathBuf,
    #[arg(long)]
    pub image: PathBuf,
    #[arg(long, default_value = DEFAULT_CHARSET_PATH)]
    pub charset: PathBuf,
}

#[derive(Debug, Args)]
pub struct RecognizePageArgs {
    #[arg(long, default_value = DEFAULT_DETECT_MODEL_PATH)]
    pub det_model: PathBuf,
    #[arg(long, default_value_t = 0.25)]
    pub det_conf_threshold: f32,
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub use_deim_detection: bool,
    #[arg(long, default_value = DEFAULT_RECOGNIZE_MODEL_PATH)]
    pub model: PathBuf,
    #[arg(long)]
    pub image: PathBuf,
    #[arg(long, default_value = DEFAULT_CHARSET_PATH)]
    pub charset: PathBuf,
    #[arg(
        long,
        default_value_t = 0,
        help = "各行 crop の前に bbox を四方向に広げるピクセル数（検出が文字ぎりぎりのときの欠け軽減用）"
    )]
    pub line_crop_padding: u32,
    #[arg(long, default_value_t = 220)]
    pub binarize_threshold: u8,
    #[arg(long, default_value_t = 0.30)]
    pub min_line_confidence: f32,
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub enable_cascade: bool,
    #[arg(long, default_value = DEFAULT_RECOGNIZE_MODEL30_PATH)]
    pub model30: PathBuf,
    #[arg(long, default_value = DEFAULT_RECOGNIZE_MODEL50_PATH)]
    pub model50: PathBuf,
    #[arg(long, default_value_t = 25)]
    pub cascade_threshold_30_to_50: usize,
    #[arg(long, default_value_t = 45)]
    pub cascade_threshold_50_to_100: usize,
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub split_long_lines: bool,
    #[arg(long, default_value_t = 80)]
    pub split_long_line_char_threshold: usize,
    #[arg(long, default_value = "false", value_parser = clap::value_parser!(bool))]
    pub quality_boost: bool,
    #[arg(long, default_value_t = 0.03)]
    pub quality_boost_min_delta: f32,
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub structure_rules: bool,
    #[arg(long)]
    pub rule_pack: Option<PathBuf>,
    #[arg(long)]
    pub output_txt: Option<PathBuf>,
    #[arg(long)]
    pub output_docx: Option<PathBuf>,
    #[arg(long)]
    pub post_dict: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MockPageArgs {
    #[arg(long)]
    pub image: PathBuf,
    #[arg(long)]
    pub output_dir: PathBuf,
    #[arg(long)]
    pub output_stem: Option<String>,
    #[arg(long)]
    pub line_text: Option<String>,
    #[arg(long)]
    pub line_count: Option<usize>,
    #[arg(long)]
    pub line_confidence: Option<f32>,
    #[arg(long, value_enum)]
    pub line_orientation: Option<LineOrientation>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LineOrientation {
    Vertical,
    Horizontal,
}
