use ndlocr_lite_rs::output::docx::{save_string_as_docx, save_text_as_docx};
use tempfile::tempdir;

#[test]
fn save_text_as_docx_writes_valid_docx_package() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.docx");
    let lines = vec!["第一行".to_string(), "第二行".to_string()];
    save_text_as_docx(&lines, &out).unwrap();
    assert!(out.is_file());

    let file = std::fs::File::open(&out).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut doc = String::new();
    {
        let mut entry = zip.by_name("word/document.xml").unwrap();
        use std::io::Read as _;
        entry.read_to_string(&mut doc).unwrap();
    }
    assert!(doc.contains("第一行"));
    assert!(doc.contains("第二行"));
}

#[test]
fn save_string_as_docx_escapes_xml_text() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("escaped.docx");
    save_string_as_docx("A & B < C > D", &out).unwrap();

    let file = std::fs::File::open(&out).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut doc = String::new();
    {
        let mut entry = zip.by_name("word/document.xml").unwrap();
        use std::io::Read as _;
        entry.read_to_string(&mut doc).unwrap();
    }
    assert!(doc.contains("A &amp; B &lt; C &gt; D"));
}
