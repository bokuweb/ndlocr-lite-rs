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
