use super::*;
use crate::infer::parseq::RecognizeResult;
use std::cell::Cell;

fn rec(text: &str, confidence: f32) -> RecognizeResult {
    RecognizeResult {
        text: text.into(),
        mean_confidence: confidence,
    }
}

#[test]
fn uses_only_two_high_confidence_agreeing_model_outputs() {
    let calls = Cell::new(0);
    let result = check_short_line_order(
        rec("昇上で", 0.91),
        71,
        22,
        || {
            calls.set(calls.get() + 1);
            Ok(rec("で上昇", 0.99))
        },
        || {
            calls.set(calls.get() + 1);
            Ok(rec("で上昇", 0.999))
        },
    );
    assert_eq!(result.text, "で上昇");
    assert_eq!(result.mean_confidence, 0.999);
    assert_eq!(calls.get(), 2);
}

#[test]
fn skips_ineligible_base_without_calling_models() {
    for (text, score, w, h) in [
        ("で", 0.9, 60, 20),
        ("本文です。", 0.9, 60, 20),
        ("AB", 0.95, 60, 20),
        ("AB", f32::NAN, 60, 20),
        ("AB", -0.1, 60, 20),
        ("A B", 0.9, 60, 20),
        ("A-", 0.9, 60, 20),
        ("ABCDE", 0.9, 60, 20),
        ("AB", 0.9, 39, 20),
        ("AB", 0.9, 121, 20),
        ("AB", 0.9, 20, 60),
        ("AB", 0.9, 0, 0),
        ("AB", 0.9, usize::MAX, usize::MAX),
    ] {
        let result = check_short_line_order(
            rec(text, score),
            w,
            h,
            || panic!("50 must not run"),
            || panic!("100 must not run"),
        );
        assert_eq!(result.text, text);
    }
}

#[test]
fn refuses_non_agreement_edits_low_scores_and_nonfinite_candidates() {
    for (a, b, s, t) in [
        ("BA", "AB", 0.99, 0.99),
        ("BC", "BC", 0.99, 0.99),
        ("BAA", "BAA", 0.99, 0.99),
        ("BA", "BA", 0.979, 0.99),
        ("BA", "BA", 0.99, 0.979),
        ("BA", "BA", f32::NAN, 0.99),
        ("BA", "BA", 0.99, f32::INFINITY),
        ("BA", "BA", 1.01, 0.99),
    ] {
        let result =
            check_short_line_order(rec("AB", 0.91), 60, 20, || Ok(rec(a, s)), || Ok(rec(b, t)));
        assert_eq!(result.text, "AB");
        assert_eq!(result.mean_confidence, 0.91);
    }
}

#[test]
fn handles_duplicate_characters_and_geometry_boundaries() {
    for width in [40, 120] {
        let result = check_short_line_order(
            rec("AAB", 0.94),
            width,
            20,
            || Ok(rec("ABA", 0.98)),
            || Ok(rec("ABA", 1.0)),
        );
        assert_eq!(result.text, "ABA");
    }
    assert_eq!(
        check_short_line_order(
            rec("AAB", 0.9),
            60,
            20,
            || Ok(rec("ABB", 1.0)),
            || Ok(rec("ABB", 1.0))
        )
        .text,
        "AAB"
    );
}

#[test]
fn fallback_failures_preserve_the_original() {
    let result = check_short_line_order(
        rec("AB", 0.9),
        60,
        20,
        || Err(anyhow::anyhow!("model50 failure")),
        || panic!("100 unnecessary after failure"),
    );
    assert_eq!(result.text, "AB");
    let result = check_short_line_order(
        rec("AB", 0.9),
        60,
        20,
        || Ok(rec("BA", 0.99)),
        || Err(anyhow::anyhow!("model100 failure")),
    );
    assert_eq!(result.text, "AB");
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn accepted_output_is_a_real_candidate_with_the_same_character_counts(chars in proptest::collection::vec(prop_oneof![Just('A'),Just('B'),Just('で'),Just('昇')],2..5)) {
        let base: String=chars.iter().collect();
        let reversed: String=chars.iter().rev().collect();
        let out=check_short_line_order(rec(&base,0.9),60,20,||Ok(rec(&reversed,0.99)),||Ok(rec(&reversed,0.995)));
        let mut actual: Vec<_>=out.text.chars().collect();actual.sort_unstable();
        let mut expected=chars;expected.sort_unstable();
        prop_assert_eq!(actual,expected);
        prop_assert!(out.text==base || out.text==reversed);
    }
}

#[test]
fn avoids_the_second_model_when_the_first_cannot_be_selected() {
    for (text, score) in [("AB", 0.999), ("BC", 0.999), ("BA", 0.97)] {
        let result = check_short_line_order(
            rec("AB", 0.9),
            60,
            20,
            || Ok(rec(text, score)),
            || panic!("100 has no eligible consensus partner"),
        );
        assert_eq!(result.text, "AB");
    }
}

/// 意味/語彙を選択条件へ渡さず、事前固定した全96合成cropを診断する。
#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires all pinned PARSeq models and synthetic corpus"]
fn synthetic_short_order_corpus() {
    use crate::{infer::cached::ParseqSession, io::load_rgb_u8};
    let names = [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ];
    let models: Vec<_> = names
        .into_iter()
        .map(|name| {
            ParseqSession::load(
                &std::path::Path::new("models").join(name),
                std::path::Path::new("models/NDLmoji.yaml"),
            )
            .unwrap()
        })
        .collect();
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read("tests/fixtures/short_line_order/manifest.json").unwrap(),
    )
    .unwrap();
    let rows = manifest["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 96);
    let mut results = Vec::new();
    for row in rows {
        let image = load_rgb_u8(std::path::Path::new(row["image"].as_str().unwrap())).unwrap();
        let before = models[0]
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        let calls = Cell::new(0);
        let after = check_short_line_order(
            before.clone(),
            image.width,
            image.height,
            || {
                calls.set(calls.get() + 1);
                models[1].recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            },
            || {
                calls.set(calls.get() + 1);
                models[2].recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            },
        );
        results.push(serde_json::json!({"id":row["id"],"truth":row["truth"],"before":{"text":before.text,"confidence":before.mean_confidence},"after":{"text":after.text,"confidence":after.mean_confidence},"extra_inferences":calls.get()}));
    }
    if let Some(path) = std::env::var_os("NDLOCR_SYNTHETIC_ORDER_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
    fn distance(a: &str, b: &str) -> usize {
        let b: Vec<_> = b.chars().collect();
        let mut previous: Vec<_> = (0..=b.len()).collect();
        for (i, x) in a.chars().enumerate() {
            let mut current = vec![i + 1];
            for (j, &y) in b.iter().enumerate() {
                current.push(
                    (previous[j + 1] + 1)
                        .min(current[j] + 1)
                        .min(previous[j] + usize::from(x != y)),
                );
            }
            previous = current;
        }
        previous[b.len()]
    }
    for row in results {
        let truth = row["truth"].as_str().unwrap();
        let before = row["before"]["text"].as_str().unwrap();
        let after = row["after"]["text"].as_str().unwrap();
        assert!(distance(truth, after) <= distance(truth, before), "{row}");
    }
}
