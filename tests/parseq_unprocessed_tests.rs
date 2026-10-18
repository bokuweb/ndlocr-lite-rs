use ndlocr_lite_rs::infer::parseq::recognize_rgb_u8_with_score_unprocessed as recognize_unprocessed;

/// 後処理なしでも存在しないファイルを成功結果に変えない。
#[test]
fn unprocessed_recognition_rejects_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let error = recognize_unprocessed(
        &directory.path().join("missing.onnx"),
        &[255; 3],
        1,
        1,
        &directory.path().join("missing.yaml"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("file not found"));
}

/// ONNXを組み込まない構成では、新しい入口も明示エラーを返す。
#[test]
#[cfg(not(feature = "onnx"))]
fn unprocessed_recognition_reports_disabled_runtime() {
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("model.onnx");
    let charset = directory.path().join("charset.yaml");
    std::fs::write(&model, b"dummy").unwrap();
    std::fs::write(&charset, "model:\n  charset_train: \"abc\"\n").unwrap();
    let error = recognize_unprocessed(&model, &[255; 3], 1, 1, &charset).unwrap_err();
    assert!(error.to_string().contains("onnx feature is disabled"));
}

/// 固定実画像の正当な反復数字を、認識後の規則で削除しない。
#[test]
#[cfg(feature = "onnx")]
#[ignore = "requires NDLOCR_TEST_PARSEQ_MODEL and NDLOCR_TEST_CHARSET"]
fn unprocessed_recognition_preserves_repeated_digits_and_patterns() {
    let model = std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model is required");
    let charset = std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset is required");
    let fixtures =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parseq_unprocessed");
    for case in ["repeated_digits", "repeated_pattern"] {
        let image = ndlocr_lite_rs::io::load_rgb_u8(&fixtures.join(format!("{case}.png"))).unwrap();
        let result = recognize_unprocessed(
            std::path::Path::new(&model),
            &image.data,
            image.width,
            image.height,
            std::path::Path::new(&charset),
        )
        .unwrap();
        let expected = std::fs::read_to_string(fixtures.join(format!("{case}.txt"))).unwrap();
        assert_eq!(result.text, expected.trim_end(), "{case}");
        assert!(result.mean_confidence.is_finite());
        assert!((0.5..=1.0).contains(&result.mean_confidence));

        let text_only_expected = match case {
            "repeated_digits" => "1111",
            "repeated_pattern" => "123123",
            _ => unreachable!(),
        };
        let processed = ndlocr_lite_rs::infer::parseq::recognize_rgb_u8_with_score(
            std::path::Path::new(&model),
            &image.data,
            image.width,
            image.height,
            std::path::Path::new(&charset),
        )
        .unwrap();
        assert_eq!(
            processed.text,
            expected.trim_end(),
            "image-supported recognition: {case}"
        );
        assert_eq!(
            ndlocr_lite_rs::infer::parseq::sanitize_recognized_text(&result.text),
            text_only_expected,
            "text-only policy remains compatible: {case}"
        );
        assert!((processed.mean_confidence - result.mean_confidence).abs() < 1e-6);
        assert_eq!(
            ndlocr_lite_rs::infer::parseq::recognize_rgb_u8(
                std::path::Path::new(&model),
                &image.data,
                image.width,
                image.height,
                std::path::Path::new(&charset),
            )
            .unwrap(),
            expected.trim_end()
        );
        assert_eq!(
            ndlocr_lite_rs::infer::parseq::recognize_image(
                std::path::Path::new(&model),
                &fixtures.join(format!("{case}.png")),
                std::path::Path::new(&charset),
            )
            .unwrap(),
            expected.trim_end()
        );
    }
}
