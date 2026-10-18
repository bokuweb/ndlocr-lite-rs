use ndlocr_lite_rs::{
    infer::parseq::{ParseqOrientation, preprocess_rgb_u8_with_orientation},
    pipeline::vertical_segment::rotate_vertical_column_rgb_u8,
};
use proptest::prelude::*;

/// 上から下の文字順が左から右になり、RGB成分を変更しない。
#[test]
fn column_rotates_counterclockwise_without_resampling() {
    let rgb: Vec<u8> = (0..18).collect();
    let rotated = rotate_vertical_column_rgb_u8(&rgb, 2, 3).unwrap();
    assert_eq!((rotated.width, rotated.height), (3, 2));
    assert_eq!(
        rotated.data,
        [3, 4, 5, 9, 10, 11, 15, 16, 17, 0, 1, 2, 6, 7, 8, 12, 13, 14]
    );
}

/// 寸法の不正・overflow・余剰も、画像を確保する前に拒否する。
#[test]
fn invalid_column_is_rejected() {
    for (rgb, w, h) in [
        (&[][..], 0, 1),
        (&[][..], usize::MAX, 2),
        (&[1, 2][..], 1, 1),
        (&[1, 2, 3, 4][..], 1, 1),
    ] {
        assert!(rotate_vertical_column_rgb_u8(rgb, w, h).is_err());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 縦長列では既存Autoと、回転後Horizontalのモデル入力が完全一致する。
    #[test]
    fn rotated_horizontal_matches_auto(w in 1usize..15, extra in 1usize..30, seed in any::<u8>()) {
        let h = w + extra;
        let rgb: Vec<u8> = (0..w*h*3).map(|n| seed.wrapping_add(n as u8)).collect();
        let rotated = rotate_vertical_column_rgb_u8(&rgb, w, h).unwrap();
        let auto = preprocess_rgb_u8_with_orientation(&rgb, w, h, 32, 8, ParseqOrientation::Auto).unwrap();
        let horizontal = preprocess_rgb_u8_with_orientation(&rotated.data, h, w, 32, 8, ParseqOrientation::Horizontal).unwrap();
        prop_assert_eq!(auto, horizontal);
    }

    /// 小さい列も比率の推測によらず回し、四回転で元の全画素へ戻る。
    #[test]
    fn four_turns_preserve_every_pixel(w in 1usize..25, h in 1usize..25, seed in any::<u8>()) {
        let rgb: Vec<u8> = (0..w*h*3).map(|n| seed.wrapping_add(n as u8)).collect();
        let mut im = rotate_vertical_column_rgb_u8(&rgb, w, h).unwrap();
        for _ in 0..3 {
            im = rotate_vertical_column_rgb_u8(&im.data, im.width, im.height).unwrap();
        }
        prop_assert_eq!((im.width, im.height), (w, h));
        prop_assert_eq!(im.data, rgb);
    }
}
