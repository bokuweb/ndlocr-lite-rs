#![cfg(all(feature = "onnx", unix))]
use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8_tight_horizontal};
use ndlocr_lite_rs::{infer::cached::ParseqSession, io::load_rgb_u8};
use std::path::{Path, PathBuf};

/// 画線が残る原寸chunkで、引用符と数字の生出力を固定する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn chunk_margins_preserve_quotes_without_extra_digits() {
    let model = PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let session = ParseqSession::load(&model, &charset).unwrap();
    for (name, expected) in [
        ("purpose", "general-purpose"),
        ("borrow", "races, its \"borrow"),
        ("year", "project in 2009"),
    ] {
        let im = load_rgb_u8(Path::new(&format!(
            "tests/fixtures/chunk_margins/{name}.png"
        )))
        .unwrap();
        let crop = crop_rgb_u8_tight_horizontal(
            &im.data,
            im.width,
            im.height,
            BBox::new(0, 0, im.width, im.height),
        )
        .unwrap();
        let result = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        assert_eq!(result.text, expected, "{name}");
    }
}
