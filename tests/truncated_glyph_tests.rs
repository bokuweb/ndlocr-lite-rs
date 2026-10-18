use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 通常の文字高を持つ行と、行投影で下端が落ちる原寸の「せ」を配置する。
fn truncated_page(dx: usize, dy: usize) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/truncated_glyphs/se.png",
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

/// 強い画素行の外まで続く連結字形を途中で切らない。
#[test]
fn a_short_seed_preserves_the_connected_lower_strokes() {
    let (rgb, width, height) = truncated_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert!(bands[2][1] <= 198 && bands[2][3] >= 237, "{bands:?}");
    assert!((bands[2][2] - bands[2][0]) * 4 >= (bands[2][3] - bands[2][1]) * 5);
}

/// 復元した画像をそのまま推論し、文字置換なしで「せ」を返す。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn the_complete_glyph_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = truncated_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    let [x0, y0, x1, y1] = bands[2];
    let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
    assert_eq!(
        session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text,
        "せ"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 余白内の移動に追従し、両検出経路が同じ全高を返す。
    #[test]
    fn restored_bounds_follow_translation(dx in 0usize..500, dy in 0usize..30) {
        let (rgb, width, height) = truncated_page(dx, dy);
        let bands = detect_textline_bands_fast(&rgb, width, height, 220);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_naive(&rgb, width, height, 220));
        proptest::prop_assert_eq!(bands.len(), 3);
        proptest::prop_assert!(bands[2][1] <= 198 + dy && bands[2][3] >= 237 + dy);
    }
}

/// 通常字高2行と斜め画が欠ける4文字見出しを配置する。
fn diagonal_heading_page(dx: usize, dy: usize) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/diagonal_heading/normal.png",
    ))
    .unwrap();
    let (width, height) = (2480, 400);
    let mut rgb = vec![255; width * height * 3];
    for y in 0..image.height {
        let start = ((280 + dy + y) * width + 330 + dx) * 3;
        rgb[start..start + image.width * 3]
            .copy_from_slice(&image.data[y * image.width * 3..(y + 1) * image.width * 3]);
    }
    for top in [50, 130] {
        for y in top + dy..top + dy + 41 {
            rgb[(y * width + 354 + dx) * 3..(y * width + 754 + dx) * 3].fill(0);
        }
    }
    (rgb, width, height)
}

/// 見出しの行幅でも、元画素の斜め画を左・上・下とも欠落させない。
#[test]
fn diagonal_heading_retains_the_complete_leading_glyph() {
    let (rgb, width, height) = diagonal_heading_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
    assert_eq!(bands.len(), 3);
    let [x0, y0, x1, y1] = bands[2];
    assert!(
        x0 <= 360 && y0 <= 310 && x1 >= 385 && y1 >= 341,
        "{bands:?}"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 見出しを移動しても斜め画の被覆と両検出経路の一致を維持する。
    #[test]
    fn diagonal_heading_translation_preserves_ink(dx in 0usize..500,dy in 0usize..20) {
        let (rgb,width,height)=diagonal_heading_page(dx,dy);
        let bands=detect_textline_bands_fast(&rgb,width,height,220);
        proptest::prop_assert_eq!(&bands,&detect_textline_bands_naive(&rgb,width,height,220));
        proptest::prop_assert_eq!(bands.len(),3);
        let [x0,y0,x1,y1]=bands[2];
        proptest::prop_assert!(x0<=360+dx && y0<=310+dy && x1>=385+dx && y1>=341+dy);
    }
}

/// 斜め画の復元後、文字置換なしで短い見出しの読み順を保持する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn diagonal_heading_recognizes_in_source_order() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = diagonal_heading_page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    let [x0, y0, x1, y1] = bands[2];
    let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
    assert_eq!(
        session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text,
        "ノーマル"
    );
}
