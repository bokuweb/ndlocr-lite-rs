//! Diagnostic support for a raster bridge together with large detached body parts.

fn primary_support_mask(
    mask: &[bool],
    width: usize,
    height: usize,
    expected_area: usize,
) -> Option<Vec<bool>> {
    if width == 0
        || height == 0
        || width.checked_mul(height) != Some(mask.len())
        || expected_area == 0
    {
        return None;
    }
    let mut remaining = mask.to_vec();
    let mut found = None;
    loop {
        let before = remaining.clone();
        let Some(c) =
            super::single_pixel_bridge::take_region(&mut remaining, width, height, 0, width)
        else {
            break;
        };
        if c.x0 == 0 && c.y0 == 0 && c.x1 == width && c.y1 == height && c.area == expected_area {
            if found.is_some() {
                return None;
            }
            found = Some(
                before
                    .iter()
                    .zip(&remaining)
                    .map(|(&was, &now)| was && !now)
                    .collect(),
            );
        }
    }
    found
}

fn body_views(
    mask: &[bool],
    width: usize,
    height: usize,
    primary_area: usize,
    min_area: usize,
) -> Option<[super::Component; 2]> {
    use super::detached_body_components::{edge_shared_large_child, group_view, primary_part};
    use super::single_pixel_bridge::{split_body, take_region};
    let primary = primary_support_mask(mask, width, height, primary_area)?;
    let [left_primary, _] = split_body(&primary, width, height)?;
    let cut = left_primary.x1;
    let max_gap = (height / 12).max(1);
    if (2 * cut as u128 + 1).abs_diff(width as u128) > 2 * max_gap as u128
        || (0..height).filter(|&y| mask[y * width + cut]).count() != 1
    {
        return None;
    }
    let y = (0..height).find(|&y| primary[y * width + cut])?;
    let mut remaining = mask.to_vec();
    remaining[y * width + cut] = false;
    let mut left = Vec::new();
    let mut right = Vec::new();
    while let Some(c) = take_region(&mut remaining, width, height, 0, cut) {
        left.push(c);
    }
    while let Some(c) = take_region(&mut remaining, width, height, cut + 1, width) {
        right.push(c);
    }
    if remaining.iter().any(|&v| v)
        || left.iter().chain(&right).any(|c| c.area < min_area)
        || left
            .iter()
            .chain(&right)
            .map(|c| c.area)
            .sum::<usize>()
            .checked_add(1)
            != Some(mask.iter().filter(|&&v| v).count())
    {
        return None;
    }
    let lrefs: Vec<_> = left.iter().collect();
    let rrefs: Vec<_> = right.iter().collect();
    let l = group_view(&lrefs, height, max_gap)?;
    let r = group_view(&rrefs, height, max_gap)?;
    let has_edge_part = |parts: &[&super::Component]| {
        primary_part(parts)
            .is_some_and(|p| parts.iter().any(|c| edge_shared_large_child(p, c, height)))
    };
    if !has_edge_part(&lrefs) && !has_edge_part(&rrefs) {
        return None;
    }
    Some([l, r])
}

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    let Some((view, h)) = component_view(text, rgb, width, height) else {
        return false;
    };
    super::supports_collected(&view, h, 3, Some(text))
}

