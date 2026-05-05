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
    let out = join_latin_hyphenated_lines(&out);
    collapse_duplicate_adjacent_lines(&out)
}

/// 行末で単語が `-` でハイフネーションされた英文を、次行と結合して 1 単語に戻す。
///
/// 例:
///   ["The implemen-", "tation is fast"] → ["The implementation is fast"]
///
/// 結合する条件 (3 つすべて満たす):
///   1. 現在行が `[A-Za-z]-` で終わる (末尾の英字 + ハイフン)
///   2. 次行が英字で始まる
///   3. 次行が見出し境界 (`第` / `(` / `（` / `1.`) で始まらない
///
/// 「`-` で終わる ≠ ハイフネーション」のケース (`X-Y` 列挙、`A-1` 識別子など)
/// は条件 1 で前 1 文字も英字を要求することで弾く。さらに次行が見出しっぽければ
/// (`is_section_boundary_like`) 結合しない。条件を厳しく取って、誤結合のリスクを
/// 行末の英字+ハイフンに限定している。
fn join_latin_hyphenated_lines(lines: &[String]) -> Vec<String> {
    // 結合は cascading に動かす: `light-` → `hearted` → `lighthearted` の後に
    // さらに次行が来れば、新しい行末 (この場合は `.`) で再評価される。直前
    // (`out.last_mut()`) と現在を peek-and-merge で組む書き方にすると、cascade
    // が自然に動き、index 管理のミスも起きない。
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for line in lines {
        let should_merge = out
            .last()
            .map(|last| is_latin_hyphenation_pair(last, line))
            .unwrap_or(false);
        if should_merge {
            // `last` を取り出して結合した上で push し直す。
            let last = out.pop().expect("checked Some above");
            let head = last.trim_end();
            // `-` は ASCII 1 byte なので byte slice で安全に末尾 1 文字落とせる。
            let head_no_hyphen = &head[..head.len() - 1];
            let tail = line.trim_start();
            out.push(format!("{head_no_hyphen}{tail}"));
            continue;
        }
        out.push(line.clone());
    }
    out
}

fn is_latin_hyphenation_pair(current: &str, next: &str) -> bool {
    let c = current.trim_end();
    let n = next.trim_start();
    if c.is_empty() || n.is_empty() {
        return false;
    }
    if is_section_boundary_like(n) {
        return false;
    }
    if !c.ends_with('-') {
        return false;
    }
    // `-` の直前の文字が ASCII 英字であること (= 単語末ハイフネーションを示唆)。
    // `X-Y` 列挙のような identifier は `-` の前が英字 1 文字でも成立してしまうが、
    // OCR テキストの 1 行末で identifier がハイフン分割されるのは非常に稀なので
    // 許容する (実害は小さい)。
    let prev_char = c.chars().rev().nth(1);
    if !prev_char.map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        return false;
    }
    // 次行先頭が ASCII 英字であること。
    let next_char = n.chars().next().unwrap();
    next_char.is_ascii_alphabetic()
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
    fn joins_latin_hyphenation_at_line_end() {
        // 英文書 PDF の OCR で行末 `-` が単語末ハイフネーションになっている
        // 典型例。次行と結合してハイフンを落とす。
        let lines = vec!["The implemen-".to_string(), "tation is fast.".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out, vec!["The implementation is fast.".to_string()]);
    }

    #[test]
    fn joins_multiple_hyphenated_words_in_sequence() {
        let lines = vec![
            "The quick brown fox jumps over the lazy".to_string(),
            "dog repeat-".to_string(),
            "edly across the field with light-".to_string(),
            "hearted abandon.".to_string(),
        ];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
        assert!(out[0].ends_with("the lazy"));
        assert_eq!(
            out[1],
            "dog repeatedly across the field with lighthearted abandon."
        );
    }

    #[test]
    fn keeps_trailing_hyphen_when_next_line_is_section_boundary() {
        // 次行が `第N条` などの見出しなら、末尾 `-` は単語ハイフンではなく
        // OCR ノイズの可能性が高いので結合しない。
        let lines = vec![
            "concluding statement-".to_string(),
            "第3条 規定".to_string(),
        ];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], "concluding statement-");
    }

    #[test]
    fn keeps_trailing_hyphen_when_prev_char_is_not_alpha() {
        // `1-` / `]-` / `)-` のようなケースは英単語ハイフネーションでないので
        // 結合しない (1 - 2 列挙、識別子の境界、等の保護)。
        let lines = vec!["item 1-".to_string(), "two next".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], "item 1-");
    }

    #[test]
    fn keeps_trailing_hyphen_when_next_starts_non_alpha() {
        // 次行が数字や日本語で始まる場合は単語結合の見込みが薄いのでそのまま。
        let lines = vec!["page-".to_string(), "1 of 3".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], "page-");

        let lines = vec!["section-".to_string(), "目次".to_string()];
        let out = apply_structural_rules(&lines);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], "section-");
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
