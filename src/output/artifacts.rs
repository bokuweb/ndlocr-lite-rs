use anyhow::Result;
use std::path::Path;

use crate::output::json::{build_ocr_json, save_ocr_json};
use crate::output::text::save_text;
use crate::output::xml::{build_ocr_xml, save_ocr_xml};
use crate::pipeline::connect::RecognizedLine;

pub fn save_page_artifacts(
    lines: &[RecognizedLine],
    texts: &[String],
    img_width: usize,
    img_height: usize,
    img_path: &str,
    img_name: &str,
    out_json_path: &Path,
    out_xml_path: &Path,
    out_txt_path: &Path,
) -> Result<()> {
    let json = build_ocr_json(lines, img_width, img_height, img_path, img_name);
    save_ocr_json(&json, out_json_path)?;
    let xml = build_ocr_xml(lines, img_width, img_height, img_name);
    save_ocr_xml(&xml, out_xml_path)?;
    save_text(texts, out_txt_path)?;
    Ok(())
}
