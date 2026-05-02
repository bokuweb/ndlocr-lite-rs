use ndlocr_lite_rs::output::json::build_ocr_json;
use ndlocr_lite_rs::pipeline::connect::RecognizedLine;
use tempfile::tempdir;

#[test]
fn build_ocr_json_matches_expected_shape() {
    let lines = vec![
        RecognizedLine {
            bbox_xyxy: [10, 20, 30, 40],
            text: "abc".into(),
            confidence: 0.9,
            is_vertical: true,
        },
        RecognizedLine {
            bbox_xyxy: [1, 2, 3, 4],
            text: "def".into(),
            confidence: 0.8,
            is_vertical: false,
        },
    ];
    let out = build_ocr_json(&lines, 100, 200, "input.jpg", "input.jpg");
    assert_eq!(out.contents.len(), 1);
    assert_eq!(out.contents[0][0].id, 0);
    assert_eq!(
        out.contents[0][0].bounding_box,
        [[10, 20], [10, 40], [30, 20], [30, 40]]
    );
    assert_eq!(out.imginfo.img_width, 100);
}

#[test]
fn save_ocr_json_writes_file() {
    let lines = vec![RecognizedLine {
        bbox_xyxy: [1, 2, 3, 4],
        text: "x".into(),
        confidence: 0.5,
        is_vertical: true,
    }];
    let out = build_ocr_json(&lines, 10, 20, "a.jpg", "a.jpg");
    let dir = tempdir().unwrap();
    let p = dir.path().join("out.json");
    ndlocr_lite_rs::output::json::save_ocr_json(&out, &p).unwrap();
    let body = std::fs::read_to_string(&p).unwrap();
    assert!(body.contains("\"contents\""));
    assert!(body.contains("\"imginfo\""));
}
