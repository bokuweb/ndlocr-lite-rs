//! A support-only view for two full-width glyphs joined by one raster pixel.
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
    {
        return None;
    }
    let leading = closer(chars[0]);
    if !leading && !closer(chars[2]) {
        return None;
    }
    if chars
        .iter()
        .filter(|&&c| !closer(c))
        .any(|c| c.width() != Some(2))
    {
        return None;
    }
    let (components, h) = collect_components(rgb, width, height)?;
    if components.len() != 2 {
        return None;
    }
    let mut iter = components.into_iter();
    let first = iter.next()?;
    let second = iter.next()?;
    let (body, mark) = if leading {
        (second, first)
    } else {
        (first, second)
    };
    let gap = if leading {
        body.x0.checked_sub(mark.x1)?
    } else {
        mark.x0.checked_sub(body.x1)?
    };
    let bh = body.y1 - body.y0;
    let bw = body.x1 - body.x0;
    let mh = mark.y1 - mark.y0;
    let mw = mark.x1 - mark.x0;
    if gap > 1
        || mh as u128 * 2 < bh as u128
        || mh as u128 * 2 > bh as u128 * 3
        || mw < bh.div_ceil(4)
        || mw as u128 * 2 > bh as u128
        || mark.area == mw * mh
    {
        return None;
    }
    let mask: Vec<_> = (body.y0..body.y1)
        .flat_map(|y| {
            (body.x0..body.x1).map(move |x| {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
                    .iter()
                    .map(|&v| u16::from(v))
                    .sum::<u16>()
                    < 660
            })
        })
        .collect();
    if mask.iter().filter(|&&v| v).count() != body.area {
        return None;
    }
    let mut pieces = split_body(&mask, bw, bh)?;
    for c in &mut pieces {
        if c.area < h.div_ceil(2).max(3) {
            return None;
        }
        c.x0 += body.x0;
        c.x1 += body.x0;
        c.y0 += body.y0;
        c.y1 += body.y0;
    }
    let [left, right] = pieces;
    Some((
        if leading {
            [mark, left, right]
        } else {
            [left, right, mark]
        },
        h,
    ))
}

fn split_body(mask: &[bool], width: usize, height: usize) -> Option<[Component; 2]> {
    if width < 3
        || height == 0
        || width.checked_mul(height) != Some(mask.len())
        || width as u128 > height as u128 * 3 + 1
    {
        return None;
    }
    let cut = (1..width - 1)
        .filter(|&x| (0..height).filter(|&y| mask[y * width + x]).count() == 1)
        .min_by_key(|&x| ((2 * x as u128 + 1).abs_diff(width as u128), x))?;
    let y = (0..height).find(|&y| mask[y * width + cut])?;
    let neighbors = y.saturating_sub(1)..=y.saturating_add(1).min(height - 1);
    if !neighbors.clone().any(|yy| mask[yy * width + cut - 1])
        || !neighbors.clone().any(|yy| mask[yy * width + cut + 1])
    {
        return None;
    }
    let mut view = mask.to_vec();
    view[y * width + cut] = false;
    let left = take_region(&mut view, width, height, 0, cut)?;
    let right = take_region(&mut view, width, height, cut + 1, width)?;
    if view.iter().any(|&v| v)
        || [&left, &right].iter().any(|c| {
            (c.y1 - c.y0) as u128 * 4 < height as u128 * 3
                || (c.x1 - c.x0) as u128 * 4 < height as u128 * 3
                || (c.x1 - c.x0) as u128 * 2 > height as u128 * 3
                || c.area < height.div_ceil(2).max(3)
        })
    {
        return None;
    }
    Some([left, right])
}

