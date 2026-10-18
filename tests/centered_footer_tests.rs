use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 原寸の頁番号を中央へ置き、本文2行の文字高を与える。
fn page(dx: usize, dy: usize, footer_x: usize, footer_y: usize) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/centered_footer/page_one.png",
    ))
    .unwrap();
    let (width, height) = (2482 + dx * 2, 500 + dy);
    let mut rgb = vec![255; width * height * 3];
    for y in 0..image.height {
        let start = ((footer_y + dy + y) * width + footer_x + dx) * 3;
        rgb[start..start + image.width * 3]
            .copy_from_slice(&image.data[y * image.width * 3..(y + 1) * image.width * 3]);
    }
    for top in [10, 90] {
        for y in top + dy..top + dy + 43 {
            rgb[(y * width + 300 + dx) * 3..(y * width + 700 + dx) * 3].fill(0);
        }
    }
    (rgb, width, height)
}

/// 本文より小さい中央頁番号を、画素を欠けさせず横書き余白付きで検出する。
#[test]
fn small_centered_footer_is_preserved() {
    let (rgb, width, height) = page(0, 0, 1226, 400);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
    assert_eq!(bands.len(), 3, "{bands:?}");
    let [x0, y0, x1, y1] = bands[2];
    assert!(x0 <= 1236 && x1 >= 1245 && y0 <= 410 && y1 >= 434);
    assert_eq!(y1 - y0, 26);
    assert!((x1 - x0) * 4 >= (y1 - y0) * 5);
}

/// 中央から外れた字形、本文付近、ヘッダー、下端に接する字形を昇格させない。
#[test]
fn small_glyphs_outside_the_footer_context_stay_excluded() {
    for (x, y) in [(1100, 400), (1226, 170), (1226, 0), (1226, 456)] {
        let (rgb, width, height) = page(0, 0, x, y);
        for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
            assert_eq!(detect(&rgb, width, height, 220).len(), 2, "{x},{y}");
        }
    }
}

/// 中央にあっても点・細線・密な矩形は文字候補にしない。
#[test]
fn centered_noise_stays_excluded() {
    let (mut rgb, width, height) = page(0, 0, 1226, 400);
    rgb[200 * width * 3..].fill(255);
    for (top, w, h) in [(250, 4, 4), (320, 2, 24), (400, 12, 24)] {
        for y in top..top + h {
            rgb[(y * width + 1240) * 3..(y * width + 1240 + w) * 3].fill(0);
        }
    }
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        assert_eq!(detect(&rgb, width, height, 220).len(), 2);
    }
}

/// 中央距離と上下の安全余白は等号を含む境界までだけ許可する。
#[test]
fn footer_context_limits_are_inclusive_and_bounded() {
    for (x, y, count) in [
        (1248, 400, 3),
        (1249, 400, 2),
        (1205, 400, 3),
        (1204, 400, 2),
        (1226, 210, 3),
        (1226, 209, 2),
        (1226, 423, 3),
        (1226, 424, 2),
    ] {
        let (rgb, width, height) = page(0, 0, x, y);
        for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
            assert_eq!(detect(&rgb, width, height, 220).len(), count, "{x},{y}");
        }
    }
}

/// 本文の文字高を支える行が1本だけなら、新たなフッター判定を使わない。
#[test]
fn a_single_body_line_does_not_enable_footer_recovery() {
    let (mut rgb, width, height) = page(0, 0, 1226, 400);
    rgb[..60 * width * 3].fill(255);
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        assert_eq!(detect(&rgb, width, height, 220).len(), 1);
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 左右等量と上側の余白を増やすと、復元境界も同量だけ移動する。
    #[test]
    fn footer_coordinates_follow_page_translation(dx in 0usize..150, dy in 0usize..30) {
        let (rgb, width, height) = page(dx, dy, 1226, 400);
        let bands = detect_textline_bands_fast(&rgb, width, height, 220);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_naive(&rgb, width, height, 220));
        proptest::prop_assert_eq!(bands.len(), 3);
        let (base, w, h) = page(0, 0, 1226, 400);
        let original = detect_textline_bands_fast(&base, w, h, 220)[2];
        proptest::prop_assert_eq!(bands[2], [original[0]+dx, original[1]+dy, original[2]+dx, original[3]+dy]);
    }
}

/// 復元した頁番号を固定PARSeqへ渡す。原文1の11へのモデル誤読は残る。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn small_footer_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = page(0, 0, 1226, 400);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 3);
    let [x0, y0, x1, y1] = bands[2];
    let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
    assert_eq!(
        session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text,
        "11"
    );
}
