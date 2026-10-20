//! Ownership of detached upper dots in short Latin bodies.

use super::Component;
mod policy;

fn owner(dot: &Component, stems: &[Component], body_height: usize) -> Option<usize> {
    let valid = |c: &Component| {
        c.x0 < c.x1
            && c.y0 < c.y1
            && c.area >= 3
            && c.area as u128 <= (c.x1 - c.x0) as u128 * (c.y1 - c.y0) as u128
    };
    if body_height < 8 || !valid(dot) || stems.iter().any(|s| !valid(s)) {
        return None;
    }
    let (dw, dh) = (dot.x1 - dot.x0, dot.y1 - dot.y0);
    if [dw, dh]
        .iter()
        .any(|&n| n < body_height.div_ceil(8) || n > body_height / 3)
    {
        return None;
    }
    let mut result = None;
    for (index, stem) in stems.iter().enumerate() {
        let (sw, sh) = (stem.x1 - stem.x0, stem.y1 - stem.y0);
        let Some(gap) = stem.y0.checked_sub(dot.y1) else {
            continue;
        };
        let overlap = stem.x1.min(dot.x1).saturating_sub(stem.x0.max(dot.x0));
        let center = dot.x0 as u128 + dot.x1 as u128;
        if sh as u128 * 4 < body_height as u128 * 3
            || sh > body_height
            || sw < body_height.div_ceil(8)
            || sw as u128 * 2 > body_height as u128 * 3
            || gap == 0
            || gap > body_height.div_ceil(4)
            || overlap as u128 * 2 < dw as u128
            || center < stem.x0 as u128 * 2
            || center > stem.x1 as u128 * 2
            || dot.area as u128 * 2 > stem.area as u128
            || (stem.y1 - dot.y0) as u128 * 2 > body_height as u128 * 3
        {
            continue;
        }
        if result.is_some() {
            return None;
        }
        result = Some(index);
    }
    result
}

