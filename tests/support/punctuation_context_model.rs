//! Regressions for the rejected automatic punctuation-space selection.
use super::*;
use std::path::PathBuf;

fn assets(model: &str) -> Option<(PathBuf, PathBuf)> {
    let model30 = PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL")?);
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET")?);
    let path = match model {
        "30" => model30,
        "50" => model30
            .parent()?
            .join("parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx"),
        "100" => model30
            .parent()?
            .join("parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
        _ => panic!("unknown model"),
    };
    Some((path, charset))
}

fn fixture(name: &str) -> crate::io::RgbImageData {
    crate::io::load_rgb_u8(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/punctuation_context")
            .join(name),
    )
    .unwrap()
}

fn truth(model: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/punctuation_context/rejection-manifest.json");
    let pins: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    pins.as_array()
        .unwrap()
        .iter()
        .find(|x| x["model"] == model)
        .unwrap()["truth"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn same(actual: &RecognizeResult, expected: &RecognizeResult) {
    assert_eq!(actual.text, expected.text);
    assert_eq!(
        actual.mean_confidence.to_bits(),
        expected.mean_confidence.to_bits()
    );
}

fn verify_public_paths(model: &str) {
    let Some((model_path, charset)) = assets(model) else {
        return;
    };
    let image = fixture(&format!("proportional_real_space{model}.png"));
    let old_positive = fixture(&format!("joined{model}.png"));
    let input_before = image.data.clone();
    let items = [
        (&image.data[..], image.width, image.height),
        (
            &old_positive.data[..],
            old_positive.width,
            old_positive.height,
        ),
    ];
    let temp = tempfile::tempdir().unwrap();
    let m = temp.path().join("model.onnx");
    let c = temp.path().join("charset.yaml");
    std::fs::copy(&model_path, &m).unwrap();
    std::fs::copy(&charset, &c).unwrap();
    let session = ParseqSession::load(&m, &c).unwrap();
    std::fs::remove_file(m).unwrap();
    std::fs::remove_file(c).unwrap();
    let raw: Vec<_> = items
        .iter()
        .map(|&(rgb, w, h)| session.recognize_rgb_u8_unprocessed(rgb, w, h).unwrap())
        .collect();
    assert_eq!(raw[0].text, truth(model));
    let candidates = session
        .recognize_rgb_u8_candidates_with_orientation(
            &image.data,
            image.width,
            image.height,
            ParseqOrientation::Auto,
            2,
        )
        .unwrap();
    same(&candidates.recognition, &raw[0]);
    assert!(session.recognize_rgb_u8(&[], 0, 0).is_err());
    for (&(rgb, w, h), expected) in items.iter().zip(&raw) {
        same(&session.recognize_rgb_u8(rgb, w, h).unwrap(), expected);
    }
    let verify = |results: Vec<RecognizeResult>| {
        assert_eq!(results.len(), raw.len());
        for (result, expected) in results.iter().zip(&raw) {
            same(result, expected);
        }
    };
    verify(session.recognize_batch_rgb_u8(&items).unwrap());
    let again = session
        .recognize_rgb_u8_candidates_with_orientation(
            &image.data,
            image.width,
            image.height,
            ParseqOrientation::Auto,
            2,
        )
        .unwrap();
    same(&again.recognition, &raw[0]);
    assert_eq!(again.steps, candidates.steps);
    for fragments in [false, true] {
        let pool = if fragments {
            ParseqPool::load_for_fragments(&model_path, &charset, 2)
        } else {
            ParseqPool::load(&model_path, &charset, 2)
        }
        .unwrap();
        assert!(pool.recognize_rgb_u8(&[], 0, 0).is_err());
        for (&(rgb, w, h), expected) in items.iter().zip(&raw) {
            same(&pool.recognize_rgb_u8(rgb, w, h).unwrap(), expected);
        }
        verify(pool.recognize_batch_rgb_u8(&items).unwrap());
        verify(pool.recognize_batch_single_session_rgb_u8(&items).unwrap());
        verify(
            pool.recognize_batch_single_session_horizontal_rgb_u8(&items)
                .unwrap(),
        );
    }
    for (&(rgb, w, h), expected) in items.iter().zip(&raw) {
        same(
            &super::super::parseq::recognize_rgb_u8_with_score(&model_path, rgb, w, h, &charset)
                .unwrap(),
            expected,
        );
        same(
            &super::super::parseq::recognize_rgb_u8_with_score_unprocessed(
                &model_path,
                rgb,
                w,
                h,
                &charset,
            )
            .unwrap(),
            expected,
        );
    }
    assert_eq!(image.data, input_before);
}

macro_rules! preserve_space_test {
    ($name:ident, $model:literal) => {
        #[test]
        #[ignore = "requires fixed PARSeq assets"]
        fn $name() {
            verify_public_paths($model);
        }
    };
}
preserve_space_test!(
    model30_keeps_proportional_word_space_on_all_public_paths,
    "30"
);
preserve_space_test!(
    model50_keeps_proportional_word_space_on_all_public_paths,
    "50"
);
preserve_space_test!(
    model100_keeps_proportional_word_space_on_all_public_paths,
    "100"
);

#[test]
#[ignore = "requires all three fixed PARSeq assets"]
fn mixed_cascade_keeps_word_spaces_models_confidence_and_order() {
    let Some((m30, charset)) = assets("30") else {
        return;
    };
    let m50 = assets("50").unwrap().0;
    let m100 = assets("100").unwrap().0;
    let cascade = ParseqCascadePool::load(&m30, &m50, &m100, &charset, 1).unwrap();
    let cases = [("100", None), ("30", Some(3.0)), ("50", Some(2.0))];
    let images: Vec<_> = cases
        .iter()
        .map(|(model, _)| fixture(&format!("proportional_real_space{model}.png")))
        .collect();
    let inputs: Vec<_> = images
        .iter()
        .zip(cases)
        .map(|(im, (_, pcc))| (&im.data[..], im.width, im.height, pcc))
        .collect();
    let results = cascade
        .recognize_batch_with_buckets_rgb_u8(&inputs)
        .unwrap();
    assert_eq!(results.len(), 3);
    for ((result, image), (model, _)) in results.iter().zip(&images).zip(cases) {
        let path = assets(model).unwrap().0;
        let raw = ParseqSession::load(&path, &charset)
            .unwrap()
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(raw.text, truth(model));
        same(result, &raw);
    }
}

#[test]
#[ignore = "requires fixed PARSeq 100 assets"]
fn condensed_real_space_is_also_preserved() {
    let Some((model, charset)) = assets("100") else {
        return;
    };
    let image = fixture("condensed_real_space100.png");
    let session = ParseqSession::load(&model, &charset).unwrap();
    let raw = session
        .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
        .unwrap();
    same(
        &session
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap(),
        &raw,
    );
    assert!(raw.text.contains(' '));
}
