use super::*;
use crate::infer::cached::{ParseqCascadePool, ParseqPool, ParseqSession};
use std::path::Path;
const MODELS: [&str; 3] = [
    "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
    "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
    "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
];
fn fixture(zip_name: &str, file: &str) -> (Vec<u8>, usize, usize) {
    use std::io::Read;
    let mut z = zip::ZipArchive::new(
        std::fs::File::open(format!("tests/fixtures/{zip_name}/corpus.zip")).unwrap(),
    )
    .unwrap();
    let mut bytes = Vec::new();
    z.by_name(&format!("images/{file}"))
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
    (
        image.as_raw().clone(),
        image.width() as usize,
        image.height() as usize,
    )
}
#[test]
#[ignore = "requires pinned PARSeq models"]
fn body_supported_closers_recover_public_single_line_apis_and_keep_negative_cases() {
    let images = [
        fixture("literal_closers", "corner_japanese_suffix_mincho_png.png"),
        fixture(
            "literal_closers",
            "corner_japanese_internal_mincho_jpeg75.jpg",
        ),
        fixture("literal_closers", "round_latin_suffix_mincho_jpeg75.jpg"),
        fixture("literal_closers", "round_double_mincho_jpeg75.jpg"),
        fixture("closer_geometry", "corner_japanese_internal_maru20_png.png"),
        fixture(
            "closer_geometry",
            "full_round_japanese_internal_mincho40_png.png",
        ),
        fixture("closer_geometry", "control_19_maru20_png.png"),
        fixture(
            "literal_closers",
            "lenticular_digits_balanced_mincho_jpeg75.jpg",
        ),
    ];
    let inputs: Vec<_> = images
        .iter()
        .map(|(data, w, h)| (data.as_slice(), *w, *h))
        .collect();
    let charset = Path::new("models/NDLmoji.yaml");
    let mut cascade_expected = Vec::new();
    for (index, file) in MODELS.iter().enumerate() {
        let expected = [
            "資料」",
            "資」料",
            "ABC)",
            "ABC))",
            "情」報",
            if index == 2 { "報告報" } else { "情報" },
            if index == 2 { " 情報" } else { "情報" },
            if index == 1 {
                "【123456"
            } else {
                "【123456】"
            },
        ];
        cascade_expected.extend(expected.iter().map(|s| s.to_string()));
        let model = Path::new("models").join(file);
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 2).unwrap();
        let fragments = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
        for (i, (rgb, w, h)) in inputs.iter().enumerate() {
            let untouched = rgb.to_vec();
            let raw = session.recognize_rgb_u8_unprocessed(rgb, *w, *h).unwrap();
            let actual = session.recognize_rgb_u8(rgb, *w, *h).unwrap();
            assert_eq!(actual.text, expected[i], "{file}/{i}");
            assert_eq!(
                actual.mean_confidence.to_bits(),
                raw.mean_confidence.to_bits()
            );
            assert_eq!(
                pool.recognize_rgb_u8(rgb, *w, *h).unwrap().text,
                expected[i]
            );
            assert_eq!(
                fragments.recognize_rgb_u8(rgb, *w, *h).unwrap().text,
                raw.text
            );
            let candidates = session
                .recognize_rgb_u8_candidates_with_orientation(
                    rgb,
                    *w,
                    *h,
                    ParseqOrientation::Auto,
                    2,
                )
                .unwrap();
            assert_eq!(candidates.recognition.text, raw.text);
            if i < 5 {
                let stateless =
                    crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, *w, *h, charset)
                        .unwrap();
                assert_eq!(stateless.text, expected[i]);
                assert!((stateless.mean_confidence - raw.mean_confidence).abs() < 1e-5);
                assert_eq!(
                    crate::infer::parseq::sanitize_recognized_text(&raw.text),
                    ["資料", "資料", "ABC", "ABC", "情報"][i]
                );
            }
            assert_eq!(*rgb, untouched);
        }
        for actual in [
            session.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_single_session_rgb_u8(&inputs).unwrap(),
        ] {
            assert_eq!(
                actual.iter().map(|x| x.text.as_str()).collect::<Vec<_>>(),
                expected
            );
        }
        assert!(session.recognize_rgb_u8(&[], 2, 2).is_err());
        assert!(pool.recognize_batch_rgb_u8(&[(&[], 2, 2)]).is_err());
        assert_eq!(
            session
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "資料」"
        );
        assert_eq!(
            pool.recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "資料」"
        );
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
        .iter()
        .flat_map(|bucket| {
            inputs
                .iter()
                .map(move |(rgb, w, h)| (*rgb, *w, *h, Some(*bucket)))
        })
        .collect();
    let actual = cascade.recognize_batch_with_buckets_rgb_u8(&items).unwrap();
    assert_eq!(
        actual.iter().map(|x| &x.text).collect::<Vec<_>>(),
        cascade_expected.iter().collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "diagnostic; explicit frozen root and phase required"]
