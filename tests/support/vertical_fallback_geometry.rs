use super::*;

fn column(groups: usize, advance: usize, filled: bool) -> (Vec<u8>, usize, usize) {
    let (w, h) = (20, groups * advance + 8);
    let mut rgb = vec![255; w * h * 3];
    for g in 0..groups {
        let top = 4 + g * advance;
        for y in top..top + 16 {
            for x in 2..18 {
                if filled || x == 2 || x == 17 || y == top || y == top + 15 {
                    let p = (y * w + x) * 3;
                    rgb[p..p + 3].fill(0);
                }
            }
        }
    }
    (rgb, w, h)
}

#[test]
fn supported_single_column_keeps_original_coordinates_and_pixels() {
    let (rgb, w, h) = column(12, 20, false);
    let original = rgb.clone();
    let bounds = single_column(&rgb, w, h).expect("coherent square glyph column");
    assert_eq!(bounds, [1, 3, 19, 241]);
    for y in 0..h {
        for x in 0..w {
            let p = (y * w + x) * 3;
            if rgb[p..p + 3] != [255, 255, 255] {
                assert!(
                    bounds[0] as usize <= x
                        && x < bounds[2] as usize
                        && bounds[1] as usize <= y
                        && y < bounds[3] as usize
                );
            }
        }
    }
    assert_eq!(rgb, original);
}

#[test]
fn six_anchors_are_required_without_accepting_blank_filled_or_thin_shapes() {
    let (rgb, w, h) = column(6, 40, false);
    assert!(single_column(&rgb, w, h).is_some(), "six supported glyphs");
    let (rgb, w, h) = column(5, 40, false);
    assert_eq!(single_column(&rgb, w, h), None, "five anchors");
    let (rgb, w, h) = column(12, 20, true);
    assert_eq!(single_column(&rgb, w, h), None, "solid squares");
    assert_eq!(single_column(&vec![255; 20 * 248 * 3], 20, 248), None);
    let mut thin = vec![255; 20 * 248 * 3];
    for y in 4..244 {
        let p = (y * 20 + 9) * 3;
        thin[p..p + 6].fill(0);
    }
    assert_eq!(single_column(&thin, 20, 248), None);
}

#[test]
fn multiple_columns_and_tall_horizontal_words_are_not_promoted() {
    let (one, w, h) = column(24, 20, false);
    let width = 44;
    let mut two = vec![255; width * h * 3];
    for y in 0..h {
        for x in 0..w {
            let a = (y * w + x) * 3;
            let b = (y * width + x) * 3;
            two[b..b + 3].copy_from_slice(&one[a..a + 3]);
            let b = (y * width + x + 22) * 3;
            two[b..b + 3].copy_from_slice(&one[a..a + 3]);
        }
    }
    assert_eq!(
        single_column(&two, width, h),
        None,
        "two distinct text columns"
    );
    let (w, h) = (56, 560);
    let mut words = vec![255; w * h * 3];
    for row in 0..12 {
        let top = 8 + row * 45;
        for letter in 0..3 {
            let left = 2 + letter * 18;
            for y in top..top + 16 {
                for x in left..left + 16 {
                    if x == left || x == left + 15 || y == top || y == top + 15 {
                        let p = (y * w + x) * 3;
                        words[p..p + 3].fill(0);
                    }
                }
            }
        }
    }
    assert_eq!(
        single_column(&words, w, h),
        None,
        "portrait horizontal words"
    );
}

#[test]
fn invalid_dimensions_and_horizontal_input_do_not_create_vertical_regions() {
    for (rgb, w, h) in [
        (vec![], 0, 0),
        (vec![0; 3], 20, 248),
        (vec![], usize::MAX, 2),
        (vec![255; 11 * 200 * 3], 11, 200),
    ] {
        assert_eq!(single_column(&rgb, w, h), None);
    }
    let (rgb, w, h) = column(12, 20, false);
    let horizontal =
        crate::pipeline::vertical_segment::rotate_vertical_column_rgb_u8(&rgb, w, h).unwrap();
    assert_eq!(single_column(&horizontal.data, h, w), None);
}

#[test]
fn aspect_ratio_boundary_is_inclusive_and_faint_ink_remains_covered() {
    let (rgb, w, _) = column(6, 26, false);
    assert!(
        single_column(&rgb[..w * 160 * 3], w, 160).is_some(),
        "height ==8width"
    );
    assert_eq!(single_column(&rgb[..w * 159 * 3], w, 159), None);
    let (mut rgb, w, h) = column(12, 20, false);
    let p = (w + 1) * 3;
    rgb[p..p + 3].fill(254);
    let b = single_column(&rgb, w, h).expect("faint AA keeps complete envelope");
    assert!(b[0] <= 1 && b[1] <= 1);
}

