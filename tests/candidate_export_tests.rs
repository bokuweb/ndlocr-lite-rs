#[path = "../examples/support/candidate_export.rs"]
mod export;
use export::ExportLine;
use ndlocr_lite_rs::infer::parseq::predict_text_from_flat_logits_with_candidates;

/// 検出で欠落した行にもIDを残し、候補を捏造しない。
#[test]
fn missing_line_keeps_empty_prediction() {
    let line = serde_json::to_value(ExportLine::missing("footer".into())).unwrap();
    assert_eq!(line["id"], "footer");
    assert_eq!(line["bounds"], serde_json::Value::Null);
    assert_eq!(line["text"], "");
    assert_eq!(line["positions"], serde_json::json!([]));
    assert_eq!(line["chunks"], serde_json::json!([]));
}

/// 区間連結の空白だけを固定し、Unicode本文と停止候補を保持する。
#[test]
fn export_keeps_raw_steps_and_fixed_join_spaces() {
    let result = predict_text_from_flat_logits_with_candidates(
        &[0., 8., 0., 0., 0., 8., 8., 0., 0.],
        3,
        3,
        &['あ', '１'],
        3,
    )
    .unwrap();
    let mut line = ExportLine::new("line".into(), [0, 0, 40, 20]);
    line.append(result.clone(), false, [0, 20]).unwrap();
    line.append(result, true, [20, 40]).unwrap();
    let value = serde_json::to_value(line).unwrap();
    assert_eq!(value["text"], "あ１ あ１");
    assert_eq!(value["positions"].as_array().unwrap().len(), 5);
    assert_eq!(value["positions"][2]["character"], " ");
    assert_eq!(
        value["positions"][2]["candidates"],
        serde_json::json!([" "])
    );
    assert_eq!(
        value["positions"][0]["candidates"],
        serde_json::json!(["あ", null, "１"])
    );
    assert_eq!(
        value["chunks"][0]["steps"][1]["byte_range"],
        serde_json::json!([3, 6])
    );
    assert_eq!(
        value["chunks"][0]["steps"][2]["byte_range"],
        serde_json::Value::Null
    );
    assert_eq!(value["chunks"][0]["stop_reason"], "EndOfSequence");
    assert_eq!(
        value["chunks"][0]["steps"][0]["candidates"][0]["probability"]
            .as_f64()
            .unwrap() as f32,
        value["chunks"][1]["steps"][0]["candidates"][0]["probability"]
            .as_f64()
            .unwrap() as f32
    );
}

/// 候補と本文範囲が食い違う結果を評価用exportへ通さない。
#[test]
fn invalid_step_mapping_is_rejected() {
    let mut result =
        predict_text_from_flat_logits_with_candidates(&[0., 8.], 1, 2, &['あ'], 2).unwrap();
    result.steps[0].text_range = Some(0..1);
    let mut line = ExportLine::new("line".into(), [0, 0, 10, 10]);
    assert!(line.append(result, false, [0, 10]).is_err());
}
