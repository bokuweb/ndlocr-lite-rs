use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 原寸の薄い行端字形を切り出す。
fn fixture(name: &str) -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&format!(
        "tests/fixtures/sparse_edges/{name}.png"
    )))
    .unwrap()
}

/// モデルへ渡る前に「へ」「二」を左右の刈込で消さない。
#[test]
fn sparse_leading_glyph_pixels_are_retained() {
    for name in ["phone", "delegation"] {
        let im = fixture(name);
        let a = detect_textline_bands_naive(&im.data, im.width, im.height, 220);
        assert_eq!(
            a,
            detect_textline_bands_fast(&im.data, im.width, im.height, 220)
        );
        assert_eq!(a.len(), 1, "{name}: {a:?}");
        assert!(a[0][0] <= 53, "missing leading glyph {name}: {a:?}");
    }
}

/// 水平反転しても右端の疎な字形を保持する。
#[test]
fn sparse_trailing_glyph_pixels_are_retained() {
    let im = fixture("phone");
    let mirrored: Vec<u8> = im
        .data
        .chunks_exact(im.width * 3)
        .flat_map(|row| row.chunks_exact(3).rev().flatten().copied())
        .collect();
    let bands = detect_textline_bands_fast(&mirrored, im.width, im.height, 220);
    assert_eq!(bands.len(), 1);
    assert!(bands[0][2] >= im.width - 53, "{bands:?}");
    assert_eq!(
        bands,
        detect_textline_bands_naive(&mirrored, im.width, im.height, 220)
    );
}

/// 行高に対して離れた余白の字形を本文へ含めない。
#[test]
fn sparse_glyph_beyond_the_search_distance_is_ignored() {
    let im = fixture("phone");
    let width = im.width + 100;
    let mut rgb = vec![255; width * im.height * 3];
    for y in 0..im.height {
        let source = &im.data[y * im.width * 3..(y + 1) * im.width * 3];
        rgb[(y * width) * 3..(y * width + 120) * 3].copy_from_slice(&source[..120 * 3]);
        rgb[(y * width + 220) * 3..(y * width + width) * 3].copy_from_slice(&source[120 * 3..]);
    }
    let bands = detect_textline_bands_fast(&rgb, width, im.height, 220);
    assert_eq!(bands[0][0], 235);
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 余白内の平行移動に対して画素保持と両実装の一致を保つ。
    #[test]
    fn sparse_edge_bounds_translate(dx in 0usize..100, dy in 0usize..20) {
        let im = fixture("phone");
        let (width,height) = (im.width + dx, im.height + dy);
        let mut rgb = vec![255; width * height * 3];
        for y in 0..im.height {
            let offset = ((y + dy) * width + dx) * 3;
            rgb[offset..offset + im.width * 3].copy_from_slice(&im.data[y * im.width * 3..(y + 1) * im.width * 3]);
        }
        let a = detect_textline_bands_naive(&rgb,width,height,220);
        proptest::prop_assert_eq!(&a, &detect_textline_bands_fast(&rgb,width,height,220));
        proptest::prop_assert_eq!(a[0], [53 + dx, 8 + dy, 326 + dx, 53 + dy]);
    }
}

/// 刈込で消えていた先頭仮名が固定PARSeqモデルへ到達する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn sparse_phone_label_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let im = fixture("phone");
    let [x0, y0, x1, y1] = detect_textline_bands_fast(&im.data, im.width, im.height, 220)[0];
    let crop = crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let actual = ParseqSession::load(&model, &charset)
        .unwrap()
        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
        .unwrap()
        .text;
    // 仮名の同形誤認識は残る。元の正解「へ」はsource.jsonと全文CERで維持する。
    assert_eq!(actual, "ヘ電話番号");
}
