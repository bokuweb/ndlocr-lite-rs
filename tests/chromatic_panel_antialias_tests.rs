use ndlocr_lite_rs::pipeline::background::normalize_chromatic_panels;
use proptest::prelude::*;

/// 保存した実画像の境界線が、長いOCR行の安全な分割を妨げない。
#[test]
fn real_resampled_panel_has_safe_word_boundaries() {
    use ndlocr_lite_rs::pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panel_antialias/edge.png",
    ))
    .unwrap();
    let out = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&out, image.width, image.height, 220);
    assert_eq!(bands.len(), 1, "{bands:?}");
    for [x0, y0, x1, y1] in bands {
        let line = crop_rgb_u8(&out, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        assert!(split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).is_ok());
    }
}

/// 白との混合率を整数で固定し、丸めた境界色を作る。
fn blend(background: [u8; 3], white: u8) -> [u8; 3] {
    background.map(|b| (u16::from(b) + (255 - u16::from(b)) * u16::from(white) / 255) as u8)
}

/// 外縁に沿う黒線・異色線は白との混合に該当せず、除去しない。
#[test]
fn black_and_colored_border_rules_are_retained() {
    for color in [[0; 3], [72, 61, 139]] {
        let (mut rgb, width, height) = panel(0, 0, [255, 165, 0]);
        for x in 16..304 {
            let y = 84 + (x / 48) % 2;
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&color);
        }
        let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
        for x in 20..300 {
            let y = 84 + (x / 48) % 2;
            assert!(out[(y * width + x) * 3] < 220, "lost border at {x},{y}");
        }
    }
}

/// 上下端が階段状にずれ、各列の境界色も異なる横長パネルを作る。
fn panel(offset: usize, phase: usize, background: [u8; 3]) -> (Vec<u8>, usize, usize) {
    let (width, height) = (320 + offset * 2, 100);
    let mut rgb = vec![255; width * height * 3];
    for x in 16 + offset..304 + offset {
        let step = (x / 48 + phase) % 2;
        for y in 12 + step..84 + step {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&background);
        }
        for y in [11 + step, 84 + step] {
            let color = blend(background, ((x * 17 + phase) % 220) as u8);
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&color);
        }
    }
    (rgb, width, height)
}

/// 文字のないパネル境界から、字間をつなぐ黒線を作らない。
#[test]
fn sloped_antialiased_edges_leave_no_foreground() {
    let (rgb, width, height) = panel(0, 0, [255, 165, 0]);
    let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
    for y in 10..87 {
        for x in 20..300 {
            let i = (y * width + x) * 3;
            assert!(
                out[i] >= 220,
                "boundary became ink at {x},{y}: {:?}",
                &out[i..i + 3]
            );
        }
    }
}

/// 内部の白黒文字・細線と、短い白文字の端の混合画素を保持する。
#[test]
fn glyphs_and_thin_rules_remain_dark() {
    let (mut rgb, width, height) = panel(0, 0, [255, 165, 0]);
    for (x, y, color) in [
        (40, 40, [255; 3]),
        (80, 40, [0; 3]),
        (120, 40, [72, 61, 139]),
        (170, 83, blend([255, 165, 0], 160)),
    ] {
        rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&color);
    }
    for (y, color) in [(60, [255; 3]), (70, [0; 3]), (75, [72, 61, 139])] {
        for x in 24..if y == 60 { 120 } else { 296 } {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&color);
        }
    }
    let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
    for (x, y) in [
        (40, 40),
        (80, 40),
        (120, 40),
        (170, 83),
        (100, 60),
        (100, 70),
        (100, 75),
    ] {
        assert!(out[(y * width + x) * 3] < 220, "lost stroke at {x},{y}");
    }
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]
    /// 平行移動と混合率を変えても上下の外縁を字形へ反転しない。
    #[test]
    fn edge_removal_survives_translation_and_blend_phase(offset in 0usize..12, phase in 0usize..220, color in any::<[u8;3]>()) {
        prop_assume!(*color.iter().max().unwrap()-*color.iter().min().unwrap() >= 24);
        let (rgb,width,height)=panel(offset,phase,color);
        let out=normalize_chromatic_panels(&rgb,width,height).unwrap();
        for x in 20+offset..300+offset {
            let step=(x/48+phase)%2;
            for y in [11+step,84+step] {
                prop_assert!(out[(y*width+x)*3] >= 220, "at {},{}", x,y);
            }
        }
    }
}
