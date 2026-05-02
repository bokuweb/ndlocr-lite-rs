use ndlocr_lite_rs::output::text::{build_text, save_text};
use tempfile::tempdir;

#[test]
fn build_text_joins_with_newline() {
    let s = build_text(&["a".to_string(), "b".to_string()]);
    assert_eq!(s, "a\nb");
}

#[test]
fn save_text_writes_file() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("out.txt");
    save_text(&["x".to_string(), "y".to_string()], &p).unwrap();
    let body = std::fs::read_to_string(&p).unwrap();
    assert_eq!(body, "x\ny");
}
