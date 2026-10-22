//! 長行の中央再認識で、近い白列と単語空白を保持する。

use super::{BlankGap, LineChunk, blank_gaps, underlines};

/// 中央近くの字形間または単語空白へ切断位置を移し、全RGBの二分を返す。
pub(crate) fn split_long_line_near_center(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Result<Option<[LineChunk; 2]>, String> {
    if width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return Err("invalid RGB line dimensions".into());
    }
    if !height
        .checked_mul(4)
        .is_some_and(|minimum| width >= minimum)
    {
        return Ok(None);
    }
    if let Some(chunks) = select_split(rgb, width, height, threshold) {
        return Ok(Some(chunks));
    }
    Ok(
        underlines::mask_supported_underline(rgb, (width, height), threshold)
            .and_then(|masked| select_split(&masked, width, height, threshold)),
    )
}

fn select_split(rgb: &[u8], width: usize, height: usize, threshold: u8) -> Option<[LineChunk; 2]> {
    let gaps = blank_gaps(rgb, width, height, threshold);
    let mid = width / 2;
    let ink = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < u16::from(threshold) * 3
    };
    let containing = gaps.iter().find(|gap| {
        let start = gap.midpoint - gap.width / 2;
        (start..=start + gap.width).contains(&mid)
    });
    let crosses = (0..height).any(|y| {
        ink(mid - 1, y) && (y.saturating_sub(1)..=(y + 1).min(height - 1)).any(|yy| ink(mid, yy))
    });
    // 既に画線を横断しない中央は、狭い字間や投影の重なりだけを理由に変更しない。
    if !crosses
        && (containing.is_some()
            || ((0..height).any(|y| ink(mid - 1, y)) && (0..height).any(|y| ink(mid, y))))
    {
        return Some(pair(mid, width, containing));
    }
    let (column, gap) = gaps
        .iter()
        .filter(|gap| !gap.internal_glyph_gap)
        .filter(|gap| gap.midpoint.abs_diff(mid) <= height)
        .min_by_key(|gap| (gap.midpoint.abs_diff(mid), gap.midpoint))
        .map(|gap| (gap.midpoint, gap))?;
    Some(pair(column, width, Some(gap)))
}

fn pair(column: usize, width: usize, gap: Option<&BlankGap>) -> [LineChunk; 2] {
    [
        LineChunk {
            x0: 0,
            x1: column,
            space_before: false,
            gap_before: 0,
        },
        LineChunk {
            x0: column,
            x1: width,
            space_before: gap.is_some_and(|g| g.word_space),
            gap_before: gap.map_or(0, |g| g.width),
        },
    ]
}

#[cfg(test)]
#[path = "../../../tests/support/long_line_split_policy.rs"]
mod tests;

#[cfg(all(test, feature = "onnx"))]
#[path = "../../../tests/support/long_line_model_probe.rs"]
mod model_tests;
