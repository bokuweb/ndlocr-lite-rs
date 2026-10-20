use super::*;

#[test]
#[ignore = "requires pinned PARSeq30 and charset; executed by inference CI"]
fn fragment_prefix_agreement_preserves_raw_and_extends_saturated_columns() {
    let model = std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            "models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx".into()
        });
    let charset = std::env::var_os("NDLOCR_TEST_CHARSET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "models/NDLmoji.yaml".into());
    let raw_session = ParseqSession::load(&model, &charset).unwrap();
    let normal = ParseqPool::load(&model, &charset, 1).unwrap();
    let fragment = ParseqPool::load_for_fragments(&model, &charset, 1).unwrap();
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vertical_prefix_agreement");
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("cases.json")).unwrap()).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 3);
    for case in cases.as_array().unwrap() {
        let image =
            crate::io::load_rgb_u8(&directory.join(case["image"].as_str().unwrap())).unwrap();
        let original = image.data.clone();
        let raw = raw_session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        let single = normal
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        let actual = fragment
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(
            raw.text,
            case["raw"].as_str().unwrap(),
            "raw/{}",
            case["id"]
        );
        assert_eq!(single.text, raw.text, "normal/{}", case["id"]);
        assert_eq!(
            single.mean_confidence.to_bits(),
            raw.mean_confidence.to_bits()
        );
        assert_eq!(
            actual.text,
            case["expected"].as_str().unwrap(),
            "fragment/{}",
            case["id"]
        );
        assert!(
            actual.mean_confidence.is_finite() && (0.0..=1.0).contains(&actual.mean_confidence)
        );
        if case["recovers"] == true {
            assert!(actual.text.starts_with(&raw.text));
            assert!(actual.text.chars().count() > raw.text.chars().count());
        } else {
            assert_eq!(
                actual.mean_confidence.to_bits(),
                raw.mean_confidence.to_bits()
            );
        }
        assert!(fragment.recognize_rgb_u8(&[], 0, 0).is_err());
        let reused = fragment
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(reused.text, actual.text, "reuse/{}", case["id"]);
        assert_eq!(
            reused.mean_confidence.to_bits(),
            actual.mean_confidence.to_bits()
        );
        assert_eq!(image.data, original);
    }
}
