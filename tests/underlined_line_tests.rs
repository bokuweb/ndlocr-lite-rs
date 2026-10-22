use ndlocr_lite_rs::pipeline::line_chunks::{split_line, split_line_for_input};

/// 多数の独立字形と、字間を接続する細い下線を元画素へ配置する。
fn sample(
    scale: usize,
    margin: usize,
    rule_y: usize,
    thickness: usize,
    glyphs: usize,
) -> (Vec<u8>, usize, usize) {
    let (w, h) = (400 * scale + 2 * margin, 24 * scale);
    let mut rgb = vec![255; w * h * 3];
    for glyph in 0..glyphs {
        for y in 2 * scale..18 * scale {
            for x in (8 + glyph * 16) * scale + margin..(16 + glyph * 16) * scale + margin {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    for y in rule_y * scale..(rule_y + thickness) * scale {
        for x in 4 * scale + margin..396 * scale + margin {
            rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
        }
    }
    (rgb, w, h)
}

#[test]
fn long_underline_allows_boundaries_without_changing_pixels() {
    let (rgb, w, h) = sample(1, 0, 21, 1, 24);
    assert!(
        split_line(&rgb, w, h, 220).is_err(),
        "strict white-column API must stay strict"
    );
    let chunks = split_line_for_input(&rgb, w, h, 220, 256, 24).unwrap();
    assert!(chunks.len() > 1);
    assert_eq!((chunks[0].x0, chunks.last().unwrap().x1), (0, w));
    assert!(chunks.windows(2).all(|cs| cs[0].x1 == cs[1].x0));
    for chunk in chunks.iter().skip(1) {
        for y in 0..h {
            if y != 21 {
                assert_eq!(
                    &rgb[(y * w + chunk.x0) * 3..(y * w + chunk.x0 + 1) * 3],
                    &[255; 3]
                );
            }
        }
    }
}

#[test]
fn unsupported_rules_keep_the_original_error() {
    for (y, thickness, glyphs) in [(10, 1, 24), (21, 3, 24), (21, 1, 2), (21, 1, 0)] {
        let (rgb, w, h) = sample(1, 0, y, thickness, glyphs);
        assert!(
            split_line_for_input(&rgb, w, h, 220, 256, 24).is_err(),
            "y={y} thickness={thickness} glyphs={glyphs}"
        );
    }
}

#[test]
fn real_url_bands_are_splittable_in_both_detection_paths() {
    use ndlocr_lite_rs::pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_segment::{detect_textline_bands_fast, detect_textline_bands_naive},
    };
    let im = image::open("tests/fixtures/eval/images/jma_kobe_notice.png")
        .unwrap()
        .to_rgb8();
    let (w, h) = (im.width() as usize, im.height() as usize);
    let bands = detect_textline_bands_naive(im.as_raw(), w, h, 220);
    assert_eq!(bands, detect_textline_bands_fast(im.as_raw(), w, h, 220));
    let urls: Vec<_> = bands.iter().filter(|b| b[1] >= 1400).collect();
    assert_eq!(urls.len(), 4);
    for &&[x0, y0, x1, y1] in &urls {
        let crop = crop_rgb_u8(im.as_raw(), w, h, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks =
            split_line_for_input(&crop.data, crop.width, crop.height, 220, 256, 24).unwrap();
        assert!(chunks.len() > 1);
        assert_eq!((chunks[0].x0, chunks.last().unwrap().x1), (0, crop.width));
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    #[test]
    fn underlined_splits_preserve_coverage_and_glyphs(scale in 1usize..4,margin in 0usize..12) {
        let (rgb,w,h)=sample(scale,margin,21,1,24);
        let chunks=split_line_for_input(&rgb,w,h,220,256,24).unwrap();
        proptest::prop_assert!(chunks.len()>1);
        proptest::prop_assert_eq!((chunks[0].x0,chunks.last().unwrap().x1),(0,w));
        proptest::prop_assert!(chunks.windows(2).all(|cs|cs[0].x1==cs[1].x0));
        for chunk in chunks.iter().skip(1) { for y in 0..h {
            if !(21*scale..22*scale).contains(&y) {
                proptest::prop_assert_eq!(&rgb[(y*w+chunk.x0)*3..(y*w+chunk.x0+1)*3],&[255;3]);
            }
        }}
    }
}

/// すでに安全に分割できる行へ下線fallbackを適用しない。
#[test]
fn already_splittable_or_short_lines_keep_original_context() {
    let (mut rgb, w, h) = sample(1, 0, 21, 1, 24);
    // 字形間の2箇所で下線を切り、既存の最大幅内へ3分割できるようにする。
    for x in [130, 258] {
        rgb[(21 * w + x) * 3..(21 * w + x + 4) * 3].fill(255);
    }
    let strict = split_line(&rgb, w, h, 220).unwrap();
    assert_eq!(
        split_line_for_input(&rgb, w, h, 220, 256, 24).unwrap(),
        strict
    );
    // 実入力寸法へ収まる短い行は、細い下線でも1区間のまま。
    let short: Vec<u8> = rgb
        .chunks_exact(w * 3)
        .flat_map(|row| row[..120 * 3].iter().copied())
        .collect();
    let chunks = split_line_for_input(&short, 120, h, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!((chunks[0].x0, chunks[0].x1), (0, 120));
}
