use ndlocr_lite_rs::io::load_rgb_u8;
use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};
use std::path::PathBuf;

fn scanned0_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("eval")
        .join("images")
        .join("scaned0.png")
}

#[test]
fn scanned0_fixture_can_be_loaded_and_segmented() {
    let fixture = scanned0_fixture_path();
    assert!(fixture.is_file(), "fixture missing: {}", fixture.display());

    let img = load_rgb_u8(&fixture).expect("failed to load scaned0.png");
    assert!(img.width > 100);
    assert!(img.height > 100);

    let threshold = 220u8;
    let naive = detect_textline_bands_naive(&img.data, img.width, img.height, threshold);
    let fast = detect_textline_bands_fast(&img.data, img.width, img.height, threshold);

    assert_eq!(fast, naive, "naive and fast segmentation should match");
    assert!(
        !naive.is_empty(),
        "scanned fixture should contain text lines"
    );

    for bbox in &naive {
        let [x0, y0, x1, y1] = *bbox;
        assert!(x0 < x1 && y0 < y1, "invalid bbox: {:?}", bbox);
        assert!(
            x1 <= img.width && y1 <= img.height,
            "bbox out of bounds: {:?}",
            bbox
        );
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires ONNX model + charset (env or default local files)"]
fn scanned0_real_ocr_contains_japanese_phrase() {
    use ndlocr_lite_rs::cli::{DEFAULT_CHARSET_PATH, DEFAULT_RECOGNIZE_MODEL_PATH};
    use ndlocr_lite_rs::infer::parseq;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};

    let model = std::env::var("NDLOCR_TEST_PARSEQ_MODEL")
        .unwrap_or_else(|_| DEFAULT_RECOGNIZE_MODEL_PATH.to_string());
    let charset =
        std::env::var("NDLOCR_TEST_CHARSET").unwrap_or_else(|_| DEFAULT_CHARSET_PATH.to_string());
    let model_path = PathBuf::from(model);
    let charset_path = PathBuf::from(charset);
    if !model_path.is_file() || !charset_path.is_file() {
        eprintln!(
            "skip: set NDLOCR_TEST_PARSEQ_MODEL / NDLOCR_TEST_CHARSET or place defaults under models/"
        );
        return;
    }

    let img = load_rgb_u8(&scanned0_fixture_path()).expect("failed to load scaned0.png");
    let boxes = detect_textline_bands_naive(&img.data, img.width, img.height, 220);
    assert!(
        !boxes.is_empty(),
        "no line boxes detected for scanned fixture"
    );

    let mut lines = Vec::new();
    for [x0, y0, x1, y1] in boxes {
        let crop =
            crop_rgb_u8(&img.data, img.width, img.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let line = parseq::recognize_rgb_u8(
            &model_path,
            &crop.data,
            crop.width,
            crop.height,
            &charset_path,
        )
        .unwrap_or_default();
        if !line.trim().is_empty() {
            lines.push(line);
        }
    }
    let joined = lines.join("\n");
    assert!(
        joined.contains("第3条"),
        "expected Japanese phrase was not found in OCR output:\n{}",
        joined
    );
    assert!(
        joined.contains("対馬"),
        "expected Japanese word was not found in OCR output:\n{}",
        joined
    );
}
