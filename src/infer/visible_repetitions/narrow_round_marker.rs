//! Support for a narrow round marker's original raster profile.
pub(super) fn supports_mark(
    mark: &super::Component,
    rgb: &[u8],
    image_width: usize,
    image_height: usize,
    reference_height: usize,
) -> bool {
    if image_width == 0
        || image_height == 0
        || image_width
            .checked_mul(image_height)
            .and_then(|n| n.checked_mul(3))
            != Some(rgb.len())
        || mark.x0 >= mark.x1
        || mark.y0 >= mark.y1
        || mark.x1 > image_width
        || mark.y1 > image_height
    {
        return false;
    }
    let width = mark.x1.saturating_sub(mark.x0);
    let height = mark.y1.saturating_sub(mark.y0);
    let dimensions_supported = width >= reference_height.div_ceil(6).max(3)
        && width < reference_height.div_ceil(4)
        && height as u128 * 4 >= reference_height as u128 * 3
        && height as u128 * 4 <= reference_height as u128 * 5;
    if !dimensions_supported {
        return false;
    }
    let rows: Vec<Vec<usize>> = (mark.y0..mark.y1)
        .map(|y| {
            (mark.x0..mark.x1)
                .filter(|&x| {
                    let i = (y * image_width + x) * 3;
                    rgb[i..i + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660
                })
                .collect()
        })
        .collect();
    if rows.iter().any(|row| {
        row.is_empty()
            || row.len() > (width * 3).div_ceil(4)
            || row.windows(2).any(|p| p[1] != p[0] + 1)
    }) {
        return false;
    }
    let left_half_end = mark.x0 + width.div_ceil(2);
    if *rows.first().unwrap().last().unwrap() >= left_half_end
        || *rows.last().unwrap().last().unwrap() >= left_half_end
    {
        return false;
    }
    let in_right_band =
        |row: &[usize]| row[0] >= mark.x0 + width / 2 && *row.last().unwrap() == mark.x1 - 1;
    let center = height / 2;
    if !in_right_band(&rows[center]) {
        return false;
    }
    let mut top = center;
    while top > 0 && in_right_band(&rows[top - 1]) {
        top -= 1;
    }
    let mut bottom = center + 1;
    while bottom < height && in_right_band(&rows[bottom]) {
        bottom += 1;
    }
    if bottom - top < height.div_ceil(3) {
        return false;
    }
    let foreground = rows.iter().map(Vec::len).sum::<usize>();
    foreground == mark.area
}

#[cfg(test)]
mod tests {
    use super::super::{
        ParseqOrientation, sanitize_visible_repetitions, sanitize_visible_single_line,
    };
    use super::*;

    fn curve(width: usize, height: usize) -> (Vec<u8>, super::super::Component, usize, usize) {
        let (w, h) = (width + 6, height + 6);
        let mut rgb = vec![255; w * h * 3];
        let flank = height / 3;
        for y in 0..height {
            let end_distance = y.min(height - y - 1);
            let x = (end_distance * (width - 1) / flank).min(width - 1);
            rgb[((y + 3) * w + x + 3) * 3..((y + 3) * w + x + 4) * 3].fill(0);
        }
        let (mut parts, _) = super::super::collect_components(&rgb, w, h).unwrap();
        assert_eq!(
            parts.len(),
            1,
            "the complete curve is one original component"
        );
        let mark = parts.remove(0);
        assert_eq!((mark.x1 - mark.x0, mark.y1 - mark.y0), (width, height));
        (rgb, mark, w, h)
    }

    #[test]
    fn narrow_widths_keep_the_inclusive_lower_and_exclusive_upper_bounds() {
        for (width, height, body_height, expected) in [
            (3, 12, 13, true),
            (4, 19, 20, true),
            (4, 24, 24, true),
            (5, 24, 24, true),
            (2, 12, 13, false),
            (3, 19, 20, false),
            (6, 24, 24, false),
        ] {
            let (rgb, mark, w, h) = curve(width, height);
            assert_eq!(
                supports_mark(&mark, &rgb, w, h, body_height),
                expected,
                "width {width}, height {height}, body {body_height}"
            );
        }
    }

    #[test]
    fn marker_height_stays_between_three_quarters_and_five_quarters_of_the_body() {
        for (height, expected) in [(17, false), (18, true), (30, true), (31, false)] {
            let (rgb, mark, w, h) = curve(4, height);
            assert_eq!(
                supports_mark(&mark, &rgb, w, h, 24),
                expected,
                "height {height}"
            );
        }
    }

    #[test]
    fn invalid_rgb_shapes_and_outside_boxes_do_not_support_a_marker() {
        let (rgb, mut mark, w, h) = curve(4, 19);
        for (bytes, width, height) in [
            (&rgb[..rgb.len() - 1], w, h),
            (rgb.as_slice(), 0, h),
            (rgb.as_slice(), w, 0),
            (rgb.as_slice(), usize::MAX, usize::MAX),
        ] {
            assert!(!supports_mark(&mark, bytes, width, height, 20));
        }
        mark.x0 += w;
        mark.x1 += w;
        assert!(!supports_mark(&mark, &rgb, w, h, 20));
        let (rgb, mut mark, w, h) = curve(4, 19);
        mark.y0 += h;
        mark.y1 += h;
        assert!(!supports_mark(&mark, &rgb, w, h, 20));
    }

    #[test]
    fn the_mark_area_must_own_every_foreground_pixel_in_its_box() {
        let (mut rgb, mut mark, w, h) = curve(4, 19);
        let before = rgb.clone();
        mark.area += 1;
        assert!(
            !supports_mark(&mark, &rgb, w, h, 20),
            "metadata must match the original pixels"
        );
        mark.area -= 1;
        assert!(supports_mark(&mark, &rgb, w, h, 20));
        let y = mark.y0 + (mark.y1 - mark.y0) / 2;
        let x = mark.x1 - 2;
        rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
        assert!(
            !supports_mark(&mark, &rgb, w, h, 20),
            "an extra neighboring point has no owner"
        );
        assert_ne!(rgb, before, "only the test itself added a pixel");
    }

    #[test]
    fn filled_rectangles_and_a_connected_fork_are_not_single_thin_row_profiles() {
        let (rgb, mark, w, h) = curve(1, 19);
        assert!(
            !supports_mark(&mark, &rgb, w, h, 20),
            "a one-pixel straight rule is too narrow"
        );
        let (mut rgb, mark, w, h) = curve(4, 19);
        for y in mark.y0..mark.y1 {
            for x in mark.x0..mark.x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
        let (parts, _) = super::super::collect_components(&rgb, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &rgb, w, h, 20),
            "a solid bar is not a round marker"
        );

        let (mut rgb, mark, w, h) = curve(5, 24);
        let y = mark.y0 + 12;
        for (x, yy) in [(mark.x0 + 1, y), (mark.x0 + 2, y - 1), (mark.x0 + 3, y - 1)] {
            rgb[(yy * w + x) * 3..(yy * w + x + 1) * 3].fill(0);
        }
        let (parts, _) = super::super::collect_components(&rgb, w, h).unwrap();
        assert_eq!(
            parts.len(),
            1,
            "both branches belong to the same eight-connected component"
        );
        assert!(
            !supports_mark(&parts[0], &rgb, w, h, 24),
            "one row has two disjoint intervals"
        );
    }

    #[test]
    fn an_empty_row_in_malformed_component_metadata_is_rejected() {
        let (mut rgb, mut mark, w, h) = curve(4, 19);
        let y = mark.y0 + 9;
        for x in mark.x0..mark.x1 {
            if rgb[(y * w + x) * 3] == 0 {
                mark.area -= 1;
            }
            rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(255);
        }
        assert!(!supports_mark(&mark, &rgb, w, h, 20));
    }

    #[test]
    fn left_facing_curves_and_diagonal_rules_do_not_have_two_left_ends() {
        let (original, mark, w, h) = curve(4, 19);
        let mut mirrored = original.clone();
        for y in mark.y0..mark.y1 {
            for x in mark.x0..mark.x1 {
                let source = (y * w + mark.x1 - 1 - (x - mark.x0)) * 3;
                mirrored[(y * w + x) * 3..(y * w + x + 1) * 3]
                    .copy_from_slice(&original[source..source + 3]);
            }
        }
        let (parts, _) = super::super::collect_components(&mirrored, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &mirrored, w, h, 20),
            "a left parenthesis faces the other way"
        );

        let mut diagonal = vec![255; original.len()];
        for y in 0..19 {
            let x = mark.x0 + y * 3 / 18;
            let yy = mark.y0 + y;
            diagonal[(yy * w + x) * 3..(yy * w + x + 1) * 3].fill(0);
        }
        let (parts, _) = super::super::collect_components(&diagonal, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &diagonal, w, h, 20),
            "one end of the straight slope is on the right"
        );
    }

    #[test]
    fn the_end_row_can_reach_the_last_left_half_column_but_not_the_next() {
        let (mut rgb, mark, w, h) = curve(4, 19);
        let first_row = mark.y0 * w;
        rgb[(first_row + mark.x0) * 3..(first_row + mark.x0 + 1) * 3].fill(255);
        rgb[(first_row + mark.x0 + 1) * 3..(first_row + mark.x0 + 2) * 3].fill(0);
        assert!(supports_mark(&mark, &rgb, w, h, 20));
        rgb[(first_row + mark.x0 + 1) * 3..(first_row + mark.x0 + 2) * 3].fill(255);
        rgb[(first_row + mark.x0 + 2) * 3..(first_row + mark.x0 + 3) * 3].fill(0);
        let next_row = (mark.y0 + 1) * w;
        rgb[(next_row + mark.x0) * 3..(next_row + mark.x0 + 1) * 3].fill(255);
        rgb[(next_row + mark.x0 + 1) * 3..(next_row + mark.x0 + 2) * 3].fill(0);
        let (parts, _) = super::super::collect_components(&rgb, w, h).unwrap();
        assert_eq!(
            parts.len(),
            1,
            "the changed end is still connected to the curve"
        );
        assert!(!supports_mark(&parts[0], &rgb, w, h, 20));
    }

    #[test]
    fn the_central_right_band_needs_one_third_of_the_height() {
        let (mut rgb, mark, w, h) = curve(4, 19);
        assert!(
            supports_mark(&mark, &rgb, w, h, 20),
            "the seven-row band includes the boundary"
        );
        let y = mark.y0 + 6;
        rgb[(y * w + mark.x1 - 1) * 3..(y * w + mark.x1) * 3].fill(255);
        rgb[(y * w + mark.x1 - 2) * 3..(y * w + mark.x1 - 1) * 3].fill(0);
        let (parts, _) = super::super::collect_components(&rgb, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &rgb, w, h, 20),
            "a six-row band is too short"
        );
    }

    #[test]
    fn a_chevron_and_a_long_band_away_from_the_center_are_not_round_profiles() {
        let (template, mark, w, h) = curve(4, 19);
        let mut chevron = vec![255; template.len()];
        for y in 0usize..19 {
            let x = (y.min(18 - y) * 3 + 4) / 9;
            let i = ((mark.y0 + y) * w + mark.x0 + x) * 3;
            chevron[i..i + 3].fill(0);
        }
        let (parts, _) = super::super::collect_components(&chevron, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &chevron, w, h, 20),
            "a pointed slope has only a short apex"
        );

        let mut off_center = vec![255; template.len()];
        for y in 0usize..19 {
            let xs: &[usize] = match y {
                1 | 10 => &[1],
                2 => &[2, 3],
                3..=8 => &[3],
                9 => &[2],
                _ => &[0],
            };
            for &x in xs {
                let i = ((mark.y0 + y) * w + mark.x0 + x) * 3;
                off_center[i..i + 3].fill(0);
            }
        }
        let (parts, _) = super::super::collect_components(&off_center, w, h).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(
            !supports_mark(&parts[0], &off_center, w, h, 20),
            "the seven-row band must contain the center row"
        );
    }

    #[test]
    fn the_foreground_sum_boundary_matches_the_original_collector() {
        let (mut rgb, mark, w, h) = curve(4, 19);
        for pixel in rgb.chunks_mut(3) {
            if pixel.iter().all(|&v| v == 0) {
                pixel.copy_from_slice(&[219, 220, 220]);
            }
        }
        assert!(
            supports_mark(&mark, &rgb, w, h, 20),
            "sum659 remains foreground"
        );
        for pixel in rgb.chunks_mut(3) {
            if pixel == [219, 220, 220] {
                pixel[2] = 221;
            }
        }
        assert!(
            !supports_mark(&mark, &rgb, w, h, 20),
            "sum660 no longer supports the marker"
        );
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn mark_profile_survives_translation_paper_gray_and_integer_scale(
            scale in 1usize..4, mx in 0usize..9, my in 0usize..9,
            paper in 0usize..13, tone in 0u8..220,
        ) {
            let original = crate::io::load_rgb_u8(std::path::Path::new(
                "tests/fixtures/owned_parts_body_bridge/narrow_round_prefix_mincho20.png"
            )).unwrap();
            let w = original.width * scale + mx + 3;
            let h = original.height * scale + my + 2 * paper + 3;
            let mut rgb = vec![255; w * h * 3];
            for y in 0..original.height {
                for x in 0..original.width {
                    let source = (y * original.width + x) * 3;
                    if original.data[source..source + 3].iter().map(|&v| u16::from(v)).sum::<u16>() >= 660 {
                        continue;
                    }
                    for yy in 0..scale {
                        for xx in 0..scale {
                            let i = ((y * scale + my + paper + yy) * w + x * scale + mx + xx) * 3;
                            rgb[i..i + 3].fill(tone);
                        }
                    }
                }
            }
            let (components, _) = super::super::collect_components(&rgb, w, h).unwrap();
            let mark = components.first().unwrap();
            proptest::prop_assert_eq!((mark.x1-mark.x0, mark.y1-mark.y0, mark.area), (4*scale, 19*scale, 35*scale*scale));
            let before = rgb.clone();
            proptest::prop_assert!(supports_mark(mark, &rgb, w, h, 20*scale));
            proptest::prop_assert_eq!(rgb, before);
        }
    }

    #[test]
    fn the_known_narrow_round_marker_is_supported_without_supporting_the_corner() {
        for (file, leading, expected) in [
            ("narrow_round_prefix_mincho20", true, true),
            ("info_corner_mincho20", false, false),
        ] {
            let image = crate::io::load_rgb_u8(std::path::Path::new(&format!(
                "tests/fixtures/owned_parts_body_bridge/{file}.png"
            )))
            .unwrap();
            let before = image.data.clone();
            let (components, _) =
                super::super::collect_components(&image.data, image.width, image.height).unwrap();
            let mark = if leading {
                components.first()
            } else {
                components.last()
            }
            .unwrap();
            assert_eq!(
                supports_mark(mark, &image.data, image.width, image.height, 20),
                expected,
                "{file}"
            );
            assert_eq!(image.data, before);
        }
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_NARROW_ROUND_ROOT with frozen baseline pairs"]
    fn record_narrow_round_policy() {
        let Ok(root) = std::env::var("NDLOCR_NARROW_ROUND_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        let output = root.join("replay");
        std::fs::create_dir(&output).unwrap();
        let expected_cases = m["expected_cases"].as_u64().unwrap_or(137991) as usize;
        assert_eq!(m["cases"].as_array().unwrap().len(), expected_cases);
        let mut records = Vec::new();
        for x in m["cases"].as_array().unwrap() {
            let id = x["id"].as_str().unwrap();
            assert!(
                id.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
            );
            let orientation = match x["orientation"].as_str() {
                None | Some("auto") => ParseqOrientation::Auto,
                Some("horizontal") => ParseqOrientation::Horizontal,
                other => panic!("invalid orientation {other:?}"),
            };
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let raw = x["raw"].as_str().unwrap();
            let baseline = sanitize_visible_single_line(raw, &rgb, w, h, orientation);
            assert_eq!(baseline, x["expected_after"].as_str().unwrap(), "{id}");
            let fragment = sanitize_visible_repetitions(raw, &rgb, w, h, orientation);
            let selected = fragment.chars().count() > baseline.chars().count()
                && (orientation == ParseqOrientation::Horizontal || h as u128 * 5 <= w as u128 * 4)
                && super::super::owned_parts_body_bridge::supports_narrow_round(
                    &fragment, &rgb, w, h,
                );
            let candidate = if selected { &fragment } else { &baseline };
            assert_eq!(rgb, before);
            for (phase, text) in [("baseline", &baseline), ("candidate", candidate)] {
                let dir = output
                    .join(x["corpus"].as_str().unwrap())
                    .join(x["model"].as_str().unwrap())
                    .join(phase);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(format!("{id}.txt")), text).unwrap();
            }
            records.push(serde_json::json!({"corpus":x["corpus"],"model":x["model"],"id":id,"raw":raw,"baseline":baseline,"candidate":candidate,"selected":selected}));
        }
        std::fs::write(
            output.join("records.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        println!("{} fixed raw pairs evaluated;no NN forwards", records.len());
    }
    #[test]
    #[ignore = "diagnostic; validates every fresh original/effective RGB tensor before NN"]
    fn verify_fresh_narrow_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_NARROW_ROUND_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 8064);
        for x in rows {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let effective = std::fs::read(x["effective_rgb"].as_str().unwrap()).unwrap();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let (fw, fh) = (
                x["forward_width"].as_u64().unwrap() as usize,
                x["forward_height"].as_u64().unwrap() as usize,
            );
            for iw in [256, 384, 768] {
                let a =
                    preprocess_rgb_u8_with_orientation(&rgb, w, h, iw, 24, ParseqOrientation::Auto)
                        .unwrap();
                let b = preprocess_rgb_u8_with_orientation(
                    &effective,
                    fw,
                    fh,
                    iw,
                    24,
                    ParseqOrientation::Horizontal,
                )
                .unwrap();
                assert_eq!(a.len(), 3 * iw * 24);
                assert_eq!(b.len(), a.len());
                assert!(
                    a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits()),
                    "{}/width{iw}",
                    x["id"]
                );
            }
        }
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":8064,"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_NARROW_ROUND_PRODUCT_MANIFEST with frozen candidate strings"]
    fn verify_product_matches_frozen_narrow_round_policy() {
        let Ok(path) = std::env::var("NDLOCR_NARROW_ROUND_PRODUCT_MANIFEST") else {
            return;
        };
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = m["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 162915);
        for x in cases {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let orientation = match x["orientation"].as_str() {
                Some("horizontal") => ParseqOrientation::Horizontal,
                None | Some("auto") => ParseqOrientation::Auto,
                other => panic!("invalid orientation {other:?}"),
            };
            let actual =
                sanitize_visible_single_line(x["raw"].as_str().unwrap(), &rgb, w, h, orientation);
            assert_eq!(
                actual,
                x["expected_after"].as_str().unwrap(),
                "{}/{}/{}",
                x["corpus"],
                x["model"],
                x["id"]
            );
        }
        println!(
            "{} fixed neural pairs match the actual product policy",
            cases.len()
        );
    }
}
