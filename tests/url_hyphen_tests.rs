use ndlocr_lite_rs::postprocess::page_rules::apply_structural_rules;
use proptest::prelude::*;

fn rules(lines: &[&str]) -> Vec<String> {
    apply_structural_rules(&lines.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn url_line_ends_keep_literal_hyphens_and_physical_lines() {
    for head in [
        "https://example.invalid/a-",
        "1 https://example.invalid/a-",
        "URL:https://example.invalid/a-",
        "FTP://example.invalid/a-",
        "www.example.invalid/a-",
        "WWW.example.invalid-",
    ] {
        assert_eq!(rules(&[head, "b/page.pdf"]), [head, "b/page.pdf"]);
    }
}

#[test]
fn visible_path_tokens_do_not_lose_their_separators() {
    for head in [
        "docs/long-",
        "/absolute/long-",
        r"C:\docs\long-",
        "See docs/long-",
    ] {
        assert_eq!(rules(&[head, "filename.txt"]), [head, "filename.txt"]);
    }
}

#[test]
fn url_earlier_in_a_sentence_does_not_disable_ordinary_english_joining() {
    assert_eq!(
        rules(&[
            "Visit https://example.invalid for implemen-",
            "tation details."
        ]),
        ["Visit https://example.invalid for implementation details."]
    );
    assert_eq!(
        rules(&["The implemen-", "tation is fast."]),
        ["The implementation is fast."]
    );
    assert_eq!(
        rules(&["参考：implemen-", "tation details."]),
        ["参考：implementation details."]
    );
}

proptest! {
    #[test]
    fn literal_url_suffixes_are_preserved(host in "[a-z]{1,12}", path in "[a-z]{1,20}", next in "[a-z]{4,16}") {
        let first=format!("Ref https://{host}.invalid/{path}-");
        let last=format!("{next}/file.pdf");
        prop_assert_eq!(rules(&[&first,&last]),vec![first,last]);
    }
    #[test]
    fn plain_latin_word_hyphenation_keeps_its_existing_behavior(first in "[a-z]{4,16}", last in "[a-z]{4,16}") {
        let head=format!("The {first}-");
        let tail=format!("{last} details.");
        prop_assert_eq!(rules(&[&head,&tail]),vec![format!("The {first}{last} details.")]);
    }
}
