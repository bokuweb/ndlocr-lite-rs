use ndlocr_lite_rs::pipeline::crop::{BBox, crop_rgb_u8, expand_bbox_xyxy_clamped};

#[test]
fn crop_rgb_u8_extracts_requested_region() {
    let rgb = vec![
        255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0,
    ];
    let out = crop_rgb_u8(&rgb, 3, 2, BBox::new(1, 0, 3, 2)).unwrap();
    assert_eq!(out.width, 2);
    assert_eq!(out.height, 2);
}

#[test]
fn expand_bbox_xyxy_clamped_adds_padding_and_clamps() {
    assert_eq!(
        expand_bbox_xyxy_clamped(2, 2, 4, 4, 1, 10, 10),
        (1, 1, 5, 5)
    );
    assert_eq!(expand_bbox_xyxy_clamped(0, 0, 3, 3, 2, 4, 4), (0, 0, 4, 4));
    assert_eq!(
        expand_bbox_xyxy_clamped(1, 1, 2, 2, 0, 10, 10),
        (1, 1, 2, 2)
    );
}
