//! 複数行を囲む左右の細い縦罫線を、行画像の外側へ残す。

/// 元RGBと上下座標を保ち、支持のある左右の枠線だけを行領域から外す。
pub(super) fn trim_frame_sides(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    bands: &mut [[usize; 4]],
) {
    if bands.len() < 3 {
        return;
    }
    let source = bands.to_vec();
    let ink = |x: usize, y: usize| {
        rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
            .iter()
            .map(|&v| u16::from(v))
            .sum::<u16>()
            < u16::from(threshold) * 3
    };
    for band in bands {
        let [x0, y0, x1, y1] = *band;
        let h = y1 - y0;
        if h < 12 || x1 - x0 < h * 8 {
            continue;
        }
        let first = (x0..(x0 + h / 4).min(x1)).find(|&x| (y0..y1).any(|y| ink(x, y)));
        let last = ((x1.saturating_sub(h / 4)).max(x0)..x1)
            .rev()
            .find(|&x| (y0..y1).any(|y| ink(x, y)));
        if !first
            .zip(last)
            .is_some_and(|(left, right)| (y0..y1).all(|y| ink(left, y) && ink(right, y)))
        {
            continue;
        }
        let mut groups = Vec::new();
        let (mut start, mut top, mut bottom) = (None, y1, y0);
        for x in x0..=x1 {
            let mut column = false;
            if x < x1 {
                for y in y0..y1 {
                    if ink(x, y) {
                        column = true;
                        top = top.min(y);
                        bottom = bottom.max(y + 1);
                    }
                }
            }
            if column {
                start.get_or_insert(x);
            } else if let Some(left) = start.take() {
                groups.push((left, x, top, bottom));
                top = y1;
                bottom = y0;
            }
        }
        if groups.len() < 8 {
            continue;
        }
        let left = groups[0];
        let right = *groups.last().unwrap();
        let thin_rule = |g: (usize, usize, usize, usize)| {
            g.1 - g.0 <= (h / 12).max(1) && (g.0..g.1).all(|x| (y0..y1).all(|y| ink(x, y)))
        };
        if !thin_rule(left) || !thin_rule(right) {
            continue;
        }
        let extent = |x: usize| {
            let (mut top, mut bottom) = (y0, y1);
            while top > 0 && ink(x, top - 1) {
                top -= 1;
            }
            while bottom < height && ink(x, bottom) {
                bottom += 1;
            }
            (top, bottom)
        };
        let l = extent(left.0);
        let r = extent(right.0);
        let (top, bottom) = (l.0.max(r.0), l.1.min(r.1));
        if bottom - top < h * 3
            || l.0.abs_diff(r.0) > h / 4
            || l.1.abs_diff(r.1) > h / 4
            || source
                .iter()
                .filter(|b| b[0] <= left.0 && b[2] >= right.1 && b[1] >= top && b[3] <= bottom)
                .count()
                < 3
        {
            continue;
        }
        let body = &groups[1..groups.len() - 1];
        let left_gap = body[0].0 - left.1;
        let right_gap = right.0 - body.last().unwrap().1;
        if left_gap < h.div_ceil(4)
            || right_gap < h.div_ceil(4)
            || left_gap.max(right_gap) < h
            || body
                .iter()
                .filter(|g| g.1 - g.0 <= h * 2 && g.3 - g.2 >= h.div_ceil(3))
                .count()
                < 6
        {
            continue;
        }
        band[0] = body[0].0.saturating_sub(2).max(x0);
        band[2] = (body.last().unwrap().1 + 2).min(x1);
    }
}