fn capture_body_closer_native_and_cascade_matrix() {
    let Ok(root) = std::env::var("NDLOCR_BODY_CLOSER_ROOT") else {
        return;
    };
    let phase = std::env::var("NDLOCR_BODY_CLOSER_PHASE").unwrap();
    assert!(["before", "after"].contains(&phase.as_str()));
    let root = Path::new(&root);
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let old: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join(if phase == "before" {
            "frozen-probe.json"
        } else {
            "before-records.json"
        }))
        .unwrap(),
    )
    .unwrap();
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1271);
    let mut all = Vec::new();
    let charset = Path::new("models/NDLmoji.yaml");
    for (index, file) in MODELS.iter().enumerate() {
        let label = format!("single{}", [30, 50, 100][index]);
        let session = ParseqSession::load(&Path::new("models").join(file), charset).unwrap();
        let mut rows = Vec::new();
        let mut actuals = Vec::new();
        let mut extra_raw = 0;
        for input in inputs {
            let id = input["id"].as_str().unwrap();
            let image =
                crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
            let existing = old["models"][index]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id);
            let mut record = if let Some(row) = existing {
                row.clone()
            } else {
                assert_eq!(phase, "before");
                extra_raw += 1;
                let out = session
                    .recognize_rgb_u8_candidates_with_orientation(
                        &image.data,
                        image.width,
                        image.height,
                        ParseqOrientation::Auto,
                        2,
                    )
                    .unwrap();
                let raw = &out.recognition.text;
                let baseline = sanitize_visible_single_line(
                    raw,
                    &image.data,
                    image.width,
                    image.height,
                    ParseqOrientation::Auto,
                );
                let fragment = sanitize_visible_repetitions(
                    raw,
                    &image.data,
                    image.width,
                    image.height,
                    ParseqOrientation::Auto,
                );
                let eligible = fragment.chars().count() > baseline.chars().count()
                    && !fragment.chars().any(char::is_whitespace)
                    && image.height as u128 * 5 <= image.width as u128 * 4;
                let supported = super::geometry_probe::supports_frozen_closer_body(
                    &image.data,
                    image.width,
                    image.height,
                    &fragment,
                );
                let predicted = if eligible && supported {
                    fragment.clone()
                } else {
                    baseline.clone()
                };
                serde_json::json!({"id":id,"raw":raw,"baseline":baseline,"closer_body":predicted,"fragment":fragment,"mean_confidence":out.recognition.mean_confidence,"stop_reason":format!("{:?}",out.stop_reason),"steps":out.steps.iter().map(|s|serde_json::json!({"timestep":s.timestep,"range":s.text_range.as_ref().map(|r|[r.start,r.end]),"candidates":s.candidates.iter().map(|c|serde_json::json!({"token_id":c.token_id,"char":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"legacy_count_supported":supports_glyph_count(&image.data,image.width,image.height,fragment.chars().count())})
            };
            let raw = record["raw"].as_str().unwrap();
            let fragment = sanitize_visible_repetitions(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            assert_eq!(fragment, record["fragment"].as_str().unwrap());
            assert_eq!(
                supports_glyph_count(
                    &image.data,
                    image.width,
                    image.height,
                    fragment.chars().count()
                ),
                record["legacy_count_supported"].as_bool().unwrap()
            );
            let actual = session
                .recognize_rgb_u8(&image.data, image.width, image.height)
                .unwrap();
            let expected = record[if phase == "before" {
                "baseline"
            } else {
                "closer_body"
            }]
            .as_str()
            .unwrap();
            assert_eq!(actual.text, expected, "{phase}/{label}/{id}");
            let frozen_conf = record["mean_confidence"].as_f64().unwrap() as f32;
            assert_eq!(
                actual.mean_confidence.to_bits(),
                frozen_conf.to_bits(),
                "confidence {phase}/{label}/{id}"
            );
            let dir = root.join(&phase).join(&label);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{id}.txt")), &actual.text).unwrap();
            if phase == "before" {
                let dir = root.join("predicted").join(&label);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(
                    dir.join(format!("{id}.txt")),
                    record["closer_body"].as_str().unwrap(),
                )
                .unwrap();
                record["truth"] = serde_json::Value::String(
                    std::fs::read_to_string(Path::new(input["truth"].as_str().unwrap())).unwrap(),
                );
                rows.push(record);
            }
            actuals.push(serde_json::json!({"id":id,"text":actual.text,"confidence_bits":actual.mean_confidence.to_bits()}));
        }
        assert_eq!(extra_raw, if phase == "before" { 32 } else { 0 });
        all.push(serde_json::json!({"label":label,"rows":if phase=="before"{serde_json::json!(rows)}else{serde_json::json!([])},"actual":actuals,"extra_raw_forward_count":extra_raw}));
    }
    let boundaries: Vec<_> = inputs
        .iter()
        .filter(|x| x["kind"] == "new_boundary")
        .collect();
    assert_eq!(boundaries.len(), 32);
    let images: Vec<_> = boundaries
        .iter()
        .map(|x| crate::io::load_rgb_u8(Path::new(x["image"].as_str().unwrap())).unwrap())
        .collect();
    let items: Vec<_> = [3.0, 2.0, 1.0]
        .iter()
        .flat_map(|bucket| {
            images
                .iter()
                .map(move |x| (x.data.as_slice(), x.width, x.height, Some(*bucket)))
        })
        .collect();
    let cascade = ParseqCascadePool::load(
        &Path::new("models").join(MODELS[0]),
        &Path::new("models").join(MODELS[1]),
        &Path::new("models").join(MODELS[2]),
        charset,
        1,
    )
    .unwrap();
    let actual = cascade.recognize_batch_with_buckets_rgb_u8(&items).unwrap();
    let mut trace = Vec::new();
    for (at, result) in actual.iter().enumerate() {
        let start = at / 32;
        let id = boundaries[at % 32]["id"].as_str().unwrap();
        let native = |index: usize| {
            all[index]["actual"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap()["text"]
                .as_str()
                .unwrap()
        };
        let mut chosen = start;
        if chosen == 0 && native(0).chars().count() >= 25 {
            chosen = 1;
        }
        if chosen == 1 && native(1).chars().count() >= 45 {
            chosen = 2;
        }
        assert_eq!(result.text, native(chosen), "cascade {phase}/{start}/{id}");
        let label = format!("cascade{}", [3, 2, 1][start]);
        let dir = root.join(&phase).join(&label);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.txt")), &result.text).unwrap();
        trace.push(serde_json::json!({"id":id,"mode":label,"selected_model":([30,50,100][chosen]),"text":result.text,"confidence_bits":result.mean_confidence.to_bits()}));
    }
    if phase == "before" {
        std::fs::write(
            root.join("before-records.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
        )
        .unwrap();
    }
    std::fs::write(root.join(format!("{phase}-api.json")),serde_json::to_vec_pretty(&serde_json::json!({"models":all,"cascade":trace,"native_calls":3813,"cascade_inputs":96,"fragment_and_legacy_gate_unchanged":true})).unwrap()).unwrap();
}

#[test]
#[ignore = "diagnostic; replays actual API choices without loading models"]
fn replay_body_closer_recovery_from_frozen_pixels() {
    let Ok(root) = std::env::var("NDLOCR_BODY_CLOSER_REPLAY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("before-records.json");
    let before = read("before-api.json");
    let after = read("after-api.json");
    let mut count = 0;
    let mut reasons = std::collections::BTreeMap::<&str, usize>::new();
    let mut boundary_boxes = Vec::new();
    for (index, model) in frozen["models"].as_array().unwrap().iter().enumerate() {
        for row in model["rows"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let input = m["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap();
            let file = Path::new(input["image"].as_str().unwrap())
                .file_name()
                .unwrap();
            let image = crate::io::load_rgb_u8(&root.join("inputs").join(file)).unwrap();
            let raw = row["raw"].as_str().unwrap();
            let fragment = sanitize_visible_repetitions(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            assert_eq!(fragment, row["fragment"].as_str().unwrap());
            assert_eq!(
                supports_glyph_count(
                    &image.data,
                    image.width,
                    image.height,
                    fragment.chars().count()
                ),
                row["legacy_count_supported"].as_bool().unwrap()
            );
            let reference = super::geometry_probe::supports_frozen_closer_body(
                &image.data,
                image.width,
                image.height,
                &fragment,
            );
            assert_eq!(
                supports_closer_body(&image.data, image.width, image.height, &fragment),
                reference
            );
            let actual = sanitize_visible_single_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            assert_eq!(actual, row["closer_body"].as_str().unwrap());
            let b = before["models"][index]["actual"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap();
            let a = after["models"][index]["actual"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap();
            assert_eq!(b["text"], row["baseline"]);
            assert_eq!(a["text"].as_str().unwrap(), actual);
            assert_eq!(a["confidence_bits"], b["confidence_bits"]);
            assert_eq!(
                a["confidence_bits"].as_u64().unwrap() as u32,
                (row["mean_confidence"].as_f64().unwrap() as f32).to_bits()
            );
            let boxes =
                super::geometry_probe::frozen_glyph_boxes(&image.data, image.width, image.height);
            if id.starts_with("boundary_") {
                boundary_boxes.push(serde_json::json!({"model":model["label"],"id":id,"boxes":boxes,"raw":raw,"fragment":fragment,"baseline":row["baseline"],"after":actual}));
            }
            if row["raw"] == row["truth"]
                && row["baseline"] != row["truth"]
                && row["closer_body"] != row["truth"]
            {
                let reason = if fragment.chars().any(char::is_whitespace)
                    || image.height as u128 * 5 > image.width as u128 * 4
                {
                    "ineligible"
                } else if boxes.len() != fragment.chars().count() {
                    "component_count"
                } else if boxes.windows(2).any(|p| p[0].1 > p[1].0) {
                    "x_overlap"
                } else {
                    "body_or_closer_geometry"
                };
                *reasons.entry(reason).or_default() += 1;
            }
            count += 1;
        }
    }
    assert_eq!(count, 3813);
    assert_eq!(reasons.values().sum::<usize>(), 289);
    std::fs::write(root.join("replay.json"),serde_json::to_vec_pretty(&serde_json::json!({"native_rows":count,"all_confidence_bits_equal":true,"all_after_texts_equal":true,"fragment_and_legacy_gate_equal":true,"body_policy_matches_frozen_reference":true,"remaining_reasons":reasons})).unwrap()).unwrap();
    std::fs::write(
        root.join("boundary-boxes.json"),
        serde_json::to_vec_pretty(&boundary_boxes).unwrap(),
    )
    .unwrap();
}
