use super::{BlankGap, LineChunk, MAX_CHUNK_INK_RUNS};

/// 単語端の小さい水平区間が縦向きに回転されないよう、収まる文脈へ戻す。
pub(super) fn merge_rotated_word_fragments(
    chunks: &mut Vec<LineChunk>,
    gaps: &[BlankGap],
    height: usize,
    input_width: usize,
    input_height: usize,
) {
    let mut index = 0;
    while index + 1 < chunks.len() {
        let left = &chunks[index];
        let right = &chunks[index + 1];
        let bounded = (index == 0 || left.space_before)
            && !right.space_before
            && chunks.get(index + 2).is_none_or(|next| next.space_before);
        let rotated = |width: usize| height as u128 * 5 > width as u128 * 4;
        let mixed_orientation = rotated(left.x1 - left.x0) != rotated(right.x1 - right.x0);
        let first = gaps.partition_point(|gap| gap.midpoint <= left.x0);
        let end = gaps.partition_point(|gap| gap.midpoint < right.x1);
        let fits = (right.x1 - left.x0) as u128 * input_height as u128
            <= height as u128 * input_width as u128;
        if bounded && mixed_orientation && fits && end - first < MAX_CHUNK_INK_RUNS {
            chunks[index].x1 = chunks[index + 1].x1;
            chunks.remove(index + 1);
        }
        index += 1;
    }
}

