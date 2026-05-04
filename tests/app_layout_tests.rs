use ndlocr_lite_rs::app::{
    build_mock_line_detections, normalize_line_confidence, normalize_line_count, prepare_line_crops,
};
use ndlocr_lite_rs::infer::deim::Detection;
use proptest::prelude::*;

#[test]
fn mock_line_detections_use_inner_margin_and_valid_boxes() {
    let dets = build_mock_line_detections(100, 50, 3, 0.7);
    assert_eq!(dets.len(), 3);
    for d in &dets {
        assert_eq!(d.class_name, "line_main");
        assert!(d.box_xyxy[0] > 0);
        assert!(d.box_xyxy[2] < 100);
        assert!(d.box_xyxy[3] > d.box_xyxy[1]);
        assert!((d.confidence - 0.7).abs() < 1e-6);
    }
}

#[test]
fn mock_line_detections_handle_tiny_images_safely() {
    let dets = build_mock_line_detections(2, 2, 10, 1.0);
    assert!(!dets.is_empty());
    for d in &dets {
        assert!(d.box_xyxy[0] >= 0);
        assert!(d.box_xyxy[1] >= 0);
        assert!(d.box_xyxy[2] > d.box_xyxy[0]);
        assert!(d.box_xyxy[3] > d.box_xyxy[1]);
        assert!(d.box_xyxy[2] <= 2);
        assert!(d.box_xyxy[3] <= 2);
    }
}

#[test]
fn normalize_line_count_clamps_to_valid_range() {
    assert_eq!(normalize_line_count(0, 10), 1);
    assert_eq!(normalize_line_count(3, 10), 3);
    assert_eq!(normalize_line_count(100, 10), 10);
    assert_eq!(normalize_line_count(5, 0), 1);
}

#[test]
fn normalize_line_confidence_clamps_to_valid_range() {
    assert!((normalize_line_confidence(-1.0) - 0.0).abs() < 1e-6);
    assert!((normalize_line_confidence(0.42) - 0.42).abs() < 1e-6);
    assert!((normalize_line_confidence(2.0) - 1.0).abs() < 1e-6);
}

#[test]
fn prepare_line_crops_filters_invalid_boxes_and_preserves_metadata_order() {
    let rgb = vec![255u8; 10 * 10 * 3];
    let detections = vec![
        Detection {
            class_index: 1,
            confidence: 0.7,
            box_xyxy: [2, 2, 5, 4],
            pred_char_count: 3.0,
            class_name: "line_main".to_string(),
        },
        Detection {
            class_index: 1,
            confidence: 0.9,
            box_xyxy: [8, 2, 11, 4],
            pred_char_count: 100.0,
            class_name: "line_main".to_string(),
        },
        Detection {
            class_index: 1,
            confidence: 0.8,
            box_xyxy: [0, 1, 2, 5],
            pred_char_count: 2.0,
            class_name: "line_caption".to_string(),
        },
    ];

    let crops = prepare_line_crops(&rgb, 10, 10, detections, 1).unwrap();

    assert_eq!(crops.len(), 2);
    assert_eq!(crops[0].bbox_xyxy, [1, 1, 6, 5]);
    assert_eq!(crops[0].crop.width, 5);
    assert_eq!(crops[0].crop.height, 4);
    assert!((crops[0].confidence - 0.7).abs() < 1e-6);
    assert_eq!(crops[0].pred_char_count, 3.0);
    assert!(!crops[0].is_vertical);

    assert_eq!(crops[1].bbox_xyxy, [0, 0, 3, 6]);
    assert_eq!(crops[1].crop.width, 3);
    assert_eq!(crops[1].crop.height, 6);
    assert!((crops[1].confidence - 0.8).abs() < 1e-6);
    assert_eq!(crops[1].pred_char_count, 2.0);
    assert!(crops[1].is_vertical);
}

proptest! {
    #[test]
    fn normalize_line_confidence_is_always_in_unit_range(v in any::<f32>()) {
        prop_assume!(v.is_finite());
        let out = normalize_line_confidence(v);
        prop_assert!(out >= 0.0 && out <= 1.0);
    }
}
