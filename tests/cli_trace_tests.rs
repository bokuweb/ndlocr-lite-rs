use clap::Parser;
use ndlocr_lite_rs::cli::{Cli, Command};

#[test]
fn page_trace_is_optional_and_accepts_a_separate_json_path() {
    let plain = Cli::try_parse_from(["ocr", "recognize-page", "--image", "page.png"]).unwrap();
    assert!(matches!(plain.command, Command::RecognizePage(_)));
    let traced = Cli::try_parse_from([
        "ocr",
        "recognize-page",
        "--image",
        "page.png",
        "--trace-json",
        "audit/page.json",
    ]);
    assert!(
        traced.is_ok(),
        "diagnostics must parse without changing TXT options: {traced:?}"
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned DEIM and PARSeq models; mandatory page trace CLI regression"]
fn trace_preserves_real_cli_stdout_and_accounts_for_recognition() {
    use std::process::Command as Process;
    let dir = tempfile::tempdir().unwrap();
    let trace = dir.path().join("nested/trace.json");
    let args = [
        "recognize-page",
        "--image",
        "tests/fixtures/eval/images/ndl_logo_ja.png",
        "--charset",
        "models/NDLmoji.yaml",
        "--structure-rules=false",
    ];
    let plain = Process::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        plain.status.success(),
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    let traced = Process::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
        .args(args)
        .arg("--trace-json")
        .arg(&trace)
        .output()
        .unwrap();
    assert!(
        traced.status.success(),
        "{}",
        String::from_utf8_lossy(&traced.stderr)
    );
    assert_eq!(traced.stdout, plain.stdout);
    assert_eq!(traced.stderr, plain.stderr);
    let audit: serde_json::Value = serde_json::from_slice(&std::fs::read(trace).unwrap()).unwrap();
    assert_eq!(audit["schema_version"], 1);
    assert_eq!(
        format!("{}\n", audit["body"].as_str().unwrap()).as_bytes(),
        plain.stdout
    );
    let prepared = audit["stages"]["prepared"].as_array().unwrap();
    let recognized = audit["stages"]["recognized"].as_array().unwrap();
    assert!(!prepared.is_empty());
    assert_eq!(prepared.len(), recognized.len());
    let regions = audit["stages"]["region_selection"].as_array().unwrap();
    assert_eq!(
        regions
            .iter()
            .filter(|r| r["decision"] == "prepared")
            .count(),
        prepared.len()
    );
    assert!(
        !audit["stages"]["reading_order"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
