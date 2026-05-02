//! 評価用フィクスチャ（`tests/fixtures/eval/`）の存在確認。
//! ONNX やモデルなしでも CI で検証できる。

use std::path::PathBuf;

fn eval_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("eval")
}

#[test]
fn eval_scaned0_image_and_truth_exist() {
    let base = eval_fixture_dir();
    let img = base.join("images").join("scaned0.png");
    let truth = base.join("truth").join("scaned0.txt");
    assert!(img.is_file(), "missing eval image: {}", img.display());
    assert!(truth.is_file(), "missing eval truth: {}", truth.display());
    let body = std::fs::read_to_string(&truth).expect("read truth");
    assert!(
        body.contains("第3条") && body.contains("対馬"),
        "truth text should contain expected Japanese phrases"
    );
}

#[test]
fn eval_ndl_logo_ja_image_and_truth_exist() {
    let base = eval_fixture_dir();
    let img = base.join("images").join("ndl_logo_ja.png");
    let truth = base.join("truth").join("ndl_logo_ja.txt");
    assert!(img.is_file(), "missing eval image: {}", img.display());
    assert!(truth.is_file(), "missing eval truth: {}", truth.display());
    let body = std::fs::read_to_string(&truth).expect("read truth");
    assert!(
        body.contains("国立国会図書館"),
        "truth text should match NDL Japanese logo phrase"
    );
}
