use ndlocr_lite_rs::output::artifacts::save_page_artifacts;
use ndlocr_lite_rs::pipeline::connect::RecognizedLine;
use tempfile::tempdir;

#[test]
fn save_page_artifacts_writes_json_xml_and_txt() {
    let lines = vec![RecognizedLine {
        bbox_xyxy: [1, 2, 3, 4],
        text: "abc".to_string(),
        confidence: 0.9,
        is_vertical: true,
    }];
    let texts = vec!["abc".to_string()];
    let dir = tempdir().unwrap();
    let json = dir.path().join("out.json");
    let xml = dir.path().join("out.xml");
    let txt = dir.path().join("out.txt");

    save_page_artifacts(
        &lines, &texts, 100, 200, "img.jpg", "img.jpg", &json, &xml, &txt,
    )
    .unwrap();

    let jb = std::fs::read_to_string(&json).unwrap();
    let xb = std::fs::read_to_string(&xml).unwrap();
    let tb = std::fs::read_to_string(&txt).unwrap();
    assert!(jb.contains("\"imginfo\""));
    assert!(xb.contains("<OCRDATASET xmlns=\"\">"));
    assert_eq!(tb, "abc");
}
