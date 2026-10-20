//! Support for an owned pair and a separate narrow glyph cell.

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    let chars: Vec<_> = text.chars().take(5).collect();
    let round = |c| matches!(c, ')' | '）');
    if chars.len() != 4
        || chars.iter().filter(|&&c| round(c)).count() != 1
        || chars
            .iter()
            .filter(|&&c| !round(c))
            .any(|c| !c.is_ascii_alphabetic())
    {
        return false;
    }
    let leading = round(chars[0]);
    if !leading && !round(chars[3]) {
        return false;
    }
    let Some((mut body, _)) = super::collect_components(rgb, width, height) else {
        return false;
    };
    if body.len() != 3 {
        return false;
    }
    let mark = if leading {
        body.remove(0)
    } else {
        body.pop().unwrap()
    };
    if body[0].x1 > body[1].x0 {
        return false;
    }
    let body_height = body.iter().map(|c| c.y1 - c.y0).max().unwrap();
    let body_start = body.iter().map(|c| c.x0).min().unwrap();
    let body_end = body.iter().map(|c| c.x1).max().unwrap();
    let gap = if leading {
        body_start.checked_sub(mark.x1)
    } else {
        mark.x0.checked_sub(body_end)
    };
    let Some(gap) = gap else { return false };
    if gap > body_height.div_ceil(4) {
        return false;
    }
    let mark_width = mark.x1 - mark.x0;
    if mark_width as u128 * 4 < body_height as u128 || mark_width as u128 * 2 > body_height as u128
    {
        return false;
    }
    let mark_height = mark.y1 - mark.y0;
    if mark_height as u128 * 4 < body_height as u128 * 3
        || mark_height as u128 * 2 > body_height as u128 * 3
    {
        return false;
    }
    // Check the marker's own narrow aspect and outline here. Its relation to
    // the body height is a separate constraint for this Latin view.
    if !super::narrow_round_marker::supports_mark(&mark, rgb, width, height, mark_height) {
        return false;
    }
    let mut pair = None;
    for (index, c) in body.iter().enumerate() {
        let (w, h) = (c.x1 - c.x0, c.y1 - c.y0);
        let mut mask = Vec::with_capacity(w * h);
        for y in c.y0..c.y1 {
            for x in c.x0..c.x1 {
                let p = (y * width + x) * 3;
                mask.push(rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660);
            }
        }
        if mask.iter().filter(|&&v| v).count() != c.area {
            return false;
        }
        if let Some(cells) = super::latin_pair_bridge::split_pair(&mask, w, h, body_height) {
            if pair.is_some() {
                return false;
            }
            pair = Some((index, cells));
        }
    }
    let Some((index, [mut left, mut right])) = pair else {
        return false;
    };
    for c in [&mut left, &mut right] {
        c.x0 += body[index].x0;
        c.x1 += body[index].x0;
        c.y0 += body[index].y0;
        c.y1 += body[index].y0;
    }
    let singleton = body.remove(1 - index);
    let singleton_width = singleton.x1 - singleton.x0;
    if singleton_width as u128 * 2 < body_height as u128
        || singleton_width as u128 * 2 > body_height as u128 * 3
    {
        return false;
    }
    let mut views = if index == 0 {
        vec![left, right, singleton]
    } else {
        vec![singleton, left, right]
    };
    if views
        .windows(2)
        .any(|p| p[0].x1 > p[1].x0 || p[1].x0 - p[0].x1 > body_height.div_ceil(4))
    {
        return false;
    }
    if leading {
        views.insert(0, mark);
    } else {
        views.push(mark);
    }
    super::supports_collected(&views, body_height, 4, Some(text))
}

#[cfg(test)]
mod tests {
    use super::super::sanitize_visible_repetitions;
    use super::super::{ParseqOrientation, sanitize_visible_single_line};
    use super::*;

    fn curve_variant(mark_width: usize, mark_height: usize) -> (Vec<u8>, usize, usize) {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let mut image = crate::io::load_rgb_u8(path).unwrap();
        assert!(62 + mark_width <= image.width && 8 + mark_height <= image.height);
        for y in 8..31 {
            for x in 62..67 {
                let p = (y * image.width + x) * 3;
                image.data[p..p + 3].fill(255);
            }
        }
        let flank = mark_height / 3;
        for y in 0..mark_height {
            let edge = y.min(mark_height - y - 1);
            let x = (edge * (mark_width - 1) / flank).min(mark_width - 1);
            let p = ((y + 8) * image.width + x + 62) * 3;
            image.data[p..p + 3].fill(0);
        }
        (image.data, image.width, image.height)
    }

    fn shifted_box(rgb: &[u8], width: usize, bbox: [usize; 4], new_x: usize) -> Vec<u8> {
        let mut result = rgb.to_vec();
        let [x0, y0, x1, y1] = bbox;
        assert!(new_x + x1 - x0 <= width);
        for y in y0..y1 {
            for x in x0..x1 {
                result[(y * width + x) * 3..(y * width + x + 1) * 3].fill(255);
            }
        }
        for y in y0..y1 {
            for x in x0..x1 {
                let from = (y * width + x) * 3;
                let to = (y * width + new_x + x - x0) * 3;
                result[to..to + 3].copy_from_slice(&rgb[from..from + 3]);
            }
        }
        result
    }

