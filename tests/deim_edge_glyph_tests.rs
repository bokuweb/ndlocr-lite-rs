#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    app::prepare_line_crops,
    infer::{cached::ParseqSession, deim_cached::DeimSession},
    io::load_rgb_u8,
};

/// 検出/crop/生モデルを保存して、最初の薄い字形の欠落を切り分ける。
#[test]
#[ignore = "diagnostic; requires pinned DEIM/all PARSeq assets"]
fn first_line_keeps_the_leading_horizontal_glyph() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/ndl_accessibility_p2.png",
    ))
    .unwrap();
    let deim = DeimSession::load(std::path::Path::new(
        &std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap(),
    ))
    .unwrap();
    let raw = deim
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    let crops = prepare_line_crops(&image.data, image.width, image.height, raw.clone(), 0).unwrap();
    let first = crops
        .iter()
        .filter(|c| c.bbox_xyxy[2] - c.bbox_xyxy[0] > 800)
        .min_by_key(|c| c.bbox_xyxy[1])
        .unwrap();
    let mut models = Vec::new();
    for name in [
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let session = ParseqSession::load(
            &std::path::Path::new("models").join(name),
            std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
        )
        .unwrap();
        let recognized = session
            .recognize_rgb_u8_unprocessed(&first.crop.data, first.crop.width, first.crop.height)
            .unwrap();
        eprintln!(
            "MODEL {name} BOUNDS {:?} RAW {:?}",
            first.bbox_xyxy, recognized.text
        );
        models.push(serde_json::json!({"model":name,"text":recognized.text,"confidence":recognized.mean_confidence}));
    }
    if let Some(directory) = std::env::var_os("NDLOCR_EDGE_GLYPH_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        image::save_buffer(
            directory.join("first-crop.png"),
            &first.crop.data,
            first.crop.width as u32,
            first.crop.height as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
        let detections:Vec<_>=raw.iter().map(|d|serde_json::json!({"bounds":d.box_xyxy,"class":d.class_name,"confidence":d.confidence,"pred_char_count":d.pred_char_count})).collect();
        std::fs::write(directory.join("diagnosis.json"),serde_json::to_vec_pretty(&serde_json::json!({"raw":detections,"first_bounds":first.bbox_xyxy,"models":models})).unwrap()).unwrap();
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
        .args([
            "recognize-page",
            "--image",
            "tests/fixtures/eval/images/ndl_accessibility_p2.png",
            "--charset",
            "models/NDLmoji.yaml",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("一部から成る"), "{text}");
}

/// ページruntimeも同じcropを使い、固定100文字モデルで先頭字形を保持する。
#[test]
#[ignore = "requires pinned DEIM and 100-character PARSeq assets"]
fn page_runtime_uses_the_completed_leading_glyph() {
    use ndlocr_lite_rs::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/ndl_accessibility_p2.png",
    ))
    .unwrap();
    let runtime = PageRecognizer::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap()),
        std::path::Path::new(
            "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
        1,
    )
    .unwrap();
    let page = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions {
                det_conf_threshold: 0.25,
                min_line_confidence: 0.0,
                drop_empty_text: false,
            },
        )
        .unwrap();
    let first = page
        .lines
        .iter()
        .find(|line| line.bbox_xyxy == [30, 16, 1054, 40])
        .unwrap();
    assert!(first.text.starts_with("一部から成る"), "{}", first.text);
}
