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
fn parseq_preprocess_into_matches_allocating_output() {
    for &(width, height, input_width, input_height) in &[
        (7usize, 5usize, 13usize, 3usize),
        (5usize, 7usize, 13usize, 3usize),
    ] {
        let rgb = make_rgb(width, height);
        let allocating = parseq::preprocess_rgb_u8(&rgb, width, height, input_width, input_height)
            .expect("allocating preprocess should succeed");
        let mut into = vec![0.0f32; allocating.len()];
        parseq::preprocess_rgb_u8_into(&mut into, &rgb, width, height, input_width, input_height)
            .expect("direct preprocess should succeed");
        assert_eq!(into, allocating);
    }
}

#[test]
fn parseq_preprocess_into_rejects_wrong_output_size() {
    let rgb = vec![255_u8, 0, 0];
    let mut out = vec![0.0f32; 2];
    let err = parseq::preprocess_rgb_u8_into(&mut out, &rgb, 1, 1, 1, 1)
        .expect_err("must reject wrong output size");
    assert!(err.to_string().contains("output buffer length"));
}

#[test]
fn deim_preprocess_pads_to_square_and_returns_metadata() {
    let rgb = vec![255_u8, 255, 255, 0, 0, 0];
    let out = deim::preprocess_rgb_u8(&rgb, 2, 1, 2, 2).unwrap();
    assert_eq!(out.padded_wh, 2);
    assert_eq!(out.tensor.len(), 12);
}

#[test]
fn deim_preprocess_matches_direct_formula_for_padding_and_pixels() {
    let rgb = make_rgb(3, 2);
    let out = deim::preprocess_rgb_u8(&rgb, 3, 2, 5, 5).unwrap();
    let expected = reference_deim_preprocess(&rgb, 3, 2, 5, 5);
    assert_eq!(out.padded_wh, 3);
    assert_eq!(out.tensor, expected);
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

fn reference_deim_preprocess(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> Vec<f32> {
    let max_wh = width.max(height);
    let mut out = vec![0.0_f32; 3 * input_width * input_height];
    let plane = input_width * input_height;
    let mean = [0.485_f32, 0.456_f32, 0.406_f32];
    let std = [0.229_f32, 0.224_f32, 0.225_f32];
    for y in 0..input_height {
        let sy = y * max_wh / input_height;
        let in_h = sy < height;
        for x in 0..input_width {
            let sx = x * max_wh / input_width;
            let i = y * input_width + x;
            if in_h && sx < width {
                let s = (sy * width + sx) * 3;
                out[i] = (rgb[s] as f32 / 255.0 - mean[0]) / std[0];
                out[plane + i] = (rgb[s + 1] as f32 / 255.0 - mean[1]) / std[1];
                out[plane * 2 + i] = (rgb[s + 2] as f32 / 255.0 - mean[2]) / std[2];
            } else {
                out[i] = -mean[0] / std[0];
                out[plane + i] = -mean[1] / std[1];
                out[plane * 2 + i] = -mean[2] / std[2];
            }
        }
    }
    out
}

fn make_rgb(w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            out[i] = ((x * 17 + y * 11) % 255) as u8;
            out[i + 1] = ((x * 7 + y * 13) % 255) as u8;
            out[i + 2] = ((x * 19 + y * 3) % 255) as u8;
        }
    }
    out
}
