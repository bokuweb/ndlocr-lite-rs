use super::{BlankGap, blank_gaps};

/// 元画像の白列へ、強い傾き補正でも確認できる単語空白だけを加える。
pub(super) fn restore_slanted_word_spaces(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    gaps: &mut [BlankGap],
) {
    if width as u128 <= height as u128 * 8 || gaps.iter().all(|gap| gap.word_space) {
        return;
    }
    let mask: Vec<_> = rgb
        .chunks_exact(3)
        .map(|pixel| {
            u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])
                < u16::from(threshold) * 3
        })
        .collect();
    let Some(projected_width) = width.checked_add((height - 1) / 4) else {
        return;
    };
    let mut counts = vec![0usize; projected_width];
    let baseline = projection_score(&mask, width, height, 0, &mut counts);
    let mut best = baseline;
    let mut slope = 0;
    for candidate in [-8, -6, -4, -2, 2, 4, 6, 8] {
        let score = projection_score(&mask, width, height, candidate, &mut counts);
        if score > best {
            best = score;
            slope = candidate;
        }
    }
    if slope == 0 || best - baseline < baseline.div_ceil(5) {
        return;
    }
    let Some(size) = projected_width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(3))
    else {
        return;
    };
    let mut projected = vec![255; size];
    for (y, row) in mask.chunks_exact(width).enumerate() {
        let offset = row_shift(y, height, slope);
        for (x, &ink) in row.iter().enumerate() {
            if ink {
                let index = (y * projected_width + x + offset) * 3;
                projected[index..index + 3].fill(0);
            }
        }
    }
    let confirmed = blank_gaps(&projected, projected_width, height, threshold);
    let center_shift = row_shift(height / 2, height, slope);
    for gap in gaps {
        let center = gap.midpoint + center_shift;
        gap.word_space |= confirmed.iter().any(|candidate| {
            let start = candidate.midpoint - candidate.width / 2;
            candidate.word_space && (start..start + candidate.width).contains(&center)
        });
    }
}

/// 32分率の傾きを、画像外へ出ない非負の行オフセットへ変換する。
fn row_shift(y: usize, height: usize, slope: i32) -> usize {
    let row = if slope < 0 { height - 1 - y } else { y };
    row * slope.unsigned_abs() as usize / 32
}

/// 列ごとのインク数の二乗和により、画線が縦に揃う傾きを比較する。
fn projection_score(
    mask: &[bool],
    width: usize,
    height: usize,
    slope: i32,
    counts: &mut [usize],
) -> u128 {
    counts.fill(0);
    for (y, row) in mask.chunks_exact(width).enumerate() {
        let offset = row_shift(y, height, slope);
        for (x, &ink) in row.iter().enumerate() {
            counts[x + offset] += usize::from(ink);
        }
    }
    counts.iter().map(|&count| (count as u128).pow(2)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::line_chunks::blank_gaps;

    /// 字間は連結して見えるが、実単語間の白列が残る斜体行を作る。
    fn slanted_words(
        scale: usize,
        margin: usize,
        reflected: bool,
        slanted: bool,
    ) -> (Vec<u8>, usize, usize) {
        let height = 48 * scale;
        let width = 6 * (4 * 26 + 24) * scale + 2 * margin + 12 * scale;
        let mut rgb = vec![255; width * height * 3];
        for word in 0..6 {
            for letter in 0..4 {
                for y in 0..height {
                    let shift = if slanted { (height - 1 - y) / 4 } else { 0 };
                    let x0 = margin + (word * (4 * 26 + 24) + letter * 26) * scale + shift;
                    for x in x0..x0 + 18 * scale {
                        let x = if reflected { width - 1 - x } else { x };
                        rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    }
                }
            }
        }
        (rgb, width, height)
    }

    /// 重なった字間を空白にせず、元のモデル入力境界に残る実単語空白を復元する。
    #[test]
    fn slanted_projection_recovers_only_supported_word_spaces() {
        for reflected in [false, true] {
            let (rgb, width, height) = slanted_words(1, 7, reflected, true);
            let mut gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(gaps.len(), 5);
            assert!(gaps.iter().all(|g| !g.word_space));
            restore_slanted_word_spaces(&rgb, width, height, 220, &mut gaps);
            assert!(gaps.iter().all(|g| g.word_space), "reflection={reflected}");
        }
    }

    /// 直立した字形は従来の判定を保ち、新しい空白を増やさない。
    #[test]
    fn upright_projection_preserves_existing_spaces() {
        let (rgb, width, height) = slanted_words(1, 11, false, false);
        let mut gaps = blank_gaps(&rgb, width, height, 220);
        let before: Vec<_> = gaps.iter().map(|g| g.word_space).collect();
        restore_slanted_word_spaces(&rgb, width, height, 220, &mut gaps);
        assert_eq!(
            before,
            gaps.iter().map(|g| g.word_space).collect::<Vec<_>>()
        );
    }

    proptest::proptest! {
        /// 拡大・移動・反転しても実空白を戻し、分割位置と字形保護属性を維持する。
        #[test]
        fn slanted_word_spaces_preserve_geometry(
            scale in 1usize..5, margin in 0usize..32, reflected in proptest::bool::ANY,
        ) {
            let (rgb,width,height)=slanted_words(scale,margin,reflected,true);
            let mut gaps=blank_gaps(&rgb,width,height,220);
            let before: Vec<_> = gaps.iter().map(|g|
                (g.midpoint,g.width,g.preferred_split,g.internal_glyph_gap)
            ).collect();
            proptest::prop_assert_eq!(gaps.len(),5);
            restore_slanted_word_spaces(&rgb,width,height,220,&mut gaps);
            proptest::prop_assert!(gaps.iter().all(|g|g.word_space));
            proptest::prop_assert_eq!(before,gaps.iter().map(|g|
                (g.midpoint,g.width,g.preferred_split,g.internal_glyph_gap)
            ).collect::<Vec<_>>());
        }
    }
}
