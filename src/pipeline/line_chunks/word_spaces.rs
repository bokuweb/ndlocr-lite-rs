use super::BlankGap;

/// 複数の実単語間隔が字間と分離している行では、実画像の間隔を使う。
pub(super) fn word_gap_threshold(
    gaps: &[BlankGap],
    typical: usize,
    height: usize,
    connected_runs: usize,
) -> usize {
    let fallback = typical.saturating_mul(3).div_ceil(5);
    if gaps.len() < 6 {
        return fallback;
    }
    if typical >= height.saturating_mul(2).div_ceil(3) && connected_runs < 3 {
        return strong_connected_gap_threshold(gaps, height, connected_runs)
            .map_or(fallback, |threshold| fallback.min(threshold));
    }
    let mut widths: Vec<_> = gaps.iter().map(|gap| gap.width).collect();
    widths.sort_unstable();
    for first in widths.len().div_ceil(2)..=widths.len() - 3 {
        let (small, large) = (widths[first - 1], widths[first]);
        // 6空白以上の支持がある場合だけ、曖昧な中間幅1個を群から除外する。
        let separated = (large as u128) >= (small as u128) * 2
            || (widths.len() - first >= 6
                && first >= 2
                && (large as u128) >= (widths[first - 2] as u128) * 2);
        if !separated {
            continue;
        }
        // 混在書体でも絶対下限と2倍の広がりを守るよう、群の下端を保守的に上げる。
        let threshold = large
            .max(height.div_ceil(5))
            .max(widths.last().unwrap().div_ceil(2));
        let retained = widths.len() - widths.partition_point(|&width| width < threshold);
        // 元の群は3個以上、絞り込んだ群は6個以上の支持を要求する。
        if threshold == large || retained >= 6 {
            return fallback.min(threshold);
        }
    }
    fallback
}

