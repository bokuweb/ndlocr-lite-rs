//! A support-only view for one body glyph with a large edge-aligned detached part.
use super::*;

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    let Some((view, h)) = component_view(text, rgb, width, height) else {
        return false;
    };
    supports_collected(&view, h, 3, Some(text))
}

fn component_view(
    text: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<([Component; 3], usize)> {
    use unicode_width::UnicodeWidthChar;
    let chars: Vec<_> = text.chars().take(4).collect();
    let closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if chars.len() != 3
        || chars.iter().any(|c| c.is_whitespace())
        || chars.iter().filter(|&&c| closer(c)).count() != 1
        || chars
            .iter()
            .filter(|&&c| !closer(c))
            .any(|c| c.width() != Some(2))
    {
        return None;
    }
    let leading = closer(chars[0]);
    if !leading && !closer(chars[2]) {
        return None;
    }
    let (mut body, h) = collect_components(rgb, width, height)?;
    if !(4..=7).contains(&body.len()) {
        return None;
    }
    let mark = if leading { body.remove(0) } else { body.pop()? };
    let x0 = body.iter().map(|c| c.x0).min()?;
    let x1 = body.iter().map(|c| c.x1).max()?;
    let y0 = body.iter().map(|c| c.y0).min()?;
    let y1 = body.iter().map(|c| c.y1).max()?;
    let bh = y1 - y0;
    let max_gap = (bh / 12).max(1);
    let gap = if leading {
        x0.checked_sub(mark.x1)?
    } else {
        mark.x0.checked_sub(x1)?
    };
    let mw = mark.x1 - mark.x0;
    let mh = mark.y1 - mark.y0;
    if x1 - x0 < 3
        || bh == 0
        || gap > max_gap
        || mw < bh.div_ceil(4)
        || mw as u128 * 2 > bh as u128
        || mh as u128 * 2 < bh as u128
        || mh as u128 * 2 > bh as u128 * 3
        || mark.area == mw * mh
    {
        return None;
    }
    let foreground = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < 660
    };
    let owned = body
        .iter()
        .try_fold(0usize, |sum, c| sum.checked_add(c.area))?;
    if (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| foreground(x, y))
        .count()
        != owned
    {
        return None;
    }
    let cut = (x0 + 1..x1 - 1)
        .filter(|&x| (y0..y1).all(|y| !foreground(x, y)))
        .min_by_key(|&x| ((2 * x as u128 + 1).abs_diff(x0 as u128 + x1 as u128), x))?;
    let mut left = Vec::new();
    let mut right = Vec::new();
    for c in &body {
        if c.x1 <= cut {
            left.push(c);
        } else if c.x0 > cut {
            right.push(c);
        } else {
            return None;
        }
    }
    let l = group_view(&left, bh, max_gap)?;
    let r = group_view(&right, bh, max_gap)?;
    let inner_gap = r.x0.checked_sub(l.x1)?;
    if inner_gap == 0 || inner_gap > max_gap {
        return None;
    }
    let has_edge_part = |parts: &[&Component]| {
        primary_part(parts).is_some_and(|p| parts.iter().any(|c| edge_shared_large_child(p, c, bh)))
    };
    if !has_edge_part(&left) && !has_edge_part(&right) {
        return None;
    }
    Some((if leading { [mark, l, r] } else { [l, r, mark] }, h))
}

fn group_view(parts: &[&Component], height: usize, max_gap: usize) -> Option<Component> {
    let primary = primary_part(parts)?;
    if height == 0 || (primary.y1 - primary.y0) as u128 * 4 < height as u128 * 3 {
        return None;
    }
    let mut view = Component {
        x0: primary.x0,
        y0: primary.y0,
        x1: primary.x1,
        y1: primary.y1,
        area: 0,
    };
    for c in parts {
        let gap =
            c.x0.saturating_sub(primary.x1)
                .max(primary.x0.saturating_sub(c.x1));
        if c.y0 < primary.y0 || c.y1 > primary.y1 || gap > max_gap {
            return None;
        }
        view.x0 = view.x0.min(c.x0);
        view.x1 = view.x1.max(c.x1);
        view.y0 = view.y0.min(c.y0);
        view.y1 = view.y1.max(c.y1);
        view.area = view.area.checked_add(c.area)?;
    }
    let width = view.x1 - view.x0;
    if width as u128 * 4 < height as u128 * 3 || width as u128 * 2 > height as u128 * 3 {
        return None;
    }
    Some(view)
}

fn primary_part<'a>(parts: &[&'a Component]) -> Option<&'a Component> {
    parts
        .iter()
        .copied()
        .max_by_key(|c| (c.y1 - c.y0, c.area, std::cmp::Reverse(c.x0)))
}

