use super::BlankGap;

/// 全幅・全高字形の中心距離が6組以上で揃う場合だけ、一定の字送りを返す。
pub(super) fn supported_glyph_advance(
    widths: &[usize],
    extents: &[usize],
    gaps: &[BlankGap],
    height: usize,
) -> Option<usize> {
    let full = |i: usize| {
        widths[i] >= height.saturating_mul(2).div_ceil(3)
            && widths[i] <= height
            && extents[i] >= height.saturating_mul(2).div_ceil(3)
    };
    let mut distances: Vec<u128> = gaps
        .iter()
        .enumerate()
        .filter(|(i, _)| full(*i) && full(*i + 1))
        .map(|(i, gap)| widths[i] as u128 + gap.width as u128 * 2 + widths[i + 1] as u128)
        // 行高の4/5〜5/4に収まる字送りだけを候補とする。
        .filter(|&distance| {
            distance * 5 >= height as u128 * 8 && distance * 2 <= height as u128 * 5
        })
        .collect();
    if distances.len() < 6 {
        return None;
    }
    distances.sort_unstable();
    let median = distances[distances.len() / 2];
    // 中央値±行高/8に6組以上かつ候補の3/4以上が集まることを要求する。
    let supported = distances
        .iter()
        .filter(|&&distance| distance.abs_diff(median) * 4 <= height as u128)
        .count();
    (supported >= 6 && supported >= distances.len().saturating_mul(3).div_ceil(4))
        .then(|| usize::try_from(median.div_ceil(2)).ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::super::blank_gaps;
    use proptest::prelude::*;

    /// 全高の独立字形を一定の中心間隔で合成し、中央に判定対象の間隔を置く。
    fn glyph_row(
        count: usize,
        target_gap: usize,
        irregular: bool,
        scale: usize,
    ) -> (Vec<u8>, usize, usize) {
        let height = 48 * scale;
        let widths = vec![32 * scale; count];
        let mut gaps = vec![17 * scale; count - 1];
        if irregular {
            for (i, gap) in gaps.iter_mut().enumerate() {
                *gap = if i % 2 == 0 { 8 * scale } else { 26 * scale };
            }
        }
        gaps[count / 2 - 1] = target_gap * scale;
        let width = widths.iter().sum::<usize>() + gaps.iter().sum::<usize>();
        let mut rgb = vec![255; width * height * 3];
        let mut x = 0;
        for (i, glyph) in widths.into_iter().enumerate() {
            for y in 4 * scale..44 * scale {
                for col in x..x + glyph {
                    rgb[(y * width + col) * 3..(y * width + col + 1) * 3].fill(0);
                }
            }
            x += glyph + gaps.get(i).copied().unwrap_or_default();
        }
        (rgb, width, height)
    }

    /// 支持のある送り幅では字形余白を引き、実空白と不規則な行は保護する。
    #[test]
    fn consistent_advance_requires_six_pairs_and_preserves_real_spaces() {
        for (count, gap, irregular, expected) in [
            (16, 20, false, false),
            (16, 23, false, false),
            (16, 24, false, true),
            (16, 39, false, true),
            (6, 20, false, true),
            (16, 20, true, true),
        ] {
            let (rgb, width, height) = glyph_row(count, gap, irregular, 1);
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(
                gaps[count / 2 - 1].word_space,
                expected,
                "{count}/{gap}/{irregular}"
            );
            assert_eq!(gaps[count / 2 - 1].preferred_split, gap >= 20);
        }
    }

    /// 背の低い成分・連結字形・細い英字だけでは一定の全角字送りを採用しない。
    #[test]
    fn unsupported_glyph_shapes_keep_the_existing_classifier() {
        let (rgb, width, height) = glyph_row(16, 20, false, 1);
        let gaps = blank_gaps(&rgb, width, height, 220);
        for (glyph_width, extent) in [(32, 20), (96, 40), (12, 40)] {
            assert_eq!(
                super::supported_glyph_advance(&[glyph_width; 16], &[extent; 16], &gaps, height,),
                None
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]
        /// 整数倍拡大・左右移動・反転でも空白属性と従来の優先分割を維持する。
        #[test]
        fn regular_bearings_scale(
            scale in 1usize..5, real_space in any::<bool>(),
            padding in 0usize..17, reflected in any::<bool>(),
        ) {
            let gap = if real_space { 41 } else { 20 };
            let (rgb, width, height) = glyph_row(16, gap, false, scale);
            let padded_width = width + 2 * padding;
            let mut padded = vec![255; padded_width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    let source = if reflected { width - x - 1 } else { x };
                    padded[(y * padded_width + padding + x) * 3..(y * padded_width + padding + x + 1) * 3]
                        .copy_from_slice(&rgb[(y * width + source) * 3..(y * width + source + 1) * 3]);
                }
            }
            let gaps = blank_gaps(&padded, padded_width, height, 220);
            prop_assert_eq!(gaps[7].word_space, real_space);
            prop_assert!(gaps[7].preferred_split);
        }
    }
}
