use super::*;
use std::io::Read;
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn complete_repeated_line_replaces_contained_fragments() {
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
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
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
                PageRecognizerOptions {
                    det_conf_threshold: 0.25,
                    min_line_confidence: 0.3,
                    drop_empty_text: true,
                },
            )
            .unwrap();
        assert_eq!(page.lines.len(), 1, "{model}: {:?}", page.lines);
        assert_eq!(page.lines[0].bbox_xyxy, [10, 8, 151, 28]);
        // Recognition still confuses 1 with l; this test specifies duplicate removal only.
        assert_eq!(page.lines[0].text, "AlAlAlAl");
    }
}

#[test]
#[ignore = "diagnostic; requires explicit replay root and pinned models"]
fn frozen_context_replay_matches_fresh_model_pages() {
    let Ok(root) = std::env::var("NDLOCR_CONTEXT_REPLAY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let mut checked = Vec::new();
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
            let expected: Vec<serde_json::Value> = serde_json::from_slice(
                &std::fs::read(root.join("after").join(&mode).join("results.json")).unwrap(),
            )
            .unwrap();
            let ledger: Vec<serde_json::Value> = serde_json::from_slice(
                &std::fs::read(root.join("context-replay-ledger.json")).unwrap(),
            )
            .unwrap();
            // Check every page changed by the post-selector, plus stable controls from each cohort.
            for (n, input) in manifest["inputs"].as_array().unwrap().iter().enumerate() {
                let id = input["id"].as_str().unwrap();
                let entry = ledger
                    .iter()
                    .find(|x| x["mode"] == mode && x["id"] == id)
                    .unwrap();
                if entry["before_lines"].as_u64().unwrap() as usize
                    == entry["kept_indices"].as_array().unwrap().len()
                    && n % 29 != 0
                {
                    continue;
                }
                let image =
                    crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
                let page = runtime
                    .recognize_page_rgb_u8(&image.data, image.width, image.height, opts)
                    .unwrap();
                let actual = serde_json::json!({"id":id,"text":page.lines.iter().map(|x|x.text.as_str()).collect::<Vec<_>>().join("\n"),"lines":page.lines.iter().map(|x|serde_json::json!({"bbox":x.bbox_xyxy,"text":x.text,"confidence":x.confidence})).collect::<Vec<_>>()});
                let mut wanted = expected.iter().find(|x| x["id"] == id).unwrap().clone();
                // JSON stores f64 decimals; compare the API's actual f32 confidence bits.
                for line in wanted["lines"].as_array_mut().unwrap() {
                    let score = line["confidence"].as_f64().unwrap() as f32;
                    line["confidence"] = serde_json::json!(score);
                }
                assert_eq!(actual, wanted, "{mode}/{id}");
                checked.push(serde_json::json!({"mode":mode,"id":id,"matches":true}));
            }
        }
    }
    assert!(checked.len() >= 40);
    std::fs::write(
        root.join("context-fresh-parity.json"),
        serde_json::to_vec_pretty(&checked).unwrap(),
    )
    .unwrap();
}
