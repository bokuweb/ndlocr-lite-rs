//! Structural support for a round closer beside a dotted Latin body.

use super::Component;

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    let chars: Vec<_> = text.chars().take(10).collect();
    let round = |c| matches!(c, ')' | '）');
    if !(4..=9).contains(&chars.len()) || chars.iter().filter(|&&c| round(c)).count() != 1 {
        return false;
    }
    let leading = round(chars[0]);
    if !leading && !round(*chars.last().unwrap()) {
        return false;
    }
    let body = if leading {
        &chars[1..]
    } else {
        &chars[..chars.len() - 1]
    };
    let dotted = |c| matches!(c, 'i' | 'j');
    let dot_count = body.iter().filter(|&&c| dotted(c)).count();
    if body.iter().any(|c| !c.is_ascii_alphabetic()) || !(1..=2).contains(&dot_count) {
        return false;
    }
    let Some(mut parts) = super::collect_all(rgb, width, height) else {
        return false;
    };
    if parts.len() != body.len() + dot_count + 1 || parts.iter().any(|c| c.area < 3) {
        return false;
    }
    if parts.iter().any(|c| {
        (c.y0..c.y1)
            .map(|y| {
                (c.x0..c.x1)
                    .filter(|&x| {
                        let p = (y * width + x) * 3;
                        rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660
                    })
                    .count()
            })
            .sum::<usize>()
            != c.area
    }) {
        return false;
    }
    let mark = if leading {
        parts.remove(0)
    } else {
        parts.pop().unwrap()
    };
    let h = parts.iter().map(|c| c.y1 - c.y0).max().unwrap();
    if h < 8 {
        return false;
    }
    let (stems, dots): (Vec<_>, Vec<_>) = parts
        .into_iter()
        .partition(|c| (c.y1 - c.y0) as u128 * 4 >= h as u128 * 3);
    if stems.len() != body.len()
        || dots.len() != dot_count
        || dots.iter().any(|c| c.y1 - c.y0 > h / 3)
        || stems.iter().any(|c| {
            c.x1 - c.x0 < h.div_ceil(8)
                || (c.x1 - c.x0) as u128 * 2 > h as u128 * 3
                || c.area < h.div_ceil(2).max(3)
        })
        || stems
            .windows(2)
            .any(|p| p[0].x1 > p[1].x0 || p[1].x0 - p[0].x1 > h.div_ceil(4))
    {
        return false;
    }
    let top = stems.iter().map(|c| c.y0).max().unwrap();
    let bottom = stems.iter().map(|c| c.y1).min().unwrap();
    let max_bottom = stems.iter().map(|c| c.y1).max().unwrap();
    if bottom.saturating_sub(top) as u128 * 3 < h as u128 * 2 || max_bottom - bottom > h.div_ceil(3)
    {
        return false;
    }
    let gap = if leading {
        stems[0].x0.checked_sub(mark.x1)
    } else {
        mark.x0.checked_sub(stems.last().unwrap().x1)
    };
    if gap.is_none_or(|g| g > h.div_ceil(4))
        || !marker_supported(&mark, rgb, width, height, h, top, bottom)
    {
        return false;
    }
    let mut owned = vec![false; stems.len()];
    for dot in dots {
        let Some(index) = super::owner(&dot, &stems, h) else {
            return false;
        };
        if !dotted(body[index]) || owned[index] {
            return false;
        }
        owned[index] = true;
    }
    body.iter().enumerate().all(|(i, &c)| dotted(c) == owned[i])
}

