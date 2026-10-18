use super::*;
use crate::infer::parseq::{ParseqOrientation, sanitize_recognized_fragment};
use proptest::prelude::*;
fn paint(rgb: &mut [u8], w: usize, x0: usize, y0: usize, cw: usize, ch: usize, tone: u8) {
    for y in y0..y0 + ch {
        for x in x0..x0 + cw {
            if x < x0 + 2 || y < y0 + 2 || y >= y0 + ch - 2 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(tone);
            }
        }
    }
}
fn row(n: usize) -> (Vec<u8>, usize, usize) {
    let (w, h) = (n * 8 + 4, 20);
    let mut rgb = vec![255; w * h * 3];
    for i in 0..n {
        paint(&mut rgb, w, 2 + i * 8, 3, 4, 12, 0);
    }
    (rgb, w, h)
}
#[test]
fn visible_glyph_count_keeps_actual_repetitions_of_any_character_kind() {
    let (rgb, w, h) = row(6);
    let original = rgb.clone();
    for raw in [
        "111111",
        "000000",
        "OOOOOO",
        "IIIIII",
        "一一一一一一",
        "A1A1A1",
        "😀😀😀😀😀😀",
        " 000000\t",
    ] {
        let expected = raw.trim();
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            expected,
            "{raw}"
        );
    }
    assert_eq!(rgb, original);
}
#[test]
fn disagreement_noise_connected_ink_and_two_rows_keep_existing_policy() {
    let raw = "111111";
    let expected = sanitize_recognized_fragment(raw);
    for n in [0, 1, 4, 5, 7, 8, 12] {
        let (rgb, w, h) = row(n);
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            expected
        );
    }
    let (mut rgb, w, h) = row(6);
    for x in 2..46 {
        rgb[(8 * w + x) * 3..(8 * w + x + 1) * 3].fill(0);
    }
    assert_eq!(
        sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
        expected
    );
    let (w, h) = (60, 42);
    let mut two = vec![255; w * h * 3];
    for i in 0..3 {
        paint(&mut two, w, 2 + i * 8, 2, 4, 12, 0);
        paint(&mut two, w, 30 + i * 8, 25, 4, 12, 0);
    }
    assert_eq!(
        sanitize_visible_repetitions(raw, &two, w, h, ParseqOrientation::Horizontal),
        expected
    );
    let mut noise = vec![255; 60 * 20 * 3];
    for x in [2, 10, 18, 26, 34, 42] {
        noise[(8 * 60 + x) * 3..(8 * 60 + x + 1) * 3].fill(0);
    }
    assert_eq!(
        sanitize_visible_repetitions(raw, &noise, 60, 20, ParseqOrientation::Horizontal),
        expected
    );
}
#[test]
fn meaningful_extra_ink_and_word_spaces_cannot_support_repetition_restoration() {
    let (mut rgb, w, h) = row(6);
    for x in 1..w - 1 {
        rgb[(18 * w + x) * 3..(18 * w + x + 1) * 3].fill(0);
    }
    let raw = "000000";
    assert_eq!(
        sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
        sanitize_recognized_fragment(raw)
    );
    let (rgb, w, h) = row(12);
    let raw = "000000 000000";
    assert_eq!(
        sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
        sanitize_recognized_fragment(raw)
    );
}
#[test]
fn invalid_rgb_and_automatic_rotation_keep_the_existing_text_policy() {
    let (rgb, w, h) = row(6);
    let raw = "000000";
    for (data, iw, ih) in [
        (&rgb[..rgb.len() - 1], w, h),
        (rgb.as_slice(), usize::MAX, h),
        (rgb.as_slice(), w, 0),
        (&[][..], 0, 0),
    ] {
        assert_eq!(
            sanitize_visible_repetitions(raw, data, iw, ih, ParseqOrientation::Horizontal),
            sanitize_recognized_fragment(raw)
        );
    }
    for width in [24, 25] {
        let mut rgb = vec![255; width * 20 * 3];
        for i in 0..6 {
            paint(&mut rgb, width, 1 + i * 4, 4, 2, 12, 0);
        }
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, width, 20, ParseqOrientation::Auto),
            if width == 25 {
                raw.to_string()
            } else {
                sanitize_recognized_fragment(raw)
            }
        );
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, width, 20, ParseqOrientation::Horizontal),
            raw
        );
    }
}
#[test]
fn common_vertical_span_and_width_limits_have_closed_boundaries() {
    for dy in [3, 4] {
        let (w, h) = (60, 22);
        let mut rgb = vec![255; w * h * 3];
        for i in 0..6 {
            paint(
                &mut rgb,
                w,
                2 + i * 8,
                3 + if i == 5 { dy } else { 0 },
                4,
                12,
                0,
            );
        }
        let raw = "000000";
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            if dy == 3 {
                raw.into()
            } else {
                sanitize_recognized_fragment(raw)
            }
        );
    }
    for cw in [24, 25] {
        let (w, h) = (200, 20);
        let mut rgb = vec![255; w * h * 3];
        for i in 0..6 {
            paint(
                &mut rgb,
                w,
                2 + i * 30,
                3,
                if i == 0 { cw } else { 4 },
                12,
                0,
            );
        }
        let raw = "000000";
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            if cw == 24 {
                raw.into()
            } else {
                sanitize_recognized_fragment(raw)
            }
        );
    }
}
#[test]
fn existing_whitespace_brackets_and_unchanged_text_rules_remain_in_effect() {
    let (rgb, w, h) = row(7);
    assert_eq!(
        sanitize_visible_repetitions("111111)", &rgb, w, h, ParseqOrientation::Horizontal),
        "111111)"
    );
    for raw in [
        "(balanced)",
        "\t hello   world\n",
        "ordinary).",
        "aという。という。という。b",
    ] {
        assert_eq!(
            sanitize_visible_repetitions(raw, &[], 0, 0, ParseqOrientation::Horizontal),
            sanitize_recognized_fragment(raw)
        );
    }
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(48))]
 #[test]
 fn restoration_preserves_raw_repetitions_under_scale_translation_and_tone(n in 5usize..14,scale in 1usize..4,dx in 0usize..13,dy in 0usize..13,tone in 0u8..220){
  let(base,bw,bh)=row(n);let(w,h)=(bw*scale+dx+5,bh*scale+dy+5);let mut rgb=vec![255;w*h*3];for y in 0..bh*scale{for x in 0..bw*scale{if base[((y/scale)*bw+x/scale)*3]==0{rgb[((y+dy)*w+x+dx)*3..((y+dy)*w+x+dx+1)*3].fill(tone);}}}let original=rgb.clone();let raw="0".repeat(n);prop_assert_eq!(sanitize_visible_repetitions(&raw,&rgb,w,h,ParseqOrientation::Horizontal),raw);prop_assert_eq!(rgb,original);
 }
}

