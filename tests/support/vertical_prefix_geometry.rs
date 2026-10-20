//! Geometry-only preflight; dummy text opens the count gate without image NN.
use super::*;

#[test]
#[ignore = "requires frozen 114-input manifest and new directory; no models or recognition"]
fn record_two_partition_geometry_without_nn() {
    let Some(path) = std::env::var_os("NDLOCR_PREFIX_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("NDLOCR_PREFIX_METADATA").unwrap()).unwrap(),
    )
    .unwrap();
    let output = std::path::PathBuf::from(std::env::var_os("NDLOCR_PREFIX_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 114);
    let mut records = Vec::new();
    let mut cuts = Vec::new();
    for input in inputs {
        let id = input["id"].as_str().unwrap();
        let rgb = std::fs::read(input["rgb"].as_str().unwrap()).unwrap();
        let dimensions = (
            input["width"].as_u64().unwrap() as usize,
            input["height"].as_u64().unwrap() as usize,
        );
        for model in metadata.as_array().unwrap() {
            let label = model["label"].as_str().unwrap();
            let dims = model["inputs"][0]["dimensions"].as_array().unwrap();
            let iw = dims[3].as_u64().unwrap() as usize;
            let ih = dims[2].as_u64().unwrap() as usize;
            let limit = model["outputs"][0]["max_raw_scalars_from_static_shape"]
                .as_u64()
                .unwrap() as usize;
            let base = RecognizeResult {
                text: "A".repeat(limit - 1),
                mean_confidence: 1.0,
            };
            let Some(plan) = plans(&base, &rgb, dimensions, (iw, ih), Some(limit)) else {
                records.push(serde_json::json!({"id":id,"model":label,"plan":false}));
                continue;
            };
            let mut routes = Vec::new();
            for (route, chunks) in plan.routes.iter().enumerate() {
                let mut parts = Vec::new();
                for (index, chunk) in chunks.iter().enumerate() {
                    let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                        &plan.column.original,
                        plan.column.width,
                        plan.column.height,
                        chunks,
                        index,
                    )
                    .unwrap();
                    let file = format!("{id}-{label}-{route}-{index}.rgb");
                    std::fs::write(output.join(&file), &crop.data).unwrap();
                    parts.push(serde_json::json!({"x_range":[chunk.x0,chunk.x1],"width":crop.width,"height":crop.height,"file":file,"space_before":chunk.space_before}));
                    if let Some(glyphs) = input["glyph_boxes"].as_array() {
                        for (glyph, box_) in glyphs.iter().enumerate() {
                            if let Some(box_) = box_.as_array() {
                                let y0 = box_[1].as_u64().unwrap() as usize;
                                let y1 = box_[3].as_u64().unwrap() as usize;
                                for boundary in [chunk.x0, chunk.x1] {
                                    if y0 < boundary && boundary < y1 {
                                        cuts.push(serde_json::json!({"id":id,"model":label,"route":route,"glyph":glyph,"bbox":box_,"boundary":boundary}));
                                    }
                                }
                            }
                        }
                    }
                }
                routes.push(parts);
            }
            records.push(serde_json::json!({"id":id,"model":label,"plan":true,"routes":routes}));
        }
    }
    std::fs::write(
        output.join("records.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    std::fs::write(
        output.join("glyph-cuts.json"),
        serde_json::to_vec_pretty(&cuts).unwrap(),
    )
    .unwrap();
    println!(
        "{} geometry pairs; {} interior glyph cuts; model loads and NN calls zero",
        records.len(),
        cuts.len()
    );
    assert!(
        cuts.is_empty(),
        "reject the whole proposal before NN, not selected inputs"
    );
}

#[test]
#[ignore = "private-only prefix geometry; frozen 270 RGBs, no models or recognition"]
fn record_verified_geometry_without_nn() {
    use std::io::Write;
    let Some(path) = std::env::var_os("NDLOCR_VERIFIED_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("NDLOCR_VERIFIED_METADATA").unwrap()).unwrap(),
    )
    .unwrap();
    let output = std::path::PathBuf::from(std::env::var_os("NDLOCR_VERIFIED_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    let expected: usize = std::env::var("NDLOCR_VERIFIED_EXPECT_INPUTS")
        .unwrap()
        .parse()
        .unwrap();
    let annotated: usize = std::env::var("NDLOCR_VERIFIED_EXPECT_ANNOTATED")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!((expected, annotated), (270, 162));
    assert_eq!(inputs.len(), expected);
    assert_eq!(
        inputs
            .iter()
            .filter(|input| input["glyph_boxes"].is_array())
            .count(),
        annotated
    );
    let mut records = Vec::new();
    let mut cuts = Vec::new();
    let mut chunk_count = 0;
    for profile in ["legacy", "supported"] {
        for model in metadata.as_array().unwrap() {
            let label = model["label"].as_str().unwrap();
            let dims = model["inputs"][0]["dimensions"].as_array().unwrap();
            let iw = dims[3].as_u64().unwrap() as usize;
            let ih = dims[2].as_u64().unwrap() as usize;
            let archive_name = format!("chunks-{profile}-{label}.zip");
            let mut archive =
                zip::ZipWriter::new(std::fs::File::create(output.join(&archive_name)).unwrap());
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for input in inputs {
                let id = input["id"].as_str().unwrap();
                assert!(
                    id.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                );
                let rgb = std::fs::read(input["rgb"].as_str().unwrap()).unwrap();
                let width = input["width"].as_u64().unwrap() as usize;
                let height = input["height"].as_u64().unwrap() as usize;
                let original = crate::pipeline::vertical_segment::rotate_vertical_column_rgb_u8(
                    &rgb, width, height,
                )
                .unwrap();
                for prep in ["raw", "otsu"] {
                    let support = if prep == "raw" {
                        Some(original.data.clone())
                    } else {
                        super::super::prepare_otsu(&rgb, width, height)
                            .map(|(column, _)| column.support)
                    };
                    for divisor in [1, 2] {
                        let mut record = serde_json::json!({"profile":profile,"id":id,"model":label,"preparation":prep,"budget_divisor":divisor,"input":[iw,ih],"support_input_width":if profile=="legacy" {iw/divisor} else {iw}});
                        let Some(ref support) = support else {
                            record["error"] = serde_json::json!("no two-level Otsu support");
                            records.push(record);
                            continue;
                        };
                        let split = if profile == "legacy" {
                            crate::pipeline::line_chunks::split_rotated_vertical_column_for_input(
                                support,
                                original.width,
                                original.height,
                                220,
                                iw / divisor,
                                ih,
                            )
                        } else {
                            crate::pipeline::line_chunks::split_supported_vertical_column_for_input(
                                support,
                                original.width,
                                original.height,
                                220,
                                iw,
                                ih,
                                divisor,
                            )
                        };
                        match split {
                            Ok(chunks) => {
                                let mut parts = Vec::new();
                                for (index, chunk) in chunks.iter().enumerate() {
                                    let crop =
                                        crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                                            &original.data,
                                            original.width,
                                            original.height,
                                            &chunks,
                                            index,
                                        )
                                        .unwrap();
                                    assert_eq!(
                                        (crop.width, crop.height),
                                        (chunk.x1 - chunk.x0, original.height)
                                    );
                                    let file = format!("{id}-{prep}-{divisor}-{index}.rgb");
                                    archive.start_file(&file, options).unwrap();
                                    archive.write_all(&crop.data).unwrap();
                                    parts.push(serde_json::json!({"x_range":[chunk.x0,chunk.x1],"width":crop.width,"height":crop.height,"archive":archive_name,"file":file,"space_before":chunk.space_before}));
                                    chunk_count += 1;
                                    if let Some(glyphs) = input["glyph_boxes"].as_array() {
                                        for (glyph, box_) in glyphs.iter().enumerate() {
                                            if let Some(box_) = box_.as_array() {
                                                let a = box_[1].as_u64().unwrap() as usize;
                                                let b = box_[3].as_u64().unwrap() as usize;
                                                for boundary in [chunk.x0, chunk.x1] {
                                                    if a < boundary && boundary < b {
                                                        cuts.push(serde_json::json!({"profile":profile,"id":id,"model":label,"preparation":prep,"budget_divisor":divisor,"glyph":glyph,"bbox":box_,"boundary":boundary}));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                record["chunks"] = serde_json::json!(parts);
                            }
                            Err(error) => record["error"] = serde_json::json!(error),
                        }
                        records.push(record);
                    }
                }
            }
            archive.finish().unwrap();
        }
    }
    std::fs::write(
        output.join("records.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    std::fs::write(
        output.join("glyph-cuts.json"),
        serde_json::to_vec_pretty(&cuts).unwrap(),
    )
    .unwrap();
    println!(
        "{} geometry records; {} raw RGB chunks in lossless ZIP; {} glyph-cut events; NN0",
        records.len(),
        chunk_count,
        cuts.len()
    );
    assert!(
        cuts.iter().all(|cut| cut["profile"] != "supported"),
        "reject whole supported candidate before NN on any annotated glyph cut"
    );
}

#[test]
#[ignore = "fixed real raw Auto results; geometric plans only, no model load or NN"]
fn record_real_raw_plans_without_nn() {
    use std::io::Write;
    let Some(path) = std::env::var_os("NDLOCR_VERIFIED_REAL_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let out = std::path::PathBuf::from(std::env::var_os("NDLOCR_VERIFIED_REAL_OUTPUT").unwrap());
    std::fs::create_dir(&out).unwrap();
    let items = manifest["items"].as_array().unwrap();
    assert!([810, 1242].contains(&items.len()));
    let mut rows = Vec::new();
    let mut archive = zip::ZipWriter::new(std::fs::File::create(out.join("chunks.zip")).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for item in items {
        let id = item["id"].as_str().unwrap();
        assert!(
            id.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        );
        let label = item["model"].as_str().unwrap();
        assert!(["30", "50", "100"].contains(&label));
        let mut row = serde_json::json!({"id":id,"model":label,"base":item["base"],"limit":item["limit"],"plan":false});
        if item["base"]["error"].is_string() {
            row["reason"] = serde_json::json!("original API error preserved");
            rows.push(row);
            continue;
        }
        let base = RecognizeResult {
            text: item["base"]["text"].as_str().unwrap().into(),
            mean_confidence: f32::from_bits(
                item["base"]["mean_bits"]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
        };
        let dimensions = (
            item["width"].as_u64().unwrap() as usize,
            item["height"].as_u64().unwrap() as usize,
        );
        let input = (
            item["input"][0].as_u64().unwrap() as usize,
            item["input"][1].as_u64().unwrap() as usize,
        );
        let limit = item["limit"].as_u64().unwrap() as usize;
        let rgb = std::fs::read(item["rgb"].as_str().unwrap()).unwrap();
        let Some(plan) = plans(&base, &rgb, dimensions, input, Some(limit)) else {
            rows.push(row);
            continue;
        };
        let mut routes = Vec::new();
        for (route, chunks) in plan.routes.iter().enumerate() {
            let mut parts = Vec::new();
            for (index, chunk) in chunks.iter().enumerate() {
                let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                    &plan.column.original,
                    plan.column.width,
                    plan.column.height,
                    chunks,
                    index,
                )
                .unwrap();
                let file = format!("{id}-{label}-{route}-{index}.rgb");
                archive.start_file(&file, options).unwrap();
                archive.write_all(&crop.data).unwrap();
                parts.push(serde_json::json!({"x_range":[chunk.x0,chunk.x1],"width":crop.width,"height":crop.height,
                    "file":file,"archive":"chunks.zip","space_before":chunk.space_before}));
            }
            routes.push(parts);
        }
        row["plan"] = serde_json::json!(true);
        row["routes"] = serde_json::json!(routes);
        rows.push(row);
    }
    archive.finish().unwrap();
    std::fs::write(
        out.join("records.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    println!(
        "{} real raw plans; {} eligible; model loads/NN zero",
        rows.len(),
        rows.iter().filter(|r| r["plan"] == true).count()
    );
}

#[test]
#[ignore = "literal saved NN records only; combines with the actual private Rust policy"]
fn combine_all_saved_real_plans_without_nn() {
    let Some(path) = std::env::var_os("NDLOCR_VERIFIED_COMBINE_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let out = std::path::PathBuf::from(std::env::var_os("NDLOCR_VERIFIED_COMBINE_OUTPUT").unwrap());
    std::fs::create_dir(&out).unwrap();
    let items = manifest["items"].as_array().unwrap();
    assert!([810, 1242].contains(&items.len()));
    let mut rows = Vec::new();
    for item in items {
        let mut row = serde_json::json!({"id":item["id"],"model":item["model"],"base":item["base"],
            "plan":item["plan"],"adopted":false,"result":item["base"]});
        if item["plan"] == true {
            let convert = |parts: &serde_json::Value| -> Option<Vec<RecognizeResult>> {
                parts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|part| {
                        if part["error"].is_string() {
                            return None;
                        }
                        Some(RecognizeResult {
                            text: part["text"].as_str().unwrap().into(),
                            mean_confidence: f32::from_bits(
                                part["mean_bits"].as_u64().unwrap().try_into().unwrap(),
                            ),
                        })
                    })
                    .collect()
            };
            let parts = item["routes"].as_array().unwrap();
            assert_eq!(parts.len(), 2);
            if let (Some(a), Some(b)) = (convert(&parts[0]), convert(&parts[1])) {
                if let Some(result) = combine(
                    item["base"]["text"].as_str().unwrap(),
                    item["limit"].as_u64().unwrap() as usize,
                    &a,
                    &b,
                ) {
                    row["adopted"] = serde_json::json!(true);
                    row["result"] = serde_json::json!({"text":result.text,"mean_bits":result.mean_confidence.to_bits()});
                }
            }
        }
        rows.push(row);
    }
    std::fs::write(
        out.join("records.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    println!(
        "{} input-model records; {} adopted; model loads/NN zero",
        rows.len(),
        rows.iter().filter(|r| r["adopted"] == true).count()
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "fixed raw Horizontal chunk audit; requires frozen per-model manifest and new output"]
fn capture_all_frozen_verified_chunks_with_raw_candidates() {
    use crate::infer::{cached::ParseqSession, parseq::ParseqOrientation};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    let Some(path) = std::env::var_os("NDLOCR_VERIFIED_NN_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let output = std::path::PathBuf::from(std::env::var_os("NDLOCR_VERIFIED_NN_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let items = manifest["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert_eq!(
        items.len(),
        manifest["expected_items"].as_u64().unwrap() as usize
    );
    let session = ParseqSession::load(
        std::path::Path::new(manifest["model"]["path"].as_str().unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let dims = manifest["input_dimensions"].as_array().unwrap();
    assert_eq!(
        session.input_dimensions(),
        (
            dims[0].as_u64().unwrap() as usize,
            dims[1].as_u64().unwrap() as usize
        )
    );
    std::fs::write(output.join("model-loaded.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "model":manifest["model"],"loaded_session":true,"input_dimensions":session.input_dimensions()})).unwrap()).unwrap();
    let mut archives: BTreeMap<String, zip::ZipArchive<std::fs::File>> = BTreeMap::new();
    let mut actual_inputs =
        zip::ZipWriter::new(std::fs::File::create(output.join("actual-rgb-inputs.zip")).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut rows = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let id = item["id"].as_str().unwrap();
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'_'));
        let archive = item["archive"].as_str().unwrap();
        let zipped = archives.entry(archive.into()).or_insert_with(|| {
            zip::ZipArchive::new(std::fs::File::open(archive).unwrap()).unwrap()
        });
        let rgb = {
            let mut part = zipped.by_name(item["member"].as_str().unwrap()).unwrap();
            let mut rgb = Vec::new();
            part.read_to_end(&mut rgb).unwrap();
            rgb
        };
        let width = item["width"].as_u64().unwrap() as usize;
        let height = item["height"].as_u64().unwrap() as usize;
        assert_eq!(
            width.checked_mul(height).and_then(|n| n.checked_mul(3)),
            Some(rgb.len())
        );
        actual_inputs
            .start_file(format!("{id}.rgb"), options)
            .unwrap();
        actual_inputs.write_all(&rgb).unwrap();
        let started = std::time::Instant::now();
        let actual = session.recognize_rgb_u8_candidates_with_orientation(
            &rgb,
            width,
            height,
            ParseqOrientation::Horizontal,
            1,
        );
        let mut row = serde_json::json!({"id":id,"width":width,"height":height,"expected_rgb_sha256":item["sha256"],
            "orientation":"Horizontal","top_k":1,"api_calls":1,"internal_session_run_calls":null,"seconds":started.elapsed().as_secs_f64()});
        match actual {
            Ok(actual) => {
                row["text"] = serde_json::json!(actual.recognition.text);
                row["mean_bits"] = serde_json::json!(actual.recognition.mean_confidence.to_bits());
                row["stop_reason"] = serde_json::json!(format!("{:?}", actual.stop_reason));
                row["steps"]=serde_json::json!(actual.steps.into_iter().map(|step|serde_json::json!({"timestep":step.timestep,
                    "text_range":step.text_range.map(|range|[range.start,range.end]),"candidates":step.candidates.into_iter().map(|c|serde_json::json!({
                        "token_id":c.token_id,"character":c.character,"probability":c.probability,"probability_bits":c.probability.to_bits()})).collect::<Vec<_>>()})).collect::<Vec<_>>());
            }
            Err(error) => row["error"] = serde_json::json!(format!("{error:#}")),
        }
        std::fs::write(
            output.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&row).unwrap(),
        )
        .unwrap();
        rows.push(row);
        if (index + 1) % 100 == 0 {
            println!("{} of {} raw chunks captured", index + 1, items.len());
        }
    }
    actual_inputs.finish().unwrap();
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    println!(
        "{} raw image API calls; all successes/errors and actual input RGB saved",
        rows.len()
    );
}
