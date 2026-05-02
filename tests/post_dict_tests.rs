use ndlocr_lite_rs::postprocess::dict::PostprocessDict;
use tempfile::tempdir;

#[test]
fn post_dict_loads_and_applies_replacements() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("dict.yaml");
    std::fs::write(
        &path,
        r#"
replacements:
  - from: "調示"
    to: "開示"
  - from: "単又は乙"
    to: "甲又は乙"
"#,
    )
    .unwrap();

    let dict = PostprocessDict::load_yaml(&path).unwrap();
    let out = dict.apply("単又は乙が調示を受けた");
    assert_eq!(out, "甲又は乙が開示を受けた");
}

#[test]
fn post_dict_ignores_empty_from_rule() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("dict.yaml");
    std::fs::write(
        &path,
        r#"
replacements:
  - from: ""
    to: "X"
  - from: "A"
    to: "B"
"#,
    )
    .unwrap();

    let dict = PostprocessDict::load_yaml(&path).unwrap();
    let out = dict.apply("A");
    assert_eq!(out, "B");
}
