/// 未検出領域にある8近傍の画素成分。
#[derive(Clone, Copy)]
struct Component {
    /// 左上・右下の半開画素境界。
    bounds: [usize; 4],
    /// 閾値未満の画素数。
    area: usize,
}

/// 既存行の文字高を基準に、行間で落ちた孤立字形だけを補助検出する。
pub(super) fn restore_isolated_bands(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    bands: &mut Vec<[usize; 4]>,
) {
    if bands.len() < 2 {
        return;
    }
    let mut heights: Vec<_> = bands.iter().map(|b| b[3] - b[1]).collect();
    heights.sort_unstable();
    let scale = heights[heights.len() / 2].saturating_sub(2);
    if scale < 12 {
        return;
    }
    let mut gaps = Vec::new();
    let mut top = 0;
    for b in bands.iter() {
        gaps.push((top, b[1]));
        top = b[3];
    }
    gaps.push((top, height));
    for (y0, y1) in gaps {
        if y1.saturating_sub(y0) * 4 < scale * 3 {
            continue;
        }
        let Some(components) = components_in_gap(rgb, width, y0, y1, threshold) else {
            continue;
        };
        let mut restored = Vec::<[usize; 4]>::new();
        for seed in &components {
            if !is_glyph_seed(*seed, scale) {
                continue;
            }
            let mut bounds = seed.bounds;
            // seed周囲を一巡し、句点や上部字形を連鎖的に遠くへ伸ばさない。
            for part in &components {
                if belongs_to_glyph(*seed, *part, scale) {
                    bounds[0] = bounds[0].min(part.bounds[0]);
                    bounds[1] = bounds[1].min(part.bounds[1]);
                    bounds[2] = bounds[2].max(part.bounds[2]);
                    bounds[3] = bounds[3].max(part.bounds[3]);
                }
            }
            let h = bounds[3] - bounds[1];
            if h * 4 < scale * 3 || h * 2 > scale * 3 {
                continue;
            }
            let band = [
                bounds[0].saturating_sub(2),
                bounds[1].saturating_sub(1).max(y0),
                (bounds[2] + 2).min(width),
                (bounds[3] + 1).min(y1),
            ];
            if let Some(existing) = restored
                .iter_mut()
                .find(|b| b[1] < band[3] && band[1] < b[3] && b[3].abs_diff(band[3]) <= scale / 4)
            {
                existing[0] = existing[0].min(band[0]);
                existing[1] = existing[1].min(band[1]);
                existing[2] = existing[2].max(band[2]);
                existing[3] = existing[3].max(band[3]);
            } else {
                restored.push(band);
            }
        }
        bands.extend(restored);
    }
    bands.sort_unstable_by_key(|b| (b[1], b[0]));
}

/// 文字高・縦横比・密度を満たす成分だけを補助検出の起点にする。
fn is_glyph_seed(component: Component, scale: usize) -> bool {
    let [x0, y0, x1, y1] = component.bounds;
    let (w, h) = (x1 - x0, y1 - y0);
    h * 3 >= scale * 2
        && h * 2 <= scale * 3
        && w * 4 >= h
        && w <= h * 3
        && component.area >= h * 2
        && component.area * 100 >= w * h * 8
        && component.area * 100 <= w * h * 65
}

/// 上部の分離字形か、seedと同じ基線付近にある近接句読点を含める。
fn belongs_to_glyph(seed: Component, part: Component, scale: usize) -> bool {
    let [sx0, sy0, sx1, sy1] = seed.bounds;
    let [x0, y0, x1, y1] = part.bounds;
    let (w, h) = (x1 - x0, y1 - y0);
    if part.area < 4 || part.area * 100 > w * h * 80 || w > scale * 3 || h * 2 > scale * 3 {
        return false;
    }
    let xgap = x0.saturating_sub(sx1).max(sx0.saturating_sub(x1));
    let upper_mark = x0 < sx1 && sx0 < x1 && y1 <= sy0 && sy0 - y1 <= scale / 4;
    let baseline_part = xgap <= scale && y0 < sy1 && sy0 < y1 && y1.abs_diff(sy1) <= scale / 4;
    upper_mark || baseline_part
}

/// 行間の成分を一度ずつ走査し、成分数が過剰なノイズ領域は打ち切る。
fn components_in_gap(
    rgb: &[u8],
    width: usize,
    y0: usize,
    y1: usize,
    threshold: u8,
) -> Option<Vec<Component>> {
    let h = y1 - y0;
    let mut ink = vec![false; width * h];
    for (i, pixel) in rgb[y0 * width * 3..y1 * width * 3]
        .chunks_exact(3)
        .enumerate()
    {
        ink[i] = pixel.iter().map(|&c| u16::from(c)).sum::<u16>() < u16::from(threshold) * 3;
    }
    let mut components = Vec::new();
    let mut pending = Vec::new();
    for origin in 0..ink.len() {
        if !ink[origin] {
            continue;
        }
        if components.len() == 256 {
            return None;
        }
        ink[origin] = false;
        pending.push(origin);
        let mut bounds = [
            origin % width,
            origin / width,
            origin % width + 1,
            origin / width + 1,
        ];
        let mut area = 0;
        while let Some(i) = pending.pop() {
            let (x, y) = (i % width, i / width);
            bounds[0] = bounds[0].min(x);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(x + 1);
            bounds[3] = bounds[3].max(y + 1);
            area += 1;
            for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let next = ny * width + nx;
                    if ink[next] {
                        ink[next] = false;
                        pending.push(next);
                    }
                }
            }
        }
        bounds[1] += y0;
        bounds[3] += y0;
        components.push(Component { bounds, area });
    }
    Some(components)
}