fn take_region(
    mask: &mut [bool],
    width: usize,
    height: usize,
    x0: usize,
    x1: usize,
) -> Option<Component> {
    let seed = (0..mask.len()).find(|&i| mask[i] && (x0..x1).contains(&(i % width)))?;
    mask[seed] = false;
    let mut pending = vec![seed];
    let mut c = Component {
        x0: width,
        y0: height,
        x1: 0,
        y1: 0,
        area: 0,
    };
    while let Some(i) = pending.pop() {
        let (x, y) = (i % width, i / width);
        c.x0 = c.x0.min(x);
        c.x1 = c.x1.max(x + 1);
        c.y0 = c.y0.min(y);
        c.y1 = c.y1.max(y + 1);
        c.area += 1;
        for yy in y.saturating_sub(1)..=y.saturating_add(1).min(height - 1) {
            for xx in x.saturating_sub(1).max(x0)..=x.saturating_add(1).min(x1 - 1) {
                let p = yy * width + xx;
                if mask[p] {
                    mask[p] = false;
                    pending.push(p);
                }
            }
        }
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic_orientation(row: &serde_json::Value) -> ParseqOrientation {
        match row["orientation"].as_str() {
            None | Some("auto") => ParseqOrientation::Auto,
            Some("horizontal") => ParseqOrientation::Horizontal,
            other => panic!("invalid diagnostic orientation: {other:?}"),
        }
    }

    #[test]
    fn replay_orientation_defaults_to_auto_and_honors_explicit_horizontal() {
        assert_eq!(
            diagnostic_orientation(&serde_json::json!({})),
            ParseqOrientation::Auto
        );
        assert_eq!(
            diagnostic_orientation(&serde_json::json!({"orientation":"horizontal"})),
            ParseqOrientation::Horizontal
        );
    }

    #[test]
    fn a_single_raster_bridge_separates_two_full_height_glyphs() {
        let (w, h) = (25, 12);
        let mut mask = vec![true; w * h];
        for y in 0..h {
            mask[y * w + 12] = y == 6;
        }
        let [left, right] = split_body(&mask, w, h).expect("one-pixel bridge");
        assert_eq!(
            (left.x0, left.y0, left.x1, left.y1, left.area),
            (0, 0, 12, 12, 144)
        );
        assert_eq!(
            (right.x0, right.y0, right.x1, right.y1, right.area),
            (13, 0, 25, 12, 144)
        );
        assert!(mask[6 * w + 12], "the original support mask is immutable");
    }

    fn picture(leading: bool, tone: u8) -> (Vec<u8>, usize, usize) {
        let (w, h) = (40, 22);
        let mut rgb = vec![255; w * h * 3];
        let bx = if leading { 8 } else { 4 };
        let mx = if leading { 4 } else { 30 };
        for y in 4..16 {
            for x in (bx..bx + 12).chain(bx + 13..bx + 25) {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(tone);
            }
        }
        rgb[(10 * w + bx + 12) * 3..(10 * w + bx + 13) * 3].fill(tone);
        for y in 0..12 {
            let x = mx
                + if y < 3 {
                    y
                } else if y >= 9 {
                    11 - y
                } else {
                    2
                };
            rgb[((y + 4) * w + x) * 3..((y + 4) * w + x + 1) * 3].fill(tone);
        }
        (rgb, w, h)
    }

    #[test]
    fn supports_compact_closer_without_changing_the_original_rgb() {
        for (leading, text) in [(false, "情報)"), (true, ")情報")] {
            let (rgb, w, h) = picture(leading, 0);
            let original = rgb.clone();
            assert!(supports(text, &rgb, w, h));
            assert_eq!(rgb, original);
        }
    }

    #[test]
    fn the_real_two_glyph_bridge_explains_the_existing_loss() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/single_pixel_bridge/info_round.png");
        let image = crate::io::load_rgb_u8(&path).unwrap();
        assert_eq!(
            sanitize_visible_single_line(
                "情報)",
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Horizontal
            ),
            "情報)"
        );
        assert!(supports("情報)", &image.data, image.width, image.height));
    }

    #[test]
    fn unrelated_text_invalid_images_and_second_connections_keep_the_baseline() {
        let (mut rgb, w, h) = picture(false, 0);
        for text in [
            "情報", "情報))", ")情)", "情)報", "AB)", "情報 )", " 情報)", "情)",
        ] {
            assert!(!supports(text, &rgb, w, h), "{text}");
        }
        assert!(!supports("情報)", &[], 0, h));
        assert!(!supports("情報)", &rgb, usize::MAX, h));
        assert!(!supports("情報)", &vec![255; w * h * 3], w, h));
        rgb[(12 * w + 16) * 3..(12 * w + 17) * 3].fill(0);
        assert!(
            !supports("情報)", &rgb, w, h),
            "two bridge pixels are not a single raster contact"
        );
    }

    #[test]
    fn split_width_and_height_bounds_are_closed_and_invalid_masks_do_not_split() {
        let make = |lw: usize, rw: usize, lh: usize, rh: usize| {
            let (w, h) = (lw + rw + 1, lh.max(rh));
            let mut mask = vec![false; w * h];
            for y in 0..lh {
                for x in 0..lw {
                    mask[y * w + x] = true;
                }
            }
            for y in 0..rh {
                for x in lw + 1..w {
                    mask[y * w + x] = true;
                }
            }
            mask[(lh.min(rh) / 2) * w + lw] = true;
            (mask, w, h)
        };
        for (lw, rw, lh, rh, expected) in [
            (9, 18, 12, 12, true),
            (8, 18, 12, 12, false),
            (9, 19, 12, 12, false),
            (12, 12, 9, 12, true),
            (12, 12, 8, 12, false),
        ] {
            let (mask, w, h) = make(lw, rw, lh, rh);
            assert_eq!(split_body(&mask, w, h).is_some(), expected);
        }
        assert!(split_body(&[], 0, 0).is_none());
        assert!(split_body(&[true], usize::MAX, 2).is_none());
        let (mut mask, w, h) = make(12, 12, 12, 12);
        mask[6 * w + 12] = false;
        assert!(
            split_body(&mask, w, h).is_none(),
            "already separated glyphs do not use this fallback"
        );
    }

    #[test]
    fn filled_marks_wide_gaps_and_unowned_inner_noise_do_not_support_closers() {
        let (mut rgb, w, h) = picture(false, 0);
        for y in 4..16 {
            for x in 30..33 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
        assert!(
            !supports("情報)", &rgb, w, h),
            "a solid rule is not a curved mark"
        );
        let (original, _, _) = picture(false, 0);
        let mut shifted = original.clone();
        for y in 4..16 {
            for x in 30..34 {
                shifted[(y * w + x) * 3..(y * w + x + 1) * 3].fill(255);
            }
            shifted[(y * w + 31) * 3..(y * w + 34) * 3]
                .copy_from_slice(&original[(y * w + 30) * 3..(y * w + 33) * 3]);
        }
        assert!(
            !supports("情報)", &shifted, w, h),
            "gap2 cannot restore a missing word separator"
        );
        let (mut outline, w, h) = picture(false, 0);
        for y in 5..15 {
            for x in (5..15).chain(18..28) {
                outline[(y * w + x) * 3..(y * w + x + 1) * 3].fill(255);
            }
        }
        assert!(supports("情報)", &outline, w, h));
        outline[(10 * w + 10) * 3..(10 * w + 11) * 3].fill(0);
        assert!(
            !supports("情報)", &outline, w, h),
            "ignored inner dots must not become glyph evidence"
        );
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn pure_support_is_invariant_under_translation_white_paper_and_gray_ink(
            leading in proptest::bool::ANY,mx in 0usize..10,my in 0usize..10,tone in 0u8..220,
        ) {
            let(rgb,w,h)=picture(leading,tone);let(pw,ph)=(w+2*mx,h+2*my);let mut padded=vec![255;pw*ph*3];
            for y in 0..h {padded[((y+my)*pw+mx)*3..((y+my)*pw+mx+w)*3].copy_from_slice(&rgb[y*w*3..(y+1)*w*3]);}
            let before=padded.clone();let text=if leading {")情報"}else{"情報)"};
            proptest::prop_assert!(supports(text,&padded,pw,ph));proptest::prop_assert_eq!(padded,before);
        }
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_BRIDGE_ROOT with fixed real neural outputs"]
    fn record_existing_bridge_policy() {
        let Ok(root) = std::env::var("NDLOCR_BRIDGE_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("existing-manifest.json")).unwrap())
                .unwrap();
        let output = root.join("existing-replay");
        std::fs::create_dir(&output).expect("new output directory");
        let mut records = Vec::new();
        for x in manifest["cases"].as_array().unwrap() {
            let raw = x["raw"].as_str().unwrap();
            let id = x["id"].as_str().unwrap();
            assert!(
                id.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
            );
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let orientation = diagnostic_orientation(x);
            let baseline = sanitize_visible_single_line(raw, &rgb, w, h, orientation);
            if let Some(expected) = x["expected_baseline"].as_str() {
                assert_eq!(baseline, expected, "baseline {id}");
            }
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
            "{} fixed neural pairs passed through baseline and bridge diagnostic;no NN forwards",
            records.len()
        );
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_BRIDGE_PRODUCT_MANIFEST with frozen candidate outputs"]
    fn verify_product_matches_frozen_bridge_policy() {
        let Ok(path) = std::env::var("NDLOCR_BRIDGE_PRODUCT_MANIFEST") else {
            return;
        };
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let rows = manifest["cases"].as_array().unwrap();
        assert_eq!(rows.len(), 33735);
        for x in rows {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let raw = x["raw"].as_str().unwrap();
            let actual = sanitize_visible_single_line(raw, &rgb, w, h, diagnostic_orientation(x));
            assert_eq!(
                actual,
                x["expected"].as_str().unwrap(),
                "{}/{}",
                x["corpus"],
                x["id"]
            );
        }
        println!(
            "{} frozen raw pairs match the actual product policy",
            rows.len()
        );
    }

    #[test]
    fn explicit_counterclockwise_rgb_is_bit_equal_to_auto_rotation() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let rgb: Vec<u8> = (0..18).collect();
        let rotated: Vec<u8> = [1usize, 3, 5, 0, 2, 4]
            .into_iter()
            .flat_map(|p| rgb[p * 3..p * 3 + 3].to_vec())
            .collect();
        for (w, h) in [(256, 24), (384, 24), (768, 24)] {
            let a = preprocess_rgb_u8_with_orientation(&rgb, 2, 3, w, h, ParseqOrientation::Auto)
                .unwrap();
            let b = preprocess_rgb_u8_with_orientation(
                &rotated,
                3,
                2,
                w,
                h,
                ParseqOrientation::Horizontal,
            )
            .unwrap();
            assert!(a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits()));
        }
    }

    #[test]
    #[ignore = "diagnostic; validates every generated effective RGB before fresh NN inference"]
    fn verify_fresh_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_BRIDGE_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 3648);
        for x in rows {
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let forward = std::fs::read(x["effective_rgb"].as_str().unwrap()).unwrap();
            let (w, h) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let (fw, fh) = (
                x["forward_width"].as_u64().unwrap() as usize,
                x["forward_height"].as_u64().unwrap() as usize,
            );
            for (iw, ih) in [(256, 24), (384, 24), (768, 24)] {
                let a =
                    preprocess_rgb_u8_with_orientation(&rgb, w, h, iw, ih, ParseqOrientation::Auto)
                        .unwrap();
                let b = preprocess_rgb_u8_with_orientation(
                    &forward,
                    fw,
                    fh,
                    iw,
                    ih,
                    ParseqOrientation::Horizontal,
                )
                .unwrap();
                assert!(
                    a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits()),
                    "{} {iw}",
                    x["id"]
                );
            }
        }
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":rows.len(),"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }
}
