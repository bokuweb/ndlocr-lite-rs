//! Diagnostic support view for a compact internal closer and detached glyph parts.
use super::*;

pub(super) fn supports(text: &str, rgb: &[u8], width: usize, height: usize) -> bool {
    use super::detached_body_components::{edge_shared_large_child, group_view, primary_part};
    use unicode_width::UnicodeWidthChar;
    let chars: Vec<_> = text.chars().take(4).collect();
    let closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if chars.len() != 3
        || !closer(chars[1])
        || [chars[0], chars[2]]
            .iter()
            .any(|&c| closer(c) || c.is_whitespace() || c.width() != Some(2))
    {
        return false;
    }
    let Some((components, h)) = collect_components(rgb, width, height) else {
        return false;
    };
    if !(4..=7).contains(&components.len()) {
        return false;
    }
    let x0 = components.iter().map(|c| c.x0).min().unwrap();
    let x1 = components.iter().map(|c| c.x1).max().unwrap();
    let y0 = components.iter().map(|c| c.y0).min().unwrap();
    let y1 = components.iter().map(|c| c.y1).max().unwrap();
    let owned = components.iter().map(|c| c.area).sum::<usize>();
    let foreground = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < 660
    };
    if (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| foreground(x, y))
        .count()
        != owned
    {
        return false;
    }
    let mut supported = 0;
    for (i, mark) in components.iter().enumerate() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut overlap = false;
        for (j, c) in components.iter().enumerate() {
            if i == j {
                continue;
            }
            if c.x1 <= mark.x0 {
                left.push(c);
            } else if c.x0 >= mark.x1 {
                right.push(c);
            } else {
                overlap = true;
                break;
            }
        }
        if overlap || left.is_empty() || right.is_empty() {
            continue;
        }
        let body_top = left.iter().chain(&right).map(|c| c.y0).min().unwrap();
        let body_bottom = left.iter().chain(&right).map(|c| c.y1).max().unwrap();
        let bh = body_bottom - body_top;
        let mw = mark.x1 - mark.x0;
        let mh = mark.y1 - mark.y0;
        if bh == 0
            || mw < bh.div_ceil(4)
            || mw as u128 * 2 > bh as u128
            || mh as u128 * 2 < bh as u128
            || mh as u128 * 2 > bh as u128 * 3
            || mark.area == mw * mh
        {
            continue;
        }
        let Some(l) = group_view(&left, bh, (bh / 12).max(1)) else {
            continue;
        };
        let Some(r) = group_view(&right, bh, (bh / 12).max(1)) else {
            continue;
        };
        if mark.x0 - l.x1 > (bh / 4).max(1) || r.x0 - mark.x1 > (bh / 4).max(1) {
            continue;
        }
        let edge_part = |parts: &[&Component]| {
            primary_part(parts)
                .is_some_and(|p| parts.iter().any(|c| edge_shared_large_child(p, c, bh)))
        };
        if !edge_part(&left) && !edge_part(&right) {
            continue;
        }
        let view = [
            l,
            Component {
                x0: mark.x0,
                x1: mark.x1,
                y0: mark.y0,
                y1: mark.y1,
                area: mark.area,
            },
            r,
        ];
        if supports_collected(&view, h, 3, Some(text)) {
            supported += 1;
        }
    }
    supported == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> (Vec<u8>, usize, usize) {
        let (w, h) = (34, 16);
        let mut rgb = vec![255; w * h * 3];
        let mut fill = |x0: usize, y0: usize, x1: usize, y1: usize| {
            for y in y0..y1 {
                for x in x0..x1 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        };
        // A tall connected corner and a large disconnected child sharing only its bottom.
        fill(1, 1, 13, 2);
        fill(1, 1, 2, 13);
        fill(12, 1, 13, 4);
        fill(5, 6, 11, 13);
        // The internal mark has a 4x12 bounding box, without a filled rectangle.
        fill(14, 1, 18, 2);
        fill(17, 1, 18, 13);
        fill(14, 12, 18, 13);
        fill(19, 1, 31, 2);
        fill(19, 12, 31, 13);
        fill(19, 1, 20, 13);
        fill(30, 1, 31, 13);
        (rgb, w, h)
    }

    #[test]
    fn a_middle_closer_is_supported_by_its_two_owned_body_groups() {
        let (rgb, w, h) = image();
        let before = rgb.clone();
        assert!(supports("情)報", &rgb, w, h));
        assert_eq!(rgb, before);
    }
    #[test]
    fn the_known_model_correct_internal_closer_has_owned_detached_parts() {
        let im = crate::io::load_rgb_u8(std::path::Path::new(
            "tests/fixtures/internal_detached_closer/info_internal40.png",
        ))
        .unwrap();
        assert_eq!(
            sanitize_visible_single_line(
                "情)報",
                &im.data,
                im.width,
                im.height,
                ParseqOrientation::Auto
            ),
            "情)報"
        );
        let before = im.data.clone();
        assert!(supports("情)報", &im.data, im.width, im.height));
        assert_eq!(im.data, before);
    }

    #[test]
    fn text_shapes_unowned_noise_and_filled_marks_are_rejected() {
        let (rgb, w, h) = image();
        for text in [
            "情報", "情報)", ")情報", "情))報", "情 )報", "情) 報", "A)報", "情)A", "情)\n",
        ] {
            assert!(!supports(text, &rgb, w, h), "{text:?}");
        }
        assert!(!supports("情)報", &[], w, h));
        assert!(!supports("情)報", &rgb, usize::MAX, h));
        assert!(!supports("情)報", &vec![255; rgb.len()], w, h));
        let mut noisy = rgb.clone();
        noisy[(6 * w + 25) * 3..(6 * w + 26) * 3].fill(0);
        assert!(
            !supports("情)報", &noisy, w, h),
            "ignored one-pixel noise inside the body bbox"
        );
        let mut filled = rgb.clone();
        for y in 1..13 {
            filled[(y * w + 14) * 3..(y * w + 18) * 3].fill(0);
        }
        assert!(!supports("情)報", &filled, w, h));
    }

    #[test]
    fn a_quarter_body_height_gap_is_inclusive_and_the_next_pixel_is_rejected() {
        let (rgb, w, h) = image();
        for (shift, expected) in [(2, true), (3, false)] {
            let nw = w + shift;
            let mut moved = vec![255; nw * h * 3];
            for y in 0..h {
                moved[y * nw * 3..(y * nw + 19) * 3]
                    .copy_from_slice(&rgb[y * w * 3..(y * w + 19) * 3]);
                moved[(y * nw + 19 + shift) * 3..(y * nw + w + shift) * 3]
                    .copy_from_slice(&rgb[(y * w + 19) * 3..(y + 1) * w * 3]);
            }
            assert_eq!(supports("情)報", &moved, nw, h), expected, "shift {shift}");
        }
    }

    #[test]
    fn two_independently_supported_middle_marks_are_ambiguous() {
        let (rgb, w, h) = image();
        let nw = w + 5;
        let mut ambiguous = vec![255; nw * h * 3];
        for y in 0..h {
            ambiguous[y * nw * 3..(y * nw + 19) * 3]
                .copy_from_slice(&rgb[y * w * 3..(y * w + 19) * 3]);
            ambiguous[(y * nw + 19) * 3..(y * nw + 23) * 3]
                .copy_from_slice(&rgb[(y * w + 14) * 3..(y * w + 18) * 3]);
            ambiguous[(y * nw + 24) * 3..(y * nw + w + 5) * 3]
                .copy_from_slice(&rgb[(y * w + 19) * 3..(y + 1) * w * 3]);
        }
        assert!(!supports("情)報", &ambiguous, nw, h));
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn support_survives_paper_translation_gray_and_integer_blocks(
            mx in 0usize..8, my in 0usize..8, tone in 0u8..220, scale in 1usize..4,
        ) {
            let (original,ow,oh)=image();let(w,h)=(ow*scale+2*mx,oh*scale+2*my);let mut rgb=vec![255;w*h*3];
            for y in 0..oh {for x in 0..ow {
                if original[(y*ow+x)*3]==255 {continue;}
                for yy in y*scale+my..(y+1)*scale+my {for xx in x*scale+mx..(x+1)*scale+mx {rgb[(yy*w+xx)*3..(yy*w+xx+1)*3].fill(tone);}}
            }}
            let before=rgb.clone();proptest::prop_assert!(supports("情)報",&rgb,w,h));proptest::prop_assert_eq!(rgb,before);
        }
    }

    #[test]
    #[ignore = "diagnostic; requires NDLOCR_INTERNAL_DETACHED_ROOT with frozen baseline pairs"]
    fn record_internal_detached_policy() {
        let Ok(root) = std::env::var("NDLOCR_INTERNAL_DETACHED_ROOT") else {
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
    fn verify_fresh_internal_auto_tensors() {
        use crate::infer::parseq::preprocess_rgb_u8_with_orientation;
        let Ok(root) = std::env::var("NDLOCR_INTERNAL_DETACHED_ROOT") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let m: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("fresh/evaluation-manifest.json")).unwrap(),
        )
        .unwrap();
        let rows = m["inputs"].as_array().unwrap();
        assert_eq!(rows.len(), 5088);
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
        std::fs::write(root.join("fresh/tensor-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"images":5088,"model_shapes":3,"all_original_auto_and_effective_horizontal_tensors_bit_equal":true})).unwrap()).unwrap();
    }
    #[test]
    #[ignore = "diagnostic; requires NDLOCR_INTERNAL_DETACHED_PRODUCT_MANIFEST with frozen candidate strings"]
    fn verify_product_matches_frozen_internal_policy() {
        let Ok(path) = std::env::var("NDLOCR_INTERNAL_DETACHED_PRODUCT_MANIFEST") else {
            return;
        };
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let cases = m["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 115023);
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
