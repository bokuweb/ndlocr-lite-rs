use ndlocr_lite_rs::infer::{deim, parseq};
use proptest::prelude::*;

#[test]
fn deim_build_detections_filters_and_scales() {
    let dets = deim::build_detections(
        &[1, 2],
        &[[10.0, 20.0, 30.0, 40.0], [1.0, 2.0, 3.0, 4.0]],
        &[0.9, 0.1],
        None,
        &["text_block".to_string(), "line_main".to_string()],
        0.2,
        deim::ScaleContext {
            image_width: 200,
            image_height: 100,
            input_width: 100,
            input_height: 100,
        },
    );
    assert_eq!(dets.len(), 1);
    assert_eq!(dets[0].box_xyxy, [20, 20, 60, 40]);
}

#[test]
fn parseq_argmax_and_decode_produces_text() {
    let logits = vec![vec![0.1, 0.9, 0.2], vec![0.8, 0.1, 0.1]];
    let t = parseq::predict_text_from_logits(&logits, &['あ', 'い']).unwrap();
    assert_eq!(t, "あ");
}

#[test]
fn parseq_flat_logits_and_nested_logits_match() {
    let logits = vec![
        vec![0.1, 0.9, 0.2],
        vec![0.8, 0.1, 0.1],
        vec![0.0, 3.0, 0.2],
    ];
    let nested = parseq::predict_text_from_logits(&logits, &['あ', 'い']).unwrap();
    let flat = logits.iter().flatten().copied().collect::<Vec<_>>();
    let fast = parseq::predict_text_from_flat_logits(&flat, 3, 3, &['あ', 'い']).unwrap();
    assert_eq!(fast, nested);
}

#[test]
fn parseq_sanitize_reduces_repeated_noise() {
    let s = "秘密保持契約書 1111111111 ((((（（（";
    let out = parseq::sanitize_recognized_text(s);
    assert_eq!(out, "秘密保持契約書 1111 (((（（（");
}

#[test]
fn parseq_sanitize_collapses_repeated_phrases() {
    let s = "株式会社Y(乙」という。)という。)という。)とする。";
    let out = parseq::sanitize_recognized_text(s);
    assert_eq!(out, "株式会社Y(乙という。)という。とする。");
}

#[test]
fn parseq_sanitize_removes_unmatched_closing_brackets() {
    let s = "株式会社Y(乙」という。)という。)」";
    let out = parseq::sanitize_recognized_text(s);
    assert_eq!(out, "株式会社Y(乙という。)という。");
}

#[test]
fn parseq_sanitize_collapses_repeated_common_phrases() {
    let s = "Aという。という。という。B";
    let out = parseq::sanitize_recognized_text(s);
    assert_eq!(out, "Aという。B");
}

#[test]
fn parseq_sanitize_applies_common_japanese_ocr_replacements() {
    let s = "単又は乙が調示を受けた後、圖示された情報";
    let out = parseq::sanitize_recognized_text(s);
    assert_eq!(out, "甲又は乙が開示を受けた後、開示された情報");
}

#[test]
fn parseq_flat_logits_stops_on_low_confidence_tail() {
    // classes: [EOS, 'あ', 'い']
    let flat = vec![
        0.0, 3.0, 0.2, // 'あ' (high confidence)
        0.0, 0.1, 3.0, // 'い' (high confidence)
        0.0, 0.1, 0.09, // low-confidence non-EOS token -> should stop
        5.0, 0.0, 0.0, // EOS (would appear later, but we stop earlier)
    ];
    let out =
        parseq::predict_text_from_flat_logits_with_confidence(&flat, 4, 3, &['あ', 'い']).unwrap();
    assert_eq!(out.text, "あい");
    assert!(out.mean_confidence > 0.5 && out.mean_confidence <= 1.0);
}

proptest! {
    #[test]
    fn parseq_argmax_is_invariant_to_constant_shift(
        row in proptest::collection::vec(-100.0f32..100.0, 2..16),
        c in -1000.0f32..1000.0f32
    ) {
        let base = vec![row.clone()];
        let shifted = vec![row.into_iter().map(|v| v+c).collect::<Vec<_>>()];
        prop_assert_eq!(parseq::argmax_token_ids(&base).unwrap(), parseq::argmax_token_ids(&shifted).unwrap());
    }
}
