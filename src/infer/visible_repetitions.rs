//! モデルが読んだ反復や閉じ括弧を、同じ入力の独立した画線群で支持する。
mod contained_components;

use crate::infer::parseq::{
    ParseqOrientation, sanitize_fragment_preserving_repetitions,
    sanitize_line_preserving_repetitions, sanitize_recognized_fragment, sanitize_recognized_text,
};

pub(crate) fn sanitize_visible_single_line(
    raw: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
    orientation: ParseqOrientation,
) -> String {
    let current = sanitize_visible_with_bracket_policy(raw, rgb, width, height, orientation, true);
    if !raw
        .chars()
        .any(|c| matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return current;
    }
    let fragment =
        sanitize_visible_with_bracket_policy(raw, rgb, width, height, orientation, false);
    if fragment.chars().count() > current.chars().count()
        && (orientation == ParseqOrientation::Horizontal || height as u128 * 5 <= width as u128 * 4)
        && width > 0
        && height > 0
        && width.checked_mul(height).and_then(|n| n.checked_mul(3)) == Some(rgb.len())
    {
        if let Some(spaced) = supported_trailing_space(&fragment, rgb, width, height)
            .or_else(|| supported_leading_space(&fragment, rgb, width, height))
        {
            return spaced;
        }
        if !fragment.chars().any(char::is_whitespace)
            && (supports_glyph_count(rgb, width, height, fragment.chars().count())
                || supports_closer_body(rgb, width, height, &fragment))
        {
            return fragment;
        }
        if let Some(candidate) = contained_components::candidate(&fragment, rgb, width, height) {
            return candidate;
        }
    }
    current
}

#[cfg(any(feature = "onnx", test))]
pub(crate) fn sanitize_visible_repetitions(
    raw: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
    orientation: ParseqOrientation,
) -> String {
    sanitize_visible_with_bracket_policy(raw, rgb, width, height, orientation, false)
}

fn sanitize_visible_with_bracket_policy(
    raw: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
    orientation: ParseqOrientation,
    remove_unmatched: bool,
) -> String {
    let original = if remove_unmatched {
        sanitize_recognized_text(raw)
    } else {
        sanitize_recognized_fragment(raw)
    };
    let source_candidate = sanitize_fragment_preserving_repetitions(raw);
    let candidate = if remove_unmatched {
        sanitize_line_preserving_repetitions(raw)
    } else {
        source_candidate.clone()
    };
    let count = source_candidate.chars().count();
    if candidate.chars().count() <= original.chars().count()
        || source_candidate.chars().any(char::is_whitespace)
        || (orientation == ParseqOrientation::Auto && height as u128 * 5 > width as u128 * 4)
        || width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return original;
    }
    if supports_glyph_count(rgb, width, height, count) {
        candidate
    } else {
        original
    }
}

fn supports_glyph_count(rgb: &[u8], width: usize, height: usize, count: usize) -> bool {
    supports_glyphs(rgb, width, height, count, None)
}

fn supports_closer_body(rgb: &[u8], width: usize, height: usize, text: &str) -> bool {
    supports_glyphs(rgb, width, height, text.chars().count(), Some(text))
}

// Page fragments keep the original common-height test. Only the single-line
// closer fallback supplies text and separates the body from shorter closers.
fn supports_glyphs(
    rgb: &[u8],
    width: usize,
    height: usize,
    count: usize,
    closer_text: Option<&str>,
) -> bool {
    let Some((components, body_height)) = collect_components(rgb, width, height) else {
        return false;
    };
    supports_collected(&components, body_height, count, closer_text)
}

struct Component {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    area: usize,
}
fn collect_components(rgb: &[u8], width: usize, height: usize) -> Option<(Vec<Component>, usize)> {
    if width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return None;
    }
    let mut mask: Vec<bool> = rgb
        .chunks_exact(3)
        .map(|p| p.iter().map(|&v| u16::from(v)).sum::<u16>() < 660)
        .collect();
    let mut components = Vec::new();
    for seed in 0..mask.len() {
        if !mask[seed] {
            continue;
        }
        mask[seed] = false;
        let mut pending = vec![(seed % width, seed / width)];
        let mut c = Component {
            x0: width,
            y0: height,
            x1: 0,
            y1: 0,
            area: 0,
        };
        while let Some((x, y)) = pending.pop() {
            c.x0 = c.x0.min(x);
            c.y0 = c.y0.min(y);
            c.x1 = c.x1.max(x + 1);
            c.y1 = c.y1.max(y + 1);
            c.area += 1;
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let p = ny * width + nx;
                    if mask[p] {
                        mask[p] = false;
                        pending.push((nx, ny));
                    }
                }
            }
        }
        if c.area >= 3 {
            components.push(c);
        }
    }
    let body_height = components.iter().map(|c| c.y1 - c.y0).max()?;
    let min_area = body_height.div_ceil(2).max(3);
    components.retain(|c| c.area >= min_area);
    components.sort_by_key(|c| c.x0);
    Some((components, body_height))
}

