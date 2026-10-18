#![cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned models and CLI"]
fn fallback_band_selection_rejects_filled_graphics_in_both_cli_modes() {
    let root = tempfile::tempdir().unwrap();
    for name in ["solid", "vertical_bars", "horizontal_marks"] {
        for deim in [false, true] {
            let output = root.path().join(format!("{name}-{deim}.txt"));
            let result = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
                .args(["recognize-page", "--image"])
                .arg(format!("tests/fixtures/dense_line/images/{name}.png"))
                .args([
                    "--charset",
                    "models/NDLmoji.yaml",
                    "--use-deim-detection",
                    if deim { "true" } else { "false" },
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
            assert!(
                std::fs::read_to_string(output).unwrap().trim().is_empty(),
                "{name}/{deim}"
            );
        }
    }
}
