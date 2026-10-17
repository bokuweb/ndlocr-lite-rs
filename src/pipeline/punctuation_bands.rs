//! 行の下端または右下から離れた疎な小成分を、元の帯へ回収する。

use super::component_bands::{Component, components_in_region};

/// 長い本文帯の下端を跨ぐ小成分と、右下の近接句読点を一巡で含める。
pub(super) fn restore_trailing_punctuation(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    bands: &mut [[usize; 4]],
) {
    for i in 0..bands.len() {
        let band = bands[i];
        let scale = band[3] - band[1];
        if scale < 12 || band[2] - band[0] < scale * 3 {
            continue;
        }
        let top = band[3] - scale / 2;
        let bottom = (band[3] + scale * 3 / 4)
            .min(height)
            .min(bands.get(i + 1).map_or(height, |b| b[1]));
        if bottom <= top {
            continue;
        }
        // 候補の開始位置上限に最大幅と1画素を加え、無関係な余白は走査しない。
        let left = band[0].saturating_sub(1);
        let right = (band[2] + scale + scale / 2 + 1).min(width);
        let Some(components) =
            components_in_region(rgb, width, [left, top, right, bottom], threshold)
        else {
            continue;
        };
        let supported_end = has_trailing_body(rgb, width, band, threshold);
        let mut bounds = band;
        for component in components {
            let [x0, y0, x1, y1] = component.bounds;
            if y0 <= top
                || y1 >= bottom
                || (left > 0 && x0 <= left)
                || (right < width && x1 >= right)
                || !is_sparse_mark(component, scale)
            {
                continue;
            }
            let crossing_bottom = x0 >= band[0] && x1 <= band[2] && y0 < band[3] && y1 > band[3];
            let trailing = supported_end
                && x0 >= band[2]
                && x0 - band[2] <= scale
                && y0 + y1 >= 2 * band[3] - scale * 2 / 3
                && y0 + y1 <= 2 * band[3] + scale;
            if !crossing_bottom && !trailing {
                continue;
            }
            // 判定は元の帯だけを参照し、追加成分の先へ探索を連鎖しない。
            bounds[2] = bounds[2].max((x1 + 2).min(width));
            bounds[3] = bounds[3].max((y1 + 1).min(bottom));
        }
        bands[i] = bounds;
    }
}

/// 行末の一文字高の範囲に、半分以上の高さを持つ本文の筆画があるか確認する。
fn has_trailing_body(rgb: &[u8], width: usize, band: [usize; 4], threshold: u8) -> bool {
    let scale = band[3] - band[1];
    let mut supported_rows = 0;
    for y in band[1]..band[3] {
        let ink = (band[2].saturating_sub(scale)..band[2])
            .filter(|&x| {
                let i = (y * width + x) * 3;
                rgb[i..i + 3].iter().map(|&c| u16::from(c)).sum::<u16>() < u16::from(threshold) * 3
            })
            .take(2)
            .count();
        supported_rows += usize::from(ink == 2);
    }
    supported_rows * 2 >= scale
}

/// 句読点程度の寸法と疎な密度を要求し、塗り潰し・小点・線を除く。
fn is_sparse_mark(component: Component, scale: usize) -> bool {
    let [x0, y0, x1, y1] = component.bounds;
    let (w, h) = (x1 - x0, y1 - y0);
    w >= 2
        && h >= 3
        && w * 2 <= scale
        && h * 2 <= scale
        && w <= h * 2
        && h <= w * 3
        && component.area >= 4
        && component.area * 100 >= w * h * 10
        && component.area * 100 <= w * h * 80
}
