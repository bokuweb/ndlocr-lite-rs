//! 切り出し矩形をまたぐ連結画素を、元RGBのまま保守的に回収する。

/// 半開矩形が、指定した元画像の画素を含むか返す。
fn contains(b: [usize; 4], x: usize, y: usize) -> bool {
    b[0] <= x && x < b[2] && b[1] <= y && y < b[3]
}

/// 二つの半開矩形が厳密に重なるか返す。
fn overlaps(a: [usize; 4], b: [usize; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// 原RGBを変えず、閾値未満の画素か判定する。
fn ink(rgb: &[u8], p: usize, threshold: u8) -> bool {
    rgb[p * 3..p * 3 + 3]
        .iter()
        .map(|&v| u16::from(v))
        .sum::<u16>()
        < u16::from(threshold) * 3
}

/// 検出帯の境界で切れた成分を、指定した最大画素数以内で完結させる。
/// 元画像は変更せず、既存の帯の順序と数を保つ。
/// 拡張部分の前景が同じ成分だけで構成される場合に限り、矩形を広げる。
/// 最大拡張量は1以上、画像の長辺以下とする。
/// 離れた字形や同形図形の意味は判定しない。既定の行検出からは自動呼出ししない。
pub fn complete_clipped_band_components(
    rgb: &[u8],
    width: usize,
    height: usize,
    bands: &[[usize; 4]],
    threshold: u8,
    max_extension: usize,
) -> Result<Vec<[usize; 4]>, String> {
    let pixels = width
        .checked_mul(height)
        .ok_or("image dimensions overflow")?;
    if width == 0 || height == 0 || pixels.checked_mul(3) != Some(rgb.len()) {
        return Err("invalid RGB dimensions".into());
    }
    if max_extension == 0 || max_extension > width.max(height) {
        return Err("invalid maximum extension".into());
    }
    if bands
        .iter()
        .any(|&[x0, y0, x1, y1]| x0 >= x1 || y0 >= y1 || x1 > width || y1 > height)
    {
        return Err("invalid band bounds".into());
    }
    let active: Vec<_> = bands
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            !bands
                .iter()
                .enumerate()
                .any(|(j, &other)| i != j && overlaps(b, other))
        })
        .collect();
    let mut output = bands.to_vec();
    if !active.iter().any(|&v| v) {
        return Ok(output);
    }
    let mut recovered = vec![Vec::new(); bands.len()];
    let mut seen = vec![false; pixels];
    let mut pending = Vec::new();
    let max_span = max_extension.saturating_mul(2);
    for (i, &[x0, y0, x1, y1]) in bands.iter().enumerate() {
        if !active[i] {
            continue;
        }
        let seeds = (x0..x1)
            .flat_map(|x| [(x, y0), (x, y1 - 1)])
            .chain((y0..y1).flat_map(|y| [(x0, y), (x1 - 1, y)]));
        for (sx, sy) in seeds {
            let seed = sy * width + sx;
            if seen[seed] || !ink(rgb, seed, threshold) {
                continue;
            }
            seen[seed] = true;
            pending.push(seed);
            let mut points = Vec::new();
            let mut bounds = [sx, sy, sx + 1, sy + 1];
            let mut owner = None;
            let mut eligible = true;
            while let Some(p) = pending.pop() {
                let (x, y) = (p % width, p / width);
                bounds = [
                    bounds[0].min(x),
                    bounds[1].min(y),
                    bounds[2].max(x + 1),
                    bounds[3].max(y + 1),
                ];
                if bounds[2] - bounds[0] > max_span || bounds[3] - bounds[1] > max_span {
                    eligible = false;
                    points.clear();
                }
                if eligible {
                    points.push(p);
                    for (j, &b) in bands.iter().enumerate() {
                        if contains(b, x, y) {
                            if owner.is_some_and(|old| old != j) {
                                eligible = false;
                                points.clear();
                                break;
                            }
                            owner = Some(j);
                        }
                    }
                }
                for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                    for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                        let next = ny * width + nx;
                        if !seen[next] && ink(rgb, next, threshold) {
                            seen[next] = true;
                            pending.push(next);
                        }
                    }
                }
            }
            let Some(j) = owner.filter(|&j| eligible && active[j]) else {
                continue;
            };
            let b = bands[j];
            if b[0].saturating_sub(bounds[0]) > max_extension
                || b[1].saturating_sub(bounds[1]) > max_extension
                || bounds[2].saturating_sub(b[2]) > max_extension
                || bounds[3].saturating_sub(b[3]) > max_extension
            {
                continue;
            }
            let out = &mut output[j];
            *out = [
                out[0].min(bounds[0]),
                out[1].min(bounds[1]),
                out[2].max(bounds[2]),
                out[3].max(bounds[3]),
            ];
            recovered[j].extend(points);
        }
    }
    for (i, b) in output.iter_mut().enumerate() {
        if *b == bands[i] {
            continue;
        }
        recovered[i].sort_unstable();
        let foreign = (b[1]..b[3]).any(|y| {
            (b[0]..b[2]).any(|x| {
                let p = y * width + x;
                !contains(bands[i], x, y)
                    && ink(rgb, p, threshold)
                    && recovered[i].binary_search(&p).is_err()
            })
        });
        if foreign {
            *b = bands[i];
        }
    }
    let proposed = output.clone();
    for (i, b) in output.iter_mut().enumerate() {
        if *b != bands[i]
            && proposed
                .iter()
                .enumerate()
                .any(|(j, &other)| i != j && overlaps(*b, other))
        {
            *b = bands[i];
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 指定座標を黒、それ以外を白とする独立したRGB入力を作る。
    fn image(w: usize, h: usize, points: &[(usize, usize)]) -> Vec<u8> {
        let mut rgb = vec![255; w * h * 3];
        for &(x, y) in points {
            rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].fill(0);
        }
        rgb
    }

    /// 全方向の境界に接する斜め画線を、半開矩形の原座標で回収する。
    #[test]
    fn completes_diagonal_strokes_at_all_four_edges() {
        for (points, expected) in [
            (vec![(3, 4), (2, 3), (1, 2)], [1, 2, 7, 7]),
            (vec![(6, 4), (7, 3), (8, 2)], [3, 2, 9, 7]),
            (vec![(4, 3), (3, 2), (2, 1)], [2, 1, 7, 7]),
            (vec![(4, 6), (3, 7), (2, 8)], [2, 3, 7, 9]),
        ] {
            let rgb = image(10, 10, &points);
            assert_eq!(
                complete_clipped_band_components(&rgb, 10, 10, &[[3, 3, 7, 7]], 220, 2).unwrap(),
                vec![expected]
            );
        }
    }

    /// 新しい包含矩形が、斜め画線から離れた図形を巻き込む場合は辞退する。
    #[test]
    fn refuses_a_rectangle_that_would_capture_a_detached_figure() {
        let rgb = image(12, 12, &[(4, 4), (3, 3), (2, 2), (2, 5)]);
        let bands = [[4, 4, 9, 9]];
        assert_eq!(
            complete_clipped_band_components(&rgb, 12, 12, &bands, 220, 3).unwrap(),
            bands
        );
    }

    /// 近い独立図形と、元帯に接続していても長い罫線は広げない。
    #[test]
    fn leaves_detached_figures_and_long_connected_rules_unchanged() {
        let mut points: Vec<_> = (0..16).map(|x| (x, 6)).collect();
        points.extend([(2, 2), (2, 3), (3, 2)]);
        let rgb = image(20, 12, &points);
        let bands = [[6, 4, 13, 9]];
        assert_eq!(
            complete_clipped_band_components(&rgb, 20, 12, &bands, 220, 3).unwrap(),
            bands
        );
    }

    /// 二帯をつなぐ成分と、拡張後の別帯との衝突は辞退する。
    #[test]
    fn does_not_merge_two_band_owners() {
        let rgb = image(12, 12, &[(5, 4), (5, 5), (5, 6), (5, 7)]);
        let bands = [[3, 2, 8, 5], [3, 7, 8, 10]];
        assert_eq!(
            complete_clipped_band_components(&rgb, 12, 12, &bands, 220, 3).unwrap(),
            bands
        );
    }

    /// 不正寸法・RGB長・帯境界・ゼロ拡張量・積のoverflowを拒否する。
    #[test]
    fn rejects_invalid_inputs() {
        for (w, h, rgb, bands, extension) in [
            (0, 1, vec![], vec![], 1),
            (2, 2, vec![255; 11], vec![], 1),
            (2, 2, vec![255; 12], vec![[0, 0, 3, 2]], 1),
            (2, 2, vec![255; 12], vec![[1, 0, 1, 2]], 1),
            (2, 2, vec![255; 12], vec![], 0),
            (usize::MAX, 2, vec![], vec![], 1),
        ] {
            assert!(complete_clipped_band_components(&rgb, w, h, &bands, 220, extension).is_err());
        }
    }
    /// 複数成分の包含矩形が別の帯へ達する場合は両方を保持する。
    #[test]
    fn refuses_new_rectangular_collisions_between_bands() {
        let points = [(3, 3), (2, 2), (1, 1), (1, 4)];
        let rgb = image(12, 12, &points);
        let bands = [[3, 3, 8, 8], [0, 4, 2, 7]];
        assert_eq!(
            complete_clipped_band_components(&rgb, 12, 12, &bands, 220, 3).unwrap(),
            bands
        );
    }

    /// 閾値と同じ明度は回収せず、暗い画素は対角を介して回収する。
    #[test]
    fn honors_threshold_and_zero_band_inputs() {
        let mut rgb = image(10, 10, &[(3, 4)]);
        rgb[(4 * 10 + 2) * 3..(4 * 10 + 2) * 3 + 3].fill(220);
        let bands = [[3, 3, 7, 7]];
        assert_eq!(
            complete_clipped_band_components(&rgb, 10, 10, &bands, 220, 2).unwrap(),
            bands
        );
        rgb[(4 * 10 + 2) * 3..(4 * 10 + 2) * 3 + 3].fill(219);
        assert_eq!(
            complete_clipped_band_components(&rgb, 10, 10, &bands, 220, 2).unwrap(),
            [[2, 3, 7, 7]]
        );
        assert!(
            complete_clipped_band_components(&rgb, 10, 10, &[], 220, 2)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            complete_clipped_band_components(&rgb, 10, 10, &bands, 0, 2).unwrap(),
            bands
        );
    }

    use proptest::prelude::*;
    proptest! {
        /// 任意の疎な画像で、追加前景が元帯と8近傍でつながることを独立集合探索で検査する。
        #[test]
        fn every_added_pixel_has_an_independent_connected_owner(values in proptest::collection::vec(0u8..6, 144)) {
            use std::collections::BTreeSet;
            let points: Vec<_> = values.iter().enumerate().filter(|&(_, v)| *v == 0).map(|(p, _)| (p % 12, p / 12)).collect();
            let rgb = image(12, 12, &points);
            let band = [4, 4, 8, 8];
            let out = complete_clipped_band_components(&rgb, 12, 12, &[band], 220, 3).unwrap()[0];
            prop_assert!(out[0] <= 4 && out[1] <= 4 && out[2] >= 8 && out[3] >= 8);
            prop_assert!(out[0] >= 1 && out[1] >= 1 && out[2] <= 11 && out[3] <= 11);
            let ink: BTreeSet<_> = points.into_iter().collect();
            for &(x, y) in &ink {
                if (out[0]..out[2]).contains(&x) && (out[1]..out[3]).contains(&y)
                    && !((4..8).contains(&x) && (4..8).contains(&y)) {
                    let mut component = BTreeSet::from([(x, y)]);
                    loop {
                        let previous = component.len();
                        for &point in &ink {
                            if component.iter().any(|&(a, b)| a.abs_diff(point.0) <= 1 && b.abs_diff(point.1) <= 1) {
                                component.insert(point);
                            }
                        }
                        if component.len() == previous { break; }
                    }
                    prop_assert!(component.iter().any(|&(a, b)| (4..8).contains(&a) && (4..8).contains(&b)));
                    let xs: Vec<_> = component.iter().map(|p| p.0).collect();
                    let ys: Vec<_> = component.iter().map(|p| p.1).collect();
                    prop_assert!(xs.iter().max().unwrap() - xs.iter().min().unwrap() < 6);
                    prop_assert!(ys.iter().max().unwrap() - ys.iter().min().unwrap() < 6);
                }
            }
        }

        /// 左右反転しても回収結果が反転し、順番依存の片側優先を作らない。
        #[test]
        fn horizontal_reflection_is_equivariant(values in proptest::collection::vec(0u8..6, 144)) {
            let points: Vec<_> = values.iter().enumerate().filter(|&(_, v)| *v == 0).map(|(p, _)| (p % 12, p / 12)).collect();
            let rgb = image(12, 12, &points);
            let reflected: Vec<_> = points.iter().map(|&(x,y)| (11-x,y)).collect();
            let other = image(12, 12, &reflected);
            let out = complete_clipped_band_components(&rgb, 12, 12, &[[3,4,7,8]], 220, 3).unwrap()[0];
            let mirrored = complete_clipped_band_components(&other, 12, 12, &[[5,4,9,8]], 220, 3).unwrap()[0];
            prop_assert_eq!(mirrored, [12-out[2],out[1],12-out[0],out[3]]);
        }
    }
}