fn marker_supported(
    mark: &Component,
    rgb: &[u8],
    width: usize,
    height: usize,
    body_height: usize,
    core_top: usize,
    core_bottom: usize,
) -> bool {
    if body_height < 8
        || width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
        || mark.x0 >= mark.x1
        || mark.y0 >= mark.y1
        || mark.x1 > width
        || mark.y1 > height
        || core_top >= core_bottom
        || mark.y0 > core_top
        || mark.y1 < core_bottom
    {
        return false;
    }
    let (mw, mh) = (mark.x1 - mark.x0, mark.y1 - mark.y0);
    if mh as u128 * 4 < body_height as u128 * 3
        || mh as u128 * 2 > body_height as u128 * 3
        || mw < mh.div_ceil(6).max(3)
        || mw > mh / 3
    {
        return false;
    }
    let rows: Vec<Vec<_>> = (mark.y0..mark.y1)
        .map(|y| {
            (mark.x0..mark.x1)
                .filter(|&x| {
                    let p = (y * width + x) * 3;
                    rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660
                })
                .collect()
        })
        .collect();
    if rows.iter().any(|r| {
        r.is_empty() || r.len() > (mw * 3).div_ceil(4) || r.windows(2).any(|p| p[1] != p[0] + 1)
    }) || *rows.first().unwrap().last().unwrap() >= mark.x0 + mw.div_ceil(2)
        || *rows.last().unwrap().last().unwrap() >= mark.x0 + mw.div_ceil(2)
        || rows.iter().map(Vec::len).sum::<usize>() != mark.area
    {
        return false;
    }
    let right_band = |r: &[usize]| r[0] >= mark.x0 + mw / 2;
    let center = mh / 2;
    if !right_band(&rows[center]) || *rows[center].last().unwrap() != mark.x1 - 1 {
        return false;
    }
    let (mut top, mut bottom) = (center, center + 1);
    while top > 0 && right_band(&rows[top - 1]) {
        top -= 1;
    }
    while bottom < mh && right_band(&rows[bottom]) {
        bottom += 1;
    }
    bottom - top >= mh.div_ceil(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn curve(mw: usize, mh: usize) -> (Vec<u8>, Component, usize, usize) {
        let (w, h) = (mw + 6, mh + 6);
        let mut rgb = vec![255; w * h * 3];
        let flank = mh / 3;
        for y in 0..mh {
            let x = ((y.min(mh - y - 1) * (mw - 1)) / flank).min(mw - 1);
            rgb[((y + 3) * w + x + 3) * 3..((y + 3) * w + x + 4) * 3].fill(0);
        }
        (
            rgb,
            Component {
                x0: 3,
                y0: 3,
                x1: 3 + mw,
                y1: 3 + mh,
                area: mh,
            },
            w,
            h,
        )
    }

    #[test]
    fn round_outline_keeps_its_own_width_and_body_height_bounds() {
        for (mw, expected) in [(3, false), (4, true), (8, true), (9, false)] {
            let (rgb, mark, w, h) = curve(mw, 24);
            assert_eq!(
                marker_supported(&mark, &rgb, w, h, 24, 9, 21),
                expected,
                "width {mw}"
            );
        }
        for (mh, expected) in [(17usize, false), (18, true), (36, true), (37, false)] {
            let (rgb, mark, w, h) = curve(mh.div_ceil(6), mh);
            assert_eq!(
                marker_supported(&mark, &rgb, w, h, 24, 8, 18),
                expected,
                "height {mh}"
            );
        }
    }

    #[test]
    fn marker_must_cover_the_whole_common_body_interval() {
        let (rgb, mark, w, h) = curve(6, 24);
        assert!(marker_supported(&mark, &rgb, w, h, 24, 3, 27));
        assert!(!marker_supported(&mark, &rgb, w, h, 24, 2, 27));
        assert!(!marker_supported(&mark, &rgb, w, h, 24, 3, 28));
        assert!(!marker_supported(&mark, &rgb, w, h, 24, 8, 8));
    }

    #[test]
    fn a_connected_curved_band_can_reach_its_farthest_column_only_at_the_peak() {
        let (w, h) = (12, 30);
        let mut rgb = vec![255; w * h * 3];
        let path = [
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5, 5, 4, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0,
        ];
        for (y, x) in path.into_iter().enumerate() {
            let p = ((y + 3) * w + x + 3) * 3;
            rgb[p..p + 3].fill(0);
        }
        let mut parts = super::super::collect_all(&rgb, w, h).unwrap();
        assert_eq!(parts.len(), 1, "the complete raster curve is connected");
        let mark = parts.remove(0);
        assert_eq!(
            (mark.x1 - mark.x0, mark.y1 - mark.y0, mark.area),
            (6, 24, 24)
        );
        assert!(marker_supported(&mark, &rgb, w, h, 24, 9, 21));
        let mut off_center = rgb.clone();
        let center = ((mark.y0 + 12) * w + mark.x1 - 1) * 3;
        off_center[center..center + 3].fill(255);
        off_center[center - 3..center].fill(0);
        assert!(
            !marker_supported(&mark, &off_center, w, h, 24, 9, 21),
            "the center still has to reach the original right edge"
        );
    }

    #[test]
    fn rectangles_branches_empty_rows_and_a_mirrored_curve_are_rejected() {
        let (rgb, mark, w, h) = curve(6, 24);
        for kind in 0..4 {
            let mut changed = rgb.clone();
            if kind == 0 {
                for y in mark.y0..mark.y1 {
                    for x in mark.x0..mark.x1 {
                        changed[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                    }
                }
            } else if kind == 1 {
                let y = mark.y0 + 12;
                changed[(y * w + mark.x0) * 3..(y * w + mark.x0 + 1) * 3].fill(0);
            } else if kind == 2 {
                let y = mark.y0 + 12;
                changed[(y * w + mark.x0) * 3..(y * w + mark.x1) * 3].fill(255);
            } else {
                for y in mark.y0..mark.y1 {
                    for x in mark.x0..mark.x1 {
                        let target = (y * w + x) * 3;
                        let source = (y * w + mark.x0 + mark.x1 - x - 1) * 3;
                        changed[target..target + 3].copy_from_slice(&rgb[source..source + 3]);
                    }
                }
            }
            let mut parts = super::super::collect_all(&changed, w, h).unwrap();
            if parts.len() == 1 {
                let actual = parts.remove(0);
                assert!(
                    !marker_supported(&actual, &changed, w, h, 24, 9, 21),
                    "outline {kind}"
                );
            } else {
                assert!(
                    !marker_supported(&mark, &changed, w, h, 24, 9, 21),
                    "outline {kind}"
                );
            }
        }
    }

    #[test]
    fn invalid_shapes_and_foreign_foreground_are_rejected() {
        let (rgb, mut mark, w, h) = curve(6, 24);
        mark.area += 1;
        assert!(!marker_supported(&mark, &rgb, w, h, 24, 9, 21));
        mark.area -= 1;
        assert!(!marker_supported(
            &mark,
            &rgb[..rgb.len() - 1],
            w,
            h,
            24,
            9,
            21
        ));
        assert!(!marker_supported(&mark, &rgb, 0, h, 24, 9, 21));
        assert!(!marker_supported(&mark, &rgb, w, 0, 24, 9, 21));
        assert!(!marker_supported(
            &mark,
            &rgb,
            usize::MAX,
            usize::MAX,
            24,
            9,
            21
        ));
        mark.x1 = w + 1;
        assert!(!marker_supported(&mark, &rgb, w, h, 24, 9, 21));
    }

    fn paint(rgb: &mut [u8], w: usize, bbox: [usize; 4], value: u8) {
        let [x0, y0, x1, y1] = bbox;
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(value);
            }
        }
    }

    fn synthetic(a_width: usize) -> (Vec<u8>, usize, usize) {
        let (w, h) = (100, 48);
        let mut rgb = vec![255; w * h * 3];
        let (mw, mh, flank) = (8, 30, 10);
        for y in 0..mh {
            let x = (y.min(mh - y - 1) * (mw - 1) / flank).min(mw - 1) + 2;
            paint(&mut rgb, w, [x, y + 2, x + 1, y + 3], 0);
        }
        let i = 14 + a_width + 4;
        let j = i + 12;
        for bbox in [
            [14, 8, 14 + a_width, 32],
            [i, 8, i + 8, 32],
            [j, 8, j + 8, 32],
            [i + 2, 1, i + 6, 5],
            [j + 2, 1, j + 6, 5],
        ] {
            paint(&mut rgb, w, bbox, 0);
        }
        (rgb, w, h)
    }

    fn shifted_region(rgb: &[u8], w: usize, bbox: [usize; 4], dx: usize, dy: usize) -> Vec<u8> {
        let mut changed = rgb.to_vec();
        paint(&mut changed, w, bbox, 255);
        let [x0, y0, x1, y1] = bbox;
        for y in y0..y1 {
            for x in x0..x1 {
                let from = (y * w + x) * 3;
                let to = ((y + dy) * w + x + dx) * 3;
                changed[to..to + 3].copy_from_slice(&rgb[from..from + 3]);
            }
        }
        changed
    }

    #[test]
    fn a_dotted_body_keeps_one_or_two_dots_at_their_character_positions() {
        let (rgb, w, h) = synthetic(12);
        assert!(supports(")Aij", &rgb, w, h));
        assert!(supports("）Aij", &rgb, w, h));
        assert!(!supports(")iAj", &rgb, w, h));
        let mut one_dot = rgb.clone();
        paint(&mut one_dot, w, [44, 1, 48, 5], 255);
        assert!(supports(")Aip", &one_dot, w, h));
        assert!(!supports(")Aij", &one_dot, w, h));
        assert_eq!(rgb, synthetic(12).0);
    }

    #[test]
    fn original_prefix_and_suffix_agjp_pixels_are_supported_but_joined_bodies_are_not() {
        for (name, text, expected) in [
            ("agjp_round_prefix_maru40.png", ")Agjp", true),
            ("agjp_round_suffix_maru40.png", "Agjp)", true),
            ("agjp_round_suffix_joined_maru20.png", "Agjp)", false),
            ("qjp_round_suffix_joined_maru40.png", "qjp)", false),
        ] {
            let path = std::path::Path::new("tests/fixtures/latin_dotted_body").join(name);
            let image = crate::io::load_rgb_u8(&path).unwrap();
            let before = image.data.clone();
            assert_eq!(
                supports(text, &image.data, image.width, image.height),
                expected,
                "{name}"
            );
            assert_eq!(image.data, before);
        }
    }

    #[test]
    fn spaces_non_latin_internal_multiple_and_mismatched_closers_are_rejected() {
        let (rgb, w, h) = synthetic(12);
        for text in [
            "", ")Ai", ")Aijjj", ")Aij ", ") Aij", ")Ａij", ")情報", ")ABC", "A)ij", ")Aij)",
            "」Aij", "Aij)", ")AII",
        ] {
            assert!(!supports(text, &rgb, w, h), "{text:?}");
        }
    }

    #[test]
    fn missing_extra_misplaced_dots_and_tiny_unowned_islands_are_rejected() {
        let (rgb, w, h) = synthetic(12);
        for kind in 0..4 {
            let mut changed = rgb.clone();
            match kind {
                0 => paint(&mut changed, w, [32, 1, 36, 5], 255),
                1 => paint(&mut changed, w, [80, 1, 84, 5], 0),
                2 => {
                    paint(&mut changed, w, [32, 1, 36, 5], 255);
                    paint(&mut changed, w, [16, 1, 20, 5], 0);
                }
                _ => paint(&mut changed, w, [80, 40, 81, 41], 0),
            }
            assert!(!supports(")Aij", &changed, w, h), "dot variant {kind}");
        }
    }

    #[test]
    fn stem_width_and_common_height_keep_the_fixed_boundaries() {
        for (aw, expected) in [(2, false), (3, true), (36, true), (37, false)] {
            let (rgb, w, h) = synthetic(aw);
            assert_eq!(supports(")Aij", &rgb, w, h), expected, "stem width {aw}");
        }
        let (rgb, w, h) = synthetic(12);
        for (shift, expected) in [(8, true), (9, false)] {
            let moved = shifted_region(&rgb, w, [30, 0, 39, 34], 0, shift);
            assert_eq!(
                supports(")Aij", &moved, w, h),
                expected,
                "core shift {shift}"
            );
        }
        for (ah, expected) in [(17, false), (18, true)] {
            let mut shortened = rgb.clone();
            paint(&mut shortened, w, [14, 8 + ah, 26, 32], 255);
            assert_eq!(
                supports(")Aij", &shortened, w, h),
                expected,
                "stem height {ah}"
            );
        }
    }

    #[test]
    fn marker_and_stem_gaps_stop_at_a_quarter_height() {
        let (rgb, w, h) = synthetic(12);
        for (shift, expected) in [(2, true), (3, false)] {
            let body_moved = shifted_region(&rgb, w, [14, 0, 51, 34], shift, 0);
            assert_eq!(
                supports(")Aij", &body_moved, w, h),
                expected,
                "marker gap shift {shift}"
            );
            let last_moved = shifted_region(&rgb, w, [42, 0, 51, 34], shift, 0);
            assert_eq!(
                supports(")Aij", &last_moved, w, h),
                expected,
                "stem gap shift {shift}"
            );
        }
    }

    #[test]
    fn invalid_rgb_and_a_filled_marker_do_not_support_text() {
        let (rgb, w, h) = synthetic(12);
        for (bytes, width, height) in [
            (&rgb[..rgb.len() - 1], w, h),
            (rgb.as_slice(), 0, h),
            (rgb.as_slice(), w, 0),
            (rgb.as_slice(), usize::MAX, usize::MAX),
        ] {
            assert!(!supports(")Aij", bytes, width, height));
        }
        let mut rectangle = rgb.clone();
        paint(&mut rectangle, w, [2, 2, 10, 32], 0);
        assert!(!supports(")Aij", &rectangle, w, h));
    }

    #[test]
    fn three_through_eight_latin_stems_fit_the_bounded_text_form() {
        for (n, expected) in [(2, false), (3, true), (8, true), (9, false)] {
            let (source, w, h) = synthetic(12);
            let width = 160;
            let mut rgb = vec![255; width * h * 3];
            for y in 0..h {
                rgb[(y * width) * 3..(y * width + 12) * 3]
                    .copy_from_slice(&source[(y * w) * 3..(y * w + 12) * 3]);
            }
            let mut body = String::new();
            for i in 0..n {
                let x = 14 + i * 12;
                paint(&mut rgb, width, [x, 8, x + 8, 32], 0);
                if i + 1 == n {
                    body.push('j');
                    paint(&mut rgb, width, [x + 2, 1, x + 6, 5], 0);
                } else {
                    body.push(char::from(b'A' + i as u8));
                }
            }
            assert_eq!(
                supports(&format!("){body}"), &rgb, width, h),
                expected,
                "body length {n}"
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn paper_translation_and_gray_levels_preserve_the_same_owned_raster(
            dx in 0usize..=12, dy in 0usize..=12,
            foreground in 0u8..=219, background in 220u8..=255,
        ) {
            let (source, w, h) = synthetic(12);
            let (width, height) = (w + dx * 2, h + dy * 2);
            let mut rgb = vec![background; width * height * 3];
            for y in 0..h {
                for x in 0..w {
                    let value = if source[(y * w + x) * 3] == 0 { foreground } else { background };
                    let p = ((y + dy) * width + x + dx) * 3;
                    rgb[p..p + 3].fill(value);
                }
            }
            prop_assert!(supports(")Aij", &rgb, width, height));
        }

        #[test]
        fn every_separate_one_or_two_pixel_island_remains_unowned(
            x in 70usize..=95, y in 35usize..=45, count in 1usize..=2,
            diagonal in 0usize..=1, foreground in 0u8..=219,
        ) {
            let (mut rgb, w, h) = synthetic(12);
            for i in 0..count {
                let p = ((y + i * diagonal) * w + x + i) * 3;
                rgb[p..p + 3].fill(foreground);
            }
            prop_assert!(!supports(")Aij", &rgb, w, h));
        }
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_DOTTED_POLICY_ROOT"]
    fn record_fixed_policy_on_saved_raw() {
        use super::super::super::{
            ParseqOrientation, sanitize_visible_repetitions, sanitize_visible_single_line,
        };
        let Ok(root) = std::env::var("NDLOCR_DOTTED_POLICY_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let input: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("policy-input.json")).unwrap())
                .unwrap();
        let data: serde_json::Value = serde_json::from_slice(
            &std::fs::read(input["cases_source"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        let cases = data["cases"].as_array().unwrap();
        assert_eq!(
            cases.len(),
            input["expected_cases"].as_u64().unwrap() as usize
        );
        let mut changes = Vec::new();
        for (index, row) in cases.iter().enumerate() {
            let rgb = std::fs::read(row["rgb"].as_str().unwrap()).unwrap();
            let before_rgb = rgb.clone();
            let raw = row["raw"].as_str().unwrap();
            let width = row["width"].as_u64().unwrap() as usize;
            let height = row["height"].as_u64().unwrap() as usize;
            let orientation = match row["orientation"].as_str() {
                None | Some("auto") => ParseqOrientation::Auto,
                Some("horizontal") => ParseqOrientation::Horizontal,
                other => panic!("invalid orientation {other:?}"),
            };
            let current = sanitize_visible_single_line(raw, &rgb, width, height, orientation);
            assert_eq!(
                current,
                row["expected_after"].as_str().unwrap(),
                "baseline {index}/{}/{}",
                row["corpus"],
                row["id"]
            );
            let fragment = sanitize_visible_repetitions(raw, &rgb, width, height, orientation);
            let selected = fragment.chars().count() > current.chars().count()
                && (orientation == ParseqOrientation::Horizontal
                    || height as u128 * 5 <= width as u128 * 4)
                && supports(&fragment, &rgb, width, height);
            if selected {
                changes.push(serde_json::json!({"index":index,"corpus":row["corpus"],"model":row["model"],"id":row["id"],"raw":raw,"truth":row["truth"],"before":current,"after":fragment,"rgb_sha256":row["rgb_sha256"]}));
            }
            assert_eq!(rgb, before_rgb);
        }
        let result = serde_json::json!({"cases_checked":cases.len(),"changes":changes});
        std::fs::write(
            root.join("policy-results.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        println!(
            "{} actual baseline and fixed candidate pairs checked; no NN",
            cases.len()
        );
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_DOTTED_POLICY_ROOT; hypothetical raw only"]
    fn record_preregistered_fresh_geometry() {
        let Ok(root) = std::env::var("NDLOCR_DOTTED_POLICY_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let mut records = Vec::new();
        for x in m["inputs"].as_array().unwrap() {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let truth = std::fs::read_to_string(x["truth"].as_str().unwrap()).unwrap();
            let width = x["width"].as_u64().unwrap() as usize;
            let height = x["height"].as_u64().unwrap() as usize;
            let hypotheses = if x["kind"] == "text" {
                vec![truth]
            } else {
                [')', '）']
                    .into_iter()
                    .map(|c| {
                        if x["leading"].as_bool().unwrap() {
                            format!("{c}{truth}")
                        } else {
                            format!("{truth}{c}")
                        }
                    })
                    .collect()
            };
            for hypothetical_raw in hypotheses {
                records.push(serde_json::json!({"id":x["id"],"kind":x["kind"],"rgb_sha256":x["rgb_sha256"],"hypothetical_raw":hypothetical_raw,"geometry_supported":supports(&hypothetical_raw,&rgb,width,height),"original_auto_is_horizontal":height as u128*5<=width as u128*4}));
            }
            assert_eq!(rgb, before);
        }
        std::fs::write(
            root.join("fresh/geometric-hypotheses.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        println!(
            "{} artificial raw hypotheses checked; no NN; not accuracy",
            records.len()
        );
    }
}
