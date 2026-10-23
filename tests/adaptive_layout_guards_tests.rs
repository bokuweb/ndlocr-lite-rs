use ndlocr_lite_rs::{app::prepare_page_line_detections, infer::deim::Detection};

#[test]
fn duplicated_captions_cannot_select_a_merged_multiline_body() {
    let (w, h) = (320, 290);
    let mut rgb = vec![255; w * h * 3];
    let mut detected = Vec::new();
    for y in [10, 50, 90] {
        for yy in y..y + 22 {
            for x in 20..280 {
                rgb[(yy * w + x) * 3..(yy * w + x) * 3 + 3].fill(0);
            }
        }
        for (x0, x1) in [(20, 280), (40, 100)] {
            detected.push(Detection {
                class_index: 1,
                class_name: "line_main".into(),
                confidence: 0.8,
                box_xyxy: [x0, y as i32, x1, y as i32 + 22],
                pred_char_count: 3.0,
            });
        }
    }
    for y in 150..270 {
        for x in 20..280 {
            rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].fill(0);
        }
    }
    let (actual, reason) = prepare_page_line_detections(&rgb, w, h, detected, 220, 25, 45);
    assert_eq!(reason, "deim");
    assert_eq!(actual.len(), 6);
}
