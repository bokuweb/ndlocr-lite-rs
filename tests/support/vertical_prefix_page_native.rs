use crate::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
use std::path::{Path, PathBuf};

#[test]
#[ignore = "requires pinned official DEIM and PARSeq30; exercised by inference CI"]
fn page_prefix_recovery_preserves_complete_text_and_session_reuse() {
    let asset = |env: &str, fallback: &str| {
        std::env::var_os(env)
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback.into())
    };
    let model = asset(
        "NDLOCR_TEST_PARSEQ_MODEL",
        "models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
    );
    let detector = asset("NDLOCR_TEST_DEIM_MODEL", "models/deim-s-1024x1024.onnx");
    let charset = asset("NDLOCR_TEST_CHARSET", "models/NDLmoji.yaml");
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vertical_prefix_page");
    let case: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("case.json")).unwrap()).unwrap();
    let image = crate::io::load_rgb_u8(&directory.join(case["image"].as_str().unwrap())).unwrap();
    let original = image.data.clone();
    let runtime = PageRecognizer::load(&detector, &model, &charset, 1).unwrap();
    for options in [
        PageRecognizerOptions {
            det_conf_threshold: 0.25,
            min_line_confidence: 0.3,
            drop_empty_text: true,
        },
        PageRecognizerOptions::default(),
    ] {
        let page = runtime
            .recognize_page_rgb_u8(&image.data, image.width, image.height, options)
            .unwrap();
        let text = page
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            text,
            case["expected"].as_str().unwrap(),
            "actual Page path must recover the complete prefix extension"
        );
        for line in &page.lines {
            let [x0, y0, x1, y1] = line.bbox_xyxy;
            assert!(
                0 <= x0
                    && x0 < x1
                    && x1 as usize <= image.width
                    && 0 <= y0
                    && y0 < y1
                    && y1 as usize <= image.height
            );
            assert!(line.confidence.is_finite() && (0.0..=1.0).contains(&line.confidence));
        }
        assert_eq!(image.data, original);
        assert!(runtime.recognize_page_rgb_u8(&[], 0, 0, options).is_err());
        let reused = runtime
            .recognize_page_rgb_u8(&image.data, image.width, image.height, options)
            .unwrap();
        assert_eq!(reused.lines.len(), page.lines.len());
        for (a, b) in reused.lines.iter().zip(&page.lines) {
            assert_eq!(a.text, b.text);
            assert_eq!(a.bbox_xyxy, b.bbox_xyxy);
            assert_eq!(a.confidence.to_bits(), b.confidence.to_bits());
        }
        assert_eq!(image.data, original);
    }
}
