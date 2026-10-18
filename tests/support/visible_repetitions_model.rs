use super::*;
use std::io::Read;
fn repeated_fixture(member: &str) -> (Vec<u8>, usize, usize) {
    let mut archive = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/repeated_glyph/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut bytes = Vec::new();
    archive
        .by_name(member)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
    let (w, h) = (image.width() as usize, image.height() as usize);
    (image.into_raw(), w, h)
}
#[test]
#[ignore = "requires pinned PARSeq models"]
fn visible_repetition_pool_preserves_supported_counts_and_single_line_scalar_batch_errors() {
    for (model, counter) in [
        (
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
            "upper_i_n06_gothic_png.png",
        ),
        (
            "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
            "upper_i_n12_gothic_png.png",
        ),
        (
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
            "kanji_one_n04_gothic_png.png",
        ),
    ] {
        let legacy = ParseqPool::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        let fragments = ParseqPool::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            2,
        )
        .unwrap()
        .for_fragments();
        let names = [
            "zero_n06_gothic_png.png",
            "zero_n08_gothic_png.png",
            "zero_n06_gothic_jpeg75.jpg",
            "zero_n08_gothic_jpeg75.jpg",
            counter,
        ];
        let images: Vec<_> = names
            .iter()
            .map(|n| repeated_fixture(&format!("images/{n}")))
            .collect();
        let inputs: Vec<_> = images
            .iter()
            .map(|(data, w, h)| (data.as_slice(), *w, *h))
            .collect();
        let old = legacy
            .recognize_batch_single_session_rgb_u8(&inputs)
            .unwrap();
        let new = fragments
            .recognize_batch_single_session_rgb_u8(&inputs)
            .unwrap();
        let forced = fragments
            .recognize_batch_single_session_horizontal_rgb_u8(&inputs)
            .unwrap();
        assert_eq!(new.len(), 5);
        assert_eq!(forced.len(), 5);
        for i in 0..5 {
            let expected = if i < 4 {
                if i % 2 == 0 { "000000" } else { "00000000" }
            } else {
                &old[i].text
            };
            assert_eq!(new[i].text, expected, "{model}/{i}");
            assert_eq!(forced[i].text, expected, "{model}/{i}");
            assert!((new[i].mean_confidence - old[i].mean_confidence).abs() < 1e-5);
            let scalar = fragments
                .recognize_rgb_u8(inputs[i].0, inputs[i].1, inputs[i].2)
                .unwrap();
            assert_eq!(scalar.text, expected);
        }
        assert_eq!(
            old.iter().map(|x| &x.text).collect::<Vec<_>>(),
            new.iter().map(|x| &x.text).collect::<Vec<_>>(),
            "both image-aware modes preserve the same literal repetitions"
        );
        let parallel = fragments.recognize_batch_rgb_u8(&inputs).unwrap();
        assert_eq!(
            parallel.iter().map(|x| &x.text).collect::<Vec<_>>(),
            new.iter().map(|x| &x.text).collect::<Vec<_>>()
        );
        assert!(fragments.recognize_rgb_u8(&[1, 2, 3], 4, 4).is_err());
        assert_eq!(
            fragments
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "000000"
        );
        let loaded = ParseqPool::load_for_fragments(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        assert_eq!(
            loaded
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "000000"
        );
    }
}
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn visible_repetition_page_uses_the_actual_crop_and_supplied_model() {
    use crate::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let runtime = PageRecognizer::load(
            Path::new("models/deim-s-1024x1024.onnx"),
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        for (suffix, expected) in [
            ("n06_gothic_png.png", "000000"),
            ("n08_gothic_jpeg75.jpg", "00000000"),
        ] {
            let (data, w, h) = repeated_fixture(&format!("images/zero_{suffix}"));
            for opts in [
                PageRecognizerOptions::default(),
                PageRecognizerOptions {
                    det_conf_threshold: 0.25,
                    min_line_confidence: 0.3,
                    drop_empty_text: true,
                },
            ] {
                let page = runtime.recognize_page_rgb_u8(&data, w, h, opts).unwrap();
                assert_eq!(page.lines.len(), 1, "{model}/{suffix}");
                assert_eq!(page.lines[0].text, expected, "{model}/{suffix}");
            }
        }
    }
}

#[test]
#[ignore = "requires pinned PARSeq models"]
fn visible_repetition_standalone_image_apis_keep_supported_counts() {
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let model = Path::new("models").join(model);
        let charset = Path::new("models/NDLmoji.yaml");
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 2).unwrap();
        let images = [
            repeated_fixture("images/zero_n06_gothic_png.png"),
            repeated_fixture("images/zero_n08_gothic_jpeg75.jpg"),
        ];
        let inputs: Vec<_> = images
            .iter()
            .map(|(data, w, h)| (data.as_slice(), *w, *h))
            .collect();
        for (i, (rgb, w, h)) in inputs.iter().enumerate() {
            let expected = if i == 0 { "000000" } else { "00000000" };
            let raw = session.recognize_rgb_u8_unprocessed(rgb, *w, *h).unwrap();
            let actual = session.recognize_rgb_u8(rgb, *w, *h).unwrap();
            assert_eq!(actual.text, expected);
            assert_eq!(raw.text, expected);
            assert!((actual.mean_confidence - raw.mean_confidence).abs() < 1e-6);
            assert_eq!(pool.recognize_rgb_u8(rgb, *w, *h).unwrap().text, expected);
            let single =
                crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, *w, *h, charset)
                    .unwrap();
            assert_eq!(single.text, expected);
            assert!((single.mean_confidence - raw.mean_confidence).abs() < 1e-5);
            assert_eq!(
                crate::infer::parseq::sanitize_recognized_text(&raw.text),
                "0000",
                "text-only API cannot use image evidence"
            );
        }
        assert_eq!(
            session
                .recognize_batch_rgb_u8(&inputs)
                .unwrap()
                .iter()
                .map(|x| x.text.as_str())
                .collect::<Vec<_>>(),
            ["000000", "00000000"]
        );
        assert_eq!(
            pool.recognize_batch_rgb_u8(&inputs)
                .unwrap()
                .iter()
                .map(|x| x.text.as_str())
                .collect::<Vec<_>>(),
            ["000000", "00000000"]
        );
        assert!(session.recognize_rgb_u8(&[1, 2, 3], 4, 4).is_err());
        assert_eq!(
            session
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            "000000"
        );
    }
}

#[test]
#[ignore = "requires pinned PARSeq models"]
fn visible_repetition_standalone_does_not_expand_graphical_false_text() {
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let session = ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        for case in [
            "blank",
            "solid",
            "rule",
            "speckles",
            "horizontal_marks",
            "vertical_bars",
        ] {
            let image = crate::io::load_rgb_u8(
                &Path::new("tests/fixtures/dense_line/images").join(format!("{case}.png")),
            )
            .unwrap();
            let raw = session
                .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
                .unwrap();
            let processed = session
                .recognize_rgb_u8(&image.data, image.width, image.height)
                .unwrap();
            assert_eq!(
                processed.text,
                sanitize_recognized_text(&raw.text),
                "{model}/{case}: raw={:?}",
                raw.text
            );
            assert!((processed.mean_confidence - raw.mean_confidence).abs() < 1e-5);
        }
    }
}
