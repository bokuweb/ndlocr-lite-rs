//! Structural view of one raster neck between unequal narrow glyph cells.

pub(super) fn split_pair(
    mask: &[bool],
    width: usize,
    height: usize,
    body_height: usize,
) -> Option<[super::Component; 2]> {
    use super::single_pixel_bridge::take_region;
    if width < 3
        || height == 0
        || body_height < 6
        || height > body_height
        || height as u128 * 4 < body_height as u128 * 3
        || width.checked_mul(height) != Some(mask.len())
        || mask.len() > 65_536
        || width as u128 > body_height as u128 * 3 + 1
    {
        return None;
    }
    let area = mask.iter().filter(|&&v| v).count();
    for cut in 1..width - 1 {
        let mut rows = (0..height).filter(|&y| mask[y * width + cut]);
        let Some(row) = rows.next() else { continue };
        if rows.next().is_some() {
            continue;
        }
        let touches =
            |x| (row.saturating_sub(1)..=(row + 1).min(height - 1)).any(|y| mask[y * width + x]);
        if !touches(cut - 1) || !touches(cut + 1) {
            continue;
        }
        let mut remaining = mask.to_vec();
        remaining[row * width + cut] = false;
        let Some(left) = take_region(&mut remaining, width, height, 0, cut) else {
            continue;
        };
        let Some(right) = take_region(&mut remaining, width, height, cut + 1, width) else {
            continue;
        };
        if remaining.iter().any(|&v| v)
            || left.area + right.area + 1 != area
            || left.x0 != 0
            || left.x1 != cut
            || right.x0 != cut + 1
            || right.x1 != width
            || left.y0.min(right.y0) != 0
            || left.y1.max(right.y1) != height
        {
            continue;
        }
        let widths = [left.x1 - left.x0, right.x1 - right.x0];
        if widths.iter().any(|&w| {
            w as u128 * 2 < body_height as u128 || w as u128 * 2 > body_height as u128 * 3
        }) || widths[0].max(widths[1]) as u128 > widths[0].min(widths[1]) as u128 * 2
        {
            continue;
        }
        let shared_height = left.y1.min(right.y1).saturating_sub(left.y0.max(right.y0));
        if shared_height as u128 * 4 < body_height as u128 * 3 {
            continue;
        }
        return Some([left, right]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(left: usize, right: usize, height: usize, row: usize) -> (Vec<bool>, usize) {
        let width = left + right + 1;
        let mut mask = vec![true; width * height];
        for y in 0..height {
            mask[y * width + left] = y == row;
        }
        (mask, width)
    }

    #[test]
    fn one_pixel_neck_separates_unequal_cells_without_changing_the_source_mask() {
        let (mask, width) = pair(13, 21, 17, 16);
        let before = mask.clone();
        let cells = split_pair(&mask, width, 17, 17)
            .expect("a 13/21-width pair has one owned neck, away from its midpoint");
        let views: Vec<_> = cells
            .iter()
            .map(|c| (c.x0, c.y0, c.x1, c.y1, c.area))
            .collect();
        assert_eq!(views, [(0, 0, 13, 17, 221), (14, 0, 35, 17, 357)]);
        assert_eq!(cells.iter().map(|c| c.area).sum::<usize>() + 1, 579);
        assert_eq!(mask, before);
    }

    #[test]
    fn each_cell_is_at_least_half_the_body_height_wide() {
        for (left, right, expected) in [(9, 18, true), (8, 16, false)] {
            let (mask, width) = pair(left, right, 17, 16);
            assert_eq!(
                split_pair(&mask, width, 17, 17).is_some(),
                expected,
                "cell widths {left}/{right} at body height 17"
            );
        }
    }

    #[test]
    fn each_cell_is_at_most_three_halves_of_the_body_height_wide() {
        for (right, expected) in [(25, true), (26, false)] {
            let (mask, width) = pair(13, right, 17, 16);
            assert_eq!(
                split_pair(&mask, width, 17, 17).is_some(),
                expected,
                "right width {right} at body height 17"
            );
        }
    }

    #[test]
    fn the_wider_cell_cannot_exceed_twice_the_narrower_one() {
        for (left, right, expected) in [(12, 24, true), (12, 25, false), (25, 12, false)] {
            let (mask, width) = pair(left, right, 17, 16);
            assert_eq!(
                split_pair(&mask, width, 17, 17).is_some(),
                expected,
                "{left}/{right}"
            );
        }
    }

    #[test]
    fn the_reference_and_pair_heights_stay_within_the_fixed_range() {
        for (left, right, height, body_height, expected) in [
            (3, 6, 6, 6, true),
            (3, 6, 5, 5, false),
            (13, 21, 15, 20, true),
            (13, 21, 14, 20, false),
            (13, 21, 17, 16, false),
        ] {
            let (mask, width) = pair(left, right, height, height - 1);
            assert_eq!(
                split_pair(&mask, width, height, body_height).is_some(),
                expected,
                "pair height {height}, reference {body_height}"
            );
        }
    }

    #[test]
    fn the_two_cells_need_three_quarters_of_the_body_height_in_common() {
        for (right_top, expected) in [(3, true), (4, false)] {
            let (mut mask, width) = pair(13, 21, 20, 10);
            for y in 18..20 {
                mask[y * width..y * width + 13].fill(false);
            }
            for y in 0..right_top {
                mask[y * width + 14..(y + 1) * width].fill(false);
            }
            assert_eq!(
                split_pair(&mask, width, 20, 20).is_some(),
                expected,
                "shared height {}",
                18 - right_top
            );
        }
    }

    #[test]
    fn the_cut_pixel_must_touch_both_cells_using_eight_neighbors() {
        for (row, expected) in [(1, true), (0, false)] {
            for mirrored in [false, true] {
                let (mut mask, width) = pair(4, 8, 8, row);
                for y in 0..2 {
                    mask[y * width + 5..(y + 1) * width].fill(false);
                }
                if mirrored {
                    for pixels in mask.chunks_mut(width) {
                        pixels.reverse();
                    }
                }
                assert_eq!(
                    split_pair(&mask, width, 8, 8).is_some(),
                    expected,
                    "support row {row}, mirrored {mirrored}"
                );
            }
        }
    }

    #[test]
    fn the_support_search_stays_within_the_bbox_pixel_budget() {
        let (mask, width) = pair(128, 128, 255, 254);
        assert_eq!(mask.len(), 65_535);
        assert!(split_pair(&mask, width, 255, 256).is_some());
        let (mask, width) = pair(128, 128, 256, 255);
        assert!(mask.len() > 65_536);
        assert!(split_pair(&mask, width, 256, 256).is_none());
    }

    #[test]
    fn an_extra_island_is_not_discarded_when_owning_the_pair_pixels() {
        let (mut mask, width) = pair(13, 21, 17, 16);
        for y in 3..6 {
            for x in 3..6 {
                mask[y * width + x] = false;
            }
        }
        assert!(split_pair(&mask, width, 17, 17).is_some());
        mask[4 * width + 4] = true;
        let before = mask.clone();
        assert!(split_pair(&mask, width, 17, 17).is_none());
        assert_eq!(mask, before);
    }

    #[test]
    fn thick_and_disconnected_columns_do_not_supply_a_neck() {
        let (mut mask, width) = pair(13, 21, 17, 16);
        mask[15 * width + 13] = true;
        assert!(split_pair(&mask, width, 17, 17).is_none());
        mask[15 * width + 13] = false;
        mask[16 * width + 13] = false;
        assert!(split_pair(&mask, width, 17, 17).is_none());
    }

    #[test]
    fn the_views_must_cover_the_original_bbox_without_paper_padding() {
        let (mask, width) = pair(13, 21, 17, 16);
        let mut padded = vec![false; (width + 2) * 17];
        for y in 0..17 {
            let row = y * (width + 2) + 1;
            padded[row..row + width].copy_from_slice(&mask[y * width..(y + 1) * width]);
        }
        assert!(split_pair(&padded, width + 2, 17, 17).is_none());
        let mut padded = vec![false; width * 19];
        padded[width..width * 18].copy_from_slice(&mask);
        assert!(split_pair(&padded, width, 19, 19).is_none());
    }

    #[test]
    fn invalid_dimensions_and_mask_lengths_are_rejected() {
        for (width, height, body_height) in [
            (0, 1, 17),
            (1, 0, 17),
            (2, 1, 17),
            (35, 17, 17),
            (usize::MAX, usize::MAX, usize::MAX),
        ] {
            assert!(split_pair(&[false], width, height, body_height).is_none());
        }
        let (mask, width) = pair(13, 21, 17, 16);
        assert!(split_pair(&mask, width, 17, 0).is_none());
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_LATIN_PAIR_KERNEL_AUDIT_ROOT"]
    fn record_known_component_kernel_views() {
        let Ok(root) = std::env::var("NDLOCR_LATIN_PAIR_KERNEL_AUDIT_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let input: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("component-input.json")).unwrap())
                .unwrap();
        let images = input["rows"].as_array().unwrap();
        assert_eq!(images.len(), 27);
        let mut records = Vec::new();
        for image in images {
            let rgb = std::fs::read(image["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let width = image["width"].as_u64().unwrap() as usize;
            let height = image["height"].as_u64().unwrap() as usize;
            let id = image["references"][0]["id"].as_str().unwrap();
            let (parts, collector_height) =
                super::super::collect_components(&rgb, width, height).unwrap();
            let expected: Vec<_> = image["components"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let b = c["bbox"].as_array().unwrap();
                    [
                        b[0].as_u64().unwrap() as usize,
                        b[1].as_u64().unwrap() as usize,
                        b[2].as_u64().unwrap() as usize,
                        b[3].as_u64().unwrap() as usize,
                        c["area"].as_u64().unwrap() as usize,
                    ]
                })
                .collect();
            let actual: Vec<_> = parts
                .iter()
                .map(|c| [c.x0, c.y0, c.x1, c.y1, c.area])
                .collect();
            assert_eq!(actual, expected, "actual collector {id}");
            for (index, c) in parts.iter().enumerate() {
                let (w, h) = (c.x1 - c.x0, c.y1 - c.y0);
                let mut mask = Vec::with_capacity(w * h);
                for y in c.y0..c.y1 {
                    for x in c.x0..c.x1 {
                        let p = (y * width + x) * 3;
                        mask.push(rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660);
                    }
                }
                let foreground = mask.iter().filter(|&&v| v).count();
                let owned = foreground == c.area;
                let views = if owned {
                    split_pair(&mask, w, h, h)
                } else {
                    None
                };
                let local: Vec<_> = views
                    .as_ref()
                    .map(|cells| {
                        cells
                            .iter()
                            .map(|v| [v.x0, v.y0, v.x1, v.y1, v.area])
                            .collect()
                    })
                    .unwrap_or_default();
                let mut global = local.clone();
                for v in &mut global {
                    v[0] += c.x0;
                    v[2] += c.x0;
                    v[1] += c.y0;
                    v[3] += c.y0;
                }
                if !local.is_empty() {
                    assert_eq!(local.len(), 2);
                    assert_eq!(local.iter().map(|v| v[4]).sum::<usize>() + 1, foreground);
                }
                records.push(serde_json::json!({
                    "image_id":id, "component_index":index,
                    "bbox":[c.x0,c.y0,c.x1,c.y1], "area":c.area,
                    "foreground_in_bbox":foreground, "owns_bbox_foreground":owned,
                    "collector_height":collector_height, "reference_height":h,
                    "supported":!local.is_empty(), "local_views":local, "global_views":global
                }));
            }
            assert_eq!(rgb, before);
        }
        assert_eq!(records.len(), 84);
        let output = serde_json::json!({"images":27,"new_model_calls":0,"records":records});
        std::fs::write(
            root.join("actual-kernel.json"),
            serde_json::to_vec_pretty(&output).unwrap(),
        )
        .unwrap();
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn unequal_cells_keep_their_pixels_under_mirrors_and_neck_row_changes(
            height in 6usize..41, left_seed in 0usize..100, right_seed in 0usize..100,
            row_seed in 0usize..100, mirror_x in proptest::bool::ANY, mirror_y in proptest::bool::ANY,
        ) {
            let lower = height.div_ceil(2);
            let left = lower + left_seed % (height - lower + 1);
            let room = (3 * height / 2 - left).min(left);
            let right = left + right_seed % (room + 1);
            let row = row_seed % height;
            let row = if mirror_y { height - row - 1 } else { row };
            let (mut mask, width) = pair(left, right, height, row);
            if mirror_x { for pixels in mask.chunks_mut(width) { pixels.reverse(); } }
            let before = mask.clone();
            let cells = split_pair(&mask, width, height, height).unwrap();
            let expected = if mirror_x { [right, left] } else { [left, right] };
            proptest::prop_assert_eq!(cells.each_ref().map(|c| c.x1 - c.x0), expected);
            proptest::prop_assert_eq!(cells.iter().map(|c| c.area).sum::<usize>() + 1, mask.iter().filter(|&&v| v).count());
            proptest::prop_assert_eq!(mask, before);
        }
    }
}
