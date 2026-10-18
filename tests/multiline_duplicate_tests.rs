use ndlocr_lite_rs::{
    app::prepare_line_crops,
    infer::deim::Detection,
    pipeline::run_page::{PageInput, run_page},
};

fn line(bounds: [i32; 4], confidence: f32) -> Detection {
    Detection {
        class_index: 1,
        class_name: "line_main".into(),
        box_xyxy: bounds,
        confidence,
        pred_char_count: 3.0,
    }
}

fn sample(scale: usize, margin: usize, vertical: bool) -> (Vec<u8>, usize, usize, Vec<Detection>) {
    let (width, height) = (400 * scale + 2 * margin, 120 * scale + 2 * margin);
    let mut rgb = vec![255; width * height * 3];
    for y0 in [22, 62] {
        for y in y0 * scale + margin..(y0 + 16) * scale + margin {
            for x in 24 * scale + margin..376 * scale + margin {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
    }
    let mut ds = vec![
        line([20, 20, 380, 40], 0.9),
        line([20, 60, 380, 80], 0.92),
        line([20, 20, 380, 80], 0.3),
    ];
    for d in &mut ds {
        for v in &mut d.box_xyxy {
            *v = *v * scale as i32 + margin as i32;
        }
    }
    if vertical {
        let mut transposed = vec![255; rgb.len()];
        for y in 0..height {
            for x in 0..width {
                transposed[(x * height + y) * 3..(x * height + y + 1) * 3]
                    .copy_from_slice(&rgb[(y * width + x) * 3..(y * width + x + 1) * 3]);
            }
        }
        for d in &mut ds {
            let [a, b, c, e] = d.box_xyxy;
            d.box_xyxy = [b, a, e, c];
        }
        (transposed, height, width, ds)
    } else {
        (rgb, width, height, ds)
    }
}

#[test]
fn two_strong_rows_remove_only_the_weak_multiline_region() {
    let (rgb, w, h, ds) = sample(1, 0, false);
    let crops = prepare_line_crops(&rgb, w, h, ds, 0).unwrap();
    assert_eq!(crops.len(), 2);
    assert_eq!(crops[0].bbox_xyxy, [20, 20, 380, 40]);
    assert_eq!(crops[1].bbox_xyxy, [20, 60, 380, 80]);
    assert_eq!((crops[0].confidence, crops[1].confidence), (0.9, 0.92));
}

#[test]
fn cascade_connection_recognizes_each_supported_row_once() {
    let (rgb, w, h, ds) = sample(1, 0, false);
    let out = run_page(
        PageInput {
            rgb: &rgb,
            width: w,
            height: h,
            detections: &ds,
        },
        |_| "same physical text".into(),
        |_| panic!("unexpected model"),
        |_| panic!("unexpected model"),
    )
    .unwrap();
    assert_eq!(out.total_detection_count, 3);
    assert_eq!(out.line_detection_count, 2);
    assert_eq!(
        out.texts.len(),
        2,
        "distinct physical rows may have equal recognized text"
    );
}

#[test]
fn uncertain_or_incomplete_support_keeps_the_candidate() {
    for case in 0..10 {
        let (mut rgb, w, h, mut ds) = sample(1, 0, false);
        match case {
            0 => {
                ds[2].confidence = 0.46;
            }
            1 => {
                ds.remove(1);
            }
            2 => {
                rgb[(50 * w + 100) * 3..(50 * w + 101) * 3].fill(0);
            }
            3 => {
                ds[1].box_xyxy[0] = 220;
            }
            4 => {
                ds[1].box_xyxy[1] = 30;
                ds[1].box_xyxy[3] = 50;
            }
            5 => {
                rgb.fill(255);
            }
            6 => {
                ds[1].box_xyxy[2] = 401;
            }
            7 => {
                ds[1].confidence = f32::NAN;
            }
            8 => rgb[(50 * w + 100) * 3] = 254,
            9 => rgb[(50 * w + 100) * 3..(50 * w + 101) * 3].copy_from_slice(&[255, 0, 255]),
            _ => unreachable!(),
        }
        let target = ds.last().unwrap().box_xyxy;
        let crops = prepare_line_crops(&rgb, w, h, ds, 0).unwrap();
        assert!(crops.iter().any(|c| c.bbox_xyxy == target), "case={case}");
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    #[test]
    fn suppression_preserves_winners_at_scale(scale in 1usize..4,margin in 0usize..10,vertical in proptest::bool::ANY,reverse in proptest::bool::ANY) {
        let (rgb,w,h,mut ds)=sample(scale,margin,vertical);
        if reverse {ds.reverse();}
        let expected:Vec<_>=ds.iter().filter(|d|d.confidence>0.8).map(|d|d.box_xyxy).collect();
        let crops=prepare_line_crops(&rgb,w,h,ds,0).unwrap();
        proptest::prop_assert_eq!(crops.iter().map(|c|c.bbox_xyxy).collect::<Vec<_>>(),expected);
    }
}

#[test]
fn exact_confidence_and_line_gap_boundaries_are_respected() {
    for (confidence, gap, expected) in [
        (0.45, 20, 3),
        (0.449, 20, 2),
        (0.3, 4, 3),
        (0.3, 5, 2),
        (0.3, 21, 3),
    ] {
        let (mut rgb, w, h, mut ds) = sample(1, 0, false);
        rgb.fill(255);
        ds[1].box_xyxy = [20, 40 + gap, 380, 60 + gap];
        ds[2].box_xyxy = [20, 20, 380, 60 + gap];
        ds[2].confidence = confidence;
        for d in &ds[..2] {
            let [x0, y0, x1, y1] = d.box_xyxy.map(|n| n as usize);
            for y in y0 + 2..y1 - 2 {
                for x in x0 + 4..x1 - 4 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
        assert_eq!(
            prepare_line_crops(&rgb, w, h, ds, 0).unwrap().len(),
            expected,
            "confidence={confidence} gap={gap}"
        );
    }
}

#[test]
fn suppressed_region_cannot_suppress_another_candidate() {
    let (w, h) = (400, 200);
    let mut rgb = vec![255; w * h * 3];
    let ds = vec![
        line([20, 40, 380, 60], 0.9),
        line([20, 80, 380, 100], 0.92),
        line([20, 120, 380, 180], 0.8),
        line([20, 40, 380, 100], 0.3),
        line([20, 40, 380, 180], 0.1),
    ];
    for d in &ds[..3] {
        let [x0, y0, x1, y1] = d.box_xyxy.map(|n| n as usize);
        for y in y0 + 2..y1 - 2 {
            for x in x0 + 4..x1 - 4 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    let crops = prepare_line_crops(&rgb, w, h, ds, 0).unwrap();
    assert_eq!(crops.len(), 4);
    assert_eq!(crops.last().unwrap().bbox_xyxy, [20, 40, 380, 180]);
    assert!(!crops.iter().any(|c| c.bbox_xyxy == [20, 40, 380, 100]));
}
