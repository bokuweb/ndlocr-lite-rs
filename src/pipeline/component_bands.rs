/// 未検出領域にある8近傍の画素成分。
#[derive(Clone, Copy)]
pub(super) struct Component {
    /// 左上・右下の半開画素境界。
    pub(super) bounds: [usize; 4],
    /// 閾値未満の画素数。
    pub(super) area: usize,
}

/// 短い既存行に重なる文字成分を調べ、行投影で欠けた上下の画を戻す。
pub(super) fn restore_truncated_bands(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    bands: &mut [[usize; 4]],
) {
    if bands.len() < 3 {
        return;
    }
    let mut heights: Vec<_> = bands.iter().map(|b| b[3] - b[1]).collect();
    heights.sort_unstable();
    let scale = heights[heights.len() / 2].saturating_sub(2);
    if scale < 12 {
        return;
    }
    for i in 0..bands.len() {
        let band = bands[i];
        if (band[3] - band[1]) * 4 >= scale * 3 || band[2] - band[0] > scale * 6 {
            continue;
        }
        let y0 = band[1]
            .saturating_sub(scale / 2)
            .max(if i == 0 { 0 } else { bands[i - 1][3] });
        let y1 = (band[3] + scale / 2).min(bands.get(i + 1).map_or(height, |b| b[1]));
        let Some(components) = components_in_gap(rgb, width, y0, y1, threshold) else {
            continue;
        };
        let mut bounds = band;
        for component in components {
            let [cx0, cy0, cx1, cy1] = component.bounds;
            if cy0 <= y0
                || cy1 >= y1
                || cx0 < band[0].saturating_sub(scale / 4)
                || cx1 > (band[2] + scale / 4).min(width)
                || cx0 >= band[2]
                || cx1 <= band[0]
                || cy0 >= band[3]
                || cy1 <= band[1]
                || (cy0 >= band[1] && cy1 <= band[3])
                || !is_glyph_seed(component, scale)
            {
                continue;
            }
            bounds[0] = bounds[0].min(cx0.saturating_sub(2));
            bounds[2] = bounds[2].max((cx1 + 2).min(width));
            let padding = (scale / 8).max(1);
            bounds[1] = bounds[1].min(cy0.saturating_sub(padding).max(y0));
            bounds[3] = bounds[3].max((cy1 + padding).min(y1));
        }
        if (bounds[1] == band[1] && bounds[3] == band[3]) || (bounds[3] - bounds[1]) * 2 > scale * 3
        {
            continue;
        }
        let target_width = (bounds[2] - bounds[0]).max(((bounds[3] - bounds[1]) * 5).div_ceil(4));
        if target_width > width {
            continue;
        }
        let left = bounds[0]
            .saturating_sub((target_width - (bounds[2] - bounds[0])) / 2)
            .min(width - target_width);
        bands[i] = [left, bounds[1], left + target_width, bounds[3]];
    }
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
            let small_contextual =
                is_small_contextual_seed(*seed, scale, bands, [width, height], y0, y1);
            if !is_glyph_seed(*seed, scale) && !small_contextual {
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
            if (!small_contextual && h * 4 < scale * 3) || h * 2 > scale * 3 {
                continue;
            }
            let mut band = [
                bounds[0].saturating_sub(2),
                bounds[1].saturating_sub(1).max(y0),
                (bounds[2] + 2).min(width),
                (bounds[3] + 1).min(y1),
            ];
            if small_contextual {
                // 小さい書体のフッターは原寸高を保ち、本文のx-heightだけ余白を補う。
                if !is_centered_footer(seed.bounds, scale, [width, height], y0, y1) {
                    band[1] = band[1].min(band[3].saturating_sub(scale + 2).max(y0));
                }
                let target_width = ((band[3] - band[1]) * 5)
                    .div_ceil(4)
                    .max(band[2] - band[0])
                    .min(width);
                band[0] = band[0]
                    .saturating_sub((target_width - (band[2] - band[0])) / 2)
                    .min(width - target_width);
                band[2] = band[0] + target_width;
            }
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

/// 本文左端または離れた中央フッターの文脈を持つ小字形を起点にする。
fn is_small_contextual_seed(
    component: Component,
    scale: usize,
    bands: &[[usize; 4]],
    page_size: [usize; 2],
    gap_top: usize,
    gap_bottom: usize,
) -> bool {
    let [x0, y0, x1, y1] = component.bounds;
    let (w, h) = (x1 - x0, y1 - y0);
    y0 > gap_top
        && y1 < gap_bottom
        && h * 5 >= scale * 2
        && h * 4 < scale * 3
        && w * 4 >= h
        && w <= h * 2
        && component.area >= h * 2
        && component.area * 100 >= w * h * 8
        && component.area * 100 <= w * h * 65
        && (bands.iter().any(|b| {
            b[2] - b[0] >= scale * 3
                && (b[3] - b[1]) * 4 >= scale * 3
                && b[0].abs_diff(x0) <= scale / 4
        }) || is_centered_footer(component.bounds, scale, page_size, gap_top, gap_bottom))
}

/// 本文から十分離れ、上下余白がある最終ギャップの中央成分だけを認める。
fn is_centered_footer(
    bounds: [usize; 4],
    scale: usize,
    page_size: [usize; 2],
    gap_top: usize,
    gap_bottom: usize,
) -> bool {
    gap_bottom == page_size[1]
        && bounds[1].saturating_sub(gap_top) >= scale * 2
        && gap_bottom.saturating_sub(bounds[3]) >= scale
        && (bounds[0] + bounds[2]).abs_diff(page_size[0]) <= scale
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

/// 上下の断片へまたがる成分が、文字の寸法・密度を持つ場合だけ結合を許す。
pub(super) fn has_bridging_glyph(
    rgb: &[u8],
    width: usize,
    bounds: [usize; 4],
    upper_end: usize,
    lower_start: usize,
    scale: usize,
    threshold: u8,
) -> bool {
    let Some(components) = components_in_gap(rgb, width, bounds[1], bounds[3], threshold) else {
        return false;
    };
    components.into_iter().any(|component| {
        let [x0, y0, x1, y1] = component.bounds;
        x0 >= bounds[0]
            && x1 <= bounds[2]
            && y0 > bounds[1]
            && y1 < bounds[3]
            && y0 < upper_end.saturating_sub(1)
            && y1 > lower_start + 1
            && is_glyph_seed(component, scale)
    })
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
pub(super) fn components_in_gap(
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

#[cfg(test)]
mod truncated_tests {
    use super::restore_truncated_bands;

    /// 通常行2本と、下端が欠けた短い行の境界を返す。
    fn bands(x: usize) -> Vec<[usize; 4]> {
        vec![[0, 5, 100, 51], [0, 60, 100, 106], [x, 130, x + 44, 157]]
    }

    /// 矩形とは異なる密度の、上下につながる文字相当の画素を描く。
    fn glyph(rgb: &mut [u8], width: usize, x: usize, top: usize, bottom: usize) {
        for y in top..bottom {
            for dx in 0..44 {
                if dx < 4 || (y >= bottom - 4 && dx < 40) {
                    rgb[(y * width + x + dx) * 3..(y * width + x + dx) * 3 + 3].fill(0);
                }
            }
        }
    }

    /// 罫線・塗り潰し・既存行と接しない別字形・小点を外端復元に使わない。
    #[test]
    fn unrelated_or_non_glyph_ink_does_not_extend_a_band() {
        for kind in 0..4 {
            let (width, height) = (256, 240);
            let mut rgb = vec![255; width * height * 3];
            match kind {
                0 => glyph(&mut rgb, width, 100, 130, 210),
                1 => {
                    for y in 130..170 {
                        rgb[(y * width + 100) * 3..(y * width + 144) * 3].fill(0);
                    }
                }
                2 => glyph(&mut rgb, width, 100, 160, 200),
                _ => rgb[(165 * width + 110) * 3..(165 * width + 112) * 3].fill(0),
            }
            let mut actual = bands(100);
            restore_truncated_bands(&rgb, width, height, 220, &mut actual);
            assert_eq!(actual, bands(100), "kind {kind}");
        }
    }

    /// 見出しの幅と斜め画の横ずれは、等号を含む上限までだけ許容する。
    #[test]
    fn heading_width_and_diagonal_offset_have_bounded_limits() {
        for (line_width, offset, restored) in [(264, 11, true), (265, 11, false), (264, 12, false)]
        {
            let (width, height) = (512, 240);
            let mut rgb = vec![255; width * height * 3];
            glyph(&mut rgb, width, 100 - offset, 130, 170);
            let mut actual = bands(100);
            actual[2][2] = 100 + line_width;
            let before = actual.clone();
            restore_truncated_bands(&rgb, width, height, 220, &mut actual);
            assert_eq!(
                actual != before,
                restored,
                "width={line_width} offset={offset}"
            );
            if restored {
                assert!(actual[2][0] <= 89 && actual[2][3] >= 170);
            }
        }
    }

    /// 隣行の開始に達する成分は、別行との接続かもしれないので復元しない。
    #[test]
    fn a_component_touching_the_neighbor_is_rejected() {
        let mut rgb = vec![255; 256 * 240 * 3];
        glyph(&mut rgb, 256, 100, 130, 170);
        let mut actual = bands(100);
        actual.push([100, 170, 144, 216]);
        let before = actual.clone();
        restore_truncated_bands(&rgb, 256, 240, 220, &mut actual);
        assert_eq!(actual, before);
    }

    /// 通常高の文脈が足りないページは切り出しを変更しない。
    #[test]
    fn no_scale_context_means_no_extension() {
        let mut actual = bands(100)[1..].to_vec();
        let before = actual.clone();
        restore_truncated_bands(&vec![0; 256 * 240 * 3], 256, 240, 220, &mut actual);
        assert_eq!(actual, before);
    }

    /// 全画素が既存範囲内なら、短い文字でも余白を追加しない。
    #[test]
    fn an_already_complete_short_glyph_is_unchanged() {
        let mut rgb = vec![255; 256 * 240 * 3];
        glyph(&mut rgb, 256, 100, 130, 160);
        let mut actual = bands(100);
        actual[2][3] = 161;
        let before = actual.clone();
        restore_truncated_bands(&rgb, 256, 240, 220, &mut actual);
        assert_eq!(actual, before);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(32))]
        /// 左右の画像端でも元範囲と全字形を保持し、画像境界からはみ出さない。
        #[test]
        fn restored_glyph_stays_in_page(x in 0usize..200, margin in 20usize..80) {
            let width = (x + 44 + margin).max(100);
            let mut rgb = vec![255; width * 240 * 3];
            glyph(&mut rgb, width, x, 130, 170);
            let mut actual = bands(x);
            restore_truncated_bands(&rgb, width, 240, 220, &mut actual);
            let b = actual[2];
            proptest::prop_assert!(b[0] <= x && b[2] >= x + 44 && b[2] <= width);
            proptest::prop_assert!(b[1] <= 130 && b[3] >= 170);
            proptest::prop_assert!((b[2] - b[0]) * 4 >= (b[3] - b[1]) * 5);
            proptest::prop_assert!(b[1] >= actual[1][3]);
        }
    }
}

#[cfg(test)]
mod small_aligned_tests {
    use super::{Component, is_small_contextual_seed};

    /// 面積と寸法が文字相当の成分を、通常本文の左端へ置く。
    fn seed(h: usize) -> Component {
        Component {
            bounds: [100, 150, 120, 150 + h],
            area: h * 8,
        }
    }

    /// 小文字高と全高の間に取りこぼす寸法帯を作らない。
    #[test]
    fn small_height_range_reaches_the_full_height_fallback() {
        let bands = [[98, 10, 400, 62], [98, 80, 400, 132]];
        for h in 20..=37 {
            assert!(
                is_small_contextual_seed(seed(h), 50, &bands, [500, 300], 132, 250),
                "height {h}"
            );
        }
        assert!(!is_small_contextual_seed(
            seed(19),
            50,
            &bands,
            [500, 300],
            132,
            250
        ));
        assert!(!is_small_contextual_seed(
            seed(38),
            50,
            &bands,
            [500, 300],
            132,
            250
        ));
    }

    /// 本文への距離とギャップ境界を等号まで検証する。
    #[test]
    fn text_margin_and_gap_edges_bound_the_fallback() {
        let bands = [[88, 10, 400, 62], [88, 80, 400, 132]];
        assert!(is_small_contextual_seed(
            seed(20),
            50,
            &bands,
            [500, 300],
            132,
            250
        ));
        let far = [[87, 10, 400, 62], [87, 80, 400, 132]];
        assert!(!is_small_contextual_seed(
            seed(20),
            50,
            &far,
            [500, 300],
            132,
            250
        ));
        assert!(!is_small_contextual_seed(
            seed(20),
            50,
            &bands,
            [500, 300],
            150,
            250
        ));
        assert!(!is_small_contextual_seed(
            seed(20),
            50,
            &bands,
            [500, 300],
            132,
            170
        ));
    }
}
