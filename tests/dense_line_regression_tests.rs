#![cfg(feature = "onnx")]
use std::path::Path;
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<_> = b.chars().collect();
    let mut prev: Vec<_> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut row = vec![i + 1];
        for (j, &y) in b.iter().enumerate() {
            row.push(
                (prev[j + 1] + 1)
                    .min(row[j] + 1)
                    .min(prev[j] + usize::from(x != y)),
            );
        }
        prev = row;
    }
    prev[b.len()]
}
#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn dense_runtime_recovers_long_horizontal_japanese_with_each_primary_model() {
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/long_line_split/images/japanese_gothic24.png",
    ))
    .unwrap();
    let truth =
        std::fs::read_to_string("tests/fixtures/long_line_split/truth/japanese_gothic24.txt")
            .unwrap();
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
        assert_eq!(page.lines.len(), 1);
        assert_eq!(page.lines[0].bbox_xyxy, [255, 9, 2855, 33]);
        assert!(page.lines[0].text.chars().count() >= 100);
        assert!(
            distance(&truth, &page.lines[0].text) <= 20,
            "{model}: {:?}",
            page.lines
        );
    }
}
#[test]
#[ignore = "requires CLI and pinned models"]
fn dense_cli_recovery_respects_explicit_split_disable() {
    let dir = tempfile::tempdir().unwrap();
    let truth =
        std::fs::read_to_string("tests/fixtures/long_line_split/truth/japanese_gothic24.txt")
            .unwrap();
    for enabled in [false, true] {
        let output = dir.path().join(format!("{enabled}.txt"));
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
            .args([
                "recognize-page",
                "--image",
                "tests/fixtures/long_line_split/images/japanese_gothic24.png",
                "--charset",
                "models/NDLmoji.yaml",
                "--use-deim-detection",
                "false",
                "--split-long-lines",
                if enabled { "true" } else { "false" },
                "--output-txt",
            ])
            .arg(&output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let text = std::fs::read_to_string(output).unwrap();
        if enabled {
            assert!(distance(&truth, text.trim_end()) <= 2, "{text}");
        } else {
            assert_eq!(text.trim_end(), "通牒では計算の条");
        }
    }
}

#[test]
#[ignore = "requires pinned DEIM/PARSeq models"]
fn ambiguous_overlapping_lines_keep_the_original_runtime_results() {
    use ndlocr_lite_rs::{
        infer::page_pool::{PageRecognizer, PageRecognizerOptions},
        io::load_rgb_u8,
    };
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/long_line_split/images/japanese_mincho24.png",
    ))
    .unwrap();
    let runtime = PageRecognizer::load(
        Path::new("models/deim-s-1024x1024.onnx"),
        Path::new("models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
        1,
    )
    .unwrap();
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
    assert_eq!(
        page.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>(),
        ["では、", "間の時に比較を続し、19年度は"]
    );
    assert_eq!(
        page.lines.iter().map(|l| l.bbox_xyxy).collect::<Vec<_>>(),
        [[78, 0, 2835, 33], [127, 7, 2832, 33]]
    );
}
