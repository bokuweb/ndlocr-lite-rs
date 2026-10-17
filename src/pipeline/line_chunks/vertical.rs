use super::{BlankGap, LineChunk, blank_gaps_with_geometry, split_line_at_preferred_gaps};

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

/// 細く背の高い画線が一字幅に収まる白帯を、回転済み縦列の字形内部と推定する。
/// 広い字間を持つ一、短い点や太い字形を結合しない。曖昧な詰め組みの完全識別は保証しない。
pub(super) fn is_internal_stroke_gap(
    widths: [usize; 2],
    extents: [usize; 2],
    gap: usize,
    height: usize,
) -> bool {
    widths.into_iter().all(|width| width <= height / 10)
        && extents
            .into_iter()
            .all(|extent| extent >= height.saturating_mul(2).div_ceil(5))
        && widths[0].saturating_add(gap).saturating_add(widths[1]) <= height.saturating_mul(2) / 3
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
    let anchors = widths
        .iter()
        .zip(extents)
        .filter(|&(&width, &extent)| width >= minimum_extent && extent >= minimum_extent)
        .take(6)
        .count();
    if anchors < 6 {
        return gaps;
    }
    let mut group_width = widths[0];
    let mut i = 0;
    gaps.retain(|gap| {
        let joined = group_width
            .saturating_add(gap.width)
            .saturating_add(widths[i + 1]);
        let internal = joined <= height.saturating_mul(2) / 3
            && is_internal_stroke_gap(
                [widths[i], widths[i + 1]],
                [extents[i], extents[i + 1]],
                gap.width,
                height,
            );
        group_width = if internal { joined } else { widths[i + 1] };
        i += 1;
        !internal
    });
    gaps
}
