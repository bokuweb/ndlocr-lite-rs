use ndlocr_lite_rs::infer::{deim, parseq};
use proptest::prelude::*;

#[test]
fn parseq_preprocess_rotates_vertical_image_and_normalizes() {
    let rgb = vec![255_u8, 0, 0, 0, 0, 255];
    let out = parseq::preprocess_rgb_u8(&rgb, 1, 2, 2, 1).unwrap();
    // NCHW B,G,R（Python parseq.py の RGB→BGR と同じ並び）
    assert_eq!(out, vec![-1.0, 1.0, -1.0, -1.0, 1.0, -1.0]);
}

#[test]
fn deim_preprocess_pads_to_square_and_returns_metadata() {
    let rgb = vec![255_u8, 255, 255, 0, 0, 0];
    let out = deim::preprocess_rgb_u8(&rgb, 2, 1, 2, 2).unwrap();
    assert_eq!(out.padded_wh, 2);
    assert_eq!(out.tensor.len(), 12);
}

proptest! {
    #[test]
    fn parseq_preprocess_normalized_values_are_in_range(
        w in 1usize..5, h in 1usize..5,
        data in proptest::collection::vec(any::<u8>(), 3usize..300)
    ) {
        let needed = w*h*3;
        prop_assume!(data.len() >= needed);
        let out = parseq::preprocess_rgb_u8(&data[0..needed], w, h, w, h).unwrap();
        for &v in &out { prop_assert!(v >= -1.0 && v <= 1.0); }
    }
}
