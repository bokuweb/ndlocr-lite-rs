#![cfg(feature = "onnx")]

use std::path::{Path, PathBuf};

fn check_literal_body(deim: bool) {
    let asset = |name: &str, fallback: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback.into())
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = asset(
        "NDLOCR_TEST_PARSEQ_MODEL",
        "models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
    );
    let detector = asset("NDLOCR_TEST_DEIM_MODEL", "models/deim-s-1024x1024.onnx");
    let charset = asset("NDLOCR_TEST_CHARSET", "models/NDLmoji.yaml");
    for file in [&model, &detector, &charset] {
        assert!(
            file.is_file(),
            "mandatory regression asset missing: {}",
            file.display()
        );
    }
    let fixture = root.join("tests/fixtures/vertical_fallback_page");
    let case: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("case.json")).unwrap()).unwrap();
    let expected = case["expected"].as_str().unwrap().as_bytes();
    let image = fixture.join(case["image"].as_str().unwrap());
    let original = std::fs::read(&image).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let output = std::env::var_os("NDLOCR_TEST_CLI_EVIDENCE")
        .map(PathBuf::from)
        .map(|p| p.join(format!("deim-{deim}")))
        .unwrap_or_else(|| temporary.path().join("evidence"));
    std::fs::create_dir(&output).unwrap();
    let absent = output.join("not-provided.onnx");
    assert!(!absent.exists());
    let binary = std::env::var_os("NDLOCR_TEST_CLI_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_ndlocr-lite-rs")));
    let body = output.join("body.txt");
    let argv = vec![
        "recognize-page".to_owned(),
        "--image".into(),
        image.display().to_string(),
        "--model".into(),
        model.display().to_string(),
        "--model30".into(),
        absent.display().to_string(),
        "--model50".into(),
        absent.display().to_string(),
        "--det-model".into(),
        detector.display().to_string(),
        "--charset".into(),
        charset.display().to_string(),
        "--use-deim-detection".into(),
        deim.to_string(),
        "--enable-cascade".into(),
        "false".into(),
        "--structure-rules".into(),
        "false".into(),
        "--line-crop-padding".into(),
        "0".into(),
        "--output-txt".into(),
        body.display().to_string(),
    ];
    let result = std::process::Command::new(&binary)
        .current_dir(root)
        .args(&argv)
        .output()
        .unwrap();
    std::fs::write(output.join("stdout"), &result.stdout).unwrap();
    std::fs::write(output.join("stderr"), &result.stderr).unwrap();
    std::fs::write(output.join("command.json"), serde_json::to_vec_pretty(&serde_json::json!({"binary":binary,"argv":argv,"exit_code":result.status.code(),"deim":deim})).unwrap()).unwrap();
    assert!(
        result.status.success(),
        "{deim}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::read(&body).unwrap(),
        expected,
        "literal CLI body, deim={deim}"
    );
    let mut stdout = expected.to_vec();
    stdout.push(b'\n');
    assert_eq!(result.stdout, stdout, "stdout LF framing, deim={deim}");
    assert_eq!(std::fs::read(&image).unwrap(), original);
}

#[test]
#[ignore = "mandatory pinned models; frozen FVC v2 regression"]
fn supported_vertical_cli_fallback_restores_literal_body_deim_off() {
    check_literal_body(false);
}

#[test]
#[ignore = "mandatory pinned models; frozen FVC v2 regression"]
fn supported_vertical_cli_fallback_restores_literal_body_deim_on() {
    check_literal_body(true);
}
