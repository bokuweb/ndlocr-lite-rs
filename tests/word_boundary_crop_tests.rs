use ndlocr_lite_rs::pipeline::{
    crop::BBox,
    crop::crop_rgb_u8,
    line_chunks::{LineChunk, crop_line_chunk_rgb_u8},
};

/// 混在した分割では両端が単語境界または行端のchunkだけ余白を除く。
#[test]
fn glyph_boundaries_keep_context() {
    let mut rgb = vec![255; 96 * 4 * 3];
    for i in 0..3 {
        for x in (i * 32 + 2..i * 32 + 22).step_by(2) {
            rgb[(96 + x) * 3] = 0;
        }
    }
    for boundaries in [[true, true], [false, true], [true, false], [false, false]] {
        let chunks = (0..3)
            .map(|i| LineChunk {
                x0: i * 32,
                x1: (i + 1) * 32,
                space_before: i > 0 && boundaries[i - 1],
                gap_before: 3,
            })
            .collect::<Vec<_>>();
        for index in 0..3 {
            let tight = (index == 0 || chunks[index].space_before)
                && (index == 2 || chunks[index + 1].space_before);
            let (left, right) = if tight {
                (index * 32 + 2, index * 32 + 21)
            } else {
                (index * 32, (index + 1) * 32)
            };
            let expected = crop_rgb_u8(&rgb, 96, 4, BBox::new(left, 0, right, 4)).unwrap();
            assert_eq!(
                crop_line_chunk_rgb_u8(&rgb, 96, 4, &chunks, index).unwrap(),
                expected
            );
        }
    }
    let single = vec![LineChunk {
        x0: 0,
        x1: 96,
        space_before: false,
        gap_before: 0,
    }];
    assert_eq!(
        crop_line_chunk_rgb_u8(&rgb, 96, 4, &single, 0)
            .unwrap()
            .data,
        rgb
    );
    assert!(crop_line_chunk_rgb_u8(&rgb, 96, 4, &single, 1).is_err());
    assert!(crop_line_chunk_rgb_u8(&rgb, 96, 4, &[], 0).is_err());
}

/// 余白除去で公式PARSeqの回転条件を跨がず、等号境界では除去できる。
#[test]
fn horizontal_context_does_not_become_rotated() {
    let chunks = vec![
        LineChunk {
            x0: 0,
            x1: 128,
            space_before: false,
            gap_before: 0,
        },
        LineChunk {
            x0: 128,
            x1: 256,
            space_before: true,
            gap_before: 4,
        },
    ];
    for height in [16, 17] {
        let mut rgb = vec![255; 256 * height * 3];
        for x in (2..22).step_by(2) {
            rgb[(256 + x) * 3] = 0;
        }
        rgb[(256 + 21) * 3] = 0; // 十番目の画線を二列にして幅20を保つ。
        let crop = crop_line_chunk_rgb_u8(&rgb, 256, height, &chunks, 0).unwrap();
        assert_eq!(crop.width, if height == 16 { 20 } else { 128 });
        assert_eq!(crop.height, height);
    }
}

proptest::proptest! {
    /// 任意の左右余白と縦横比でも回転方向と全画線を保持する。
    #[test]
    fn margin_changes_preserve_rotation(
        left in 0usize..16, right in 0usize..16,
        runs in 1usize..21, height in 1usize..32,
    ) {
        let ink_width=runs*2-1;
        let width = left + ink_width + right;
        let mut rgb = vec![255; width * 2 * height * 3];
        for x in (left..left + ink_width).step_by(2) { rgb[x * 3] = 254; }
        let chunks = vec![
            LineChunk { x0: 0, x1: width, space_before: false, gap_before: 0 },
            LineChunk { x0: width, x1: width * 2, space_before: true, gap_before: 3 },
        ];
        let crop = crop_line_chunk_rgb_u8(&rgb, width * 2, height, &chunks, 0).unwrap();
        proptest::prop_assert_eq!(height * 5 > width * 4, height * 5 > crop.width * 4);
        proptest::prop_assert_eq!(crop.height, height);
        proptest::prop_assert_eq!(crop.data.chunks_exact(3).filter(|p| *p != [255; 3]).count(), runs);
        if (left + right) * 4 < height {
            let original = crop_rgb_u8(&rgb, width * 2, height, BBox::new(0, 0, width, height)).unwrap();
            proptest::prop_assert_eq!(crop, original);
        }
    }
}

/// 幅/高さ比5以上のchunkを調整し、短い単語の文脈を保つ。
#[test]
fn short_word_context_is_kept_at_aspect_boundary() {
    for width in [79, 80, 96, 112] {
        let mut rgb = vec![255; width * 2 * 16 * 3];
        for x in (2..22).step_by(2) {
            rgb[x * 3] = 0;
        }
        rgb[21 * 3] = 0;
        let chunks = vec![
            LineChunk {
                x0: 0,
                x1: width,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: width,
                x1: width * 2,
                space_before: true,
                gap_before: 3,
            },
        ];
        assert_eq!(
            crop_line_chunk_rgb_u8(&rgb, width * 2, 16, &chunks, 0)
                .unwrap()
                .width,
            if width == 79 { 79 } else { 20 }
        );
    }
}

/// 幅が長くても疎な記号の文脈を保ち、インク列群の支持数の等号でだけ除去する。
#[test]
fn sparse_marks_keep_context_at_support_boundary() {
    for runs in [8, 9, 10] {
        let mut rgb = vec![255; 32 * 2 * 2 * 3];
        for x in (2..2 + runs * 2).step_by(2) {
            rgb[x * 3] = 254;
        }
        let chunks = vec![
            LineChunk {
                x0: 0,
                x1: 32,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 32,
                x1: 64,
                space_before: true,
                gap_before: 3,
            },
        ];
        assert_eq!(
            crop_line_chunk_rgb_u8(&rgb, 64, 2, &chunks, 0)
                .unwrap()
                .width,
            if runs == 8 { 32 } else { runs * 2 - 1 }
        );
    }
}

/// 高さの四分の一未満の小さな余白削除では入力幅を変えず文脈を保つ。
#[test]
fn tiny_margin_removal_keeps_context_at_boundary() {
    for right in [2, 3, 4] {
        let width = 7 + 287 + right;
        let mut rgb = vec![255; width * 2 * 44 * 3];
        for x in (7..294).step_by(2) {
            rgb[x * 3] = 254;
        }
        let chunks = vec![
            LineChunk {
                x0: 0,
                x1: width,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: width,
                x1: width * 2,
                space_before: true,
                gap_before: 3,
            },
        ];
        let crop = crop_line_chunk_rgb_u8(&rgb, width * 2, 44, &chunks, 0).unwrap();
        assert_eq!(crop.width, if right < 4 { width } else { 287 });
    }
}
