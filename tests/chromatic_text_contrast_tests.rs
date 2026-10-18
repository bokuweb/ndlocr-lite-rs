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
        glyph_top in 12usize..24, glyph_bottom in 66usize..79,
    ) {
        let (width,height)=(320*scale+2*padding,96*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 8*scale..80*scale { for x in 16*scale+padding..304*scale+padding {
            rgb[(y*width+x)*3..(y*width+x)*3+3].copy_from_slice(&[high,low,95]);
        }}
        for y in glyph_top*scale..glyph_bottom*scale { for x in 100*scale+padding..112*scale+padding {
            rgb[(y*width+x)*3..(y*width+x)*3+3].copy_from_slice(&[low,high,95]);
        }}
        let out=normalize_chromatic_panels(&rgb,width,height).unwrap();
        for y in glyph_top*scale..glyph_bottom*scale { for x in 100*scale+padding..112*scale+padding {
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

/// 独立した緑背景の紫文字を白地へ変換し、2行すべてを安全に分割する。
#[test]
fn independent_green_panel_keeps_safe_blank_columns() {
    use ndlocr_lite_rs::pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/independent_chromatic_formatting/region.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    assert_eq!(bands.len(), 2, "{bands:?}");
    // 背景上端の字形外画素は白にし、字間を背景で接続しない。
    for [x0, y0, x1, y1] in bands {
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        assert!(
            split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).is_ok(),
            "band {x0},{y0},{x1},{y1}"
        );
    }
}

/// 一列で色文字が縦方向の過半数でも、色文字を背景中央値に混ぜない。
#[test]
fn tall_colored_glyph_does_not_invert_its_background_column() {
    let (width, height) = (320, 96);
    let mut rgb = vec![255; width * height * 3];
    for y in 8..80 {
        for x in 16..304 {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[152, 251, 152]);
        }
    }
    for y in 16..72 {
        for x in 100..112 {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[72, 61, 139]);
        }
    }
    let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
    for y in [10, 76] {
        let i = (y * width + 104) * 3;
        assert_eq!(&out[i..i + 3], &[255; 3], "paper at 104,{y}");
    }
    assert!(
        out[(40 * width + 104) * 3] < 100,
        "colored stem must remain dark"
    );
}

/// 色文字が横の過半数を覆う薄い画線でも、背景行を分割せず白地にする。
#[test]
fn dense_colored_strokes_do_not_split_off_thin_background_rows() {
    let (width, height) = (320, 96);
    let mut rgb = vec![255; width * height * 3];
    for y in 8..80 {
        for x in 16..304 {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[152, 251, 152]);
        }
    }
    for y in 40..43 {
        for x in 16..304 {
            if (x - 16) % 28 < 20 {
                rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[72, 61, 139]);
            }
        }
    }
    let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
    let paper = (41 * width + 40) * 3;
    assert_eq!(&out[paper..paper + 3], &[255; 3]);
    assert!(out[(41 * width + 20) * 3] < 100);
}

/// 固定PARSeqでも、独立した緑背景の行を記号・空白込みで完全一致させる。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn independent_green_panel_recognizes_exactly() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            crop::{BBox, crop_rgb_u8},
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
            line_segment::detect_textline_bands_naive,
        },
    };
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/independent_chromatic_formatting/region.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    assert_eq!(bands.len(), 2);
    let [x0, y0, x1, y1] = bands[1];
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
    assert_eq!(
        actual,
        "4. Using TextFormat.ForeColor and TextFormat.BackColor to colorize the text."
    );
}

/// 広い背景に隣接する全高の色枠は除去し、部分高の色文字は保持する。
#[test]
fn full_height_border_is_removed_without_sampling_tall_glyph() {
    let (width, height) = (320, 96);
    let mut rgb = vec![255; width * height * 3];
    for y in 8..80 {
        for x in 16..304 {
            let value = if !(18..302).contains(&x) {
                [59, 92, 170]
            } else {
                [213, 221, 240]
            };
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&value);
        }
    }
    for y in 16..72 {
        for x in 100..112 {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[72, 61, 139]);
        }
    }
    let out = normalize_chromatic_panels(&rgb, width, height).unwrap();
    for x in [16, 17, 302, 303] {
        assert_eq!(
            &out[(40 * width + x) * 3..(40 * width + x) * 3 + 3],
            &[255; 3]
        );
    }
    assert!(out[(40 * width + 104) * 3] < 100);
}
