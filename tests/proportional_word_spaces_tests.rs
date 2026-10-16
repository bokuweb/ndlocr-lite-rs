use ndlocr_lite_rs::{
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    },
};

/// 原寸の残る2境界では、分割座標を維持し実空白だけを復元する。
#[test]
fn mixed_format_residual_boundaries_keep_their_space_attribute() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    for (row, positions, spaces) in [
        (
            5,
            vec![0, 483, 958, 1285, 1944],
            vec![false, true, false, true],
        ),
        (
            13,
            vec![0, 189, 744, 950, 1441, 1907],
            vec![false, true, false, true, true],
        ),
    ] {
        let [x0, y0, x1, y1] = bands[row];
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks =
            split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
        assert_eq!(
            chunks
                .iter()
                .map(|c| c.x0)
                .chain([line.width])
                .collect::<Vec<_>>(),
            positions
        );
        assert_eq!(
            chunks.iter().map(|c| c.space_before).collect::<Vec<_>>(),
            spaces
        );
    }
}

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

/// 書式混在行の残る空白欠落を、区間別推論と連結属性から分離する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn residual_formatted_word_spaces_are_preserved() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession, pipeline::line_chunks::crop_line_chunk_rgb_u8,
    };
    let image = load_rgb_u8(std::path::Path::new(
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
    let mut missing = Vec::new();
    for (row, expected) in [
        (5, "background color Color"),
        (6, "font size 18 points"),
        (13, "bold italic text"),
    ] {
        let [x0, y0, x1, y1] = bands[row];
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks =
            split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
        let mut actual = String::new();
        for (index, chunk) in chunks.iter().enumerate() {
            let crop = crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, index)
                .unwrap();
            let result = session
                .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                .unwrap();
            eprintln!(
                "ROW {row} {chunk:?} {}x{} {:?}",
                crop.width, crop.height, result.text
            );
            if chunk.insert_space_before(&actual, &result.text, line.height) {
                actual.push(' ');
            }
            actual.push_str(&result.text);
        }
        eprintln!("ROW {row} ACTUAL {actual:?}");
        if !actual.contains(expected) {
            missing.push((row, expected, actual));
        }
    }
    assert!(missing.is_empty(), "{missing:?}");
}

/// 斜体の数詞後の実空白を、元の区間とモデル入力を維持して復元する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn italic_number_word_space_is_preserved() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession, pipeline::line_chunks::crop_line_chunk_rgb_u8,
    };
    let image = load_rgb_u8(std::path::Path::new(
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
    let row = 14;
    let [x0, y0, x1, y1] = bands[row];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    assert_eq!(
        chunks.iter().map(|c| (c.x0, c.x1)).collect::<Vec<_>>(),
        [(0, 135), (135, 552), (552, 987), (987, 1351), (1351, 1822)],
        "original partition must remain unchanged"
    );
    assert!(chunks[4].space_before);
    let mut actual = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let crop =
            crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, index).unwrap();
        assert_eq!(crop.width, [135, 417, 435, 364, 471][index]);
        let result = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        eprintln!(
            "ROW {row} {chunk:?} {}x{} {:?}",
            crop.width, crop.height, result.text
        );
        if chunk.insert_space_before(&actual, &result.text, line.height) {
            actual.push(' ');
        }
        actual.push_str(&result.text);
    }
    eprintln!("ROW {row} ACTUAL {actual:?}");
    assert!(actual.contains("16 points"), "{actual:?}");
}