/// 連結字形で中央値が膨らんでも、5空白以上が字間の2.5倍なら実間隔を採る。
pub(super) fn strong_connected_gap_threshold(
    gaps: &[BlankGap],
    height: usize,
    connected_runs: usize,
) -> Option<usize> {
    if connected_runs == 0 || gaps.len() < 10 {
        return None;
    }
    let mut widths: Vec<_> = gaps.iter().map(|gap| gap.width).collect();
    widths.sort_unstable();
    for first in widths.len().div_ceil(2)..=widths.len() - 5 {
        let (small, large) = (widths[first - 1], widths[first]);
        if (large as u128) * 2 < (small as u128) * 5 {
            continue;
        }
        let threshold = large
            .max(height.div_ceil(5))
            .max(widths.last()?.div_ceil(2));
        let retained = widths.len() - widths.partition_point(|&width| width < threshold);
        if retained >= 3 {
            return Some(threshold);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// 字間と単語間隔の二群に十分な支持がある場合だけ、字幅基準を下げる。
    #[test]
    fn proportional_gap_population_requires_separation_and_support() {
        for (widths, typical, height, expected) in [
            (vec![1, 1, 2, 2, 3, 10, 12, 15], 23, 37, 10),
            (vec![1, 2, 3, 5, 6, 12, 13, 15], 23, 47, 12),
            (vec![1, 2, 3, 5, 7, 12, 13, 15], 23, 47, 14),
            (vec![1, 2, 3, 4, 12, 13], 23, 47, 14),
            (vec![1, 2, 3, 12, 13, 14, 15], 23, 47, 14),
            (vec![1, 2, 3, 12, 13, 30], 23, 47, 14),
            (vec![1, 1, 2, 2, 3, 10, 12, 15], 25, 37, 15),
            (vec![1, 1, 2, 2, 2, 6, 7, 8], 23, 47, 14),
            (vec![1, 2, 5, 10, 12, 20], 23, 47, 10),
            (vec![1, 2, 5, 10, 12, 21], 23, 47, 14),
            (vec![1, 2, 4, 9, 12, 15], 23, 47, 14),
            // 6個以上の単語空白なら、1個だけ広い字間が混ざっても支持が残る。
            (
                vec![1, 1, 2, 2, 3, 4, 7, 12, 13, 13, 14, 15, 20],
                22,
                48,
                12,
            ),
            (vec![1, 1, 2, 2, 3, 4, 7, 12, 13, 13, 14, 20], 22, 48, 14),
            (
                vec![1, 1, 2, 2, 3, 4, 7, 12, 13, 13, 14, 15, 25],
                22,
                48,
                14,
            ),
            // 広い字間が2個なら分離した二群とは扱わない。
            (
                vec![1, 1, 2, 2, 3, 7, 7, 12, 13, 13, 14, 15, 20],
                22,
                48,
                14,
            ),
        ] {
            let gaps: Vec<_> = widths
                .iter()
                .map(|&width| BlankGap {
                    midpoint: 0,
                    width,
                    word_space: false,
                    preferred_split: false,
                    internal_glyph_gap: false,
                })
                .collect();
            assert_eq!(
                word_gap_threshold(&gaps, typical, height, 0),
                expected,
                "{widths:?}, {typical}/{height}"
            );
        }
    }

    /// 連結字形が3個未満なら全角相当中央値の除外を維持する。
    #[test]
    fn joined_glyph_support_does_not_relax_wide_rows_without_evidence() {
        let gaps: Vec<_> = [1, 1, 2, 2, 3, 3, 4, 5, 5, 11, 14, 14, 15, 15, 16, 20]
            .into_iter()
            .map(|width| BlankGap {
                midpoint: 0,
                width,
                word_space: false,
                preferred_split: false,
                internal_glyph_gap: false,
            })
            .collect();
        for (connected_runs, expected) in [(0, 21), (1, 21), (2, 21), (3, 11)] {
            assert_eq!(word_gap_threshold(&gaps, 35, 49, connected_runs), expected);
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(32))]
        /// 同じ間隔分布を拡大しても、支持数と空白閾値の関係を保つ。
        #[test]
        fn supported_word_gap_distribution_scales(scale in 1usize..9, connected in 0usize..5) {
            let widths = [1, 1, 2, 2, 3, 3, 4, 5, 5, 11, 14, 14, 15, 15, 16, 20];
            let gaps: Vec<_> = widths.into_iter().map(|width| BlankGap {
                midpoint: 0, width: width * scale, word_space: false,
                preferred_split: false, internal_glyph_gap: false,
            }).collect();
            let expected = if connected >= 3 { 11 * scale } else { (35 * scale * 3).div_ceil(5) };
            prop_assert_eq!(word_gap_threshold(&gaps, 35 * scale, 49 * scale, connected), expected);
            let gaps: Vec<_> = [1, 1, 2, 2, 3, 4, 7, 12, 13, 13, 14, 15, 20]
                .into_iter().map(|width| BlankGap {
                    midpoint: 0, width: width * scale, word_space: false,
                    preferred_split: false, internal_glyph_gap: false,
                }).collect();
            prop_assert_eq!(word_gap_threshold(&gaps, 22 * scale, 48 * scale, 0), 12 * scale);
        }
    }

    /// 混在書体の広い群は、絶対下限/広がり制限へ絞っても6空白残る場合だけ採る。
    #[test]
    fn mixed_format_gap_population_requires_six_retained_spaces() {
        for (widths, typical, height, expected) in [
            (
                vec![
                    1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 15, 16, 17, 17, 20, 20, 21, 21, 31,
                ],
                35,
                68,
                16,
            ),
            (
                vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 15, 16, 17, 20, 21, 31],
                35,
                68,
                21,
            ),
            (
                vec![2, 3, 4, 4, 5, 6, 7, 7, 12, 15, 16, 18, 18, 21, 29, 30],
                36,
                76,
                16,
            ),
            (
                vec![2, 3, 4, 4, 5, 6, 7, 7, 12, 13, 15, 16, 18, 18, 21, 29, 30],
                36,
                76,
                22,
            ),
            (
                vec![2, 3, 4, 4, 5, 6, 7, 7, 14, 15, 16, 18, 18, 21, 29, 30],
                36,
                76,
                16,
            ),
        ] {
            let gaps = widths
                .iter()
                .map(|&width| BlankGap {
                    midpoint: 0,
                    width,
                    word_space: false,
                    preferred_split: false,
                    internal_glyph_gap: false,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                word_gap_threshold(&gaps, typical, height, 0),
                expected,
                "{widths:?}"
            );
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(32))]
        /// 拡大や並び替えでも曖昧な幅を除き、十分な空白支持がなければ緩和しない。
        #[test]
        fn mixed_format_gap_support_survives_scale_and_permutation(
            scale in 1usize..9, ambiguous in 8usize..15, rotation in 0usize..16,
        ) {
            let mut widths = vec![2,3,4,4,5,6,7,7,ambiguous,15,16,18,18,21,29,30];
            widths.rotate_left(rotation);
            let gaps = widths.into_iter().map(|width| BlankGap {
                midpoint: 0, width: width * scale, word_space: false,
                preferred_split: false, internal_glyph_gap: false,
            }).collect::<Vec<_>>();
            let threshold = word_gap_threshold(&gaps,36*scale,76*scale,0);
            prop_assert!(threshold > ambiguous * scale);
            prop_assert!(threshold <= 18 * scale);
            let mut reversed = gaps;
            reversed.reverse();
            prop_assert_eq!(word_gap_threshold(&reversed,36*scale,76*scale,0),threshold);
            let gaps = [2,3,4,4,5,6,7,7,ambiguous,15,16,18,18,21,30]
                .into_iter().map(|width| BlankGap {
                    midpoint: 0, width: width * scale, word_space: false,
                    preferred_split: false, internal_glyph_gap: false,
                }).collect::<Vec<_>>();
            prop_assert_eq!(word_gap_threshold(&gaps,36*scale,76*scale,0),(36*scale*3).div_ceil(5));
        }
    }
}

