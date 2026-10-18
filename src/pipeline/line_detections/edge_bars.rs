//! 検出矩形の左右で切られた短い横画線だけを完結させる。

const THRESHOLD_SUM: u16 = 220 * 3;

fn valid(b: [i32; 4], w: usize, h: usize) -> bool {
    b[0] >= 0 && b[1] >= 0 && b[0] < b[2] && b[1] < b[3] && b[2] as usize <= w && b[3] as usize <= h
}

fn overlaps(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// 原検出を保持し、認識用の矩形だけを元RGBに沿って広げる。
pub(super) fn recognition_bounds(
    rgb: &[u8],
    w: usize,
    h: usize,
    bounds: &[[i32; 4]],
) -> Vec<[i32; 4]> {
    let mut output = bounds.to_vec();
    if w.checked_mul(h).and_then(|n| n.checked_mul(3)) != Some(rgb.len()) || w == 0 || h == 0 {
        return output;
    }
    for (i, &b) in bounds.iter().enumerate() {
        if !valid(b, w, h)
            || bounds
                .iter()
                .enumerate()
                .any(|(j, &other)| i != j && valid(other, w, h) && overlaps(b, other))
        {
            continue;
        }
        let b = b.map(|v| v as usize);
        let height = b[3] - b[1];
        if height < 12 || (b[2] - b[0]) / 4 < height {
            continue;
        }
        let left = complete_side(rgb, w, h, b, false).unwrap_or(b[0]);
        let right = complete_side(rgb, w, h, b, true).unwrap_or(b[2]);
        if let (Ok(left), Ok(right)) = (i32::try_from(left), i32::try_from(right)) {
            let proposed = [left, b[1] as i32, right, b[3] as i32];
            if !bounds
                .iter()
                .enumerate()
                .any(|(j, &other)| i != j && valid(other, w, h) && overlaps(proposed, other))
            {
                output[i] = proposed;
            }
        }
    }
    let proposed = output.clone();
    for (i, b) in output.iter_mut().enumerate() {
        if *b != bounds[i]
            && proposed
                .iter()
                .enumerate()
                .any(|(j, &other)| i != j && valid(other, w, h) && overlaps(*b, other))
        {
            *b = bounds[i];
        }
    }
    output
}

/// 境界を横切る単一の小さい成分だけを局所窓内で探索する。
fn complete_side(rgb: &[u8], w: usize, h: usize, b: [usize; 4], right: bool) -> Option<usize> {
    let [x0, y0, x1, y1] = b;
    let height = y1 - y0;
    let edge = if right { x1 - 1 } else { x0 };
    let ink = |x: usize, y: usize| {
        rgb[(y * w + x) * 3..(y * w + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < THRESHOLD_SUM
    };
    let rows: Vec<_> = (y0..y1).filter(|&y| ink(edge, y)).collect();
    let (&top, &last) = rows.first().zip(rows.last())?;
    if last + 1 - top > height / 4 || (top + last + 1).abs_diff(y0 + y1) > height / 2 {
        return None;
    }
    let left = edge.saturating_sub(height);
    let end = (edge + height + 1).min(w);
    let window_width = end - left;
    let mut seen = vec![false; window_width * height];
    let index = |x: usize, y: usize| (y - y0) * window_width + x - left;
    let mut pending = vec![(edge, top)];
    seen[index(edge, top)] = true;
    let mut component = [edge, top, edge + 1, top + 1];
    while let Some((x, y)) = pending.pop() {
        component = [
            component[0].min(x),
            component[1].min(y),
            component[2].max(x + 1),
            component[3].max(y + 1),
        ];
        if component[2] - component[0] > height * 3 / 2 || component[3] - component[1] > height / 4
        {
            return None;
        }
        for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
            for nx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                if !ink(nx, ny) {
                    continue;
                }
                if nx < left || nx >= end || ny < y0 || ny >= y1 {
                    return None;
                }
                let i = index(nx, ny);
                if !seen[i] {
                    seen[i] = true;
                    pending.push((nx, ny));
                }
            }
        }
    }
    if component[2] - component[0] < height.div_ceil(3)
        || rows.iter().any(|&y| !seen[index(edge, y)])
    {
        return None;
    }
    let (new, start, end) = if right {
        if component[2] <= x1 {
            return None;
        }
        let new = (component[2] + 2).min(x1 + height).min(w);
        (new, x1, new)
    } else {
        if component[0] >= x0 {
            return None;
        }
        let new = component[0]
            .saturating_sub(2)
            .max(x0.saturating_sub(height));
        (new, new, x0)
    };
    // 新たな矩形に、別の前景成分を余白ごと巻き込まない。
    for y in y0..y1 {
        for x in start..end {
            if ink(x, y) && (x < left || x >= left + window_width || !seen[index(x, y)]) {
                return None;
            }
        }
    }
    Some(new)
}