fn varied_columns(
    side: usize,
    groups: usize,
    gap: usize,
    padding: usize,
    second: Option<usize>,
) -> (Vec<u8>, usize, usize) {
    let w = if second.is_some() {
        side * 2 + gap + 4 + 2 * padding
    } else {
        side + 4 + 2 * padding
    };
    let h = groups * (side + 4) + 8 + 2 * padding + second.unwrap_or(0);
    let mut rgb = vec![255; w * h * 3];
    for col in 0..if second.is_some() { 2 } else { 1 } {
        let left = 2 + padding + col * (side + gap);
        let shift = if col == 1 { second.unwrap() } else { 0 };
        for group in 0..groups {
            let top = 4 + padding + group * (side + 4) + shift;
            for y in top..top + side {
                for x in left..left + side {
                    if x == left || x == left + side - 1 || y == top || y == top + side - 1 {
                        let p = (y * w + x) * 3;
                        rgb[p..p + 3].fill(40);
                    }
                }
            }
        }
    }
    (rgb, w, h)
}

use proptest::prelude::*;
proptest! {
 #![proptest_config(ProptestConfig::with_cases(64))]
 #[test]
 fn translated_and_scaled_columns_preserve_every_nonwhite_pixel(side in 12usize..25,groups in 16usize..25,padding in 0usize..4){
  let(mut rgb,w,h)=varied_columns(side,groups,4,padding,None);let p=(w+1)*3;rgb[p..p+3].fill(254);let original=rgb.clone();
  let b=single_column(&rgb,w,h).expect("supported single column across geometry variations");
  for y in 0..h{for x in 0..w{let p=(y*w+x)*3;if rgb[p..p+3]!=[255,255,255]{prop_assert!(b[0]as usize<=x&&x<b[2]as usize&&b[1]as usize<=y&&y<b[3]as usize);}}}
  prop_assert_eq!(rgb,original);
 }
 #[test]
 fn separate_staggered_columns_are_not_a_single_column(side in 12usize..25,groups in 32usize..41,gap in 4usize..13,shift in 0usize..12){
  let(rgb,w,h)=varied_columns(side,groups,gap,0,Some(shift));prop_assert_eq!(single_column(&rgb,w,h),None);
 }
}

#[test]
fn shared_support_keeps_exact_density_and_threshold_boundaries() {
    fn raster(width: usize, value: u8) -> Vec<u8> {
        let mut rgb = vec![255; width * 20 * 3];
        for group in 0..6 {
            for x in group * 36..group * 36 + 12 {
                for y in 4..16 {
                    let p = (y * width + x) * 3;
                    rgb[p..p + 3].fill(value);
                }
            }
        }
        rgb
    }
    assert!(
        crate::pipeline::line_chunks::has_fallback_column_support(&raster(216, 219), 216, 20),
        "six anchors at exactly one-third coverage"
    );
    assert!(
        !crate::pipeline::line_chunks::has_fallback_column_support(&raster(217, 219), 217, 20),
        "one more blank column drops coverage below one-third"
    );
    assert!(
        !crate::pipeline::line_chunks::has_fallback_column_support(&raster(216, 220), 216, 20),
        "threshold220 is not ink"
    );
}

#[test]
fn successful_vertical_candidate_calls_once_and_preserves_text_and_confidence() {
    use crate::infer::parseq::RecognizeResult;
    let (rgb, w, h) = column(12, 20, false);
    let original = rgb.clone();
    let calls = std::cell::Cell::new(0);
    let expected: Vec<_> = rgb
        .chunks_exact(w * 3)
        .skip(3)
        .take(238)
        .flat_map(|r| r[3..57].iter().copied())
        .collect();
    let chosen = try_recognize(&rgb, w, h, 0.3, |pixels, cw, ch| {
        calls.set(calls.get() + 1);
        assert_eq!((cw, ch), (18, 238));
        assert_eq!(pixels, expected);
        Ok(RecognizeResult {
            text: "  本文試験です A　".into(),
            mean_confidence: 0.95,
        })
    })
    .expect("supported result is reused");
    assert_eq!(calls.get(), 1);
    assert_eq!(chosen.0, [1, 3, 19, 241]);
    assert_eq!(chosen.1.text, "  本文試験です A　");
    assert_eq!(chosen.1.mean_confidence.to_bits(), 0.95f32.to_bits());
    assert_eq!(rgb, original);
}

