use super::*;

const MODELS: [&str; 3] = [
    "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
    "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
    "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
];

#[test]
#[ignore = "diagnostic; requires fixed NDLOCR_DETACHED_NATIVE_MANIFEST and model label"]
fn verify_changed_and_boundary_cases_through_public_apis() {
    let Ok(path) = std::env::var("NDLOCR_DETACHED_NATIVE_MANIFEST") else {
        return;
    };
    let label = std::env::var("NDLOCR_DETACHED_MODEL").unwrap();
    let index = ["30", "50", "100"]
        .iter()
        .position(|&x| x == label)
        .unwrap();
    let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let output = std::env::var("NDLOCR_DETACHED_NATIVE_OUTPUT").unwrap();
    let model = Path::new("models").join(MODELS[index]);
    let charset = Path::new("models/NDLmoji.yaml");
    let session = ParseqSession::load(&model, charset).unwrap();
    let pool = ParseqPool::load(&model, charset, 1).unwrap();
    let mut records = Vec::new();
    for x in m["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["model"] == label)
    {
        let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
        let before = rgb.clone();
        let (w, h) = (
            x["width"].as_u64().unwrap() as usize,
            x["height"].as_u64().unwrap() as usize,
        );
        let orientation = if x["orientation"] == "horizontal" {
            ParseqOrientation::Horizontal
        } else {
            ParseqOrientation::Auto
        };
        let raw = session
            .recognize_rgb_u8_candidates_with_orientation(&rgb, w, h, orientation, 1)
            .unwrap()
            .recognition;
        assert_eq!(
            raw.text,
            x["raw"].as_str().unwrap(),
            "raw {label}/{}",
            x["id"]
        );
        assert_eq!(
            raw.mean_confidence.to_bits() as u64,
            x["mean_bits"].as_u64().unwrap(),
            "raw mean {label}/{}",
            x["id"]
        );
        let actual = if orientation == ParseqOrientation::Horizontal {
            pool.recognize_horizontal_rgb_u8(&rgb, w, h).unwrap()
        } else {
            session.recognize_rgb_u8(&rgb, w, h).unwrap()
        };
        assert_eq!(
            actual.text,
            x["expected_after"].as_str().unwrap(),
            "actual {label}/{}",
            x["id"]
        );
        assert_eq!(
            actual.mean_confidence.to_bits(),
            raw.mean_confidence.to_bits()
        );
        assert_eq!(rgb, before);
        records.push(serde_json::json!({"corpus":x["corpus"],"model":label,"id":x["id"],"raw":raw.text,"text":actual.text,"mean_bits":actual.mean_confidence.to_bits(),"selected":x["selected"]}));
    }
    assert_eq!(records.len() as u64, m["counts"][&label].as_u64().unwrap());
    assert!(!records.is_empty());
    std::fs::write(output, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
    println!(
        "{label}: {} actual API pairs match raw, product policy and confidence bits",
        records.len()
    );
}

#[test]
#[ignore = "requires pinned PARSeq models"]
fn detached_parts_keep_the_model_closer_in_single_line_apis() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/detached_body_components/info_round40.png",
    ))
    .unwrap();
    let (rgb, w, h) = (&image.data, image.width, image.height);
    let untouched = rgb.clone();
    let input = [(rgb.as_slice(), w, h)];
    let charset = Path::new("models/NDLmoji.yaml");
    for file in MODELS {
        let model = Path::new("models").join(file);
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 2).unwrap();
        let fragments = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
        let raw = session.recognize_rgb_u8_unprocessed(rgb, w, h).unwrap();
        assert_eq!(
            raw.text, "情報)",
            "model itself already reads the closer: {file}"
        );
        let processed = session.recognize_rgb_u8(rgb, w, h).unwrap();
        assert_eq!(
            processed.text, "情報)",
            "preserve the supported model closer: {file}"
        );
        assert_eq!(
            processed.mean_confidence.to_bits(),
            raw.mean_confidence.to_bits()
        );
        for actual in [
            pool.recognize_rgb_u8(rgb, w, h).unwrap(),
            fragments.recognize_rgb_u8(rgb, w, h).unwrap(),
            session.recognize_batch_rgb_u8(&input).unwrap().remove(0),
            pool.recognize_batch_rgb_u8(&input).unwrap().remove(0),
            pool.recognize_batch_single_session_rgb_u8(&input)
                .unwrap()
                .remove(0),
            pool.recognize_batch_single_session_horizontal_rgb_u8(&input)
                .unwrap()
                .remove(0),
            crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, w, h, charset).unwrap(),
        ] {
            assert_eq!(actual.text, raw.text, "{file}");
            assert_eq!(
                actual.mean_confidence.to_bits(),
                raw.mean_confidence.to_bits()
            );
        }
        let candidates = session
            .recognize_rgb_u8_candidates_with_orientation(rgb, w, h, ParseqOrientation::Auto, 2)
            .unwrap();
        assert_eq!(candidates.recognition.text, raw.text);
        assert_eq!(
            candidates.recognition.mean_confidence.to_bits(),
            raw.mean_confidence.to_bits()
        );
        assert!(session.recognize_rgb_u8(&[], w, h).is_err());
        assert_eq!(session.recognize_rgb_u8(rgb, w, h).unwrap().text, raw.text);
        assert_eq!(*rgb, untouched);
    }
    let cascade = ParseqCascadePool::load(
        &Path::new("models").join(MODELS[0]),
        &Path::new("models").join(MODELS[1]),
        &Path::new("models").join(MODELS[2]),
        charset,
        1,
    )
    .unwrap();
    let items: Vec<_> = [3.0, 2.0, 1.0]
        .into_iter()
        .map(|bucket| (rgb.as_slice(), w, h, Some(bucket)))
        .collect();
    assert_eq!(
        cascade
            .recognize_batch_with_buckets_rgb_u8(&items)
            .unwrap()
            .iter()
            .map(|x| x.text.as_str())
            .collect::<Vec<_>>(),
        ["情報)"; 3]
    );
}
