use super::{LineChunk, split_line_for_input};
use crate::pipeline::crop::{BBox, crop_rgb_u8};

/// 通常分割で拒否された行だけを、行高×8を超える完全白の内側余白で分離する。
/// 各群には半行高の白余白を残し、非白画素と語間幅を元座標に保存する。
pub(super) fn split_separated_groups(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    input_width: usize,
    input_height: usize,
    max_width: usize,
) -> Option<Result<Vec<LineChunk>, String>> {
    let mut gaps = Vec::new();
    let mut blank_start = None;
    let mut seen_nonwhite = false;
    for x in 0..width {
        let white =
            (0..height).all(|y| rgb[(y * width + x) * 3..(y * width + x + 1) * 3] == [255; 3]);
        if white {
            if seen_nonwhite {
                blank_start.get_or_insert(x);
            }
        } else {
            if let Some(start) = blank_start.take().filter(|start| x - start > max_width) {
                gaps.push((start, x));
            }
            seen_nonwhite = true;
        }
    }
    if gaps.is_empty() {
        return None;
    }
    Some(split_groups(
        rgb,
        (width, height),
        threshold,
        (input_width, input_height),
        &gaps,
    ))
}

/// 完全白の中央だけを飛ばし、全群が既存の安全分割に成功したときだけ返す。
fn split_groups(
    rgb: &[u8],
    (width, height): (usize, usize),
    threshold: u8,
    (input_width, input_height): (usize, usize),
    gaps: &[(usize, usize)],
) -> Result<Vec<LineChunk>, String> {
    let padding = height.div_ceil(2);
    let mut result = Vec::new();
    let mut start = 0;
    let mut gap_before = 0;
    for &(left, right) in gaps.iter().chain(std::iter::once(&(width, width))) {
        let end = if left == width { width } else { left + padding };
        let image = crop_rgb_u8(rgb, width, height, BBox::new(start, 0, end, height))
            .map_err(|error| error.to_string())?;
        // 全ての広い白帯を先に列挙しているので、各群には同じ分離条件が残らない。
        let mut chunks = split_line_for_input(
            &image.data,
            image.width,
            image.height,
            threshold,
            input_width,
            input_height,
        )?;
        for chunk in &mut chunks {
            chunk.x0 += start;
            chunk.x1 += start;
        }
        if gap_before != 0 {
            chunks[0].space_before = true;
            chunks[0].gap_before = gap_before;
        }
        result.extend(chunks);
        start = right.saturating_sub(padding);
        gap_before = right - left;
    }
    Ok(result)
}
