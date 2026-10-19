use super::*;
#[test]
#[ignore = "diagnostic; requires explicit frozen closer root and pinned models"]
fn capture_literal_closer_raw_matrix() {
    use crate::infer::{cached::ParseqSession, parseq::ParseqOrientation};
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_CLOSER_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 583);
    std::fs::create_dir_all(root.join("inputs")).unwrap();
    let mut models = Vec::new();
    for (label, model) in [
        (
            "single30",
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "single50",
            "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "single100",
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
    ] {
        let session = ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let mut rows = Vec::new();
        for row in inputs {
            let id = row["id"].as_str().unwrap();
            let source = crate::io::load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
            let decoded = session
                .recognize_rgb_u8_candidates_with_orientation(
                    &source.data,
                    source.width,
                    source.height,
                    ParseqOrientation::Auto,
                    2,
                )
                .unwrap();
            let raw = &decoded.recognition.text;
            let current = sanitize_visible_single_line(
                raw,
                &source.data,
                source.width,
                source.height,
                ParseqOrientation::Auto,
            );
            let fragment = sanitize_visible_repetitions(
                raw,
                &source.data,
                source.width,
                source.height,
                ParseqOrientation::Auto,
            );
            let supported = supports_glyph_count(
                &source.data,
                source.width,
                source.height,
                fragment.chars().count(),
            );
            let horizontal = source.height as u128 * 5 <= source.width as u128 * 4;
            let pixel = if horizontal && supported {
                fragment.clone()
            } else {
                current.clone()
            };
            let truth = std::fs::read_to_string(Path::new(row["truth"].as_str().unwrap())).unwrap();
            let png = root.join("inputs").join(format!("{id}.png"));
            if !png.exists() {
                image::RgbImage::from_raw(source.width as u32, source.height as u32, source.data)
                    .unwrap()
                    .save(&png)
                    .unwrap();
            }
            if row["cohort"] != "literal_closers" {
                let expected = std::fs::read_to_string(
                    root.join("expected").join(label).join(format!("{id}.txt")),
                )
                .unwrap();
                assert_eq!(current, expected, "{label}/{id}: raw={raw:?}");
            }
            rows.push(serde_json::json!({"id":id,"cohort":row["cohort"],"png":png,"width":source.width,"height":source.height,"truth":truth,"raw":raw,"mean_confidence":decoded.recognition.mean_confidence,"stop_reason":format!("{:?}",decoded.stop_reason),"steps":decoded.steps.iter().map(|s|serde_json::json!({"timestep":s.timestep,"range":s.text_range.as_ref().map(|r|[r.start,r.end]),"candidates":s.candidates.iter().map(|c|serde_json::json!({"token_id":c.token_id,"char":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"current":current,"fragment":fragment,"count_supported":supported,"horizontal_auto":horizontal,"pixel_candidate":pixel}));
        }
        models.push(serde_json::json!({"label":label,"model":model,"native":session.input_dimensions(),"rows":rows}));
    }
    std::fs::write(
        root.join("raw.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":models})).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires pinned PARSeq models"]
fn literal_closer_image_apis_preserve_supported_raw_closers() {
    use crate::infer::{
        cached::{ParseqCascadePool, ParseqPool, ParseqSession},
        parseq::ParseqOrientation,
    };
    use std::{io::Read, path::Path};
    let mut archive = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/literal_closers/corpus.zip").unwrap(),
    )
    .unwrap();
    let names = [
        "round_latin_suffix_gothic_png.png",
        "round_latin_prefix_gothic_jpeg75.jpg",
        "round_latin_internal_gothic_png.png",
        "full_round_latin_internal_gothic_png.png",
    ];
    let expected = ["ABC)", ")ABC", "A)BC", "A BC"];
    let mut images = Vec::new();
    for name in names {
        let mut bytes = Vec::new();
        archive
            .by_name(&format!("images/{name}"))
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
        images.push((
            image.width() as usize,
            image.height() as usize,
            image.into_raw(),
        ));
    }
    let inputs: Vec<_> = images
        .iter()
        .map(|(w, h, data)| (data.as_slice(), *w, *h))
        .collect();
    let charset = Path::new("models/NDLmoji.yaml");
    let models = [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ];
    for file in models {
        let model = Path::new("models").join(file);
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 2).unwrap();
        let fragment = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
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
            let stateless =
                crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, *w, *h, charset)
                    .unwrap();
            assert_eq!(stateless.text, expected[i]);
            assert!((stateless.mean_confidence - raw.mean_confidence).abs() < 1e-5);
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
            assert_eq!(
                fragment.recognize_rgb_u8(rgb, *w, *h).unwrap().text,
                raw.text
            );
            if i < 3 {
                assert_eq!(raw.text, expected[i]);
                assert_eq!(
                    crate::infer::parseq::sanitize_recognized_text(&raw.text),
                    "ABC"
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
        assert!(session.recognize_rgb_u8(&[], 4, 4).is_err());
        assert!(pool.recognize_batch_rgb_u8(&[(&[], 4, 4)]).is_err());
        assert_eq!(
            session
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "ABC)"
        );
        assert_eq!(
            pool.recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "ABC)"
        );
    }
    let cascade = ParseqCascadePool::load(
        &Path::new("models").join(models[0]),
        &Path::new("models").join(models[1]),
        &Path::new("models").join(models[2]),
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
        actual.iter().map(|x| x.text.as_str()).collect::<Vec<_>>(),
        expected.repeat(3)
    );
}

#[test]
#[ignore = "diagnostic; requires explicit frozen closer root"]
fn replay_literal_closer_candidate_outputs() {
    let Ok(root) = std::env::var("NDLOCR_CLOSER_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("raw.json")).unwrap()).unwrap();
    let mut ledger = Vec::new();
    for model in raw["models"].as_array().unwrap() {
        let label = model["label"].as_str().unwrap();
        let dir = root.join("predicted").join(label);
        std::fs::create_dir_all(&dir).unwrap();
        let mut count = 0;
        for row in model["rows"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let image =
                crate::io::load_rgb_u8(&root.join("inputs").join(format!("{id}.png"))).unwrap();
            let result = sanitize_visible_single_line(
                row["raw"].as_str().unwrap(),
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let fragment = sanitize_visible_repetitions(
                row["raw"].as_str().unwrap(),
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            assert_eq!(fragment, row["fragment"].as_str().unwrap());
            let expected = if row["horizontal_auto"].as_bool().unwrap()
                && row["count_supported"].as_bool().unwrap()
                && !fragment.chars().any(char::is_whitespace)
            {
                fragment
            } else {
                row["current"].as_str().unwrap().to_string()
            };
            assert_eq!(result, expected, "{label}/{id}");
            std::fs::write(dir.join(format!("{id}.txt")), &result).unwrap();
            count += 1;
        }
        assert_eq!(count, 583);
        ledger.push(serde_json::json!({"label":label,"inputs":count,"fragment_unchanged":true}));
    }
    std::fs::write(
        root.join("candidate-replay.json"),
        serde_json::to_vec_pretty(&ledger).unwrap(),
    )
    .unwrap();
}
