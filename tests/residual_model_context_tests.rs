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

/// 回転される小区間を単語の文脈へ戻し、固定モデルで末尾記号を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn short_horizontal_word_fragment_keeps_its_context() {
    let (actual, count) = recognize_row(15);
    assert_eq!(count, 5, "the small fragment must rejoin its word");
    assert_eq!(
        actual,
        "Color [SlateBlue], background color Color [Orange]. The end."
    );
}

/// 各区間の正しい出力を実空白で連結し、元の5推論で全文を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn italic_punctuation_boundary_keeps_space() {
    let (actual, count) = recognize_row(6);
    assert_eq!(count, 5);
    assert_eq!(
        actual,
        "using font 'Times New Roman Bold Italic', font size 18 points,"
    );
}
