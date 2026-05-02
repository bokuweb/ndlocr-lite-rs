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
    let spans = detect_spans(&row_ink, width, height);
    spans
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
            [
                x0,
                y0.saturating_sub(1),
                x1.max(x0 + 1),
                (y1 + 1).min(height),
            ]
        })
        .collect()
}

pub fn detect_textline_bands_fast(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<[usize; 4]> {
    let mut ink = vec![false; width * height];
    let mut row_ink = vec![0usize; height];
    for (y, row_count) in row_ink.iter_mut().enumerate() {
        let mut c = 0usize;
        for x in 0..width {
            let i = (y * width + x) * 3;
            let gray = ((rgb[i] as u16 + rgb[i + 1] as u16 + rgb[i + 2] as u16) / 3) as u8;
            if gray < threshold {
                ink[y * width + x] = true;
                c += 1;
            }
        }
        *row_count = c;
    }

    let spans = detect_spans(&row_ink, width, height);
    if spans.is_empty() {
        return Vec::new();
    }

    spans
        .into_iter()
        .map(|(y0, y1)| {
            let band_h = (y1 - y0).max(1);
            let mut col_ink = vec![0usize; width];
            for (x, col) in col_ink.iter_mut().enumerate() {
                let mut c = 0usize;
                for y in y0..y1 {
                    if ink[y * width + x] {
                        c += 1;
                    }
                }
                *col = c;
            }
            let (x0, x1) = refine_x_bounds(&col_ink, band_h, width);
            [
                x0,
                y0.saturating_sub(1),
                x1.max(x0 + 1),
                (y1 + 1).min(height),
            ]
        })
        .collect()
}

fn detect_spans(row_ink: &[usize], width: usize, height: usize) -> Vec<(usize, usize)> {
    let row_threshold = (width / 80).max(8);
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    let mut gap = 0usize;
    for (y, &ink) in row_ink.iter().enumerate() {
        if ink >= row_threshold {
            if start.is_none() {
                start = Some(y);
            }
            gap = 0;
        } else if start.is_some() {
            gap += 1;
            if gap > 2 {
                let s = start.take().unwrap_or(0);
                let e = y.saturating_sub(gap).saturating_add(1);
                if e > s + 6 {
                    spans.push((s, e));
                }
                gap = 0;
            }
        }
    }
    if let Some(s) = start
        && height > s + 6
    {
        spans.push((s, height));
    }
    spans
}

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
