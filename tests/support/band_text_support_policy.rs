use super::*;
use proptest::prelude::*;
fn paint(
    rgb: &mut [u8],
    w: usize,
    xs: std::ops::Range<usize>,
    ys: std::ops::Range<usize>,
    color: u8,
) {
    for y in ys {
        for x in xs.clone() {
            rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(color);
        }
    }
}
fn frame() -> Vec<u8> {
    let mut rgb = vec![255; 80 * 24 * 3];
    paint(&mut rgb, 80, 4..34, 2..5, 0);
    paint(&mut rgb, 80, 4..7, 2..22, 0);
    paint(&mut rgb, 80, 4..34, 19..22, 0);
    rgb
}
#[test]
fn complex_letter_shape_supports_band_without_changing_pixels() {
    let rgb = frame();
    let original = rgb.clone();
    assert!(has_text_like_ink(&rgb, 80, 24, [0, 0, 80, 24], 220));
    assert_eq!(rgb, original);
}
#[test]
fn rectangular_blocks_stripes_and_points_do_not_support_text() {
    for (w, h) in [(64, 64), (160, 32), (32, 160), (3000, 48)] {
        assert!(!has_text_like_ink(
            &vec![0; w * h * 3],
            w,
            h,
            [0, 0, w as i32, h as i32],
            220
        ));
    }
    for vertical in [true, false] {
        let (w, h) = (300, 48);
        let mut rgb = vec![255; w * h * 3];
        for x in (0..w).step_by(30) {
            paint(
                &mut rgb,
                w,
                x..x + 15,
                if vertical { 5..43 } else { 22..24 },
                0,
            );
        }
        assert!(!has_text_like_ink(
            &rgb,
            w,
            h,
            [0, 0, w as i32, h as i32],
            220
        ));
    }
    let mut rgb = vec![255; 80 * 24 * 3];
    for x in (0..80).step_by(4) {
        paint(&mut rgb, 80, x..x + 1, 12..13, 0);
    }
    assert!(!has_text_like_ink(&rgb, 80, 24, [0, 0, 80, 24], 220));
}
#[test]
fn repeated_thin_vertical_strokes_are_supported_without_word_rules() {
    let mut rgb = vec![255; 80 * 24 * 3];
    for x in [5, 25, 45] {
        paint(&mut rgb, 80, x..x + 3, 3..21, 0);
    }
    assert!(has_text_like_ink(&rgb, 80, 24, [0, 0, 80, 24], 220));
    let mut one = vec![255; 80 * 24 * 3];
    paint(&mut one, 80, 5..8, 3..21, 0);
    assert!(!has_text_like_ink(&one, 80, 24, [0, 0, 80, 24], 220));
}
#[test]
fn fill_ratio_boundary_is_exact() {
    for hole in [40, 41] {
        let mut rgb = vec![255; 48 * 24 * 3];
        paint(&mut rgb, 48, 2..42, 2..22, 0);
        paint(&mut rgb, 48, 10..30, 10..12, 255);
        if hole == 41 {
            paint(&mut rgb, 48, 30..31, 11..12, 255);
        }
        assert_eq!(
            has_text_like_ink(&rgb, 48, 24, [0, 0, 48, 24], 220),
            hole == 41
        );
    }
}
#[test]
fn invalid_dimensions_or_empty_foreground_are_rejected() {
    let rgb = frame();
    for b in [
        [-1, 0, 80, 24],
        [0, 0, 81, 24],
        [0, 0, 80, 25],
        [0, 0, 0, 24],
        [5, 5, 4, 4],
    ] {
        assert!(!has_text_like_ink(&rgb, 80, 24, b, 220));
    }
    assert!(!has_text_like_ink(
        &rgb[..rgb.len() - 1],
        80,
        24,
        [0, 0, 80, 24],
        220
    ));
    assert!(!has_text_like_ink(&[], usize::MAX, 24, [0, 0, 80, 24], 220));
    assert!(!has_text_like_ink(
        &vec![255; 80 * 24 * 3],
        80,
        24,
        [0, 0, 80, 24],
        220
    ));
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(24))]
 #[test]
 fn text_shape_support_is_scale_translation_and_tone_invariant(scale in 1usize..4,dx in 0usize..20,dy in 0usize..10,gray in 0u8..220){
  let source=frame();let(w,h)=(80*scale+dx+7,24*scale+dy+5);let mut rgb=vec![255;w*h*3];for y in 0..24*scale{for x in 0..80*scale{if source[((y/scale)*80+x/scale)*3]==0{rgb[((y+dy)*w+x+dx)*3..((y+dy)*w+x+dx+1)*3].fill(gray);}}}
  prop_assert!(has_text_like_ink(&rgb,w,h,[dx as i32,dy as i32,(dx+80*scale)as i32,(dy+24*scale)as i32],220));
 }
}

#[test]
fn narrow_horizontal_repeated_strokes_are_still_text_like() {
    let mut rgb = vec![255; 40 * 24 * 3];
    for x in [3, 13, 23] {
        paint(&mut rgb, 40, x..x + 3, 3..21, 0);
    }
    assert!(has_text_like_ink(&rgb, 40, 24, [0, 0, 40, 24], 220));
}
