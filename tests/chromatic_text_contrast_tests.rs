use ndlocr_lite_rs::pipeline::background::normalize_chromatic_panels;

/// 同じRGB平均の異色の字形を、背景と取り違えて消さない。
#[test]
fn equal_brightness_colored_glyph_is_preserved() {
    let mut rgb = vec![255; 320 * 96 * 3];
    for y in 8..80 {
        for x in 16..304 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[0, 160, 95]);
        }
    }
    for y in 28..56 {
        for x in 100..112 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[160, 0, 95]);
        }
    }
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    let i = (40 * 320 + 104) * 3;
    assert!(
        out[i] < 100,
        "different-colored glyph: {:?}",
        &out[i..i + 3]
    );
    assert_eq!(
        &out[(40 * 320 + 50) * 3..(40 * 320 + 50) * 3 + 3],
        &[255; 3]
    );
}

/// 同じ行の背景外にある色文字を消さず、その字間の白紙も反転しない。
#[test]
fn colored_text_outside_partial_panel_is_unchanged() {
    let mut rgb = vec![255; 320 * 96 * 3];
    for y in 8..80 {
        let left = if y < 48 { 160 } else { 16 };
        for x in left..304 {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[255, 165, 0]);
        }
    }
    for y in 20..40 {
        for x in [32..40, 64..72, 144..152].into_iter().flatten() {
            rgb[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&[255, 99, 71]);
        }
    }
    let out = normalize_chromatic_panels(&rgb, 320, 96).unwrap();
    for y in 8..48 {
        for x in 0..160 {
            let i = (y * 320 + x) * 3;
            assert_eq!(&out[i..i + 3], &rgb[i..i + 3], "at {x},{y}");
        }
    }
}

/// 原寸の混在書式ページから16行を検出し、全行を白列で安全に分割する。
#[test]
fn public_mixed_formats_keep_all_splittable_rows() {
    use ndlocr_lite_rs::pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    assert_eq!(bands.len(), 16, "{bands:?}");
    for [x0, y0, x1, y1] in bands {
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        assert!(
            split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).is_ok(),
            "band {x0},{y0},{x1},{y1}"
        );
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 色成分を交換して明度が等しくても、拡大・位置ずれ後の字形を消さない。
    #[test]
    fn equal_brightness_contrast_survives_scale_and_translation(
        high in 160u8..=255, low in 0u8..=40,
        scale in 1usize..4, padding in 0usize..9,
    ) {
        let (width,height)=(320*scale+2*padding,96*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 8*scale..80*scale { for x in 16*scale+padding..304*scale+padding {
            rgb[(y*width+x)*3..(y*width+x)*3+3].copy_from_slice(&[high,low,95]);
        }}
        for y in 28*scale..56*scale { for x in 100*scale+padding..112*scale+padding {
            rgb[(y*width+x)*3..(y*width+x)*3+3].copy_from_slice(&[low,high,95]);
        }}
        let out=normalize_chromatic_panels(&rgb,width,height).unwrap();
        for y in 28*scale..56*scale { for x in 100*scale+padding..112*scale+padding {
            proptest::prop_assert!(out[(y*width+x)*3] < 100);
        }}
        for y in 0..height { for x in (0..16*scale+padding).chain(304*scale+padding..width) {
            let i=(y*width+x)*3;
            proptest::prop_assert_eq!(&out[i..i+3],&rgb[i..i+3]);
        }}
    }
}

/// 実モデルでも、背景外の赤文字と背景内の紫文字を同一行で認識できる。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn public_mixed_color_line_recognizes_without_replacements() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            crop::{BBox, crop_rgb_u8},
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
            line_segment::detect_textline_bands_naive,
        },
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let [x0, y0, x1, y1] = *bands
        .iter()
        .find(|band| (1390..1420).contains(&band[1]))
        .unwrap();
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    let session = ParseqSession::load(
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
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
    assert!(
        actual.starts_with("color Color [Transparent]. This is bold "),
        "{actual}"
    );
    assert!(actual.ends_with("text drawn using"), "{actual}");
}
