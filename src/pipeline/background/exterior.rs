/// 補間でわずかに着色された紙も含める各成分の下限。
const MIN_PAPER_COMPONENT: u8 = 250;

/// ページ外周へ4近傍で接続する近白を、行区間単位で一度だけ訪問する。
pub(super) fn white_paper(rgb: &[u8], width: usize) -> Vec<bool> {
    let height = rgb.len() / 3 / width;
    let mut seen = vec![false; width * height];
    let mut stack = Vec::new();
    for x in 0..width {
        visit(rgb, width, x, 0, &mut seen, &mut stack);
        visit(rgb, width, x, height - 1, &mut seen, &mut stack);
    }
    for y in 1..height.saturating_sub(1) {
        visit(rgb, width, 0, y, &mut seen, &mut stack);
        visit(rgb, width, width - 1, y, &mut seen, &mut stack);
    }
    while let Some((left, right, y)) = stack.pop() {
        for next in [y.checked_sub(1), (y + 1 < height).then_some(y + 1)] {
            let Some(next) = next else {
                continue;
            };
            let mut x = left;
            while x < right {
                x = visit(rgb, width, x, next, &mut seen, &mut stack).max(x + 1);
            }
        }
    }
    seen
}

/// 未訪問の白い連続区間を記録し、隣接行の探索へ渡す。
fn visit(
    rgb: &[u8],
    width: usize,
    x: usize,
    y: usize,
    seen: &mut [bool],
    stack: &mut Vec<(usize, usize, usize)>,
) -> usize {
    let white = |x: usize, seen: &[bool]| {
        let i = y * width + x;
        !seen[i]
            && rgb[i * 3..i * 3 + 3]
                .iter()
                .all(|&v| v >= MIN_PAPER_COMPONENT)
    };
    if !white(x, seen) {
        return x + 1;
    }
    let mut left = x;
    while left > 0 && white(left - 1, seen) {
        left -= 1;
    }
    let mut right = x + 1;
    while right < width && white(right, seen) {
        right += 1;
    }
    seen[y * width + left..y * width + right].fill(true);
    stack.push((left, right, y));
    right
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// 小さな白黒迷路を単純な画素キューで解く比較用実装。
    fn reference(rgb: &[u8], w: usize) -> Vec<bool> {
        let h = rgb.len() / 3 / w;
        let mut seen = vec![false; w * h];
        let mut stack = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                    stack.push((x, y));
                }
            }
        }
        while let Some((x, y)) = stack.pop() {
            let i = y * w + x;
            if seen[i] || rgb[i * 3..i * 3 + 3].iter().any(|&v| v < 250) {
                continue;
            }
            seen[i] = true;
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < w {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < h {
                stack.push((x, y + 1));
            }
        }
        seen
    }

    #[test]
    /// 幅1・高さ1・全白・非白の端を扱う。
    fn one_dimensional_edges() {
        for (w, h) in [(1, 1), (1, 8), (8, 1)] {
            for value in [255, 254, 250, 249, 0] {
                let rgb = vec![value; w * h * 3];
                assert_eq!(white_paper(&rgb, w), vec![value >= 250; w * h]);
            }
        }
    }

    #[test]
    /// 1成分でも下限を割る画素と斜め接触を接続経路にしない。
    fn nearwhite_threshold_and_diagonal_barriers() {
        for channel in 0..3 {
            let mut barrier = [255; 3];
            barrier[channel] = 249;
            let mut rgb = vec![0; 3 * 3 * 3];
            rgb[0..3].copy_from_slice(&[250, 254, 255]);
            rgb[3..6].copy_from_slice(&barrier);
            rgb[12..15].copy_from_slice(&[250; 3]);
            assert_eq!(
                white_paper(&rgb, 3),
                vec![true, false, false, false, false, false, false, false, false]
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        /// 分岐や細路がある場合も素朴な4近傍探索と一致する。
        fn run_fill_matches_pixel_search(w in 1usize..32, bits in prop::collection::vec(any::<bool>(),1..1000)) {
            let rgb:Vec<_>=bits.iter().take(w*20).cycle().take(w*20)
                .flat_map(|&b|if b {[255;3]} else {[249,255,255]}).collect();
            prop_assert_eq!(white_paper(&rgb,w),reference(&rgb,w));
        }

        #[test]
        /// 成分ごとの下限と近白経路の分岐を独立した画素探索で照合する。
        fn nearwhite_run_fill_matches_pixel_search(w in 1usize..24, pixels in prop::collection::vec(prop::array::uniform3(248u8..=255),1..500)) {
            let rgb:Vec<_>=pixels.iter().cycle().take(w*16).flatten().copied().collect();
            prop_assert_eq!(white_paper(&rgb,w),reference(&rgb,w));
        }
    }
}
