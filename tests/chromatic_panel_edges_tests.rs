use ndlocr_lite_rs::pipeline::background::normalize_chromatic_panels;
use proptest::prelude::*;

/// 上部より短い末尾行を持つ色背景と、背景内の白黒字形を生成する。
fn ragged_panel(left: usize, right: usize, tail_height: usize) -> Vec<u8> {
    let mut rgb = vec![255; 320 * 96 * 3];
    for y in 8..80 {
        let (x0, x1) = if y < 80 - tail_height {
            (16, 304)
        } else {
            (left, right)
        };
        for x in x0..x1 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[152, 251, 152]);
        }
    }
    for (x, value) in [(left + 3, 0), (left + 6, 255)] {
        rgb[(75 * 320 + x) * 3..(75 * 320 + x) * 3 + 3].fill(value);
    }
    rgb
}

/// 末尾行の左右にある白紙を白文字と取り違えて黒い帯にしない。
#[test]
fn shorter_tail_keeps_outside_paper_and_inside_glyphs() {
    let rgb = ragged_panel(48, 160, 16);
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    for (x, expected) in [(32, 255), (48, 255), (51, 0), (54, 0), (200, 255)] {
        let i = (75 * 320 + x) * 3;
        assert_eq!(&out[i..i + 3], &[expected; 3], "at {x},75");
    }
}

/// 隣接する別色の背景を列中央値で混ぜ、背景そのものを文字行にしない。
#[test]
fn adjacent_background_colors_keep_separate_glyph_rows() {
    let mut rgb = vec![255; 320 * 120 * 3];
    for y in 8..104 {
        for x in 16..304 {
            let color = if y < 56 {
                [152, 251, 152]
            } else {
                [238, 232, 170]
            };
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&color);
        }
    }
    for y0 in [20, 68] {
        for y in y0..y0 + 20 {
            for x in 40..48 {
                rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].fill(0);
            }
        }
    }
    let out = normalize_chromatic_panels(&rgb, 320, 120).unwrap();
    for y in [10, 58] {
        let i = (y * 320 + 100) * 3;
        assert_eq!(&out[i..i + 3], &[255; 3], "at 100,{y}");
    }
    let bands =
        ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive(&out, 320, 120, 220);
    assert_eq!(bands.len(), 2, "{bands:?}");
}

/// 原寸の短い末尾行に背景外の黒い帯を作らず、白列で安全に分割する。
#[test]
fn public_highlight_tail_remains_safely_splittable() {
    use ndlocr_lite_rs::pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panel_edges/wetlands-tail.png",
    ))
    .unwrap();
    let out = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let i = (170 * image.width + 1000) * 3;
    assert_eq!(&out[i..i + 3], &[255; 3]);
    let bands = detect_textline_bands_naive(&out, image.width, image.height, 220);
    let tail = bands.iter().find(|b| (140..180).contains(&b[1])).unwrap();
    assert!(tail[2] < 800, "{bands:?}");
    let line = crop_rgb_u8(
        &out,
        image.width,
        image.height,
        BBox::new(tail[0], tail[1], tail[2], tail[3]),
    )
    .unwrap();
    assert!(split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).is_ok());
}

proptest! {
    /// 末尾行の左右端と高さが変わっても、外側の画素を変更しない。
    #[test]
    fn ragged_edges_preserve_outside_pixels(left in 16usize..96, right in 160usize..304, tail_height in 8usize..32) {
        let rgb = ragged_panel(left, right, tail_height);
        let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
        for y in 80-tail_height..80 {
            for x in (0..left).chain(right..320) {
                let i = (y*320+x)*3;
                prop_assert_eq!(&out[i..i+3], &rgb[i..i+3]);
            }
        }
    }
}

/// 実モデルでも短いハイライト末尾行を復元し、追加の文字補正を必要としない。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn public_highlight_tail_recognizes_original_text() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            crop::{BBox, crop_rgb_u8},
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
            line_segment::detect_textline_bands_naive,
        },
    };
    let model = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panel_edges/wetlands-tail.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let tail = bands.iter().find(|b| (140..180).contains(&b[1])).unwrap();
    let line = crop_rgb_u8(
        &rgb,
        image.width,
        image.height,
        BBox::new(tail[0], tail[1], tail[2], tail[3]),
    )
    .unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    let session = ParseqSession::load(&model, &charset).unwrap();
    let mut actual = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let crop =
            crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, index).unwrap();
        let result = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        if chunk.insert_space_before(&actual, &result.text, line.height) {
            actual.push(' ');
        }
        actual.push_str(&result.text);
    }
    assert_eq!(actual, "whose growth may clog rivers, foul");
}

/// 隣接する色背景の実ページでも、元の38行を独立した字形の高さで検出する。
#[test]
fn public_adjacent_panels_preserve_all_text_rows() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panel_edges/wetlands-page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive(
        &rgb,
        image.width,
        image.height,
        220,
    );
    assert_eq!(bands.len(), 38, "{bands:?}");
}

/// 長い行が段落の少数でも、その行だけにある背景列を黒帯として残さない。
#[test]
fn minority_long_rows_still_remove_background() {
    let rgb = ragged_panel(48, 160, 60);
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    let i = (10 * 320 + 250) * 3;
    assert_eq!(&out[i..i + 3], &[255; 3]);
}
