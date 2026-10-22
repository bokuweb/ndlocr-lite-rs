use super::*;
use std::io::Read;
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn missing_complete_line_context_recovers_default_api_fragments_with_the_supplied_model() {
    let mut archive = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/repeated_glyph/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut bytes = Vec::new();
    archive
        .by_name("images/mixed_a1_n04_mincho_jpeg75.jpg")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
    for (model, expected) in [
        (
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
            "AlAIAlAl",
        ),
        (
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
            "AlAlAlAl",
        ),
    ] {
        let runtime = PageRecognizer::load(
            Path::new("models/deim-s-1024x1024.onnx"),
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
            1,
        )
        .unwrap();
        let page = runtime
            .recognize_page_rgb_u8(
                image.as_raw(),
                image.width() as usize,
                image.height() as usize,
                PageRecognizerOptions::default(),
            )
            .unwrap();
        assert_eq!(page.lines.len(), 1, "{model}: {:?}", page.lines);
        assert_eq!(page.lines[0].text, expected);
        assert_eq!(page.lines[0].bbox_xyxy, [0, 0, 164, 37]);
        let eval = runtime
            .recognize_page_rgb_u8(
                image.as_raw(),
                image.width() as usize,
                image.height() as usize,
                PageRecognizerOptions {
                    det_conf_threshold: 0.25,
                    min_line_confidence: 0.3,
                    drop_empty_text: true,
                },
            )
            .unwrap();
        assert_eq!(eval.lines.len(), 1);
        assert_eq!(eval.lines[0].text, "AlAlAlAl");
        assert_eq!(eval.lines[0].bbox_xyxy, [10, 8, 151, 28]);
        let empty = runtime
            .recognize_page_rgb_u8(
                image.as_raw(),
                image.width() as usize,
                image.height() as usize,
                PageRecognizerOptions {
                    min_line_confidence: 1.0,
                    ..PageRecognizerOptions::default()
                },
            )
            .unwrap();
        assert!(empty.lines.is_empty());
    }
}

#[test]
#[ignore = "diagnostic; requires explicit matrix root and pinned models"]
fn record_missing_context_runtime_matrix() {
    let Ok(root) = std::env::var("NDLOCR_CONTEXT_MATRIX_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    assert!(std::env::var_os("NDLOCR_CONTEXT_CAPTURE").is_some());
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
            let dir = root.join("after").join(&mode);
            std::fs::create_dir_all(&dir).unwrap();
            let mut rows = Vec::new();
            for input in inputs {
                let id = input["id"].as_str().unwrap();
                let image =
                    crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
                super::super::missing_line_context::take_context_trace();
                let page = runtime
                    .recognize_page_rgb_u8(&image.data, image.width, image.height, opts)
                    .unwrap();
                let trace = super::super::missing_line_context::take_context_trace();
                let text = page
                    .lines
                    .iter()
                    .map(|x| x.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(dir.join(format!("{id}.txt")), &text).unwrap();
                rows.push(serde_json::json!({"id":id,"text":text,"lines":page.lines.iter().map(|x|serde_json::json!({"bbox":x.bbox_xyxy,"text":x.text,"confidence":x.confidence})).collect::<Vec<_>>(),"context_attempts":trace}));
            }
            std::fs::write(
                dir.join("results.json"),
                serde_json::to_vec_pretty(&rows).unwrap(),
            )
            .unwrap();
        }
    }
}
