use ndlocr_lite_rs::app::fallback_line_detections_from_bands;

fn ink(rgb: &mut [u8], width: usize, b: [usize; 4]) {
    for y in b[1]..b[3] {
        for x in b[0]..b[2] {
            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(0);
        }
    }
}

#[test]
fn staggered_columns_do_not_become_one_multiline_region() {
    let (w, h) = (440, 180);
    let mut rgb = vec![255; w * h * 3];
    for y in [10, 50, 90, 130] {
        ink(&mut rgb, w, [20, y, 195, y + 22]);
    }
    for y in [29, 69, 109, 149] {
        ink(&mut rgb, w, [245, y, 420, y + 22]);
    }
    let lines = fallback_line_detections_from_bands(&rgb, w, h, 220, 25, 45);
    assert_eq!(
        lines.len(),
        8,
        "four lines per column, not a page-wide merged band: {lines:?}"
    );
    assert!(lines.iter().all(|l| l.box_xyxy[2] - l.box_xyxy[0] < 200));
    assert_eq!(lines.iter().filter(|l| l.box_xyxy[0] < 220).count(), 4);
    assert_eq!(lines.iter().filter(|l| l.box_xyxy[0] > 220).count(), 4);
}

#[test]
fn a_word_gap_in_one_line_does_not_create_columns() {
    let (w, h) = (440, 60);
    let mut rgb = vec![255; w * h * 3];
    ink(&mut rgb, w, [20, 10, 195, 32]);
    ink(&mut rgb, w, [245, 10, 420, 32]);
    let lines = fallback_line_detections_from_bands(&rgb, w, h, 220, 25, 45);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].box_xyxy[2] - lines[0].box_xyxy[0] > 390);
}

proptest::proptest! {
    #[test]
    fn columns_preserve_all_staggered_lines_and_image_coordinates(offset in 0usize..12, gap in 20usize..55, rows in 3usize..8) {
        let (w,h)=(400+gap, rows*40+offset+25);
        let mut rgb=vec![255;w*h*3];
        for i in 0..rows {
            ink(&mut rgb,w,[10,10+i*40,190,30+i*40]);
            ink(&mut rgb,w,[190+gap,25+offset+i*40,370+gap,45+offset+i*40]);
        }
        let lines=fallback_line_detections_from_bands(&rgb,w,h,220,25,45);
        proptest::prop_assert_eq!(lines.len(),2*rows);
        for line in lines {
            let [x0,y0,x1,y1]=line.box_xyxy;
            proptest::prop_assert!(x0>=0 && y0>=0 && x1<=w as i32 && y1<=h as i32);
            proptest::prop_assert!(x1-x0<200 && y1-y0<30);
        }
    }
}

#[test]
fn spanning_header_keeps_the_legacy_layout() {
    let (w, h) = (440, 180);
    let mut rgb = vec![255; w * h * 3];
    ink(&mut rgb, w, [20, 5, 420, 20]);
    for y in [45, 85, 125] {
        ink(&mut rgb, w, [20, y, 195, y + 22]);
        ink(&mut rgb, w, [245, y, 420, y + 22]);
    }
    let actual = fallback_line_detections_from_bands(&rgb, w, h, 220, 25, 45);
    let expected =
        ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(
        actual
            .iter()
            .map(|l| l.box_xyxy.map(|v| v as usize))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn tall_vertical_regions_do_not_justify_horizontal_columns() {
    let (w, h) = (440, 220);
    let mut rgb = vec![255; w * h * 3];
    ink(&mut rgb, w, [20, 10, 50, 210]);
    ink(&mut rgb, w, [300, 10, 330, 210]);
    let actual = fallback_line_detections_from_bands(&rgb, w, h, 220, 25, 45);
    let expected =
        ndlocr_lite_rs::pipeline::line_segment::detect_textline_bands_naive(&rgb, w, h, 220);
    assert_eq!(
        actual
            .iter()
            .map(|l| l.box_xyxy.map(|v| v as usize))
            .collect::<Vec<_>>(),
        expected
    );
}
