use anyhow::{Result, bail};
use ndlocr_lite_rs::infer::{
    margin_candidates::recognize_horizontal_margin_candidates, parseq::RecognizeResult,
};
use proptest::prelude::*;

/// 幅100/高8の中央にRGB列を置き、確実に縮小対象を作る。
fn image() -> Vec<u8> {
    let mut rgb = vec![255; 100 * 8 * 3];
    for y in 0..8 {
        rgb[(y * 100 + 45) * 3..(y * 100 + 45) * 3 + 3].copy_from_slice(&[10, 20, 30]);
    }
    rgb
}
/// 本文をそのまま返すテスト用結果。
fn result(text: &str, confidence: f32) -> RecognizeResult {
    RecognizeResult {
        text: text.into(),
        mean_confidence: confidence,
    }
}
/// 高信頼度の候補を本文の修正なしに採り、元画像を必ず先に認識する。
#[test]
fn chooses_compact_and_preserves_both_outputs() {
    let mut seen = Vec::new();
    let out = recognize_horizontal_margin_candidates(&image(), 100, 8, |rgb, w, h| {
        seen.push((w, h, rgb.to_vec()));
        Ok(if w == 100 {
            result("original.", 0.5)
        } else {
            result("候補..", 0.8)
        })
    })
    .unwrap();
    assert_eq!(
        seen.iter().map(|v| (v.0, v.1)).collect::<Vec<_>>(),
        [(100, 8), (5, 8)]
    );
    assert_eq!(seen[0].2, image());
    for row in seen[1].2.chunks_exact(15) {
        assert_eq!(&row[6..9], &[10, 20, 30]);
    }
    assert_eq!(out.original().text, "original.");
    assert_eq!(out.compact().unwrap().text, "候補..");
    assert_eq!(out.selected().text, "候補..");
}
/// 同点と低信頼度なら原画像の本文を保持する。
#[test]
fn original_wins_ties_or_lower_compact_confidence() {
    for confidence in [0.0, 0.5, 0.75] {
        let out = recognize_horizontal_margin_candidates(&image(), 100, 8, |_, w, _| {
            Ok(result(
                if w == 100 { "元" } else { "縮小" },
                if w == 100 { 0.75 } else { confidence },
            ))
        })
        .unwrap();
        assert_eq!(out.selected().text, "元");
        assert!(out.compact().is_some());
    }
}
/// 密な入力・白紙・短い横幅では追加推論しない。
#[test]
fn unchanged_or_narrow_images_use_one_call() {
    for (rgb, w, h) in [
        (vec![255; 100 * 8 * 3], 100, 8),
        (vec![0; 100 * 8 * 3], 100, 8),
        (vec![255; 10 * 8 * 3], 10, 8),
        (vec![0; 3], 1, 1),
    ] {
        let mut calls = 0;
        let out = recognize_horizontal_margin_candidates(&rgb, w, h, |data, width, height| {
            calls += 1;
            assert_eq!((data, width, height), (rgb.as_slice(), w, h));
            Ok(result("同じ", 1.0))
        })
        .unwrap();
        assert_eq!(calls, 1);
        assert!(out.compact().is_none());
    }
}
/// 不正RGB・空寸法・積overflowはcallbackを呼ぶ前に拒否する。
#[test]
fn invalid_inputs_do_not_infer() {
    for (rgb, w, h) in [
        (&[][..], 0, 1),
        (&[][..], 1, 0),
        (&[1][..], 1, 1),
        (&[][..], usize::MAX, 2),
    ] {
        let mut calls = 0;
        let out = recognize_horizontal_margin_candidates(rgb, w, h, |_, _, _| {
            calls += 1;
            Ok(result("bad", 1.0))
        });
        assert!(out.is_err());
        assert_eq!(calls, 0);
    }
}
/// 原画像と候補の推論失敗は元の内容で伝播し、次を実行しない。
#[test]
fn inference_failures_are_propagated() {
    for fail_at in [1, 2] {
        let mut calls = 0;
        let out =
            recognize_horizontal_margin_candidates(&image(), 100, 8, |_, _, _| -> Result<_> {
                calls += 1;
                if calls == fail_at {
                    bail!("inference failure {fail_at}")
                }
                Ok(result("ok", 0.5))
            });
        assert_eq!(
            out.unwrap_err().to_string(),
            format!("inference failure {fail_at}")
        );
        assert_eq!(calls, fail_at);
    }
}
/// NaN/無限大・範囲外信頼度は成功候補として扱わない。
#[test]
fn invalid_confidences_are_rejected() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01] {
        for bad_call in [1, 2] {
            let mut calls = 0;
            let out = recognize_horizontal_margin_candidates(&image(), 100, 8, |_, _, _| {
                calls += 1;
                Ok(result("x", if calls == bad_call { invalid } else { 0.5 }))
            });
            assert!(out.is_err());
            assert_eq!(calls, bad_call);
        }
    }
}
proptest! {
    /// 独立な大小比較と候補選択・2呼出しを有限信頼度全域で照合する。
    #[test]
    fn confidence_selection_matches_oracle(original in 0u16..=1000, compact in 0u16..=1000) {
        let mut calls=0;
        let out=recognize_horizontal_margin_candidates(&image(),100,8,|_,w,_| {calls+=1;Ok(result(if w==100 {"o"} else {"c"},f32::from(if w==100 {original} else {compact})/1000.0))}).unwrap();
        prop_assert_eq!(calls,2);
        prop_assert_eq!(&out.selected().text,if compact>original {"c"} else {"o"});
    }
}