fn edge_shared_large_child(parent: &Component, child: &Component, height: usize) -> bool {
    if height == 0
        || child.x0 >= child.x1
        || child.y0 >= child.y1
        || child.x0 < parent.x0
        || child.y0 < parent.y0
        || child.x1 > parent.x1
        || child.y1 > parent.y1
        || child.x1 - child.x0 < height.div_ceil(2)
        || child.y1 - child.y0 < height.div_ceil(2)
    {
        return false;
    }
    [
        child.x0 == parent.x0,
        child.y0 == parent.y0,
        child.x1 == parent.x1,
        child.y1 == parent.y1,
    ]
    .into_iter()
    .filter(|&equal| equal)
    .count()
        == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_DETACHED_PRODUCT_MANIFEST with frozen candidate strings"]
    fn verify_product_matches_frozen_detached_policy() {
        let Ok(path) = std::env::var("NDLOCR_DETACHED_PRODUCT_MANIFEST") else {
            return;
        };
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = m["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 99591);
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

    #[test]
    #[ignore = "diagnostic; validates every fresh original/effective RGB tensor before NN"]
    fn verify_fresh_detached_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_DETACHED_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 10944);
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
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":10944,"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_DETACHED_ROOT with fixed baseline raw pairs"]
    fn record_detached_body_policy() {
        let Ok(root) = std::env::var("NDLOCR_DETACHED_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        let output = root.join("replay");
        std::fs::create_dir(&output).expect("new replay directory");
        let mut records = Vec::new();
        for x in m["cases"].as_array().unwrap() {
            let raw = x["raw"].as_str().unwrap();
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
            let baseline = sanitize_visible_single_line(raw, &rgb, w, h, orientation);
            assert_eq!(baseline, x["expected"].as_str().unwrap(), "{id}");
            let fragment = sanitize_visible_repetitions(raw, &rgb, w, h, orientation);
            let selected = fragment.chars().count() > baseline.chars().count()
                && !fragment.chars().any(char::is_whitespace)
                && (orientation == ParseqOrientation::Horizontal || h as u128 * 5 <= w as u128 * 4)
                && supports(&fragment, &rgb, w, h);
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
        println!(
            "{} fixed raw pairs passed through product baseline and test-only detached-body view;no NN forwards",
            records.len()
        );
    }

    fn c(x0: usize, y0: usize, x1: usize, y1: usize) -> Component {
        Component {
            x0,
            y0,
            x1,
            y1,
            area: 0,
        }
    }

    #[test]
    fn exactly_one_shared_edge_and_half_height_parts_are_required() {
        let parent = c(0, 0, 20, 20);
        for (child, expected) in [
            (c(5, 10, 15, 20), true),
            (c(5, 0, 15, 10), true),
            (c(0, 5, 10, 15), true),
            (c(10, 5, 20, 15), true),
            (c(5, 9, 15, 19), false),
            (c(0, 10, 15, 20), false),
            (c(0, 0, 20, 20), false),
            (c(5, 10, 14, 20), false),
            (c(5, 11, 15, 20), false),
            (c(5, 10, 15, 21), false),
        ] {
            assert_eq!(edge_shared_large_child(&parent, &child, 20), expected);
        }
    }

    #[test]
    fn detached_parts_of_the_real_glyph_support_the_model_closer() {
        let image = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/detached_body_components/info_round40.png",
        ))
        .unwrap();
        let before = image.data.clone();
        assert_eq!(
            sanitize_visible_single_line(
                "情報)",
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto
            ),
            "情報)"
        );
        assert!(supports("情報)", &image.data, image.width, image.height));
        assert_eq!(image.data, before);
    }

    #[test]
    fn unrelated_text_invalid_rgb_and_unowned_inner_noise_do_not_support() {
        let image = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/detached_body_components/info_round40.png",
        ))
        .unwrap();
        let (rgb, w, h) = (&image.data, image.width, image.height);
        for text in [
            "情報", "情報))", ")情報)", "情)報", "AB)", "情報 )", " 情報)", "情)",
        ] {
            assert!(!supports(text, rgb, w, h), "{text}");
        }
        assert!(!supports("情報)", &[], w, h));
        assert!(!supports("情報)", rgb, usize::MAX, h));
        assert!(!supports("情報)", &vec![255; w * h * 3], w, h));
        let mut noisy = rgb.clone();
        let is_white = |x: usize, y: usize| {
            rgb[(y * w + x) * 3..(y * w + x + 1) * 3]
                .iter()
                .map(|&v| u16::from(v))
                .sum::<u16>()
                >= 660
        };
        let (x, y) = (10usize..46)
            .flat_map(|y| (10usize..86).map(move |x| (x, y)))
            .find(|&(x, y)| (y - 1..=y + 1).all(|yy| (x - 1..=x + 1).all(|xx| is_white(xx, yy))))
            .expect("an isolated white spot inside the body bbox");
        noisy[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
        assert!(!supports("情報)", &noisy, w, h));
    }

    #[test]
    fn a_filled_mark_and_one_pixel_beyond_the_fixed_outer_gap_are_rejected() {
        let image = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/detached_body_components/info_round40.png",
        ))
        .unwrap();
        let (w, h) = (image.width, image.height);
        let mut filled = image.data.clone();
        for y in 12..50 {
            filled[(y * w + 90) * 3..(y * w + 100) * 3].fill(0);
        }
        assert!(!supports("情報)", &filled, w, h));
        let nw = w + 1;
        let mut moved = vec![255; nw * h * 3];
        for y in 0..h {
            moved[y * nw * 3..(y * nw + w) * 3]
                .copy_from_slice(&image.data[y * w * 3..(y + 1) * w * 3]);
        }
        for y in 12..50 {
            moved[(y * nw + 90) * 3..(y * nw + 101) * 3].fill(255);
            moved[(y * nw + 91) * 3..(y * nw + 101) * 3]
                .copy_from_slice(&image.data[(y * w + 90) * 3..(y * w + 100) * 3]);
        }
        assert!(!supports("情報)", &moved, nw, h));
    }

    #[test]
    fn the_unchanged_closer_also_supports_a_compact_prefix() {
        let image = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/detached_body_components/info_round40.png",
        ))
        .unwrap();
        let (w, h) = (image.width, image.height);
        let nw = w + 13;
        let mut moved = vec![255; nw * h * 3];
        for y in 9..47 {
            moved[(y * nw + 22) * 3..(y * nw + 100) * 3]
                .copy_from_slice(&image.data[(y * w + 9) * 3..(y * w + 87) * 3]);
        }
        for y in 12..50 {
            moved[(y * nw + 9) * 3..(y * nw + 19) * 3]
                .copy_from_slice(&image.data[(y * w + 90) * 3..(y * w + 100) * 3]);
        }
        let before = moved.clone();
        assert!(supports(")情報", &moved, nw, h));
        assert_eq!(moved, before);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn binary_support_survives_paper_translation_gray_and_integer_block_scale(
            mx in 0usize..9, my in 0usize..9, tone in 0u8..220, scale in 1usize..4,
        ) {
            let image = crate::io::load_rgb_u8(std::path::Path::new("tests/fixtures/detached_body_components/info_round40.png")).unwrap();
            let (w,h)=(image.width*scale+2*mx,image.height*scale+2*my);let mut rgb=vec![255;w*h*3];
            for y in 0..image.height {for x in 0..image.width {
                if image.data[(y*image.width+x)*3..(y*image.width+x+1)*3].iter().map(|&v|u16::from(v)).sum::<u16>()>=660 {continue;}
                for yy in y*scale+my..(y+1)*scale+my {for xx in x*scale+mx..(x+1)*scale+mx {rgb[(yy*w+xx)*3..(yy*w+xx+1)*3].fill(tone);}}
            }}
            let before=rgb.clone();proptest::prop_assert!(supports("情報)",&rgb,w,h));proptest::prop_assert_eq!(rgb,before);
        }
    }

    #[test]
    fn grouped_parts_keep_their_original_bounds_and_area_and_reject_distant_strokes() {
        let mut primary = c(4, 0, 24, 20);
        primary.area = 112;
        let mut child = c(9, 10, 20, 20);
        child.area = 110;
        let mut side = c(1, 7, 3, 13);
        side.area = 12;
        let view =
            group_view(&[&primary, &child, &side], 20, 1).expect("one physical full-width glyph");
        assert_eq!(
            [view.x0, view.y0, view.x1, view.y1, view.area],
            [1, 0, 24, 20, 234]
        );
        assert!(group_view(&[&primary, &child, &c(0, 7, 2, 13)], 20, 1).is_none());
        assert!(group_view(&[&primary, &child, &c(1, 19, 3, 23)], 20, 1).is_none());
        for (part, expected) in [
            (c(0, 0, 15, 20), true),
            (c(0, 0, 14, 20), false),
            (c(0, 0, 30, 20), true),
            (c(0, 0, 31, 20), false),
            (c(0, 0, 20, 15), true),
            (c(0, 0, 20, 14), false),
        ] {
            assert_eq!(group_view(&[&part], 20, 1).is_some(), expected);
        }
        assert!(group_view(&[], 20, 1).is_none());
    }
}
