use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};

fn sample(
    scale: usize,
    margin: usize,
    sides: usize,
    thickness: usize,
    end: usize,
    glyphs: usize,
) -> (Vec<u8>, usize, usize) {
    let (w, h) = (600 * scale + 2 * margin, 200 * scale);
    let mut rgb = vec![255; w * h * 3];
    for y0 in [40, 90, 140] {
        for glyph in 0..glyphs {
            for y in y0 * scale..(y0 + 24) * scale {
                for x in (50 + glyph * 25) * scale + margin..(65 + glyph * 25) * scale + margin {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
    }
    for side in 0..sides {
        for y in 20 * scale..end * scale {
            for x in
                (20 + 560 * side) * scale + margin..(20 + 560 * side + thickness) * scale + margin
            {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    (rgb, w, h)
}

#[test]
fn paired_multiline_rules_stay_outside_text_bands() {
    let (rgb, w, h) = sample(1, 0, 2, 1, 185, 20);
    let bands = detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(bands, detect_textline_bands_fast(&rgb, w, h, 220));
    assert_eq!(bands.len(), 3);
    for b in bands {
        assert_eq!((b[0], b[2]), (48, 542));
    }
}

#[test]
fn short_thick_single_or_sparse_rules_are_not_trimmed() {
    for (sides, thickness, end, glyphs) in [
        (1, 1, 185, 20),
        (2, 4, 185, 20),
        (2, 1, 70, 20),
        (2, 1, 185, 2),
    ] {
        let (rgb, w, h) = sample(1, 0, sides, thickness, end, glyphs);
        let bands = detect_textline_bands_naive(&rgb, w, h, 220);
        assert!(
            bands[0][0] <= 20,
            "{sides} {thickness} {end} {glyphs}: {bands:?}"
        );
    }
}

#[test]
fn kobe_box_sides_do_not_become_text() {
    let im = image::open("tests/fixtures/eval/images/jma_kobe_notice.png")
        .unwrap()
        .to_rgb8();
    let (w, h) = (im.width() as usize, im.height() as usize);
    let bands = detect_textline_bands_naive(im.as_raw(), w, h, 220);
    assert_eq!(bands, detect_textline_bands_fast(im.as_raw(), w, h, 220));
    assert_eq!(bands.len(), 32);
    assert_eq!((bands[5][0], bands[5][2]), (66, 948));
    for b in [bands[5], bands[8]] {
        assert!(b[0] > 29 && b[2] <= 951, "{b:?}");
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(16))]
    #[test]
    fn frame_trimming_preserves_body_at_scale(scale in 1usize..3,margin in 0usize..8) {
        let (rgb,w,h)=sample(scale,margin,2,1,185,20);
        let bands=detect_textline_bands_fast(&rgb,w,h,220);
        proptest::prop_assert_eq!(bands.len(),3);
        for (b,y) in bands.iter().zip([40,90,140]) {
            proptest::prop_assert_eq!((b[0],b[2]),(50*scale+margin-2,540*scale+margin+2));
            proptest::prop_assert!(b[1]<=y*scale && b[3]>=(y+24)*scale);
        }
    }
}

/// 近い枠を除くと正読の中点を落とすので、元のモデル文脈を維持する。
#[test]
fn close_frame_preserves_known_correct_middle_dot_context() {
    let im = image::open("tests/fixtures/eval/images/jma_kobe_notice.png")
        .unwrap()
        .to_rgb8();
    let bands =
        detect_textline_bands_naive(im.as_raw(), im.width() as usize, im.height() as usize, 220);
    assert_eq!((bands[7][0], bands[7][2]), (26, 965));
}