/// 本文高と、既に読んだ文字数に対する画線全幅が狭い白列を支持するときだけ再認識する。
#[cfg(any(feature = "onnx", test))]
pub(super) fn supports_punctuation_space_context(
    rgb: &[u8],
    width: usize,
    height: usize,
    count: usize,
) -> bool {
    if !(6..=12).contains(&count) {
        return false;
    }
    let Some((components, body_height)) = collect_components(rgb, width, height) else {
        return false;
    };
    let mut columns = vec![false; width];
    for c in components {
        columns[c.x0..c.x1].fill(true);
    }
    let Some(left) = columns.iter().position(|&v| v) else {
        return false;
    };
    let right = columns.iter().rposition(|&v| v).unwrap() + 1;
    let gap = columns[left..right]
        .split(|&v| v)
        .map(<[bool]>::len)
        .max()
        .unwrap_or(0);
    gap > 0
        && gap as u128 * 4 <= body_height as u128 * 3
        && gap as u128 * 4 * count as u128 <= (right - left) as u128 * 3
}

/// 新しい画像支持の診断だけに、既存collectorの読み取り結果を渡す。
#[cfg(test)]
pub(super) fn probe_space_components(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<(Vec<[usize; 5]>, usize)> {
    let (components, body_height) = collect_components(rgb, width, height)?;
    Some((
        components
            .iter()
            .map(|c| [c.x0, c.y0, c.x1, c.y1, c.area])
            .collect(),
        body_height,
    ))
}

fn supports_collected(
    components: &[Component],
    body_height: usize,
    count: usize,
    closer_text: Option<&str>,
) -> bool {
    if components.len() != count {
        return false;
    }
    if components.windows(2).any(|p| p[0].x1 > p[1].x0) {
        return false;
    }
    if let Some(text) = closer_text {
        let is_closer = |c: char| matches!(c, ')' | '）' | '」' | '』' | '】');
        if text.chars().any(char::is_whitespace) || !text.chars().any(is_closer) {
            return false;
        }
        let body: Vec<_> = components
            .iter()
            .zip(text.chars())
            .filter(|(_, c)| !is_closer(*c))
            .map(|(c, _)| c)
            .collect();
        if body.len() < 2 {
            return false;
        }
        let body_h = body.iter().map(|c| c.y1 - c.y0).max().unwrap();
        let top = body.iter().map(|c| c.y0).max().unwrap();
        let bottom = body.iter().map(|c| c.y1).min().unwrap();
        if bottom.saturating_sub(top) as u128 * 4 < body_h as u128 * 3
            || components
                .iter()
                .any(|c| (c.x1 - c.x0) as u128 > body_h as u128 * 2)
        {
            return false;
        }
        return components
            .iter()
            .zip(text.chars())
            .filter(|(_, c)| is_closer(*c))
            .all(|(g, _)| {
                let h = g.y1 - g.y0;
                h as u128 * 2 >= body_h as u128
                    && g.y1.min(bottom).saturating_sub(g.y0.max(top)) as u128 * 4
                        >= h.min(body_h) as u128 * 3
            });
    }
    let common_top = components.iter().map(|c| c.y0).max().unwrap();
    let common_bottom = components.iter().map(|c| c.y1).min().unwrap();
    common_bottom.saturating_sub(common_top) as u128 * 4 >= body_height as u128 * 3
        && components
            .iter()
            .all(|c| (c.x1 - c.x0) as u128 <= body_height as u128 * 2)
}

// Only an isolated trailing closer can supply a word-space hypothesis. Two
// compact body gaps provide a local reference; thin rules and distant marks do not.
fn supported_trailing_space(
    fragment: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<String> {
    // Check text before scanning pixels. Unsupported forms keep the existing policy.
    let closer = fragment.chars().last()?;
    if !matches!(closer, ')' | '）' | '」' | '』' | '】') {
        return None;
    }
    let prefix = &fragment[..fragment.len() - closer.len_utf8()];
    let body = prefix.strip_suffix(' ').unwrap_or(prefix);
    if body.chars().count() < 3
        || body
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return None;
    }
    let compact = format!("{body}{closer}");
    let (components, body_height) = collect_components(rgb, width, height)?;
    let count = compact.chars().count();
    if !(supports_collected(&components, body_height, count, None)
        || supports_collected(&components, body_height, count, Some(&compact)))
    {
        return None;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    trailing_space_from_boxes(fragment, &boxes)
}

fn trailing_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
) -> Option<String> {
    let closer = fragment.chars().last()?;
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let prefix = &fragment[..fragment.len() - closer.len_utf8()];
    let body = prefix.strip_suffix(' ').unwrap_or(prefix);
    if body.chars().count() < 3
        || body.chars().any(|c| char::is_whitespace(c) || is_closer(c))
        || boxes.len() != body.chars().count() + 1
        || boxes.iter().any(|&(x0, x1, y0, y1)| x0 >= x1 || y0 >= y1)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let body_boxes = &boxes[..boxes.len() - 1];
    let h = body_boxes.iter().map(|g| g.3 - g.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    if gaps.contains(&0) || max_gap as u128 * 4 > h as u128 {
        return None;
    }
    let close = boxes.last()?;
    let gap = close.0 - body_boxes.last()?.1;
    let threshold = (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3));
    if close.1 - close.0 < h.div_ceil(4)
        || (close.3 - close.2) as u128 > h as u128 * 2
        || gap < threshold
        || gap > h
    {
        return None;
    }
    Some(format!("{body} {closer}"))
}

fn supported_leading_space(
    fragment: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<String> {
    // Check text before scanning pixels. Unsupported forms keep the existing policy.
    let closer = fragment.chars().next()?;
    if !matches!(closer, ')' | '）' | '」' | '』' | '】') {
        return None;
    }
    let suffix = &fragment[closer.len_utf8()..];
    let body = suffix.strip_prefix(' ').unwrap_or(suffix);
    if body.chars().count() < 3
        || body
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return None;
    }
    let compact = format!("{closer}{body}");
    let (components, body_height) = collect_components(rgb, width, height)?;
    let count = compact.chars().count();
    if !(supports_collected(&components, body_height, count, None)
        || supports_collected(&components, body_height, count, Some(&compact)))
    {
        return None;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    compact_leading_space_from_boxes(fragment, &boxes)
}

fn compact_leading_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
) -> Option<String> {
    let closer = fragment.chars().next()?;
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let suffix = &fragment[closer.len_utf8()..];
    let body = suffix.strip_prefix(' ').unwrap_or(suffix);
    if body.chars().count() < 3
        || body.chars().any(|c| c.is_whitespace() || is_closer(c))
        || boxes.len() != body.chars().count() + 1
        || boxes.iter().any(|&(x0, x1, y0, y1)| x0 >= x1 || y0 >= y1)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let body_boxes = &boxes[1..];
    let h = body_boxes.iter().map(|g| g.3 - g.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    // A fullwidth closer can leave a large bearing without a word space.
    // Require its ink and following gap to fit within one body height.
    if gaps.contains(&0) || max_gap as u128 * 4 > h as u128 || boxes[1].0 - boxes[0].0 > h {
        return None;
    }
    let close = boxes.first()?;
    let gap = body_boxes.first()?.0 - close.1;
    let threshold = (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3));
    if close.1 - close.0 < h.div_ceil(4)
        || (close.3 - close.2) as u128 > h as u128 * 2
        || gap < threshold
        || gap > h
    {
        return None;
    }
    Some(format!("{closer} {body}"))
}

#[cfg(test)]
#[path = "../../tests/support/visible_repetitions_policy.rs"]
mod tests;

#[cfg(all(test, feature = "onnx"))]
#[path = "../../tests/support/literal_closer_probe.rs"]
mod closer_probe;

#[cfg(test)]
#[path = "../../tests/support/closer_geometry_probe.rs"]
mod geometry_probe;

#[cfg(all(test, feature = "onnx"))]
#[path = "../../tests/support/body_closer_recovery.rs"]
mod body_closer_tests;

#[cfg(test)]
#[path = "../../tests/support/closer_word_spaces_probe.rs"]
mod word_spaces_probe;

#[cfg(test)]
#[path = "../../tests/support/leading_closer_space.rs"]
mod leading_space_probe;

#[cfg(test)]
#[path = "../../tests/support/contained_closer_components.rs"]
mod contained_closer_probe;
