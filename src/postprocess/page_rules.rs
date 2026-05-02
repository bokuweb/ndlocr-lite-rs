pub fn apply_structural_rules(lines: &[String]) -> Vec<String> {
    // ページ装飾 (`-`, `1,` 等のページ番号・罫線片) は merge / dedup の前に
    // 落としておく。残しておくと隣接行と結合されて見出しを汚すことがある。
    let lines: Vec<String> = lines
        .iter()
        .filter(|l| !is_decoration_line(l))
        .cloned()
        .collect();
    let mut out: Vec<String> = lines.iter().map(|line| clean_line(line)).collect();
    let snapshot = out.clone();
    for (idx, line) in out.iter_mut().enumerate() {
        *line = clean_line_with_context(line, snapshot.get(idx + 1).map(|s| s.as_str()));
    }
    let out = merge_adjacent_lines(&out);
    collapse_duplicate_adjacent_lines(&out)
}

/// 1〜3 文字以下で「漢字でも仮名でもない (= 記号・数字・空白だけ)」行は
/// ページ番号・罫線片・印字汚れと見なして OCR 出力から落とす。
///
/// - `-` `1,` `。、` のような短い記号片を除去
/// - `目次` `附則` 等の短い日本語見出しは残す
/// - 4 文字以上の数字や英字は落とさない (年度 `2024` 等は本文の可能性)
pub fn is_decoration_line(line: &str) -> bool {
    let t = line.trim();
    let n = t.chars().count();
    if n == 0 {
        return true;
    }
    if n > 3 {
        return false;
    }
    // 全文字が日本語本文文字でないなら装飾扱い
    !t.chars().any(is_textual_char)
}

fn is_textual_char(c: char) -> bool {
    // ひらがな・カタカナ・CJK 漢字・全角英字
    matches!(
        c,
        '\u{3040}'..='\u{309F}'   // hiragana
        | '\u{30A0}'..='\u{30FF}' // katakana
        | '\u{3400}'..='\u{4DBF}' // CJK ext A
        | '\u{4E00}'..='\u{9FFF}' // CJK base
        | '\u{FF21}'..='\u{FF3A}' // fullwidth A-Z
        | '\u{FF41}'..='\u{FF5A}' // fullwidth a-z
    )
}

fn clean_line(input: &str) -> String {
    let mut line = input.trim().to_string();
    if line.is_empty() {
        return line;
    }
    if is_article_heading(&line) {
        line = normalize_article_brackets(&line);
        line = trim_article_tail_noise(&line);
    }
    if looks_like_short_parenthetical_label_line(&line) {
        line = trim_short_parenthetical_line_noise(&line);
    }
    if looks_numbered_item(&line) {
        line = normalize_numbered_item_prefix(&line);
    }
    line
}

/// Standalone lines like `(目的)` / `（目的）` (not 第n条 headings, not (1) items).
fn looks_like_short_parenthetical_label_line(line: &str) -> bool {
    let t = line.trim();
    if !t.starts_with('(') && !t.starts_with('（') {
        return false;
    }
    if looks_numbered_item(t) {
        return false;
    }
    if is_article_heading(t) {
        return false;
    }
    true
}

/// OCR often appends hiragana-only garbage after a short parenthetical label (e.g. `(目的)のようになっている`).
/// Truncate after the closing paren when the suffix looks like that kind of noise (not normal prose with kanji/katakana).
fn trim_short_parenthetical_line_noise(line: &str) -> String {
    let t = line.trim();
    let (after_open, close_ch) = if t.starts_with('(') {
        (1usize, ')')
    } else if t.starts_with('（') {
        ("（".len(), '）')
    } else {
        return line.to_string();
    };
    let Some(rel) = t[after_open..].find(close_ch) else {
        return line.to_string();
    };
    let close_idx = after_open + rel;
    let inner = &t[after_open..close_idx];
    if inner.chars().count() > 8 {
        return line.to_string();
    }
    let close_end = close_idx + close_ch.len_utf8();
    if close_end >= t.len() {
        return line.to_string();
    }
    let after = &t[close_end..];
    if should_trim_parenthetical_hiragana_tail(after) {
        return t[..close_end].to_string();
    }
    line.to_string()
}

