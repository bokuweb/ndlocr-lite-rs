use std::collections::HashSet;

use ndlocr_lite_rs::postprocess::morph_correction::{CorrectionToken, correct_unknown_tokens_with};

#[test]
fn corrects_unknown_ocr_confusion_when_candidate_is_known() {
    let known = HashSet::from(["テスト".to_string()]);
    let out = correct_unknown_tokens_with(
        "これはテス卜です",
        &[
            CorrectionToken::known("これは", 0..9),
            CorrectionToken::unknown("テス卜", 9..18),
            CorrectionToken::known("です", 18..24),
        ],
        |candidate| known.contains(candidate),
    );

    assert_eq!(out, "これはテストです");
}

#[test]
fn corrects_unknown_span_collapse_when_candidate_is_known() {
    let known = HashSet::from(["権利".to_string()]);
    let out = correct_unknown_tokens_with(
        "権禾リ",
        &[CorrectionToken::unknown("権禾リ", 0.."権禾リ".len())],
        |candidate| known.contains(candidate),
    );

    assert_eq!(out, "権利");
}

#[test]
fn keeps_unknown_token_when_no_candidate_is_known() {
    let out =
        correct_unknown_tokens_with("カ士", &[CorrectionToken::unknown("カ士", 0..6)], |_| false);

    assert_eq!(out, "カ士");
}

#[test]
fn does_not_touch_known_tokens_even_if_confusable() {
    let out = correct_unknown_tokens_with(
        "テス卜",
        &[CorrectionToken::known("テス卜", 0..9)],
        |candidate| candidate == "テスト",
    );

    assert_eq!(out, "テス卜");
}
