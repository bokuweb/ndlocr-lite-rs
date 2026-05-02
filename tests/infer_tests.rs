#[cfg(feature = "onnx")]
use ndlocr_lite_rs::infer::cached::default_parseq_parallelism;
use ndlocr_lite_rs::infer::{deim, parseq};
use proptest::prelude::*;
#[cfg(not(feature = "onnx"))]
use tempfile::tempdir;

#[test]
fn deim_scale_boxes_scales_to_original_image_space() {
    let input_boxes = vec![[100.0, 200.0, 300.0, 400.0]];
    let scaled = deim::scale_boxes_to_image_space(&input_boxes, 2000, 1000, 1000, 1000);
    assert_eq!(scaled, vec![[200, 200, 600, 400]]);
}

#[test]
fn parseq_decode_stops_at_eos() {
    let charset: Vec<char> = vec!['あ', 'い', 'う'];
    assert_eq!(parseq::decode_indices(&[1, 2, 0, 3], &charset), "あい");
}

#[cfg(feature = "onnx")]
#[test]
fn default_parseq_parallelism_is_small_and_nonzero() {
    let p = default_parseq_parallelism();
    assert!((1..=4).contains(&p));
}

proptest! {
    #[test]
    fn parseq_decode_is_unchanged_by_suffix_after_eos(
        prefix in proptest::collection::vec(1i64..5, 0..8),
        suffix in proptest::collection::vec(1i64..5, 0..8),
    ) {
        let charset: Vec<char> = vec!['a', 'b', 'c', 'd'];
        let mut with_eos = prefix.clone();
        with_eos.push(0);
        with_eos.extend_from_slice(&suffix);
        let expected = parseq::decode_indices(&[prefix, vec![0]].concat(), &charset);
        let actual = parseq::decode_indices(&with_eos, &charset);
        prop_assert_eq!(actual, expected);
    }
}

#[cfg(not(feature = "onnx"))]
#[test]
fn smoke_detect_reports_feature_disabled_message() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let model = dir.path().join("model.onnx");
    let img = image::RgbImage::from_raw(1, 1, vec![255, 0, 0]).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&model, b"dummy").unwrap();

    let err = deim::smoke_detect(&model, &image).expect_err("onnx disabled should fail");
    assert!(err.to_string().contains("onnx feature is disabled"));
    assert!(err.to_string().contains("--features onnx"));
}

#[cfg(not(feature = "onnx"))]
#[test]
fn smoke_recognize_reports_feature_disabled_message() {
    let dir = tempdir().unwrap();
    let image = dir.path().join("input.png");
    let model = dir.path().join("model.onnx");
    let charset = dir.path().join("charset.yaml");
    let img = image::RgbImage::from_raw(1, 1, vec![0, 255, 0]).unwrap();
    img.save(&image).unwrap();
    std::fs::write(&model, b"dummy").unwrap();
    std::fs::write(&charset, "model:\n  charset_train: \"abc\"\n").unwrap();

    let err =
        parseq::smoke_recognize(&model, &image, &charset).expect_err("onnx disabled should fail");
    assert!(err.to_string().contains("onnx feature is disabled"));
    assert!(err.to_string().contains("--features onnx"));
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires ONNX model/image fixtures via env vars"]
fn smoke_detect_with_onnx_feature_can_load_model() {
    let Some(model) = std::env::var_os("NDLOCR_TEST_DEIM_MODEL") else {
        eprintln!("skip: set NDLOCR_TEST_DEIM_MODEL and NDLOCR_TEST_IMAGE to run");
        return;
    };
    let Some(image) = std::env::var_os("NDLOCR_TEST_IMAGE") else {
        eprintln!("skip: set NDLOCR_TEST_DEIM_MODEL and NDLOCR_TEST_IMAGE to run");
        return;
    };
    deim::smoke_detect(std::path::Path::new(&model), std::path::Path::new(&image)).unwrap();
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires ONNX model/image/charset fixtures via env vars"]
fn smoke_recognize_with_onnx_feature_can_load_model_and_charset() {
    let Some(model) = std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL") else {
        eprintln!(
            "skip: set NDLOCR_TEST_PARSEQ_MODEL NDLOCR_TEST_IMAGE NDLOCR_TEST_CHARSET to run"
        );
        return;
    };
    let Some(image) = std::env::var_os("NDLOCR_TEST_IMAGE") else {
        eprintln!(
            "skip: set NDLOCR_TEST_PARSEQ_MODEL NDLOCR_TEST_IMAGE NDLOCR_TEST_CHARSET to run"
        );
        return;
    };
    let Some(charset) = std::env::var_os("NDLOCR_TEST_CHARSET") else {
        eprintln!(
            "skip: set NDLOCR_TEST_PARSEQ_MODEL NDLOCR_TEST_IMAGE NDLOCR_TEST_CHARSET to run"
        );
        return;
    };
    parseq::smoke_recognize(
        std::path::Path::new(&model),
        std::path::Path::new(&image),
        std::path::Path::new(&charset),
    )
    .unwrap();
}
