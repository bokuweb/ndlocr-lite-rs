use ndlocr_lite_rs::infer::deim::Detection;
use ndlocr_lite_rs::pipeline::connect::recognize_line_detections_with_cascade;

#[test]
fn connect_filters_non_line_and_runs_cascade() {
    let rgb = vec![
        255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0,
    ];
    let detections = vec![
        Detection {
            class_index: 0,
            confidence: 0.9,
            box_xyxy: [0, 0, 2, 1],
            pred_char_count: 3.0,
            class_name: "line_main".into(),
        },
        Detection {
            class_index: 7,
            confidence: 0.9,
            box_xyxy: [0, 0, 1, 1],
            pred_char_count: 100.0,
            class_name: "block_ad".into(),
        },
        Detection {
            class_index: 1,
            confidence: 0.9,
            box_xyxy: [1, 0, 3, 2],
            pred_char_count: 2.0,
            class_name: "line_caption".into(),
        },
    ];
    let out = recognize_line_detections_with_cascade(
        &rgb,
        3,
        2,
        &detections,
        |_| "x".repeat(25),
        |_| "y".repeat(45),
        |_| "ok".into(),
    )
    .unwrap();
    assert_eq!(out, vec!["ok", "ok"]);
}
