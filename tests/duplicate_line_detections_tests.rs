use ndlocr_lite_rs::app::prepare_line_crops;
use ndlocr_lite_rs::infer::deim::Detection;
use ndlocr_lite_rs::pipeline::run_page::{PageInput, run_page};
use std::cell::Cell;

/// 固定モデルで観測した2行の競合と、その周囲の非行候補を返す。
fn competing_lines() -> Vec<Detection> {
    [
        ("text_block", [40, 28, 263, 116], 0.856),
        ("block_fig", [58, 230, 924, 378], 0.802),
        ("line_title", [42, 28, 262, 56], 0.784),
        ("line_main", [42, 88, 262, 115], 0.709),
        ("line_title", [42, 88, 262, 115], 0.587),
        ("line_main", [42, 28, 262, 56], 0.471),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (name, bounds, confidence))| Detection {
        class_index: index,
        confidence,
        box_xyxy: bounds,
        pred_char_count: 3.0,
        class_name: name.into(),
    })
    .collect()
}

/// 同じ本文へのmain/title競合を、PARSeqへ重複して渡さない。
#[test]
fn competing_line_classes_are_recognized_once_per_physical_line() {
    let rgb = vec![255; 1400 * 500 * 3];
    let detections = competing_lines();
    let calls = Cell::new(0);
    let output = run_page(
        PageInput {
            rgb: &rgb,
            width: 1400,
            height: 500,
            detections: &detections,
        },
        |_| {
            calls.set(calls.get() + 1);
            "通常の本文です。".into()
        },
        |_| panic!("short line must not need a second recognition"),
        |_| panic!("short line must not need a third recognition"),
    )
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(output.total_detection_count, 6);
    assert_eq!(output.line_detection_count, 2);
    assert_eq!(output.texts, ["通常の本文です。", "通常の本文です。"]);
    assert_eq!(output.lines[0].bbox_xyxy, [42, 28, 262, 56]);
    assert_eq!(output.lines[0].confidence, 0.784);
    assert_eq!(output.lines[1].confidence, 0.709);
}

/// CLI用cropでも検出時の強い候補を使い、padding前に競合を除く。
#[test]
fn cli_crops_keep_the_winning_detection_metadata() {
    let crops =
        prepare_line_crops(&vec![255; 1400 * 500 * 3], 1400, 500, competing_lines(), 1).unwrap();
    assert_eq!(crops.len(), 2);
    assert_eq!(crops[0].bbox_xyxy, [41, 27, 263, 57]);
    assert_eq!(crops[1].bbox_xyxy, [41, 87, 263, 116]);
    assert_eq!(crops[0].confidence, 0.784);
    assert_eq!(crops[1].confidence, 0.709);
    assert_eq!(crops[0].pred_char_count, 3.0);
}

/// 画像外の強い候補を捨てても、正常な本文候補は失われない。
#[test]
fn out_of_image_candidate_cannot_suppress_valid_line() {
    let mut detections = competing_lines();
    detections.truncate(2);
    let template = Detection {
        class_index: 1,
        class_name: "line_main".into(),
        box_xyxy: [0, 0, 100, 20],
        confidence: 0.7,
        pred_char_count: 3.0,
    };
    detections.push(template.clone());
    detections.push(Detection {
        class_name: "line_title".into(),
        box_xyxy: [0, 0, 101, 20],
        confidence: 0.99,
        ..template
    });
    let crops = prepare_line_crops(&vec![255; 100 * 30 * 3], 100, 30, detections, 0).unwrap();
    assert_eq!(crops.len(), 1);
    assert_eq!(crops[0].bbox_xyxy, [0, 0, 100, 20]);
}

/// 固定公式モデルでも生検出を保ち、通常本文の二重認識を除く。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires pinned DEIM/PARSeq assets"]
fn real_deim_competition_preserves_raw_results_and_unique_recognition() {
    use ndlocr_lite_rs::infer::{
        cached::ParseqSession,
        deim,
        deim_cached::DeimSession,
        page_pool::{PageRecognizer, PageRecognizerOptions},
    };
    use std::path::PathBuf;
    let model = PathBuf::from(std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap());
    let parseq_model = PathBuf::from(std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = PathBuf::from(std::env::var("NDLOCR_TEST_CHARSET").unwrap());
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/noisy_line_gaps/mixed_height.png",
    ))
    .unwrap();
    let raw = deim::detect_rgb_u8(&model, &image.data, image.width, image.height, 0.25).unwrap();
    let cached = DeimSession::load(&model)
        .unwrap()
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    assert_eq!(raw.len(), cached.len());
    for (a, b) in raw.iter().zip(&cached) {
        assert_eq!(a.box_xyxy, b.box_xyxy);
        assert_eq!(a.class_index, b.class_index);
        assert_eq!(a.class_name, b.class_name);
        assert_eq!(a.pred_char_count, b.pred_char_count);
        assert!((a.confidence - b.confidence).abs() < 1e-5);
    }
    assert_eq!(
        raw.iter()
            .filter(|d| d.class_name.starts_with("line_"))
            .count(),
        4
    );
    let crops = prepare_line_crops(&image.data, image.width, image.height, raw, 0).unwrap();
    assert_eq!(crops.len(), 2);
    assert_eq!(crops[0].bbox_xyxy, [42, 28, 262, 56]);
    assert_eq!(crops[1].bbox_xyxy, [42, 88, 262, 115]);
    let parseq = ParseqSession::load(&parseq_model, &charset).unwrap();
    let texts: Vec<_> = crops
        .iter()
        .map(|c| {
            parseq
                .recognize_rgb_u8_unprocessed(&c.crop.data, c.crop.width, c.crop.height)
                .unwrap()
                .text
        })
        .collect();
    assert_eq!(texts, ["通常の本文です。", "通常の本文です。"]);
    let runtime = PageRecognizer::load(&model, &parseq_model, &charset, 1).unwrap();
    let result = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions {
                det_conf_threshold: 0.25,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        result
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>(),
        texts
    );
    println!(
        "raw_line_detections=4 selected_crops=2 pooled_lines={} text={texts:?}; giant heading 二三ごニニ remains missing",
        result.lines.len()
    );
}
