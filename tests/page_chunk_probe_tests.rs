#![cfg(feature = "onnx")]

#[path = "support/native_chunk_groups.rs"]
mod native_chunk_groups;

/// 全幅と既存の安全なchunkの差を、truthを認識へ渡さず保存する診断。
#[test]
#[ignore = "diagnostic; requires pinned DEIM/100-character model and output directory"]
fn compare_full_width_and_safe_chunks_on_frozen_pages() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{
            cached::ParseqSession,
            deim_cached::DeimSession,
            parseq::{ParseqOrientation, RecognizeResult, sanitize_recognized_fragment},
        },
        io::load_rgb_u8,
        pipeline::{
            background::normalize_chromatic_panels,
            connect::RecognizedLine,
            line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
            reading_order::sort_lines_in_reading_order,
        },
        postprocess::page_rules::apply_structural_rules,
    };
    fn render(mut lines: Vec<RecognizedLine>) -> String {
        lines.retain(|line| line.confidence >= 0.3 && !line.text.trim().is_empty());
        sort_lines_in_reading_order(&mut lines);
        apply_structural_rules(&lines.into_iter().map(|line| line.text).collect::<Vec<_>>())
            .join("\n")
    }
    let root = std::path::PathBuf::from(
        std::env::var_os("NDLOCR_PAGE_CHUNK_PROBE_DIR").expect("output directory"),
    );
    let model = ParseqSession::load(
        std::path::Path::new(
            "models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
        std::path::Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let deim = DeimSession::load(std::path::Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let (input_w, input_h) = model.input_dimensions();
    let native_groups = std::env::var_os("NDLOCR_PAGE_CHUNK_GROUP_NATIVE").is_some();
    let mut paths = Vec::new();
    for dir in [
        "tests/fixtures/eval/images",
        "docs/benchmarks/deim-edge/quality/images",
    ] {
        paths.extend(
            std::fs::read_dir(dir)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|x| x.to_str()),
                        Some("png" | "jpg")
                    )
                }),
        );
    }
    paths.sort();
    assert_eq!(paths.len(), 20);
    let mut pages = Vec::new();
    for path in paths {
        for mode in ["deim", "bands"] {
            let mut image = load_rgb_u8(&path).unwrap();
            let detections = if mode == "deim" {
                deim.detect_rgb_u8(&image.data, image.width, image.height, 0.25)
                    .unwrap()
                    .into_iter()
                    .filter(|d| d.class_name.starts_with("line_"))
                    .collect()
            } else {
                Vec::new()
            };
            let detections = if detections.is_empty() {
                if let std::borrow::Cow::Owned(prepared) =
                    normalize_chromatic_panels(&image.data, image.width, image.height).unwrap()
                {
                    image.data = prepared;
                }
                fallback_line_detections_from_bands(
                    &image.data,
                    image.width,
                    image.height,
                    220,
                    25,
                    45,
                )
            } else {
                detections
            };
            let crops =
                prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap();
            let mut before_lines = Vec::new();
            let mut after_lines = Vec::new();
            let mut records = Vec::new();
            for (index, line) in crops.iter().enumerate() {
                let image = &line.crop;
                let base = model
                    .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
                    .unwrap();
                let raw_before = base.text.clone();
                let base = RecognizeResult {
                    text: sanitize_recognized_fragment(&base.text),
                    mean_confidence: base.mean_confidence,
                };
                let mut candidate = base.clone();
                let mut chunks_record = Vec::new();
                let mut error = None;
                let eligible = image.width as u128 * input_h as u128
                    > image.height as u128 * input_w as u128
                    && !line.is_vertical;
                if eligible {
                    match split_line_for_input(
                        &image.data,
                        image.width,
                        image.height,
                        220,
                        input_w,
                        input_h,
                    ) {
                        Ok(chunks) if chunks.len() > 1 => {
                            let chunks = if native_groups {
                                native_chunk_groups::group_native_chunks(
                                    &chunks,
                                    image.height,
                                    input_w,
                                    input_h,
                                )
                                .unwrap()
                            } else {
                                chunks
                            };
                            let mut text = String::new();
                            let mut confidence_sum = 0.0;
                            let mut count = 0;
                            for (i, chunk) in chunks.iter().enumerate() {
                                let crop = crop_line_chunk_rgb_u8(
                                    &image.data,
                                    image.width,
                                    image.height,
                                    &chunks,
                                    i,
                                )
                                .unwrap();
                                let result = model
                                    .recognize_rgb_u8_unprocessed_with_orientation(
                                        &crop.data,
                                        crop.width,
                                        crop.height,
                                        ParseqOrientation::Horizontal,
                                    )
                                    .unwrap();
                                let fragment = sanitize_recognized_fragment(&result.text);
                                if chunk.insert_space_before(&text, &fragment, image.height) {
                                    text.push(' ');
                                }
                                text.push_str(&fragment);
                                let length = fragment.chars().count();
                                confidence_sum += result.mean_confidence * length as f32;
                                count += length;
                                chunks_record.push(serde_json::json!({"bounds":[chunk.x0,chunk.x1],"size":[crop.width,crop.height],"space_before":chunk.space_before,"gap_before":chunk.gap_before,"raw":result.text,"fragment":fragment,"confidence":result.mean_confidence}));
                            }
                            if count > 0 {
                                candidate = RecognizeResult {
                                    text,
                                    mean_confidence: confidence_sum / count as f32,
                                };
                            }
                        }
                        Ok(_) => {}
                        Err(reason) => {
                            error = Some(reason);
                        }
                    }
                }
                before_lines.push(RecognizedLine {
                    bbox_xyxy: line.bbox_xyxy,
                    text: base.text.clone(),
                    confidence: base.mean_confidence,
                    is_vertical: line.is_vertical,
                });
                after_lines.push(RecognizedLine {
                    bbox_xyxy: line.bbox_xyxy,
                    text: candidate.text.clone(),
                    confidence: candidate.mean_confidence,
                    is_vertical: line.is_vertical,
                });
                records.push(serde_json::json!({"index":index,"bounds":line.bbox_xyxy,"bucket":line.pred_char_count,"eligible":eligible,"raw_before":raw_before,"before":{"text":base.text,"confidence":base.mean_confidence},"after":{"text":candidate.text,"confidence":candidate.mean_confidence},"chunks":chunks_record,"error":error}));
            }
            let name = format!("{}.txt", path.file_stem().unwrap().to_str().unwrap());
            for (version, lines) in [("before", before_lines), ("after", after_lines)] {
                let directory = root.join(mode).join(version);
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(directory.join(&name), render(lines)).unwrap();
            }
            eprintln!("{mode} {}: {} crops", path.display(), records.len());
            pages.push(serde_json::json!({"image":path,"mode":mode,"lines":records}));
        }
    }
    assert_eq!(pages.len(), 40);
    std::fs::write(
        root.join("results.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"input_dimensions":[input_w,input_h],"native_groups":native_groups,"pages":pages}),
        )
        .unwrap(),
    )
    .unwrap();
}
