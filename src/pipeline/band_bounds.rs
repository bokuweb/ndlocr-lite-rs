//! 行端の細い字形を、近傍の元画素から回収する。

/// 列のインク密度で除外された横長で細い行端字形を、行高以内の近傍から回収する。
/// 入力画像とbandは行検出で検証済み。字形かノイズかの判定は形状による推定。
pub(super) fn restore_edge_ink(
    rgb: &[u8],
    width: usize,
    band: [usize; 4],
    threshold: u8,
) -> [usize; 4] {
    let [mut x0, y0, mut x1, y1] = band;
    let line_height = y1 - y0;
    let has_ink = |x: usize| {
        (y0..y1).any(|y| {
            let i = (y * width + x) * 3;
            (u16::from(rgb[i]) + u16::from(rgb[i + 1]) + u16::from(rgb[i + 2])) / 3
                < u16::from(threshold)
        })
    };
    // 本文字形の細い部品は飛び越え、横棒状の独立した列群だけを対象にする。
    for (start, end) in [
        (x0.saturating_sub(line_height), x0),
        (x1, x1.saturating_add(line_height).min(width)),
    ] {
        let mut x = start;
        while x < end {
            if !has_ink(x) {
                x += 1;
                continue;
            }
            let mut left = x;
            while left > 0 && has_ink(left - 1) {
                left -= 1;
            }
            let mut right = x + 1;
            while right < width && has_ink(right) {
                right += 1;
            }
            x = right;
            if left < band[2] && right > band[0] {
                continue;
            }
            let span_width = right - left;
            if span_width < line_height.div_ceil(3) || span_width > line_height.saturating_mul(2) {
                continue;
            }
            let ink_rows = (y0..y1).filter(|&y| {
                (left..right).any(|x| {
                    let i = (y * width + x) * 3;
                    (u16::from(rgb[i]) + u16::from(rgb[i + 1]) + u16::from(rgb[i + 2])) / 3
                        < u16::from(threshold)
                })
            });
            let mut rows = ink_rows;
            if let Some(first) = rows.next() {
                let last = rows.next_back().unwrap_or(first);
                if last - first < line_height.div_ceil(4) {
                    x0 = x0.min(left.saturating_sub(2));
                    x1 = x1.max(right.saturating_add(2).min(width));
                }
            }
        }
    }
    [x0, y0, x1, y1]
}

#[cfg(test)]
mod tests {
    use super::super::line_segment::detect_textline_bands_naive;
    use super::*;
    use proptest::prelude::*;

    /// 横棒と十分な高さを持つ文字を配置した独立した合成画像を作る。
    fn image(marker_x: usize) -> Vec<u8> {
        let mut rgb = vec![255; 180 * 32 * 3];
        for y in 5..27 {
            for x in 70..120 {
                rgb[(y * 180 + x) * 3..(y * 180 + x) * 3 + 3].fill(0);
            }
        }
        for x in marker_x..marker_x + 18 {
            rgb[(16 * 180 + x) * 3..(16 * 180 + x) * 3 + 3].fill(0);
        }
        rgb
    }

    /// 横棒の各列が1画素でも、隣接文字と同じ行にある一を切り落とさない。
    #[test]
    fn restores_thin_leading_character_discarded_by_upstream() {
        let mut rgb = image(38);
        // 刈込端の直前に本文字形の細い部品があっても、その外側の一を探す。
        for x in 64..70 {
            rgb[(26 * 180 + x) * 3..(26 * 180 + x) * 3 + 3].fill(0);
        }
        let bands = detect_textline_bands_naive(&rgb, 180, 32, 220);
        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0], [36, 4, 122, 28]);
        let restored = restore_edge_ink(&rgb, 180, bands[0], 220);
        assert_eq!(restored, [36, 4, 122, 28]);
    }

    /// 行高を超える余白の外にある孤立ノイズへは範囲を広げない。
    #[test]
    fn does_not_capture_distant_margin_noise() {
        let rgb = image(2);
        let band = detect_textline_bands_naive(&rgb, 180, 32, 220)[0];
        assert_eq!(restore_edge_ink(&rgb, 180, band, 220), band);
    }

    /// 右端の細い字形も対称に回収する。
    #[test]
    fn restores_thin_trailing_character() {
        let rgb = image(38);
        let mirrored: Vec<u8> = rgb
            .chunks_exact(180 * 3)
            .flat_map(|row| row.chunks_exact(3).rev().flatten().copied())
            .collect();
        let band = detect_textline_bands_naive(&mirrored, 180, 32, 220)[0];
        assert_eq!(
            restore_edge_ink(&mirrored, 180, band, 220),
            [58, 4, 144, 28]
        );
    }

    proptest! {
        /// 任意画素でも元の範囲を縮めず、画像内の水平範囲だけを変更する。
        #[test]
        fn expanded_bounds_stay_inside_image(pixels in prop::collection::vec(any::<u8>(), 40*32*3)) {
            let band = [12, 3, 28, 29];
            let [x0,y0,x1,y1] = restore_edge_ink(&pixels, 40, band, 220);
            prop_assert!(x0 <= band[0]);
            prop_assert!(x1 >= band[2] && x1 <= 40);
            prop_assert_eq!([y0,y1], [band[1],band[3]]);
        }
    }
}
