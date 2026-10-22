use super::split_long_line_near_center;

/// 固定長行で、初回認識の長さと中央の連結画素を入力側から保存する。
#[test]
#[ignore = "diagnostic; requires pinned 100-character model"]
fn long_line_midpoint_does_not_cut_a_connected_stroke() {
    use crate::{
        infer::{cached::ParseqSession, parseq::sanitize_recognized_fragment},
        io::load_rgb_u8,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let rows: serde_json::Value = serde_json::from_slice(
        &std::fs::read("tests/fixtures/long_line_split/manifest.json").unwrap(),
    )
    .unwrap();
    let session = ParseqSession::load(
        std::path::Path::new(
            "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
        std::path::Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let mut records = Vec::new();
    for row in rows["rows"].as_array().unwrap() {
        let image = load_rgb_u8(std::path::Path::new(row["image"].as_str().unwrap())).unwrap();
        let recognized = session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        let text = sanitize_recognized_fragment(&recognized.text);
        let eligible = text.chars().count() >= 80 && image.width >= image.height * 4;
        let mid = image.width / 2;
        let ink = |x: usize, y: usize| {
            image.data[(y * image.width + x) * 3..(y * image.width + x + 1) * 3]
                .iter()
                .map(|&v| u16::from(v))
                .sum::<u16>()
                < 660
        };
        let crosses_body_ink = |x: usize| {
            (0..image.height).any(|y| {
                let known_underline = row["underlined"] == true && y == image.height - 4;
                !known_underline
                    && ink(x - 1, y)
                    && (y.saturating_sub(1)..=(y + 1).min(image.height - 1)).any(|yy| ink(x, yy))
            })
        };
        let legacy_cuts = eligible && crosses_body_ink(mid);
        let selected = if eligible {
            split_long_line_near_center(&image.data, image.width, image.height, 220).unwrap()
        } else {
            None
        };
        let cut = selected.as_ref().map(|chunks| chunks[0].x1);
        let cuts = cut.is_some_and(crosses_body_ink);
        let mut pieces = Vec::new();
        if eligible {
            for (x0, x1) in [(0, mid), (mid, image.width)] {
                let crop = crop_rgb_u8(
                    &image.data,
                    image.width,
                    image.height,
                    BBox::new(x0, 0, x1, image.height),
                )
                .unwrap();
                let result = session
                    .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                    .unwrap();
                pieces.push(serde_json::json!({"bounds":[x0,x1],"text":result.text,"confidence":result.mean_confidence}));
            }
        }
        records.push(serde_json::json!({"id":row["id"],"size":[image.width,image.height],"raw":recognized.text,"fragment":text,"confidence":recognized.mean_confidence,"eligible":eligible,"midpoint":mid,"cuts_connected_ink":cuts,"legacy_cuts_connected_ink":legacy_cuts,"selected_cut":cut,"legacy_pieces":pieces}));
    }
    if let Some(path) = std::env::var_os("NDLOCR_LONG_LINE_PROBE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
    }
    let eligible = records.iter().filter(|r| r["eligible"] == true).count();
    let cuts = records
        .iter()
        .filter(|r| r["cuts_connected_ink"] == true)
        .count();
    eprintln!("{} eligible crops, {} midpoint cuts", eligible, cuts);
    assert!(eligible > 0, "must exercise the default 80-character gate");
    assert!(
        records
            .iter()
            .any(|r| r["legacy_cuts_connected_ink"] == true),
        "must retain the legacy counterexample"
    );
    assert_eq!(cuts, 0, "must not cut connected foreground strokes");
}

/// DEIMが返す薄い部分領域を、元境界・移動境界・confidenceに分けて診断する。
#[test]
#[ignore = "diagnostic; requires DEIM and 100-character assets"]
fn inspect_partial_detected_long_line_inputs() {
    use crate::{
        app::prepare_line_crops,
        infer::{cached::ParseqSession, deim_cached::DeimSession},
        io::load_rgb_u8,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/long_line_split/images/uri_gothic36.png",
    ))
    .unwrap();
    let detector = DeimSession::load(std::path::Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let raw = detector
        .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
        .unwrap();
    let crops = prepare_line_crops(&image.data, image.width, image.height, raw, 0).unwrap();
    let session = ParseqSession::load(
        std::path::Path::new(
            "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
        std::path::Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let mut rows = Vec::new();
    for (index, line) in crops.iter().enumerate() {
        let crop = &line.crop;
        let before = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap();
        let moved = split_long_line_near_center(&crop.data, crop.width, crop.height, 220)
            .unwrap()
            .map(|c| c[0].x1);
        let mut variants = Vec::new();
        for cut in [Some(crop.width / 2), moved] {
            if let Some(cut) = cut {
                let mut parts = Vec::new();
                for (x0, x1) in [(0, cut), (cut, crop.width)] {
                    let piece = crop_rgb_u8(
                        &crop.data,
                        crop.width,
                        crop.height,
                        BBox::new(x0, 0, x1, crop.height),
                    )
                    .unwrap();
                    let result = session
                        .recognize_rgb_u8_unprocessed(&piece.data, piece.width, piece.height)
                        .unwrap();
                    parts.push(
                        serde_json::json!({"text":result.text,"confidence":result.mean_confidence}),
                    );
                }
                variants.push(serde_json::json!({"cut":cut,"parts":parts}));
            }
        }
        rows.push(serde_json::json!({"bounds":line.bbox_xyxy,"size":[crop.width,crop.height],"bucket":line.pred_char_count,"raw":before.text,"parseq_confidence":before.mean_confidence,"detector_confidence":line.confidence,"variants":variants}));
        if let Some(dir) = std::env::var_os("NDLOCR_LONG_LINE_CROP_PROBE_DIR") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            image::save_buffer(
                dir.join(format!("crop-{index}.png")),
                &crop.data,
                crop.width as u32,
                crop.height as u32,
                image::ColorType::Rgb8,
            )
            .unwrap();
        }
    }
    assert!(!rows.is_empty());
    if let Some(dir) = std::env::var_os("NDLOCR_LONG_LINE_CROP_PROBE_DIR") {
        std::fs::write(
            std::path::Path::new(&dir).join("results.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
