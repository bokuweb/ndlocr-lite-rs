#![cfg(feature = "onnx")]
#[test]
#[ignore = "requires CLI and pinned models"]
fn dense_bands_keep_isolated_comma_in_sentence_context() {
    use std::path::Path;
    let out = tempfile::tempdir().unwrap();
    for id in ["japanese_gothic36", "japanese_mincho36"] {
        let image = Path::new("tests/fixtures/long_line_split/images").join(format!("{id}.png"));
        let text = out.path().join(format!("{id}.txt"));
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
            .args([
                "recognize-page",
                "--charset",
                "models/NDLmoji.yaml",
                "--use-deim-detection",
                "false",
                "--image",
            ])
            .arg(image)
            .arg("--output-txt")
            .arg(&text)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let truth = std::fs::read_to_string(
            Path::new("tests/fixtures/long_line_split/truth").join(format!("{id}.txt")),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(text).unwrap().trim_end(),
            truth,
            "{id}"
        );
    }
}
