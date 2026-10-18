#![cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires pinned models and output path"]
fn record_page_band_runtime_matrix() {
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    use std::path::Path;
    let root = std::env::var("NDLOCR_BAND_RUNTIME_OUTPUT").expect("output directory");
    let manifest = std::env::var("NDLOCR_BAND_MANIFEST")
        .unwrap_or_else(|_| "tmp/page-band-fallback/evaluation-manifest.json".into());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    for (label, model) in [
        (
            "runtime30",
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "runtime100",
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
    ] {
        let runtime = PageRecognizer::load(
            Path::new("models/deim-s-1024x1024.onnx"),
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        for (suffix, opts) in [
            (
                "eval",
                PageRecognizerOptions {
                    det_conf_threshold: 0.25,
                    min_line_confidence: 0.3,
                    drop_empty_text: true,
                },
            ),
            ("default", PageRecognizerOptions::default()),
        ] {
            let dir = Path::new(&root).join(format!("{label}_{suffix}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut rows = Vec::new();
            for row in manifest["inputs"].as_array().unwrap() {
                let image = load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
                let page = runtime
                    .recognize_page_rgb_u8(&image.data, image.width, image.height, opts)
                    .unwrap();
                let text = page
                    .lines
                    .iter()
                    .map(|l| l.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(
                    dir.join(format!("{}.txt", row["id"].as_str().unwrap())),
                    &text,
                )
                .unwrap();
                rows.push(serde_json::json!({"id":row["id"],"text":text,"lines":page.lines.iter().map(|l|serde_json::json!({"bbox":l.bbox_xyxy,"text":l.text,"confidence":l.confidence})).collect::<Vec<_>>()}));
            }
            assert_eq!(rows.len(), 127);
            std::fs::write(
                dir.join("results.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
#[ignore = "diagnostic; requires pinned PARSeq models"]
fn inspect_sparse_band_chunks() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{cached::ParseqSession, parseq::ParseqOrientation},
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    use std::path::Path;
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/dense_line/images/spaced_short.png",
    ))
    .unwrap();
    let ds =
        fallback_line_detections_from_bands(&image.data, image.width, image.height, 220, 25, 45);
    let crops = prepare_line_crops(&image.data, image.width, image.height, ds, 0).unwrap();
    let mut out = Vec::new();
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
        for c in &crops {
            let c = &c.crop;
            let mut row = serde_json::json!({"model":name,"crop_size":[c.width,c.height]});
            match split_line_for_input(&c.data, c.width, c.height, 220, iw, ih) {
                Err(error) => row["error"] = error.into(),
                Ok(chunks) => {
                    let mut parts = Vec::new();
                    let mut compact_pieces = Vec::new();
                    for (i, ch) in chunks.iter().enumerate() {
                        let p =
                            crop_line_chunk_rgb_u8(&c.data, c.width, c.height, &chunks, i).unwrap();
                        let compact = ndlocr_lite_rs::pipeline::horizontal_margins::compact_horizontal_margins(&p.data,p.width,p.height,p.height/16).unwrap();
                        let compact_auto = model
                            .recognize_rgb_u8_unprocessed(
                                &compact.image.data,
                                compact.image.width,
                                compact.image.height,
                            )
                            .unwrap();
                        let compact_rec = model
                            .recognize_rgb_u8_unprocessed_with_orientation(
                                &compact.image.data,
                                compact.image.width,
                                compact.image.height,
                                ParseqOrientation::Horizontal,
                            )
                            .unwrap();
                        let pad = compact.image.height / 8;
                        let ph = compact.image.height + pad * 2;
                        let mut padded = vec![255; compact.image.width * ph * 3];
                        let start = pad * compact.image.width * 3;
                        padded[start..start + compact.image.data.len()]
                            .copy_from_slice(&compact.image.data);
                        let padded_rec = model
                            .recognize_rgb_u8_unprocessed_with_orientation(
                                &padded,
                                compact.image.width,
                                ph,
                                ParseqOrientation::Horizontal,
                            )
                            .unwrap();
                        let result = model
                            .recognize_rgb_u8_unprocessed_with_orientation(
                                &p.data,
                                p.width,
                                p.height,
                                ParseqOrientation::Horizontal,
                            )
                            .unwrap();
                        parts.push(serde_json::json!({"bounds":[ch.x0,ch.x1],"space_before":ch.space_before,"gap_before":ch.gap_before,"size":[p.width,p.height],"text":result.text,"confidence":result.mean_confidence,"compact_size":[compact.image.width,compact.image.height],"content_width":compact.content_width,"compact_text":compact_rec.text,"compact_confidence":compact_rec.mean_confidence,"padded_text":padded_rec.text,"padded_confidence":padded_rec.mean_confidence,"padding":pad,"auto_text":compact_auto.text,"auto_confidence":compact_auto.mean_confidence}));
                        compact_pieces.push(compact.image);
                    }
                    let cw: usize = compact_pieces.iter().map(|p| p.width).sum();
                    let ch = c.height;
                    let mut context = vec![255; cw * ch * 3];
                    let mut offset = 0;
                    for p in &compact_pieces {
                        for y in 0..ch {
                            let from = y * p.width * 3;
                            let to = (y * cw + offset) * 3;
                            context[to..to + p.width * 3]
                                .copy_from_slice(&p.data[from..from + p.width * 3]);
                        }
                        offset += p.width;
                    }
                    let context_rec = model
                        .recognize_rgb_u8_unprocessed_with_orientation(
                            &context,
                            cw,
                            ch,
                            ParseqOrientation::Horizontal,
                        )
                        .unwrap();
                    row["context"] = serde_json::json!({"size":[cw,ch],"text":context_rec.text,"confidence":context_rec.mean_confidence});
                    row["parts"] = parts.into();
                }
            }
            out.push(row);
        }
    }
    std::fs::write(
        "tmp/page-band-fallback/sparse-diagnosis.json",
        serde_json::to_vec_pretty(&out).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "diagnostic; requires pinned PARSeq models"]
fn inspect_band_aspect_padding() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{cached::ParseqSession, parseq::ParseqOrientation},
        io::load_rgb_u8,
    };
    use std::path::Path;
    let mut rows = Vec::new();
    for model_name in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let model = ParseqSession::load(
            &Path::new("models").join(model_name),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let (iw, ih) = model.input_dimensions();
        for id in [
            "repeated_i_gothic_png",
            "repeated_i_mincho_png",
            "single_ja_gothic_png",
            "sentence_en_mincho_png",
        ] {
            let image = load_rgb_u8(
                &Path::new("tests/fixtures/page_band_fallback/images").join(format!("{id}.png")),
            )
            .unwrap();
            let ds = fallback_line_detections_from_bands(
                &image.data,
                image.width,
                image.height,
                220,
                25,
                45,
            );
            let crops = prepare_line_crops(&image.data, image.width, image.height, ds, 0).unwrap();
            for p in crops {
                let p = p.crop;
                let target = (p.height * iw).div_ceil(ih).max(p.width);
                let left = (target - p.width) / 2;
                let mut padded = vec![255; target * p.height * 3];
                for y in 0..p.height {
                    let from = y * p.width * 3;
                    let to = (y * target + left) * 3;
                    padded[to..to + p.width * 3].copy_from_slice(&p.data[from..from + p.width * 3]);
                }
                let copies = target.div_ceil(p.width);
                let tw = p.width * copies;
                let mut tiled = vec![255; tw * p.height * 3];
                for y in 0..p.height {
                    for x in 0..tw {
                        let from = (y * p.width + x % p.width) * 3;
                        let to = (y * tw + x) * 3;
                        tiled[to..to + 3].copy_from_slice(&p.data[from..from + 3]);
                    }
                }
                let tile_rec = model
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &tiled,
                        tw,
                        p.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                let a = model
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &p.data,
                        p.width,
                        p.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                let b = model
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &padded,
                        target,
                        p.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                rows.push(serde_json::json!({"id":id,"model":model_name,"original_size":[p.width,p.height],"prepared_size":[target,p.height],"before":a.text,"before_confidence":a.mean_confidence,"after":b.text,"after_confidence":b.mean_confidence,"tile_size":[tw,p.height],"tile_text":tile_rec.text,"tile_confidence":tile_rec.mean_confidence,"copies":copies}));
            }
        }
    }
    std::fs::write(
        "tmp/page-band-fallback/aspect-diagnosis.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "diagnostic; requires pinned models"]
fn inspect_band_native_capacity_candidates() {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    use std::path::Path;
    let mut rows = Vec::new();
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let session = ndlocr_lite_rs::infer::cached::ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        for image in [
            "tests/fixtures/dense_line/images/repeated_i.png",
            "tests/fixtures/long_line_split/images/uri_mincho36.png",
        ] {
            let im = load_rgb_u8(Path::new(image)).unwrap();
            let ds =
                fallback_line_detections_from_bands(&im.data, im.width, im.height, 220, 25, 45);
            let crops = prepare_line_crops(&im.data, im.width, im.height, ds, 0).unwrap();
            for c in crops {
                let c = c.crop;
                let full = session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &c.data,
                        c.width,
                        c.height,
                        ndlocr_lite_rs::infer::parseq::ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                let (iw, ih) = session.input_dimensions();
                let mut chunks = Vec::new();
                if let Ok(cs) = split_line_for_input(&c.data, c.width, c.height, 220, iw, ih) {
                    for (i, ch) in cs.iter().enumerate() {
                        let p = crop_line_chunk_rgb_u8(&c.data, c.width, c.height, &cs, i).unwrap();
                        let rec = session
                            .recognize_rgb_u8_unprocessed_with_orientation(
                                &p.data,
                                p.width,
                                p.height,
                                ndlocr_lite_rs::infer::parseq::ParseqOrientation::Horizontal,
                            )
                            .unwrap();
                        chunks.push(serde_json::json!({"x0":ch.x0,"x1":ch.x1,"text":rec.text,"confidence":rec.mean_confidence}));
                    }
                }
                rows.push(serde_json::json!({"image":image,"model":model,"crop":[c.width,c.height],"native":[iw,ih],"full_text":full.text,"full_confidence":full.mean_confidence,"chunks":chunks}));
            }
        }
    }
    std::fs::write(
        "tmp/page-band-fallback/native-capacity-probe.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
