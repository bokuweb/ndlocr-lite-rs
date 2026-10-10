#![cfg(all(feature = "onnx", unix))]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::recognize_rgb_u8_with_score_unprocessed},
    io::load_rgb_u8,
};
use std::path::PathBuf;

/// 同じロード済みモデルを使い、反復本文と信頼度を後処理なしで保持する。
#[test]
#[ignore = "requires fixed PARSeq assets and the 111111 image"]
fn reusable_session_preserves_repetitions_without_reloading_files() {
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let image_path =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_REPEAT_IMAGE").expect("image required"));
    let image = load_rgb_u8(&image_path).unwrap();
    let expected = recognize_rgb_u8_with_score_unprocessed(
        &model,
        &image.data,
        image.width,
        image.height,
        &charset,
    )
    .unwrap();
    assert_eq!(expected.text, "111111");
    let directory = tempfile::tempdir().unwrap();
    let model_copy = directory.path().join("model.onnx");
    let charset_copy = directory.path().join("charset.yaml");
    std::fs::copy(model, &model_copy).unwrap();
    std::fs::copy(charset, &charset_copy).unwrap();
    let session = ParseqSession::load(&model_copy, &charset_copy).unwrap();
    std::fs::remove_file(model_copy).unwrap();
    std::fs::remove_file(charset_copy).unwrap();
    for _ in 0..3 {
        let actual = session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(actual.text, expected.text);
        assert!((actual.mean_confidence - expected.mean_confidence).abs() < 1e-5);
    }
    assert!(session.recognize_rgb_u8_unprocessed(&[], 0, 0).is_err());
    assert_eq!(
        session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap()
            .text,
        expected.text
    );
}
