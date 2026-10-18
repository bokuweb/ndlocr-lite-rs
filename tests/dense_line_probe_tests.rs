#![cfg(feature = "onnx")]

/// 全幅100の停止理由と、元RGBのモデル別chunkを辞書/真値なしで比較する。
#[test]
#[ignore = "diagnostic; requires all pinned models and output path"]
fn inspect_dense_line_stop_reasons_and_model_context() {
    use ndlocr_lite_rs::{
        infer::{cached::ParseqSession, parseq::ParseqOrientation},
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    let names = [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ];
    let models: Vec<_> = names
        .iter()
        .map(|name| {
            ParseqSession::load(
                &std::path::Path::new("models").join(name),
                std::path::Path::new("models/NDLmoji.yaml"),
            )
            .unwrap()
        })
        .collect();
    let rows: serde_json::Value = serde_json::from_slice(
        &std::fs::read("tests/fixtures/long_line_split/manifest.json").unwrap(),
    )
    .unwrap();
    let mut results = Vec::new();
    for row in rows["rows"].as_array().unwrap() {
        let image = load_rgb_u8(std::path::Path::new(row["image"].as_str().unwrap())).unwrap();
        let (iw, ih) = models[2].input_dimensions();
        let full = models[2]
            .recognize_rgb_u8_candidates_with_orientation(
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Horizontal,
                3,
            )
            .unwrap();
        let wide = image.width as u128 * ih as u128 > 2 * image.height as u128 * iw as u128;
        let mut columns = 0;
        let mut groups = 0;
        let mut previous = false;
        let mut top = image.height;
        let mut bottom = 0;
        for x in 0..image.width {
            let mut column = false;
            for y in 0..image.height {
                let p = (y * image.width + x) * 3;
                if image.data[p..p + 3]
                    .iter()
                    .map(|&v| u16::from(v))
                    .sum::<u16>()
                    < 660
                {
                    column = true;
                    top = top.min(y);
                    bottom = bottom.max(y + 1);
                }
            }
            if column {
                columns += 1;
                if !previous {
                    groups += 1;
                }
            }
            previous = column;
        }
        let mut variants = Vec::new();
        if wide {
            for (name, model) in names.iter().zip(&models) {
                let (iw, ih) = model.input_dimensions();
                match split_line_for_input(&image.data, image.width, image.height, 220, iw, ih) {
                    Ok(chunks) => {
                        let mut text = String::new();
                        let mut recorded = Vec::new();
                        for (index, chunk) in chunks.iter().enumerate() {
                            let crop = crop_line_chunk_rgb_u8(
                                &image.data,
                                image.width,
                                image.height,
                                &chunks,
                                index,
                            )
                            .unwrap();
                            let result = model
                                .recognize_rgb_u8_candidates_with_orientation(
                                    &crop.data,
                                    crop.width,
                                    crop.height,
                                    ParseqOrientation::Horizontal,
                                    3,
                                )
                                .unwrap();
                            if chunk.insert_space_before(
                                &text,
                                &result.recognition.text,
                                image.height,
                            ) {
                                text.push(' ');
                            }
                            text.push_str(&result.recognition.text);
                            recorded.push(serde_json::json!({"bounds":[chunk.x0,chunk.x1],"size":[crop.width,crop.height],"raw":result.recognition.text,"confidence":result.recognition.mean_confidence,"stop_reason":format!("{:?}",result.stop_reason)}));
                        }
                        variants
                            .push(serde_json::json!({"model":name,"text":text,"chunks":recorded}));
                    }
                    Err(error) => variants.push(serde_json::json!({"model":name,"error":error})),
                }
            }
        }
        let steps:Vec<_>=full.steps.iter().map(|step|serde_json::json!({"timestep":step.timestep,"emitted":step.text_range.is_some(),"candidates":step.candidates.iter().map(|c|serde_json::json!({"character":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect();
        results.push(serde_json::json!({"id":row["id"],"size":[image.width,image.height],"ink_columns":columns,"ink_groups":groups,"body_height":bottom.saturating_sub(top),"wide":wide,"raw":full.recognition.text,"confidence":full.recognition.mean_confidence,"stop_reason":format!("{:?}",full.stop_reason),"steps":steps,"variants":variants}));
    }
    assert_eq!(results.len(), 26);
    std::fs::write(
        std::env::var_os("NDLOCR_DENSE_LINE_OUTPUT").expect("output path"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
}
