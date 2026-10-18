use super::*;
fn primary100(parallelism: usize) -> PageRecognizer {
    PageRecognizer::load(
        Path::new("models/deim-s-1024x1024.onnx"),
        Path::new("models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
        parallelism,
    )
    .unwrap()
}
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_recovers_text_when_deim_has_no_lines() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/dense_line/images/library_bold36_original.png",
    ))
    .unwrap();
    let raw =
        super::super::deim_cached::DeimSession::load(Path::new("models/deim-s-1024x1024.onnx"))
            .unwrap()
            .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
            .unwrap();
    assert!(
        crate::app::prepare_line_crops(&image.data, image.width, image.height, raw, 0)
            .unwrap()
            .is_empty()
    );
    let runtime = primary100(1);
    let page = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions::default(),
        )
        .unwrap();
    let truth =
        std::fs::read_to_string("tests/fixtures/dense_line/truth/library_bold36_original.txt")
            .unwrap();
    assert_eq!(page.lines.len(), 1);
    assert_eq!(page.lines[0].text, truth);
    assert_eq!(page.lines[0].bbox_xyxy, [4, 4, 4303, 42]);
}
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_keeps_graphical_controls_empty() {
    let runtime = primary100(1);
    for name in [
        "blank",
        "solid",
        "rule",
        "speckles",
        "horizontal_marks",
        "vertical_bars",
    ] {
        let image = crate::io::load_rgb_u8(
            &Path::new("tests/fixtures/dense_line/images").join(format!("{name}.png")),
        )
        .unwrap();
        let page = runtime
            .recognize_page_rgb_u8(
                &image.data,
                image.width,
                image.height,
                PageRecognizerOptions::default(),
            )
            .unwrap();
        assert!(page.lines.is_empty(), "{name}: {:?}", page.lines);
    }
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_preserves_single_horizontal_japanese() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/page_band_fallback/images/single_ja_gothic_png.png",
    ))
    .unwrap();
    let runtime = primary100(1);
    let page = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions::default(),
        )
        .unwrap();
    assert_eq!(
        page.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>(),
        ["雨"]
    );
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_preserves_sparse_horizontal_words() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/dense_line/images/spaced_short.png",
    ))
    .unwrap();
    for name in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let runtime = PageRecognizer::load(
            Path::new("models/deim-s-1024x1024.onnx"),
            &Path::new("models").join(name),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        let page = runtime
            .recognize_page_rgb_u8(
                &image.data,
                image.width,
                image.height,
                PageRecognizerOptions::default(),
            )
            .unwrap();
        assert_eq!(
            page.lines
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>(),
            ["図 書 館"]
        );
    }
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_from_pools_parallel_order_options_and_error_reuse() {
    let runtime = PageRecognizer::from_pools(
        DeimPool::load(Path::new("models/deim-s-1024x1024.onnx"), 2).unwrap(),
        ParseqPool::load(
            Path::new("models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
            Path::new("models/NDLmoji.yaml"),
            2,
        )
        .unwrap(),
    );
    let names = [
        "tests/fixtures/dense_line/images/blank.png",
        "tests/fixtures/dense_line/images/library_bold36_original.png",
        "tests/fixtures/page_band_fallback/images/single_ja_gothic_png.png",
        "tests/fixtures/page_band_fallback/images/two_rows_gothic_png.png",
        "tests/fixtures/dense_line/images/spaced_short.png",
    ];
    let images: Vec<_> = names
        .iter()
        .map(|name| crate::io::load_rgb_u8(Path::new(name)).unwrap())
        .collect();
    let payload: Vec<_> = images
        .iter()
        .map(|im| (im.data.as_slice(), im.width, im.height))
        .collect();
    let opts = PageRecognizerOptions::default();
    let sequential: Vec<_> = payload
        .iter()
        .map(|&(data, w, h)| runtime.recognize_page_rgb_u8(data, w, h, opts).unwrap())
        .collect();
    let parallel = runtime.recognize_pages_rgb_u8(&payload, opts).unwrap();
    assert_eq!(parallel.len(), payload.len());
    for (index, (a, b)) in sequential.iter().zip(parallel).enumerate() {
        assert_eq!(b.page_index, index);
        assert_eq!(a.lines, b.lines);
    }
    assert!(sequential[0].lines.is_empty());
    assert_eq!(sequential[1].lines[0].text.chars().count(), 120);
    assert_eq!(sequential[2].lines[0].text, "雨");
    assert_eq!(sequential[3].lines.len(), 2);
    assert_eq!(sequential[4].lines[0].text, "図 書 館");
    let high = PageRecognizerOptions {
        min_line_confidence: 1.01,
        ..opts
    };
    assert!(
        runtime
            .recognize_page_rgb_u8(payload[1].0, payload[1].1, payload[1].2, high)
            .unwrap()
            .lines
            .is_empty()
    );
    assert!(
        runtime
            .recognize_page_rgb_u8(&[1, 2, 3], 4, 4, opts)
            .is_err()
    );
    assert_eq!(
        runtime
            .recognize_page_rgb_u8(payload[2].0, payload[2].1, payload[2].2, opts)
            .unwrap()
            .lines[0]
            .text,
        "雨"
    );
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_keeps_supported_repeated_thin_letters() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/page_band_fallback/images/repeated_i_gothic_png.png",
    ))
    .unwrap();
    let runtime = PageRecognizer::load(
        Path::new("models/deim-s-1024x1024.onnx"),
        Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
        1,
    )
    .unwrap();
    let page = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions::default(),
        )
        .unwrap();
    assert_eq!(page.lines.len(), 1);
    let text = &page.lines[0].text;
    assert!(text.chars().all(|c| c == 'I'));
    assert!((4..=6).contains(&text.chars().count()), "{text}");
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_rejects_unsupported_short_readings() {
    let runtime = PageRecognizer::load(
        Path::new("models/deim-s-1024x1024.onnx"),
        Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
        1,
    )
    .unwrap();
    for font in ["gothic", "mincho"] {
        for format in ["png", "jpeg75"] {
            let extension = if format == "png" { "png" } else { "jpg" };
            let name = format!(
                "tests/fixtures/page_band_fallback/images/single_ja_{font}_{format}.{extension}"
            );
            let image = crate::io::load_rgb_u8(Path::new(&name)).unwrap();
            let page = runtime
                .recognize_page_rgb_u8(
                    &image.data,
                    image.width,
                    image.height,
                    PageRecognizerOptions::default(),
                )
                .unwrap();
            assert!(page.lines.is_empty(), "{name}: {:?}", page.lines);
        }
    }
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/page_band_fallback/images/repeated_i_gothic_png.png",
    ))
    .unwrap();
    let page = primary100(1)
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions::default(),
        )
        .unwrap();
    assert!(
        page.lines.is_empty(),
        "unsupported I→一 must not be added: {:?}",
        page.lines
    );
}

