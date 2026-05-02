use ndlocr_lite_rs::infer::parseq;

#[test]
fn load_charset_from_yaml_extracts_charset_train() {
    let y = "model:\n  charset_train: \"abcあい\"\n";
    let chars = parseq::load_charset_from_yaml_str(y).unwrap();
    assert_eq!(chars, vec!['a', 'b', 'c', 'あ', 'い']);
}

#[test]
fn load_charset_from_yaml_fails_without_charset_train() {
    let y = "model:\n  charset_test: \"abc\"\n";
    assert!(parseq::load_charset_from_yaml_str(y).is_err());
}
