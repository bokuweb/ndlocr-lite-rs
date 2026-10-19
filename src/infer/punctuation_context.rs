//! 句読点の字形余白を、同じモデルの独立した純白余白候補で確認する。
use super::parseq::{ParseqOrientation, RecognizeResult};
use anyhow::Result;

#[cfg(test)]
#[path = "../../tests/support/punctuation_space_production_replay.rs"]
mod production_replay_tests;

#[allow(clippy::too_many_arguments)]
pub(super) fn select(
    raw: &RecognizeResult,
    rgb: &[u8],
    width: usize,
    height: usize,
    model: (usize, usize),
    orientation: ParseqOrientation,
    mut recognize: impl FnMut(&[u8], usize, usize) -> Result<RecognizeResult>,
) -> Option<RecognizeResult> {
    let (left, right) = raw.text.split_once(' ')?;
    let japanese_body = |c| {
        matches!(c, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{4dbf}'
        | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
    };
    let count = left.chars().count() + right.chars().count();
    if !(6..=12).contains(&count)
        || !matches!(left.chars().next_back(), Some('、' | '。'))
        || !right.chars().next().is_some_and(japanese_body)
        || !left
            .chars()
            .chain(right.chars())
            .all(|c| japanese_body(c) || matches!(c, '、' | '。'))
        || !raw.mean_confidence.is_finite()
        || !(0.0..=1.0).contains(&raw.mean_confidence)
        || width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
        || model.0 == 0
        || model.1 == 0
        || (orientation == ParseqOrientation::Auto && height as u128 * 5 > width as u128 * 4)
    {
        return None;
    }
    let padded_width = width.checked_add(4)?;
    if padded_width as u128 * model.1 as u128 > height as u128 * model.0 as u128
        || !super::visible_repetitions::supports_punctuation_space_context(
            rgb, width, height, count,
        )
    {
        return None;
    }
    let expected = format!("{left}{right}");
    let mut first = None;
    for yp in 0..3 {
        let padded_height = height.checked_add(2 * yp)?;
        let len = padded_width.checked_mul(padded_height)?.checked_mul(3)?;
        let mut padded = Vec::new();
        padded.try_reserve_exact(len).ok()?;
        padded.resize(len, 255);
        for y in 0..height {
            let start = ((y + yp) * padded_width + 2) * 3;
            padded[start..start + width * 3]
                .copy_from_slice(&rgb[y * width * 3..(y + 1) * width * 3]);
        }
        let candidate = recognize(&padded, padded_width, padded_height).ok()?;
        if candidate.text != expected
            || !candidate.mean_confidence.is_finite()
            || !(0.98..=1.0).contains(&candidate.mean_confidence)
            || (yp == 0 && candidate.mean_confidence <= raw.mean_confidence)
        {
            return None;
        }
        if yp == 0 {
            first = Some(candidate);
        }
    }
    first
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(text: &str, mean_confidence: f32) -> RecognizeResult {
        RecognizeResult {
            text: text.into(),
            mean_confidence,
        }
    }

    fn image() -> (Vec<u8>, usize, usize) {
        // 複数文字が連結した左右の語ブロックと、その間の狭い句読点余白。
        let (w, h) = (59, 20);
        let mut rgb = vec![255; w * h * 3];
        for y in 4..16 {
            for x in (4..28).chain(31..55) {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
        (rgb, w, h)
    }

    #[test]
    fn returns_the_first_real_candidate_after_three_pixel_preserving_confirmations() {
        let (rgb, w, h) = image();
        let before = rgb.clone();
        let raw = result("資料確認、 次回連絡", 0.99);
        let mut calls = 0;
        let chosen = select(
            &raw,
            &rgb,
            w,
            h,
            (256, 24),
            ParseqOrientation::Auto,
            |p, pw, ph| {
                let yp = calls;
                assert_eq!((pw, ph), (w + 4, h + 2 * yp));
                for y in 0..ph {
                    for x in 0..pw {
                        let pixel = &p[(y * pw + x) * 3..(y * pw + x + 1) * 3];
                        if (2..w + 2).contains(&x) && (yp..h + yp).contains(&y) {
                            assert_eq!(
                                pixel,
                                &rgb[((y - yp) * w + x - 2) * 3..((y - yp) * w + x - 1) * 3]
                            );
                        } else {
                            assert_eq!(pixel, [255; 3]);
                        }
                    }
                }
                calls += 1;
                Ok(result(
                    "資料確認、次回連絡",
                    if calls == 1 { 0.999 } else { 0.98 },
                ))
            },
        )
        .expect("three agreeing model results must be eligible");
        assert_eq!(calls, 3);
        assert_eq!(chosen.text, "資料確認、次回連絡");
        assert_eq!(chosen.mean_confidence, 0.999);
        assert_eq!(rgb, before);
        assert_eq!(raw.text, "資料確認、 次回連絡");
        assert_eq!(raw.mean_confidence, 0.99);
    }

    #[test]
    fn real_word_spaces_other_boundaries_and_invalid_original_scores_never_retry() {
        let (rgb, w, h) = image();
        for text in [
            "資料確認 次回連絡",
            "資料確認 、次回連絡",
            "資料確認、  次回連絡",
            " 資料確認、次回連絡",
            "資料確認、次回連絡 ",
            "資料、 次",
            "資料確認、 A次回連絡",
            "資料確認、 。次回連絡",
            "資料確認確認確認確認、 次回連絡",
        ] {
            assert!(
                select(
                    &result(text, 0.99),
                    &rgb,
                    w,
                    h,
                    (256, 24),
                    ParseqOrientation::Auto,
                    |_, _, _| panic!("ineligible text: {text}")
                )
                .is_none()
            );
        }
        for score in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01] {
            assert!(
                select(
                    &result("資料確認、 次回連絡", score),
                    &rgb,
                    w,
                    h,
                    (256, 24),
                    ParseqOrientation::Auto,
                    |_, _, _| panic!("invalid original confidence")
                )
                .is_none()
            );
        }
    }

    #[test]
    fn invalid_geometry_rotation_and_over_capacity_never_retry() {
        let (rgb, w, h) = image();
        let raw = result("資料確認、 次回連絡", 0.99);
        let white = vec![255; rgb.len()];
        let black = vec![0; rgb.len()];
        for (p, pw, ph, model) in [
            (&rgb[..5], w, h, (256, 24)),
            (&rgb[..], usize::MAX, h, (256, 24)),
            (&white[..], w, h, (256, 24)),
            (&black[..], w, h, (256, 24)),
            (&rgb[..], w, h, (35, 24)),
            (&rgb[..], w, h, (0, 24)),
            (&rgb[..], w, h, (256, 0)),
            (&[][..], 0, 0, (256, 24)),
        ] {
            assert!(
                select(
                    &raw,
                    p,
                    pw,
                    ph,
                    model,
                    ParseqOrientation::Auto,
                    |_, _, _| panic!("ineligible geometry")
                )
                .is_none()
            );
        }
        let mut tall = vec![255; 80 * 65 * 3];
        for y in 0..h {
            tall[y * 80 * 3..(y * 80 + w) * 3].copy_from_slice(&rgb[y * w * 3..(y + 1) * w * 3]);
        }
        assert!(
            select(
                &raw,
                &tall,
                80,
                65,
                (256, 24),
                ParseqOrientation::Auto,
                |_, _, _| panic!("would rotate")
            )
            .is_none()
        );
        let mut calls = 0;
        assert!(
            select(
                &raw,
                &tall,
                80,
                65,
                (256, 24),
                ParseqOrientation::Horizontal,
                |_, _, _| {
                    calls += 1;
                    Ok(result("資料確認、次回連絡", 0.999))
                }
            )
            .is_some()
        );
        assert_eq!(calls, 3);
        assert!(
            select(
                &raw,
                &rgb,
                w,
                h,
                (63, 20),
                ParseqOrientation::Auto,
                |_, _, _| Ok(result("資料確認、次回連絡", 0.999))
            )
            .is_some()
        );
        assert!(
            select(
                &raw,
                &tall[..80 * 64 * 3],
                80,
                64,
                (256, 24),
                ParseqOrientation::Auto,
                |_, _, _| Ok(result("資料確認、次回連絡", 0.999))
            )
            .is_some()
        );
    }

    #[test]
    fn full_word_gap_in_narrow_components_does_not_trigger_a_model_retry() {
        let (w, h) = (26, 20);
        let mut rgb = vec![255; w * h * 3];
        for y in 4..16 {
            for x in (4..10).chain(16..22) {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
        assert!(
            select(
                &result("資料確認、 次回連絡", 0.99),
                &rgb,
                w,
                h,
                (256, 24),
                ParseqOrientation::Auto,
                |_, _, _| panic!(
                    "a gap as wide as the whole estimated character pitch must be kept"
                )
            )
            .is_none()
        );
    }

    #[test]
    fn each_failed_confirmation_preserves_original_without_further_calls() {
        let (rgb, w, h) = image();
        let raw = result("資料確認、 次回連絡", 0.99);
        for at in 0..3 {
            for kind in 0..7 {
                let mut calls = 0;
                let chosen = select(
                    &raw,
                    &rgb,
                    w,
                    h,
                    (256, 24),
                    ParseqOrientation::Auto,
                    |_, _, _| {
                        let i = calls;
                        calls += 1;
                        if i != at {
                            return Ok(result("資料確認、次回連絡", 0.999));
                        }
                        match kind {
                            0 => Err(anyhow::anyhow!("forward failed")),
                            1 => Ok(result("別の資料", 0.999)),
                            2 => Ok(result("資料確認、次回連絡", 0.97999)),
                            3 => Ok(result("資料確認、次回連絡", f32::NAN)),
                            4 => Ok(result("資料確認、次回連絡", f32::INFINITY)),
                            5 => Ok(result("資料確認、次回連絡", 1.0001)),
                            _ => Ok(result("資料確認、 次回連絡", 0.999)),
                        }
                    },
                );
                assert!(chosen.is_none(), "at {at}, kind {kind}");
                assert_eq!(calls, at + 1);
            }
        }
        let mut calls = 0;
        assert!(
            select(
                &raw,
                &rgb,
                w,
                h,
                (256, 24),
                ParseqOrientation::Auto,
                |_, _, _| {
                    calls += 1;
                    Ok(result("資料確認、次回連絡", 0.99))
                }
            )
            .is_none()
        );
        assert_eq!(calls, 1);
        assert_eq!(raw.text, "資料確認、 次回連絡");
        assert_eq!(raw.mean_confidence, 0.99);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn finite_confidence_selection_obeys_closed_threshold_and_strict_gain(
            base in -0.05f32..1.05,candidate in 0.95f32..1.05
        ) {
            let (rgb,w,h)=image();let mut calls=0;
            let selected=select(&result("資料確認、 次回連絡",base),&rgb,w,h,(256,24),ParseqOrientation::Auto,|_,_,_|{
                calls+=1;Ok(result("資料確認、次回連絡",candidate))
            });
            let valid_base=(0.0..=1.0).contains(&base);
            let accepted=valid_base&&(0.98..=1.0).contains(&candidate)&&candidate>base;
            proptest::prop_assert_eq!(selected.is_some(),accepted);
            proptest::prop_assert_eq!(calls,if !valid_base {0}else if accepted{3}else{1});
        }
    }
}
