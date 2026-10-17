use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8_tight_horizontal};

/// 真っ白な外側の列だけを除き、色・薄い画線・内部空白・上下余白を保持する。
#[test]
fn only_exterior_white_columns_are_removed() {
    let mut rgb = vec![255; 8 * 4 * 3];
    rgb[(8 + 2) * 3] = 254;
    rgb[(16 + 5) * 3 + 2] = 0;
    let crop = crop_rgb_u8_tight_horizontal(&rgb, 8, 4, BBox::new(1, 0, 7, 4)).unwrap();
    assert_eq!((crop.width, crop.height), (4, 4));
    for y in 0..4 {
        assert_eq!(
            &crop.data[y * 12..(y + 1) * 12],
            &rgb[(y * 8 + 2) * 3..(y * 8 + 6) * 3]
        );
    }
}

/// 白紙は非空の元領域を保持し、領域外の画線を取り込まない。
#[test]
fn white_region_stays_nonempty() {
    let mut rgb = vec![255; 6 * 4 * 3];
    rgb[0] = 0;
    let crop = crop_rgb_u8_tight_horizontal(&rgb, 6, 4, BBox::new(1, 1, 5, 3)).unwrap();
    assert_eq!((crop.width, crop.height), (4, 2));
    assert_eq!(crop.data, vec![255; 4 * 2 * 3]);
}

/// 不正長・空領域・画像外領域・寸法overflowを切り出し前に拒否する。
#[test]
fn invalid_inputs_are_rejected() {
    assert!(crop_rgb_u8_tight_horizontal(&[255; 2], 1, 1, BBox::new(0, 0, 1, 1)).is_err());
    for bbox in [
        BBox::new(0, 0, 0, 1),
        BBox::new(0, 0, 2, 1),
        BBox::new(0, 1, 1, 2),
    ] {
        assert!(crop_rgb_u8_tight_horizontal(&[255; 3], 1, 1, bbox).is_err());
    }
    assert!(crop_rgb_u8_tight_horizontal(&[], usize::MAX, 2, BBox::new(0, 0, 1, 1)).is_err());
}

proptest::proptest! {
    /// 左右白余白・領域の移動・任意の内部画素を変えても全画素を保つ。
    #[test]
    fn padding_and_translation_preserve_pixels(
        pixels in proptest::collection::vec(proptest::array::uniform3(0u8..=255), 1..48),
        left in 0usize..12, right in 0usize..12, dx in 0usize..8, dy in 0usize..8,
        height in 1usize..8,
    ) {
        let inner = pixels.len();
        let width = dx + left + inner + right + 3;
        let full_height = dy + height + 2;
        let mut rgb = vec![255; width * full_height * 3];
        for y in dy..dy + height {
            for (x, pixel) in pixels.iter().enumerate() {
                let start = (y * width + dx + left + x) * 3;
                rgb[start..start + 3].copy_from_slice(pixel);
            }
        }
        // 両端の画線を保証し、内部の完全白を除去しない仕様を検証する。
        rgb[(dy * width + dx + left) * 3] = 254;
        rgb[((dy + height - 1) * width + dx + left + inner - 1) * 3 + 2] = 254;
        let crop = crop_rgb_u8_tight_horizontal(&rgb, width, full_height,
            BBox::new(dx, dy, dx + left + inner + right, dy + height)).unwrap();
        proptest::prop_assert_eq!((crop.width, crop.height), (inner, height));
        for y in 0..height {
            let start = ((dy + y) * width + dx + left) * 3;
            proptest::prop_assert_eq!(&crop.data[y * inner * 3..(y + 1) * inner * 3], &rgb[start..start + inner * 3]);
        }
    }
}
