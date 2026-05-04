use ndlocr_lite_rs::output::json::build_ocr_json;
use ndlocr_lite_rs::output::text::build_text;
use ndlocr_lite_rs::output::xml::build_ocr_xml;
use ndlocr_lite_rs::pipeline::connect::RecognizedLine;

#[test]
fn output_builders_match_regression_fixtures() {
    let lines = vec![
        RecognizedLine {
            bbox_xyxy: [10, 20, 30, 40],
            text: "abc".into(),
            confidence: 0.9,
            is_vertical: true,
        },
        RecognizedLine {
            bbox_xyxy: [1, 2, 3, 4],
            text: "<x&\"'>".into(),
            confidence: 0.8,
            is_vertical: false,
        },
    ];

    let actual_json =
        serde_json::to_string_pretty(&build_ocr_json(&lines, 100, 200, "input.jpg", "input.jpg"))
            .unwrap();
    let actual_xml = build_ocr_xml(&lines, 100, 200, "input.jpg");
    let actual_txt = build_text(&lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());

    let expected_json = include_str!("fixtures/regression/expected_output.json");
    let expected_xml = include_str!("fixtures/regression/expected_output.xml");
    let expected_txt = include_str!("fixtures/regression/expected_output.txt");

    assert_eq!(actual_json, normalize_newlines(expected_json).trim_end());
    assert_eq!(
        actual_xml.trim_end(),
        normalize_newlines(expected_xml).trim_end()
    );
    assert_eq!(actual_txt, normalize_newlines(expected_txt).trim_end());
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
