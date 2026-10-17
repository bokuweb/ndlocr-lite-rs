use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 通常行の高いseedと、原寸で分断する単独漢字を同じページへ配置する。
fn fragmented_page(dx: usize, dy: usize) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/fragmented_glyphs/ki.png",
    ))
    .unwrap();
    let (width, height) = (2482, 320);
    let mut rgb = vec![255; width * height * 3];
    for y in 0..image.height {
        let i = ((y + 180 + dy) * width + 1190 + dx) * 3;
        rgb[i..i + image.width * 3]
            .copy_from_slice(&image.data[y * image.width * 3..(y + 1) * image.width * 3]);
    }
    for top in [10, 90] {
        for y in top..top + 44 {
            rgb[(y * width + 300) * 3..(y * width + 700) * 3].fill(0);
        }
    }
    (rgb, width, height)
}

/// 細い縦画の低密度部分で分断された漢字を、全高でモデルへ渡す。
#[test]
fn a_connected_isolated_glyph_is_not_split_into_two_rows() {
    let (rgb, width, height) = fragmented_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert!(bands[2][0] <= 1218 && bands[2][2] >= 1264, "{bands:?}");
    assert!(bands[2][1] <= 204 && bands[2][3] >= 248, "{bands:?}");
}

/// 上下の短い文字行が実際にはつながっていなければ結合しない。
#[test]
fn separate_short_rows_stay_separate() {
    let (mut rgb, width, height) = fragmented_page(0, 0);
    rgb[180 * width * 3..].fill(255);
    for top in [190, 222] {
        for y in top..top + 12 {
            rgb[(y * width + 300) * 3..(y * width + 344) * 3].fill(0);
        }
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 4, "{bands:?}");
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
}

/// 断片を貫く罫線は、候補範囲外まで続くため字形の接続と見なさない。
#[test]
fn a_rule_crossing_separate_rows_does_not_join_them() {
    let (mut rgb, width, height) = fragmented_page(0, 0);
    rgb[180 * width * 3..].fill(255);
    for top in [190, 222] {
        for y in top..top + 12 {
            rgb[(y * width + 300) * 3..(y * width + 344) * 3].fill(0);
        }
    }
    for y in 180..255 {
        rgb[(y * width + 320) * 3..(y * width + 322) * 3].fill(0);
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 4, "{bands:?}");
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
}

/// 元の文字を全高で渡すと、文章固有の置換なしで認識できる。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn the_rejoined_glyph_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = fragmented_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 3);
    let [x0, y0, x1, y1] = bands[2];
    let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
    assert_eq!(
        session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text,
        "記"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 余白内の平行移動でも字形全体を保ち、両検出経路が一致する。
    #[test]
    fn rejoined_bounds_follow_translation(dx in 0usize..500, dy in 0usize..30) {
        let (rgb, width, height) = fragmented_page(dx, dy);
        let bands = detect_textline_bands_fast(&rgb, width, height, 220);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_naive(&rgb, width, height, 220));
        proptest::prop_assert_eq!(bands.len(), 3);
        proptest::prop_assert!(bands[2][0] <= 1218 + dx && bands[2][2] >= 1264 + dx);
        proptest::prop_assert!(bands[2][1] <= 204 + dy && bands[2][3] >= 248 + dy);
    }
}
