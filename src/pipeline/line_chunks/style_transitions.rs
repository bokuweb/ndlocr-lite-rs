use super::{BlankGap, LineChunk};

/// 投影で確認できた直立、または強い斜体の区分。
#[derive(Clone, Copy)]
enum Slant {
    /// 傾き候補の改善が5%未満の直立画線。
    Upright,
    /// 傾き候補が20%以上改善する斜体画線。
    Strong,
}

/// 強い斜体と直立が切り替わる実空白へ、一つだけ区間境界を追加する。
pub(super) fn split_style_transitions(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    chunks: &mut Vec<LineChunk>,
    gaps: &[BlankGap],
) {
    let original = std::mem::take(chunks);
    for chunk in original {
        let candidate = gaps
            .iter()
            .filter(|gap| {
                gap.word_space
                    && gap.width >= height.div_ceil(5)
                    && gap.midpoint.saturating_sub(chunk.x0) >= height.saturating_mul(2)
                    && chunk.x1.saturating_sub(gap.midpoint) >= height.saturating_mul(2)
            })
            .find(|gap| {
                let Some(left) =
                    classify_slant(rgb, width, height, threshold, chunk.x0, gap.midpoint)
                else {
                    return false;
                };
                let right = classify_slant(rgb, width, height, threshold, gap.midpoint, chunk.x1);
                matches!(
                    (left, right),
                    (Slant::Upright, Some(Slant::Strong)) | (Slant::Strong, Some(Slant::Upright))
                )
            });
        if let Some(gap) = candidate {
            chunks.push(LineChunk {
                x0: chunk.x0,
                x1: gap.midpoint,
                space_before: chunk.space_before,
                gap_before: chunk.gap_before,
            });
            chunks.push(LineChunk {
                x0: gap.midpoint,
                x1: chunk.x1,
                space_before: true,
                gap_before: gap.width,
            });
        } else {
            chunks.push(chunk);
        }
    }
}