    fn singleton_rectangle(singleton_width: usize) -> (Vec<u8>, usize, usize) {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let source = crate::io::load_rgb_u8(path).unwrap();
        let width = source.width + 20;
        let mut rgb = vec![255; width * source.height * 3];
        for y in 0..source.height {
            let to = (y * width + 20) * 3;
            let from = y * source.width * 3;
            rgb[to..to + source.width * 3]
                .copy_from_slice(&source.data[from..from + source.width * 3]);
        }
        for y in 9..26 {
            for x in 29..45 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(255);
            }
            for x in 45 - singleton_width..45 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
        (rgb, width, source.height)
    }

    #[test]
    fn an_owned_pair_and_singleton_support_the_existing_round_fragment() {
        for (file, text) in [
            ("klm_round_suffix_mincho24.png", "KLM)"),
            ("klm_round_prefix_mincho24.png", ")KLM"),
            ("xyz_round_suffix_mincho.png", "XYZ)"),
        ] {
            let path = std::path::Path::new("tests/fixtures/latin_pair_bridge").join(file);
            let image = crate::io::load_rgb_u8(&path).unwrap();
            let before = image.data.clone();
            assert_eq!(
                sanitize_visible_single_line(
                    text,
                    &image.data,
                    image.width,
                    image.height,
                    ParseqOrientation::Auto
                ),
                text,
                "the product keeps the supported original fragment for {file}"
            );
            assert!(
                supports(text, &image.data, image.width, image.height),
                "{file}"
            );
            assert_eq!(image.data, before);
        }
    }

