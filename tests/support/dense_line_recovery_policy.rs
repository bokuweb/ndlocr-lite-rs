use super::*;
use crate::pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input};
use proptest::prelude::*;
use std::cell::Cell;
fn rec(text: &str, confidence: f32) -> RecognizeResult {
    RecognizeResult {
        text: text.into(),
        mean_confidence: confidence,
    }
}
fn image(groups: usize, height: usize, glyph_height: usize) -> (Vec<u8>, usize) {
    let width = groups * 28;
    let mut rgb = vec![255; width * height * 3];
    for group in 0..groups {
        for y in 2..2 + glyph_height {
            for x in group * 28 + 3..group * 28 + 17 {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].copy_from_slice(&[20, 40, 60]);
            }
        }
    }
    (rgb, width)
}
#[test]
fn accepts_only_real_chunk_outputs_and_keeps_original_rgb() {
    let (rgb, w) = image(100, 24, 18);
    let original = rgb.clone();
    let chunks = split_line_for_input(&rgb, w, 24, 220, 768, 24).unwrap();
    assert!(chunks.len() > 1);
    let crops: Vec<_> = (0..chunks.len())
        .map(|i| crop_line_chunk_rgb_u8(&rgb, w, 24, &chunks, i).unwrap())
        .collect();
    let mut index = 0;
    let result = recover_dense_horizontal_line(
        &rec("欠落", 0.99),
        &rgb,
        w,
        24,
        (768, 24),
        |data, cw, ch| {
            let crop = &crops[index];
            assert_eq!((cw, ch), (crop.width, crop.height));
            assert_eq!(data, crop.data);
            index += 1;
            Ok(rec(
                "甲乙丙丁戊己庚辛壬癸",
                if index % 2 == 0 { 1.0 } else { 0.9 },
            ))
        },
    )
    .expect("many supported glyphs explain a truncated prefix");
    assert_eq!(index, chunks.len());
    assert_eq!(rgb, original);
    let expected = std::iter::repeat_n("甲乙丙丁戊己庚辛壬癸", chunks.len())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(result.text, expected);
    assert!(
        (result.mean_confidence
            - (0.9f32 * (index.div_ceil(2)) as f32 + 1.0 * (index / 2) as f32) / index as f32)
            .abs()
            < 0.00001
    );
}
#[test]
fn ineligible_inputs_never_call_recognition() {
    let (rgb, w) = image(100, 24, 18);
    for (text, width, height, input, data) in [
        ("ABCDEFGHIJKLMNOPQ", w, 24, (768, 24), rgb.as_slice()),
        ("AB", w, 24, (0, 24), rgb.as_slice()),
        ("AB", w, 24, (768, 0), rgb.as_slice()),
        ("AB", w, 0, (768, 24), rgb.as_slice()),
        ("AB", usize::MAX, 24, (768, 24), rgb.as_slice()),
        ("AB", w, 24, (768, 24), &rgb[..rgb.len() - 1]),
    ] {
        assert!(
            recover_dense_horizontal_line(
                &rec(text, 0.9),
                data,
                width,
                height,
                input,
                |_, _, _| panic!("ineligible")
            )
            .is_none()
        );
    }
    for (groups, h, gh) in [(39, 24, 18), (100, 11, 7), (100, 24, 7)] {
        let (data, width) = image(groups, h, gh);
        assert!(
            recover_dense_horizontal_line(
                &rec("AB", 0.9),
                &data,
                width,
                h,
                (768, 24),
                |_, _, _| panic!("insufficient support")
            )
            .is_none()
        );
    }
    for color in [0, 255] {
        assert!(
            recover_dense_horizontal_line(
                &rec("", 0.9),
                &vec![color; w * 24 * 3],
                w,
                24,
                (768, 24),
                |_, _, _| panic!("flat image")
            )
            .is_none()
        );
    }
    // Exact aspect boundary is excluded even with enough supported ink groups.
    let (data, width) = image(100, 25, 18);
    assert!(
        recover_dense_horizontal_line(
            &rec("AB", 0.9),
            &data,
            width,
            25,
            (1344, 24),
            |_, _, _| panic!("aspect boundary")
        )
        .is_none()
    );
}
#[test]
fn support_must_explain_four_times_the_initial_text() {
    let (data, w) = image(60, 16, 12);
    assert!(
        recover_dense_horizontal_line(
            &rec("ABCDEFGHIJKLMNOP", 0.99),
            &data,
            w,
            16,
            (768, 24),
            |_, _, _| panic!("not enough groups")
        )
        .is_none()
    );
}
#[test]
fn failed_or_uncertain_chunks_abort_without_replacing_the_base() {
    let (data, w) = image(100, 24, 18);
    for score in [0.8999, 1.0001, f32::NAN, f32::INFINITY, -1.0] {
        let count = Cell::new(0);
        assert!(
            recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| {
                count.set(count.get() + 1);
                Ok(rec("甲乙丙丁戊己庚辛", score))
            })
            .is_none()
        );
        assert_eq!(count.get(), 1);
    }
    assert!(
        recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| Err(
            anyhow::anyhow!("inference failed")
        ))
        .is_none()
    );
    assert!(
        recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| Ok(
            rec("", 1.0)
        ))
        .is_none()
    );
}
#[test]
fn short_or_repetitive_candidates_do_not_explain_dense_text() {
    let (data, w) = image(100, 24, 18);
    for text in [
        "a",
        "IIIIIIIIIIIIIIIIIIII",
        "甲甲甲甲甲甲甲甲甲甲",
        "--------------------",
    ] {
        assert!(
            recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| Ok(
                rec(text, 1.0)
            ))
            .is_none(),
            "{text}"
        );
    }
}
#[test]
fn distinct_but_too_short_candidate_is_rejected() {
    let (data, w) = image(100, 24, 18);
    let calls = Cell::new(0);
    let out =
        recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| {
            calls.set(calls.get() + 1);
            Ok(rec(if calls.get() == 1 { "abcd" } else { "x" }, 1.0))
        });
    assert!(
        calls.get() + 3 < 50,
        "fixture must emit less than half the supported groups"
    );
    assert!(out.is_none());
}

