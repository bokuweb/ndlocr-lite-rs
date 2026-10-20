//! Fixed-capacity vertical recovery preserving the entire original raw prefix.
use crate::infer::parseq::RecognizeResult;

#[cfg(test)]
#[path = "../../../tests/support/vertical_prefix_geometry.rs"]
mod capture;

struct Plans {
    column: super::PreparedColumn,
    routes: [Vec<crate::pipeline::line_chunks::LineChunk>; 2],
}

fn plans(
    base: &RecognizeResult,
    rgb: &[u8],
    (width, height): (usize, usize),
    (iw, ih): (usize, usize),
    limit: Option<usize>,
) -> Option<Plans> {
    let limit = limit?;
    if limit < 3
        || !(limit - 1..=limit).contains(&base.text.chars().count())
        || width < 12
        || iw == 0
        || ih == 0
        || base
            .text
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
        || iw.checked_mul(ih).is_none()
        || width.checked_mul(height)?.checked_mul(3)? != rgb.len()
        || height as u128 * 5 <= width as u128 * 4
        || height as u128 * ih as u128 <= (width as u128 * iw as u128).checked_mul(2)?
    {
        return None;
    }
    let (column, _) = super::prepare_otsu(rgb, width, height)?;
    let (w, h) = (column.width, column.height);
    let (mut columns, mut top, mut bottom) = (0usize, h, 0usize);
    for x in 0..w {
        let mut ink = false;
        for y in 0..h {
            let p = (y * w + x) * 3;
            if column.support[p..p + 3]
                .iter()
                .map(|&v| u16::from(v))
                .sum::<u16>()
                < 660
            {
                ink = true;
                top = top.min(y);
                bottom = bottom.max(y + 1);
            }
        }
        columns += usize::from(ink);
    }
    if columns < w.div_ceil(3) || bottom.saturating_sub(top) < h.div_ceil(2) {
        return None;
    }
    let split = |span_divisor| {
        crate::pipeline::line_chunks::split_supported_vertical_column_for_input(
            &column.support,
            w,
            h,
            220,
            iw,
            ih,
            span_divisor,
        )
        .ok()
    };
    let routes = [split(1)?, split(2)?];
    if routes.iter().any(|route| route.len() < 2)
        || routes[0]
            .iter()
            .map(|c| (c.x0, c.x1))
            .eq(routes[1].iter().map(|c| (c.x0, c.x1)))
    {
        return None;
    }
    Some(Plans { column, routes })
}

pub(super) fn recover<F>(
    base: &RecognizeResult,
    rgb: &[u8],
    dimensions: (usize, usize),
    input: (usize, usize),
    limit: Option<usize>,
    mut recognize: F,
) -> Option<RecognizeResult>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    let limit = limit?;
    let plan = plans(base, rgb, dimensions, input, Some(limit))?;
    let mut parts = [Vec::new(), Vec::new()];
    for (chunks, results) in plan.routes.iter().zip(&mut parts) {
        for index in 0..chunks.len() {
            let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                &plan.column.original,
                plan.column.width,
                plan.column.height,
                chunks,
                index,
            )
            .ok()?;
            results.push(recognize(&crop.data, crop.width, crop.height).ok()?);
        }
    }
    combine(&base.text, limit, &parts[0], &parts[1])
}

