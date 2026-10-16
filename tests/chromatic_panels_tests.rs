use ndlocr_lite_rs::pipeline::background::normalize_chromatic_panels;
use ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive;
use proptest::prelude::*;
use std::borrow::Cow;

/// 青い横長パネルの両極性の文字と、外側の本文を固定する。
fn panel() -> Vec<u8> {
    let mut rgb = vec![255; 320 * 96 * 3];
    for y in 8..56 {
        for x in 16..304 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[
                20 + (x / 4) as u8,
                60 + (x / 4) as u8,
                150 + (x / 4) as u8,
            ]);
        }
    }
    for (x0, color) in [(32, 255), (128, 0)] {
        for y in 20..40 {
            for x in x0..x0 + 8 {
                rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].fill(color);
            }
        }
    }
    rgb[(80 * 320 + 50) * 3..(80 * 320 + 50) * 3 + 3].fill(0);
    rgb
}

/// 背景を除き、白文字も黒文字も同じ座標の暗い字形へ変える。
#[test]
fn gradient_panel_preserves_both_text_polarities_and_outside_pixels() {
    let rgb = panel();
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    assert_eq!(out.len(), rgb.len());
    for (x, y, expected) in [(20, 30, 255), (36, 30, 0), (132, 30, 0), (50, 80, 0)] {
        let i = (y * 320 + x) * 3;
        assert_eq!(&out[i..i + 3], &[expected; 3], "at {x},{y}");
    }
    assert_eq!(&out[56 * 320 * 3..], &rgb[56 * 320 * 3..]);
}

/// 元の実画像では背景の高さを行高と取り違え、字形の行に分離できない。
#[test]
fn public_gradient_header_is_detected_at_glyph_height() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panels/header.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    assert_eq!(bands.len(), 1);
    let [x0, y0, x1, y1] = bands[0];
    assert!(x0 <= 46 && x1 >= 1060, "{bands:?}");
    assert!(
        (45..=60).contains(&y0) && (110..=125).contains(&y1),
        "{bands:?}"
    );
}

/// 単独の色文字や細い罫線を背景として消さない。
#[test]
fn colored_glyphs_and_thin_rule_are_unchanged() {
    let mut rgb = vec![255; 320 * 96 * 3];
    for y in 20..40 {
        for x in 30..40 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[0, 80, 180]);
        }
    }
    for x in 10..310 {
        rgb[(70 * 320 + x) * 3..(70 * 320 + x) * 3 + 3].copy_from_slice(&[0, 80, 180]);
    }
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    assert!(matches!(out, Cow::Borrowed(_)));
    assert_eq!(&*out, &rgb);
}

/// サイズ不一致を画素アクセス前に拒否する。
#[test]
fn malformed_dimensions_are_rejected() {
    for (rgb, w, h) in [
        (&[][..], 0, 1),
        (&[][..], usize::MAX, 2),
        (&[255][..], 1, 1),
    ] {
        assert!(normalize_chromatic_panels(rgb, w, h).is_err());
    }
}

proptest! {
    /// 灰色画像の全画素と借用を維持し、従来の認識を変えない。
    #[test]
    fn achromatic_pages_remain_borrowed(gray in prop::collection::vec(any::<u8>(), 64..512)) {
        let row: Vec<_> = gray.iter().flat_map(|&g| [g; 3]).collect();
        let rgb = row.repeat(32);
        let out = normalize_chromatic_panels(&rgb, gray.len(), 32).unwrap();
        prop_assert!(matches!(out, Cow::Borrowed(_)));
        prop_assert_eq!(&*out, &rgb);
    }
}

/// 固定した実モデルで正解の白文字を読み、文字置換を必要としない。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn real_gradient_header_recognizes_without_text_correction() {
    use ndlocr_lite_rs::infer::cached::ParseqSession;
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use ndlocr_lite_rs::pipeline::line_chunks::split_line;
    use std::path::PathBuf;
    let model =
        PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"));
    let charset = PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"));
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panels/header.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let session = ParseqSession::load(&model, &charset).unwrap();
    let mut text = String::new();
    for [x0, y0, x1, y1] in detect_textline_bands_naive(&rgb, image.width, image.height, 220) {
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        for chunk in split_line(&line.data, line.width, line.height, 220).unwrap() {
            let crop = crop_rgb_u8(
                &line.data,
                line.width,
                line.height,
                BBox::new(chunk.x0, 0, chunk.x1, line.height),
            )
            .unwrap();
            let result = session
                .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                .unwrap();
            if chunk.insert_space_before(&text, &result.text, line.height) {
                text.push(' ');
            }
            text.push_str(&result.text);
        }
    }
    let expected = std::fs::read_to_string("tests/fixtures/chromatic_panels/truth.txt").unwrap();
    eprintln!("COLOR HEADER: {text}");
    // 固定モデルは全角括弧をASCIIで返す。本文の字形改善と残る2誤りを区別する。
    assert!(text.starts_with("電子入札運用基準 "), "{text}");
    let truth: Vec<_> = expected.trim_end().chars().collect();
    let actual: Vec<_> = text.chars().collect();
    assert_eq!(actual.len(), truth.len());
    let substitutions = actual.iter().zip(&truth).filter(|(a, b)| a != b).count();
    assert!(
        substitutions <= 2,
        "{substitutions}/{}: {text}",
        truth.len()
    );
}

/// 幅制限の直前に単語境界がないとき、末尾の漢字1字だけを孤立させない。
#[test]
fn gradient_header_chunks_do_not_isolate_last_heading_glyph() {
    use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8};
    use ndlocr_lite_rs::pipeline::line_chunks::split_line;
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_panels/header.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let band = detect_textline_bands_naive(&rgb, image.width, image.height, 220)[0];
    let line = crop_rgb_u8(
        &rgb,
        image.width,
        image.height,
        BBox::new(band[0], band[1], band[2], band[3]),
    )
    .unwrap();
    let chunks = split_line(&line.data, line.width, line.height, 220).unwrap();
    // 見出しと括弧の間の実空白は細い括弧の字幅余白で消さない。
    assert!(
        chunks.iter().any(|c| c.x0 == 551 && c.space_before),
        "{chunks:?}"
    );
    assert!(
        chunks.iter().all(|c| c.x1 - c.x0 >= line.height * 2),
        "{chunks:?}"
    );
}

proptest! {
    /// 背景色の端値を含め、明暗両極性を消さずに同じ位置へ保つ。
    #[test]
    fn varied_chromatic_backgrounds_keep_black_and_white_glyphs(color in any::<[u8; 3]>()) {
        let max = *color.iter().max().unwrap();
        let min = *color.iter().min().unwrap();
        prop_assume!(max - min >= 24);
        let mut rgb = panel();
        for y in 8..56 {
            for x in 16..304 {
                let i = (y * 320 + x) * 3;
                if !((20..40).contains(&y) && ((32..40).contains(&x) || (128..136).contains(&x))) {
                    rgb[i..i+3].copy_from_slice(&color);
                }
            }
        }
        let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
        for (x,y,expected) in [(20,30,255),(36,30,0),(132,30,0)] {
            let i = (y * 320 + x) * 3;
            prop_assert_eq!(&out[i..i+3], &[expected;3]);
        }
    }
}
