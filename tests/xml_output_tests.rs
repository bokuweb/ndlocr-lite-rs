use ndlocr_lite_rs::output::xml::{build_ocr_xml, save_ocr_xml};
use ndlocr_lite_rs::pipeline::connect::RecognizedLine;
use tempfile::tempdir;

#[test]
fn build_ocr_xml_contains_page_and_lines() {
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

    let xml = build_ocr_xml(&lines, 100, 200, "input.jpg");
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>"));
    assert!(xml.contains("<OCRDATASET xmlns=\"\">"));
    assert!(xml.contains("<PAGE IMAGENAME=\"input.jpg\" WIDTH=\"100\" HEIGHT=\"200\">"));
    assert!(xml.contains("TYPE=\"line_main\""));
    assert!(xml.contains("CONF=\"0.900\""));
    assert!(xml.contains("STRING=\"abc\""));
}

#[test]
fn save_ocr_xml_writes_file() {
    let lines = vec![RecognizedLine {
        bbox_xyxy: [1, 2, 3, 4],
        text: "x".into(),
        confidence: 0.5,
        is_vertical: true,
    }];
    let xml = build_ocr_xml(&lines, 10, 20, "a.jpg");
    let dir = tempdir().unwrap();
    let p = dir.path().join("out.xml");
    save_ocr_xml(&xml, &p).unwrap();
    let body = std::fs::read_to_string(&p).unwrap();
    assert!(body.contains("<OCRDATASET xmlns=\"\">"));
    assert!(body.contains("<LINE"));
}

#[test]
fn build_ocr_xml_escapes_special_chars_in_text() {
    let lines = vec![RecognizedLine {
        bbox_xyxy: [1, 2, 3, 4],
        text: "<tag attr='x'>&\"".into(),
        confidence: 0.5,
        is_vertical: true,
    }];
    let xml = build_ocr_xml(&lines, 10, 20, "a.jpg");
    assert!(xml.contains("STRING=\"&lt;tag attr=&apos;x&apos;&gt;&amp;&quot;\""));
}
