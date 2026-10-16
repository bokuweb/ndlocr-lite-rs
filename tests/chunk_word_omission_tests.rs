#![cfg(all(feature = "onnx", unix))]
use ndlocr_lite_rs::{
    infer::cached::ParseqSession,
    io::load_rgb_u8,
    pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
};
use std::path::{Path, PathBuf};

/// 左右の純白余白により中央の単語を欠落させず、上下寸法を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn horizontal_margin_does_not_hide_middle_word() {
    let model = PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let session = ParseqSession::load(&model, &charset).unwrap();
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/chunk_word_omission/article2-band.png",
    ))
    .unwrap();
    let chunks =
        split_line_for_input(&image.data, image.width, image.height, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 7);
    let crop = crop_line_chunk_rgb_u8(&image.data, image.width, image.height, &chunks, 2).unwrap();
    assert_eq!(crop.height, image.height);
    assert_eq!(crop.width, 231);
    let result = session
        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
        .unwrap();
    assert_eq!(result.text, "the Code of");
}
