use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 通常行の字形高を基準に、未検出の短い実字形を元寸法で配置する。
fn page_with_short_tail() -> (Vec<u8>, usize, usize) {
    page_with_short_tail_at(0, 0)
}

/// 原寸字形を任意の余白位置へ平行移動する。
fn page_with_short_tail_at(dx: usize, dy: usize) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/short_lines/body_tail.png",
    ))
    .unwrap();
    let (width, height) = (2482, 280);
    let mut rgb = vec![255; width * height * 3];
    for y in 0..image.height {
        let i = ((y + 180 + dy) * width + 300 + dx) * 3;
        rgb[i..i + image.width * 3]
            .copy_from_slice(&image.data[y * image.width * 3..(y + 1) * image.width * 3]);
    }
    for top in [10, 90] {
        for y in top..top + 43 {
            paint(&mut rgb, width, 300, y, 400);
        }
    }
    (rgb, width, height)
}

/// 1画素行の指定区間を黒くする。
fn paint(rgb: &mut [u8], width: usize, x: usize, y: usize, len: usize) {
    rgb[(y * width + x) * 3..(y * width + x + len) * 3].fill(0);
}

/// row seedを満たさない字形と句点を全高のまま保持する。
#[test]
fn isolated_body_tail_is_preserved_at_page_scale() {
    let (rgb, width, height) = page_with_short_tail();
    let a = detect_textline_bands_naive(&rgb, width, height, 220);
    assert_eq!(a, detect_textline_bands_fast(&rgb, width, height, 220));
    assert_eq!(a.len(), 3, "{a:?}");
    assert!(a[2][0] <= 407 && a[2][2] >= 458, "{a:?}");
    assert!(a[2][1] <= 197 && a[2][3] >= 242, "{a:?}");
    assert!(a[2][3] - a[2][1] <= 49, "{a:?}");
}

/// 小点、細線、密な矩形は高さがあっても文字seedに昇格させない。
#[test]
fn isolated_noise_is_not_promoted_to_a_line() {
    let (mut rgb, width, height) = page_with_short_tail();
    rgb[180 * width * 3..].fill(255);
    for y in 190..233 {
        paint(&mut rgb, width, 300, y, 2);
        paint(&mut rgb, width, 400, y, 12);
    }
    for y in 190..194 {
        paint(&mut rgb, width, 500, y, 4);
    }
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        assert_eq!(detect(&rgb, width, height, 220).len(), 2);
    }
}

/// 離れた小点と別の行を短い字形へ結合しない。
#[test]
fn isolated_tail_does_not_absorb_distant_dots_or_other_rows() {
    let (mut rgb, width, height) = page_with_short_tail();
    for y in 250..254 {
        paint(&mut rgb, width, 410, y, 4);
    }
    for y in 231..235 {
        paint(&mut rgb, width, 600, y, 4);
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert!(bands[2][2] < 500 && bands[2][3] < 250, "{bands:?}");
}

/// 孤立字形が通常行の高密度seedを持たなくても実モデルへ届く。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn isolated_body_tail_reaches_real_recognition() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    let (rgb, width, height) = page_with_short_tail();
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    let [x0, y0, x1, y1] = *bands.get(2).expect("short tail required");
    let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
    let actual = session
        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
        .unwrap()
        .text;
    assert_eq!(actual, "う。");
}

/// 字形に近い塗り潰し矩形も句読点として取り込まない。
#[test]
fn a_nearby_solid_block_is_not_attached_to_the_tail() {
    let (mut rgb, width, height) = page_with_short_tail();
    for y in 215..242 {
        paint(&mut rgb, width, 465, y, 12);
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert!(bands[2][2] <= 460, "{bands:?}");
}

/// ノイズ成分が上限を超える領域では補助検出を打ち切る。
#[test]
fn excessive_components_disable_the_fallback() {
    let (mut rgb, width, height) = page_with_short_tail();
    for n in 0..257 {
        paint(&mut rgb, width, 10 + n % 20 * 3, 150 + n / 20 * 3, 1);
    }
    assert_eq!(
        detect_textline_bands_fast(&rgb, width, height, 220).len(),
        2
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    /// 余白内の位置が変わっても元字形の範囲と両経路の一致を保つ。
    #[test]
    fn isolated_tail_coordinates_follow_translation(dx in 0usize..1000, dy in 0usize..20) {
        let (rgb, width, height) = page_with_short_tail_at(dx, dy);
        let bands = detect_textline_bands_fast(&rgb, width, height, 220);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_naive(&rgb, width, height, 220));
        proptest::prop_assert_eq!(bands.len(), 3);
        proptest::prop_assert_eq!(bands[2], [405 + dx, 196 + dy, 460 + dx, 243 + dy]);
    }
}

/// 同じ高さに複数の疎な字形seedがある場合、後続字形を捨てない。
#[test]
fn all_sparse_glyph_seeds_on_the_same_line_are_kept() {
    let (mut rgb, width, height) = page_with_short_tail();
    rgb[180 * width * 3..].fill(255);
    for x in [300, 375, 450] {
        for y in 0..43 {
            paint(&mut rgb, width, x + y / 2, 190 + y, 3);
        }
    }
    let bands = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert!(bands[2][0] <= 300 && bands[2][2] >= 474, "{bands:?}");
}
