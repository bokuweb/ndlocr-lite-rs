use ndlocr_lite_rs::infer::{deim, parseq};
use ndlocr_lite_rs::io::load_rgb_u8;
use std::fs;
use tempfile::tempdir;

#[test]
fn load_rgb_u8_reads_image_dimensions_and_data() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("tiny.png");
    let img = image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 0, 255, 0]).unwrap();
    img.save(&p).unwrap();
    let loaded = load_rgb_u8(&p).unwrap();
    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
}

#[test]
fn smoke_recognize_returns_feature_error_when_onnx_is_disabled() {
    let dir = tempdir().unwrap();
    let model = dir.path().join("m.onnx");
    let imgp = dir.path().join("a.png");
    let ch = dir.path().join("c.yaml");
    fs::write(&model, b"x").unwrap();
    fs::write(&ch, "model:\n  charset_train: ['a']\n").unwrap();
    image::RgbImage::from_raw(1, 1, vec![255, 255, 255])
        .unwrap()
        .save(&imgp)
        .unwrap();
    let err = parseq::smoke_recognize(&model, &imgp, &ch).unwrap_err();
    assert!(err.to_string().contains("onnx feature is disabled"));
}

#[test]
fn smoke_detect_returns_feature_error_when_onnx_is_disabled() {
    let dir = tempdir().unwrap();
    let model = dir.path().join("m.onnx");
    let imgp = dir.path().join("a.png");
    fs::write(&model, b"x").unwrap();
    image::RgbImage::from_raw(1, 1, vec![255, 255, 255])
        .unwrap()
        .save(&imgp)
        .unwrap();
    let err = deim::smoke_detect(&model, &imgp).unwrap_err();
    assert!(err.to_string().contains("onnx feature is disabled"));
}
