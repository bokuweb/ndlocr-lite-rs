use ndlocr_lite_rs::pipeline::line_segment::{
    detect_textline_bands_fast, detect_textline_bands_naive,
};
use proptest::prelude::*;

/// 画素行ごとのインク量から、境界を固定した合成画像を作る。
fn image_with_row_ink(width: usize, counts: &[usize]) -> Vec<u8> {
    let mut rgb = vec![255; width * counts.len() * 3];
    for (y, &count) in counts.iter().enumerate() {
        for x in 100..100 + count {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(0);
        }
    }
    rgb
}

/// 高密度seedの前後の弱い字形行を切らず、7px以上の文字として保持する。
#[test]
fn sparse_glyph_rows_reach_the_strong_seed() {
    let mut counts = vec![0; 48];
    counts[10..34].fill(12);
    counts[18..20].fill(32);
    let rgb = image_with_row_ink(2482, &counts);
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        let bands = detect(&rgb, 2482, 48, 220);
        assert_eq!(bands.len(), 1, "{bands:?}");
        assert_eq!((bands[0][1], bands[0][3]), (9, 35));
        assert!(bands[0][0] <= 100 && bands[0][2] >= 112);
    }
}

/// 弱いノイズだけの候補は高さがあっても行に昇格させない。
#[test]
fn weak_noise_without_a_seed_is_rejected() {
    let rgb = image_with_row_ink(2482, &[12; 24]);
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        assert!(detect(&rgb, 2482, 24, 220).is_empty());
    }
}

/// 3画素行の空白で分離し、下端の弱い画素も強いseedと一緒に残す。
#[test]
fn sparse_lines_remain_separate_at_the_bottom_edge() {
    let mut counts = vec![12; 35];
    counts[3] = 32;
    counts[16..19].fill(0);
    counts[23] = 32;
    let rgb = image_with_row_ink(2482, &counts);
    for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
        let bands = detect(&rgb, 2482, 35, 220);
        assert_eq!(bands.len(), 2, "{bands:?}");
        assert_eq!((bands[0][1], bands[0][3]), (0, 17));
        assert_eq!((bands[1][1], bands[1][3]), (18, 35));
    }
}

/// 実字形をページ幅へ配置しても、短い見出しを縦の断片へ分解しない。
#[test]
fn real_short_headings_keep_their_glyph_height_on_a_wide_page() {
    for case in ["purpose", "contents"] {
        let (rgb, width, height) = wide_heading(case);
        for detect in [detect_textline_bands_naive, detect_textline_bands_fast] {
            let bands = detect(&rgb, width, height, 220);
            assert_eq!(bands.len(), 1, "{case}: {bands:?}");
            assert!(bands[0][3] - bands[0][1] >= 43, "{case}: {bands:?}");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    /// seed位置・行高・行間が変わっても候補の全高と行の分離を保つ。
    #[test]
    fn seeded_sparse_spans_keep_extent_and_gap_boundaries(
        width in 1280usize..3200,
        h in 7usize..32,
        seed in 0usize..32,
        gap in 0usize..6,
        tail in 0usize..5,
    ) {
        let mut counts = vec![8; h * 2 + gap + tail];
        counts[seed % h] = width / 80;
        counts[h..h+gap].fill(0);
        counts[h+gap+seed % h] = width / 80;
        counts[h*2+gap..].fill(0);
        let rgb = image_with_row_ink(width, &counts);
        let bands = detect_textline_bands_naive(&rgb, width, counts.len(), 220);
        prop_assert_eq!(&bands, &detect_textline_bands_fast(&rgb, width, counts.len(), 220));
        prop_assert_eq!(bands.len(), if gap > 2 { 2 } else { 1 });
        prop_assert_eq!(bands[0][1], 0);
        prop_assert_eq!(bands.last().unwrap()[3], (h*2+gap+1).min(counts.len()));
    }
}

/// 原寸の短い見出しを全ページ幅で検出し、固定モデルへ欠落なく渡す。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn real_short_headings_reach_recognition_on_a_wide_page() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let session = ParseqSession::load(&model, &charset).unwrap();
    for (case, truth, limit) in [("purpose", "（目的）", 2), ("contents", "目次", 0)] {
        let (rgb, width, height) = wide_heading(case);
        let bands = detect_textline_bands_naive(&rgb, width, height, 220);
        assert_eq!(bands.len(), 1, "{case}: {bands:?}");
        let [x0, y0, x1, y1] = bands[0];
        let crop = crop_rgb_u8(&rgb, width, height, BBox::new(x0, y0, x1, y1)).unwrap();
        let actual = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text;
        eprintln!("SHORT HEADING {case}: {actual}");
        // 全角括弧も正解に含め、既知の2誤りを消さない。
        assert_eq!(actual.chars().count(), truth.chars().count());
        assert!(
            actual
                .chars()
                .zip(truth.chars())
                .filter(|(a, b)| a != b)
                .count()
                <= limit,
            "{case}: {actual}"
        );
    }
}

/// 実字形の原寸を維持し、ページ幅の白いcanvasへ配置する。
fn wide_heading(case: &str) -> (Vec<u8>, usize, usize) {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&format!(
        "tests/fixtures/short_lines/{case}.png"
    )))
    .unwrap();
    let width = 2482;
    let mut rgb = vec![255; width * image.height * 3];
    for y in 0..image.height {
        let i = (y * width + 360) * 3;
        rgb[i..i + image.width * 3]
            .copy_from_slice(&image.data[y * image.width * 3..(y + 1) * image.width * 3]);
    }
    (rgb, width, image.height)
}
