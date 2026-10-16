use super::band_bounds::restore_edge_ink;
use super::component_bands::{restore_isolated_bands, restore_truncated_bands};
use super::fragment_bands::rejoin_fragmented_bands;

/// 行・列のインク密度で横書き行を検出し、細い行端字形を保持する。
pub fn detect_textline_bands_naive(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<[usize; 4]> {
    let mut row_ink = vec![0usize; height];
    for (y, row_count) in row_ink.iter_mut().enumerate() {
        let mut count = 0usize;
        for x in 0..width {
            let i = (y * width + x) * 3;
            let gray = ((rgb[i] as u16 + rgb[i + 1] as u16 + rgb[i + 2] as u16) / 3) as u8;
            if gray < threshold {
                count += 1;
            }
        }
        *row_count = count;
    }
    let spans = detect_spans(&row_ink, width);
    let mut bands = spans
        .into_iter()
        .map(|(y0, y1)| {
            let band_h = (y1 - y0).max(1);
            let mut col_ink = vec![0usize; width];
            for (x, col) in col_ink.iter_mut().enumerate() {
                let mut c = 0usize;
                for y in y0..y1 {
                    let i = (y * width + x) * 3;
                    let gray = ((rgb[i] as u16 + rgb[i + 1] as u16 + rgb[i + 2] as u16) / 3) as u8;
                    if gray < threshold {
                        c += 1;
                    }
                }
                *col = c;
            }
            let (x0, x1) = refine_x_bounds(&col_ink, band_h, width);
            restore_edge_ink(
                rgb,
                width,
                [
                    x0,
                    y0.saturating_sub(1),
                    x1.max(x0 + 1),
                    (y1 + 1).min(height),
                ],
                threshold,
            )
        })
        .collect();
    rejoin_fragmented_bands(rgb, width, height, threshold, &mut bands);
    restore_truncated_bands(rgb, width, height, threshold, &mut bands);
    restore_isolated_bands(rgb, width, height, threshold, &mut bands);
    bands
}

/// インクマスクを共有して横書き行を検出し、細い行端字形を保持する。
pub fn detect_textline_bands_fast(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<[usize; 4]> {
    let threshold_sum = threshold as u16 * 3;
    let stride = width * 3;
    let mut ink = vec![false; width * height];
    let mut row_ink = vec![0usize; height];
    for (y, row_count) in row_ink.iter_mut().enumerate() {
        let mut c = 0usize;
        let row = &rgb[(y * stride)..((y + 1) * stride)];
        for x in 0..width {
            let i = x * 3;
            let sum = row[i] as u16 + row[i + 1] as u16 + row[i + 2] as u16;
            if sum < threshold_sum {
                ink[y * width + x] = true;
                c += 1;
            }
        }
        *row_count = c;
    }

    let spans = detect_spans(&row_ink, width);
    if spans.is_empty() {
        return Vec::new();
    }

    let mut bands = spans
        .into_iter()
        .map(|(y0, y1)| {
            let band_h = (y1 - y0).max(1);
            let mut col_ink = vec![0usize; width];
            for y in y0..y1 {
                let row = &ink[(y * width)..((y + 1) * width)];
                for (x, &is_ink) in row.iter().enumerate() {
                    if is_ink {
                        col_ink[x] += 1;
                    }
                }
            }
            let (x0, x1) = refine_x_bounds(&col_ink, band_h, width);
            restore_edge_ink(
                rgb,
                width,
                [
                    x0,
                    y0.saturating_sub(1),
                    x1.max(x0 + 1),
                    (y1 + 1).min(height),
                ],
                threshold,
            )
        })
        .collect();
    rejoin_fragmented_bands(rgb, width, height, threshold, &mut bands);
    restore_truncated_bands(rgb, width, height, threshold, &mut bands);
    restore_isolated_bands(rgb, width, height, threshold, &mut bands);
    bands
}

/// 強い画素行を含む低密度の候補を保ち、短いノイズと行間を除く。
fn detect_spans(row_ink: &[usize], width: usize) -> Vec<(usize, usize)> {
    let strong_threshold = (width / 80).max(8);
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    let mut end = 0usize;
    let mut seeded = false;
    let mut gap = 0usize;
    for (y, &ink) in row_ink.iter().enumerate() {
        if ink >= 8 {
            start.get_or_insert(y);
            end = y + 1;
            seeded |= ink >= strong_threshold;
            gap = 0;
        } else if let Some(s) = start {
            gap += 1;
            if gap > 2 {
                if seeded && end > s + 6 {
                    spans.push((s, end));
                }
                start = None;
                seeded = false;
                gap = 0;
            }
        }
    }
    if let Some(s) = start
        && seeded
        && end > s + 6
    {
        spans.push((s, end));
    }
    spans
}

/// 列のインク密度から左右の余白を決める。
fn refine_x_bounds(col_ink: &[usize], band_h: usize, width: usize) -> (usize, usize) {
    let max_ink = col_ink.iter().copied().max().unwrap_or(0);
    if max_ink == 0 {
        return (0, width);
    }
    let abs_threshold = (band_h / 8).max(1);
    let rel_threshold = ((max_ink as f32) * 0.15).ceil() as usize;
    let threshold = abs_threshold.max(rel_threshold.max(1));

    let mut left = 0usize;
    let mut right = width;
    for (x, &v) in col_ink.iter().enumerate() {
        if v >= threshold {
            left = x.saturating_sub(2);
            break;
        }
    }
    for (x, &v) in col_ink.iter().enumerate().rev() {
        if v >= threshold {
            right = (x + 3).min(width);
            break;
        }
    }
    (left, right)
}
