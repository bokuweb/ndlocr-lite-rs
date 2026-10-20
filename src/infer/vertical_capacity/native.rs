use crate::infer::cached::{ParseqPool, ParseqSession};
use std::path::Path;

#[test]
#[ignore = "requires pinned PARSeq30 model; preserves the fixed raw selector decisions"]
fn fragment_auto_recovers_supported_columns_and_preserves_rejected_candidates() {
    let model = Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx");
    let charset = Path::new("models/NDLmoji.yaml");
    let session = ParseqSession::load(model, charset).unwrap();
    let single = ParseqPool::load(model, charset, 1).unwrap();
    let fragment = ParseqPool::load_for_fragments(model, charset, 1).unwrap();
    let cases: serde_json::Value = serde_json::from_slice(
        &std::fs::read("tests/fixtures/vertical_capacity/cases.json").unwrap(),
    )
    .unwrap();
    for x in cases.as_array().unwrap() {
        let image = crate::io::load_rgb_u8(Path::new(x["image"].as_str().unwrap())).unwrap();
        let original = image.data.clone();
        let raw = session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(
            raw.text,
            x["raw"].as_str().unwrap(),
            "original decoder/{}",
            x["id"]
        );
        let normal = single
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(normal.text, raw.text, "normal single line/{}", x["id"]);
        assert_eq!(
            normal.mean_confidence.to_bits(),
            raw.mean_confidence.to_bits()
        );
        let actual = fragment
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(
            actual.text,
            x["expected"].as_str().unwrap(),
            "fragment Auto/{}",
            x["id"]
        );
        if !x["recovers"].as_bool().unwrap() {
            assert_eq!(
                actual.mean_confidence.to_bits(),
                raw.mean_confidence.to_bits()
            );
        }
        assert_eq!(image.data, original);
    }
}