proptest! {
    #[test]
    fn insufficient_ink_height_is_always_skipped(groups in 40usize..140,h in 12usize..48) {
        let (data,w)=image(groups,h,h/3-1);
        let out=recover_dense_horizontal_line(&rec("AB",0.99),&data,w,h,(256,24),|_,_,_|panic!("short marks are not glyphs"));
        prop_assert!(out.is_none());
    }
}

/// 生モデルの停止を確認し、真値を渡さずに全幅の元RGBから復旧する。
#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned 100-character model"]
fn dense_full_width_model_recovers_four_frozen_japanese_lines() {
    use crate::{infer::cached::ParseqPool, io::load_rgb_u8};
    use std::path::Path;
    let pool = ParseqPool::load_for_fragments(
        Path::new("models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
        1,
    )
    .unwrap();
    for name in [
        "japanese_gothic24",
        "japanese_gothic36",
        "japanese_mincho24",
        "japanese_mincho36",
    ] {
        let image = load_rgb_u8(
            &Path::new("tests/fixtures/long_line_split/images").join(format!("{name}.png")),
        )
        .unwrap();
        let base = pool
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert!(base.text.chars().count() <= 16);
        let result = recover_dense_horizontal_line(
            &base,
            &image.data,
            image.width,
            image.height,
            pool.input_dimensions(),
            |rgb, w, h| pool.recognize_horizontal_rgb_u8(rgb, w, h),
        )
        .expect(name);
        let truth = std::fs::read_to_string(
            Path::new("tests/fixtures/long_line_split/truth").join(format!("{name}.txt")),
        )
        .unwrap();
        assert_eq!(result.text, truth, "{name}");
    }
}

#[test]
fn confidence_weights_emitted_characters_without_added_spaces() {
    let (data, w) = image(100, 24, 18);
    let mut calls = 0;
    let mut chars = 0;
    let mut sum = 0.0f64;
    let out =
        recover_dense_horizontal_line(&rec("AB", 0.99), &data, w, 24, (768, 24), |_, _, _| {
            calls += 1;
            let (text, score) = if calls % 2 == 0 {
                ("abcdefgh", 1.0)
            } else {
                ("甲乙丙丁戊己庚辛壬癸", 0.9)
            };
            chars += text.chars().count();
            sum += f64::from(score) * text.chars().count() as f64;
            Ok(rec(text, score))
        })
        .unwrap();
    assert!((out.mean_confidence - (sum / chars as f64) as f32).abs() < 1e-6);
    assert!(
        out.text.chars().count() > chars,
        "real word gaps add unweighted spaces"
    );
}

#[test]
fn overlapping_half_of_smaller_region_is_ambiguous_for_both() {
    for bounds in [
        vec![[0, 0, 100, 20], [50, 0, 150, 20]],
        vec![[0, 0, 100, 20], [10, 5, 90, 15]],
        vec![[78, 0, 2835, 33], [127, 7, 2832, 33]],
    ] {
        assert_eq!(isolated_recovery_regions(&bounds), vec![false, false]);
    }
    assert_eq!(
        isolated_recovery_regions(&[[0, 0, 100, 20], [51, 0, 151, 20]]),
        vec![true, true]
    );
    assert_eq!(
        isolated_recovery_regions(&[[0, 0, 100, 20], [0, 20, 100, 40], [200, 0, 300, 20]]),
        vec![true, true, true]
    );
    assert_eq!(
        isolated_recovery_regions(&[[0, 0, 0, 20], [10, 10, 5, 0]]),
        vec![false, false]
    );
    assert!(isolated_recovery_regions(&[]).is_empty());
    assert_eq!(
        isolated_recovery_regions(&[[i32::MIN, i32::MIN, i32::MAX, i32::MAX]; 2]),
        vec![false, false]
    );
}
proptest! {
    #[test]
    fn ambiguous_regions_are_invariant_under_order_scale_and_translation(scale in 1i32..30,x in -100i32..100,y in -100i32..100,reverse in any::<bool>()) {
        let mut boxes=vec![[x,y,x+100*scale,y+20*scale],[x+25*scale,y,x+125*scale,y+20*scale],[x,y+30*scale,x+100*scale,y+50*scale]];
        let mut expected=vec![false,false,true];if reverse{boxes.reverse();expected.reverse();}
        prop_assert_eq!(isolated_recovery_regions(&boxes),expected);
    }
}

#[test]
fn empty_and_sixteen_character_prefixes_can_be_recovered() {
    let (data, w) = image(100, 24, 18);
    for prefix in ["", "ABCDEFGHIJKLMNOP"] {
        let result = recover_dense_horizontal_line(
            &rec(prefix, 0.99),
            &data,
            w,
            24,
            (768, 24),
            |_, _, _| Ok(rec("甲乙丙丁戊己庚辛壬癸", 0.99)),
        );
        assert!(
            result.is_some(),
            "prefix at supported character boundary: {prefix}"
        );
    }
}
