use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostprocessDict {
    pub replacements: Vec<Replacement>,
}

impl PostprocessDict {
    pub fn load_yaml(path: &Path) -> Result<Self> {
        let body = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read postprocess dict: {}", path.display()))?;
        let mut dict: PostprocessDict =
            serde_yaml::from_str(&body).context("invalid postprocess dict yaml")?;
        // Ignore empty keys to avoid accidental infinite-like replacements.
        dict.replacements.retain(|r| !r.from.is_empty());
        Ok(dict)
    }

    pub fn apply(&self, text: &str) -> String {
        let mut out = text.to_string();
        for r in &self.replacements {
            out = out.replace(&r.from, &r.to);
        }
        out
    }
}
