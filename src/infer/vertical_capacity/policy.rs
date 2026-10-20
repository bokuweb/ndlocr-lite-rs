//! Frozen support and output guards for the test-only vertical recovery proposal.
use super::PreparedColumn;
use crate::infer::parseq::RecognizeResult;
use crate::pipeline::line_chunks::LineChunk;

struct Plan {
    column: PreparedColumn,
    chunks: Vec<LineChunk>,
    groups: usize,
}

fn plan(
    count: usize,
    rgb: &[u8],
    width: usize,
    height: usize,
    (iw, ih): (usize, usize),
) -> Option<Plan> {
    if count > 16
        || width < 12
        || iw == 0
        || ih == 0
        || iw.checked_mul(ih).is_none()
        || width.checked_mul(height)?.checked_mul(3)? != rgb.len()
        || height as u128 * 5 <= width as u128 * 4
        || height as u128 * ih as u128 <= (width as u128 * iw as u128).checked_mul(2)?
    {
        return None;
    }
    let (column, _) = super::prepare_otsu(rgb, width, height)?;
    let (w, h) = (column.width, column.height);
    let (mut columns, mut groups, mut top, mut bottom) = (0usize, 0usize, h, 0usize);
    let mut group: Option<(usize, usize, usize)> = None;
    for x in 0..=w {
        let (mut first, mut last) = (h, 0usize);
        if x < w {
            for y in 0..h {
                let p = (y * w + x) * 3;
                if column.support[p..p + 3]
                    .iter()
                    .map(|&v| u16::from(v))
                    .sum::<u16>()
                    < 660
                {
                    first = first.min(y);
                    last = y + 1;
                }
            }
        }
        if last > 0 {
            columns += 1;
            top = top.min(first);
            bottom = bottom.max(last);
            let (start, a, b) = group.unwrap_or((x, h, 0));
            group = Some((start, a.min(first), b.max(last)));
        } else if let Some((start, a, b)) = group.take()
            && x - start <= h.saturating_mul(2)
            && b - a >= h.div_ceil(3)
        {
            groups += 1;
        }
    }
    if columns < w.div_ceil(3)
        || bottom.saturating_sub(top) < h.div_ceil(2)
        || groups < 40
        || groups < count * 4
    {
        return None;
    }
    let chunks = crate::pipeline::line_chunks::split_rotated_vertical_column_for_input(
        &column.support,
        w,
        h,
        220,
        iw,
        ih,
    )
    .ok()?;
    if chunks.len() < 2 {
        return None;
    }
    Some(Plan {
        column,
        chunks,
        groups,
    })
}

fn combine(base: &str, groups: usize, parts: &[RecognizeResult]) -> Option<RecognizeResult> {
    if parts.len() < 2 {
        return None;
    }
    let (mut text, mut emitted, mut weighted) = (String::new(), 0usize, 0.0f64);
    for part in parts {
        if part.text.trim().is_empty() || !(0.9..=1.0).contains(&part.mean_confidence) {
            return None;
        }
        let count = part.text.chars().count();
        emitted = emitted.checked_add(count)?;
        weighted += f64::from(part.mean_confidence) * count as f64;
        text.push_str(&part.text);
    }
    if emitted < base.chars().count().checked_mul(2)?.max(groups.div_ceil(2))
        || text
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            < 4
        || text
            .chars()
            .filter(|c| c.is_whitespace())
            .collect::<Vec<_>>()
            != base
                .chars()
                .filter(|c| c.is_whitespace())
                .collect::<Vec<_>>()
    {
        return None;
    }
    Some(RecognizeResult {
        text,
        mean_confidence: (weighted / emitted as f64) as f32,
    })
}

