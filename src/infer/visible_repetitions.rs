//! モデルが読んだ反復や閉じ括弧を、同じ入力の独立した画線群で支持する。
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
        && !fragment.chars().any(char::is_whitespace)
        && (orientation == ParseqOrientation::Horizontal || height as u128 * 5 <= width as u128 * 4)
        && width > 0
        && height > 0
        && width.checked_mul(height).and_then(|n| n.checked_mul(3)) == Some(rgb.len())
        && (supports_glyph_count(rgb, width, height, fragment.chars().count())
            || supports_closer_body(rgb, width, height, &fragment))
    {
        fragment
    } else {
        current
    }
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
    if width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return false;
    }
    struct Component {
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        area: usize,
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
    let Some(body_height) = components.iter().map(|c| c.y1 - c.y0).max() else {
        return false;
    };
    let min_area = body_height.div_ceil(2).max(3);
    components.retain(|c| c.area >= min_area);
    if components.len() != count {
        return false;
    }
    components.sort_by_key(|c| c.x0);
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
