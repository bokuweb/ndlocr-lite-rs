use super::*;

fn assets() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    Some((
        std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL")?.into(),
        std::env::var_os("NDLOCR_TEST_CHARSET")?.into(),
    ))
}

fn images() -> (crate::io::RgbImageData, crate::io::RgbImageData) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/punctuation_context");
    (
        crate::io::load_rgb_u8(&root.join("joined30.png")).unwrap(),
        crate::io::load_rgb_u8(&root.join("real_space30.png")).unwrap(),
    )
}

#[test]
#[ignore = "requires fixed PARSeq 30 assets"]
fn session_recovers_punctuation_context_without_reloading_and_preserves_raw() {
    let Some((model, charset)) = assets() else {
        return;
    };
    let (image, negative) = images();
    let temp = tempfile::tempdir().unwrap();
    let m = temp.path().join("model.onnx");
    let c = temp.path().join("chars.yaml");
    std::fs::copy(model, &m).unwrap();
    std::fs::copy(charset, &c).unwrap();
    let session = ParseqSession::load(&m, &c).unwrap();
    assert_eq!(session.input_dimensions(), (256, 24));
    std::fs::remove_file(m).unwrap();
    std::fs::remove_file(c).unwrap();
    let raw = session
        .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
        .unwrap();
    assert_eq!(raw.text, "測定場所、 時刻、");
    let candidates = session
        .recognize_rgb_u8_candidates_with_orientation(
            &image.data,
            image.width,
            image.height,
            ParseqOrientation::Auto,
            2,
        )
        .unwrap();
    assert_eq!(candidates.recognition.text, raw.text);
    assert!(session.recognize_rgb_u8(&[], 0, 0).is_err());
    let got = session
        .recognize_rgb_u8(&image.data, image.width, image.height)
        .unwrap();
    assert_eq!(got.text, "測定場所、時刻、");
    assert!(got.mean_confidence > raw.mean_confidence);
    let again = session
        .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
        .unwrap();
    assert_eq!(again.text, raw.text);
    assert_eq!(again.mean_confidence, raw.mean_confidence);
    let after = session
        .recognize_rgb_u8_candidates_with_orientation(
            &image.data,
            image.width,
            image.height,
            ParseqOrientation::Auto,
            2,
        )
        .unwrap();
    assert_eq!(candidates.recognition.text, after.recognition.text);
    assert_eq!(
        candidates.recognition.mean_confidence,
        after.recognition.mean_confidence
    );
    assert_eq!(candidates.steps, after.steps);
    assert_eq!(candidates.stop_reason, after.stop_reason);
    let raw_negative = session
        .recognize_rgb_u8_unprocessed(&negative.data, negative.width, negative.height)
        .unwrap();
    let got_negative = session
        .recognize_rgb_u8(&negative.data, negative.width, negative.height)
        .unwrap();
    assert_eq!(got_negative.text, "受付番地、 通知先");
    assert_eq!(got_negative.mean_confidence, raw_negative.mean_confidence);
}

#[test]
#[ignore = "requires fixed PARSeq 30 assets"]
fn session_batch_and_both_pool_policies_recover_the_same_candidate() {
    let Some((model, charset)) = assets() else {
        return;
    };
    let (image, negative) = images();
    let items = [
        (&image.data[..], image.width, image.height),
        (&negative.data[..], negative.width, negative.height),
    ];
    let verify = |out: Vec<RecognizeResult>| {
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].text, "測定場所、時刻、");
        assert_eq!(out[1].text, "受付番地、 通知先");
    };
    let session = ParseqSession::load(&model, &charset).unwrap();
    verify(session.recognize_batch_rgb_u8(&items).unwrap());
    for fragment in [false, true] {
        let pool = if fragment {
            ParseqPool::load_for_fragments(&model, &charset, 2)
        } else {
            ParseqPool::load(&model, &charset, 2)
        }
        .unwrap();
        assert!(pool.recognize_rgb_u8(&[], 0, 0).is_err());
        assert_eq!(
            pool.recognize_rgb_u8(&image.data, image.width, image.height)
                .unwrap()
                .text,
            "測定場所、時刻、"
        );
        verify(pool.recognize_batch_single_session_rgb_u8(&items).unwrap());
        verify(pool.recognize_batch_rgb_u8(&items).unwrap());
        verify(
            pool.recognize_batch_single_session_horizontal_rgb_u8(&items)
                .unwrap(),
        );
    }
}

