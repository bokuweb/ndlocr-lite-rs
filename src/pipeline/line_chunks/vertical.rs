use super::{BlankGap, LineChunk, blank_gaps_with_geometry, split_line_at_preferred_gaps};

#[cfg(test)]
#[path = "../../../tests/support/supported_vertical_contract.rs"]
mod supported_contract;

#[cfg(any(feature = "onnx", test))]
pub(crate) fn split_supported_vertical_column_for_input(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    input_width: usize,
    input_height: usize,
    span_divisor: usize,
) -> Result<Vec<LineChunk>, String> {
    if ![1, 2].contains(&span_divisor) {
        return Err("invalid vertical span divisor".into());
    }
    let length = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return Err("invalid RGB column dimensions".into());
    }
    if input_width == 0 || input_height == 0 || input_width.checked_mul(input_height).is_none() {
        return Err("invalid model input dimensions".into());
    }
    let model_width = height
        .checked_mul(input_width)
        .and_then(|n| n.checked_mul(2))
        .ok_or("column dimensions overflow")?
        / input_height;
    let max_width = height
        .checked_mul(20)
        .ok_or("column dimensions overflow")?
        .min(model_width)
        / span_divisor;
    if max_width == 0 {
        return Err("zero vertical span budget".into());
    }
    let scan = super::scan_ink_runs(rgb, width, height, threshold);
    let mut gaps = retain_advance_boundaries(scan.gaps, &scan.widths, &scan.extents, height);
    for gap in &mut gaps {
        gap.preferred_split = false;
        gap.word_space = false;
    }
    let mut chunks = split_line_at_preferred_gaps(&gaps, width, height, max_width)?;
    for chunk in &mut chunks {
        chunk.space_before = false;
    }
    Ok(chunks)
}

#[cfg(any(feature = "onnx", test))]
fn retain_advance_boundaries(
    mut gaps: Vec<BlankGap>,
    widths: &[usize],
    extents: &[usize],
    height: usize,
) -> Vec<BlankGap> {
    let minimum_extent = height.saturating_mul(2).div_ceil(5);
    let mut anchor_widths: Vec<_> = widths
        .iter()
        .zip(extents)
        .filter(|&(&width, &extent)| width >= minimum_extent && extent >= minimum_extent)
        .map(|(&width, _)| width)
        .collect();
    if anchor_widths.len() < 6 {
        return gaps;
    }
    // 列の余白が切り落とされても、一字幅の推定を支持字形の中央値から保つ。
    let middle = anchor_widths.len() / 2;
    let (_, median, _) = anchor_widths.select_nth_unstable(middle);
    let base_group_width = (*median).min(height);
    let supported = |i: usize| widths[i] >= minimum_extent && extents[i] >= minimum_extent;
    let mut advances: Vec<_> = gaps
        .iter()
        .enumerate()
        .filter_map(|(i, gap)| {
            if !supported(i) || !supported(i + 1) {
                return None;
            }
            gap.width
                .checked_mul(2)?
                .checked_add(widths[i])?
                .checked_add(widths[i + 1])
        })
        .collect();
    let max_group_width = if advances.len() >= 6 {
        let middle = advances.len() / 2;
        let (_, median, _) = advances.select_nth_unstable(middle);
        median.div_ceil(2).max(base_group_width).min(height)
    } else {
        base_group_width
    };
    let mut group_width = widths[0];
    let mut i = 0;
    gaps.retain(|gap| {
        let joined = group_width
            .saturating_add(gap.width)
            .saturating_add(widths[i + 1]);
        let internal = joined <= base_group_width
            || (joined <= max_group_width
                && !is_tall_narrow_stroke_pair(
                    [widths[i], widths[i + 1]],
                    [extents[i], extents[i + 1]],
                    height,
                ));
        group_width = if internal { joined } else { widths[i + 1] };
        i += 1;
        !internal
    });
    gaps
}

