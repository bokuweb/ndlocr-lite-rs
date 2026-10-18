#![cfg(feature = "onnx")]

/// 短い横書きの継続行を、入力・モデル・整序に分離する。
#[test]
#[ignore = "diagnostic; requires pinned DEIM and all PARSeq models"]
fn climate_short_horizontal_continuation_keeps_its_order() {
    use ndlocr_lite_rs::{
        app::prepare_line_crops,
        infer::{cached::ParseqSession, deim_cached::DeimSession, parseq::ParseqOrientation},
        io::load_rgb_u8,
    };
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_climate_2025.png",
    ))
    .unwrap();
    let deim = DeimSession::load(std::path::Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let detections = deim
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    let crops = prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap();
    let mut records = Vec::new();
    for name in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let model = ParseqSession::load(
            &std::path::Path::new("models").join(name),
            std::path::Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        for crop in crops
            .iter()
            .filter(|c| c.bbox_xyxy[1] > 800 && c.crop.width < 150)
        {
            for orientation in [ParseqOrientation::Auto, ParseqOrientation::Horizontal] {
                let raw = model
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &crop.crop.data,
                        crop.crop.width,
                        crop.crop.height,
                        orientation,
                    )
                    .unwrap();
                eprintln!(
                    "{name} {:?} {:?} {:?}",
                    crop.bbox_xyxy, orientation, raw.text
                );
                let expected = if crop.bbox_xyxy == [103, 824, 148, 845] {
                    "進行"
                } else if name.contains("-30-") {
                    "昇上で"
                } else {
                    "で上昇"
                };
                assert_eq!(raw.text, expected);
                records.push(serde_json::json!({"model":name,"bounds":crop.bbox_xyxy,"pred_char_count":crop.pred_char_count,"orientation":format!("{orientation:?}"),"text":raw.text,"confidence":raw.mean_confidence}));
            }
        }
    }
    assert_eq!(
        records.len(),
        12,
        "two short crops, three models, two orientations"
    );
    if let Some(path) = std::env::var_os("NDLOCR_SHORT_LINE_PROBE") {
        std::fs::write(path, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
    }
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
        .args([
            "recognize-page",
            "--image",
            "tests/fixtures/eval/images/jma_climate_2025.png",
            "--charset",
            "models/NDLmoji.yaml",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.lines().any(|s| s == "で上昇"), "{text}");
}

/// 単一モデル設定と欠落poolでは既存の実行契約を維持する。
#[test]
#[ignore = "requires pinned DEIM and all PARSeq models"]
fn short_order_check_respects_cascade_and_available_pools() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("absent50.onnx");
    for (extra, expected) in [
        (vec!["--enable-cascade", "false"], "で上昇"),
        (vec!["--model50", missing.to_str().unwrap()], "昇上で"),
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
            .args([
                "recognize-page",
                "--image",
                "tests/fixtures/eval/images/jma_climate_2025.png",
                "--charset",
                "models/NDLmoji.yaml",
            ])
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let body = String::from_utf8(output.stdout).unwrap();
        assert!(body.lines().any(|line| line == expected), "{body}");
    }
}
