use ndlocr_lite_rs::app::prepare_page_line_detections;
use ndlocr_lite_rs::infer::deim::Detection;

fn line(bbox: [i32; 4], confidence: f32) -> Detection {
    Detection {
        class_index: 1,
        class_name: "line_main".into(),
        box_xyxy: bbox,
        confidence,
        pred_char_count: 3.0,
    }
}
fn block(rgb: &mut [u8], w: usize, b: [usize; 4]) {
    for y in b[1]..b[3] {
        for x in b[0]..b[2] {
            rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].fill(0);
        }
    }
}

#[test]
fn partial_deim_does_not_hide_the_other_confirmed_column_rows() {
    let (w, h) = (440, 180);
    let mut rgb = vec![255; w * h * 3];
    for y in [10, 50, 90, 130] {
        block(&mut rgb, w, [20, y, 195, y + 22]);
        block(&mut rgb, w, [245, y + 10, 420, y + 32]);
    }
    let detected = vec![line([20, 10, 195, 32], 0.95)];
    let (actual, reason) = prepare_page_line_detections(&rgb, w, h, detected, 220, 25, 45);
    assert_eq!(actual.len(), 8);
    assert_eq!(reason, "horizontal_columns");
}

#[test]
fn repeated_same_row_fragments_choose_complete_pixel_bands() {
    let (w, h) = (300, 140);
    let mut rgb = vec![255; w * h * 3];
    let mut detected = Vec::new();
    for y in [10, 50, 90] {
        block(&mut rgb, w, [20, y, 280, y + 22]);
        detected.push(line([20, y as i32, 280, y as i32 + 22], 0.8));
        detected.push(line([40, y as i32, 100, y as i32 + 22], 0.7));
    }
    let (actual, reason) = prepare_page_line_detections(&rgb, w, h, detected, 220, 25, 45);
    assert_eq!(actual.len(), 3);
    assert_eq!(reason, "duplicate_horizontal_rows");
}

#[test]
fn unique_lines_and_one_ambiguous_row_keep_deim_identity() {
    let (w, h) = (300, 140);
    let mut rgb = vec![255; w * h * 3];
    let mut detected = Vec::new();
    for y in [10, 50, 90] {
        block(&mut rgb, w, [20, y, 280, y + 22]);
        detected.push(line([20, y as i32, 280, y as i32 + 22], 0.8));
    }
    detected.push(line([40, 10, 100, 32], 0.7));
    let expected: Vec<_> = detected.iter().map(|d| d.box_xyxy).collect();
    let (actual, reason) = prepare_page_line_detections(&rgb, w, h, detected, 220, 25, 45);
    assert_eq!(
        actual.iter().map(|d| d.box_xyxy).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(reason, "deim");
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn repeated_fragment_selection_is_translation_and_scale_invariant(dx in 0usize..12, dy in 0usize..12, scale in 1usize..4) {
        let (w,h)=((300+dx)*scale,(140+dy)*scale);
        let mut rgb=vec![255;w*h*3];let mut detected=Vec::new();
        for y in [10,50,90] {
            let b=[(20+dx)*scale,(y+dy)*scale,(280+dx)*scale,(y+dy+22)*scale];
            block(&mut rgb,w,b);detected.push(line(b.map(|v|v as i32),0.8));
            let c=[(40+dx)*scale,(y+dy)*scale,(100+dx)*scale,(y+dy+22)*scale];
            detected.push(line(c.map(|v|v as i32),0.7));
        }
        let (actual,reason)=prepare_page_line_detections(&rgb,w,h,detected,220,25,45);
        proptest::prop_assert_eq!(actual.len(),3);proptest::prop_assert_eq!(reason,"duplicate_horizontal_rows");
    }
}

#[test]
fn invalid_uncertain_or_nearly_identical_hypotheses_are_not_switch_evidence() {
    for kind in 0..7 {
        let (w, h) = (300, 140);
        let mut rgb = vec![255; w * h * 3];
        let mut detected = Vec::new();
        for y in [10, 50, 90] {
            block(&mut rgb, w, [20, y, 280, y + 22]);
            let mut parent = line([20, y as i32, 280, y as i32 + 22], 0.8);
            let mut child = line([40, y as i32, 100, y as i32 + 22], 0.7);
            match kind {
                0 => parent.confidence = f32::NAN,
                1 => parent.confidence = 1.1,
                2 => parent.confidence = 0.49,
                3 => child.confidence = 0.24,
                4 => child.box_xyxy = parent.box_xyxy,
                5 => child.box_xyxy[1] += 10,
                _ => child.class_name = "text_block".into(),
            }
            detected.extend([parent, child]);
        }
        let (actual, reason) = prepare_page_line_detections(&rgb, w, h, detected, 220, 25, 45);
        assert_eq!(reason, "deim", "kind {kind}");
        assert_eq!(actual.len(), 6);
    }
}
