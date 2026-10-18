#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{app::prepare_line_crops, infer::deim_cached::DeimSession, io::load_rgb_u8};

/// 元検出を保ちながら、同じ神戸市資料の本文1行を一度だけ認識する。
#[test]
#[ignore = "requires pinned DEIM assets"]
fn physical_body_line_has_one_selected_crop() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let session = DeimSession::load(std::path::Path::new(
        &std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap(),
    ))
    .unwrap();
    let detections = session
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    let raw:Vec<_>=detections.iter().map(|d|serde_json::json!({"class_index":d.class_index,"class_name":d.class_name,"confidence":d.confidence,"bounds":d.box_xyxy,"pred_char_count":d.pred_char_count})).collect();
    let crops = prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap();
    let target: Vec<_> = crops
        .iter()
        .filter(|c| {
            c.bbox_xyxy[1] <= 875
                && c.bbox_xyxy[3] > 875
                && c.bbox_xyxy[0] < 100
                && c.bbox_xyxy[2] > 900
        })
        .collect();
    eprintln!(
        "TARGET {:?}",
        target
            .iter()
            .map(|c| (c.bbox_xyxy, c.confidence, c.pred_char_count))
            .collect::<Vec<_>>()
    );
    let selected:Vec<_>=crops.iter().map(|c|serde_json::json!({"bounds":c.bbox_xyxy,"confidence":c.confidence,"pred_char_count":c.pred_char_count})).collect();
    if let Some(path) = std::env::var_os("NDLOCR_DEIM_EXTENT_OUTPUT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({"raw":raw,"selected":selected})).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(target.len(), 1, "one physical line must produce one crop");
}

/// 再利用するページ実行系にも同じ選択を適用し、crop側と一意性を揃える。
#[test]
#[ignore = "requires pinned DEIM/PARSeq assets"]
fn pooled_page_selects_the_same_physical_rows() {
    use ndlocr_lite_rs::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let runtime = PageRecognizer::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
        1,
    )
    .unwrap();
    let output = runtime
        .recognize_page_rgb_u8(
            &image.data,
            image.width,
            image.height,
            PageRecognizerOptions {
                det_conf_threshold: 0.25,
                min_line_confidence: 0.0,
                drop_empty_text: false,
            },
        )
        .unwrap();
    assert_eq!(output.lines.len(), 32);
    let target: Vec<_> = output
        .lines
        .iter()
        .filter(|c| {
            c.bbox_xyxy[1] <= 875
                && c.bbox_xyxy[3] > 875
                && c.bbox_xyxy[0] < 100
                && c.bbox_xyxy[2] > 900
        })
        .collect();
    assert_eq!(target.len(), 1);
    assert_eq!(target[0].bbox_xyxy, [69, 863, 948, 887]);
}
