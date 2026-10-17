#![cfg(all(feature = "onnx", unix))]

use ndlocr_lite_rs::{
    infer::cached::ParseqSession,
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
        line_segment::detect_textline_bands_naive,
    },
};

/// 固定モデルで一行を区間ごとに認識し、連結結果と推論数を返す。
fn recognize_row(row: usize) -> (String, usize) {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/font_emulation_context/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let session = ParseqSession::load(
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let [x0, y0, x1, y1] = bands[row];
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
            "ROW {row} CHUNK {index} {chunk:?} {}x{} confidence={} {:?}",
            crop.width, crop.height, result.mean_confidence, result.text
        );
        if chunk.insert_space_before(&actual, &result.text, line.height) {
            actual.push(' ');
        }
        actual.push_str(&result.text);
    }
    eprintln!("ROW {row} ACTUAL {actual:?}");
    (actual, chunks.len())
}

/// 実書体・擬似書体9行の全文を保ち、元の32推論より増やさない。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn emulated_fonts_keep_words_and_spaces() {
    let truth = include_str!("fixtures/font_emulation_context/truth.txt");
    assert_eq!(truth.trim_end_matches('\n').chars().count(), 427);
    assert_eq!(truth.lines().count(), 9);
    let mut total = 0;
    for (row, expected) in truth.lines().enumerate() {
        let (actual, count) = recognize_row(row);
        assert_eq!(actual, expected, "source row {row}");
        total += count;
    }
    assert_eq!(total, 32, "preserve the original model invocation count");
}
