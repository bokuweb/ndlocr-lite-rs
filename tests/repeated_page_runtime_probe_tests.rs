#![cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires pinned models and explicit phase"]
fn record_repeated_page_runtime_matrix() {
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    use std::path::Path;
    let root = std::env::var("NDLOCR_REPEAT_ROOT").unwrap_or_else(|_| "tmp/repeated-glyph".into());
    let root = Path::new(&root);
    let phase = std::env::var("NDLOCR_REPEAT_PHASE").expect("explicit before/after phase");
    assert!(["before", "after"].contains(&phase.as_str()));
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 287);
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
            let mode = format!("{label}_{suffix}");
            let dir = root.join(&phase).join(&mode);
            std::fs::create_dir_all(&dir).unwrap();
            let reused: Vec<serde_json::Value> = if phase == "before" {
                serde_json::from_slice(
                    &std::fs::read(root.join("reused").join(&mode).join("results.json")).unwrap(),
                )
                .unwrap()
            } else {
                Vec::new()
            };
            let mut rows = Vec::new();
            let mut calls = 0;
            for row in inputs {
                let id = row["id"].as_str().unwrap();
                let result = if phase == "before" && row["cohort"] != "repeated_glyph" {
                    let old = reused
                        .iter()
                        .find(|x| x["id"] == row["id"])
                        .unwrap()
                        .clone();
                    assert_eq!(
                        std::fs::read_to_string(dir.join(format!("{id}.txt"))).unwrap(),
                        old["text"].as_str().unwrap()
                    );
                    old
                } else {
                    let image = load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
                    let page = runtime
                        .recognize_page_rgb_u8(&image.data, image.width, image.height, opts)
                        .unwrap();
                    calls += 1;
                    let text = page
                        .lines
                        .iter()
                        .map(|x| x.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    serde_json::json!({"id":id,"text":text,"lines":page.lines.iter().map(|x|serde_json::json!({"bbox":x.bbox_xyxy,"text":x.text,"confidence":x.confidence})).collect::<Vec<_>>()})
                };
                std::fs::write(
                    dir.join(format!("{id}.txt")),
                    result["text"].as_str().unwrap(),
                )
                .unwrap();
                rows.push(result);
            }
            assert_eq!(calls, if phase == "before" { 160 } else { 287 });
            assert_eq!(rows.len(), 287);
            std::fs::write(
                dir.join("results.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
        }
    }
}