/// 回転済み縦列を白列で分割し、横書きの単語間隔による断片化・空白追加を避ける。
/// 最大20インク列群・幅/高さ比20・モデル入力比の2倍を開発上限とする。
/// 30文字PARSeq向けの経験的制約であり、全モデルの精度保証ではない。
pub fn split_rotated_vertical_column_for_input(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    input_width: usize,
    input_height: usize,
) -> Result<Vec<LineChunk>, String> {
    let length = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return Err("invalid RGB column dimensions".into());
    }
    if input_width == 0 || input_height == 0 || input_width.checked_mul(input_height).is_none() {
        return Err("invalid model input dimensions".into());
    }
    let model_width = height
        .checked_mul(input_width)
        .and_then(|n| n.checked_mul(2))
        .ok_or("column dimensions overflow")?
        / input_height;
    let max_width = height
        .checked_mul(20)
        .ok_or("column dimensions overflow")?
        .min(model_width);
    let mut gaps = blank_gaps_with_geometry(rgb, width, height, threshold, usize::MAX, true);
    // 回転した縦書き字形の間隔は横書きの単語空白ではない。
    for gap in &mut gaps {
        gap.preferred_split = false;
        gap.word_space = false;
    }
    let mut chunks = split_line_at_preferred_gaps(&gaps, width, height, max_width)?;
    for chunk in &mut chunks {
        chunk.space_before = false;
    }
    Ok(chunks)
}

/// 回転済み縦列の隣接成分が、ともに細く背の高い画線か判定する。
/// 厚みが自身の長さの四分の一以下の画線だけを扱う。曖昧な詰め組みの完全識別は保証しない。
pub(super) fn is_tall_narrow_stroke_pair(
    widths: [usize; 2],
    extents: [usize; 2],
    height: usize,
) -> bool {
    widths
        .into_iter()
        .zip(extents)
        .all(|(width, extent)| width <= extent / 4)
        && extents
            .into_iter()
            .all(|extent| extent >= height.saturating_mul(2).div_ceil(5))
}

/// 十分な全幅字形の支持がある列だけで内部白帯を除き、連鎖した群の幅も制限する。
/// 不明瞭な細線だけの列は従来の境界を保ち、インク群数上限の迂回を防ぐ。
pub(super) fn retain_glyph_boundaries(
    mut gaps: Vec<BlankGap>,
    widths: &[usize],
    extents: &[usize],
    height: usize,
) -> Vec<BlankGap> {
    let minimum_extent = height.saturating_mul(2).div_ceil(5);
    let mut anchor_widths: Vec<_> = widths
        .iter()
        .zip(extents)
        .filter(|&(&width, &extent)| width >= minimum_extent && extent >= minimum_extent)
        .map(|(&width, _)| width)
        .collect();
    if anchor_widths.len() < 6 {
        return gaps;
    }
    // 列の余白が切り落とされても、一字幅の推定を支持字形の中央値から保つ。
    let middle = anchor_widths.len() / 2;
    let (_, median, _) = anchor_widths.select_nth_unstable(middle);
    let max_group_width = (*median).min(height);
    let mut group_width = widths[0];
    let mut i = 0;
    gaps.retain(|gap| {
        let joined = group_width
            .saturating_add(gap.width)
            .saturating_add(widths[i + 1]);
        let internal = joined <= max_group_width
            && is_tall_narrow_stroke_pair(
                [widths[i], widths[i + 1]],
                [extents[i], extents[i + 1]],
                height,
            );
        group_width = if internal { joined } else { widths[i + 1] };
        i += 1;
        !internal
    });
    gaps
}

/// Direction evidence for a single fallback column, using the existing ink scan.
#[cfg(test)]
pub(crate) fn has_fallback_column_support(rgb: &[u8], width: usize, height: usize) -> bool {
    fallback_column_anchor_count(rgb, width, height).is_some()
}

#[cfg(any(feature = "onnx", test))]
pub(crate) fn fallback_column_anchor_count(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<usize> {
    let length = width.checked_mul(height).and_then(|x| x.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return None;
    }
    let scan = super::scan_ink_runs(rgb, width, height, 220);
    let minimum_anchor = (height * 3).div_ceil(5);
    let anchors = scan
        .widths
        .iter()
        .zip(&scan.extents)
        .filter(|&(&w, &e)| w >= minimum_anchor && e >= minimum_anchor)
        .count();
    (anchors >= 6
        && (scan.widths.iter().sum::<usize>() as u128) * 3 >= width as u128
        && scan.extents.iter().copied().max().unwrap_or(0) >= height.div_ceil(2))
    .then_some(anchors)
}
