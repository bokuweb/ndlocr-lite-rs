#![cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires pinned models and output directory"]
fn inspect_dense_crop_and_fixed_vertical_context() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{
            cached::ParseqSession,
            deim_cached::DeimSession,
            parseq::{ParseqOrientation, sanitize_recognized_fragment},
        },
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    use std::path::Path;
    let output = std::env::var("NDLOCR_DENSE_CROP_OUTPUT").expect("output path");
    let rows: serde_json::Value = serde_json::from_slice(
        &std::fs::read("docs/benchmarks/dense-line/evaluation-manifest.json").unwrap(),
    )
    .unwrap();
    let model = ParseqSession::load(
        Path::new("models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let deim = DeimSession::load(Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let (iw, ih) = model.input_dimensions();
    let mut records = Vec::new();
    for row in rows["inputs"].as_array().unwrap() {
        if let Ok(ids) = std::env::var("NDLOCR_DENSE_CROP_IDS") {
            if !ids.split(',').any(|id| Some(id) == row["id"].as_str()) {
                continue;
            }
        }
        let image = load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
        for mode in ["original", "bands", "deim"] {
            let crops = if mode == "original" {
                vec![ndlocr_lite_rs::app::PreparedLineCrop {
                    crop: ndlocr_lite_rs::pipeline::crop::CroppedImage {
                        data: image.data.clone(),
                        width: image.width,
                        height: image.height,
                    },
                    bbox_xyxy: [0, 0, image.width as i32, image.height as i32],
                    confidence: 1.0,
                    pred_char_count: 0.0,
                    is_vertical: false,
                }]
            } else {
                let detections = if mode == "bands" {
                    fallback_line_detections_from_bands(
                        &image.data,
                        image.width,
                        image.height,
                        220,
                        25,
                        45,
                    )
                } else {
                    deim.detect_rgb_u8(&image.data, image.width, image.height, 0.25)
                        .unwrap()
                };
                prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap()
            };
            let bounds: Vec<_> = crops.iter().map(|c| c.bbox_xyxy).collect();
            for (index, line) in crops.iter().enumerate() {
                let c = &line.crop;
                if c.height < 12
                    || c.width as u128 * ih as u128 <= 2 * c.height as u128 * iw as u128
                {
                    continue;
                }
                let mut columns = 0;
                let mut groups = 0;
                let mut top = c.height;
                let mut bottom = 0;
                let mut run_start = 0;
                let mut run_top = c.height;
                let mut run_bottom = 0;
                let mut in_run = false;
                for x in 0..=c.width {
                    let mut ct = c.height;
                    let mut cb = 0;
                    if x < c.width {
                        for y in 0..c.height {
                            let p = (y * c.width + x) * 3;
                            if c.data[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660 {
                                ct = ct.min(y);
                                cb = cb.max(y + 1);
                            }
                        }
                    }
                    if cb > 0 {
                        columns += 1;
                        top = top.min(ct);
                        bottom = bottom.max(cb);
                        if !in_run {
                            run_start = x;
                            run_top = c.height;
                            run_bottom = 0;
                        }
                        run_top = run_top.min(ct);
                        run_bottom = run_bottom.max(cb);
                        in_run = true;
                    } else if in_run {
                        if x - run_start <= 2 * c.height
                            && run_bottom - run_top >= c.height.div_ceil(3)
                        {
                            groups += 1;
                        }
                        in_run = false;
                    }
                }
                let base = model
                    .recognize_rgb_u8_candidates_with_orientation(
                        &c.data,
                        c.width,
                        c.height,
                        ParseqOrientation::Horizontal,
                        3,
                    )
                    .unwrap();
                let text = sanitize_recognized_fragment(&base.recognition.text);
                let mut variants = Vec::new();
                if text.chars().count() <= 16 {
                    if let Ok(chunks) =
                        split_line_for_input(&c.data, c.width, c.height, 220, iw, ih)
                    {
                        for padding in [0usize, 1, 2, 4, 8] {
                            let mut joined = String::new();
                            let mut parts = Vec::new();
                            for (ci, chunk) in chunks.iter().enumerate() {
                                let piece =
                                    crop_line_chunk_rgb_u8(&c.data, c.width, c.height, &chunks, ci)
                                        .unwrap();
                                let h = piece.height + padding * 2;
                                let mut data = vec![255; piece.width * h * 3];
                                let start = padding * piece.width * 3;
                                data[start..start + piece.data.len()].copy_from_slice(&piece.data);
                                let result = model
                                    .recognize_rgb_u8_candidates_with_orientation(
                                        &data,
                                        piece.width,
                                        h,
                                        ParseqOrientation::Horizontal,
                                        3,
                                    )
                                    .unwrap();
                                let text = sanitize_recognized_fragment(&result.recognition.text);
                                if chunk.insert_space_before(&joined, &text, c.height) {
                                    joined.push(' ');
                                }
                                joined.push_str(&text);
                                parts.push(serde_json::json!({"bounds":[chunk.x0,chunk.x1],"size":[piece.width,h],"text":text,"confidence":result.recognition.mean_confidence,"stop_reason":format!("{:?}",result.stop_reason),"last_step":result.steps.last().map(|s|serde_json::json!({"timestep":s.timestep,"candidates":s.candidates.iter().map(|c|serde_json::json!({"character":c.character,"probability":c.probability})).collect::<Vec<_>>()}))}));
                            }
                            variants.push(
                                serde_json::json!({"padding":padding,"text":joined,"parts":parts}),
                            );
                        }
                    }
                }
                records.push(serde_json::json!({"id":row["id"],"mode":mode,"index":index,"bounds":bounds,"crop_bounds":line.bbox_xyxy,"size":[c.width,c.height],"ink_columns":columns,"supported_groups":groups,"body_height":bottom.saturating_sub(top),"top_margin":top,"bottom_margin":c.height-bottom,"base":text,"base_confidence":base.recognition.mean_confidence,"stop_reason":format!("{:?}",base.stop_reason),"variants":variants}));
            }
        }
    }
    std::fs::write(output, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
}

#[test]
#[ignore = "diagnostic; requires pinned models"]
fn inspect_dense_mark_neighbor_context() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{
            cached::ParseqSession,
            parseq::{ParseqOrientation, sanitize_recognized_fragment},
        },
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    use std::path::Path;
    let mut records = Vec::new();
    for name in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let model = ParseqSession::load(
            &Path::new("models").join(name),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let (iw, ih) = model.input_dimensions();
        for id in ["japanese_gothic36", "japanese_mincho36"] {
            let image = load_rgb_u8(
                &Path::new("tests/fixtures/long_line_split/images").join(format!("{id}.png")),
            )
            .unwrap();
            let detections = fallback_line_detections_from_bands(
                &image.data,
                image.width,
                image.height,
                220,
                25,
                45,
            );
            let crops =
                prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap();
            assert_eq!(crops.len(), 1);
            let c = &crops[0].crop;
            let original = split_line_for_input(&c.data, c.width, c.height, 220, iw, ih).unwrap();
            let original_fields:Vec<_>=original.iter().map(|p|serde_json::json!({"bounds":[p.x0,p.x1],"space_before":p.space_before,"gap_before":p.gap_before})).collect();
            let Some(index) = original
                .iter()
                .position(|p| p.x1 - p.x0 <= c.height && p.x0 > 2100 && p.x1 < 2200)
            else {
                records.push(serde_json::json!({"id":id,"model":name,"status":"no isolated narrow chunk in baseline diagnostic interval","chunks":original_fields}));
                continue;
            };
            for side in ["left", "right"] {
                let mut chunks =
                    split_line_for_input(&c.data, c.width, c.height, 220, iw, ih).unwrap();
                if side == "left" {
                    chunks[index - 1].x1 = chunks[index].x1;
                    chunks.remove(index);
                } else {
                    chunks[index].x1 = chunks[index + 1].x1;
                    chunks.remove(index + 1);
                }
                let mut text = String::new();
                let mut parts = Vec::new();
                for (i, p) in chunks.iter().enumerate() {
                    let crop =
                        crop_line_chunk_rgb_u8(&c.data, c.width, c.height, &chunks, i).unwrap();
                    let result = model
                        .recognize_rgb_u8_unprocessed_with_orientation(
                            &crop.data,
                            crop.width,
                            crop.height,
                            ParseqOrientation::Horizontal,
                        )
                        .unwrap();
                    let next = sanitize_recognized_fragment(&result.text);
                    if p.insert_space_before(&text, &next, c.height) {
                        text.push(' ');
                    }
                    text.push_str(&next);
                    parts.push(serde_json::json!({"bounds":[p.x0,p.x1],"text":next,"confidence":result.mean_confidence}));
                }
                records.push(serde_json::json!({"id":id,"model":name,"side":side,"original_chunks":original_fields,"text":text,"parts":parts}));
            }
        }
    }
    std::fs::write(
        "tmp/dense-crop/neighbor.json",
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "diagnostic; requires pinned DEIM model"]
fn record_dense_crop_inventory() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::deim_cached::DeimSession,
        io::load_rgb_u8,
    };
    use std::path::Path;
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::env::var("NDLOCR_DENSE_INVENTORY_MANIFEST")
                .unwrap_or_else(|_| "docs/benchmarks/dense-line/evaluation-manifest.json".into()),
        )
        .unwrap(),
    )
    .unwrap();
    let detector = DeimSession::load(Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let mut records = Vec::new();
    for row in manifest["inputs"].as_array().unwrap() {
        if let Ok(ids) = std::env::var("NDLOCR_DENSE_CROP_IDS") {
            if !ids.split(',').any(|id| Some(id) == row["id"].as_str()) {
                continue;
            }
        }
        let image = load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
        let mut modes = Vec::new();
        for mode in ["bands", "deim"] {
            let ds = if mode == "bands" {
                fallback_line_detections_from_bands(
                    &image.data,
                    image.width,
                    image.height,
                    220,
                    25,
                    45,
                )
            } else {
                detector
                    .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
                    .unwrap()
            };
            let crops = prepare_line_crops(&image.data, image.width, image.height, ds, 0).unwrap();
            modes.push(serde_json::json!({"mode":mode,"crops":crops.iter().map(|p|serde_json::json!({"bounds":p.bbox_xyxy,"size":[p.crop.width,p.crop.height],"wide":p.crop.height>=12 && p.crop.width as u128 *24>2*p.crop.height as u128*768})).collect::<Vec<_>>()}));
        }
        records.push(serde_json::json!({"id":row["id"],"image":row["image"],"original_size":[image.width,image.height],"modes":modes}));
    }
    assert!(!records.is_empty());
    std::fs::write(
        std::env::var("NDLOCR_DENSE_INVENTORY_OUTPUT")
            .unwrap_or_else(|_| "tmp/dense-crop/inventory.json".into()),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
}
