use ndlocr_lite_rs::pipeline::line_chunks::{split_line, split_line_for_input};
use proptest::prelude::*;

/// 白列を挟んだ連続字形を合成する。
fn image(width: usize, height: usize, period: usize) -> Vec<u8> {
    (0..height)
        .flat_map(|_| (0..width).flat_map(|x| [if x % period == period - 1 { 255 } else { 0 }; 3]))
        .collect()
}

/// 8倍を超えてもモデル幅以下の短い行は分割しない。
#[test]
fn fitting_short_line_preserves_context() {
    let rgb = image(383, 47, 40);
    assert!(split_line(&rgb, 383, 47, 220).unwrap().len() > 1);
    let chunks = split_line_for_input(&rgb, 383, 47, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!((chunks[0].x0, chunks[0].x1), (0, 383));
    assert!(!chunks[0].space_before);
}

/// 入力比率の境界を整数で比較し、縮小モデルでは従来分割へ戻す。
#[test]
fn actual_input_ratio_controls_the_boundary() {
    for (width, input_width, expected) in [
        (320, 256, 1),
        (321, 256, 2),
        (300, 240, 1),
        (301, 240, 2),
        (300, 192, 2),
    ] {
        let rgb = image(width, 30, 40);
        assert_eq!(
            split_line_for_input(&rgb, width, 30, 220, input_width, 24)
                .unwrap()
                .len(),
            expected
        );
    }
}

/// 字形数または入力比率を超えた長文の区間は従来と一致する。
#[test]
fn long_or_dense_lines_keep_conservative_chunks() {
    for (width, height, period) in [(500, 40, 40), (400, 40, 10), (320, 30, 15)] {
        let rgb = image(width, height, period);
        assert_eq!(
            split_line_for_input(&rgb, width, height, 220, 256, 24).unwrap(),
            split_line(&rgb, width, height, 220).unwrap()
        );
    }
}

/// 不正寸法は文脈保持経路でも受け付けない。
#[test]
fn malformed_dimensions_are_rejected() {
    for (input_width, input_height) in [(0, 24), (256, 0), (usize::MAX, 24)] {
        assert!(split_line_for_input(&[255; 3], 1, 1, 220, input_width, input_height).is_err());
    }
    for (width, height, input_width, input_height) in [
        (1, 1, 0, 24),
        (1, 1, 256, 0),
        (1, 1, usize::MAX, 24),
        (usize::MAX, 2, 256, 24),
        (0, 1, 256, 24),
    ] {
        assert!(split_line_for_input(&[], width, height, 220, input_width, input_height).is_err());
    }
    assert!(split_line_for_input(&[255; 3], 2, 1, 220, 256, 24).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    /// 入力内の疎な行は画素列を欠落・重複させず、文脈を維持する。
    #[test]
    fn fitting_sparse_lines_remain_whole(height in 8usize..80, aspect in 8usize..11) {
        let width=height*aspect;
        let rgb=image(width,height,width.div_ceil(10));
        let chunks=split_line_for_input(&rgb,width,height,220,aspect*24,24).unwrap();
        prop_assert_eq!(chunks.len(),1);
        prop_assert_eq!((chunks[0].x0,chunks[0].x1),(0,width));
    }
}

/// 実見出しを同じ検出・切出し経路で認識し、本文の逆順化を防ぐ。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn real_heading_preserves_model_context() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            crop::{BBox, crop_rgb_u8},
            line_segment::detect_textline_bands_naive,
        },
    };
    use std::path::PathBuf;
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/model_context/target.png",
    ))
    .unwrap();
    let session = ParseqSession::load(
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let (input_width, input_height) = session.input_dimensions();
    assert_eq!((input_width, input_height), (256, 24));
    let bands = detect_textline_bands_naive(&image.data, image.width, image.height, 220);
    assert_eq!(bands.len(), 1);
    let [x0, y0, x1, y1] = bands[0];
    let line = crop_rgb_u8(
        &image.data,
        image.width,
        image.height,
        BBox::new(x0, y0, x1, y1),
    )
    .unwrap();
    let chunks = split_line_for_input(
        &line.data,
        line.width,
        line.height,
        220,
        input_width,
        input_height,
    )
    .unwrap();
    assert_eq!(chunks.len(), 1);
    let actual = session
        .recognize_rgb_u8_unprocessed(&line.data, line.width, line.height)
        .unwrap()
        .text;
    assert_eq!(actual, "(電子入札の対象)");
}
