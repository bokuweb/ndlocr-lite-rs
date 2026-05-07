use std::ops::Range;
#[cfg(feature = "morph-correct")]
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionToken {
    surface: String,
    byte_range: Range<usize>,
    is_unknown: bool,
}

impl CorrectionToken {
    pub fn known(surface: impl Into<String>, byte_range: Range<usize>) -> Self {
        Self {
            surface: surface.into(),
            byte_range,
            is_unknown: false,
        }
    }

    pub fn unknown(surface: impl Into<String>, byte_range: Range<usize>) -> Self {
        Self {
            surface: surface.into(),
            byte_range,
            is_unknown: true,
        }
    }
}

pub fn correct_unknown_tokens_with<F>(
    input: &str,
    tokens: &[CorrectionToken],
    mut is_known: F,
) -> String
where
    F: FnMut(&str) -> bool,
{
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    for token in tokens {
        if cursor <= token.byte_range.start && token.byte_range.end <= input.len() {
            out.push_str(&input[cursor..token.byte_range.start]);
            if token.is_unknown {
                out.push_str(
                    candidate_surfaces(&token.surface)
                        .into_iter()
                        .find(|candidate| is_known(candidate))
                        .as_deref()
                        .unwrap_or(&token.surface),
                );
            } else {
                out.push_str(&input[token.byte_range.clone()]);
            }
            cursor = token.byte_range.end;
        }
    }
    out.push_str(&input[cursor..]);
    out
}

fn candidate_surfaces(surface: &str) -> Vec<String> {
    let chars = surface.chars().collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for (idx, ch) in chars.iter().enumerate() {
        for &replacement in confusable_replacements(*ch) {
            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            candidates.push(candidate.iter().collect());
        }
    }
    for (from, to) in confusable_span_collapses() {
        if surface.contains(from) {
            candidates.push(surface.replacen(from, to, 1));
        }
    }
    candidates
}

fn confusable_replacements(ch: char) -> &'static [char] {
    match ch {
        '卜' => &['ト'],
        'ト' => &['卜'],
        '力' => &['カ'],
        'カ' => &['力'],
        '口' => &['ロ'],
        'ロ' => &['口'],
        '一' => &['ー'],
        'ー' => &['一'],
        '士' => &['土'],
        '土' => &['士'],
        '曰' => &['日'],
        '日' => &['曰'],
        _ => &[],
    }
}

fn confusable_span_collapses() -> &'static [(&'static str, &'static str)] {
    &[("禾リ", "利"), ("禾刂", "利")]
}

#[cfg(feature = "morph-correct")]
pub struct DelarochaMorphCorrector {
    tokenizer: delarocha::VibratoSystemTokenizer,
}

#[cfg(feature = "morph-correct")]
impl DelarochaMorphCorrector {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let tokenizer = delarocha::VibratoSystemDictionary::from_path(path)
            .map_err(|err| anyhow::anyhow!(err))?
            .into_tokenizer()
            .ignore_space(true)
            .map_err(|err| anyhow::anyhow!(err))?
            .max_grouping_len(24);
        Ok(Self { tokenizer })
    }

    pub fn correct_line(&self, line: &str) -> anyhow::Result<String> {
        let tokens = self
            .tokenizer
            .tokenize(line)
            .map_err(|err| anyhow::anyhow!(err))?
            .into_iter()
            .map(|token| {
                if token.is_unknown() {
                    CorrectionToken::unknown(token.surface().to_string(), token.range_byte())
                } else {
                    CorrectionToken::known(token.surface().to_string(), token.range_byte())
                }
            })
            .collect::<Vec<_>>();

        Ok(correct_unknown_tokens_with(line, &tokens, |candidate| {
            self.is_known_text(candidate).unwrap_or(false)
        }))
    }

    fn is_known_text(&self, text: &str) -> anyhow::Result<bool> {
        let tokens = self
            .tokenizer
            .tokenize(text)
            .map_err(|err| anyhow::anyhow!(err))?;
        Ok(!tokens.is_empty()
            && tokens
                .iter()
                .map(|token| token.surface())
                .collect::<String>()
                == text
            && tokens.iter().all(|token| !token.is_unknown()))
    }
}

#[cfg(test)]
mod tests {
    use super::candidate_surfaces;

    #[test]
    fn candidate_surfaces_include_generic_ocr_confusions() {
        assert!(candidate_surfaces("テス卜").contains(&"テスト".to_string()));
        assert!(candidate_surfaces("カ口").contains(&"力口".to_string()));
        assert!(candidate_surfaces("権禾リ").contains(&"権利".to_string()));
    }
}
