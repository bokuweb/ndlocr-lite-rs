use super::*;
use clap::Parser;

#[test]
#[ignore = "frozen known image and pinned three models; preserves finishing options"]
fn accepted_vertical_finishes_like_existing_page_batch_with_quality_options() {
    let image_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/vertical_fallback_cli_finish/arial12.png");
    let temporary = tempfile::tempdir().unwrap();
    let output = std::env::var_os("NDLOCR_TEST_FVC_FINISH_EVIDENCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temporary.path().join("quality"));
    std::fs::create_dir(&output).unwrap();
    let cli = Cli::try_parse_from([
        "ndlocr-lite-rs",
        "recognize-page",
        "--image",
        image_path.to_str().unwrap(),
        "--use-deim-detection",
        "false",
        "--structure-rules",
        "false",
    ])
    .unwrap();
    let Command::RecognizePage(mut args) = cli.command else {
        panic!("wrong CLI command")
    };
    let image = crate::io::load_rgb_u8(&args.image).unwrap();
    let cached = CachedPageRecognizer::load(&args).unwrap();
    let mut records = Vec::new();
    let mut option_changed_initial = false;
    for (quality, delta) in [(false, 0.03), (true, 0.03), (true, -100.0)] {
        args.quality_boost = quality;
        args.quality_boost_min_delta = delta;
        let accepted = vertical_fallback::try_prepared_if_empty(
            false,
            &image.data,
            (image.width, image.height),
            vertical_fallback::Options {
                minimum_confidence: args.min_line_confidence,
                padding: args.line_crop_padding as usize,
            },
            (
                args.cascade_threshold_30_to_50,
                args.cascade_threshold_50_to_100,
            ),
            |line| {
                let mut results =
                    cached.recognize_initial_batch(std::slice::from_ref(line), &args)?;
                assert_eq!(results.len(), 1);
                Ok(results.remove(0))
            },
        )
        .expect("fixed known image must exercise accepted vertical fallback");
        let line = accepted.0.clone();
        let initial = accepted.1.clone();
        let expected = cached
            .recognize_for_page_batch(std::slice::from_ref(&line), &args)
            .unwrap();
        let (crops, actual) = vertical_fallback::reuse_or_legacy(
            Some(accepted),
            |crops, results| cached.finish_for_page_batch(crops, results, &args),
            || panic!("known accepted column must not use legacy"),
        )
        .unwrap();
        assert_eq!(crops.len(), 1);
        assert_eq!(crops[0].crop, line.crop);
        assert_eq!(crops[0].bbox_xyxy, line.bbox_xyxy);
        assert_eq!(actual.len(), 1);
        assert_eq!(expected.len(), 1);
        assert_eq!(actual[0].text, expected[0].text);
        assert_eq!(
            actual[0].mean_confidence.to_bits(),
            expected[0].mean_confidence.to_bits()
        );
        if quality
            && (actual[0].text != initial.text
                || actual[0].mean_confidence.to_bits() != initial.mean_confidence.to_bits())
        {
            option_changed_initial = true;
        }
        records.push(serde_json::json!({"quality_boost":quality,"quality_boost_min_delta":delta,"bbox":line.bbox_xyxy,"crop_width":line.crop.width,"crop_height":line.crop.height,"initial_text":initial.text,"initial_mean_bits":initial.mean_confidence.to_bits(),"expected_text":expected[0].text,"expected_mean_bits":expected[0].mean_confidence.to_bits(),"actual_text":actual[0].text,"actual_mean_bits":actual[0].mean_confidence.to_bits()}));
        std::fs::write(
            output.join(format!("crop-{}.rgb", records.len())),
            &line.crop.data,
        )
        .unwrap();
    }
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    assert!(
        option_changed_initial,
        "known quality option must change text or f32 confidence so skipping finish would fail this regression"
    );
}
