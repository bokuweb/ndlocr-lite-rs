use super::{BlankGap, LineChunk};

/// 単語の後ろから孤立した上下の小画線を、次の実空白まで前区間へ戻す。
pub(super) fn restore_separated_marks(
    rgb: &[u8],
    size: (usize, usize),
    threshold: u8,
    chunks: &mut [LineChunk],
    gaps: &[BlankGap],
    input: (usize, usize),
) {
    let (width, height) = size;
    if height < 16 {
        return;
    }
    for index in 0..chunks.len().saturating_sub(1) {
        let left = &chunks[index];
        let right = &chunks[index + 1];
        if right.space_before {
            continue;
        }
        let first = gaps.partition_point(|gap| gap.midpoint <= right.x0);
        let Some(gap) = gaps[first..]
            .iter()
            .find(|gap| gap.width >= height.div_ceil(10))
        else {
            continue;
        };
        if gap.midpoint >= right.x1 || gap.width < height.div_ceil(10) {
            continue;
        }
        let rotated = |span: usize| height as u128 * 5 > span as u128 * 4;
        let start = gaps.partition_point(|gap| gap.midpoint <= left.x0);
        let end = gaps.partition_point(|item| item.midpoint < gap.midpoint);
        let fits =
            (gap.midpoint - left.x0) as u128 * input.1 as u128 <= height as u128 * input.0 as u128;
        if !rotated(gap.midpoint - right.x0)
            || rotated(left.x1 - left.x0)
            || rotated(right.x1 - gap.midpoint)
            || !fits
            || end - start >= super::MAX_CHUNK_INK_RUNS
            || (!gap.word_space && !supported_space_width(gaps, gap.width))
        {
            continue;
        }
        let xs = right.x0..gap.midpoint;
        let radius = height / 16;
        let middle = height / 2 - radius..height / 2 + radius + 1;
        let ink = |x: usize, y: usize| {
            rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
                .iter()
                .map(|&v| u16::from(v))
                .sum::<u16>()
                < u16::from(threshold) * 3
        };
        if middle.clone().any(|y| xs.clone().any(|x| ink(x, y)))
            || !small_edge_mark(
                rgb,
                width,
                height,
                threshold,
                xs.clone(),
                0..middle.start,
                true,
            )
            || !small_edge_mark(rgb, width, height, threshold, xs, middle.end..height, false)
        {
            continue;
        }
        chunks[index].x1 = gap.midpoint;
        chunks[index + 1].x0 = gap.midpoint;
        chunks[index + 1].space_before = true;
        chunks[index + 1].gap_before = gap.width;
    }
}

/// 三つ以上の既存単語空白がある行で、その最小実幅を下限として支持する。
fn supported_space_width(gaps: &[BlankGap], width: usize) -> bool {
    let (mut count, mut smallest) = (0, usize::MAX);
    for gap in gaps.iter().filter(|gap| gap.word_space) {
        count += 1;
        smallest = smallest.min(gap.width);
    }
    count >= 3 && width >= smallest
}

