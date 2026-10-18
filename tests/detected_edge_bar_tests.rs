use ndlocr_lite_rs::{
    app::prepare_line_crops,
    infer::deim::Detection,
    pipeline::run_page::{PageInput, run_page},
};

fn sample(scale: usize, margin: usize, reflected: bool) -> (Vec<u8>, usize, usize, Detection) {
    let (w, h) = (400 * scale + 2 * margin, 80 * scale);
    let mut rgb = vec![255; w * h * 3];
    for (xs, ys) in [(46..76, 39..42), (80..350, 24..56)] {
        for y in ys.start * scale..ys.end * scale {
            for x in xs.start * scale + margin..xs.end * scale + margin {
                let x = if reflected { w - 1 - x } else { x };
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    let (x0, x1) = (65 * scale + margin, 360 * scale + margin);
    let bounds = if reflected {
        [w - x1, 22 * scale, w - x0, 58 * scale]
    } else {
        [x0, 22 * scale, x1, 58 * scale]
    };
    let detection = Detection {
        class_index: 1,
        class_name: "line_main".into(),
        confidence: 0.8,
        pred_char_count: 3.0,
        box_xyxy: bounds.map(|v| v as i32),
    };
    (rgb, w, h, detection)
}

#[test]
fn clipped_edge_bar_recovers_source_pixels_before_padding() {
    let (rgb, w, h, d) = sample(1, 0, false);
    let crop = prepare_line_crops(&rgb, w, h, vec![d.clone()], 1)
        .unwrap()
        .remove(0);
    assert_eq!(crop.bbox_xyxy, [43, 21, 361, 59]);
    assert_eq!(
        (crop.confidence, crop.pred_char_count),
        (d.confidence, d.pred_char_count)
    );
    assert!(!crop.is_vertical);
    for y in 0..crop.crop.height {
        assert_eq!(
            &crop.crop.data[y * crop.crop.width * 3..(y + 1) * crop.crop.width * 3],
            &rgb[((y + 21) * w + 43) * 3..((y + 21) * w + 361) * 3]
        );
    }
}

#[test]
fn cascade_uses_the_same_completed_recognition_region() {
    let (rgb, w, h, d) = sample(1, 0, false);
    let out = run_page(
        PageInput {
            rgb: &rgb,
            width: w,
            height: h,
            detections: &[d],
        },
        |crop| {
            assert_eq!(crop.width, 316);
            "body".into()
        },
        |_| panic!("unexpected model"),
        |_| panic!("unexpected model"),
    )
    .unwrap();
    assert_eq!(out.lines[0].bbox_xyxy, [44, 22, 360, 58]);
}

#[test]
fn tall_foreign_or_ambiguous_components_keep_original_bounds() {
    for case in 0..7 {
        let (mut rgb, w, h, mut d) = sample(1, 0, false);
        let mut others = Vec::new();
        match case {
            0 => {
                for y in 24..56 {
                    rgb[(y * w + 65) * 3..(y * w + 66) * 3].fill(0);
                }
            }
            1 => {
                for x in 0..76 {
                    rgb[(40 * w + x) * 3..(40 * w + x + 1) * 3].fill(0);
                }
            }
            2 => rgb[(50 * w + 50) * 3..(50 * w + 51) * 3].fill(0),
            3 => {
                let mut other = d.clone();
                other.box_xyxy = [30, 22, 45, 58];
                others.push(other);
            }
            4 => {
                let mut other = d.clone();
                other.box_xyxy = [30, 22, 70, 58];
                others.push(other);
            }
            5 => d.box_xyxy = [65, 22, 100, 58],
            6 => {
                for y in 28..31 {
                    rgb[(y * w + 65) * 3..(y * w + 66) * 3].fill(0);
                }
            }
            _ => unreachable!(),
        }
        let old = d.box_xyxy;
        let mut ds = vec![d];
        ds.extend(others);
        let crops = prepare_line_crops(&rgb, w, h, ds, 0).unwrap();
        assert_eq!(crops[0].bbox_xyxy, old, "case={case}");
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    #[test]
    fn completed_bar_follows_scale_shift_and_reflection(scale in 1usize..4,margin in 0usize..12,reflected in proptest::bool::ANY) {
        let (rgb,w,h,d)=sample(scale,margin,reflected);
        let crop=prepare_line_crops(&rgb,w,h,vec![d.clone()],0).unwrap().remove(0);
        let (left,right)=(46*scale+margin-2,360*scale+margin);
        let expected=if reflected {[w-right,22*scale,w-left,58*scale]}else{[left,22*scale,right,58*scale]};
        proptest::prop_assert_eq!(crop.bbox_xyxy,expected.map(|v|v as i32));
        proptest::prop_assert_eq!(crop.confidence,d.confidence);
        proptest::prop_assert_eq!(crop.pred_char_count,d.pred_char_count);
    }
}

/// 両側が白余白へ拡張して衝突する場合も、重複cropを増やさない。
#[test]
fn proposed_regions_cannot_collide_in_the_shared_margin() {
    let (w, h) = (350, 48);
    let mut rgb = vec![255; w * h * 3];
    for (xs, ys) in [
        (10..150, 12..32),
        (190..330, 12..32),
        (156..171, 21..23),
        (174..184, 21..23),
    ] {
        for y in ys {
            for x in xs.clone() {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    let bounds = [[0, 10, 160, 34], [180, 10, 340, 34]];
    let ds = bounds.map(|box_xyxy| Detection {
        class_index: 1,
        class_name: "line_main".into(),
        confidence: 0.8,
        pred_char_count: 3.0,
        box_xyxy,
    });
    let crops = prepare_line_crops(&rgb, w, h, ds.to_vec(), 0).unwrap();
    assert_eq!(
        crops.iter().map(|c| c.bbox_xyxy).collect::<Vec<_>>(),
        bounds
    );
}