fn combine(
    base: &str,
    limit: usize,
    primary: &[RecognizeResult],
    secondary: &[RecognizeResult],
) -> Option<RecognizeResult> {
    let base_count = base.chars().count();
    if limit < 3
        || !(limit - 1..=limit).contains(&base_count)
        || base.chars().next_back().is_some_and(char::is_whitespace)
    {
        return None;
    }
    let join = |parts: &[RecognizeResult]| {
        if parts.len() < 2 {
            return None;
        }
        let (mut text, mut count, mut weighted) = (String::new(), 0usize, 0f64);
        for part in parts {
            let scalars = part.text.chars().count();
            if part.text.trim().is_empty()
                || scalars >= limit
                || !(0.9..=1.0).contains(&part.mean_confidence)
            {
                return None;
            }
            text.push_str(&part.text);
            count = count.checked_add(scalars)?;
            weighted += scalars as f64 * f64::from(part.mean_confidence);
        }
        Some((text, count, weighted))
    };
    let (text, count, weighted) = join(primary)?;
    let (other, _, _) = join(secondary)?;
    if text != other
        || !text.starts_with(base)
        || count <= base_count
        || text
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            < 4
        || !text
            .chars()
            .filter(|c| c.is_whitespace())
            .eq(base.chars().filter(|c| c.is_whitespace()))
    {
        return None;
    }
    Some(RecognizeResult {
        text,
        mean_confidence: (weighted / count as f64) as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn part(text: &str, mean_confidence: f32) -> RecognizeResult {
        RecognizeResult {
            text: text.into(),
            mean_confidence,
        }
    }

    fn column() -> (Vec<u8>, usize, usize) {
        column_groups(43)
    }

    fn column_groups(groups: usize) -> (Vec<u8>, usize, usize) {
        let (w, h) = (14, groups * 14 + 16);
        let mut rgb = vec![255; w * h * 3];
        for group in 0..groups {
            for y in 8 + group * 14..18 + group * 14 {
                for x in 2..12 {
                    let p = (y * w + x) * 3;
                    rgb[p..p + 3].copy_from_slice(&[group as u8, 40, 60]);
                }
            }
        }
        (rgb, w, h)
    }

    #[test]
    fn both_routes_crop_the_same_original_pixels_with_distinct_boundaries() {
        let (rgb, w, h) = column();
        let base = part("ABCDEF", 1.0);
        let plan = plans(&base, &rgb, (w, h), (256, 24), Some(7)).unwrap();
        assert_eq!(plan.routes.each_ref().map(Vec::len), [3, 5]);
        let texts = ["ABCD", "EFGH", "IJ", "AB", "CD", "EF", "GH", "IJ"];
        let expected: Vec<_> = plan
            .routes
            .iter()
            .flat_map(|chunks| chunks.iter())
            .collect();
        let mut calls = 0;
        let result = recover(&base, &rgb, (w, h), (256, 24), Some(7), |data, cw, ch| {
            let chunk = expected[calls];
            assert_eq!((cw, ch), (chunk.x1 - chunk.x0, w));
            for y in 0..ch {
                for x in 0..cw {
                    let from = ((chunk.x0 + x) * w + (w - 1 - y)) * 3;
                    let to = (y * cw + x) * 3;
                    assert_eq!(&data[to..to + 3], &rgb[from..from + 3]);
                }
            }
            let result = part(texts[calls], 0.95);
            calls += 1;
            Ok(result)
        })
        .unwrap();
        assert_eq!(calls, 8);
        assert_eq!(result.text, "ABCDEFGHIJ");
    }

    #[test]
    fn full_and_half_budgets_support_model100_and_odd_input_dimensions() {
        for (groups, input, limit) in [
            (107, (768, 24), 101),
            (107, (767, 25), 101),
            (43, (255, 23), 31),
            (43, (256, 23), 31),
            (43, (255, 24), 31),
        ] {
            let (rgb, w, h) = column_groups(groups);
            let base = part(&"A".repeat(limit - 1), 1.0);
            let plan = plans(&base, &rgb, (w, h), input, Some(limit)).unwrap();
            assert!(plan.routes.iter().all(|route| route.len() >= 2));
            assert_ne!(
                plan.routes[0]
                    .iter()
                    .map(|c| (c.x0, c.x1))
                    .collect::<Vec<_>>(),
                plan.routes[1]
                    .iter()
                    .map(|c| (c.x0, c.x1))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn trailing_unicode_whitespace_cannot_become_an_internal_boundary() {
        let (rgb, w, h) = column();
        for space in [' ', '\u{3000}', '\n', '\t', '\u{a0}'] {
            let base = format!("ABCDE{space}");
            assert!(
                recover(
                    &part(&base, 1.0),
                    &rgb,
                    (w, h),
                    (256, 24),
                    Some(7),
                    |_, _, _| panic!("trailing whitespace must skip before recognition")
                )
                .is_none()
            );
        }
    }

    #[test]
    fn agreement_rejects_a_trailing_base_space_even_when_both_routes_preserve_it() {
        for space in [' ', '\u{3000}', '\n', '\t', '\u{a0}'] {
            let base = format!("ABCDE{space}");
            let parts = [part("ABCD", 1.0), part(&format!("E{space}FGH"), 1.0)];
            assert!(combine(&base, 7, &parts, &parts).is_none());
        }
    }

    #[test]
    fn unsupported_capacity_and_geometry_never_call_recognition() {
        let (rgb, w, h) = column();
        for (text, limit, dimensions, input) in [
            ("ABCDEF", None, (w, h), (256, 24)),
            ("ABCDE", Some(7), (w, h), (256, 24)),
            ("ABCDEFGH", Some(7), (w, h), (256, 24)),
            ("ABCDEF", Some(7), (w, h), (0, 24)),
            ("ABCDEF", Some(7), (w - 1, h), (256, 24)),
            ("AB", Some(2), (w, h), (256, 24)),
        ] {
            assert!(
                recover(
                    &part(text, 1.0),
                    &rgb,
                    dimensions,
                    input,
                    limit,
                    |_, _, _| panic!("unsupported must not reach NN")
                )
                .is_none()
            );
        }
        assert!(
            recover(
                &part("ABCDEF", 1.0),
                &vec![255; rgb.len()],
                (w, h),
                (256, 24),
                Some(7),
                |_, _, _| panic!("white paper must not reach NN")
            )
            .is_none()
        );
        assert!(
            recover(
                &part("ABCDEF", 1.0),
                &vec![0; 14 * 40 * 3],
                (14, 40),
                (256, 24),
                Some(7),
                |_, _, _| panic!("short column must not reach NN")
            )
            .is_none()
        );
    }

    #[test]
    fn callback_failure_never_returns_a_partial_extension() {
        let (rgb, w, h) = column();
        let mut calls = 0;
        let actual = recover(
            &part("ABCDEF", 1.0),
            &rgb,
            (w, h),
            (256, 24),
            Some(7),
            |_, _, _| {
                calls += 1;
                if calls == 2 {
                    anyhow::bail!("diagnostic failure")
                }
                Ok(part("ABCD", 1.0))
            },
        );
        assert!(actual.is_none());
        assert_eq!(calls, 2);
    }

    #[test]
    fn agreement_extends_the_whole_prefix_and_weights_only_the_primary_partition() {
        let primary = [part("ABCD", 0.95), part("EFGH", 0.97), part("IJ", 0.91)];
        let secondary = [
            part("AB", 0.9),
            part("CD", 1.0),
            part("EF", 0.9),
            part("GH", 1.0),
            part("IJ", 1.0),
        ];
        let actual = combine("ABCDEF", 7, &primary, &secondary).unwrap();
        assert_eq!(actual.text, "ABCDEFGHIJ");
        let expected =
            ((f64::from(0.95_f32) * 4.0 + f64::from(0.97_f32) * 4.0 + f64::from(0.91_f32) * 2.0)
                / 10.0) as f32;
        assert_eq!(actual.mean_confidence.to_bits(), expected.to_bits());
    }

    #[test]
    fn agreement_cannot_change_prefix_whitespace_or_accept_saturated_and_bad_chunks() {
        let primary = [part("ABCD", 0.95), part("EFGH", 0.97), part("IJ", 0.91)];
        for secondary in [
            vec![part("ABCD", 1.0), part("EFGHIX", 1.0)],
            vec![part("ABCD", 1.0), part("EFGHIJ", f32::NAN)],
            vec![part("ABC", 1.0), part("DEFGHIJ", 1.0)],
            vec![part("ABCDEF", 1.0), part("GHIJ", 0.899)],
            vec![part("ABCD", 1.01), part("EFGHIJ", 1.0)],
        ] {
            assert!(combine("ABCDEF", 7, &primary, &secondary).is_none());
        }
        assert!(combine("ABCDEX", 7, &primary, &primary).is_none());
        let spaces = [part("ABCD", 1.0), part("EFG", 1.0), part("HIJ ", 1.0)];
        assert!(combine("ABCDEF", 7, &spaces, &spaces).is_none());
        let same = [part("ABC", 1.0), part("DEF", 1.0)];
        assert!(combine("ABCDEF", 7, &same, &same).is_none());
        let only_spaces = [part("ABCD", 1.0), part(" ", 1.0), part("EFGHIJ", 1.0)];
        assert!(combine("ABCDEF", 7, &only_spaces, &only_spaces).is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn prefix_and_unicode_whitespace_are_invariant(
            suffix in proptest::collection::vec(prop_oneof![Just('字'),Just('カ'),Just('。'),Just(' '),Just('\u{3000}')],1..5),
            confidence in 900u16..1001,
        ) {
            let suffix: String = suffix.into_iter().collect();
            let base = "ABCD日本語文";
            let tail = format!("日本語文{suffix}");
            let mean = confidence as f32 / 1000.0;
            let a = [part("ABCD",mean),part(&tail,mean)];
            let b = [part("AB",mean),part("CD",mean),part(&tail,mean)];
            let actual = combine(base,9,&a,&b);
            prop_assert_eq!(actual.is_some(), !suffix.chars().any(char::is_whitespace));
            if let Some(actual) = actual {
                prop_assert!(actual.text.starts_with(base));
                prop_assert_eq!(actual.text,format!("{base}{suffix}"));
            }
        }
    }
}
