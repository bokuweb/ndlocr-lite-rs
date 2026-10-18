#![cfg(feature = "onnx")]

/// 長行の字形と単語空白をCLI全文で保ち、既存の正読も維持する。
#[test]
#[ignore = "requires pinned DEIM and PARSeq models"]
fn long_line_cli_preserves_literal_text_and_existing_correct_boundaries() {
    for name in [
        "library_mincho36",
        "paragraph_normalized",
        "uri_gothic36",
        "observations_mincho36",
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
            .args([
                "recognize-page",
                "--image",
                &format!("tests/fixtures/long_line_split/images/{name}.png"),
                "--charset",
                "models/NDLmoji.yaml",
                "--use-deim-detection",
                "false",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual = String::from_utf8(output.stdout).unwrap();
        let expected =
            std::fs::read_to_string(format!("tests/fixtures/long_line_split/truth/{name}.txt"))
                .unwrap();
        assert_eq!(actual.trim_end_matches(['\n', '\r']), expected, "{name}");
    }
}
