use ndlocr_lite_rs::io::{collect_input_images, is_supported_image_extension};
use proptest::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn supports_known_extensions_case_insensitive() {
    assert!(is_supported_image_extension("a.jpg"));
    assert!(is_supported_image_extension("a.JPEG"));
    assert!(is_supported_image_extension("a.PnG"));
}

#[test]
fn reject_unknown_extensions() {
    assert!(!is_supported_image_extension("a.gif"));
}

proptest! {
    #[test]
    fn extension_check_is_case_insensitive(upper in any::<bool>()) {
        let ext = if upper { "JPG" } else { "jpg" };
        let name = format!("x.{}", ext);
        prop_assert!(is_supported_image_extension(&name));
    }
}

#[test]
fn collect_input_images_from_sourceimg() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("a.JPG");
    fs::write(&p, b"x").unwrap();
    let out = collect_input_images(None, Some(p.clone())).unwrap();
    assert_eq!(out, vec![p]);
}
