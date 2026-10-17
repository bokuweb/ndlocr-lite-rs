use super::component_bands::components_in_region;

/// 通常行高を示す複数行があるときだけ、結合した高い領域の非連結な谷を分ける。
pub(super) fn separate_noisy_line_gaps(
    rgb: &[u8],
    width: usize,
    row_ink: &[usize],
    threshold: u8,
    spans: Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut heights: Vec<_> = spans.iter().map(|&(start, end)| end - start).collect();
    heights.sort_unstable();
    let Some(&scale) = heights.get(heights.len() / 4) else {
        return spans;
    };
    if scale < 12
        || heights
            .iter()
            .filter(|&&h| h >= scale / 2 && h <= scale * 3 / 2)
            .count()
            < 2
    {
        return spans;
    }
    let mut result = Vec::new();
    for (start, end) in spans {
        if end - start < scale * 2 {
            result.push((start, end));
            continue;
        }
        let peak = row_ink[start..end].iter().copied().max().unwrap_or(0);
        let density_limit = (peak / 20).max(8);
        let mut previous = start;
        let mut y = start;
        while y < end {
            if row_ink[y] > density_limit {
                y += 1;
                continue;
            }
            let valley_start = y;
            while y < end && row_ink[y] <= density_limit {
                y += 1;
            }
            if y - valley_start < 3 {
                continue;
            }
            let middle = valley_start + (y - valley_start) / 2;
            let cut = (valley_start..y)
                .min_by_key(|&row| (row_ink[row], row.abs_diff(middle)))
                .unwrap();
            if cut - previous < scale / 2 || end - cut < scale / 2 {
                continue;
            }
            let minimum_ink_height = scale - scale / 4;
            if !has_body_height(&row_ink[previous..cut], density_limit, minimum_ink_height)
                || !has_body_height(&row_ink[cut..end], density_limit, minimum_ink_height)
            {
                continue;
            }
            let top = cut.saturating_sub(scale / 4).max(start);
            let bottom = (cut + scale / 4).min(end);
            let Some(components) =
                components_in_region(rgb, width, [0, top, width, bottom], threshold)
            else {
                continue;
            };
            if components
                .iter()
                .any(|c| c.bounds[1] == top && c.bounds[3] == bottom)
            {
                continue;
            }
            result.push((previous, cut));
            previous = cut;
        }
        result.push((previous, end));
    }
    result
}

/// 疎な汚れや空白を含めず、本文相当の連続したインク高があるかを調べる。
fn has_body_height(rows: &[usize], density_limit: usize, minimum_height: usize) -> bool {
    let mut run = 0;
    for &ink in rows {
        if ink > density_limit {
            run += 1;
            if run >= minimum_height {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::separate_noisy_line_gaps;

    /// 局所成分数の上限を超えた候補は、安全と判断せず元領域を保つ。
    #[test]
    fn component_limit_preserves_region() {
        let width = 2000;
        let mut rgb = vec![255u8; width * 200 * 3];
        let mut row_ink = vec![0usize; 200];
        let spans = vec![(0, 30), (50, 80), (100, 200)];
        for (y, count) in row_ink.iter_mut().enumerate() {
            if y < 30 || (50..80).contains(&y) || (100..130).contains(&y) || y >= 170 {
                rgb[y * width * 3..(y + 1) * width * 3].fill(0);
                *count = width;
            } else if (130..170).contains(&y) {
                for i in 0..20 {
                    let x = i * 20 + y % 2 * 10;
                    rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    *count += 1;
                }
            }
        }
        assert_eq!(
            separate_noisy_line_gaps(&rgb, width, &row_ink, 220, spans.clone()),
            spans
        );
    }
}
