use clap::Parser;
use ndlocr_lite_rs::cli::{
    Cli, Command, DEFAULT_CHARSET_PATH, DEFAULT_DETECT_MODEL_PATH, DEFAULT_RECOGNIZE_MODEL_PATH,
    DEFAULT_RECOGNIZE_MODEL30_PATH, DEFAULT_RECOGNIZE_MODEL50_PATH,
};

#[test]
fn parse_detect_command() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "detect",
        "--model",
        "m.onnx",
        "--image",
        "a.jpg",
    ]);
    match cli.command {
        Command::Detect(args) => {
            assert_eq!(args.model.to_string_lossy(), "m.onnx");
            assert_eq!(args.image.to_string_lossy(), "a.jpg");
        }
        _ => panic!("detect expected"),
    }
}

#[test]
fn parse_recognize_command() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize",
        "--model",
        "m.onnx",
        "--image",
        "a.jpg",
        "--charset",
        "c.yaml",
    ]);
    match cli.command {
        Command::Recognize(args) => {
            assert_eq!(args.charset.to_string_lossy(), "c.yaml");
        }
        _ => panic!("recognize expected"),
    }
}

#[test]
fn parse_detect_command_uses_default_model_path() {
    let cli = Cli::parse_from(["ndlocr-lite-rs", "detect", "--image", "a.jpg"]);
    match cli.command {
        Command::Detect(args) => {
            assert_eq!(args.model.to_string_lossy(), DEFAULT_DETECT_MODEL_PATH);
            assert_eq!(args.image.to_string_lossy(), "a.jpg");
        }
        _ => panic!("detect expected"),
    }
}

#[test]
fn parse_recognize_command_uses_default_model_and_charset_paths() {
    let cli = Cli::parse_from(["ndlocr-lite-rs", "recognize", "--image", "a.jpg"]);
    match cli.command {
        Command::Recognize(args) => {
            assert_eq!(args.model.to_string_lossy(), DEFAULT_RECOGNIZE_MODEL_PATH);
            assert_eq!(args.charset.to_string_lossy(), DEFAULT_CHARSET_PATH);
            assert_eq!(args.image.to_string_lossy(), "a.jpg");
        }
        _ => panic!("recognize expected"),
    }
}

#[test]
fn parse_recognize_page_command_uses_defaults_and_optional_output() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "a.jpg",
        "--output-txt",
        "out/result.txt",
        "--output-docx",
        "out/result.docx",
        "--post-dict",
        "config/post_dict.yaml",
    ]);
    match cli.command {
        Command::RecognizePage(args) => {
            assert_eq!(args.det_model.to_string_lossy(), DEFAULT_DETECT_MODEL_PATH);
            assert!((args.det_conf_threshold - 0.25).abs() < f32::EPSILON);
            assert!(args.use_deim_detection);
            assert_eq!(args.model.to_string_lossy(), DEFAULT_RECOGNIZE_MODEL_PATH);
            assert_eq!(args.charset.to_string_lossy(), DEFAULT_CHARSET_PATH);
            assert_eq!(args.image.to_string_lossy(), "a.jpg");
            assert!((args.min_line_confidence - 0.30).abs() < f32::EPSILON);
            assert!(args.enable_cascade);
            assert_eq!(
                args.model30.to_string_lossy(),
                DEFAULT_RECOGNIZE_MODEL30_PATH
            );
            assert_eq!(
                args.model50.to_string_lossy(),
                DEFAULT_RECOGNIZE_MODEL50_PATH
            );
            assert_eq!(args.cascade_threshold_30_to_50, 25);
            assert_eq!(args.cascade_threshold_50_to_100, 45);
            assert!(args.split_long_lines);
            assert_eq!(args.split_long_line_char_threshold, 80);
            assert!(!args.quality_boost);
            assert!((args.quality_boost_min_delta - 0.03).abs() < f32::EPSILON);
            assert!(args.structure_rules);
            assert_eq!(args.line_crop_padding, 0);
            assert!(args.rule_pack.is_none());
            assert_eq!(args.output_txt.unwrap().to_string_lossy(), "out/result.txt");
            assert_eq!(
                args.output_docx.unwrap().to_string_lossy(),
                "out/result.docx"
            );
            assert_eq!(
                args.post_dict.unwrap().to_string_lossy(),
                "config/post_dict.yaml"
            );
            assert!(args.morph_correct_dict.is_none());
        }
        _ => panic!("recognize-page expected"),
    }
}

#[test]
fn parse_recognize_page_command_keeps_quality_boost_off_with_model_override() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "a.jpg",
        "--model",
        "/tmp/model.onnx",
    ]);
    match cli.command {
        Command::RecognizePage(args) => {
            assert!(!args.quality_boost);
        }
        _ => panic!("expected recognize-page"),
    }
}

#[test]
fn default_parseq_models_track_ndlocr_lite_v1_2_handwriting_models() {
    assert_eq!(
        DEFAULT_RECOGNIZE_MODEL30_PATH,
        "models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"
    );
    assert_eq!(
        DEFAULT_RECOGNIZE_MODEL50_PATH,
        "models/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx"
    );
    assert_eq!(
        DEFAULT_RECOGNIZE_MODEL_PATH,
        "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"
    );
}

#[test]
fn parse_recognize_page_command_accepts_line_crop_padding() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "a.jpg",
        "--line-crop-padding",
        "2",
    ]);
    match cli.command {
        Command::RecognizePage(args) => {
            assert_eq!(args.line_crop_padding, 2);
        }
        _ => panic!("recognize-page expected"),
    }
}

#[test]
fn parse_recognize_page_command_accepts_rule_pack() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "a.jpg",
        "--rule-pack",
        "rules/scaned0.yaml",
    ]);
    match cli.command {
        Command::RecognizePage(args) => {
            assert_eq!(
                args.rule_pack.unwrap().to_string_lossy(),
                "rules/scaned0.yaml"
            );
        }
        _ => panic!("recognize-page expected"),
    }
}

#[test]
fn parse_recognize_page_command_accepts_morph_correct_dict() {
    let cli = Cli::parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        "a.jpg",
        "--morph-correct-dict",
        "dict/system.dic.zst",
    ]);
    match cli.command {
        Command::RecognizePage(args) => {
            assert_eq!(
                args.morph_correct_dict.unwrap().to_string_lossy(),
                "dict/system.dic.zst"
            );
        }
        _ => panic!("recognize-page expected"),
    }
}
