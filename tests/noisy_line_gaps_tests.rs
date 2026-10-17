use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

/// 塗り潰し画素を書き、検出用の合成文字と汚れを作る。
fn rect(rgb: &mut [u8], width: usize, bounds: [usize; 4]) {
    let [x0, y0, x1, y1] = bounds;
    for y in y0..y1 {
        rgb[(y * width + x0) * 3..(y * width + x1) * 3].fill(0);
    }
}

/// 正常な2行と、行間の細かい汚れで結合される3行を返す。
fn page(scale: usize, offset: usize, noise_shift: usize, bridge: bool) -> (Vec<u8>, usize, usize) {
    let (width, height) = (400 * scale, 250 * scale + offset);
    let mut rgb = vec![255; width * height * 3];
    for base in [10, 55, 100, 145, 190] {
        for x in (40..280).step_by(30) {
            rect(
                &mut rgb,
                width,
                [
                    x * scale,
                    base * scale + offset,
                    (x + 20) * scale,
                    (base + 30) * scale + offset,
                ],
            );
        }
    }
    for base in [130, 175] {
        for y in base..base + 15 {
            let x = if bridge {
                50
            } else {
                10 + (y + noise_shift) % 4 * 15
            };
            rect(
                &mut rgb,
                width,
                [
                    x * scale,
                    y * scale + offset,
                    (x + 8) * scale,
                    (y + 1) * scale + offset,
                ],
            );
        }
    }
    (rgb, width, height)
}

/// 物理的には独立した5行を、小さな行間汚れだけで結合しない。
#[test]
fn small_disconnected_gap_noise_does_not_merge_lines() {
    let (rgb, w, h) = page(1, 0, 0, false);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands.len(), 5, "{bands:?}");
    assert_eq!(bands, detect_textline_bands_fast(&rgb, w, h, 220));
}

/// 中央が細くても上下を結ぶ筆画がある領域を別の行へ切断しない。
#[test]
fn connected_vertical_strokes_are_not_split_at_density_valleys() {
    let (rgb, w, h) = page(1, 0, 0, true);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands.len(), 3, "{bands:?}");
    assert_eq!(bands, detect_textline_bands_fast(&rgb, w, h, 220));
}

/// 既存スキャンの本文5行と見出し2行を分離する。
#[test]
fn scanned_page_preserves_all_seven_physical_lines() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/scaned0.png",
    ))
    .unwrap();
    let bands = detect_textline_bands_naive(&image.data, image.width, image.height, 220);
    assert_eq!(bands.len(), 7, "{bands:?}");
    assert_eq!(
        bands,
        detect_textline_bands_fast(&image.data, image.width, image.height, 220)
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(16))]
    /// 拡大と移動、汚れの水平位置によらず本文の行を保つ。
    #[test]
    fn scaled_noise_keeps_separate_lines(scale in 1usize..4, offset in 0usize..20, shift in 0usize..8) {
        let (rgb, w, h) = page(scale, offset, shift, false);
        let bands = detect_textline_bands_naive(&rgb, w, h, 220);
        proptest::prop_assert_eq!(bands.len(), 5, "{:?}", bands);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_fast(&rgb, w, h, 220));
        for (i, b) in bands.iter().enumerate() {
            let y = [10,55,100,145,190][i] * scale + offset;
            proptest::prop_assert!(b[1] <= y && b[3] >= y + 30 * scale);
            proptest::prop_assert!(b[0] <= 40 * scale && b[2] >= 270 * scale);
            proptest::prop_assert!(b[2] <= w && b[3] <= h);
        }
    }
}

/// 比較できる通常行がないときは、疎な谷だけを根拠に分割しない。
#[test]
fn no_reference_lines_keeps_original_region() {
    let (mut rgb, w, h) = page(1, 0, 0, false);
    rgb[..w * 95 * 3].fill(255);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands.len(), 1, "{bands:?}");
    assert_eq!(bands, detect_textline_bands_fast(&rgb, w, h, 220));
}

/// 拡大した連結筆画も低密度の谷で切断しない。
#[test]
fn scaled_connected_strokes_remain_connected() {
    for scale in 1..4 {
        let (rgb, w, h) = page(scale, 7, 0, true);
        let bands = detect_textline_bands_naive(&rgb, w, h, 220);
        assert_eq!(bands.len(), 3, "scale={scale} {bands:?}");
        assert_eq!(bands, detect_textline_bands_fast(&rgb, w, h, 220));
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(16))]
    /// 短い横画は拡大・移動や汚れ位置によらず本文から分割しない。
    #[test]
    fn short_glyph_fragment_does_not_become_a_new_line(
        scale in 1usize..4, offset in 0usize..20, shift in 0usize..8
    ) {
        let (width, height) = (400 * scale, 210 * scale + offset);
        let mut rgb = vec![255; width * height * 3];
        for [x0, y0, x1, y1] in [[40, 10, 280, 40], [40, 55, 280, 85], [100, 100, 200, 115], [40, 145, 280, 175]] {
            rect(&mut rgb, width, [x0 * scale, y0 * scale + offset, x1 * scale, y1 * scale + offset]);
        }
        for y in 115..145 {
            let x = 10 + (y + shift) % 4 * 15;
            rect(&mut rgb, width, [x * scale, y * scale + offset, (x + 8) * scale, (y + 1) * scale + offset]);
        }
        let bands = detect_textline_bands_naive(&rgb, width, height, 220);
        proptest::prop_assert_eq!(bands.len(), 3, "scale={} offset={}: {:?}", scale, offset, bands);
        proptest::prop_assert_eq!(&bands, &detect_textline_bands_fast(&rgb, width, height, 220));
        proptest::prop_assert!(bands[2][1] <= 100 * scale + offset && bands[2][3] >= 175 * scale + offset);
        proptest::prop_assert!(bands[2][0] <= 40 * scale && bands[2][2] >= 280 * scale);
    }
}

/// 実フォントの短い横画を追加分割せず、元の下側領域を保持する。
#[test]
fn mixed_font_height_preserves_existing_lower_region() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/noisy_line_gaps/mixed_height.png",
    ))
    .unwrap();
    let bands = detect_textline_bands_naive(&image.data, image.width, image.height, 220);
    assert_eq!(
        bands,
        vec![
            [40, 27, 263, 56],
            [40, 87, 263, 116],
            [68, 229, 912, 271],
            [56, 296, 926, 372],
        ]
    );
    assert_eq!(
        bands,
        detect_textline_bands_fast(&image.data, image.width, image.height, 220)
    );
}
