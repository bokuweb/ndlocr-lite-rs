use ndlocr_lite_rs::app::{
    build_mock_line_detections, normalize_line_confidence, normalize_line_count,
};
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

proptest! {
    #[test]
    fn normalize_line_confidence_is_always_in_unit_range(v in any::<f32>()) {
        prop_assume!(v.is_finite());
        let out = normalize_line_confidence(v);
        prop_assert!(out >= 0.0 && out <= 1.0);
    }
}
