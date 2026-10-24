use ndlocr_lite_rs::output::docx::{save_string_as_docx, save_text_as_docx};
use tempfile::tempdir;

#[test]
fn generated_docx_has_readable_required_parts() {
    use std::io::Read;
    let dir = tempdir().unwrap();
    let out = dir.path().join("parts.docx");
    save_string_as_docx("日本語 & text", &out).unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(out).unwrap()).unwrap();
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().unwrap().into_owned();
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).unwrap();
        if name == "word/document.xml" {
            assert!(
                String::from_utf8(contents)
                    .unwrap()
                    .contains("日本語 &amp; text")
            );
        }
        names.push(name);
    }
    for required in ["[Content_Types].xml", "_rels/.rels", "word/document.xml"] {
        assert!(names.iter().any(|name| name == required));
    }
}

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
