use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulePack {
    #[serde(default)]
    pub line_replacements: Vec<LineReplacement>,
    #[serde(default)]
    pub merge_rules: Vec<MergeRule>,
    #[serde(default)]
    pub collapse_duplicate_adjacent_lines: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineReplacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeRule {
    pub left_suffix: String,
    pub right_prefix: String,
    #[serde(default)]
    pub drop_right_prefix_chars: usize,
}

impl RulePack {
    pub fn load_yaml(path: &Path) -> Result<Self> {
        let body = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read rule pack: {}", path.display()))?;
        let mut pack: RulePack = serde_yaml::from_str(&body).context("invalid rule pack yaml")?;
        pack.line_replacements.retain(|r| !r.from.is_empty());
        pack.merge_rules
            .retain(|r| !r.left_suffix.is_empty() && !r.right_prefix.is_empty());
        Ok(pack)
    }

    pub fn apply(&self, lines: &[String]) -> Vec<String> {
        let mut out = lines.to_vec();
        out = self.apply_merge_rules(&out);
        out = self.apply_line_replacements(&out);
        if self.collapse_duplicate_adjacent_lines {
            out = collapse_duplicate_adjacent(&out);
        }
        out
    }

    fn apply_line_replacements(&self, lines: &[String]) -> Vec<String> {
        let mut out = lines.to_vec();
        for line in &mut out {
            for r in &self.line_replacements {
                *line = line.replace(&r.from, &r.to);
            }
        }
        out
    }

    fn apply_merge_rules(&self, lines: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < lines.len() {
            if i + 1 < lines.len() {
                let left = lines[i].trim_end();
                let right = lines[i + 1].trim_start();
                if let Some(rule) = self
                    .merge_rules
                    .iter()
                    .find(|r| left.ends_with(&r.left_suffix) && right.starts_with(&r.right_prefix))
                {
                    let keep_right = right
                        .chars()
                        .skip(rule.drop_right_prefix_chars)
                        .collect::<String>();
                    out.push(format!("{left}{keep_right}"));
                    i += 2;
                    continue;
                }
            }
            out.push(lines[i].clone());
            i += 1;
        }
        out
    }
}

fn collapse_duplicate_adjacent(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        if out.last().map(|l: &String| l == line).unwrap_or(false) {
            continue;
        }
        out.push(line.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::RulePack;

    #[test]
    fn apply_rule_pack_merges_and_replaces() {
        let pack = RulePack {
            line_replacements: vec![super::LineReplacement {
                from: "本拠約".to_string(),
                to: "本契約".to_string(),
            }],
            merge_rules: vec![super::MergeRule {
                left_suffix: "本契".to_string(),
                right_prefix: "約".to_string(),
                drop_right_prefix_chars: 0,
            }],
            collapse_duplicate_adjacent_lines: true,
        };
        let out = pack.apply(&[
            "以下「本契".to_string(),
            "約という。".to_string(),
            "A".to_string(),
            "A".to_string(),
            "本拠約".to_string(),
        ]);
        assert_eq!(
            out,
            vec![
                "以下「本契約という。".to_string(),
                "A".to_string(),
                "本契約".to_string()
            ]
        );
    }
}
