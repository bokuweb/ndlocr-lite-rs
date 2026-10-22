use super::*;
fn column() -> (Vec<u8>, usize, usize) {
    let (w, h) = (20, 248);
    let mut rgb = vec![255; w * h * 3];
    for group in 0..12 {
        let top = 4 + group * 20;
        for y in top..top + 16 {
            for x in 2..18 {
                if x == 2 || x == 17 || y == top || y == top + 15 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].copy_from_slice(&[10, 40, 90]);
                }
            }
        }
    }
    (rgb, w, h)
}
fn recognition() -> RecognizeResult {
    RecognizeResult {
        text: "あいうえおかきくけこさし".to_string(),
        mean_confidence: 0.97,
    }
}
#[test]
fn no_detection_attempts_once_and_returns_the_original_pixel_result_with_requested_padding() {
    let (rgb, w, h) = column();
    let before = rgb.clone();
    let mut calls = 0;
    let actual = try_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 3,
        },
        |pixels, cw, ch| {
            calls += 1;
            assert_eq!((cw, ch), (20, 244));
            assert_eq!(pixels, &rgb[..w * 244 * 3]);
            Ok(recognition())
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(actual.0, [0, 0, 20, 244]);
    assert_eq!(actual.1.text, recognition().text);
    assert_eq!(
        actual.1.mean_confidence.to_bits(),
        recognition().mean_confidence.to_bits()
    );
    assert_eq!(rgb, before);
}
#[test]
fn existing_detection_or_unsupported_image_never_calls_an_additional_recognizer() {
    let (rgb, w, h) = column();
    for (present, image) in [(true, rgb), (false, vec![255; w * h * 3])] {
        assert!(
            try_if_empty(
                present,
                &image,
                (w, h),
                Options {
                    minimum_confidence: 0.3,
                    padding: 0
                },
                |_, _, _| panic!("no new NN")
            )
            .is_none()
        );
    }
}
#[test]
fn failed_or_low_confidence_attempt_leaves_legacy_bands_available() {
    let (rgb, w, h) = column();
    let mut calls = 0;
    assert!(
        try_if_empty(
            false,
            &rgb,
            (w, h),
            Options {
                minimum_confidence: 0.3,
                padding: 0
            },
            |_, _, _| {
                calls += 1;
                anyhow::bail!("recognition failed")
            }
        )
        .is_none()
    );
    assert_eq!(calls, 1);
    let mut calls = 0;
    assert!(
        try_if_empty(
            false,
            &rgb,
            (w, h),
            Options {
                minimum_confidence: 0.99,
                padding: 0
            },
            |_, _, _| {
                calls += 1;
                Ok(recognition())
            }
        )
        .is_none()
    );
    assert_eq!(calls, 1);
}

#[test]
fn vertical_model_bucket_uses_the_reading_axis_and_preserves_threshold_equality() {
    assert_eq!(vertical_bucket((20, 248), (25, 45)), 2.0);
    assert_eq!(vertical_bucket((70, 3000), (25, 45)), 100.0);
    assert_eq!(vertical_bucket((70, 3000), (125, 200)), 3.0);
    for (h, bucket) in [(100, 3.0), (104, 2.0), (180, 2.0), (184, 100.0)] {
        assert_eq!(vertical_bucket((10, h), (25, 45)), bucket);
    }
}

#[test]
fn prepared_callback_receives_reading_axis_bucket_and_retains_original_crop_for_reuse() {
    let (rgb, w, h) = column();
    let mut calls = 0;
    let (line, result) = try_prepared_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 3,
        },
        (25, 45),
        |line| {
            calls += 1;
            assert_eq!(line.pred_char_count, 2.0);
            assert!(line.is_vertical);
            assert_eq!((line.crop.width, line.crop.height), (20, 244));
            assert_eq!(line.crop.data, &rgb[..20 * 244 * 3]);
            Ok(recognition())
        },
    )
    .expect("supported prepared fallback");
    assert_eq!(calls, 1);
    assert_eq!(line.bbox_xyxy, [0, 0, 20, 244]);
    assert_eq!(line.confidence, 1.0);
    assert_eq!(line.crop.data, &rgb[..20 * 244 * 3]);
    assert_eq!(result.text, recognition().text);
    assert_eq!(
        result.mean_confidence.to_bits(),
        recognition().mean_confidence.to_bits()
    );
    let custom = try_prepared_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 0,
        },
        (125, 200),
        |line| {
            assert_eq!(line.pred_char_count, 3.0);
            Ok(recognition())
        },
    )
    .unwrap();
    assert_eq!(custom.0.bbox_xyxy, [1, 3, 19, 241]);
}
#[test]
fn prepared_callback_is_skipped_for_detection_and_recovers_after_failed_attempt() {
    let (rgb, w, h) = column();
    let options = Options {
        minimum_confidence: 0.3,
        padding: 0,
    };
    assert!(
        try_prepared_if_empty(true, &rgb, (w, h), options, (25, 45), |_| panic!(
            "detected inputs skip fallback"
        ))
        .is_none()
    );
    let mut attempts = 0;
    assert!(
        try_prepared_if_empty(false, &rgb, (w, h), options, (25, 45), |_| {
            attempts += 1;
            anyhow::bail!("failed initial batch")
        })
        .is_none()
    );
    assert_eq!(attempts, 1);
    assert!(
        try_prepared_if_empty(false, &rgb, (w, h), options, (25, 45), |_| {
            attempts += 1;
            Ok(recognition())
        })
        .is_some()
    );
    assert_eq!(attempts, 2);
}