pub(super) fn recover<F>(
    base: &RecognizeResult,
    rgb: &[u8],
    (width, height): (usize, usize),
    input: (usize, usize),
    mut recognize: F,
) -> Option<RecognizeResult>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    let plan = plan(base.text.chars().count(), rgb, width, height, input)?;
    let mut parts = Vec::new();
    parts.try_reserve_exact(plan.chunks.len()).ok()?;
    for (index, _) in plan.chunks.iter().enumerate() {
        let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
            &plan.column.original,
            plan.column.width,
            plan.column.height,
            &plan.chunks,
            index,
        )
        .ok()?;
        parts.push(recognize(&crop.data, crop.width, crop.height).ok()?);
    }
    combine(&base.text, plan.groups, &parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn recovery_reads_original_chunks_even_when_the_old_confidence_is_high() {
        let (rgb, w, h) = column(43, 14);
        let p = plan(2, &rgb, w, h, (256, 24)).unwrap();
        let mut calls = 0;
        let base = RecognizeResult {
            text: "AB".into(),
            mean_confidence: 1.0,
        };
        let result = recover(&base, &rgb, (w, h), (256, 24), |data, cw, ch| {
            let chunk = &p.chunks[calls];
            assert_eq!((cw, ch), (chunk.x1 - chunk.x0, w));
            for y in 0..ch {
                for x in 0..cw {
                    let a = (y * cw + x) * 3;
                    let b = ((chunk.x0 + x) * w + (w - 1 - y)) * 3;
                    assert_eq!(&data[a..a + 3], &rgb[b..b + 3]);
                }
            }
            calls += 1;
            Ok(RecognizeResult {
                text: "ABCDEFGHIJKL".into(),
                mean_confidence: 0.95,
            })
        })
        .unwrap();
        assert_eq!(calls, p.chunks.len());
        assert_eq!(result.text, "ABCDEFGHIJKL".repeat(calls));
    }

    #[test]
    fn unsupported_geometry_never_calls_recognition_and_an_error_never_returns_partial_text() {
        let (rgb, w, h) = column(43, 14);
        let base = RecognizeResult {
            text: "A".repeat(17),
            mean_confidence: 1.0,
        };
        assert!(
            recover(&base, &rgb, (w, h), (256, 24), |_, _, _| panic!(
                "unsupported must not call NN"
            ))
            .is_none()
        );
        let base = RecognizeResult {
            text: "AB".into(),
            mean_confidence: 1.0,
        };
        assert!(
            recover(&base, &rgb, (w, h), (768, 24), |_, _, _| panic!(
                "unsupported must not call NN"
            ))
            .is_none()
        );
        let mut calls = 0;
        assert!(
            recover(&base, &rgb, (w, h), (256, 24), |_, _, _| {
                calls += 1;
                anyhow::bail!("inference failed")
            })
            .is_none()
        );
        assert_eq!(calls, 1);
    }

    #[test]
    fn raw_chunk_whitespace_is_not_trimmed_to_make_a_candidate_acceptable() {
        let (rgb, w, h) = column(43, 14);
        let base = RecognizeResult {
            text: "AB".into(),
            mean_confidence: 1.0,
        };
        let result = recover(&base, &rgb, (w, h), (256, 24), |_, _, _| {
            Ok(RecognizeResult {
                text: "ABCDEFGHIJKL ".into(),
                mean_confidence: 1.0,
            })
        });
        assert!(result.is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn every_requested_crop_retains_original_coordinates_and_colored_paper(
            glyphs in 40usize..76,
            red in 225u8..=255,
            green in 150u8..221,
            blue in 30u8..150,
        ) {
            let (mut rgb,w,h)=column(glyphs,14);
            for pixel in rgb.as_chunks_mut::<3>().0 {
                if *pixel==[240,220,170] { *pixel=[red,green,blue]; }
            }
            let p=plan(2,&rgb,w,h,(256,24)).unwrap();
            let mut calls=0;
            let result=recover(&RecognizeResult { text:"AB".into(),mean_confidence:1.0 },&rgb,(w,h),(256,24),|data,cw,ch| {
                let chunk=&p.chunks[calls];
                assert_eq!((cw,ch),(chunk.x1-chunk.x0,w));
                for y in 0..ch {
                    for x in 0..cw {
                        let a=(y*cw+x)*3;
                        let b=((chunk.x0+x)*w+(w-1-y))*3;
                        assert_eq!(&data[a..a+3],&rgb[b..b+3]);
                    }
                }
                calls+=1;
                Ok(RecognizeResult { text:"ABCDEFGHIJKL".into(),mean_confidence:0.95 })
            });
            prop_assert!(result.is_some());
            prop_assert_eq!(calls,p.chunks.len());
        }
    }

    fn reads(texts: &[(&str, f32)]) -> Vec<RecognizeResult> {
        texts
            .iter()
            .map(|(text, mean)| RecognizeResult {
                text: (*text).to_owned(),
                mean_confidence: *mean,
            })
            .collect()
    }

    #[test]
    fn complete_supported_parts_use_scalar_weighted_confidence_and_exact_text() {
        let parts = reads(&[("ABCDEFGHIJ", 0.9), ("甲乙丙丁戊己庚辛壬癸子丑", 1.0)]);
        let result = combine("AB", 43, &parts).unwrap();
        assert_eq!(result.text, "ABCDEFGHIJ甲乙丙丁戊己庚辛壬癸子丑");
        // Ten ASCII and twelve CJK scalars; UTF-8 byte weights and a simple average differ.
        assert_eq!(result.mean_confidence.to_bits(), 0.95454544_f32.to_bits());
        let parts = reads(&[("ABCDEFGHIJ　", 0.9), ("甲乙丙丁戊己庚辛壬癸子丑", 1.0)]);
        let result = combine("AB　", 43, &parts).unwrap();
        assert_eq!(result.text, "ABCDEFGHIJ　甲乙丙丁戊己庚辛壬癸子丑");
    }

    #[test]
    fn whitespace_addition_removal_type_change_or_reordering_rejects_the_candidate() {
        let plain = reads(&[("ABCDEFGHIJ", 1.0), ("甲乙丙丁戊己庚辛壬癸子丑", 1.0)]);
        let space = reads(&[("ABCDEFGHIJ ", 1.0), ("甲乙丙丁戊己庚辛壬癸子丑", 1.0)]);
        let reversed = reads(&[("ABCDEFGHIJ　", 1.0), ("甲乙丙丁戊己庚辛壬癸子丑 ", 1.0)]);
        assert!(combine("AB", 43, &space).is_none());
        assert!(combine("AB ", 43, &plain).is_none());
        assert!(combine("AB　", 43, &space).is_none());
        assert!(combine("AB 　", 43, &reversed).is_none());
    }

    #[test]
    fn weak_empty_short_or_repetitive_parts_do_not_replace_the_original() {
        let good = "甲乙丙丁戊己庚辛壬癸子丑";
        for confidence in [0.899, f32::NAN, f32::INFINITY, 1.001] {
            assert!(
                combine("AB", 43, &reads(&[("ABCDEFGHIJ", confidence), (good, 1.0)])).is_none()
            );
        }
        for text in ["", "　 ", "ABC", "AAAAAAAAAA"] {
            let second = if text == "AAAAAAAAAA" {
                "AAAAAAAAAAAA"
            } else {
                good
            };
            assert!(combine("AB", 43, &reads(&[(text, 1.0), (second, 1.0)])).is_none());
        }
        assert!(combine("AB", 43, &[]).is_none());
        assert!(combine("AB", 43, &reads(&[("ABCDEFGHIJKLMNOPQRSTUVWXYZ", 1.0)])).is_none());
        assert!(combine("AB", 45, &reads(&[("ABCDEFGHIJ", 1.0), (good, 1.0)])).is_none());
    }

    fn column(glyphs: usize, thickness: usize) -> (Vec<u8>, usize, usize) {
        let (w, h) = (24, glyphs * 22 + 4);
        let mut rgb: Vec<_> = (0..w * h).flat_map(|_| [240, 220, 170]).collect();
        for glyph in 0..glyphs {
            for y in 3 + glyph * 22..3 + glyph * 22 + thickness {
                for x in 5..18 {
                    rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(&[0; 3]);
                }
            }
        }
        (rgb, w, h)
    }

    #[test]
    fn a_supported_long_column_splits_original_rgb_without_word_spaces() {
        let (rgb, w, h) = column(43, 14);
        let before = rgb.clone();
        let p = plan(6, &rgb, w, h, (256, 24)).unwrap();
        assert_eq!(p.groups, 43);
        assert_eq!((p.column.width, p.column.height), (h, w));
        assert!(p.chunks.len() >= 2);
        assert!(p.chunks.iter().all(|c| !c.space_before));
        assert_eq!(&p.column.original[..3], &[240, 220, 170]);
        assert_eq!(&p.column.support[..3], &[255; 3]);
        assert_eq!(rgb, before);
    }

    #[test]
    fn capacity_is_strict_and_existing_complete_results_are_not_replanned() {
        let (rgb, w, h) = column(43, 14);
        assert!(plan(6, &rgb, w, h, (475, 24)).is_none());
        assert!(plan(6, &rgb, w, h, (474, 24)).is_some());
        assert!(plan(17, &rgb, w, h, (256, 24)).is_none());
        assert!(plan(6, &rgb, w, h, (768, 24)).is_none());
    }

    #[test]
    fn sparse_marks_insufficient_groups_and_invalid_inputs_are_not_supported() {
        let (rgb, w, h) = column(39, 14);
        assert!(plan(6, &rgb, w, h, (256, 24)).is_none());
        let (rgb, w, h) = column(43, 2);
        assert!(plan(6, &rgb, w, h, (256, 24)).is_none());
        assert!(plan(6, &rgb[..rgb.len() - 1], w, h, (256, 24)).is_none());
        assert!(plan(6, &[], usize::MAX, usize::MAX, (256, 24)).is_none());
        assert!(plan(6, &rgb, w, h, (0, 24)).is_none());
        assert!(plan(6, &rgb, w, h, (256, 0)).is_none());
        assert!(plan(6, &rgb, w, h, (usize::MAX, 2)).is_none());
    }
}
