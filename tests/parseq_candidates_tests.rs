use ndlocr_lite_rs::infer::parseq::{
    ParseqStopReason, predict_text_from_flat_logits_with_candidates as candidates,
    predict_text_from_flat_logits_with_confidence as legacy,
};
use proptest::prelude::*;

/// 上位候補を全クラスの確率で返し、反復・空白・Unicode範囲・EOSを保持する。
#[test]
fn candidates_keep_raw_text_full_softmax_and_eos() {
    let row = [0.0, 3.0, 2.0, 1.0];
    let mut flat = row.repeat(2);
    flat.extend_from_slice(&[0.0, 0.0, 0.0, 5.0]);
    flat.extend_from_slice(&[5.0, 0.0, 0.0, 0.0]);
    flat.extend_from_slice(&[0.0, 9.0, 0.0, 0.0]);
    let actual = candidates(&flat, 5, 4, &['あ', '１', ' '], 2).unwrap();
    let original = legacy(&flat, 5, 4, &['あ', '１', ' ']).unwrap();
    assert_eq!(actual.recognition.text, "ああ ");
    assert_eq!(actual.recognition.text, original.text);
    assert_eq!(actual.recognition.mean_confidence, original.mean_confidence);
    assert_eq!(actual.stop_reason, ParseqStopReason::EndOfSequence);
    assert_eq!(actual.steps.len(), 4);
    assert_eq!(actual.steps[0].text_range, Some(0..3));
    assert_eq!(actual.steps[1].text_range, Some(3..6));
    assert_eq!(actual.steps[2].text_range, Some(6..7));
    assert_eq!(actual.steps[3].text_range, None);
    let first = &actual.steps[0].candidates;
    assert_eq!(first.iter().map(|c| c.token_id).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(first[0].character, Some('あ'));
    assert_eq!(first[1].character, Some('１'));
    let denominator: f32 = row.iter().map(|x| (*x - 3.0_f32).exp()).sum();
    assert!((first[0].probability - denominator.recip()).abs() < 1e-6);
    assert!(first.iter().map(|c| c.probability).sum::<f32>() < 0.99);
    let eos = &actual.steps[3].candidates[0];
    assert_eq!((eos.token_id, eos.character), (0, None));
}

/// 低信頼度で打ち切った位置も返し、後続の文字を捏造しない。
#[test]
fn low_confidence_step_is_visible_without_emitting_text() {
    let flat = [0.0, 5.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.1, 0.09, 0.0, 8.0, 0.0];
    let actual = candidates(&flat, 4, 3, &['あ', 'い'], 3).unwrap();
    assert_eq!(actual.recognition.text, "あい");
    assert_eq!(actual.stop_reason, ParseqStopReason::LowConfidence);
    assert_eq!(actual.steps.len(), 3);
    assert_eq!(actual.steps[2].timestep, 2);
    assert_eq!(actual.steps[2].text_range, None);
    assert!(actual.steps[2].candidates[0].probability < 0.35);
    assert_eq!(
        actual.recognition.mean_confidence,
        legacy(&flat, 4, 3, &['あ', 'い']).unwrap().mean_confidence
    );
}

/// 同率は小さいIDを先にし、巨大なkはクラス数に制限する。
#[test]
fn ties_limits_extreme_logits_and_exhaustion_are_defined() {
    let actual = candidates(
        &[f32::MAX, f32::MAX, -f32::MAX],
        1,
        3,
        &['あ', 'い'],
        usize::MAX,
    )
    .unwrap();
    assert_eq!(
        actual.steps[0]
            .candidates
            .iter()
            .map(|c| c.token_id)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(actual.steps[0].candidates[0].probability, 0.5);
    assert_eq!(actual.steps[0].candidates[2].probability, 0.0);
    assert_eq!(actual.stop_reason, ParseqStopReason::EndOfSequence);
    let exhausted = candidates(&[0.0, 2.0, 2.0], 1, 3, &['あ', 'い'], 1).unwrap();
    assert_eq!(exhausted.recognition.text, "あ");
    assert_eq!(exhausted.stop_reason, ParseqStopReason::TimestepsExhausted);
}

/// 候補スコアに使えない入力は失敗し、形状外データだけは従来どおり無視する。
#[test]
fn invalid_candidate_inputs_fail_before_decoding() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(candidates(&[0.0, bad], 1, 2, &['あ'], 1).is_err());
        assert!(candidates(&[8.0, 0.0, bad, 0.0], 2, 2, &['あ'], 1).is_err());
    }
    assert!(candidates(&[], 0, 2, &['あ'], 1).is_err());
    assert!(candidates(&[], 1, 0, &[], 1).is_err());
    assert!(candidates(&[], usize::MAX, 2, &['あ'], 1).is_err());
    assert!(candidates(&[0.0], 1, 2, &['あ'], 1).is_err());
    assert!(candidates(&[0.0, 2.0], 1, 2, &[], 1).is_err());
    assert!(candidates(&[0.0, 2.0], 1, 2, &['あ', 'い'], 1).is_err());
    assert!(candidates(&[0.0, 2.0], 1, 2, &['あ'], 0).is_err());
    assert_eq!(
        candidates(&[0.0, 2.0, f32::NAN], 1, 2, &['あ'], 1)
            .unwrap()
            .recognition
            .text,
        "あ"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    /// 候補の順位・確率・範囲を検証し、候補数や定数シフトで既存本文を変えない。
    #[test]
    fn candidate_observation_preserves_legacy_decoding(
        rows in prop::collection::vec(prop::collection::vec(-10_i16..11, 5), 1..10),
        k in 1usize..9, shift in -20_i16..21,
    ) {
        let flat: Vec<f32> = rows.iter().flatten().map(|v| *v as f32).collect();
        let shifted: Vec<f32> = flat.iter().map(|v| v + shift as f32).collect();
        let charset = ['あ', '１', ' ', 'a'];
        let old = legacy(&flat, rows.len(), 5, &charset).unwrap();
        let actual = candidates(&flat, rows.len(), 5, &charset, k).unwrap();
        let other = candidates(&shifted, rows.len(), 5, &charset, k).unwrap();
        prop_assert_eq!(&actual.recognition.text, &old.text);
        prop_assert_eq!(actual.recognition.mean_confidence, old.mean_confidence);
        prop_assert_eq!(&actual.recognition.text, &other.recognition.text);
        prop_assert_eq!(actual.stop_reason, other.stop_reason);
        let mut emitted = String::new();
        for (index, step) in actual.steps.iter().enumerate() {
            prop_assert_eq!(step.timestep, index);
            prop_assert_eq!(step.candidates.len(), k.min(5));
            let sum: f32 = step.candidates.iter().map(|c| c.probability).sum();
            prop_assert!(sum <= 1.00001);
            for (a, b) in step.candidates.iter().zip(&other.steps[index].candidates) {
                prop_assert_eq!(a.token_id, b.token_id);
                prop_assert!((a.probability - b.probability).abs() < 1e-5);
            }
            let winner = &step.candidates[0];
            for pair in step.candidates.windows(2) {
                prop_assert!(pair[0].probability >= pair[1].probability);
                if pair[0].probability == pair[1].probability {
                    prop_assert!(pair[0].token_id < pair[1].token_id);
                }
            }
            if let Some(range) = &step.text_range {
                let ch = winner.character.unwrap();
                prop_assert_eq!(range.start, emitted.len());
                emitted.push(ch);
                prop_assert_eq!(range.end, emitted.len());
                prop_assert_eq!(&actual.recognition.text[range.clone()], ch.to_string());
            } else {
                prop_assert_eq!(index + 1, actual.steps.len());
            }
        }
        prop_assert_eq!(emitted, actual.recognition.text);
    }
}
