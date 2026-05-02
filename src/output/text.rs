use anyhow::Result;
use std::path::Path;

pub fn build_text(lines: &[String]) -> String {
    lines.join("\n")
}

pub fn save_text(lines: &[String], path: &Path) -> Result<()> {
    std::fs::write(path, build_text(lines))?;
    Ok(())
}