/// 3画線群以上の区間を、強い斜体・直立・曖昧のいずれかへ分類する。
fn classify_slant(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    x0: usize,
    x1: usize,
) -> Option<Slant> {
    let section_width = x1.checked_sub(x0)?;
    let projected_width = section_width.checked_add((height - 1) / 4)?;
    let mut columns = vec![0usize; projected_width];
    let mut ink_pixels = Vec::new();
    for y in 0..height {
        for x in x0..x1 {
            let i = (y * width + x) * 3;
            if u16::from(rgb[i]) + u16::from(rgb[i + 1]) + u16::from(rgb[i + 2])
                < u16::from(threshold) * 3
            {
                ink_pixels.push((x - x0, y));
                columns[x - x0] += 1;
            }
        }
    }
    let mut runs = 0;
    let mut previous = false;
    for &count in &columns {
        let ink = count > 0;
        runs += usize::from(ink && !previous);
        previous = ink;
    }
    if runs < 3 {
        return None;
    }
    let mut scores = Vec::with_capacity(9);
    scores.push((
        0,
        columns
            .iter()
            .map(|&count| (count as u128).pow(2))
            .sum::<u128>(),
    ));
    for slope in [-8i32, -6, -4, -2, 2, 4, 6, 8] {
        columns.fill(0);
        for &(x, y) in &ink_pixels {
            let row = if slope < 0 { height - 1 - y } else { y };
            let offset = row * slope.unsigned_abs() as usize / 32;
            columns[x + offset] += 1;
        }
        scores.push((
            slope,
            columns
                .iter()
                .map(|&count| (count as u128).pow(2))
                .sum::<u128>(),
        ));
    }
    let baseline = scores[0].1;
    if baseline == 0 {
        return None;
    }
    let &(_, best) = scores.iter().max_by_key(|(_, score)| score)?;
    if best - baseline >= baseline.div_ceil(5) {
        Some(Slant::Strong)
    } else if best - baseline < baseline.div_ceil(20) {
        Some(Slant::Upright)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::split_line_for_input;
    use super::*;

    /// 原寸の混在書式でも、斜体と直立の間の実空白を安全な追加境界にする。
    #[test]
    fn italic_to_upright_word_boundary_is_retained() {
        use crate::{
            io::load_rgb_u8,
            pipeline::{
                background::normalize_chromatic_panels,
                crop::{BBox, crop_rgb_u8},
                line_segment::detect_textline_bands_naive,
            },
        };
        let image = load_rgb_u8(std::path::Path::new(
            "tests/fixtures/chromatic_text_contrast/page.png",
        ))
        .unwrap();
        let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
        let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
        let [x0, y0, x1, y1] = bands[8];
        let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks =
            split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].x1, 457);
        assert!(chunks[1].space_before);
    }

    /// 斜体と直立の4画線を、十分広い実空白の両側に置く。
    fn words(
        scale: usize,
        margin: usize,
        reflected: bool,
        slanted: [bool; 2],
    ) -> (Vec<u8>, usize, usize, Vec<LineChunk>, Vec<BlankGap>) {
        let height = 32 * scale;
        let width = 208 * scale + margin * 2;
        let mut rgb = vec![255; width * height * 3];
        for (word, &italic) in slanted.iter().enumerate() {
            for y in 3 * scale..29 * scale {
                for stroke in 0..4 {
                    let start = (6 + 116 * word + 20 * stroke) * scale
                        + margin
                        + if italic { (height - 1 - y) / 4 } else { 0 };
                    for x in start..start + 3 * scale {
                        let x = if reflected { width - 1 - x } else { x };
                        rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    }
                }
            }
        }
        let chunks = vec![LineChunk {
            x0: 0,
            x1: width,
            space_before: false,
            gap_before: 0,
        }];
        let gaps = vec![BlankGap {
            midpoint: 104 * scale + margin,
            width: 16 * scale,
            word_space: true,
            preferred_split: true,
            internal_glyph_gap: false,
        }];
        (rgb, width, height, chunks, gaps)
    }

    /// 文体の切替を実空白で分け、既存の全画素被覆を保つ。
    #[test]
    fn distinct_slant_adds_one_word_boundary() {
        let (rgb, w, h, mut chunks, gaps) = words(1, 0, false, [true, false]);
        split_style_transitions(&rgb, w, h, 220, &mut chunks, &gaps);
        assert_eq!(
            chunks,
            vec![
                LineChunk {
                    x0: 0,
                    x1: 104,
                    space_before: false,
                    gap_before: 0
                },
                LineChunk {
                    x0: 104,
                    x1: w,
                    space_before: true,
                    gap_before: 16
                }
            ]
        );
    }

    /// 一様な文体・狭い字間・単語でない空白は分割しない。
    #[test]
    fn uniform_or_unsupported_spaces_keep_context() {
        for style in [[false, false], [true, true]] {
            let (rgb, w, h, mut chunks, gaps) = words(1, 0, false, style);
            split_style_transitions(&rgb, w, h, 220, &mut chunks, &gaps);
            assert_eq!(chunks.len(), 1);
        }
        for guard in 0..3 {
            let (rgb, w, h, mut chunks, mut gaps) = words(1, 0, false, [true, false]);
            match guard {
                0 => gaps[0].width = 6,
                1 => gaps[0].word_space = false,
                _ => gaps[0].midpoint = 40,
            };
            split_style_transitions(&rgb, w, h, 220, &mut chunks, &gaps);
            assert_eq!(chunks.len(), 1, "guard={guard}");
        }
    }

    /// 直立から斜体への切替も同じ条件で分け、孤立した少数画線は無視する。
    #[test]
    fn reverse_transition_requires_multiple_ink_runs() {
        let (mut rgb, w, h, mut chunks, gaps) = words(1, 0, false, [false, true]);
        split_style_transitions(&rgb, w, h, 220, &mut chunks, &gaps);
        assert_eq!(chunks.len(), 2);
        for y in 0..h {
            rgb[y * w * 3..(y * w + 50) * 3].fill(255);
        }
        let mut chunks = vec![LineChunk {
            x0: 0,
            x1: w,
            space_before: false,
            gap_before: 0,
        }];
        split_style_transitions(&rgb, w, h, 220, &mut chunks, &gaps);
        assert_eq!(chunks.len(), 1);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(24))]
        /// 拡大・左右移動・反転でも被覆と実空白を維持する。
        #[test]
        fn transition_preserves_geometry(scale in 1usize..4,margin in 0usize..20,reflected in proptest::bool::ANY) {
            let (rgb,w,h,mut chunks,gaps)=words(scale,margin,reflected,[true,false]);
            split_style_transitions(&rgb,w,h,220,&mut chunks,&gaps);
            proptest::prop_assert_eq!(chunks.len(),2);
            proptest::prop_assert_eq!((chunks[0].x0,chunks[0].x1,chunks[1].x0,chunks[1].x1),(0,gaps[0].midpoint,gaps[0].midpoint,w));
            proptest::prop_assert!(chunks[1].space_before);
            proptest::prop_assert_eq!(chunks[1].gap_before,gaps[0].width);
        }
    }
}