fn has_cjk_ideograph(s: &str) -> bool {
    s.chars()
        .any(|c| matches!(c, '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}'))
}

fn has_katakana(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '\u{30a0}'..='\u{30ff}'))
}

fn should_trim_parenthetical_hiragana_tail(after: &str) -> bool {
    if after.is_empty() || after.chars().count() < 8 {
        return false;
    }
    if has_cjk_ideograph(after) || has_katakana(after) {
        return false;
    }
    let starts_like_noise = after.starts_with("のよう") || after.starts_with("によう");
    if !starts_like_noise {
        return false;
    }
    after.chars().all(|c| {
        matches!(
            c,
            '\u{3040}'..='\u{309f}' | '\u{30fc}' | '\u{3000}' | '\u{0020}'
        )
    })
}

fn clean_line_with_context(line: &str, next_line: Option<&str>) -> String {
    let mut out = line.to_string();
    if next_line.map(is_section_boundary_like).unwrap_or(false) {
        out = trim_boundary_tail_noise(&out);
    }
    out.trim().to_string()
}

fn is_article_heading(line: &str) -> bool {
    line.starts_with('第') && line.contains('条')
}

fn looks_numbered_item(line: &str) -> bool {
    line.starts_with("(1)")
        || line.starts_with("(2)")
        || line.starts_with("(3)")
        || line.starts_with("(4)")
        || line.starts_with("(5)")
        || line.starts_with("（1)")
        || line.starts_with("（2)")
        || line.starts_with("（3)")
        || line.starts_with("（4)")
        || line.starts_with("（5)")
        || line.starts_with("（1）")
        || line.starts_with("（2）")
        || line.starts_with("（3）")
        || line.starts_with("（4）")
        || line.starts_with("（5）")
}

fn normalize_numbered_item_prefix(line: &str) -> String {
    if line.starts_with("（")
        && line
            .chars()
            .nth(2)
            .map(|c| c == ')' || c == '）')
            .unwrap_or(false)
    {
        let n = line.chars().nth(1).unwrap_or('1');
        let rest = line.chars().skip(3).collect::<String>();
        return format!("({n}){rest}");
    }
    line.to_string()
}

fn normalize_article_brackets(line: &str) -> String {
    let mut out = line.to_string();
    if out.contains("条(") {
        out = out.replacen("条(", "条（", 1);
        if !out.contains('）') && out.contains(')') {
            out = out.replacen(')', "）", 1);
        }
    }
    out
}

fn trim_article_tail_noise(line: &str) -> String {
    let mut out = line.to_string();
    if let Some(p) = out.find(") 第") {
        out.truncate(p + 1);
        return out;
    }
    if let Some(p) = out.find("） 第") {
        out.truncate(p + "）".len());
        return out;
    }
    if let Some(p) = out.find('）')
        && p + "）".len() < out.len()
    {
        out.truncate(p + "）".len());
        return out;
    }
    if let Some(p) = out.find(')')
        && p + ")".len() < out.len()
    {
        out.truncate(p + ")".len());
        return out;
    }
    out
}

fn is_section_boundary_like(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('第') || t.starts_with('(') || t.starts_with('（') || t.starts_with("1.")
}

fn trim_boundary_tail_noise(line: &str) -> String {
    let mut out = line.to_string();
    if out.ends_with("(1") {
        out.truncate(out.len().saturating_sub(2));
    }
    if out.ends_with('第') {
        out.pop();
    }
    out
}

fn merge_adjacent_lines(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        if i + 1 < lines.len() && should_merge_pair(lines[i].as_str(), lines[i + 1].as_str()) {
            out.push(merge_pair(lines[i].as_str(), lines[i + 1].as_str()));
            i += 2;
            continue;
        }
        out.push(lines[i].clone());
        i += 1;
    }
    out
}