#[test]
fn fallback_recognition_support_has_closed_finite_bounds() {
    let below = f32::from_bits(0.9f32.to_bits() - 1);
    let above = f32::from_bits(0.9f32.to_bits() + 1);
    for (value, expected) in [
        (below, false),
        (0.9, true),
        (above, true),
        (1.0, true),
        (f32::from_bits(1.0f32.to_bits() + 1), false),
        (f32::NAN, false),
        (f32::NEG_INFINITY, false),
        (f32::INFINITY, false),
        (0.0, false),
        (-1.0, false),
    ] {
        assert_eq!(supported_fallback_confidence(value), expected, "{value:?}");
    }
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_does_not_emit_unverified_overwide_model_prose() {
    let image =
        crate::io::load_rgb_u8(Path::new("tests/fixtures/dense_line/images/repeated_i.png"))
            .unwrap();
    let page = primary100(1)
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions::default(),
        )
        .unwrap();
    assert!(
        page.lines.is_empty(),
        "unsupported overwide inference: {:?}",
        page.lines
    );
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn page_band_fallback_recovers_native_overflow_uri_with_the_supplied_model() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/long_line_split/images/uri_mincho36.png",
    ))
    .unwrap();
    let truth =
        std::fs::read_to_string("tests/fixtures/long_line_split/truth/uri_mincho36.txt").unwrap();
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
        let page = runtime
            .recognize_page_rgb_u8(
                &image.data,
                image.width,
                image.height,
                PageRecognizerOptions::default(),
            )
            .unwrap();
        assert_eq!(page.lines.len(), 1, "{model}");
        assert_eq!(page.lines[0].text, truth, "{model}");
    }
}
