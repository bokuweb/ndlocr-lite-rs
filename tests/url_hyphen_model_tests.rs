#![cfg(feature = "onnx")]

/// 検出器によらず、認識済みのURL末尾ハイフンを構造処理で削除しない。
#[test]
#[ignore = "requires pinned DEIM and PARSeq models"]
fn page_cli_keeps_both_wrapped_url_hyphens() {
    for mode in ["true", "false"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
            .args([
                "recognize-page",
                "--image",
                "tests/fixtures/eval/images/jma_kobe_notice.png",
                "--charset",
                "models/NDLmoji.yaml",
                "--use-deim-detection",
                mode,
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let body = String::from_utf8(output.stdout).unwrap();
        let starts: Vec<_> = body
            .lines()
            .filter(|line| line.contains("://") && line.ends_with("/kobe-"))
            .collect();
        assert_eq!(starts.len(), 2, "detection={mode}\n{body}");
        assert_eq!(
            body.lines()
                .filter(|line| line.starts_with("c/press/"))
                .count(),
            2
        );
    }
}
