use super::{
    MIN_CHROMA,
    antialias::{MAX_FRINGE, is_white_mix},
    chromatic, normalized_color_intensity,
};
use std::borrow::Cow;

/// 外側紙へ接する一行の背景境界。
#[derive(Clone, Copy)]
struct Edge {
    /// 確認済み背景の列。
    inner: usize,
    /// 外周へ接続する紙の列。
    paper: usize,
    /// この境界の背景色。
    background: [u8; 3],
}

/// 外周紙で裏付けられた長い左右境界の混合色だけを白へ戻す。
pub(super) fn remove_fringe(
    rgb: &[u8],
    width: usize,
    y0: usize,
    y1: usize,
    backgrounds: &[Option<[u8; 3]>],
    exterior: &[bool],
    output: &mut Cow<'_, [u8]>,
) {
    for left in [true, false] {
        let edges: Vec<_> = (y0..y1)
            .map(|y| {
                let is_background = |x: &usize| {
                    let i = (y * width + *x) * 3;
                    chromatic(&rgb[i..i + 3])
                        && backgrounds[*x]
                            .is_some_and(|bg| normalized_color_intensity(&rgb[i..i + 3], bg) >= 220)
                };
                // 孤立した色文字は外縁の起点にせず、広い背景一致区間を選ぶ。
                let mut run_start = 0;
                let mut selected = None;
                for x in 0..=width {
                    if x < width && is_background(&x) {
                        continue;
                    }
                    if x - run_start >= 64 {
                        selected = Some(if left { run_start } else { x - 1 });
                        if left {
                            break;
                        }
                    }
                    run_start = x + 1;
                }
                let inner = selected?;
                let background = backgrounds[inner]?;
                for distance in 1..=MAX_FRINGE + 1 {
                    let x = if left {
                        inner.checked_sub(distance)?
                    } else {
                        inner.checked_add(distance).filter(|&x| x < width)?
                    };
                    if exterior[y * width + x] {
                        return Some(Edge {
                            inner,
                            paper: x,
                            background,
                        });
                    }
                    let i = (y * width + x) * 3;
                    if !is_white_mix(&rgb[i..i + 3], background) {
                        return None;
                    }
                }
                None
            })
            .collect();
        let mut start = 0;
        for row in 0..=edges.len() {
            let connected = row < edges.len()
                && edges[row].is_some()
                && (row == start
                    || edges[row - 1].zip(edges[row]).is_some_and(|(a, b)| {
                        a.paper.abs_diff(b.paper) <= 1
                            && a.background
                                .iter()
                                .zip(b.background)
                                .all(|(&a, b)| a.abs_diff(b) < MIN_CHROMA)
                    }));
            if connected {
                continue;
            }
            if row - start >= 64 {
                for (dy, edge) in edges.iter().enumerate().take(row).skip(start) {
                    let edge = edge.expect("continuous lateral edge");
                    let columns = if left {
                        edge.paper + 1..edge.inner
                    } else {
                        edge.inner + 1..edge.paper
                    };
                    for x in columns {
                        let i = ((y0 + dy) * width + x) * 3;
                        output.to_mut()[i..i + 3].fill(255);
                    }
                }
            }
            start = if row < edges.len() && edges[row].is_some() {
                row
            } else {
                row + 1
            };
        }
    }
}
