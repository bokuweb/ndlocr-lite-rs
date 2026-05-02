use ndlocr_lite_rs::infer::deim::Detection;
use ndlocr_lite_rs::pipeline::run_page::{PageInput, run_page};

#[test]
fn run_page_returns_counts_and_texts() {
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
            class_index: 1,
            confidence: 0.9,
            box_xyxy: [1, 0, 3, 2],
            pred_char_count: 2.0,
            class_name: "line_caption".into(),
        },
        Detection {
            class_index: 7,
            confidence: 0.9,
            box_xyxy: [0, 0, 1, 1],
            pred_char_count: 100.0,
            class_name: "block_ad".into(),
        },
    ];
    let out = run_page(
        PageInput {
            rgb: &rgb,
            width: 3,
            height: 2,
            detections: &detections,
        },
        |_| "x".repeat(25),
        |_| "y".repeat(45),
        |_| "ok".into(),
    )
    .unwrap();
    assert_eq!(out.total_detection_count, 3);
    assert_eq!(out.line_detection_count, 2);
    assert_eq!(out.texts, vec!["ok", "ok"]);
    assert_eq!(out.lines[0].bbox_xyxy, [0, 0, 2, 1]);
}

#[test]
fn run_page_sorts_texts_in_reading_order() {
    let rgb = vec![255; 3 * 4 * 3];
    let detections = vec![
        Detection {
            class_index: 0,
            confidence: 0.9,
            box_xyxy: [0, 2, 3, 3],
            pred_char_count: 3.0,
            class_name: "line_main".into(),
        },
        Detection {
            class_index: 0,
            confidence: 0.9,
            box_xyxy: [0, 0, 3, 1],
            pred_char_count: 3.0,
            class_name: "line_main".into(),
        },
    ];
    let out = run_page(
        PageInput {
            rgb: &rgb,
            width: 3,
            height: 4,
            detections: &detections,
        },
        |_| "x".repeat(25),
        |_| "y".repeat(45),
        |_| "ok".to_string(),
    )
    .unwrap();
    assert_eq!(out.lines[0].bbox_xyxy, [0, 0, 3, 1]);
    assert_eq!(out.lines[1].bbox_xyxy, [0, 2, 3, 3]);
}
