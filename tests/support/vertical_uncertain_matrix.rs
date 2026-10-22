use super::*;

fn matrix_result(
    value: anyhow::Result<crate::infer::parseq::RecognizeResult>,
) -> serde_json::Value {
    match value {
        Ok(value) => {
            serde_json::json!({"text":value.text,"mean_bits":value.mean_confidence.to_bits()})
        }
        Err(error) => serde_json::json!({"error":format!("{error:#}")}),
    }
}

fn matrix_batch(
    value: anyhow::Result<Vec<crate::infer::parseq::RecognizeResult>>,
    count: usize,
) -> (Vec<serde_json::Value>, Option<String>) {
    match value {
        Ok(values) if values.len() == count => (
            values.into_iter().map(|v| matrix_result(Ok(v))).collect(),
            None,
        ),
        Ok(values) => {
            let error = format!(
                "whole batch cardinality: expected {count}, got {}",
                values.len()
            );
            (
                vec![serde_json::json!({"error":error,"origin":"whole_batch"}); count],
                Some(error),
            )
        }
        Err(error) => {
            let error = format!("{error:#}");
            (
                vec![serde_json::json!({"error":error,"origin":"whole_batch"}); count],
                Some(error),
            )
        }
    }
}

#[test]
#[ignore = "all frozen RGB, eight real API routes, fixed model and new output directory"]
fn capture_all_uncertain_native_routes_without_gt() {
    use crate::infer::parseq::ParseqOrientation;
    let Some(path) = std::env::var_os("NDLOCR_UNCERTAIN_MATRIX_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let label = std::env::var("NDLOCR_UNCERTAIN_MATRIX_MODEL").unwrap();
    let output =
        std::path::PathBuf::from(std::env::var_os("NDLOCR_UNCERTAIN_MATRIX_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let model = manifest["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["label"] == label)
        .unwrap();
    let cases: Vec<_> = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["label"] == label)
        .collect();
    let count = manifest["expected_per_model"].as_u64().unwrap() as usize;
    assert!([72, 414].contains(&count));
    assert_eq!(cases.len(), count);
    let charset = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let raw = ParseqSession::load(Path::new(model["path"].as_str().unwrap()), &charset).unwrap();
    let normal = ParseqPool::load(Path::new(model["path"].as_str().unwrap()), &charset, 1).unwrap();
    let fragment =
        ParseqPool::load_for_fragments(Path::new(model["path"].as_str().unwrap()), &charset, 1)
            .unwrap();
    std::fs::write(output.join("models-loaded.json"),serde_json::to_vec_pretty(&serde_json::json!({"model":model,"model_loads":3,"input_dimensions":raw.input_dimensions()})).unwrap()).unwrap();
    let mut images = Vec::new();
    for case in &cases {
        let rgb = std::fs::read(case["rgb"].as_str().unwrap()).unwrap();
        let (w, h) = (
            case["width"].as_u64().unwrap() as usize,
            case["height"].as_u64().unwrap() as usize,
        );
        assert_eq!(w * h * 3, rgb.len());
        images.push((rgb, w, h));
    }
    let mut rows = Vec::new();
    let mut calls = 0usize;
    let mut errors = 0usize;
    for (index, (case, (rgb, w, h))) in cases.iter().zip(&images).enumerate() {
        let id = case["id"].as_str().unwrap();
        assert!(
            id.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        );
        let raw_auto = matrix_result(raw.recognize_rgb_u8_unprocessed(rgb, *w, *h));
        calls += 1;
        let raw_horizontal = matrix_result(raw.recognize_rgb_u8_unprocessed_with_orientation(
            rgb,
            *w,
            *h,
            ParseqOrientation::Horizontal,
        ));
        calls += 1;
        let normal_auto = matrix_result(normal.recognize_rgb_u8(rgb, *w, *h));
        calls += 1;
        let normal_horizontal = matrix_result(normal.recognize_horizontal_rgb_u8(rgb, *w, *h));
        calls += 1;
        let fragment_auto = matrix_result(fragment.recognize_rgb_u8(rgb, *w, *h));
        calls += 1;
        let fragment_horizontal = matrix_result(fragment.recognize_horizontal_rgb_u8(rgb, *w, *h));
        calls += 1;
        for v in [
            &raw_auto,
            &raw_horizontal,
            &normal_auto,
            &normal_horizontal,
            &fragment_auto,
            &fragment_horizontal,
        ] {
            errors += usize::from(v["error"].is_string());
        }
        let row = serde_json::json!({"id":id,"label":label,"raw_auto":raw_auto,"raw_horizontal":raw_horizontal,
            "normal_auto":normal_auto,"normal_horizontal":normal_horizontal,"fragment_auto":fragment_auto,"fragment_horizontal":fragment_horizontal});
        std::fs::write(
            output.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&row).unwrap(),
        )
        .unwrap();
        rows.push(row);
        if (index + 1) % 50 == 0 {
            println!(
                "{label}: {} of{count} single-route images; {} outer image APIs",
                index + 1,
                calls
            );
        }
    }
    let items: Vec<_> = images
        .iter()
        .map(|(rgb, w, h)| (rgb.as_slice(), *w, *h))
        .collect();
    let (batch, batch_error) = matrix_batch(fragment.recognize_batch_rgb_u8(&items), items.len());
    calls += 1;
    let (exclusive, exclusive_error) = matrix_batch(
        fragment.recognize_batch_single_session_rgb_u8(&items),
        items.len(),
    );
    calls += 1;
    errors += usize::from(batch_error.is_some()) + usize::from(exclusive_error.is_some());
    for ((row, batch), exclusive) in rows.iter_mut().zip(batch).zip(exclusive) {
        row["fragment_batch_auto"] = batch;
        row["fragment_exclusive_batch_auto"] = exclusive;
        std::fs::write(
            output.join(format!("{}.json", row["id"].as_str().unwrap())),
            serde_json::to_vec_pretty(row).unwrap(),
        )
        .unwrap();
    }
    let equivalent = rows.iter().all(|r| {
        r["fragment_auto"] == r["fragment_batch_auto"]
            && r["fragment_auto"] == r["fragment_exclusive_batch_auto"]
    });
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    std::fs::write(output.join("summary.json"),serde_json::to_vec_pretty(&serde_json::json!({"label":label,"inputs":count,
        "outer_image_API_calls":calls,"image_results":count*8,"API_errors":errors,"batch_error":batch_error,"exclusive_batch_error":exclusive_error,
        "fragment_single_batch_exclusive_equal":equivalent,"internal_session_run_calls":null})).unwrap()).unwrap();
    assert_eq!(calls, count * 6 + 2);
    assert_eq!(errors, 0);
    assert!(equivalent, "same Fragment policy across all three APIs");
    println!(
        "{label}: all{count} images x8 routes recorded; {calls} outer APIs; batch/exclusive equality verified; internal forwards unmeasured"
    );
}
