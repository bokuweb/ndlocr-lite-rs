//! タイル分割の行検出の統合（`docs/tiled_detection.md`）。枠の幾何だけで決める。

use crate::infer::deim::Detection;

/// ページ（`width`×`height`）を `n`×`n` に分け、隣と重ねたタイルの枠 `[x0, y0, x1, y1]`（x1・y1 は含まない）。
pub fn tile_rects(width: usize, height: usize, n: usize) -> Vec<[usize; 4]> {
    if n <= 1 || width == 0 || height == 0 {
        return vec![[0, 0, width, height]];
    }
    let span = |len: usize, k: usize| {
        let step = len as f64 / n as f64;
        let pad = step / 8.0;
        let a = ((k as f64 * step) - pad).max(0.0) as usize;
        let b = (((k + 1) as f64 * step) + pad).min(len as f64) as usize;
        (a, b)
    };
    let mut out = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let (x0, x1) = span(width, i);
            let (y0, y1) = span(height, j);
            out.push([x0, y0, x1, y1]);
        }
    }
    out
}

fn area(b: &[i32; 4]) -> f64 {
    f64::from((b[2] - b[0]).max(0)) * f64::from((b[3] - b[1]).max(0))
}

fn intersection(a: &[i32; 4], b: &[i32; 4]) -> f64 {
    let w = (a[2].min(b[2]) - a[0].max(b[0])).max(0);
    let h = (a[3].min(b[3]) - a[1].max(b[1])).max(0);
    f64::from(w) * f64::from(h)
}

/// ページ全体の検出 `full` と、タイルごとの検出（ページの座標に戻したもの）とそのタイルの枠 `tiles` を統合する。
pub fn merge_tiled_detections(
    full: Vec<Detection>,
    tiles: Vec<(Detection, [usize; 4])>,
    width: usize,
    height: usize,
) -> Vec<Detection> {
    // 1. ページの縁でないタイルの縁に接する検出は、タイルで切れた行。
    let touches_inner_edge = |d: &Detection, t: &[usize; 4]| {
        let b = d.box_xyxy;
        let tol = 2;
        (t[0] > 0 && b[0] <= t[0] as i32 + tol)
            || (t[1] > 0 && b[1] <= t[1] as i32 + tol)
            || (t[2] < width && b[2] >= t[2] as i32 - tol)
            || (t[3] < height && b[3] >= t[3] as i32 - tol)
    };
    let kept: Vec<Detection> = tiles.into_iter().filter(|(d, t)| !touches_inner_edge(d, t)).map(|(d, _)| d).collect();
    // 2. 複数の行をつないだページ全体の枠を捨てる。中にあるタイルの検出のうち、互いに重ならないもの（別の行）が 2 つ以上あるとき
    // だけ。同じ行の断片（「公示送」「送達」「送」）は重なり合うので数えない。
    let inside = |f: &Detection, t: &Detection| intersection(&f.box_xyxy, &t.box_xyxy) >= 0.8 * area(&t.box_xyxy) && area(&t.box_xyxy) < area(&f.box_xyxy);
    let joins_lines = |f: &Detection| {
        let parts: Vec<&Detection> = kept.iter().filter(|t| inside(f, t)).collect();
        parts.iter().enumerate().any(|(i, a)| parts[i + 1..].iter().any(|b| intersection(&a.box_xyxy, &b.box_xyxy) == 0.0))
    };
    let full: Vec<Detection> = full.into_iter().filter(|f| !joins_lines(f)).collect();
    // 3. 残ったページ全体の枠の中にほぼ収まるタイルの検出は、その行の断片なので捨てる。
    let kept: Vec<Detection> = kept.into_iter().filter(|t| !full.iter().any(|f| inside(f, t))).collect();
    // 4. 重なる組は確信度の高い方を残す。
    let mut all: Vec<Detection> = full.into_iter().chain(kept).collect();
    all.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    let mut out: Vec<Detection> = Vec::with_capacity(all.len());
    for d in all {
        let dup = out.iter().any(|o| {
            let i = intersection(&o.box_xyxy, &d.box_xyxy);
            let u = area(&o.box_xyxy) + area(&d.box_xyxy) - i;
            u > 0.0 && i / u > 0.5
        });
        if !dup {
            out.push(d);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det(b: [i32; 4], c: f32) -> Detection {
        Detection { class_index: 0, confidence: c, box_xyxy: b, pred_char_count: 0.0, class_name: "line_main".into() }
    }

    #[test]
    fn one_tile_is_the_whole_page() {
        assert_eq!(tile_rects(300, 200, 1), vec![[0, 0, 300, 200]]);
    }

    #[test]
    fn tiles_overlap_and_cover_the_page() {
        let t = tile_rects(800, 800, 2);
        assert_eq!(t.len(), 4);
        assert_eq!(t[0], [0, 0, 450, 450]);
        assert_eq!(t[3], [350, 350, 800, 800]);
    }

    #[test]
    fn a_full_page_box_joining_two_lines_is_replaced_by_the_tile_lines() {
        let full = vec![det([0, 100, 700, 130], 0.9)];
        let tiles = vec![(det([10, 100, 300, 130], 0.8), [0, 0, 450, 450]), (det([380, 100, 440, 130], 0.8), [0, 0, 450, 450])];
        let out = merge_tiled_detections(full, tiles, 800, 800);
        let mut boxes: Vec<[i32; 4]> = out.iter().map(|d| d.box_xyxy).collect();
        boxes.sort();
        assert_eq!(boxes, vec![[10, 100, 300, 130], [380, 100, 440, 130]]);
    }

    #[test]
    fn tile_lines_cut_at_an_inner_tile_edge_are_dropped_and_the_full_line_kept() {
        let full = vec![det([100, 200, 700, 230], 0.9)];
        let tiles = vec![(det([100, 200, 450, 230], 0.95), [0, 0, 450, 450])];
        let out = merge_tiled_detections(full, tiles, 800, 800);
        assert_eq!(out.iter().map(|d| d.box_xyxy).collect::<Vec<_>>(), vec![[100, 200, 700, 230]]);
    }

    #[test]
    fn overlapping_fragments_of_one_line_keep_the_full_page_box() {
        let full = vec![det([2297, 441, 2781, 479], 0.9)];
        let t = [1645, 0, 2741, 800];
        let tiles = vec![(det([2297, 441, 2631, 474], 0.8), t), (det([2595, 441, 2700, 475], 0.8), t), (det([2595, 441, 2631, 474], 0.7), t)];
        let out = merge_tiled_detections(full, tiles, 3509, 2480);
        assert_eq!(out.iter().map(|d| d.box_xyxy).collect::<Vec<_>>(), vec![[2297, 441, 2781, 479]]);
    }

    #[test]
    fn a_tile_box_inside_a_full_page_box_of_the_same_line_keeps_the_full_page_box() {
        let full = vec![det([10, 10, 200, 40], 0.7)];
        let tiles = vec![(det([12, 11, 199, 40], 0.9), [0, 0, 450, 450])];
        let out = merge_tiled_detections(full, tiles, 800, 800);
        assert_eq!(out.iter().map(|d| d.box_xyxy).collect::<Vec<_>>(), vec![[10, 10, 200, 40]]);
    }

    #[test]
    fn overlapping_boxes_not_nested_keep_the_more_confident_one() {
        let full = vec![det([10, 10, 200, 40], 0.7)];
        let tiles = vec![(det([20, 12, 230, 42], 0.9), [0, 0, 450, 450])];
        let out = merge_tiled_detections(full, tiles, 800, 800);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].confidence, 0.9);
    }
}
