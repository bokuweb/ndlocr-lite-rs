use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 原寸の仮名内部に狭い白列を持つ行を読み込む。
fn fixture() -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/internal_glyph_gaps/efficiency.png",
    ))
    .unwrap()
}

/// 「が」の左右画線を別の認識入力へ分けない。
#[test]
fn kana_internal_gap_keeps_complete_glyph() {
    let im = fixture();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    assert!(
        chunks.iter().any(|c| c.x0 <= 950 && c.x1 >= 990),
        "split glyph: {chunks:?}"
    );
}

/// 字形を保持した同じ固定モデルの入力で、濁点付き仮名を認識する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn complete_kana_reaches_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    use std::path::PathBuf;
    let im = fixture();
    let session = ParseqSession::load(
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let (width, height) = session.input_dimensions();
    let mut actual = String::new();
    for chunk in split_line_for_input(&im.data, im.width, im.height, 220, width, height).unwrap() {
        let crop = crop_rgb_u8(
            &im.data,
            im.width,
            im.height,
            BBox::new(chunk.x0, 0, chunk.x1, im.height),
        )
        .unwrap();
        let text = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text;
        eprintln!("{chunk:?}: {text}");
        if chunk.insert_space_before(&actual, &text, im.height) {
            actual.push(' ');
        }
        actual.push_str(&text);
    }
    assert!(
        actual.contains("でメモリ効率が高く"),
        "incomplete kana: {actual}"
    );
}
