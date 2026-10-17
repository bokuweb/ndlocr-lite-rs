use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 字間に実空白を持たない原寸の小書き仮名入り行を読み込む。
fn fixture() -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/small_kana_bearings/thread.png",
    ))
    .unwrap()
}

/// 小書き仮名の左右余白を単語空白と誤判定しない。
#[test]
fn small_kana_bearings_do_not_insert_word_spaces() {
    let image = fixture();
    let chunks =
        split_line_for_input(&image.data, image.width, image.height, 220, 256, 24).unwrap();
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.gap_before != 25 || !chunk.space_before),
        "small kana bearing must not become a preferred word-space boundary: {chunks:?}"
    );
}

/// 診断として各認識入力とそのままのモデル本文を記録する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn small_kana_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let im = fixture();
    let mut actual = String::new();
    for chunk in split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap() {
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
    assert_eq!(
        actual,
        "スレッド安全性が保証されます。さらに様々な種類のバグをコンパイ"
    );
}
