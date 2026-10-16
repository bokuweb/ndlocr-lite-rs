use ndlocr_lite_rs::{
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    },
};

/// 混在書式の原寸行の実単語間隔を、chunk連結で消さない。
#[test]
fn proportional_word_boundaries_keep_their_space_attribute() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let [x0, y0, x1, y1] = bands[0];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    assert_eq!(
        chunks.iter().map(|c| (c.x0, c.x1)).collect::<Vec<_>>(),
        [
            (0, 141),
            (141, 520),
            (520, 735),
            (735, 986),
            (986, 1258),
            (1258, 1556),
            (1556, 1875)
        ]
    );
    for x in [520, 986] {
        assert!(
            chunks
                .iter()
                .any(|chunk| chunk.x0 == x && chunk.space_before),
            "{chunks:?}"
        );
    }
}

/// 斜体の文字内分割は連結し、実単語境界だけに空白を付ける。
#[test]
fn italic_boundaries_keep_spaces_without_separating_joined_letters() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/independent_chromatic_formatting/region.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let [x0, y0, x1, y1] = bands[0];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    assert_eq!(
        chunks
            .iter()
            .map(|c| (c.x0, c.x1, c.space_before))
            .collect::<Vec<_>>(),
        [
            (0, 270, false),
            (270, 571, false),
            (571, 878, false),
            (878, 1214, true),
            (1214, 1451, true)
        ]
    );
}

/// 混在書式の原寸行で、認識区間間の実単語空白を保持する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn mixed_format_word_spaces_are_not_dropped_between_chunks() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            background::normalize_chromatic_panels,
            crop::{BBox, crop_rgb_u8},
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
            line_segment::detect_textline_bands_naive,
        },
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let session = ParseqSession::load(
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let [x0, y0, x1, y1] = bands[0];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    let mut actual = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let crop =
            crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, index).unwrap();
        let result = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        eprintln!(
            "CHUNK {chunk:?} {}x{} {:?}",
            crop.width, crop.height, result.text
        );
        if chunk.insert_space_before(&actual, &result.text, line.height) {
            actual.push(' ');
        }
        actual.push_str(&result.text);
    }
    assert!(actual.contains("drawn using"), "{actual}");
    assert!(actual.contains("New Roman"), "{actual}");
}

/// 独立資料の斜体行の空白欠落を、固定モデルとchunk別出力で固定する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn italic_line_keeps_original_word_spaces() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession, pipeline::line_chunks::crop_line_chunk_rgb_u8,
    };
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/independent_chromatic_formatting/region.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let [x0, y0, x1, y1] = bands[0];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    let session = ParseqSession::load(
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let mut actual = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let crop =
            crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, index).unwrap();
        let result = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        eprintln!(
            "ITALIC {chunk:?} {}x{} {:?}",
            crop.width, crop.height, result.text
        );
        if chunk.insert_space_before(&actual, &result.text, line.height) {
            actual.push(' ');
        }
        actual.push_str(&result.text);
    }
    assert_eq!(
        actual,
        "3. Using TextFormat.FontStyle.BoldItalic to emulate bold italic style."
    );
}
