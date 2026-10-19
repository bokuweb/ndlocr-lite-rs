use super::*;
use proptest::prelude::*;
use std::cell::Cell;
fn rec(text: &str, score: f32) -> RecognizeResult {
    RecognizeResult {
        text: text.into(),
        mean_confidence: score,
    }
}
fn sample() -> Vec<u8> {
    let mut rgb = vec![255; 1200 * 24 * 3];
    for start in [4, 584, 1176] {
        for y in 3..21 {
            for x in start..start + 12 {
                rgb[(y * 1200 + x) * 3..(y * 1200 + x + 1) * 3].copy_from_slice(&[20, 50, 80]);
            }
        }
    }
    rgb
}
#[test]
fn sparse_word_context_preserves_each_foreground_pixel_once() {
    let rgb = sample();
    let original = rgb.clone();
    let mut calls = 0;
    let mut seen = 0;
    let out = recover_overwide_band(&rgb, 1200, 24, (768, 24), |data, w, h| {
        calls += 1;
        assert_eq!(data.len(), w * h * 3);
        for p in data.chunks_exact(3) {
            if p != [255, 255, 255] {
                assert_eq!(p, [20, 50, 80]);
                seen += 1;
            }
        }
        Ok(rec(["図", "書", "館"][calls - 1], 0.99))
    })
    .expect("three separate words");
    assert_eq!(calls, 3);
    assert_eq!(out.text, "図 書 館");
    assert_eq!(seen, 3 * 12 * 18);
    assert_eq!(rgb, original);
    assert!((out.mean_confidence - 0.99).abs() < 1e-6);
}
#[test]
fn fitting_invalid_or_unsplittable_inputs_never_call_the_model() {
    let rgb = sample();
    for (data, w, h, input) in [
        (rgb.as_slice(), 1200, 24, (0, 24)),
        (rgb.as_slice(), 1200, 24, (768, 0)),
        (rgb.as_slice(), 1200, 0, (768, 24)),
        (&rgb[..rgb.len() - 1], 1200, 24, (768, 24)),
        (rgb.as_slice(), usize::MAX, 24, (768, 24)),
        (rgb.as_slice(), 1200, 24, (1200, 24)),
    ] {
        assert!(recover_overwide_band(data, w, h, input, |_, _, _| panic!("ineligible")).is_none());
    }
    assert!(
        recover_overwide_band(
            &vec![0; 1200 * 24 * 3],
            1200,
            24,
            (768, 24),
            |_, _, _| panic!("dense")
        )
        .is_none()
    );
}
#[test]
fn uncertainty_or_failure_aborts_the_candidate() {
    let rgb = sample();
    for score in [-0.01, 1.01, f32::NAN, f32::INFINITY] {
        let calls = Cell::new(0);
        assert!(
            recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| {
                calls.set(calls.get() + 1);
                Ok(rec("A", score))
            })
            .is_none()
        );
        assert_eq!(calls.get(), 1);
    }
    assert!(recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| Ok(rec("", 1.0))).is_none());
    let calls = Cell::new(0);
    assert!(
        recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                Err(anyhow::anyhow!("second chunk failed"))
            } else {
                Ok(rec("A", 1.0))
            }
        })
        .is_none()
    );
    assert_eq!(calls.get(), 2);
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(24))]
 #[test]
 fn selected_text_is_only_the_actual_ordered_candidate_outputs(words in proptest::collection::vec("[A-Z]{1,4}",3)){
  let rgb=sample();let mut index=0;let out=recover_overwide_band(&rgb,1200,24,(768,24),|_,_,_|{let value=rec(&words[index],0.99);index+=1;Ok(value)}).unwrap();prop_assert_eq!(index,3);prop_assert_eq!(out.text,words.join(" "));
 }
}

#[test]
fn sparse_isolated_words_keep_small_balanced_pure_white_margins() {
    let rgb = sample();
    let mut calls = 0;
    let out = recover_overwide_band(&rgb, 1200, 24, (768, 24), |data, w, h| {
        calls += 1;
        assert_eq!((w, h), (14, 24));
        for y in 0..h {
            assert_eq!(&data[(y * w) * 3..(y * w + 1) * 3], &[255; 3]);
            assert_eq!(&data[(y * w + w - 1) * 3..(y * w + w) * 3], &[255; 3]);
        }
        Ok(rec("A", 0.99))
    });
    assert!(out.is_some());
    assert_eq!(calls, 3);
}

#[test]
fn low_confidence_square_words_need_a_high_confidence_native_context() {
    let rgb = sample();
    let mut calls = 0;
    let out = recover_overwide_band(&rgb, 1200, 24, (768, 24), |data, w, h| {
        calls += 1;
        if calls <= 3 {
            Ok(rec("XX", 0.6))
        } else {
            assert_eq!((w, h), (42, 24));
            assert_eq!(
                data.chunks_exact(3)
                    .filter(|p| *p != [255, 255, 255])
                    .count(),
                3 * 12 * 18
            );
            Ok(rec("ABC", 0.99))
        }
    })
    .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(out.text, "A B C");
    assert_eq!(out.mean_confidence, 0.99);
}
#[test]
fn unmatched_or_uncertain_context_cannot_assign_arbitrary_words() {
    let rgb = sample();
    for (text, score) in [
        ("AB", 0.99),
        ("ABCD", 0.99),
        ("ABC", 0.9799),
        ("", 1.0),
        ("ABC", f32::NAN),
    ] {
        let mut calls = 0;
        assert!(
            recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| {
                calls += 1;
                if calls <= 3 {
                    Ok(rec("X", 0.6))
                } else {
                    Ok(rec(text, score))
                }
            })
            .is_none(),
            "{text}: {score}"
        );
        assert_eq!(calls, 4);
    }
}