#[test]
#[ignore = "requires fixed PARSeq 30 assets"]
fn standalone_recovers_context_and_leaves_unprocessed_output_unchanged() {
    let Some((model, charset)) = assets() else {
        return;
    };
    let (image, negative) = images();
    let raw = super::super::parseq::recognize_rgb_u8_with_score_unprocessed(
        &model,
        &image.data,
        image.width,
        image.height,
        &charset,
    )
    .unwrap();
    assert_eq!(raw.text, "測定場所、 時刻、");
    let got = super::super::parseq::recognize_rgb_u8_with_score(
        &model,
        &image.data,
        image.width,
        image.height,
        &charset,
    )
    .unwrap();
    assert_eq!(got.text, "測定場所、時刻、");
    assert!(got.mean_confidence > raw.mean_confidence);
    assert_eq!(
        super::super::parseq::recognize_rgb_u8_with_score(
            &model,
            &negative.data,
            negative.width,
            negative.height,
            &charset
        )
        .unwrap()
        .text,
        "受付番地、 通知先"
    );
}

#[test]
#[ignore = "requires all three fixed PARSeq assets"]
fn mixed_cascade_buckets_keep_the_supplied_models_and_original_order() {
    let Some((model30, charset)) = assets() else {
        return;
    };
    let directory = model30.parent().unwrap();
    let model50 = directory.join("parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx");
    let model100 = directory.join("parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx");
    let cascade = ParseqCascadePool::load(&model30, &model50, &model100, &charset, 1).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/punctuation_context");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    // 各bucketに1枚ずつ。本文長は25/45を跨がず、別モデルへのfallbackを起こさない。
    let cases = [
        ("joined100.png", None, "model100"),
        ("joined30.png", Some(3.0), "model30"),
        ("joined50.png", Some(2.0), "model50"),
    ];
    let images: Vec<_> = cases
        .iter()
        .map(|(f, _, _)| crate::io::load_rgb_u8(&fixture.join(f)).unwrap())
        .collect();
    let inputs: Vec<_> = images
        .iter()
        .zip(cases)
        .map(|(im, (_, pcc, _))| (&im.data[..], im.width, im.height, pcc))
        .collect();
    let got = cascade
        .recognize_batch_with_buckets_rgb_u8(&inputs)
        .unwrap();
    assert_eq!(got.len(), 3);
    for ((file, _, key), result) in cases.into_iter().zip(got) {
        let pin = manifest
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["file"] == file)
            .unwrap();
        assert_eq!(result.text, pin["truth"].as_str().unwrap());
        assert!(
            (result.mean_confidence
                - pin[key]["candidates"][0]["mean_confidence"]
                    .as_f64()
                    .unwrap() as f32)
                .abs()
                < 1e-6
        );
    }
}

#[test]
#[ignore = "requires fixed PARSeq 100 assets"]
fn condensed_real_punctuation_space_is_not_removed() {
    let Some((model30, charset)) = assets() else {
        return;
    };
    let model = model30
        .parent()
        .unwrap()
        .join("parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx");
    let image = crate::io::load_rgb_u8(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/punctuation_context/condensed_real_space100.png"),
    )
    .unwrap();
    let session = ParseqSession::load(&model, &charset).unwrap();
    let raw = session
        .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
        .unwrap();
    assert_eq!(raw.text, "案内方法、 連絡先");
    let got = session
        .recognize_rgb_u8(&image.data, image.width, image.height)
        .unwrap();
    assert_eq!(got.text, raw.text);
    assert_eq!(got.mean_confidence, raw.mean_confidence);
}
