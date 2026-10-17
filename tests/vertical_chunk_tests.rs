use ndlocr_lite_rs::pipeline::line_chunks::split_rotated_vertical_column_for_input;
use proptest::prelude::*;

/// 白列を挟む縦書き字形を、回転後の座標で作る。
fn glyphs(count: usize, advance: usize, height: usize) -> Vec<u8> {
    let width = count * advance;
    let mut rgb = vec![255; width * height * 3];
    for n in 0..count {
        for y in 0..height {
            for x in n * advance..(n + 1) * advance - 2 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    rgb
}

/// 文字数上限内の縦列は、横書きの単語間隔で小断片へ分解しない。
#[test]
fn preserves_short_vertical_context_and_rejects_unsplittable_columns() {
    let rgb = glyphs(18, 20, 20);
    let chunks = split_rotated_vertical_column_for_input(&rgb, 360, 20, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!((chunks[0].x0, chunks[0].x1), (0, 360));
    assert!(
        split_rotated_vertical_column_for_input(&vec![0; 500 * 20 * 3], 500, 20, 220, 256, 24)
            .is_err()
    );
    for (rgb, w, h, iw, ih) in [
        (&[][..], 0, 1, 256, 24),
        (&[0; 3][..], 1, 1, 0, 24),
        (&[][..], usize::MAX, 2, 256, 24),
    ] {
        assert!(split_rotated_vertical_column_for_input(rgb, w, h, 220, iw, ih).is_err());
    }
}

proptest! {
    /// 長い列でも本文画素を切らず全範囲を覆い、字数・比率を制限して空白を捏造しない。
    #[test]
    fn chunks_cover_all_pixels_without_inventing_word_spaces(count in 21usize..100, advance in 5usize..20) {
        let height = 20;
        let width = count * advance;
        let rgb = glyphs(count, advance, height);
        let chunks = split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
        prop_assert!(chunks.len() >= 2);
        prop_assert_eq!(chunks[0].x0, 0);
        prop_assert_eq!(chunks.last().unwrap().x1, width);
        for (i, chunk) in chunks.iter().enumerate() {
            prop_assert!(!chunk.space_before);
            prop_assert!(chunk.x0 < chunk.x1 && chunk.x1 - chunk.x0 <= 400);
            if i > 0 {
                prop_assert_eq!(chunks[i-1].x1, chunk.x0);
                prop_assert!((0..height).all(|y| rgb[(y*width+chunk.x0)*3] == 255));
            }
        }
    }
}
