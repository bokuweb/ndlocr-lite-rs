use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};
use proptest::prelude::*;

#[test]
fn line_segment_returns_empty_for_blank_page() {
    let width = 200usize;
    let height = 120usize;
    let rgb = vec![255u8; width * height * 3];
    let out = detect_textline_bands_fast(&rgb, width, height, 220);
    assert!(out.is_empty());
}

#[test]
fn line_segment_detects_two_horizontal_bands() {
    let width = 240usize;
    let height = 140usize;
    let mut rgb = vec![255u8; width * height * 3];
    draw_black_band(&mut rgb, width, 20, 36, 16, 16);
    draw_black_band(&mut rgb, width, 72, 88, 12, 14);

    let out = detect_textline_bands_fast(&rgb, width, height, 220);
    assert_eq!(out.len(), 2);
    assert!(out[0][1] <= 21 && out[0][3] >= 35);
    assert!(out[1][1] <= 73 && out[1][3] >= 87);
}

proptest! {
    #[test]
    fn fast_and_naive_segmentation_are_equivalent(
        width in 48usize..96usize,
        height in 48usize..96usize,
        threshold in 160u8..240u8,
    ) {
        let mut rgb = vec![255u8; width * height * 3];
        for y in (8..height.saturating_sub(8)).step_by(14) {
            let band_h = (height / 16).max(5);
            let start_x = (y * 7) % (width / 3).max(1);
            let end_x = (start_x + (width / 2)).min(width.saturating_sub(1));
            for yy in y..(y + band_h).min(height) {
                for xx in start_x..=end_x {
                    let i = (yy * width + xx) * 3;
                    rgb[i] = 20;
                    rgb[i + 1] = 20;
                    rgb[i + 2] = 20;
                }
            }
        }

        let naive = detect_textline_bands_naive(&rgb, width, height, threshold);
        let fast = detect_textline_bands_fast(&rgb, width, height, threshold);
        prop_assert_eq!(fast, naive);
    }
}

fn draw_black_band(
    rgb: &mut [u8],
    width: usize,
    y0: usize,
    y1: usize,
    left_margin: usize,
    right_margin: usize,
) {
    for y in y0..y1 {
        for x in left_margin..(width - right_margin) {
            let i = (y * width + x) * 3;
            rgb[i] = 0;
            rgb[i + 1] = 0;
            rgb[i + 2] = 0;
        }
    }
}