/// 次の実空白までの短い単語末尾を、画素を分断せず前区間へ戻す。
pub(super) fn restore_short_word_tails(
    chunks: &mut [LineChunk],
    gaps: &[BlankGap],
    height: usize,
    input_width: usize,
    input_height: usize,
) {
    for index in 0..chunks.len().saturating_sub(1) {
        let left = &chunks[index];
        let right = &chunks[index + 1];
        if (index != 0 && !left.space_before) || right.space_before {
            continue;
        }
        let first = gaps.partition_point(|gap| gap.midpoint <= right.x0);
        let Some(gap) = gaps[first..].iter().find(|gap| gap.word_space) else {
            continue;
        };
        if gap.midpoint >= right.x1 || gap.width < height.div_ceil(5) {
            continue;
        }
        let rotated = |width: usize| height as u128 * 5 > width as u128 * 4;
        let fits = (gap.midpoint - left.x0) as u128 * input_height as u128
            <= height as u128 * input_width as u128;
        let ink_start = gaps.partition_point(|gap| gap.midpoint <= left.x0);
        let ink_end = gaps.partition_point(|item| item.midpoint < gap.midpoint);
        if rotated(gap.midpoint - right.x0)
            && !rotated(left.x1 - left.x0)
            && !rotated(right.x1 - gap.midpoint)
            && fits
            && ink_end - ink_start < MAX_CHUNK_INK_RUNS
        {
            chunks[index].x1 = gap.midpoint;
            chunks[index + 1].x0 = gap.midpoint;
            chunks[index + 1].space_before = true;
            chunks[index + 1].gap_before = gap.width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回転される75列の断片が、水平な同じ単語の末尾に残る区間を作る。
    fn chunks() -> Vec<LineChunk> {
        vec![
            LineChunk {
                x0: 0,
                x1: 400,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 400,
                x1: 475,
                space_before: false,
                gap_before: 3,
            },
            LineChunk {
                x0: 475,
                x1: 800,
                space_before: true,
                gap_before: 20,
            },
        ]
    }

    /// 単語内の字間を同じ高さ・上限の条件で用意する。
    fn gaps(count: usize) -> Vec<BlankGap> {
        (1..=count)
            .map(|i| BlankGap {
                midpoint: i * 475 / (count + 1),
                width: 3,
                word_space: false,
                preferred_split: false,
                internal_glyph_gap: false,
            })
            .collect()
    }

    /// 回転される末尾だけを文脈へ戻し、ページ内座標と次の単語空白を維持する。
    #[test]
    fn short_word_fragment_rejoins_horizontal_context() {
        let mut actual = chunks();
        merge_rotated_word_fragments(&mut actual, &gaps(19), 64, 256, 24);
        assert_eq!(
            actual,
            vec![
                LineChunk {
                    x0: 0,
                    x1: 475,
                    space_before: false,
                    gap_before: 0
                },
                LineChunk {
                    x0: 475,
                    x1: 800,
                    space_before: true,
                    gap_before: 20
                },
            ]
        );
    }

    /// 列群数・モデル比率・単語境界・回転条件の不足では変更しない。
    #[test]
    fn unsupported_fragment_preserves_partition() {
        for case in 0..5 {
            let mut actual = chunks();
            let mut support = gaps(19);
            let mut input_width = 256;
            match case {
                0 => support = gaps(20),
                1 => input_width = 128,
                2 => actual[1].space_before = true,
                3 => actual[2].space_before = false,
                4 => {
                    actual[1].x1 = 480;
                    actual[2].x0 = 480;
                }
                _ => unreachable!(),
            }
            let before = actual
                .iter()
                .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                .collect::<Vec<_>>();
            merge_rotated_word_fragments(&mut actual, &support, 64, input_width, 24);
            assert_eq!(
                before,
                actual
                    .iter()
                    .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                    .collect::<Vec<_>>(),
                "case={case}"
            );
        }
    }

    /// 次の実空白を境界にし、区間数と画素被覆を保つ。
    #[test]
    fn short_tail_moves_to_next_word_space() {
        let mut actual = chunks();
        actual[1].x1 = 800;
        actual.pop();
        let support = vec![BlankGap {
            midpoint: 450,
            width: 20,
            word_space: true,
            preferred_split: true,
            internal_glyph_gap: false,
        }];
        restore_short_word_tails(&mut actual, &support, 64, 256, 24);
        assert_eq!(actual[0].x1, 450);
        assert_eq!(actual[1].x0, 450);
        assert_eq!(actual[1].x1, 800);
        assert!(actual[1].space_before);
        assert_eq!(actual[1].gap_before, 20);
    }

    /// 文脈・短さ・実空白・入力比率・列群上限が不足する場合は維持する。
    #[test]
    fn unsupported_short_tail_preserves_partition() {
        for case in 0..8 {
            let mut actual = chunks();
            actual[1].x1 = 800;
            actual.pop();
            let mut support = vec![BlankGap {
                midpoint: 450,
                width: 20,
                word_space: true,
                preferred_split: true,
                internal_glyph_gap: false,
            }];
            let mut input_width = 256;
            match case {
                0 => actual[1].space_before = true,
                1 => support[0].word_space = false,
                2 => support[0].midpoint = 500,
                3 => support[0].midpoint = 780,
                4 => input_width = 128,
                5 => {
                    support.extend(gaps(20).into_iter().map(|mut gap| {
                        gap.midpoint = gap.midpoint * 400 / 475;
                        gap
                    }));
                    support.sort_by_key(|gap| gap.midpoint);
                }
                6 => actual[0].x0 = 350,
                7 => support[0].width = 4,
                _ => unreachable!(),
            }
            let before = actual
                .iter()
                .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                .collect::<Vec<_>>();
            restore_short_word_tails(&mut actual, &support, 64, input_width, 24);
            assert_eq!(
                actual
                    .iter()
                    .map(|c| (c.x0, c.x1, c.space_before, c.gap_before))
                    .collect::<Vec<_>>(),
                before,
                "case={case}"
            );
        }
    }

    proptest::proptest! {
        /// 拡大・平行移動後も同じ空白を選び、全画素の被覆と区間数を保つ。
        #[test]
        fn short_tail_preserves_coverage(
            scale in 1usize..9, margin in 0usize..40,
        ) {
            let mut actual = vec![
                LineChunk { x0: margin, x1: 400*scale+margin, space_before: false, gap_before: 0 },
                LineChunk { x0: 400*scale+margin, x1: 800*scale+margin, space_before: false, gap_before: 3*scale },
            ];
            let support = vec![BlankGap {
                midpoint: 450*scale+margin, width: 20*scale, word_space: true,
                preferred_split: true, internal_glyph_gap: false,
            }];
            restore_short_word_tails(&mut actual, &support, 64*scale, 256, 24);
            proptest::prop_assert_eq!(actual.len(),2);
            proptest::prop_assert_eq!(actual[0].x0,margin);
            proptest::prop_assert_eq!(actual[0].x1,450*scale+margin);
            proptest::prop_assert_eq!(actual[0].x1,actual[1].x0);
            proptest::prop_assert_eq!(actual[1].x1,800*scale+margin);
            proptest::prop_assert!(actual[1].space_before);
            proptest::prop_assert_eq!(actual[1].gap_before,20*scale);
        }
    }

    proptest::proptest! {
        /// 拡大・移動・左右反転しても単語の画素範囲を保ち、小断片だけを結合する。
        #[test]
        fn fragment_context_preserves_scale_translation_and_reflection(
            scale in 1usize..9, margin in 0usize..40, reflected in proptest::bool::ANY,
        ) {
            let bounds = if reflected { [0,325,400,800] } else { [0,400,475,800] };
            let spaces = if reflected { [false,true,false] } else { [false,false,true] };
            let mut actual = (0..3).map(|i| LineChunk {
                x0: bounds[i]*scale+margin, x1: bounds[i+1]*scale+margin,
                space_before: spaces[i], gap_before: if spaces[i] {20*scale} else {0},
            }).collect::<Vec<_>>();
            let mut support = gaps(19);
            for gap in &mut support {
                gap.midpoint = (if reflected {800-gap.midpoint} else {gap.midpoint})*scale+margin;
            }
            support.sort_by_key(|g| g.midpoint);
            merge_rotated_word_fragments(&mut actual,&support,64*scale,256,24);
            proptest::prop_assert_eq!(actual.len(),2);
            proptest::prop_assert_eq!(actual[0].x0,margin);
            proptest::prop_assert_eq!(actual[1].x1,800*scale+margin);
            proptest::prop_assert_eq!(actual[0].x1,actual[1].x0);
            proptest::prop_assert_eq!(actual[0].x1,(if reflected {325} else {475})*scale+margin);
            proptest::prop_assert!(actual[1].space_before);
        }
    }
}
