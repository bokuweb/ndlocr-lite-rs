use super::*;
use proptest::prelude::*;

fn long_column() -> (Vec<u8>, usize, usize) {
    let (width, height) = (43 * 14 + 16, 14);
    let mut rgb = vec![255; width * height * 3];
    for group in 0..43 {
        for x in 8 + group * 14..18 + group * 14 {
            for y in 2..12 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    (rgb, width, height)
}

#[test]
fn new_support_does_not_replace_the_legacy_classifier() {
    let widths = [vec![20; 6], vec![5, 12]].concat();
    let extents = [vec![26; 6], vec![8, 24]].concat();
    let spaces = [vec![6; 5], vec![8, 3]].concat();
    assert_eq!(
        retain_glyph_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34).len(),
        7
    );
    assert_eq!(
        retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34).len(),
        6
    );
}

#[test]
fn full_and_half_spans_remain_different_for_wide_and_odd_model_shapes() {
    let (rgb, width, height) = long_column();
    for (iw, ih) in [(256, 24), (384, 24), (768, 24), (255, 23), (767, 25)] {
        let a =
            split_supported_vertical_column_for_input(&rgb, width, height, 220, iw, ih, 1).unwrap();
        let b =
            split_supported_vertical_column_for_input(&rgb, width, height, 220, iw, ih, 2).unwrap();
        assert_eq!((a.len(), b.len()), (3, 5));
        assert_ne!(
            a.iter().map(|c| (c.x0, c.x1)).collect::<Vec<_>>(),
            b.iter().map(|c| (c.x0, c.x1)).collect::<Vec<_>>()
        );
        for (c, budget) in a
            .iter()
            .map(|c| (c, 20 * height))
            .chain(b.iter().map(|c| (c, 10 * height)))
        {
            assert!(c.x1 - c.x0 <= budget);
            assert!(!c.space_before);
        }
    }
}

#[test]
fn supported_split_rejects_bad_shapes_and_divisors_without_touching_rgb() {
    let (rgb, width, height) = long_column();
    let original = rgb.clone();
    for (iw, ih, divisor) in [(0, 24, 1), (256, 0, 1), (256, 24, 0), (256, 24, 3)] {
        assert!(
            split_supported_vertical_column_for_input(&rgb, width, height, 220, iw, ih, divisor)
                .is_err()
        );
    }
    assert!(
        split_supported_vertical_column_for_input(&rgb, width - 1, height, 220, 256, 24, 1)
            .is_err()
    );
    assert!(split_supported_vertical_column_for_input(&[], 0, 0, 220, 256, 24, 1).is_err());
    assert_eq!(rgb, original);
}
fn advanced_gaps(rgb: &[u8], width: usize, height: usize, threshold: u8) -> Vec<BlankGap> {
    let scan = super::super::scan_ink_runs(rgb, width, height, threshold);
    retain_advance_boundaries(scan.gaps, &scan.widths, &scan.extents, height)
}

#[test]
fn advance_supported_by_six_adjacent_pairs_covers_a_broader_composite() {
    let widths = [vec![20; 7], vec![5, 13]].concat();
    let extents = [vec![26; 7], vec![8, 24]].concat();
    let spaces = [vec![6; 6], vec![8, 3]].concat();
    let original = gaps(&widths, &spaces, 0);
    let interior = original.last().unwrap().midpoint;
    let actual = retain_advance_boundaries(original, &widths, &extents, 34);
    assert!(
        actual.iter().all(|g| g.midpoint != interior),
        "21px glyph fits the supported 26px advance, beyond the 20px component width"
    );
}

#[test]
fn advance_five_adjacent_pairs_are_insufficient_but_six_are_supported() {
    for anchors in [6, 7] {
        let widths = [vec![20; anchors], vec![5, 13]].concat();
        let extents = [vec![26; anchors], vec![8, 24]].concat();
        let spaces = [vec![6; anchors - 1], vec![8, 3]].concat();
        let original = gaps(&widths, &spaces, 0);
        let interior = original.last().unwrap().midpoint;
        let actual = retain_advance_boundaries(original, &widths, &extents, 34);
        assert_eq!(actual.iter().any(|g| g.midpoint == interior), anchors == 6);
    }
}

