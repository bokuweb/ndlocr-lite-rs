use ndlocr_lite_rs::pipeline::{
    crop::BBox,
    crop::crop_rgb_u8,
    line_chunks::{LineChunk, crop_line_chunk_rgb_u8, split_line, split_line_for_input},
};
use proptest::prelude::*;

/// 離れた字群と行外余白を、元RGBを保って合成する。
fn separated(height: usize, gap: usize, padding: usize, color: [u8; 3]) -> (Vec<u8>, usize) {
    let width = 6 * height + 2 * gap + 2 * padding;
    let mut rgb = vec![255; width * height * 3];
    for group in 0..3 {
        let start = padding + group * (2 * height + gap);
        for y in 0..height {
            for x in start..start + 2 * height {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].copy_from_slice(&color);
            }
        }
    }
    (rgb, width)
}

/// 全非白画素の一意な元座標と、認識へ渡るRGBの保存を検証する。
fn assert_pixels(rgb: &[u8], width: usize, height: usize, chunks: &[LineChunk]) {
    let mut coverage = vec![0; width];
    for (index, chunk) in chunks.iter().enumerate() {
        assert!(chunk.x0 < chunk.x1 && chunk.x1 <= width);
        // 認識用APIは実モデル比率内の疎な短文を未分割で保持する。
        assert!(chunk.x1 - chunk.x0 <= (height * 8).max(height * 256 / 24));
        for count in &mut coverage[chunk.x0..chunk.x1] {
            *count += 1;
        }
        let actual = crop_line_chunk_rgb_u8(rgb, width, height, chunks, index).unwrap();
        let expected =
            crop_rgb_u8(rgb, width, height, BBox::new(chunk.x0, 0, chunk.x1, height)).unwrap();
        assert_eq!(actual.data, expected.data);
        assert!(actual.data.chunks_exact(3).any(|p| p != [255; 3]));
    }
    for (x, count) in coverage.into_iter().enumerate() {
        assert!(count <= 1, "duplicate column {x}");
        if (0..height).any(|y| rgb[(y * width + x) * 3..(y * width + x + 1) * 3] != [255; 3]) {
            assert_eq!(count, 1, "lost nonwhite column {x}");
        }
    }
}

/// 広い白余白から白だけのモデル入力を作らず、元語間を一度保持する。
#[test]
fn widely_separated_groups_are_safe_recognition_chunks() {
    for gap in [161, 162, 700] {
        let (rgb, width) = separated(20, gap, 3, [0, 0, 0]);
        assert!(split_line(&rgb, width, 20, 220).is_err());
        let chunks = split_line_for_input(&rgb, width, 20, 220, 256, 24).unwrap();
        assert_eq!(chunks.len(), 3);
        assert!(!chunks[0].space_before);
        for chunk in &chunks[1..] {
            assert!(chunk.insert_space_before("left", "right", 20));
            assert_eq!(chunk.gap_before, gap);
        }
        assert_pixels(&rgb, width, 20, &chunks);
    }
}

/// 既存の実失敗6帯でも、薄い画素を落とさず元座標の認識区間を返す。
#[test]
fn six_upright_label_bands_keep_every_nonwhite_pixel() {
    for name in [
        "latin_1_+0",
        "latin_2_+0",
        "numeric_1_+0",
        "numeric_2_+0",
        "japanese_1_+0",
        "japanese_2_+0",
    ] {
        let path = format!("tests/fixtures/wide_blank_gaps/{name}.png");
        let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&path)).unwrap();
        let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
        assert_pixels(&im.data, im.width, im.height, &chunks);
        assert!(chunks.iter().skip(1).all(|c| c.space_before));
    }
}

/// 薄い色点が広い余白を分断するなら、閾値未満のインクがなくても消さない。
#[test]
fn nearwhite_marks_are_never_omitted() {
    let (mut rgb, width) = separated(20, 500, 0, [1, 2, 3]);
    for x in [220, 400, 750, 900] {
        rgb[(width + x) * 3..(width + x + 1) * 3].copy_from_slice(&[255, 255, 254]);
    }
    let chunks = split_line_for_input(&rgb, width, 20, 220, 256, 24).unwrap();
    assert_pixels(&rgb, width, 20, &chunks);
}

/// 分離した群自体が安全に分割できないときは部分成功を返さない。
#[test]
fn unbreakable_group_still_rejects_the_whole_line() {
    let (mut rgb, width) = separated(10, 400, 0, [0; 3]);
    for y in 0..10 {
        for x in 0..100 {
            rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
        }
    }
    assert!(split_line_for_input(&rgb, width, 10, 220, 128, 24).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    /// 縮尺・奇偶余白・RGB・平行移動が変わっても、元画素と語間を保持する。
    #[test]
    fn sparse_groups_preserve_pixels_and_spaces(h in 3usize..40, factor in 9usize..35, odd in 0usize..2, padding in 0usize..5, color in prop::array::uniform3(0u8..180)) {
        let gap = h * factor + odd;
        let (rgb, width) = separated(h, gap, padding, color);
        let chunks = split_line_for_input(&rgb,width,h,220,256,24).unwrap();
        prop_assert_eq!(chunks.len(),3);
        assert_pixels(&rgb,width,h,&chunks);
        for c in &chunks[1..] { prop_assert!(c.space_before); prop_assert_eq!(c.gap_before,gap); }
    }
}