    #[test]
    fn a_filled_rectangle_at_the_marker_bbox_does_not_support_a_round_closer() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let mut image = crate::io::load_rgb_u8(path).unwrap();
        for y in 8..31 {
            for x in 62..67 {
                let p = (y * image.width + x) * 3;
                image.data[p..p + 3].fill(0);
            }
        }
        let before = image.data.clone();
        assert!(!supports("KLM)", &image.data, image.width, image.height));
        assert_eq!(image.data, before);
    }

    #[test]
    fn the_marker_width_is_relative_to_the_body_as_well_as_its_own_height() {
        for (mark_width, expected) in [(5, true), (4, false)] {
            let (rgb, width, height) = curve_variant(mark_width, 23);
            assert_eq!(
                supports("KLM)", &rgb, width, height),
                expected,
                "marker width {mark_width}"
            );
        }
    }

    #[test]
    fn the_marker_height_does_not_exceed_three_halves_of_the_body_height() {
        for (mark_height, expected) in [(25, true), (26, false)] {
            let (rgb, width, height) = curve_variant(6, mark_height);
            assert_eq!(
                supports("KLM)", &rgb, width, height),
                expected,
                "marker height {mark_height}"
            );
        }
    }

    #[test]
    fn the_marker_gap_cannot_exceed_one_quarter_of_the_body_height_rounded_up() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let image = crate::io::load_rgb_u8(path).unwrap();
        for (shift, expected) in [(4, true), (5, false)] {
            let rgb = shifted_box(&image.data, image.width, [62, 8, 67, 31], 62 + shift);
            assert_eq!(
                supports("KLM)", &rgb, image.width, image.height),
                expected,
                "gap {}",
                1 + shift
            );
        }
    }

    #[test]
    fn each_body_gap_stays_within_one_quarter_of_the_body_height_rounded_up() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let image = crate::io::load_rgb_u8(path).unwrap();
        for (shift, expected) in [(4, true), (5, false)] {
            let rgb = shifted_box(&image.data, image.width, [9, 9, 25, 26], 9 - shift);
            assert_eq!(
                supports("KLM)", &rgb, image.width, image.height),
                expected,
                "body gap {}",
                1 + shift
            );
        }
    }

    #[test]
    fn the_singleton_uses_the_same_cell_width_range_as_the_split_pair() {
        for (singleton_width, expected) in [(9, true), (8, false), (25, true), (26, false)] {
            let (rgb, width, height) = singleton_rectangle(singleton_width);
            assert_eq!(
                supports("KLM)", &rgb, width, height),
                expected,
                "singleton width {singleton_width}"
            );
        }
    }

    #[test]
    fn wrong_text_positions_scripts_and_closers_keep_the_existing_policy() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let image = crate::io::load_rgb_u8(path).unwrap();
        for text in [
            "", "KLM", ")KLM", "KL)M", "KLM))", "KLM」", "K情M)", "KL )", " KLM)", "AB1)",
        ] {
            assert!(
                !supports(text, &image.data, image.width, image.height),
                "{text:?}"
            );
        }
        assert!(supports("KLM）", &image.data, image.width, image.height));
        for file in [
            "xyz_round_thick_jpeg.png",
            "xyz_round_overlapping_gothic_jpeg.png",
        ] {
            let image = crate::io::load_rgb_u8(
                &std::path::Path::new("tests/fixtures/latin_pair_bridge").join(file),
            )
            .unwrap();
            assert!(
                !supports("XYZ)", &image.data, image.width, image.height),
                "{file}"
            );
        }
    }

    #[test]
    fn a_filtered_tiny_island_inside_the_pair_bbox_is_not_silently_discarded() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let mut image = crate::io::load_rgb_u8(path).unwrap();
        for y in 14..17 {
            for x in 33..36 {
                let p = (y * image.width + x) * 3;
                assert!(
                    image.data[p..p + 3]
                        .iter()
                        .map(|&v| u16::from(v))
                        .sum::<u16>()
                        >= 660
                );
            }
        }
        let p = (15 * image.width + 34) * 3;
        image.data[p..p + 3].fill(0);
        assert!(!supports("KLM)", &image.data, image.width, image.height));
    }

    #[test]
    fn two_splittable_components_are_ambiguous_even_if_either_fits_a_single_cell() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let source = crate::io::load_rgb_u8(path).unwrap();
        for (second_thick, expected) in [(true, true), (false, false)] {
            let mut rgb = vec![255; source.data.len()];
            for start in [9, 29] {
                for y in 9..26 {
                    for x in start..start + 19 {
                        if x != start + 9 || y == 25 || (start == 29 && second_thick && y == 24) {
                            rgb[(y * source.width + x) * 3..(y * source.width + x + 1) * 3].fill(0);
                        }
                    }
                }
            }
            for y in 8..31 {
                for dx in 0..5 {
                    let from = (y * source.width + 62 + dx) * 3;
                    let to = (y * source.width + 49 + dx) * 3;
                    rgb[to..to + 3].copy_from_slice(&source.data[from..from + 3]);
                }
            }
            assert_eq!(
                supports("KLM)", &rgb, source.width, source.height),
                expected,
                "second thick {second_thick}"
            );
        }
    }

    #[test]
    fn the_original_foreground_sum_boundary_keeps_the_neck_owned() {
        let path =
            std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
        let image = crate::io::load_rgb_u8(path).unwrap();
        let mut rgb = image.data.clone();
        for pixel in rgb.chunks_mut(3) {
            if pixel.iter().map(|&v| u16::from(v)).sum::<u16>() < 660 {
                pixel.copy_from_slice(&[219, 220, 220]);
            } else {
                pixel.fill(255);
            }
        }
        assert!(supports("KLM)", &rgb, image.width, image.height));
        let p = (25 * image.width + 39) * 3;
        assert_eq!(&rgb[p..p + 3], &[219, 220, 220]);
        rgb[p + 2] = 221;
        assert!(!supports("KLM)", &rgb, image.width, image.height));
        assert!(!supports("KLM)", &[], image.width, image.height));
        assert!(!supports("KLM)", &[0; 3], usize::MAX, usize::MAX));
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn translated_paper_and_gray_preserve_the_owned_fragment_support(
            mx in 0usize..11, my in 0usize..11, tone in 0u8..220,
        ) {
            let path = std::path::Path::new("tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png");
            let source = crate::io::load_rgb_u8(path).unwrap();
            let (width, height) = (source.width + 2 * mx, source.height + 2 * my);
            let mut rgb = vec![255; width * height * 3];
            for y in 0..source.height { for x in 0..source.width {
                let p = (y * source.width + x) * 3;
                if source.data[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660 {
                    let to = ((y + my) * width + x + mx) * 3;
                    rgb[to..to + 3].fill(tone);
                }
            }}
            let before = rgb.clone();
            proptest::prop_assert!(supports("KLM)", &rgb, width, height));
            proptest::prop_assert_eq!(rgb, before);
        }
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_LATIN_PAIR_POLICY_ROOT with frozen baseline pairs"]
    fn record_latin_pair_policy() {
        let Ok(root) = std::env::var("NDLOCR_LATIN_PAIR_POLICY_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        let output = root.join("replay");
        std::fs::create_dir(&output).unwrap();
        let expected_cases = m["expected_cases"].as_u64().unwrap_or(162915) as usize;
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
                && super::supports(&fragment, &rgb, w, h);
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
    fn verify_fresh_latin_pair_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_LATIN_PAIR_POLICY_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 19440);
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
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":19440,"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_LATIN_PAIR_PRODUCT_MANIFEST with frozen candidate strings"]
    fn verify_product_matches_frozen_latin_pair_policy() {
        let Ok(path) = std::env::var("NDLOCR_LATIN_PAIR_PRODUCT_MANIFEST") else {
            return;
        };
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = m["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 223143);
        for x in cases {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
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
            assert_eq!(rgb, before);
        }
        println!(
            "{} frozen raw pairs match the actual product; no NN",
            cases.len()
        );
    }
}