#[test]
fn advance_cannot_skip_narrow_components_to_invent_a_large_pitch() {
    let mut widths = vec![20];
    let mut extents = vec![26];
    for _ in 0..6 {
        widths.extend([1, 20]);
        extents.extend([2, 26]);
    }
    widths.extend([5, 13]);
    extents.extend([8, 24]);
    let mut spaces = vec![30; widths.len() - 1];
    *spaces.last_mut().unwrap() = 3;
    let expected: Vec<_> = gaps(&widths, &spaces, 0)
        .iter()
        .map(|g| g.midpoint)
        .collect();
    let actual = retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34);
    assert_eq!(
        actual.iter().map(|g| g.midpoint).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn advance_does_not_join_independent_thin_strokes_with_jittered_spacing() {
    let widths = [vec![20; 7], vec![2; 31]].concat();
    let extents = vec![26; widths.len()];
    let spaces = [
        vec![6; 6],
        vec![20],
        (0..30).map(|i| if i % 2 == 0 { 22 } else { 26 }).collect(),
    ]
    .concat();
    let expected: Vec<_> = gaps(&widths, &spaces, 0)
        .iter()
        .map(|g| g.midpoint)
        .collect();
    let actual = retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34);
    assert_eq!(
        actual.iter().map(|g| g.midpoint).collect::<Vec<_>>(),
        expected,
        "the pitch is not enough evidence to merge separate full-height strokes"
    );
}