#[test]
#[ignore = "diagnostic; replays frozen raw outputs with explicit input/output paths"]
fn replay_visible_repetition_policy_on_frozen_raw_outputs() {
    // This replay consumes an explicitly supplied diagnostic, not a CI fixture.
    let Ok(input) = std::env::var("NDLOCR_REPEAT_RAW_PROBE") else {
        return;
    };
    let output = std::env::var("NDLOCR_REPEAT_POLICY_OUTPUT").expect("policy output path");
    let mut data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(input).unwrap()).unwrap();
    let mut count = 0;
    for model in data["models"].as_array_mut().unwrap() {
        for sample in model["images"].as_array_mut().unwrap() {
            for crop in sample["inputs"].as_array_mut().unwrap() {
                let image =
                    crate::io::load_rgb_u8(std::path::Path::new(crop["png"].as_str().unwrap()))
                        .unwrap();
                let raw = crop["raw_text"].as_str().unwrap();
                let candidate = sanitize_fragment_preserving_repetitions(raw);
                let visible = sanitize_visible_repetitions(
                    raw,
                    &image.data,
                    image.width,
                    image.height,
                    ParseqOrientation::Horizontal,
                );
                let supported = supports_glyph_count(
                    &image.data,
                    image.width,
                    image.height,
                    candidate.chars().count(),
                );
                crop["visible_text"] = visible.into();
                crop["without_repetition_limits"] = candidate.into();
                crop["image_supports_candidate_count"] = supported.into();
                count += 1;
            }
        }
    }
    assert!(count >= 924);
    std::fs::write(output, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
}

#[test]
fn standalone_bracket_policy_checks_all_visible_glyphs_before_removing_closers() {
    for raw in ["111111)", ")111111"] {
        let (rgb, w, h) = row(7);
        assert_eq!(
            sanitize_visible_single_line(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            "111111"
        );
        assert_eq!(sanitize_recognized_text(raw), "1111");
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            raw
        );
    }
    let (rgb, w, h) = row(8);
    assert_eq!(
        sanitize_visible_single_line("(000000)", &rgb, w, h, ParseqOrientation::Horizontal),
        "(000000)"
    );
}
#[test]
fn minimum_component_area_and_luminance_threshold_are_exact() {
    for area in [2, 3] {
        let (w, h) = (40, 8);
        let mut rgb = vec![255; w * h * 3];
        for i in 0..6 {
            for y in 1..1 + area {
                rgb[(y * w + 2 + i * 6) * 3..(y * w + 3 + i * 6) * 3].fill(0);
            }
        }
        let raw = "IIIIII";
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            if area == 3 {
                raw.into()
            } else {
                sanitize_recognized_fragment(raw)
            }
        );
    }
    for tone in [219, 220] {
        let (mut rgb, w, h) = row(6);
        for p in rgb.chunks_exact_mut(3) {
            if p[0] == 0 {
                p.fill(tone);
            }
        }
        let raw = "000000";
        assert_eq!(
            sanitize_visible_repetitions(raw, &rgb, w, h, ParseqOrientation::Horizontal),
            if tone == 219 {
                raw.into()
            } else {
                sanitize_recognized_fragment(raw)
            }
        );
    }
}
