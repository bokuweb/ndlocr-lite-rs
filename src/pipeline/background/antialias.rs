use super::{MIN_CHROMA, chromatic, normalized_color_intensity};
use std::borrow::Cow;

/// 白紙と色背景の間に許す混合画素の最大厚み。
const MAX_FRINGE: usize = 3;

/// 白との線形混合に許す成分ごとの整数丸め誤差。
const MIX_TOLERANCE: u16 = 3;

/// 一列の外縁を支える背景・白紙の位置。
#[derive(Clone, Copy)]
struct Edge {
    /// 背景として確認した内側の行。
    inner: usize,
    /// 混合部分のすぐ外側にある白紙の行。
    paper: usize,
    /// この列で推定された背景色。
    background: [u8; 3],
}

/// パネル上下の長い外縁だけから、白文字と誤認された混合画素を除く。
pub(super) fn remove_panel_fringe(
    rgb: &[u8],
    width: usize,
    y0: usize,
    y1: usize,
    backgrounds: &[Option<[u8; 3]>],
    output: &mut Cow<'_, [u8]>,
) {
    let height = rgb.len() / 3 / width;
    let min_run = (width / 4).max(64);
    for top in [true, false] {
        let edges: Vec<_> = backgrounds
            .iter()
            .enumerate()
            .map(|(x, background)| {
                let background = (*background)?;
                let is_background = |y: &usize| {
                    let i = (*y * width + x) * 3;
                    chromatic(&rgb[i..i + 3])
                        && normalized_color_intensity(&rgb[i..i + 3], background) >= 220
                };
                let inner = if top {
                    (y0..y1).find(is_background)
                } else {
                    (y0..y1).rfind(is_background)
                }?;
                for distance in 1..=MAX_FRINGE + 1 {
                    let y = if top {
                        inner.checked_sub(distance)?
                    } else {
                        inner.checked_add(distance).filter(|&y| y < height)?
                    };
                    let i = (y * width + x) * 3;
                    let pixel = &rgb[i..i + 3];
                    if pixel == [255; 3] {
                        return Some(Edge {
                            inner,
                            paper: y,
                            background,
                        });
                    }
                    if !is_white_mix(pixel, background) {
                        return None;
                    }
                }
                None
            })
            .collect();
        let mut start = 0;
        for x in 0..=width {
            let connected = x < width
                && edges[x].is_some()
                && (x == start
                    || edges[x - 1].zip(edges[x]).is_some_and(|(left, right)| {
                        left.paper.abs_diff(right.paper) <= 1
                            && left
                                .background
                                .iter()
                                .zip(right.background)
                                .all(|(&a, b)| a.abs_diff(b) < MIN_CHROMA)
                    }));
            if !connected {
                if x - start >= min_run {
                    for (column, edge) in edges.iter().enumerate().take(x).skip(start) {
                        let edge = edge.expect("continuous outer edge");
                        // 白紙側の画素も、他列のパネル内と誤判定されていれば戻す。
                        let rows = if top {
                            edge.paper..edge.inner
                        } else {
                            edge.inner + 1..edge.paper + 1
                        };
                        for y in rows {
                            let i = (y * width + column) * 3;
                            output.to_mut()[i..i + 3].fill(255);
                        }
                    }
                }
                start = if x < width && edges[x].is_some() {
                    x
                } else {
                    x + 1
                };
            }
        }
    }
}

/// 背景色から白へ向かう同じ混合率で全成分を説明できる場合だけ認める。
fn is_white_mix(pixel: &[u8], background: [u8; 3]) -> bool {
    let channel = (0..3).min_by_key(|&c| background[c]).unwrap();
    let denominator = 255 - u16::from(background[channel]);
    let Some(numerator) = pixel[channel].checked_sub(background[channel]) else {
        return false;
    };
    (0..3).all(|c| {
        let expected = u16::from(background[c])
            + (255 - u16::from(background[c])) * u16::from(numerator) / denominator;
        u16::from(pixel[c]).abs_diff(expected) <= MIX_TOLERANCE
    })
}