#[test]
fn unsupported_or_failed_vertical_attempt_keeps_legacy_fallback_available() {
    use crate::infer::parseq::RecognizeResult;
    let (rgb, w, h) = column(12, 20, false);
    for (text, c, min, accepted) in [
        ("本文試験です", 0.899, 0.3, false),
        ("本文試験です", 0.9, 0.3, true),
        ("本文試験です", 1.0, 0.3, true),
        ("本文試験です", 1.001, 0.3, false),
        ("本文試験です", f32::NAN, 0.3, false),
        ("本文試験です", f32::INFINITY, 0.3, false),
        ("本文試験です", 0.94, 0.95, false),
        ("本文試験です", 0.95, 0.95, true),
        ("\n \t　", 1.0, 0.3, false),
        ("本文試験です", 1.0, f32::NAN, false),
    ] {
        let result = try_recognize(&rgb, w, h, min, |_, _, _| {
            Ok(RecognizeResult {
                text: text.into(),
                mean_confidence: c,
            })
        });
        assert_eq!(result.is_some(), accepted, "text={text:?}/c={c}/min={min}");
    }
    assert!(
        try_recognize(&rgb, w, h, 0.3, |_, _, _| anyhow::bail!(
            "recorded recognizer error"
        ))
        .is_none()
    );
    let calls = std::cell::Cell::new(0);
    assert!(
        try_recognize(&[], 0, 0, 0.3, |_, _, _| {
            calls.set(1);
            anyhow::bail!("must not call recognizer")
        })
        .is_none()
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn sparse_decoding_cannot_cover_many_anchors_even_with_high_confidence() {
    use crate::infer::parseq::RecognizeResult;
    let (rgb, w, h) = column(12, 20, false);
    for (text, accepted) in [
        ("甲乙丙丁一", false),
        ("甲乙丙丁一二", true),
        (" A B C D　　", false),
        ("甲乙丙𠮷甲乙", true),
    ] {
        let result = try_recognize(&rgb, w, h, 0.3, |_, _, _| {
            Ok(RecognizeResult {
                text: text.into(),
                mean_confidence: 0.99,
            })
        });
        assert_eq!(
            result.is_some(),
            accepted,
            "{text:?}: support counts Unicode nonwhitespace scalars, not bytes"
        );
    }
}

#[test]
fn repeated_or_three_kind_output_does_not_support_a_new_direction() {
    use crate::infer::parseq::RecognizeResult;
    let (rgb, w, h) = column(12, 20, false);
    for (text, accepted) in [
        ("AAAAAAAAAAAA", false),
        ("甲乙丙甲乙丙", false),
        ("甲乙丙丁甲乙", true),
        ("!@#$!@#$", false),
    ] {
        let result = try_recognize(&rgb, w, h, 0.3, |_, _, _| {
            Ok(RecognizeResult {
                text: text.into(),
                mean_confidence: 0.99,
            })
        });
        assert_eq!(
            result.is_some(),
            accepted,
            "{text:?}: at least four alphanumeric kinds are needed"
        );
    }
}

proptest::proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(48))]
 #[test]
 fn coverage_boundary_tracks_anchor_count_without_whitespace_padding(groups in 12usize..25) {
  use crate::infer::parseq::RecognizeResult;
  let (rgb,w,h)=column(groups,20,false);let needed=groups.div_ceil(2);
  for (count,accepted)in[(needed-1,false),(needed,true)] {
   let text:String="甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳".chars().cycle().take(count).collect();
   for decorated in [text.clone(),format!("  {}　",text.chars().map(|c|format!("{c}\t")).collect::<String>())] {
    let result=try_recognize(&rgb,w,h,0.3,|_,_,_|Ok(RecognizeResult{text:decorated.clone(),mean_confidence:0.99}));
    proptest::prop_assert_eq!(result.is_some(),accepted,"groups={}, count={}, text={:?}",groups,count,decorated);
    if let Some((_,result))=result{proptest::prop_assert_eq!(result.text,decorated);}
   }
  }
 }
}

mod envelope_refactor_properties {
    use super::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]
        #[test]
        fn complete_rgb_triplets_preserve_coordinate_envelope(
            width in 1usize..40, height in 1usize..50,
            points in proptest::collection::vec((0usize..2000, any::<[u8;3]>()),0..40),
        ) {
            let mut rgb=vec![255; width*height*3];
            for (offset,color) in points {let index=offset%(width*height);rgb[index*3..index*3+3].copy_from_slice(&color);}
            let mut occupied=Vec::new();
            for y in 0..height {for x in 0..width {
                if rgb[(y*width+x)*3..(y*width+x+1)*3] != [255,255,255] {occupied.push((x,y));}
            }}
            let expected=if occupied.is_empty(){None}else{
                let minx=occupied.iter().map(|p|p.0).min().unwrap();let maxx=occupied.iter().map(|p|p.0).max().unwrap();
                let miny=occupied.iter().map(|p|p.1).min().unwrap();let maxy=occupied.iter().map(|p|p.1).max().unwrap();
                Some(crate::pipeline::crop::BBox::new(minx.saturating_sub(1),miny.saturating_sub(1),(maxx+2).min(width),(maxy+2).min(height)))
            };
            prop_assert_eq!(nonwhite_envelope(&rgb,width,height),expected);
        }
    }
}
