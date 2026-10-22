//! 代替検出の帯に、単純な塗り図形以外の文字形状があるか確認する。
pub(crate) fn has_text_like_ink(
    rgb: &[u8],
    width: usize,
    height: usize,
    bounds: [i32; 4],
    threshold: u8,
) -> bool {
    let [x0, y0, x1, y1] = bounds;
    if x0 < 0
        || y0 < 0
        || x1 <= x0
        || y1 <= y0
        || x1 as usize > width
        || y1 as usize > height
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return false;
    }
    let (x0, y0, x1, y1) = (x0 as usize, y0 as usize, x1 as usize, y1 as usize);
    let (w, h) = (x1 - x0, y1 - y0);
    let mut mask = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let p = ((y + y0) * width + x + x0) * 3;
            mask[y * w + x] =
                rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < u16::from(threshold) * 3;
        }
    }
    let mut thin_vertical = 0;
    for seed in 0..mask.len() {
        if !mask[seed] {
            continue;
        }
        mask[seed] = false;
        let mut pending = vec![(seed % w, seed / w)];
        let (mut min_x, mut min_y, mut max_x, mut max_y, mut area) = (w, h, 0, 0, 0usize);
        while let Some((x, y)) = pending.pop() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + 1);
            max_y = max_y.max(y + 1);
            area += 1;
            for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                    let p = ny * w + nx;
                    if mask[p] {
                        mask[p] = false;
                        pending.push((nx, ny));
                    }
                }
            }
        }
        if area < h.div_ceil(2).max(3) {
            continue;
        }
        let (cw, ch) = (max_x - min_x, max_y - min_y);
        if (area as u128) * 20 < cw as u128 * ch as u128 * 19 {
            return true;
        }
        if ch as u128 >= 3 * cw as u128 {
            thin_vertical += 1;
        }
    }
    thin_vertical >= 2 && w >= h
}

#[cfg(test)]
#[path = "../../tests/support/band_text_support_policy.rs"]
mod tests;
