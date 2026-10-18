use super::{LineChunk, split_line_for_input};

/// 通常分割に失敗した行の細い下線だけを、分割判定から除外する。
/// 返す座標は元RGBを覆い、認識用の下線画素も変更しない。
pub(super) fn split_underlined_line(
    rgb: &[u8],
    (width, height): (usize, usize),
    threshold: u8,
    input: (usize, usize),
) -> Option<Vec<LineChunk>> {
    if height < 12 {
        return None;
    }
    let minimum = height.checked_mul(8)?;
    let ink = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < u16::from(threshold) * 3
    };
    let mut rules = Vec::new();
    for y in height - height / 4..height {
        let mut start = 0;
        for x in 0..=width {
            if x < width && ink(x, y) {
                continue;
            }
            if x - start >= minimum {
                rules.push((y, start, x));
            }
            start = x + 1;
        }
    }
    let &(top, _, _) = rules.first()?;
    let &(bottom, _, _) = rules.last()?;
    if rules.len() != bottom - top + 1 || rules.len() > (height / 12).max(1) {
        return None;
    }
    let mut masked = rgb.to_vec();
    for &(y, start, end) in &rules {
        masked[(y * width + start) * 3..(y * width + end) * 3].fill(255);
    }
    // 分離した複数の本文字形を要求し、長い単一画線から境界を作らない。
    let (mut groups, mut start, mut ink_top, mut ink_bottom) = (0, None, height, 0);
    let (mut body_top, mut body_bottom) = (height, 0);
    for x in 0..=width {
        let mut column = false;
        if x < width {
            for y in 0..top {
                let i = (y * width + x) * 3;
                if masked[i..i + 3].iter().map(|&v| u16::from(v)).sum::<u16>()
                    < u16::from(threshold) * 3
                {
                    column = true;
                    ink_top = ink_top.min(y);
                    ink_bottom = ink_bottom.max(y + 1);
                    body_top = body_top.min(y);
                    body_bottom = body_bottom.max(y + 1);
                }
            }
        }
        if column {
            start.get_or_insert(x);
        } else if let Some(left) = start.take() {
            if x - left <= height * 2 && ink_bottom - ink_top >= height.div_ceil(3) {
                groups += 1;
            }
            ink_top = height;
            ink_bottom = 0;
        }
    }
    if groups < 6 || body_bottom.saturating_sub(body_top) < height.div_ceil(2) {
        return None;
    }
    // 全ての対象画線を除外済みなので、同じfallbackには再入しない。
    split_line_for_input(&masked, width, height, threshold, input.0, input.1).ok()
}