fn collect_all(rgb: &[u8], width: usize, height: usize) -> Option<Vec<Component>> {
    let pixels = width.checked_mul(height)?;
    if width == 0 || height == 0 || pixels > 65_536 || pixels.checked_mul(3)? != rgb.len() {
        return None;
    }
    let mut mask: Vec<_> = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| p.iter().map(|&v| u16::from(v)).sum::<u16>() < 660)
        .collect();
    let mut parts = Vec::new();
    while let Some(c) = super::single_pixel_bridge::take_region(&mut mask, width, height, 0, width)
    {
        parts.push(c);
    }
    parts.sort_by_key(|c| (c.x0, c.y0, c.x1, c.y1));
    Some(parts)
}

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    policy::supports(text, rgb, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    #[ignore = "diagnostic; requires frozen dotted Latin product manifest"]
    fn product_matches_all_frozen_outputs() {
        let Ok(path) = std::env::var("NDLOCR_DOTTED_PRODUCT_MANIFEST") else {
            return;
        };
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = manifest["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 282465);
        for x in cases {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let orientation = match x["orientation"].as_str() {
                Some("horizontal") => super::super::ParseqOrientation::Horizontal,
                None | Some("auto") => super::super::ParseqOrientation::Auto,
                other => panic!("invalid orientation {other:?}"),
            };
            let actual = super::super::sanitize_visible_single_line(
                x["raw"].as_str().unwrap(),
                &rgb,
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
                orientation,
            );
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
        println!("{} frozen raw pairs match product; no NN", cases.len());
    }

    fn cell(x0: usize, y0: usize, x1: usize, y1: usize, area: usize) -> Component {
        Component {
            x0,
            y0,
            x1,
            y1,
            area,
        }
    }

    fn dot() -> Component {
        cell(14, 2, 18, 6, 12)
    }

    fn stem() -> Component {
        cell(12, 10, 22, 34, 120)
    }

    #[test]
    fn one_upper_dot_has_one_owner_without_changing_the_boxes() {
        let d = dot();
        let stems = [cell(0, 10, 9, 34, 100), stem()];
        assert_eq!(owner(&d, &stems, 24), Some(1));
        assert_eq!((d.x0, d.y0, d.x1, d.y1, d.area), (14, 2, 18, 6, 12));
        assert_eq!(stems[1].area, 120);
    }

    #[test]
    fn ambiguous_or_missing_owners_are_rejected() {
        assert_eq!(owner(&dot(), &[], 24), None);
        assert_eq!(owner(&dot(), &[stem(), stem()], 24), None);
        assert_eq!(owner(&dot(), &[cell(0, 10, 9, 34, 100)], 24), None);
    }

    #[test]
    fn dot_width_and_height_have_inclusive_eighth_and_third_bounds() {
        for size in [2, 3, 8, 9] {
            let expected = if (3..=8).contains(&size) {
                Some(0)
            } else {
                None
            };
            let wide_stem = cell(10, 10, 24, 34, 200);
            assert_eq!(
                owner(&cell(12, 1, 12 + size, 4, size * 3), &[wide_stem], 24),
                expected,
                "dot width {size}"
            );
            let lowered_stem = cell(12, 12, 22, 36, 180);
            assert_eq!(
                owner(&cell(14, 10 - size, 18, 10, 3 * size), &[lowered_stem], 24),
                expected,
                "dot height {size}"
            );
        }
        assert_eq!(owner(&dot(), &[stem()], 7), None);
    }

    #[test]
    fn stem_width_and_height_keep_the_fixed_bounds() {
        for width in [2, 3, 36, 37] {
            let d = cell(13, 4, 16, 8, 10);
            let s = cell(12, 10, 12 + width, 34, width * 12);
            assert_eq!(
                owner(&d, &[s], 24),
                if (3..=36).contains(&width) {
                    Some(0)
                } else {
                    None
                },
                "stem width {width}"
            );
        }
        for height in [17, 18, 24, 25] {
            let s = cell(12, 10, 22, 10 + height, 100);
            assert_eq!(
                owner(&dot(), &[s], 24),
                if (18..=24).contains(&height) {
                    Some(0)
                } else {
                    None
                },
                "stem height {height}"
            );
        }
    }

    #[test]
    fn dot_must_be_above_with_a_positive_bounded_gap() {
        for gap in [0, 1, 6, 7] {
            let d = cell(14, 8 - gap, 18, 10 - gap, 6);
            assert_eq!(
                owner(&d, &[stem()], 24),
                None,
                "two-pixel dot is always too short, gap {gap}"
            );
            let d = cell(14, 7 - gap, 18, 10 - gap, 9);
            assert_eq!(
                owner(&d, &[stem()], 24),
                if (1..=6).contains(&gap) {
                    Some(0)
                } else {
                    None
                },
                "vertical gap {gap}"
            );
        }
        assert_eq!(owner(&cell(14, 12, 18, 16, 12), &[stem()], 24), None);
    }

    #[test]
    fn horizontal_overlap_and_center_both_have_to_support_the_owner() {
        for (x, expected) in [
            (8, None),
            (9, None),
            (10, Some(0)),
            (20, Some(0)),
            (21, None),
            (22, None),
        ] {
            assert_eq!(
                owner(&cell(x, 2, x + 4, 6, 12), &[stem()], 24),
                expected,
                "dot x {x}"
            );
        }
    }

    #[test]
    fn dot_area_and_total_height_do_not_grow_unbounded() {
        assert_eq!(
            owner(&cell(14, 2, 18, 6, 12), &[cell(12, 10, 22, 34, 23)], 24),
            None
        );
        assert_eq!(
            owner(&cell(14, 2, 18, 6, 12), &[cell(12, 10, 22, 34, 24)], 24),
            Some(0)
        );
        for (top, expected) in [(1, Some(0)), (0, Some(0))] {
            assert_eq!(
                owner(&cell(14, top, 18, 4, 12), &[cell(12, 10, 22, 34, 120)], 24),
                expected
            );
        }
        assert_eq!(
            owner(&cell(14, 0, 18, 8, 24), &[cell(12, 14, 22, 38, 120)], 24),
            None
        );
        assert_eq!(
            owner(&cell(14, 0, 18, 6, 18), &[cell(12, 12, 22, 36, 120)], 24),
            Some(0)
        );
    }

    #[test]
    fn invalid_boxes_and_impossible_areas_in_any_input_are_rejected() {
        for d in [
            cell(18, 2, 14, 6, 12),
            cell(14, 6, 18, 2, 12),
            cell(14, 2, 18, 2, 12),
            cell(14, 2, 18, 6, 2),
            cell(14, 2, 18, 6, 17),
        ] {
            assert_eq!(owner(&d, &[stem()], 24), None);
        }
        assert_eq!(owner(&dot(), &[stem(), cell(0, 0, 0, 1, 3)], 24), None);
        assert_eq!(owner(&dot(), &[cell(12, 10, 22, 34, 241)], 24), None);
        assert_eq!(owner(&dot(), &[stem()], 0), None);
    }

    fn shifted(c: &Component, dx: usize, dy: usize) -> Component {
        cell(c.x0 + dx, c.y0 + dy, c.x1 + dx, c.y1 + dy, c.area)
    }

    #[test]
    fn reader_keeps_diagonal_neighbors_in_one_owned_component() {
        let mut rgb = vec![255; 3 * 3 * 3];
        for pixel in [0, 4, 8] {
            rgb[pixel * 3..pixel * 3 + 3].fill(0);
        }
        let before = rgb.clone();
        let parts = collect_all(&rgb, 3, 3).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(
            (
                parts[0].x0,
                parts[0].y0,
                parts[0].x1,
                parts[0].y1,
                parts[0].area
            ),
            (0, 0, 3, 3, 3)
        );
        assert_eq!(rgb, before);
    }

    #[test]
    fn reader_keeps_one_and_two_pixel_islands_instead_of_dropping_them() {
        let mut rgb = vec![255; 6 * 5 * 3];
        for pixel in [0, 22, 23] {
            rgb[pixel * 3..pixel * 3 + 3].fill(0);
        }
        let parts = collect_all(&rgb, 6, 5).unwrap();
        assert_eq!(parts.iter().map(|c| c.area).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(parts.iter().map(|c| c.area).sum::<usize>(), 3);
    }

    #[test]
    fn reader_rejects_invalid_or_excessive_sizes_and_accepts_white() {
        assert!(collect_all(&[255; 12], 2, 2).unwrap().is_empty());
        for (rgb, width, height) in [
            (&[0; 12][..], 0, 2),
            (&[0; 12][..], 2, 0),
            (&[0; 11][..], 2, 2),
            (&[][..], usize::MAX, usize::MAX),
        ] {
            assert!(collect_all(rgb, width, height).is_none());
        }
        assert!(collect_all(&vec![255; 65_537 * 3], 65_537, 1).is_none());
        assert!(
            collect_all(&vec![255; 65_536 * 3], 256, 256)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn reader_uses_the_same_inclusive_foreground_boundary_as_the_product() {
        let parts = collect_all(&[219, 220, 220, 220, 220, 220], 2, 1).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!((parts[0].x0, parts[0].x1, parts[0].area), (0, 1, 1));
    }

    #[test]
    fn coordinate_sums_near_usize_max_do_not_wrap_or_change_ownership() {
        let dx = usize::MAX - 24;
        let dy = usize::MAX - 36;
        assert_eq!(
            owner(&shifted(&dot(), dx, dy), &[shifted(&stem(), dx, dy)], 24),
            Some(0)
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn translation_preserves_ownership_and_ambiguity(
            dx in 0usize..=usize::MAX - 64,
            dy in 0usize..=usize::MAX - 64,
        ) {
            let d = shifted(&dot(), dx, dy);
            prop_assert_eq!(owner(&d, &[shifted(&stem(), dx, dy)], 24), Some(0));
            prop_assert_eq!(owner(&d, &[shifted(&stem(), dx, dy), shifted(&stem(), dx, dy)], 24), None);
        }

        #[test]
        fn the_same_relative_dot_has_one_owner_at_different_body_sizes(h in 16usize..=128) {
            let n = h.div_ceil(8);
            let top = n * 2;
            let d = cell(10 + n, 0, 10 + n * 2, n, n * n);
            let s = cell(10, top, 10 + h / 2, top + h, h * h / 4);
            prop_assert_eq!(owner(&d, &[s], h), Some(0));
        }
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_DOTTED_BODY_AUDIT_ROOT"]
    fn record_saved_component_owners() {
        let Ok(root) = std::env::var("NDLOCR_DOTTED_BODY_AUDIT_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let input: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("component-input.json")).unwrap())
                .unwrap();
        let mut records = Vec::new();
        for row in input["rows"].as_array().unwrap() {
            let rgb = std::fs::read(row["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let width = row["width"].as_u64().unwrap() as usize;
            let height = row["height"].as_u64().unwrap() as usize;
            let parts = collect_all(&rgb, width, height).unwrap();
            let actual: Vec<_> = parts
                .iter()
                .map(|c| [c.x0, c.y0, c.x1, c.y1, c.area])
                .collect();
            assert_eq!(serde_json::to_value(&actual).unwrap(), row["parts"]);
            assert_eq!(
                parts.iter().map(|c| c.area).sum::<usize>(),
                row["foreground"].as_u64().unwrap() as usize
            );
            let mut contexts = Vec::new();
            for leading in [false, true] {
                let references: Vec<_> = row["references"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|x| x["raw"].as_str().unwrap().starts_with([')', '）']) == leading)
                    .cloned()
                    .collect();
                if references.is_empty() || parts.len() < 2 {
                    continue;
                }
                let marker = if leading { 0 } else { parts.len() - 1 };
                let body: Vec<_> = parts
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != marker)
                    .collect();
                let body_height = body.iter().map(|(_, c)| c.y1 - c.y0).max().unwrap();
                let mut dots = Vec::new();
                for &(dot_index, dot) in &body {
                    let indices: Vec<_> = body
                        .iter()
                        .filter(|(i, _)| *i != dot_index)
                        .map(|(i, _)| *i)
                        .collect();
                    let stems: Vec<_> = indices
                        .iter()
                        .map(|&i| {
                            let c = &parts[i];
                            Component {
                                x0: c.x0,
                                y0: c.y0,
                                x1: c.x1,
                                y1: c.y1,
                                area: c.area,
                            }
                        })
                        .collect();
                    dots.push(serde_json::json!({"dot_index":dot_index,"owner_index":owner(dot, &stems, body_height).map(|i| indices[i])}));
                }
                contexts.push(serde_json::json!({"leading":leading,"marker_hypothesis_index":marker,"body_height":body_height,"references":references,"dot_hypotheses":dots}));
            }
            assert_eq!(rgb, before);
            records.push(serde_json::json!({"rgb_sha256":row["rgb_sha256"],"parts":actual,"contexts":contexts}));
        }
        std::fs::write(
            root.join("component-results.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        println!("{} saved RGB ownership inspections; no NN", records.len());
    }
}