fn should_merge_pair(current: &str, next: &str) -> bool {
    let c = current.trim_end();
    let n = next.trim_start();
    if c.is_empty() || n.is_empty() || is_section_boundary_like(n) {
        return false;
    }
    is_cjk_digit_split_pair(c, n)
}

fn merge_pair(current: &str, next: &str) -> String {
    let c = current.trim_end();
    let n = next.trim_start();
    if is_cjk_digit_split_pair(c, n) {
        let mut head = c.to_string();
        head.pop();
        return format!("{head}{n}");
    }
    format!("{c}{n}")
}

fn is_cjk_digit_split_pair(current: &str, next: &str) -> bool {
    let mut rev = current.chars().rev();
    let Some(last) = rev.next() else {
        return false;
    };
    let Some(prev) = rev.next() else {
        return false;
    };
    let Some(next_first) = next.chars().next() else {
        return false;
    };
    last.is_ascii_digit() && is_cjk(prev) && is_cjk(next_first)
}

fn is_cjk(ch: char) -> bool {
    matches!(ch, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '々')
}

fn collapse_duplicate_adjacent_lines(lines: &[String]) -> Vec<String> {
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
    use super::apply_structural_rules;

    #[test]
    fn article_heading_normalizes_parenthesis() {
        let lines = vec!["第2条(秘密情報等の取扱い)".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "第2条（秘密情報等の取扱い）");
    }

    #[test]
    fn article_heading_trims_trailing_noise() {
        let lines = vec!["第2条（秘密情報等の取扱い） 第1".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "第2条（秘密情報等の取扱い）");
    }

    #[test]
    fn article_heading_drops_garbled_tail_after_bracket() {
        let lines = vec!["第1条（秘密情報）にようについて".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "第1条（秘密情報）");
    }

    #[test]
    fn numbered_item_prefix_is_normalized() {
        let lines = vec!["（3）開示を受けたとき".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "(3)開示を受けたとき");
    }

    #[test]
    fn merges_cjk_digit_split_pair() {
        let lines = vec!["対馬全1".to_string(), "域の海藻".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out, vec!["対馬全域の海藻"]);
    }

    #[test]
    fn parenthetical_line_trims_hiragana_only_noise_after_label() {
        let lines = vec!["(目的)のようになっている".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "(目的)");
    }

    #[test]
    fn parenthetical_line_keeps_prose_after_label_when_kanji_follows() {
        let lines = vec!["(目的)のような状態において".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "(目的)のような状態において");
    }

    #[test]
    fn collapses_duplicate_adjacent_lines() {
        let lines = vec![
            "(目的)のようになっている".to_string(),
            "(目的)のようになっている".to_string(),
            "第3条...".to_string(),
        ];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn trims_orphan_dai_at_section_boundary() {
        let lines = vec!["第2章目的及び事業第".to_string(), "(目的)".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out[0], "第2章目的及び事業");
    }

    #[test]
    fn drops_page_decoration_lines() {
        use super::is_decoration_line;
        assert!(is_decoration_line("-"));
        assert!(is_decoration_line("1,"));
        assert!(is_decoration_line(""));
        assert!(is_decoration_line("   "));
        assert!(is_decoration_line("。、"));

        assert!(!is_decoration_line("目次"));
        assert!(!is_decoration_line("附則"));
        assert!(!is_decoration_line("(税目)"));
        assert!(!is_decoration_line("第1章総則"));
    }

    #[test]
    fn structural_rules_drop_decoration_at_top() {
        // OCR が冒頭にページ番号片を拾った想定
        let lines = vec![
            "-".to_string(),
            "1,".to_string(),
            "○横浜市市税条例".to_string(),
            "目次".to_string(),
            "目次".to_string(), // マージン+本文の二重抽出
            "第1章総則".to_string(),
        ];
        let out = apply_structural_rules(&lines);
        assert_eq!(
            out,
            vec![
                "○横浜市市税条例".to_string(),
                "目次".to_string(),
                "第1章総則".to_string(),
            ]
        );
    }
}
