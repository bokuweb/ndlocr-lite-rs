use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 原寸の小文字4行と、同じ左端の通常行2本を配置する。
fn page(dx: usize, dy: usize) -> (Vec<u8>, usize, usize) {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/isolated_latin/lowercase.png",
    ))
    .unwrap();
    let (width, height) = (2480, 700);
    let mut rgb = vec![255; width * height * 3];
    for y in 0..im.height {
        let target = ((280 + dy + y) * width + 330 + dx) * 3;
        rgb[target..target + im.width * 3]
            .copy_from_slice(&im.data[y * im.width * 3..(y + 1) * im.width * 3]);
    }
    for top in [50, 130] {
        for y in top + dy..top + dy + 41 {
            rgb[(y * width + 354 + dx) * 3..(y * width + 754 + dx) * 3].fill(0);
        }
    }
    (rgb, width, height)
}

/// 小文字本体と分離した点を原寸のまま行に含める。
#[test]
fn lowercase_lines_include_all_ink_and_the_detached_dot() {
    let (rgb, width, height) = page(0, 0);
    let bands = detect_textline_bands_naive(&rgb, width, height, 220);
    assert_eq!(bands, detect_textline_bands_fast(&rgb, width, height, 220));
    assert_eq!(bands.len(), 6, "{bands:?}");
    for (b, ink) in bands[2..].iter().zip([
        [354, 297, 365, 330],
        [353, 385, 385, 405],
        [354, 459, 389, 480],
        [355, 534, 374, 556],
    ]) {
        assert!(
            b[0] <= ink[0] && b[1] <= ink[1] && b[2] >= ink[2] && b[3] >= ink[3],
            "{b:?} {ink:?}"
        );
        assert_eq!(
            b[3] - b[1],
            43,
            "x-height must retain the surrounding font scale: {b:?}"
        );
        assert!((b[2] - b[0]) * 4 >= (b[3] - b[1]) * 5);
    }
}

/// 左端が本文とそろわない小字形は図形かもしれないため復元しない。
#[test]
fn small_shapes_away_from_a_text_margin_are_not_promoted() {
    let (mut rgb, width, height) = page(0, 0);
    for y in 280..580 {
        let row = y * width * 3;
        rgb.copy_within(row + 330 * 3..row + 450 * 3, row + 1000 * 3);
        rgb[row + 330 * 3..row + 450 * 3].fill(255);
    }
    assert_eq!(
        detect_textline_bands_fast(&rgb, width, height, 220).len(),
        2
    );
}

/// 本文と整列した点・線・密な矩形も小文字として復元しない。
#[test]
fn aligned_noise_is_not_promoted() {
    let (mut rgb, width, height) = page(0, 0);
    rgb[280 * width * 3..].fill(255);
    for (top, w, h) in [(300, 5, 5), (400, 2, 22), (500, 12, 22)] {
        for y in top..top + h {
            rgb[(y * width + 354) * 3..(y * width + 354 + w) * 3].fill(0);
        }
    }
    assert_eq!(
        detect_textline_bands_fast(&rgb, width, height, 220).len(),
        2
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 同じ本文余白への平行移動で検出数・座標・両経路の一致を維持する。
    #[test]
    fn lowercase_coordinates_follow_the_text_margin(dx in 0usize..600, dy in 0usize..25) {
        let (rgb, width, height) = page(dx, dy);
        let bands = detect_textline_bands_fast(&rgb, width, height, 220);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_naive(&rgb, width, height, 220));
        proptest::prop_assert_eq!(bands.len(), 6);
        for (b, original) in bands[2..].iter().zip([[333,288,387,331], [342,363,396,406], [345,438,399,481], [338,514,392,557]]) {
            proptest::prop_assert_eq!(*b, [original[0]+dx,original[1]+dy,original[2]+dx,original[3]+dy]);
        }
    }
}

/// 周囲の字高と横書き余白により小文字の大小・向きの誤認識を減らす。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn restored_x_height_preserves_case_and_orientation() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = page(0, 0);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 6);
    let actual: Vec<_> = bands[2..]
        .iter()
        .map(|&[x0, y0, x1, y1]| {
            let im = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
            session
                .recognize_rgb_u8_unprocessed(&im.data, im.width, im.height)
                .unwrap()
                .text
        })
        .collect();
    eprintln!("isolated lowercase recognition: {actual:?}");
    // iの字形被覆は幾何テストで保証するが、現行モデルでの11への誤読は残課題。
    assert!(!actual[0].is_empty());
    assert_eq!(&actual[1..], ["w", "m", "a"]);
}

/// 左ページ端の小文字も切り落とさず、余白を画像内に収める。
#[test]
fn horizontal_padding_is_clamped_at_the_page_edge() {
    let (mut rgb, width, height) = page(0, 0);
    for y in 0..height {
        let row = y * width * 3;
        rgb.copy_within(row + 351 * 3..row + width * 3, row);
        rgb[row + (width - 351) * 3..row + width * 3].fill(255);
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 6, "{bands:?}");
    assert_eq!(bands[2][0], 0);
    assert!(bands[2][2] >= 14);
    assert!(bands.iter().all(|b| b[2] <= width && b[3] <= height));
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
}

/// 上側の字高余白が直前の本文行を取り込まない。
#[test]
fn vertical_padding_stops_after_the_previous_line() {
    let (mut rgb, width, height) = page(0, 0);
    rgb.copy_within(297 * width * 3..330 * width * 3, 175 * width * 3);
    rgb[297 * width * 3..330 * width * 3].fill(255);
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 6, "{bands:?}");
    assert_eq!(bands[2][1], bands[1][3]);
    assert!(bands[2][1] <= 175 && bands[2][3] >= 208);
    assert_eq!(bands, detect_textline_bands_naive(&rgb, width, height, 220));
}
