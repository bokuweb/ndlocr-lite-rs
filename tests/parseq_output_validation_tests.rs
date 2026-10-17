use ndlocr_lite_rs::infer::parseq::{
    predict_text_from_flat_logits as text,
    predict_text_from_flat_logits_with_confidence as recognize,
};
use proptest::prelude::*;

/// 不正なモデル出力を正常な空文字や文字欠落として返さない。
#[test]
fn rejects_nonfinite_logits_including_after_eos() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(recognize(&[0.0, bad], 1, 2, &['あ']).is_err());
        assert!(text(&[0.0, bad], 1, 2, &['あ']).is_err());
        assert!(recognize(&[8.0, 0.0, bad, 0.0], 2, 2, &['あ']).is_err());
        let after_low_confidence = [0.0, 8.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.1, 0.09, bad, 0.0, 0.0];
        assert!(recognize(&after_low_confidence, 4, 3, &['あ', 'い']).is_err());
    }
}

/// 未知クラスを捨てたり、余分な文字集合を黙認したりしない。
#[test]
fn rejects_charset_size_mismatch() {
    assert!(recognize(&[0.0, 8.0], 1, 2, &[]).is_err());
    assert!(text(&[0.0, 8.0], 1, 2, &[]).is_err());
    assert!(recognize(&[0.0, 8.0], 1, 2, &['あ', 'い']).is_err());
}

/// 不正形状を拒否し、有効な形状の外側は読み取らない。
#[test]
fn shape_boundaries_and_ignored_tail_are_preserved() {
    assert!(recognize(&[], 0, 2, &['あ']).is_err());
    assert!(recognize(&[], 1, 0, &[]).is_err());
    assert!(recognize(&[], usize::MAX, 2, &['あ']).is_err());
    assert!(recognize(&[0.0], 1, 2, &['あ']).is_err());
    let actual = recognize(&[-f32::MAX, f32::MAX, f32::NAN], 1, 2, &['あ']).unwrap();
    assert_eq!(actual.text, "あ");
    assert_eq!(actual.mean_confidence, 1.0);
    let eos_only = recognize(&[0.0], 1, 1, &[]).unwrap();
    assert_eq!(eos_only.text, "");
    assert_eq!(eos_only.mean_confidence, 0.0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// EOSや低信頼度停止の位置に依存せず形状内の非有限値を拒否する。
    #[test]
    fn any_in_shape_nonfinite_value_is_rejected(
        rows in prop::collection::vec(prop::array::uniform3(-1000_f32..1000_f32), 1..20),
        position in any::<usize>(),
        bad in prop::sample::select(vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY]),
    ) {
        let mut logits: Vec<f32> = rows.iter().flatten().copied().collect();
        let valid = recognize(&logits, rows.len(), 3, &['あ', ' ']).unwrap();
        prop_assert!(valid.mean_confidence.is_finite());
        prop_assert!((0.0..=1.0).contains(&valid.mean_confidence));
        let index = position % logits.len();
        logits[index] = bad;
        prop_assert!(recognize(&logits, rows.len(), 3, &['あ', ' ']).is_err());
    }
}
