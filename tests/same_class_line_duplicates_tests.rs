use ndlocr_lite_rs::{app::prepare_line_crops, infer::deim::Detection};
use proptest::prelude::*;
fn line(b: [i32; 4], score: f32) -> Detection {
    Detection {
        class_index: 1,
        class_name: "line_main".into(),
        box_xyxy: b,
        confidence: score,
        pred_char_count: 2.0,
    }
}
#[test]
fn overlapping_same_class_queries_are_recognized_once() {
    let ds = vec![
        line([224, 0, 2494, 39], 0.9656),
        line([152, 0, 2479, 40], 0.9790),
        line([168, 0, 2488, 41], 0.9785),
    ];
    let crops = prepare_line_crops(&vec![255; 2566 * 47 * 3], 2566, 47, ds, 0).unwrap();
    assert_eq!(crops.len(), 1);
    assert_eq!(crops[0].bbox_xyxy, [152, 0, 2479, 40]);
    assert_eq!(crops[0].confidence, 0.9790);
    assert_eq!(crops[0].pred_char_count, 2.0);
}
#[test]
fn distinct_same_class_rows_and_partial_regions_are_retained() {
    let ds = vec![
        line([0, 0, 100, 20], 0.9),
        line([0, 21, 100, 41], 0.8),
        line([10, 5, 90, 15], 0.7),
    ];
    assert_eq!(
        prepare_line_crops(&vec![255; 100 * 50 * 3], 100, 50, ds, 0)
            .unwrap()
            .len(),
        3
    );
}
proptest! {
    #[test]
    fn same_class_winner_keeps_metadata_under_scale_and_input_reversal(scale in 1i32..8,x in 0i32..100,y in 0i32..100,reverse in any::<bool>()) {
        let b=[x,y,x+100*scale,y+20*scale];let mut ds=vec![line(b,0.8),line(b,0.9)];if reverse{ds.reverse();}
        let w=(x+100*scale+1)as usize;let h=(y+20*scale+1)as usize;
        let crops=prepare_line_crops(&vec![255;w*h*3],w,h,ds,0).unwrap();
        prop_assert_eq!(crops.len(),1);prop_assert_eq!(crops[0].bbox_xyxy,b);prop_assert_eq!(crops[0].confidence,0.9);prop_assert_eq!(crops[0].pred_char_count,2.0);
    }
}
#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned DEIM and long-line fixture"]
fn real_deim_retains_raw_queries_but_recognizes_one_physical_uri_line() {
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/long_line_split/images/uri_gothic36.png",
    ))
    .unwrap();
    let detector = ndlocr_lite_rs::infer::deim_cached::DeimSession::load(std::path::Path::new(
        "models/deim-s-1024x1024.onnx",
    ))
    .unwrap();
    let raw = detector
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    let crops = prepare_line_crops(&image.data, image.width, image.height, raw.clone(), 0).unwrap();
    let line_queries: Vec<_> = raw
        .iter()
        .filter(|d| d.class_name.starts_with("line_"))
        .collect();
    assert!(line_queries.len() >= 3);
    let winner = line_queries
        .iter()
        .max_by(|a, b| a.confidence.total_cmp(&b.confidence))
        .unwrap();
    eprintln!(
        "RAW {:?}",
        line_queries
            .iter()
            .map(|d| (&d.class_name, d.box_xyxy, d.confidence, d.pred_char_count))
            .collect::<Vec<_>>()
    );
    if let Some(path) = std::env::var_os("NDLOCR_DUPLICATE_RAW_PATH") {
        let records: Vec<_>=line_queries.iter().map(|d|serde_json::json!({"bounds":d.box_xyxy,"class":d.class_name,"detector_confidence":d.confidence,"pred_char_count":d.pred_char_count})).collect();
        std::fs::write(path, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
    }
    assert_eq!(crops.len(), 1);
    assert_eq!(crops[0].bbox_xyxy, winner.box_xyxy);
    assert_eq!(crops[0].confidence, winner.confidence);
    assert_eq!(crops[0].pred_char_count, winner.pred_char_count);
}
