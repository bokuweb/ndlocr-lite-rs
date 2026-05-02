use crate::pipeline::connect::RecognizedLine;
use std::fmt::Write as _;
use std::path::Path;

pub fn build_ocr_xml(
    lines: &[RecognizedLine],
    img_width: usize,
    img_height: usize,
    img_name: &str,
) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n");
    out.push_str("<OCRDATASET xmlns=\"\">\n");
    let _ = writeln!(
        out,
        "  <PAGE IMAGENAME=\"{img_name}\" WIDTH=\"{img_width}\" HEIGHT=\"{img_height}\">"
    );
    out.push_str("    <TEXTBLOCK>\n");

    for l in lines {
        let [x0, y0, x1, y1] = l.bbox_xyxy;
        let w = (x1 - x0).max(1);
        let h = (y1 - y0).max(1);
        let escaped = escape_xml_text(&l.text);
        let _ = writeln!(
            out,
            "      <LINE TYPE=\"line_main\" X=\"{x0}\" Y=\"{y0}\" WIDTH=\"{w}\" HEIGHT=\"{h}\" CONF=\"{:.3}\" STRING=\"{}\" ORIENT=\"{}\"></LINE>",
            l.confidence,
            escaped,
            if l.is_vertical {
                "vertical"
            } else {
                "horizontal"
            }
        );
    }

    out.push_str("    </TEXTBLOCK>\n");
    out.push_str("  </PAGE>\n");
    out.push_str("</OCRDATASET>\n");
    out
}

pub fn save_ocr_xml(xml: &str, path: &Path) -> anyhow::Result<()> {
    std::fs::write(path, xml)?;
    Ok(())
}

fn escape_xml_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}
