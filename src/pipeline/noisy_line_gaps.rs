use super::component_bands::components_in_region;

/// 通常行高を示す複数行があるときだけ、結合した高い領域の非連結な谷を分ける。
pub(super) fn separate_noisy_line_gaps(
    rgb: &[u8],
    width: usize,
    row_ink: &[usize],
    threshold: u8,
    spans: Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut heights: Vec<_> = spans.iter().map(|&(start, end)| end - start).collect();
    heights.sort_unstable();
    let Some(&scale) = heights.get(heights.len() / 4) else {
        return spans;
    };
    if scale < 12
        || heights
            .iter()
            .filter(|&&h| h >= scale / 2 && h <= scale * 3 / 2)
            .count()
            < 2
    {
        return spans;
    }
    let mut result = Vec::new();
    for (start, end) in spans {
        if end - start < scale * 2 {
            result.push((start, end));
            continue;
        }
        let peak = row_ink[start..end].iter().copied().max().unwrap_or(0);
        let density_limit = (peak / 20).max(8);
        let mut previous = start;
        let mut y = start;
        while y < end {
            if row_ink[y] > density_limit {
                y += 1;
                continue;
            }
            let valley_start = y;
            while y < end && row_ink[y] <= density_limit {
                y += 1;
            }
            // 狭い行間も候補に含め、上下の本文高と連結成分で字形の分断を防ぐ。
            let middle = valley_start + (y - valley_start) / 2;
            let cut = (valley_start..y)
                .min_by_key(|&row| (row_ink[row], row.abs_diff(middle)))
                .unwrap();
            if cut - previous < scale / 2 || end - cut < scale / 2 {
                continue;
            }
            let minimum_ink_height = scale - scale / 4;
            if !has_body_height(&row_ink[previous..cut], density_limit, minimum_ink_height)
                || !has_body_height(&row_ink[cut..end], density_limit, minimum_ink_height)
            {
                continue;
            }
            let top = cut.saturating_sub(scale / 4).max(start);
            let bottom = (cut + scale / 4).min(end);
            let Some(components) =
                components_in_region(rgb, width, [0, top, width, bottom], threshold)
            else {
                continue;
            };
            let crossing: Vec<_> = components
                .iter()
                .filter(|c| c.bounds[1] == top && c.bounds[3] == bottom)
                .collect();
            if !crossing.is_empty() {
                // 細い行間で少数の隣接文字が接触していても、行全体の大半が離れていれば分ける。
                let narrow_contact = y - valley_start < 3
                    && peak >= scale * 8
                    && crossing
                        .iter()
                        .all(|c| c.bounds[2] - c.bounds[0] <= scale * 2)
                    && crossing
                        .iter()
                        .map(|c| c.bounds[2] - c.bounds[0])
                        .sum::<usize>()
                        <= peak / 10;
                if !narrow_contact {
                    continue;
                }
            }
            result.push((previous, cut));
            previous = cut;
        }
        result.push((previous, end));
    }
    result
}

/// 疎な汚れや空白を含めず、本文相当の連続したインク高があるかを調べる。
fn has_body_height(rows: &[usize], density_limit: usize, minimum_height: usize) -> bool {
    let mut run = 0;
    for &ink in rows {
        if ink > density_limit {
            run += 1;
            if run >= minimum_height {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::separate_noisy_line_gaps;

    /// 局所成分数の上限を超えた候補は、安全と判断せず元領域を保つ。
    #[test]
    fn component_limit_preserves_region() {
        let width = 2000;
        let mut rgb = vec![255u8; width * 200 * 3];
        let mut row_ink = vec![0usize; 200];
        let spans = vec![(0, 30), (50, 80), (100, 200)];
        for (y, count) in row_ink.iter_mut().enumerate() {
            if y < 30 || (50..80).contains(&y) || (100..130).contains(&y) || y >= 170 {
                rgb[y * width * 3..(y + 1) * width * 3].fill(0);
                *count = width;
            } else if (130..170).contains(&y) {
                for i in 0..20 {
                    let x = i * 20 + y % 2 * 10;
                    rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    *count += 1;
                }
            }
        }
        assert_eq!(
            separate_noisy_line_gaps(&rgb, width, &row_ink, 220, spans.clone()),
            spans
        );
    }

    /// 実資料の太字見出し2行を独立した行として残し、両実装で一致させる。
    #[test]
    fn tightly_spaced_jma_titles_are_separate_bands() {
        use crate::pipeline::line_segment::{
            detect_textline_bands_fast, detect_textline_bands_naive,
        };
        let image = image::load_from_memory(include_bytes!(
            "../../tests/fixtures/line_segment/jma-notice-body.png"
        ))
        .unwrap()
        .to_rgb8();
        let (w, h) = (image.width() as usize, image.height() as usize);
        let bands = detect_textline_bands_naive(image.as_raw(), w, h, 220);
        assert_eq!(bands, detect_textline_bands_fast(image.as_raw(), w, h, 220));
        let first: Vec<_> = bands.iter().filter(|b| b[1] <= 162 && 162 < b[3]).collect();
        let second: Vec<_> = bands.iter().filter(|b| b[1] <= 190 && 190 < b[3]).collect();
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert!(first[0][3] <= 190, "second title merged: {:?}", first[0]);
        assert!(second[0][1] > 162, "first title merged: {:?}", second[0]);
    }

    /// 投影が同じでも、谷の縦画で接続する字形を分断しない。
    fn narrow_gap_case(scale: usize, connected: bool, offset: usize) -> Vec<(usize, usize)> {
        let width = 240;
        let height = 6 * scale + 1;
        let mut rgb = vec![255; width * height * 3];
        let mut rows = vec![0; height];
        let start = 4 * scale;
        for (y, count) in rows.iter_mut().enumerate() {
            let body = y < scale
                || (2 * scale..3 * scale).contains(&y)
                || (start..start + scale).contains(&y)
                || y > start + scale;
            let range = if body {
                20..220
            } else if y == start + scale {
                if connected {
                    20 + offset..28 + offset
                } else {
                    0..8
                }
            } else {
                0..0
            };
            for x in range {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                *count += 1;
            }
        }
        separate_noisy_line_gaps(
            &rgb,
            width,
            &rows,
            220,
            vec![(0, scale), (2 * scale, 3 * scale), (start, height)],
        )
    }

    proptest::proptest! {
        /// 薄い孤立汚れのある1px行間だけを分け、連結した縦画は全高を保つ。
        #[test]
        fn single_row_valley_preserves_connected_glyphs(scale in 12usize..40, offset in 0usize..180) {
            let start = 4 * scale;
            let end = 6 * scale + 1;
            proptest::prop_assert_eq!(narrow_gap_case(scale, false, offset),
                vec![(0, scale), (2 * scale, 3 * scale), (start, start + scale), (start + scale, end)]);
            proptest::prop_assert_eq!(narrow_gap_case(scale, true, offset),
                vec![(0, scale), (2 * scale, 3 * scale), (start, end)]);
        }
    }
}
