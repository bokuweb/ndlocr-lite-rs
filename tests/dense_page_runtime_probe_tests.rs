#![cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires pinned models, manifest and output directory"]
fn record_dense_page_runtime() {
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    use std::path::Path;
    let root = std::env::var("NDLOCR_DENSE_RUNTIME_OUTPUT").expect("output directory");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::env::var("NDLOCR_DENSE_MANIFEST")
                .unwrap_or_else(|_| "docs/benchmarks/dense-line/evaluation-manifest.json".into()),
        )
        .unwrap(),
    )
    .unwrap();
    for (label, name) in [
        (
            "runtime30",
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "runtime100",
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
    ] {
        let directory = Path::new(&root).join(label);
        std::fs::create_dir_all(&directory).unwrap();
        let runtime = PageRecognizer::load(
            Path::new("models/deim-s-1024x1024.onnx"),
            &Path::new("models").join(name),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        let mut rows = Vec::new();
        for row in manifest["inputs"].as_array().unwrap() {
            let image = load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
            let page = runtime
                .recognize_page_rgb_u8(
                    &image.data,
                    image.width,
                    image.height,
                    PageRecognizerOptions {
                        det_conf_threshold: 0.25,
                        min_line_confidence: 0.3,
                        drop_empty_text: true,
                    },
                )
                .unwrap();
            let text = page
                .lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(
                directory.join(format!("{}.txt", row["id"].as_str().unwrap())),
                &text,
            )
            .unwrap();
            rows.push(serde_json::json!({"id":row["id"],"text":text,"lines":page.lines.iter().map(|l|serde_json::json!({"bbox":l.bbox_xyxy,"text":l.text,"confidence":l.confidence})).collect::<Vec<_>>()}));
        }
        assert_eq!(rows.len(), 79);
        std::fs::write(
            directory.join("results.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