/// 上下端の小画線について、画素数と縦横の広がりを確認する。
fn small_edge_mark(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    xs: std::ops::Range<usize>,
    ys: std::ops::Range<usize>,
    upper: bool,
) -> bool {
    let (mut x0, mut x1, mut y0, mut y1, mut area) = (width, 0, height, 0, 0);
    for y in ys {
        for x in xs.clone() {
            if rgb[(y * width + x) * 3..(y * width + x + 1) * 3]
                .iter()
                .map(|&v| u16::from(v))
                .sum::<u16>()
                < u16::from(threshold) * 3
            {
                x0 = x0.min(x);
                x1 = x1.max(x + 1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
                area += 1;
            }
        }
    }
    area >= 3
        && x1 - x0 < height.div_ceil(2)
        && y1 - y0 < height.div_ceil(2)
        && if upper {
            y0 <= height / 8
        } else {
            y1 >= height - height / 4
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 単語途中で始まる前区間と、上下の小画線・確認済み空白を作る。
    fn sample() -> (Vec<u8>, Vec<LineChunk>, Vec<BlankGap>) {
        let mut rgb = vec![255; 800 * 64 * 3];
        for (xs, ys) in [
            (100..496, 5..60),
            (505..510, 2..14),
            (512..518, 45..61),
            (540..790, 5..60),
        ] {
            for y in ys {
                for x in xs.clone() {
                    rgb[(y * 800 + x) * 3..(y * 800 + x + 1) * 3].fill(0);
                }
            }
        }
        let chunks = vec![
            LineChunk {
                x0: 0,
                x1: 100,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 100,
                x1: 500,
                space_before: false,
                gap_before: 2,
            },
            LineChunk {
                x0: 500,
                x1: 800,
                space_before: false,
                gap_before: 3,
            },
        ];
        let gaps = vec![BlankGap {
            midpoint: 530,
            width: 8,
            word_space: true,
            preferred_split: false,
            internal_glyph_gap: false,
        }];
        (rgb, chunks, gaps)
    }

    /// 比較用に区間の全フィールドを複製する。
    fn copy_chunks(chunks: &[LineChunk]) -> Vec<LineChunk> {
        chunks
            .iter()
            .map(|chunk| LineChunk {
                x0: chunk.x0,
                x1: chunk.x1,
                space_before: chunk.space_before,
                gap_before: chunk.gap_before,
            })
            .collect()
    }

    /// 単語途中からの文脈でも小画線だけを実空白まで戻し、区間数を保つ。
    #[test]
    fn separated_marks_rejoin_preceding_word() {
        let (rgb, mut chunks, gaps) = sample();
        restore_separated_marks(&rgb, (800, 64), 220, &mut chunks, &gaps, (256, 24));
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[1].x1, 530);
        assert_eq!(chunks[2].x0, 530);
        assert_eq!(chunks[2].x1, 800);
        assert!(chunks[2].space_before);
        assert_eq!(chunks[2].gap_before, 8);
    }
    /// 小画線の余白として除外された間隔は、三つ以上の既存単語空白で支持する。
    #[test]
    fn separated_marks_use_supported_original_gap_width() {
        let (mut rgb, mut chunks, mut gaps) = sample();
        gaps[0].word_space = false;
        for (midpoint, gap_width) in [(184, 8), (284, 16), (384, 16)] {
            for y in 0..64 {
                rgb[(y * 800 + midpoint - gap_width / 2) * 3
                    ..(y * 800 + midpoint + gap_width / 2) * 3]
                    .fill(255);
            }
            gaps.push(BlankGap {
                midpoint,
                width: gap_width,
                word_space: true,
                preferred_split: true,
                internal_glyph_gap: false,
            });
        }
        gaps.sort_by_key(|gap| gap.midpoint);
        restore_separated_marks(&rgb, (800, 64), 220, &mut chunks, &gaps, (256, 24));
        assert_eq!(chunks[1].x1, 530);
        assert!(chunks[2].space_before);
    }

    /// 全高画線・単独記号・狭い空白やモデル上限不足では境界を維持する。
    #[test]
    fn unsupported_marks_preserve_partition() {
        for case in 0..13 {
            let (mut rgb, mut chunks, mut gaps) = sample();
            let mut input = (256, 24);
            match case {
                0 => gaps[0].word_space = false,
                1 => gaps[0].width = 4,
                2 => {
                    for y in 0..32 {
                        rgb[(y * 800 + 500) * 3..(y * 800 + 530) * 3].fill(255);
                    }
                }
                3 => {
                    for y in 32..64 {
                        rgb[(y * 800 + 500) * 3..(y * 800 + 530) * 3].fill(255);
                    }
                }
                4 => {
                    for y in 12..46 {
                        rgb[(y * 800 + 508) * 3..(y * 800 + 509) * 3].fill(0);
                    }
                }
                5 => {
                    gaps[0].midpoint = 540;
                    for x in 500..534 {
                        rgb[(3 * 800 + x) * 3..(3 * 800 + x + 1) * 3].fill(0);
                    }
                }
                6 => input.0 = 128,
                7 => chunks[2].space_before = true,
                8 => {
                    gaps.extend((1..=20).map(|i| BlankGap {
                        midpoint: 100 + i * 19,
                        width: 2,
                        word_space: false,
                        preferred_split: false,
                        internal_glyph_gap: false,
                    }));
                    gaps.sort_by_key(|gap| gap.midpoint);
                }
                9 => gaps[0].midpoint = 600,
                10 => chunks[1].x0 = 480,
                11 | 12 => {
                    gaps[0].word_space = false;
                    for midpoint in [184, 284, 384]
                        .into_iter()
                        .take(if case == 11 { 3 } else { 2 })
                    {
                        gaps.push(BlankGap {
                            midpoint,
                            width: if case == 11 { 9 } else { 8 },
                            word_space: true,
                            preferred_split: true,
                            internal_glyph_gap: false,
                        });
                    }
                    gaps.sort_by_key(|gap| gap.midpoint);
                }
                _ => unreachable!(),
            }
            let before = copy_chunks(&chunks);
            restore_separated_marks(&rgb, (800, 64), 220, &mut chunks, &gaps, input);
            assert_eq!(chunks, before, "case={case}");
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(24))]
        /// 整数拡大と左余白の移動でも、区間数・被覆・モデル比率を維持する。
        #[test]
        fn separated_mark_context_is_scale_and_translation_invariant(scale in 1usize..4, offset in 0usize..17) {
            let (source, mut chunks, mut gaps) = sample();
            let (width,height) = (800*scale+offset,64*scale);
            let mut rgb=vec![255;width*height*3];
            for y in 0..height { for x in 0..800*scale {
                let dest=(y*width+x+offset)*3;
                let src=((y/scale)*800+x/scale)*3;
                rgb[dest..dest+3].copy_from_slice(&source[src..src+3]);
            }}
            for chunk in &mut chunks {
                chunk.x0=chunk.x0*scale+offset;
                chunk.x1=chunk.x1*scale+offset;
                chunk.gap_before*=scale;
            }
            chunks[0].x0=0;
            gaps[0].midpoint=gaps[0].midpoint*scale+offset;
            gaps[0].width*=scale;
            let mut expected=copy_chunks(&chunks);
            expected[1].x1=530*scale+offset;
            expected[2].x0=530*scale+offset;
            expected[2].space_before=true;
            expected[2].gap_before=8*scale;
            restore_separated_marks(&rgb,(width,height),220,&mut chunks,&gaps,(256,24));
            proptest::prop_assert_eq!(&chunks,&expected);
            proptest::prop_assert_eq!(chunks[0].x0,0);
            proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
            proptest::prop_assert!(chunks.windows(2).all(|pair|pair[0].x1==pair[1].x0));
            proptest::prop_assert!((chunks[1].x1-chunks[1].x0)*24<=height*256);
        }
    }
}
