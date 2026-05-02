//! ページ内 line bbox を「読み順」に並び替えるユーティリティ。
//!
//! - 横書き: 上→下、同じバンド内は左→右
//! - 縦書き: 右→左、同じ列内は上→下
//! - 混在ページ: 縦書きを先、横書きを後
//!
//! 公開 API は 2 系統:
//!
//! - [`sort_lines_in_reading_order`] : `RecognizedLine` (本クレート内部の認識結果型) 用
//! - [`sort_bboxes_in_reading_order`] : 任意の `[x0, y0, x1, y1]` bbox 列用。
//!   ndlocr-lite-rs の cascade パイプラインを使わずに自前で recognize する
//!   下流クレート (例: ellisii-ocr) からも使える、低レベル API。

use crate::pipeline::connect::RecognizedLine;
use std::cmp::Ordering;

/// 縦横は `(height > width)` で判定する。`connect.rs` の RecognizedLine 構築側と
/// 完全に揃えてある。
fn bbox_is_vertical(bbox: [i32; 4]) -> bool {
    let w = bbox[2] - bbox[0];
    let h = bbox[3] - bbox[1];
    h > w
}

pub fn sort_lines_in_reading_order(lines: &mut [RecognizedLine]) {
    lines.sort_by(compare_line_reading_order);
}

/// bbox 列を読み順で並び替える。
///
/// - 各 bbox の `is_vertical` は `(y1-y0) > (x1-x0)` で内部判定する
/// - スライス長が 1 以下なら何もしない
///
/// # 例
/// ```
/// use ndlocr_lite_rs::pipeline::reading_order::sort_bboxes_in_reading_order;
/// // 3 行 × 2 列の bbox を「読み順」と逆順に並べた状態
/// let mut bs = vec![
///     [200, 100, 300, 130], // row1 right
///     [10, 10, 100, 40],    // row0 left
///     [200, 10, 300, 40],   // row0 right
///     [10, 100, 100, 130],  // row1 left
/// ];
/// sort_bboxes_in_reading_order(&mut bs);
/// assert_eq!(bs[0], [10, 10, 100, 40]);
/// assert_eq!(bs[1], [200, 10, 300, 40]);
/// assert_eq!(bs[2], [10, 100, 100, 130]);
/// assert_eq!(bs[3], [200, 100, 300, 130]);
/// ```
pub fn sort_bboxes_in_reading_order(bboxes: &mut [[i32; 4]]) {
    if bboxes.len() < 2 {
        return;
    }
    bboxes.sort_by(|a, b| compare_bbox_reading_order(*a, *b));
}

fn compare_line_reading_order(a: &RecognizedLine, b: &RecognizedLine) -> Ordering {
    compare_inner(a.bbox_xyxy, a.is_vertical, b.bbox_xyxy, b.is_vertical)
}

fn compare_bbox_reading_order(a: [i32; 4], b: [i32; 4]) -> Ordering {
    compare_inner(a, bbox_is_vertical(a), b, bbox_is_vertical(b))
}

fn compare_inner(a_box: [i32; 4], a_vert: bool, b_box: [i32; 4], b_vert: bool) -> Ordering {
    match (a_vert, b_vert) {
        (true, true) => {
            // 縦書き: 列単位で右→左、同列内は上→下
            let ax = a_box[0];
            let bx = b_box[0];
            bx.cmp(&ax).then_with(|| a_box[1].cmp(&b_box[1]))
        }
        (false, false) => {
            // 横書き: 上→下、同行内は左→右
            a_box[1]
                .cmp(&b_box[1])
                .then_with(|| a_box[0].cmp(&b_box[0]))
        }
        // 混在ページでは縦を先に出す
        (false, true) => Ordering::Greater,
        (true, false) => Ordering::Less,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x0: i32, y0: i32, x1: i32, y1: i32) -> [i32; 4] {
        [x0, y0, x1, y1]
    }

    #[test]
    fn horizontal_sorts_top_then_left() {
        let mut bs = vec![
            b(200, 100, 300, 130), // row1 right
            b(10, 10, 100, 40),    // row0 left
            b(200, 10, 300, 40),   // row0 right
            b(10, 100, 100, 130),  // row1 left
        ];
        sort_bboxes_in_reading_order(&mut bs);
        assert_eq!(
            bs,
            vec![
                b(10, 10, 100, 40),
                b(200, 10, 300, 40),
                b(10, 100, 100, 130),
                b(200, 100, 300, 130),
            ]
        );
    }

    #[test]
    fn vertical_sorts_right_to_left_then_top() {
        // 縦書き: 各列が w<h な背の高い箱、右の列から先に来る
        let mut bs = vec![
            b(10, 10, 30, 200),    // 左列 上
            b(100, 10, 120, 200),  // 右列 上
            b(10, 220, 30, 400),   // 左列 下
            b(100, 220, 120, 400), // 右列 下
        ];
        sort_bboxes_in_reading_order(&mut bs);
        assert_eq!(
            bs,
            vec![
                b(100, 10, 120, 200),  // 右列 上
                b(100, 220, 120, 400), // 右列 下
                b(10, 10, 30, 200),    // 左列 上
                b(10, 220, 30, 400),   // 左列 下
            ]
        );
    }

    #[test]
    fn mixed_orientation_puts_vertical_first() {
        // 縦書き 1 つと横書き 1 つが混在 → 縦が先
        let mut bs = vec![
            b(100, 10, 300, 30),  // 横書き
            b(400, 10, 420, 200), // 縦書き
        ];
        sort_bboxes_in_reading_order(&mut bs);
        assert_eq!(bs[0], b(400, 10, 420, 200));
        assert_eq!(bs[1], b(100, 10, 300, 30));
    }

    #[test]
    fn empty_and_single_are_no_op() {
        let mut empty: Vec<[i32; 4]> = Vec::new();
        sort_bboxes_in_reading_order(&mut empty);
        assert!(empty.is_empty());

        let mut single = vec![b(0, 0, 10, 10)];
        sort_bboxes_in_reading_order(&mut single);
        assert_eq!(single, vec![b(0, 0, 10, 10)]);
    }
}
