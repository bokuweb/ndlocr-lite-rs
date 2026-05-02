use anyhow::Result;
use std::io::Write;
use std::path::Path;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub fn save_text_as_docx(lines: &[String], out_path: &Path) -> Result<()> {
    let text = lines.join("\n");
    save_string_as_docx(&text, out_path)
}

pub fn save_string_as_docx(text: &str, out_path: &Path) -> Result<()> {
    if let Some(parent) = out_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(out_path)?;
    let mut zip = ZipWriter::new(file);
    let opt = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opt)?;
    zip.write_all(content_types_xml().as_bytes())?;

    zip.add_directory("_rels/", opt)?;
    zip.start_file("_rels/.rels", opt)?;
    zip.write_all(root_rels_xml().as_bytes())?;

    zip.add_directory("word/", opt)?;
    zip.start_file("word/document.xml", opt)?;
    zip.write_all(document_xml(text).as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn content_types_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#
        .to_string()
}

fn root_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#
        .to_string()
}

fn document_xml(text: &str) -> String {
    let mut body = String::new();
    for line in text.lines() {
        body.push_str("<w:p><w:r><w:t xml:space=\"preserve\">");
        body.push_str(&escape_xml(line));
        body.push_str("</w:t></w:r></w:p>");
    }
    if body.is_empty() {
        body.push_str("<w:p/>");
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr/></w:body>
</w:document>"#
    )
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