#[test]
fn context_agreement_preserves_multi_character_words_and_unicode_groups() {
    let rgb = sample();
    for (context, local, expected) in [("甲乙丙", "XX", "甲 乙 丙"), ("XYXYXY", "XY", "XY XY XY")]
    {
        let mut calls = 0;
        let out = recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| {
            calls += 1;
            Ok(if calls <= 3 {
                rec(local, 0.6)
            } else {
                rec(context, 0.99)
            })
        })
        .unwrap();
        assert_eq!(calls, 4);
        assert_eq!(out.text, expected);
    }
}
#[test]
fn verification_context_must_fit_the_supplied_model() {
    let (w, h) = (11420, 24);
    let mut rgb = vec![255; w * h * 3];
    for i in 0..20 {
        for y in 3..21 {
            for x in i * 600 + 4..i * 600 + 16 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    let mut calls = 0;
    let out = recover_overwide_band(&rgb, w, h, (256, 24), |_, _, _| {
        calls += 1;
        assert!(calls <= 20, "must not compress context beyond native input");
        Ok(rec("A", 0.6))
    });
    assert!(out.is_none());
    assert_eq!(calls, 20);
}
#[test]
fn context_confidence_boundary_is_inclusive_and_not_calibrated_as_truth() {
    let rgb = sample();
    for score in [0.9799, 0.98] {
        let mut calls = 0;
        let out = recover_overwide_band(&rgb, 1200, 24, (768, 24), |_, _, _| {
            calls += 1;
            Ok(if calls <= 3 {
                rec("A", 0.6)
            } else {
                rec("ABC", score)
            })
        });
        assert_eq!(out.is_some(), score == 0.98);
        assert_eq!(calls, 4);
    }
}

#[test]
fn native_overflow_dense_text_requires_the_same_safe_chunk_verification() {
    let (w, h) = (1200, 24);
    let mut rgb = vec![255; w * h * 3];
    for x0 in (3..1180).step_by(18) {
        for y in 3..21 {
            for x in x0..x0 + 12 {
                if y < 6 || y >= 18 || x < x0 + 3 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
    }
    let original = rgb.clone();
    let mut calls = 0;
    let mut seen = 0;
    let out = recover_overwide_band(&rgb, w, h, (768, 24), |data, pw, ph| {
        calls += 1;
        assert_eq!(data.len(), pw * ph * 3);
        seen += data.chunks_exact(3).filter(|p| *p != [255; 3]).count();
        Ok(rec("ABC", 0.99))
    });
    assert!(
        out.is_some(),
        "pixel density must not bypass native-capacity verification"
    );
    assert!(calls >= 2);
    assert_eq!(seen, rgb.chunks_exact(3).filter(|p| *p != [255; 3]).count());
    assert_eq!(rgb, original);
}

/// 末尾2文字だけが不確かな場合、前半を保持した実モデル候補を選ぶ。
#[test]
fn short_tail_recheck_preserves_prefix_and_original_rgb() {
    use crate::pipeline::line_chunks::LineChunk;
    let width = 400;
    let height = 40;
    let rgb: Vec<_> = (0..width * height * 3).map(|i| (i % 251) as u8).collect();
    let original = rgb.clone();
    let chunks = vec![
        LineChunk {
            x0: 20,
            x1: 280,
            space_before: false,
            gap_before: 0,
        },
        LineChunk {
            x0: 280,
            x1: 340,
            space_before: false,
            gap_before: 3,
        },
        LineChunk {
            x0: 340,
            x1: 400,
            space_before: true,
            gap_before: 24,
        },
    ];
    let pair = TailPair {
        rgb: &rgb,
        size: (width, height),
        input: (768, 24),
        chunks: &chunks,
        index: 1,
    };
    let mut calls = 0;
    let out = pair
        .recheck(
            &rec("本文の続き", 0.99),
            &rec("5.", 0.91),
            &mut |data, w, h| {
                calls += 1;
                assert_eq!((w, h), (320, 40));
                let expected: Vec<_> = (0..height)
                    .flat_map(|y| {
                        rgb[(y * width + 20) * 3..(y * width + 340) * 3]
                            .iter()
                            .copied()
                    })
                    .collect();
                assert_eq!(data, expected);
                Ok(rec("本文の続きら、", 0.999))
            },
        )
        .expect("prefix-preserving context");
    assert_eq!(out.text, "本文の続きら、");
    assert_eq!(out.mean_confidence, 0.999);
    assert_eq!(calls, 1);
    assert_eq!(rgb, original);
}

/// 不十分な文脈・境界・容量は追加forwardせず、候補の不整合/失敗は採用しない。
#[test]
fn short_tail_recheck_requires_independent_support() {
    use crate::pipeline::line_chunks::LineChunk;
    for case in 0..15 {
        let rgb = vec![255; 400 * 40 * 3];
        let mut chunks = vec![
            LineChunk {
                x0: 20,
                x1: 280,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 280,
                x1: 340,
                space_before: false,
                gap_before: 3,
            },
            LineChunk {
                x0: 340,
                x1: 400,
                space_before: true,
                gap_before: 24,
            },
        ];
        let mut previous = rec("本文の続き", 0.99);
        let mut current = rec("5.", 0.91);
        let mut input = (768, 24);
        match case {
            0 => chunks[1].space_before = true,
            1 => chunks[2].space_before = false,
            2 => chunks[1].x0 += 1,
            3 => {
                chunks[1].x1 = 361;
                chunks[2].x0 = 361;
            }
            4 => chunks[0].x0 = 170,
            5 => input = (191, 24),
            6 => previous.mean_confidence = 0.979,
            7 => current.mean_confidence = 0.95,
            8 => current.mean_confidence = 0.899,
            9 => current.text = "長い末尾".into(),
            10 => current.text = " ".into(),
            11 => current.mean_confidence = f32::NAN,
            12 => previous.mean_confidence = f32::INFINITY,
            13 => input = (0, 24),
            14 => input = (768, 0),
            _ => unreachable!(),
        }
        let pair = TailPair {
            rgb: &rgb,
            size: (400, 40),
            input,
            chunks: &chunks,
            index: 1,
        };
        assert!(
            pair.recheck(&previous, &current, &mut |_, _, _| panic!(
                "ineligible case {case}"
            ))
            .is_none()
        );
    }
    let rgb = vec![255; 400 * 40 * 3];
    let chunks = vec![
        LineChunk {
            x0: 20,
            x1: 280,
            space_before: false,
            gap_before: 0,
        },
        LineChunk {
            x0: 280,
            x1: 340,
            space_before: false,
            gap_before: 3,
        },
    ];
    let pair = TailPair {
        rgb: &rgb,
        size: (400, 40),
        input: (192, 24),
        chunks: &chunks,
        index: 1,
    };
    for (text, score) in [
        ("前半を変更ら、", 0.999),
        ("本文の続きら", 0.999),
        ("本文の続きら、。", 0.999),
        ("本文の続き 、", 0.999),
        ("本文の続きら、", 0.97),
        ("本文の続きら、", f32::NAN),
        ("本文の続きら、", 1.01),
        ("本文の続きら、", f32::INFINITY),
    ] {
        assert!(
            pair.recheck(
                &rec("本文の続き", 0.99),
                &rec("5.", 0.91),
                &mut |_, _, _| Ok(rec(text, score))
            )
            .is_none(),
            "{text}/{score}"
        );
    }
    assert!(
        pair.recheck(
            &rec("本文の続き", 0.99),
            &rec("5.", 0.91),
            &mut |_, _, _| Err(anyhow::anyhow!("failed"))
        )
        .is_none()
    );
    // 容量境界は等号を含み、元の文字数加重confidenceとの同点は採用しない。
    assert!(
        pair.recheck(
            &rec("本文の続き", 0.99),
            &rec("5.", 0.91),
            &mut |_, _, _| Ok(rec("本文の続きら、", 0.999))
        )
        .is_some()
    );
    assert!(
        pair.recheck(
            &rec("A".repeat(30).as_str(), 1.0),
            &rec("5.", 0.9375),
            &mut |_, _, _| Ok(rec(&("A".repeat(30) + "ら、"), 0.99609375))
        )
        .is_none()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    /// 画素の整数拡大と移動でも、同じ容量境界・モデル候補を保持する。
    #[test]
    fn short_tail_recheck_scales_with_original_coordinates(scale in 1usize..6, margin in 0usize..20, tone in 0u8..220) {
        use crate::pipeline::line_chunks::LineChunk;
        let width=400*scale+margin;let height=40*scale;
        let rgb=vec![tone;width*height*3];
        let chunks=vec![
            LineChunk{x0:20*scale+margin,x1:280*scale+margin,space_before:false,gap_before:0},
            LineChunk{x0:280*scale+margin,x1:340*scale+margin,space_before:false,gap_before:3*scale},
        ];
        for (iw,eligible) in [(191,false),(192,true)] {
            let pair=TailPair{rgb:&rgb,size:(width,height),input:(iw,24),chunks:&chunks,index:1};
            let mut calls=0;
            let out=pair.recheck(&rec("本文の続き",0.99),&rec("5.",0.91),&mut |data,w,h|{
                calls+=1;assert_eq!((w,h),(320*scale,40*scale));assert!(data.iter().all(|&x|x==tone));Ok(rec("本文の続きら、",0.999))
            });
            prop_assert_eq!(out.is_some(),eligible);prop_assert_eq!(calls,usize::from(eligible));
        }
    }
}