fn component_view(
    text: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<([super::Component; 3], usize)> {
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
    let (mut body, h) = super::collect_components(rgb, width, height)?;
    if !(3..=6).contains(&body.len()) {
        return None;
    }
    let mark = if leading { body.remove(0) } else { body.pop()? };
    let x0 = body.iter().map(|c| c.x0).min()?;
    let x1 = body.iter().map(|c| c.x1).max()?;
    let y0 = body.iter().map(|c| c.y0).min()?;
    let y1 = body.iter().map(|c| c.y1).max()?;
    let bh = y1 - y0;
    let bw = x1 - x0;
    let mut matching = body
        .iter()
        .filter(|c| c.x0 == x0 && c.x1 == x1 && c.y0 == y0 && c.y1 == y1);
    let primary_area = matching.next()?.area;
    if matching.next().is_some() {
        return None;
    }
    let gap = if leading {
        x0.checked_sub(mark.x1)?
    } else {
        mark.x0.checked_sub(x1)?
    };
    let mw = mark.x1 - mark.x0;
    let mh = mark.y1 - mark.y0;
    if bh == 0
        || bw < 3
        || bw as u128 > bh as u128 * 3 + 1
        || gap > 1
        || mw < bh.div_ceil(4)
        || mw as u128 * 2 > bh as u128
        || mh as u128 * 2 < bh as u128
        || mh as u128 * 2 > bh as u128 * 3
        || mark.area == mw * mh
    {
        return None;
    }
    let mask: Vec<_> = (y0..y1)
        .flat_map(|y| {
            (x0..x1).map(move |x| {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
                    .iter()
                    .map(|&v| u16::from(v))
                    .sum::<u16>()
                    < 660
            })
        })
        .collect();
    let owned = body
        .iter()
        .try_fold(0usize, |area, c| area.checked_add(c.area))?;
    if mask.iter().filter(|&&v| v).count() != owned {
        return None;
    }
    let mut views = body_views(&mask, bw, bh, primary_area, h.div_ceil(2).max(3))?;
    for c in &mut views {
        c.x0 += x0;
        c.x1 += x0;
        c.y0 += y0;
        c.y1 += y0;
    }
    let [left, right] = views;
    Some((
        if leading {
            [mark, left, right]
        } else {
            [left, right, mark]
        },
        h,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{
        ParseqOrientation, sanitize_visible_repetitions, sanitize_visible_single_line,
    };
    use super::*;

    fn body() -> (Vec<bool>, Vec<bool>, usize, usize) {
        let (w, h) = (25, 12);
        let mut primary = vec![false; w * h];
        let mut fill = |x0: usize, y0: usize, x1: usize, y1: usize| {
            for y in y0..y1 {
                for x in x0..x1 {
                    primary[y * w + x] = true;
                }
            }
        };
        fill(0, 0, 12, 1);
        fill(0, 0, 1, 12);
        fill(11, 0, 12, 3);
        fill(13, 0, 25, 1);
        fill(13, 11, 25, 12);
        fill(13, 0, 14, 12);
        fill(24, 0, 25, 12);
        primary[w + 12] = true;
        let mut all = primary.clone();
        // This child shares only its bottom with the left glyph's outer part.
        for y in 5..12 {
            for x in 3..9 {
                all[y * w + x] = true;
            }
        }
        (all, primary, w, h)
    }

    #[test]
    fn recovers_only_the_original_connected_primary_without_filling_its_child() {
        let (all, expected, w, h) = body();
        let before = all.clone();
        assert_eq!(expected.iter().filter(|&&v| v).count(), 70);
        assert_eq!(all.iter().filter(|&&v| v).count(), 112);
        assert_eq!(primary_support_mask(&all, w, h, 70), Some(expected));
        assert_eq!(all, before);
    }
    #[test]
    fn a_child_area_an_incorrect_bbox_and_invalid_shapes_do_not_name_the_primary() {
        let (mut all, _, w, h) = body();
        for area in [0, 42, 69, 71, 112] {
            assert!(
                primary_support_mask(&all, w, h, area).is_none(),
                "area {area}"
            );
        }
        all[w + 12] = false;
        assert!(
            primary_support_mask(&all, w, h, 69).is_none(),
            "the two body glyphs are already disconnected"
        );
        assert!(primary_support_mask(&[], 0, 0, 70).is_none());
        assert!(primary_support_mask(&all, usize::MAX, h, 70).is_none());
        assert!(primary_support_mask(&all, w, h + 1, 70).is_none());
    }
    #[test]
    fn the_same_support_pixel_splits_the_primary_and_keeps_all_detached_parts_owned() {
        let (all, _, w, h) = body();
        let before = all.clone();
        let [left, right] = body_views(&all, w, h, 70, 6)
            .expect("the bridge and detached child together support two glyphs");
        assert_eq!(
            [left.x0, left.y0, left.x1, left.y1, left.area],
            [0, 0, 12, 12, 67]
        );
        assert_eq!(
            [right.x0, right.y0, right.x1, right.y1, right.area],
            [13, 0, 25, 12, 44]
        );
        assert_eq!(left.area + right.area, 111);
        assert_eq!(all, before);
    }

    #[test]
    fn two_connections_do_not_move_the_cut_into_a_thin_stroke_of_the_first_glyph() {
        let (mut all, _, w, h) = body();
        all[2 * w + 12] = true;
        assert!(body_views(&all, w, h, 71, 6).is_none());
    }

    #[test]
    fn the_center_distance_limit_is_inclusive() {
        for (left_width, expected) in [(13, true), (14, false)] {
            let (w, h) = (25, 12);
            let start = left_width + 1;
            let mut mask = vec![false; w * h];
            let mut fill = |x0: usize, y0: usize, x1: usize, y1: usize| {
                for y in y0..y1 {
                    for x in x0..x1 {
                        mask[y * w + x] = true;
                    }
                }
            };
            fill(0, 0, left_width, 2);
            fill(0, 0, 1, h);
            fill(left_width - 1, 0, left_width, 4);
            fill(start, 0, w, 1);
            fill(start, h - 1, w, h);
            fill(start, 0, start + 1, h);
            fill(w - 1, 0, w, h);
            mask[w + left_width] = true;
            assert_eq!(mask.iter().filter(|&&v| v).count(), 81);
            for y in 5..12 {
                for x in 3..9 {
                    mask[y * w + x] = true;
                }
            }
            assert_eq!(
                body_views(&mask, w, h, 81, 6).is_some(),
                expected,
                "cut at {left_width}"
            );
        }
    }
    #[test]
    fn the_real_body_bridge_and_bottom_shared_part_support_the_original_closer() {
        let im = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/owned_parts_body_bridge/info_corner_mincho20.png",
        ))
        .unwrap();
        let before = im.data.clone();
        assert_eq!(
            super::super::sanitize_visible_single_line(
                "情報」",
                &im.data,
                im.width,
                im.height,
                super::super::ParseqOrientation::Auto
            ),
            "情報」"
        );
        assert!(!super::super::single_pixel_bridge::supports(
            "情報」",
            &im.data,
            im.width,
            im.height
        ));
        assert!(!super::super::detached_body_components::supports(
            "情報」",
            &im.data,
            im.width,
            im.height
        ));
        assert!(supports("情報」", &im.data, im.width, im.height));
        assert_eq!(im.data, before);
    }
    fn picture(leading: bool) -> (Vec<u8>, usize, usize) {
        let (mask, _, bw, bh) = body();
        let (w, h) = (34, 16);
        let bx = if leading { 6 } else { 1 };
        let mx = if leading { 1 } else { 27 };
        let mut rgb = vec![255; w * h * 3];
        for y in 0..bh {
            for x in 0..bw {
                if mask[y * bw + x] {
                    rgb[((y + 1) * w + x + bx) * 3..((y + 1) * w + x + bx + 1) * 3].fill(0);
                }
            }
        }
        for y in 1..13 {
            for x in mx..mx + 4 {
                if y == 1 || y == 12 || x == mx + 3 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
        (rgb, w, h)
    }

    #[test]
    fn both_edge_positions_keep_the_original_marker_and_text_shape_restrictions() {
        for leading in [false, true] {
            let (rgb, w, h) = picture(leading);
            let text = if leading { "」情報" } else { "情報」" };
            assert!(supports(text, &rgb, w, h));
            for invalid in [
                "情報",
                "情」報",
                "AB)",
                "情報 )",
                " 情報」",
                "情報」」",
                ")情報」",
                "情」",
            ] {
                assert!(!supports(invalid, &rgb, w, h), "{invalid}");
            }
            assert!(!supports(text, &[], w, h));
            assert!(!supports(text, &rgb, usize::MAX, h));
        }
    }

    #[test]
    fn unowned_inner_noise_filled_marks_and_wide_outer_gaps_are_rejected() {
        let (rgb, w, h) = picture(false);
        let mut noisy = rgb.clone();
        noisy[(4 * w + 7) * 3..(4 * w + 8) * 3].fill(0);
        assert!(!supports("情報」", &noisy, w, h));
        let mut filled = rgb.clone();
        for y in 1..13 {
            filled[(y * w + 27) * 3..(y * w + 31) * 3].fill(0);
        }
        assert!(!supports("情報」", &filled, w, h));
        let nw = w + 1;
        let mut moved = vec![255; nw * h * 3];
        for y in 0..h {
            moved[y * nw * 3..(y * nw + 27) * 3].copy_from_slice(&rgb[y * w * 3..(y * w + 27) * 3]);
            moved[(y * nw + 28) * 3..(y * nw + w + 1) * 3]
                .copy_from_slice(&rgb[(y * w + 27) * 3..(y + 1) * w * 3]);
        }
        assert!(!supports("情報」", &moved, nw, h));
    }

    #[test]
    fn the_narrow_round_marker_still_fails_the_unchanged_quarter_height_width_bound() {
        let im = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/owned_parts_body_bridge/narrow_round_prefix_mincho20.png",
        ))
        .unwrap();
        assert!(!supports(")情報", &im.data, im.width, im.height));
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn threshold_support_survives_paper_translation_and_gray(
            leading in proptest::bool::ANY, mx in 0usize..9, my in 0usize..9, tone in 0u8..220,
        ) {
            let(original,ow,oh)=picture(leading);let(w,h)=(ow+2*mx,oh+2*my);let mut rgb=vec![255;w*h*3];
            for y in 0..oh {for x in 0..ow {if original[(y*ow+x)*3]==0 {rgb[((y+my)*w+x+mx)*3..((y+my)*w+x+mx+1)*3].fill(tone);}}}
            let text=if leading {"」情報"}else{"情報」"};let before=rgb.clone();proptest::prop_assert!(supports(text,&rgb,w,h));proptest::prop_assert_eq!(rgb,before);
        }
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_OWNED_BRIDGE_ROOT with frozen baseline pairs"]
    fn record_owned_parts_bridge_policy() {
        let Ok(root) = std::env::var("NDLOCR_OWNED_BRIDGE_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        let output = root.join("replay");
        std::fs::create_dir(&output).unwrap();
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
        println!("{} fixed raw pairs evaluated;no NN forwards", records.len());
    }
    #[test]
    #[ignore = "diagnostic; validates every fresh original/effective RGB tensor before NN"]
    fn verify_fresh_owned_bridge_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_OWNED_BRIDGE_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 7632);
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
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":7632,"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_OWNED_BRIDGE_PRODUCT_MANIFEST with frozen candidate strings"]
    fn verify_product_matches_frozen_owned_bridge_policy() {
        let Ok(path) = std::env::var("NDLOCR_OWNED_BRIDGE_PRODUCT_MANIFEST") else {
            return;
        };
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = m["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 137991);
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