#[test]
fn accepted_result_and_pixels_are_reused_without_invoking_legacy_preparation_or_recognition() {
    let (rgb, w, h) = column();
    let accepted = try_prepared_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 0,
        },
        (25, 45),
        |_| Ok(recognition()),
    )
    .unwrap();
    let expected = accepted.0.clone();
    let (crops, results) = reuse_or_legacy(
        Some(accepted),
        |_, results| Ok(results),
        || panic!("accepted result must not recognize again"),
    )
    .unwrap();
    assert_eq!(crops.len(), 1);
    assert_eq!(results.len(), 1);
    assert_eq!(crops[0].crop, expected.crop);
    assert_eq!(crops[0].bbox_xyxy, expected.bbox_xyxy);
    assert_eq!(crops[0].pred_char_count, expected.pred_char_count);
    assert_eq!(results[0].text, recognition().text);
    assert_eq!(
        results[0].mean_confidence.to_bits(),
        recognition().mean_confidence.to_bits()
    );
}
#[test]
fn rejected_attempt_invokes_legacy_once_and_preserves_its_errors() {
    let mut calls = 0;
    let (crops, results) = reuse_or_legacy(
        None,
        |_, results| Ok(results),
        || {
            calls += 1;
            Ok((Vec::new(), vec![recognition()]))
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert!(crops.is_empty());
    assert_eq!(results[0].text, recognition().text);
    let error = reuse_or_legacy(
        None,
        |_, results| Ok(results),
        || {
            calls += 1;
            anyhow::bail!("legacy model failed")
        },
    )
    .unwrap_err();
    assert_eq!(calls, 2);
    assert_eq!(error.to_string(), "legacy model failed");
}

#[test]
fn accepted_initial_result_runs_existing_finish_once_on_the_same_crop_and_retains_its_result() {
    let (rgb, w, h) = column();
    let accepted = try_prepared_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 0,
        },
        (25, 45),
        |_| Ok(recognition()),
    )
    .unwrap();
    let expected = accepted.0.clone();
    let mut calls = 0;
    let (crops, results) = reuse_or_legacy(
        Some(accepted),
        |crops, initial| {
            calls += 1;
            assert_eq!(crops.len(), 1);
            assert_eq!(crops[0].crop, expected.crop);
            assert_eq!(crops[0].bbox_xyxy, expected.bbox_xyxy);
            assert_eq!(initial.len(), 1);
            assert_eq!(initial[0].text, recognition().text);
            Ok(vec![RecognizeResult {
                text: "FINISHED".into(),
                mean_confidence: f32::from_bits(0x3f71_2345),
            }])
        },
        || panic!("accepted crop must not use legacy recognition"),
    )
    .unwrap();
    assert_eq!(
        calls, 1,
        "the existing finishing pipeline must run after initial recognition"
    );
    assert_eq!(crops[0].crop, expected.crop);
    assert_eq!(results[0].text, "FINISHED");
    assert_eq!(results[0].mean_confidence.to_bits(), 0x3f71_2345);
}

#[test]
fn finish_errors_are_not_hidden_and_rejected_attempts_never_run_finish() {
    let (rgb, w, h) = column();
    let accepted = try_prepared_if_empty(
        false,
        &rgb,
        (w, h),
        Options {
            minimum_confidence: 0.3,
            padding: 0,
        },
        (25, 45),
        |_| Ok(recognition()),
    )
    .unwrap();
    let result = reuse_or_legacy(
        Some(accepted),
        |_, _| anyhow::bail!("finish failed"),
        || panic!("finish error must propagate"),
    );
    assert_eq!(result.unwrap_err().to_string(), "finish failed");
    let mut calls = 0;
    let (_, results) = reuse_or_legacy(
        None,
        |_, _| panic!("legacy already applies its own finish"),
        || {
            calls += 1;
            Ok((Vec::new(), vec![recognition()]))
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(results[0].text, recognition().text);
}
