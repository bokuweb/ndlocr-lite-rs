//! 本文から分離された下側の小画線を前の文脈へ戻す。
use super::{BlankGap, LineChunk};
pub(super) fn restore_trailing_lower_mark(
    rgb: &[u8],
    size: (usize, usize),
    threshold: u8,
    chunks: &mut Vec<LineChunk>,
    gaps: &[BlankGap],
    input: (usize, usize),
) {
    let (width, height) = size;
    if height < 16
        || input.0 == 0
        || input.1 == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return;
    }
    let ink = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < u16::from(threshold) * 3
    };
    let mut index = 1;
    while index < chunks.len() {
        let i = index;
        index += 1;
        let (left, mark) = (&chunks[i - 1], &chunks[i]);
        if left.x1 != mark.x0
            || mark.x1 > width
            || mark.x0 >= mark.x1
            || mark.space_before
            || mark.gap_before > height / 4
            || mark.x1 - mark.x0 > height
            || left.x1 - left.x0 < height.saturating_mul(2)
            || (mark.x1 - left.x0) as u128 * input.1 as u128 > height as u128 * input.0 as u128
        {
            continue;
        }
        let start = gaps.partition_point(|g| g.midpoint <= left.x0);
        let end = gaps.partition_point(|g| g.midpoint < mark.x1);
        if end - start >= super::MAX_CHUNK_INK_RUNS {
            continue;
        }
        let span = mark.x1 - mark.x0;
        let mut mask = vec![false; span * height];
        let (mut x0, mut x1, mut y0, mut y1, mut area) = (span, 0, height, 0, 0);
        for y in 0..height {
            for x in 0..span {
                if ink(mark.x0 + x, y) {
                    mask[y * span + x] = true;
                    x0 = x0.min(x);
                    x1 = x1.max(x + 1);
                    y0 = y0.min(y);
                    y1 = y1.max(y + 1);
                    area += 1;
                }
            }
        }
        if area < 3 || x1 - x0 > height / 2 || y1 - y0 > height / 2 || y0 < height.div_ceil(2) {
            continue;
        }
        let (mut body_top, mut body_bottom) = (height, 0);
        for y in 0..height {
            if (left.x0..left.x1).any(|x| ink(x, y)) {
                body_top = body_top.min(y);
                body_bottom = body_bottom.max(y + 1);
            }
        }
        if body_bottom.saturating_sub(body_top) < height.div_ceil(2) {
            continue;
        }
        let seed = mask.iter().position(|&v| v).unwrap();
        let mut pending = vec![(seed % span, seed / span)];
        mask[seed] = false;
        let mut connected = 0;
        while let Some((x, y)) = pending.pop() {
            connected += 1;
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(span - 1) {
                    let p = ny * span + nx;
                    if mask[p] {
                        mask[p] = false;
                        pending.push((nx, ny));
                    }
                }
            }
        }
        if connected == area {
            chunks[i - 1].x1 = mark.x1;
            chunks.remove(i);
            index = i;
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/support/trailing_lower_mark_policy.rs"]
mod tests;
