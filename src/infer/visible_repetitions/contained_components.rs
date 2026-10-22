//! A closer-only view of components strictly enclosed by another glyph bbox.
//! The original count gate and page-fragment policy keep their original view.
use super::*;
use unicode_width::UnicodeWidthChar;

pub(super) fn maximal_component_indices(boxes: &[(usize, usize, usize, usize)]) -> Vec<usize> {
    boxes
        .iter()
        .enumerate()
        .filter(|(_, child)| {
            !boxes.iter().any(|outer| {
                outer.0 < child.0 && child.1 < outer.1 && outer.2 < child.2 && child.3 < outer.3
            })
        })
        .map(|(i, _)| i)
        .collect()
}
fn component_view(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<(Vec<Component>, usize, bool)> {
    let (all, h) = collect_components(rgb, width, height)?;
    let boxes: Vec<_> = all.iter().map(|c| (c.x0, c.x1, c.y0, c.y1)).collect();
    let keep = maximal_component_indices(&boxes);
    let changed = keep.len() != all.len();
    Some((
        all.into_iter()
            .enumerate()
            .filter(|(i, _)| keep.contains(i))
            .map(|(_, c)| c)
            .collect(),
        h,
        changed,
    ))
}

pub(super) fn candidate(fragment: &str, rgb: &[u8], width: usize, height: usize) -> Option<String> {
    let (components, h, changed) = component_view(rgb, width, height)?;
    if !changed {
        return None;
    }
    let compact: String = fragment.chars().filter(|&c| c != ' ').collect();
    let n = compact.chars().count();
    if components.len() != n {
        return None;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    if (supports_collected(&components, h, n, None)
        || supports_collected(&components, h, n, Some(&compact)))
        && let Some(spaced) = cell_aware_space_from_boxes(fragment, &boxes, true)
            .or_else(|| cell_aware_space_from_boxes(fragment, &boxes, false))
    {
        return Some(spaced);
    }
    let wide_count = fragment
        .chars()
        .filter(|&c| c.width() == Some(2) && !matches!(c, ')' | '）' | '」' | '』' | '】'))
        .count();
    if wide_count < 2
        && let Some(spaced) = grouped_edge_space(fragment, &components, h, false)
            .or_else(|| grouped_edge_space(fragment, &components, h, true))
    {
        return Some(spaced);
    }
    // Strict enclosure entails x-overlap in the original view, so its legacy
    // count gate cannot support a newly restored closer here.
    (!fragment.chars().any(char::is_whitespace)
        && supports_collected(&components, h, fragment.chars().count(), Some(fragment)))
    .then(|| fragment.to_owned())
}

fn grouped_edge_space(
    fragment: &str,
    components: &[Component],
    height: usize,
    leading: bool,
) -> Option<String> {
    let closer = if leading {
        fragment.chars().next()?
    } else {
        fragment.chars().last()?
    };
    if !matches!(closer, ')' | '）' | '」' | '』' | '】') {
        return None;
    }
    let body = if leading {
        let tail = &fragment[closer.len_utf8()..];
        tail.strip_prefix(' ').unwrap_or(tail)
    } else {
        let head = &fragment[..fragment.len() - closer.len_utf8()];
        head.strip_suffix(' ').unwrap_or(head)
    };
    if body.chars().count() < 3
        || body
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return None;
    }
    let compact = if leading {
        format!("{closer}{body}")
    } else {
        format!("{body}{closer}")
    };
    let n = compact.chars().count();
    if !(supports_collected(components, height, n, None)
        || supports_collected(components, height, n, Some(&compact)))
    {
        return None;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    if leading {
        compact_leading_space_from_boxes(fragment, &boxes)
    } else {
        trailing_space_from_boxes(fragment, &boxes)
    }
}
pub(super) fn cell_aware_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
    leading: bool,
) -> Option<String> {
    use unicode_width::UnicodeWidthChar;
    let closer = if leading {
        fragment.chars().next()?
    } else {
        fragment.chars().last()?
    };
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let body = if leading {
        let tail = &fragment[closer.len_utf8()..];
        tail.strip_prefix(' ').unwrap_or(tail)
    } else {
        let head = &fragment[..fragment.len() - closer.len_utf8()];
        head.strip_suffix(' ').unwrap_or(head)
    };
    let widths: Vec<_> = body.chars().map(|c| c.width().unwrap_or(0)).collect();
    if body.chars().any(|c| c.is_whitespace() || is_closer(c))
        || widths.iter().any(|w| !matches!(w, 1 | 2))
        || widths.iter().filter(|&&w| w == 2).count() < 2
        || boxes.len() != widths.len() + 1
        || boxes.iter().any(|&(a, b, c, d)| a >= b || c >= d)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let (close, body_boxes) = if leading {
        (&boxes[0], &boxes[1..])
    } else {
        (boxes.last()?, &boxes[..boxes.len() - 1])
    };
    let h = body_boxes.iter().map(|b| b.3 - b.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    let wide_widths: Vec<_> = body_boxes
        .iter()
        .zip(&widths)
        .filter(|(_, w)| **w == 2)
        .map(|(b, _)| b.1 - b.0)
        .collect();
    let max_width = *wide_widths.iter().max()?;
    let min_width = *wide_widths.iter().min()?;
    let unit = max_width + max_gap;
    if gaps.contains(&0)
        || max_gap as u128 * 4 > h as u128
        || min_width < h.div_ceil(2)
        || max_width - min_width > h.div_ceil(4)
        || unit as u128 * 4 < h as u128 * 3
        || unit as u128 * 2 > h as u128 * 3
        || close.1 - close.0 < h.div_ceil(5)
        || (close.3 - close.2) as u128 > h as u128 * 2
    {
        return None;
    }
    for (p, &width) in body_boxes.windows(2).zip(&widths) {
        let pitch = p[1].0 - p[0].0;
        if (width == 2 && pitch.abs_diff(unit) > h.div_ceil(4))
            || (width == 1 && (pitch < unit.div_ceil(4) || pitch > unit + h.div_ceil(4)))
        {
            return None;
        }
    }
    let gap = if leading {
        body_boxes[0].0 - close.1
    } else {
        close.0 - body_boxes.last()?.1
    };
    let advance = body_boxes[0].0.saturating_sub(close.0);
    let narrow = gap >= (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3))
        && gap <= h
        && (!leading || (advance <= h && advance + h.div_ceil(8) <= unit));
    let wide = leading && advance >= unit + h.div_ceil(4) && advance <= unit + h;
    if narrow || wide {
        Some(if leading {
            format!("{closer} {body}")
        } else {
            format!("{body} {closer}")
        })
    } else {
        None
    }
}
