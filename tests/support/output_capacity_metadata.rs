//! Load the pinned models and inspect metadata only; no image and no Session.run.
use super::*;

#[test]
#[ignore = "metadata diagnostic; requires model manifest and new output directory, no recognition"]
fn capture_output_capacity_metadata_without_nn() {
    let Some(path) = std::env::var_os("NDLOCR_OUTPUT_CAPACITY_MODEL_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let out = std::path::PathBuf::from(std::env::var_os("NDLOCR_OUTPUT_CAPACITY_OUTPUT").unwrap());
    std::fs::create_dir(&out).unwrap();
    let charset_path = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let charset =
        load_charset_from_yaml_str(&std::fs::read_to_string(charset_path).unwrap()).unwrap();
    let mut records = Vec::new();
    for model in manifest["models"].as_array().unwrap() {
        let session = build_session(Path::new(model["path"].as_str().unwrap()), 1).unwrap();
        let mut inputs = Vec::new();
        for input in session.inputs() {
            let dimensions = input.dtype().tensor_shape().map(|shape| {
                let dims: &[i64] = shape;
                dims.to_vec()
            });
            inputs.push(serde_json::json!({"name":input.name(),"dtype_debug":format!("{:?}",input.dtype()),"dimensions":dimensions}));
        }
        let mut outputs = Vec::new();
        for output in session.outputs() {
            let dimensions = output.dtype().tensor_shape().map(|shape| {
                let dims: &[i64] = shape;
                dims.to_vec()
            });
            let bound = dimensions.as_deref().and_then(|shape| {
                super::super::output_capacity::static_frame_limit(shape, charset.len())
            });
            outputs.push(serde_json::json!({"name":output.name(),"dtype_debug":format!("{:?}",output.dtype()),"dimensions":dimensions,"max_raw_scalars_from_static_shape":bound}));
        }
        records.push(serde_json::json!({"label":model["label"],"path":model["path"],"charset_scalars":charset.len(),"inputs":inputs,"outputs":outputs,"image_recognition_calls":0,"session_run_calls":0}));
    }
    std::fs::write(
        out.join("results.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    println!(
        "{} models loaded; metadata inspected, no images or Session.run",
        records.len()
    );
}

/// Audit the existing candidate decoder; no new recognition policy or GT input.
#[test]
#[ignore = "raw stop diagnostic; requires frozen RGB/model manifests and a new output directory"]
fn capture_raw_stops_for_all_frozen_vertical_inputs() {
    let Some(path) = std::env::var_os("NDLOCR_RAW_STOP_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let out = std::path::PathBuf::from(std::env::var_os("NDLOCR_RAW_STOP_OUTPUT").unwrap());
    std::fs::create_dir(&out).unwrap();
    let session = ParseqSession::load(
        Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 54);
    let mut rows = Vec::new();
    for input in inputs {
        let rgb = std::fs::read(input["rgb"].as_str().unwrap()).unwrap();
        let width = input["width"].as_u64().unwrap() as usize;
        let height = input["height"].as_u64().unwrap() as usize;
        let started = std::time::Instant::now();
        let result = session.recognize_rgb_u8_candidates_with_orientation(
            &rgb,
            width,
            height,
            ParseqOrientation::Auto,
            1,
        );
        let mut row = serde_json::json!({"id":input["id"],"api_calls":1,"internal_session_run_calls":null,"seconds":started.elapsed().as_secs_f64()});
        match result {
            Ok(result) => {
                row["text"] = serde_json::json!(result.recognition.text);
                row["mean_bits"] = serde_json::json!(result.recognition.mean_confidence.to_bits());
                row["stop_reason"] = serde_json::json!(format!("{:?}", result.stop_reason));
                row["steps"] = serde_json::json!(result.steps.into_iter().map(|step| {
                    serde_json::json!({"timestep":step.timestep,"text_range":step.text_range.map(|r|[r.start,r.end]),
                        "candidates":step.candidates.into_iter().map(|c|serde_json::json!({"token_id":c.token_id,"character":c.character,"probability":c.probability,"probability_bits":c.probability.to_bits()})).collect::<Vec<_>>()})
                }).collect::<Vec<_>>());
            }
            Err(error) => row["error"] = serde_json::json!(format!("{error:#}")),
        }
        std::fs::write(
            out.join(format!("{}.json", input["id"].as_str().unwrap())),
            serde_json::to_vec_pretty(&row).unwrap(),
        )
        .unwrap();
        rows.push(row);
    }
    std::fs::write(
        out.join("results.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    println!(
        "{} frozen inputs captured; all successes and errors retained",
        rows.len()
    );
}
