use ndlocr_lite_rs::infer::parseq::{sanitize_recognized_fragment, sanitize_recognized_text};

#[test]
fn closing_brackets_in_a_continuation_fragment_are_preserved() {
    assert_eq!(
        sanitize_recognized_fragment("  continuation).  "),
        "continuation)."
    );
    assert_eq!(
        sanitize_recognized_fragment("国立国会図書館）が作成しました。"),
        "国立国会図書館）が作成しました。"
    );
    assert_eq!(
        sanitize_recognized_text("continuation)."),
        "continuation.",
        "existing single-line policy stays compatible"
    );
}

#[test]
fn fragment_changes_only_the_bracket_deletion_policy() {
    for text in [
        "a   b\n c",
        "1111111111",
        "Aという。という。という。B",
        "(balanced)",
        "普通の本文です。",
    ] {
        assert_eq!(
            sanitize_recognized_fragment(text),
            sanitize_recognized_text(text)
        );
    }
}

proptest::proptest! {
    #[test]
    fn every_supported_closer_survives_without_an_opener(word in "[a-zA-Z]{1,3}",kind in 0usize..5) {
        let (open,close)=[('(',')'),('（','）'),('「','」'),('『','』'),('【','】')][kind];
        let source=format!("  {word}{close}\t");
        proptest::prop_assert_eq!(sanitize_recognized_fragment(&source),format!("{word}{close}"));
        let balanced=format!("{open}{word}{close}");
        proptest::prop_assert_eq!(sanitize_recognized_fragment(&balanced),sanitize_recognized_text(&balanced));
    }
}
