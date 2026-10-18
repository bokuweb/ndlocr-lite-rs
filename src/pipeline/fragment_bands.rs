use super::component_bands::has_bridging_glyph;

/// 通常文字高より短い上下断片を、実際につながる文字成分がある場合だけ戻す。
pub(super) fn rejoin_fragmented_bands(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    bands: &mut Vec<[usize; 4]>,
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
    let mut result: Vec<[usize; 4]> = Vec::with_capacity(bands.len());
    let mut i = 0;
    while i < bands.len() {
        let upper = bands[i];
        if let Some(&lower) = bands.get(i + 1)
            && let Some(mut bounds) = candidate_bounds(upper, lower, scale, width, height)
        {
            bounds[1] = bounds[1].max(result.last().map_or(0, |b| b[3]));
            bounds[3] = bounds[3].min(bands.get(i + 2).map_or(height, |b| b[1]));
            if has_bridging_glyph(rgb, width, bounds, upper[3], lower[1], scale, threshold) {
                result.push(bounds);
                i += 2;
                continue;
            }
        }
        result.push(upper);
        i += 1;
    }
    *bands = result;
}

/// 横位置と結合後の文字寸法を検証し、低密度の外端を含む余白を返す。
fn candidate_bounds(
    upper: [usize; 4],
    lower: [usize; 4],
    scale: usize,
    width: usize,
    height: usize,
) -> Option<[usize; 4]> {
    let (uh, lh) = (upper[3] - upper[1], lower[3] - lower[1]);
    let combined = lower[3].saturating_sub(upper[1]);
    let x0 = upper[0].min(lower[0]);
    let x1 = upper[2].max(lower[2]);
    if uh * 4 >= scale * 3
        || lh * 4 >= scale * 3
        || upper[3] > lower[1]
        || lower[1] - upper[3] > scale / 2
        || combined * 4 < scale * 3
        || combined * 2 > scale * 3
        || upper[0] >= lower[2]
        || lower[0] >= upper[2]
        || x1 - x0 > scale * 3
    {
        return None;
    }
    let padding = (scale / 16).max(1);
    let y0 = upper[1].saturating_sub(padding);
    let y1 = (lower[3] + padding).min(height);
    // 横書きの結合字形を、公式PARSeqの縦書き回転条件へ入れない。
    let target_width = (x1 - x0).max(((y1 - y0) * 5).div_ceil(4));
    if target_width > width {
        return None;
    }
    let left = x0
        .saturating_sub((target_width - (x1 - x0)) / 2)
        .min(width - target_width);
    Some([left, y0, left + target_width, y1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 候補を作るための通常行と、上下に分かれた文字相当の行を返す。
    fn bands() -> Vec<[usize; 4]> {
        vec![[10, 5, 100, 51], [30, 70, 74, 84], [30, 102, 74, 116]]
    }

    /// 塗り潰し矩形を文字の連結成分として採用しない。
    #[test]
    fn solid_blocks_cannot_bridge_fragments() {
        let (width, height) = (200, 150);
        let mut rgb = vec![255; width * height * 3];
        for y in 72..114 {
            rgb[(y * width + 32) * 3..(y * width + 72) * 3].fill(0);
        }
        let mut actual = bands();
        rejoin_fragmented_bands(&rgb, width, height, 220, &mut actual);
        assert_eq!(actual, bands());
    }

    /// 通常行高の推定値が不足する場合は結合を行わない。
    #[test]
    fn insufficient_page_context_leaves_fragments_untouched() {
        let mut actual = bands()[1..].to_vec();
        let before = actual.clone();
        rejoin_fragmented_bands(&vec![0; 200 * 150 * 3], 200, 150, 220, &mut actual);
        assert_eq!(actual, before);
    }

    /// 横書きの単独字形を、PARSeqが縦書きとして回転する寸法で返さない。
    #[test]
    fn joined_horizontal_glyph_has_enough_side_bearing() {
        let bounds = candidate_bounds([30, 70, 74, 84], [30, 102, 74, 116], 44, 200, 150).unwrap();
        assert!(
            (bounds[2] - bounds[0]) * 4 >= (bounds[3] - bounds[1]) * 5,
            "{bounds:?}"
        );
    }

    /// 横に離れた行や大きすぎる間隔・通常高の行を結合候補にしない。
    #[test]
    fn distinct_rows_do_not_become_candidates() {
        let upper = [30, 70, 74, 84];
        for lower in [
            [80, 102, 124, 116],
            [30, 120, 74, 134],
            [30, 102, 74, 148],
            [30, 102, 200, 116],
        ] {
            assert_eq!(candidate_bounds(upper, lower, 44, 200, 150), None);
        }
    }

    /// 追加する外端余白が隣の行領域へ入り込まない。
    #[test]
    fn extra_padding_never_overlaps_neighboring_rows() {
        let (width, height) = (200, 160);
        let mut rgb = vec![255; width * height * 3];
        for y in 54..96 {
            let left = if !(64..86).contains(&y) { 32 } else { 69 };
            rgb[(y * width + left) * 3..(y * width + 72) * 3].fill(0);
        }
        let mut actual = vec![
            [10, 5, 100, 51],
            [30, 52, 74, 66],
            [30, 84, 74, 98],
            [10, 99, 100, 145],
        ];
        rejoin_fragmented_bands(&rgb, width, height, 220, &mut actual);
        assert_eq!(actual.len(), 3, "{actual:?}");
        assert!(
            actual[0][3] <= actual[1][1] && actual[1][3] <= actual[2][1],
            "{actual:?}"
        );
    }

    proptest::proptest! {
        /// ページ端で余白を片側へ寄せても、元範囲・画像境界・横書き比率を保つ。
        #[test]
        fn side_bearing_stays_inside_the_page(x in 0usize..200, margin in 0usize..40) {
            let width = x + 44 + margin;
            if let Some(bounds) = candidate_bounds([x, 70, x + 44, 84], [x, 102, x + 44, 116], 44, width, 150) {
                proptest::prop_assert!(bounds[0] <= x && bounds[2] >= x + 44);
                proptest::prop_assert!(bounds[2] <= width && bounds[1] <= 70 && bounds[3] >= 116);
                proptest::prop_assert!((bounds[2] - bounds[0]) * 4 >= (bounds[3] - bounds[1]) * 5);
            } else {
                proptest::prop_assert!(width < 63);
            }
        }
    }
}