#[cfg(test)]
mod connected_population_tests {
    use super::*;
    use proptest::prelude::*;
    /// 強く分離した5個以上の実空白と連結成分の支持だけが全角相当の除外を超える。
    #[test]
    fn strong_population_requires_joined_ink_and_five_gaps() {
        for (widths, connected, expected) in [
            (vec![1, 1, 1, 2, 2, 3, 3, 4, 11, 12, 13, 14], 1, 20),
            (vec![1, 1, 1, 2, 2, 3, 3, 4, 11, 11, 11, 12, 13, 14], 1, 11),
            (vec![1, 1, 1, 2, 2, 3, 3, 4, 11, 11, 11, 12, 13, 14], 0, 20),
            (vec![1, 1, 1, 2, 2, 3, 3, 4, 11, 11, 12, 13, 14], 1, 11),
            (vec![1, 1, 1, 2, 2, 3, 3, 5, 11, 11, 11, 12, 13, 14], 1, 20),
            (vec![1, 1, 1, 2, 2, 3, 3, 4, 11, 11, 11, 12, 13, 60], 1, 20),
            (vec![1, 1, 1, 2, 2, 3, 3, 3, 8, 10, 11, 13, 14, 16], 1, 9),
        ] {
            let gaps: Vec<_> = widths
                .into_iter()
                .map(|width| BlankGap {
                    midpoint: 0,
                    width,
                    word_space: false,
                    preferred_split: false,
                    internal_glyph_gap: false,
                })
                .collect();
            assert_eq!(word_gap_threshold(&gaps, 33, 44, connected), expected);
        }
    }
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(24))]
        /// 拡大しても強い群分離と連結成分の必要性を維持する。
        #[test]
        fn strong_population_scales(scale in 1usize..9,connected in 0usize..3) {
            let gaps:Vec<_>=[1,1,1,2,2,3,3,4,11,11,11,12,13,14].into_iter().map(|width| BlankGap {midpoint:0,width:width*scale,word_space:false,preferred_split:false,internal_glyph_gap:false}).collect();
            let expected=if connected>0 {11*scale} else {(33*scale*3).div_ceil(5)};
            prop_assert_eq!(word_gap_threshold(&gaps,33*scale,44*scale,connected),expected);
        }
    }
}
