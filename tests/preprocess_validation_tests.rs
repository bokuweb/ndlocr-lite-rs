use ndlocr_lite_rs::infer::{deim, parseq};
use proptest::prelude::*;

/// 算術overflowとVecのbyte上限超過は、panicや巨大確保より先に拒否する。
#[test]
fn allocating_preprocessors_reject_impossible_tensor_dimensions_without_panicking() {
    let rgb = [0, 128, 255];
    let over_capacity = isize::MAX as usize / size_of::<f32>() / 3 + 1;
    for (width, height) in [(usize::MAX / 3 + 1, 1), (over_capacity, 1), (usize::MAX, 0)] {
        for orientation in [
            parseq::ParseqOrientation::Auto,
            parseq::ParseqOrientation::Horizontal,
        ] {
            let result = std::panic::catch_unwind(|| {
                parseq::preprocess_rgb_u8_with_orientation(&rgb, 1, 1, width, height, orientation)
            });
            assert!(result.is_ok(), "PARSeq panicked for {width}x{height}");
            assert!(result.unwrap().is_err());
        }
        let result =
            std::panic::catch_unwind(|| deim::preprocess_rgb_u8(&rgb, 1, 1, width, height));
        assert!(result.is_ok(), "DEIM panicked for {width}x{height}");
        assert!(result.unwrap().is_err());
    }
}

/// DEIMも空の元画像を有効な黒画像として受け入れない。
#[test]
fn deim_rejects_empty_source_axes() {
    for (width, height) in [(0, 0), (0, 8), (8, 0)] {
        assert!(deim::preprocess_rgb_u8(&[], width, height, 2, 2).is_err());
    }
}

/// 不正なRGBは出力確保より先に検証し、確保サイズでpanicしない。
#[test]
fn invalid_source_is_rejected_before_allocating_output() {
    for (rgb, width, height, expected) in [
        (&[][..], 0, 1, "source image dimension"),
        (&[0][..], 1, 1, "RGB buffer length"),
        (&[][..], usize::MAX, 2, "image size overflow"),
    ] {
        let capacity = isize::MAX as usize / size_of::<f32>() / 3 + 1;
        let parseq =
            std::panic::catch_unwind(|| parseq::preprocess_rgb_u8(rgb, width, height, capacity, 1));
        assert!(parseq.is_ok());
        assert!(parseq.unwrap().unwrap_err().to_string().contains(expected));
        let deim =
            std::panic::catch_unwind(|| deim::preprocess_rgb_u8(rgb, width, height, capacity, 1));
        assert!(deim.is_ok());
        assert!(deim.unwrap().err().unwrap().to_string().contains(expected));
    }
}

/// 失敗後も書き込み先と座標cacheを再利用できる。
#[test]
fn direct_preprocess_preserves_output_on_error_and_recovers() {
    let rgb = [0, 128, 255];
    let mut scratch = parseq::PreprocessScratch::new();
    let mut output = [42.0; 3];
    for (width, height) in [(0, 1), (usize::MAX, 2)] {
        assert!(
            parseq::preprocess_rgb_u8_into_with_scratch(
                &mut output,
                &rgb,
                1,
                1,
                width,
                height,
                &mut scratch
            )
            .is_err()
        );
        assert_eq!(output, [42.0; 3]);
    }
    parseq::preprocess_rgb_u8_into_with_scratch(&mut output, &rgb, 1, 1, 1, 1, &mut scratch)
        .unwrap();
    assert_eq!(
        output,
        parseq::preprocess_rgb_u8(&rgb, 1, 1, 1, 1)
            .unwrap()
            .as_slice()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    /// 乗算overflowする出力寸法の近傍を、両前処理で拒否する。
    #[test]
    fn overflow_dimension_neighborhood_is_rejected(offset in 0usize..1024) {
        let width = usize::MAX - offset;
        let rgb = [0, 0, 0];
        prop_assert!(parseq::preprocess_rgb_u8(&rgb, 1, 1, width, 2).is_err());
        prop_assert!(deim::preprocess_rgb_u8(&rgb, 1, 1, width, 2).is_err());
    }
}