#[test]
fn advance_envelope_is_capped_by_the_original_column_height() {
    for internal_gap in [2, 4] {
        let widths = [vec![20; 7], vec![8, 24]].concat();
        let extents = [vec![26; 7], vec![8, 26]].concat();
        let spaces = [vec![40; 7], vec![internal_gap]].concat();
        let original = gaps(&widths, &spaces, 0);
        let interior = original.last().unwrap().midpoint;
        let actual = retain_advance_boundaries(original, &widths, &extents, 34);
        assert_eq!(
            actual.iter().any(|g| g.midpoint == interior),
            internal_gap == 4
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn advance_pixel_support_survives_scale_translation_reflection_and_color(
        scale in 1usize..5,padding in 0usize..18,reflected in any::<bool>(),
        foreground in prop::array::uniform3(0u8..121),background in prop::array::uniform3(230u8..=255),
    ) {
        let widths:Vec<_>=[vec![20;7],vec![5,13]].concat().into_iter().map(|n|n*scale).collect();
        let extents:Vec<_>=[vec![26;7],vec![8,24]].concat().into_iter().map(|n|n*scale).collect();
        let spaces:Vec<_>=[vec![6;6],vec![8,3]].concat().into_iter().map(|n|n*scale).collect();
        let width=2*padding+widths.iter().sum::<usize>()+spaces.iter().sum::<usize>();let height=34*scale;
        let mut rgb=background.repeat(width*height);let mut x0=padding;
        for (i,(&w,&extent))in widths.iter().zip(&extents).enumerate() {
            let y0=(height-extent)/2;
            for y in y0..y0+extent {for x in x0..x0+w {
                let actual_x=if reflected {width-1-x}else{x};
                rgb[(y*width+actual_x)*3..(y*width+actual_x+1)*3].copy_from_slice(&foreground);
            }}
            x0+=w+spaces.get(i).copied().unwrap_or(0);
        }
        let actual=advanced_gaps(&rgb,width,height,220);
        prop_assert_eq!(actual.len(),7,"preserve all seven exterior gaps and remove only the supported internal gap");
    }
}

fn gaps(widths: &[usize], spaces: &[usize], padding: usize) -> Vec<BlankGap> {
    assert_eq!(widths.len(), spaces.len() + 1);
    let mut end = padding + widths[0];
    spaces
        .iter()
        .enumerate()
        .map(|(i, &width)| {
            let gap = BlankGap {
                midpoint: end + width / 2,
                width,
                word_space: false,
                preferred_split: false,
                internal_glyph_gap: false,
            };
            end += width + widths[i + 1];
            gap
        })
        .collect()
}

#[test]
fn supported_composite_stays_whole_even_with_a_low_wide_component() {
    let widths = [vec![20; 6], vec![5, 12]].concat();
    let extents = [vec![26; 6], vec![8, 24]].concat();
    let spaces = [vec![6; 5], vec![8, 3]].concat();
    let original = gaps(&widths, &spaces, 0);
    let expected: Vec<_> = original[..6].iter().map(|g| g.midpoint).collect();
    let actual = retain_advance_boundaries(original, &widths, &extents, 34);
    assert_eq!(
        actual.iter().map(|g| g.midpoint).collect::<Vec<_>>(),
        expected,
        "a low component and a body in the same supported width envelope are one glyph"
    );
}

#[test]
fn an_envelope_one_pixel_over_the_median_keeps_its_boundary() {
    let widths = [vec![20; 6], vec![5, 13]].concat();
    let extents = [vec![26; 6], vec![8, 24]].concat();
    let spaces = [vec![6; 5], vec![8, 3]].concat();
    let expected: Vec<_> = gaps(&widths, &spaces, 0)
        .iter()
        .map(|g| g.midpoint)
        .collect();
    let actual = retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34);
    assert_eq!(
        actual.iter().map(|g| g.midpoint).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn five_anchors_cannot_establish_a_composite_width() {
    let widths = [vec![20; 5], vec![5, 12]].concat();
    let extents = [vec![26; 5], vec![8, 24]].concat();
    let spaces = [vec![6; 4], vec![8, 3]].concat();
    let actual = retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34);
    assert_eq!(
        actual.len(),
        6,
        "do not suppress boundaries without all six anchors"
    );
}

#[test]
fn a_chain_restarts_its_envelope_instead_of_growing_without_bound() {
    let widths = [vec![20; 6], vec![6; 7]].concat();
    let extents = [vec![26; 6], vec![8; 7]].concat();
    let spaces = [vec![6; 5], vec![8], vec![2; 6]].concat();
    let expected: Vec<_> = gaps(&widths, &spaces, 0)
        .iter()
        .enumerate()
        .filter(|(i, _)| ![6, 8, 10].contains(i))
        .map(|(_, g)| g.midpoint)
        .collect();
    let actual = retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34);
    assert_eq!(
        actual.iter().map(|g| g.midpoint).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn short_support_does_not_count_as_full_height_anchors() {
    let widths = [vec![20; 6], vec![5, 12]].concat();
    let extents = [vec![13; 6], vec![8, 24]].concat();
    let spaces = [vec![6; 5], vec![8, 3]].concat();
    assert_eq!(
        retain_advance_boundaries(gaps(&widths, &spaces, 0), &widths, &extents, 34).len(),
        7
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn supported_pixel_components_preserve_envelopes_under_scale_translation_reflection_and_color(
        scale in 1usize..5, padding in 0usize..18, reflected in any::<bool>(),
        foreground in prop::array::uniform3(0u8..121),background in prop::array::uniform3(230u8..=255),
    ) {
        let widths:Vec<_>=[vec![20;6],vec![5,12]].concat().into_iter().map(|n|n*scale).collect();
        let extents:Vec<_>=[vec![26;6],vec![8,24]].concat().into_iter().map(|n|n*scale).collect();
        let spaces:Vec<_>=[vec![6;5],vec![8,3]].concat().into_iter().map(|n|n*scale).collect();
        let width=2*padding+widths.iter().sum::<usize>()+spaces.iter().sum::<usize>();
        let height=34*scale;
        let mut rgb=background.repeat(width*height);
        let mut x0=padding;
        for (i,(&w,&extent)) in widths.iter().zip(&extents).enumerate() {
            let y0=(height-extent)/2;
            for y in y0..y0+extent {
                for x in x0..x0+w {
                    let actual_x=if reflected {width-1-x}else{x};
                    rgb[(y*width+actual_x)*3..(y*width+actual_x+1)*3].copy_from_slice(&foreground);
                }
            }
            x0+=w+spaces.get(i).copied().unwrap_or(0);
        }
        let actual=advanced_gaps(&rgb,width,height,220);
        prop_assert_eq!(actual.len(),6,"six exterior gaps must survive, the one internal gap must not");
        let internal=gaps(&widths,&spaces,padding).last().unwrap().midpoint;
        let internal=if reflected {width-internal}else{internal};
        prop_assert!(actual.iter().all(|g|g.midpoint!=internal));
    }
}
