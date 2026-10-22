use crate::infer::deim::Detection;

mod edge_bars;
mod multiline;

/// 検出選択後の認識領域を返し、境界で切れた短い横画線だけを完結させる。
pub(crate) fn line_recognition_regions<'a>(
    detections: &'a [Detection],
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Vec<(&'a Detection, [i32; 4])> {
    let selected = select_line_detections_with_pixels(detections, rgb, width, height);
    let bounds: Vec<_> = selected.iter().map(|d| d.box_xyxy).collect();
    let completed = edge_bars::recognition_bounds(rgb, width, height, &bounds);
    selected.into_iter().zip(completed).collect()
}

/// 近似同一領域の選択後、既存の強い2行に覆われる複数行候補だけを追加除外する。
pub(crate) fn select_line_detections_with_pixels<'a>(
    detections: &'a [Detection],
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Vec<&'a Detection> {
    let selected = select_line_detections(detections, width, height);
    if selected.len() < 3
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return selected;
    }
    let mut order: Vec<_> = selected.iter().copied().enumerate().collect();
    order.sort_by(|a, b| {
        b.1.confidence
            .total_cmp(&a.1.confidence)
            .then(a.0.cmp(&b.0))
    });
    let mut retained = vec![false; selected.len()];
    let mut kept = Vec::new();
    for (index, candidate) in order {
        if !multiline::covered_by_two_lines(candidate, &kept, rgb, width, height) {
            retained[index] = true;
            kept.push(candidate);
        }
    }
    selected
        .into_iter()
        .zip(retained)
        .filter_map(|(d, keep)| keep.then_some(d))
        .collect()
}

/// クラスを問わず近似同一領域の重複を除き、元順で認識行を選ぶ。
pub(crate) fn select_line_detections(
    detections: &[Detection],
    width: usize,
    height: usize,
) -> Vec<&Detection> {
    let mut candidates: Vec<_> = detections
        .iter()
        .enumerate()
        .filter(|(_, d)| d.class_name.starts_with("line_"))
        .collect();
    candidates.sort_by(|a, b| {
        b.1.confidence
            .total_cmp(&a.1.confidence)
            .then(a.0.cmp(&b.0))
    });
    let mut selected: Vec<(usize, &Detection)> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !selected.iter().any(|(_, kept)| {
            eligible(kept, width, height)
                && eligible(candidate.1, width, height)
                && nearly_same_box(kept.box_xyxy, candidate.1.box_xyxy)
        }) {
            selected.push(candidate);
        }
    }
    selected.sort_unstable_by_key(|(index, _)| *index);
    selected
        .into_iter()
        .map(|(_, detection)| detection)
        .collect()
}

/// 画像内の正常な候補だけを重複抑制の根拠にする。
fn eligible(detection: &Detection, width: usize, height: usize) -> bool {
    let [x0, y0, x1, y1] = detection.box_xyxy;
    detection.confidence.is_finite()
        && x0 >= 0
        && y0 >= 0
        && x0 < x1
        && y0 < y1
        && x1 as usize <= width
        && y1 as usize <= height
}

/// 正の矩形同士のIoUが90%以上か、i32全域でも桁あふれせず比較する。
fn nearly_same_box(a: [i32; 4], b: [i32; 4]) -> bool {
    let a = a.map(i64::from);
    let b = b.map(i64::from);
    if a[0] >= a[2] || a[1] >= a[3] || b[0] >= b[2] || b[1] >= b[3] {
        return false;
    }
    let iw = (a[2].min(b[2]) - a[0].max(b[0])).max(0) as u128;
    let ih = (a[3].min(b[3]) - a[1].max(b[1])).max(0) as u128;
    let intersection = iw * ih;
    let area_a = (a[2] - a[0]) as u128 * (a[3] - a[1]) as u128;
    let area_b = (b[2] - b[0]) as u128 * (b[3] - b[1]) as u128;
    intersection * 10 >= (area_a + area_b - intersection) * 9
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 検出の識別情報を保持する最小候補を作る。
    fn line(name: &str, bounds: [i32; 4], confidence: f32) -> Detection {
        Detection {
            class_index: 1,
            class_name: name.into(),
            box_xyxy: bounds,
            confidence,
            pred_char_count: 7.0,
        }
    }

    /// 別行・部分重複・小領域の包含・不正候補は削除しない。
    #[test]
    fn distinct_and_invalid_detections_are_retained() {
        let base = line("line_main", [0, 0, 100, 20], 0.9);
        for other in [
            line("line_title", [0, 21, 100, 41], 0.8),
            line("line_title", [0, 0, 89, 20], 0.8),
            line("line_title", [10, 5, 90, 15], 0.8),
            line("line_title", [0, 0, 100, 0], 0.8),
            line("line_title", [100, 0, 0, 20], 0.8),
            line("line_title", [0, 0, 100, 20], f32::NAN),
            line("line_title", [0, 0, 100, 20], f32::INFINITY),
        ] {
            let input = [base.clone(), other];
            assert_eq!(
                select_line_detections(&input, 2_000_000, 2_000_000).len(),
                2,
                "{input:?}"
            );
        }
    }

    /// 同点は先頭を残し、非行クラスの強い検出は本文を抑制しない。
    #[test]
    fn tied_scores_and_non_line_classes_keep_input_priority() {
        let input = [
            line("text_block", [0, 0, 100, 20], 1.0),
            line("line_title", [0, 0, 100, 20], 0.9),
            line("line_main", [0, 0, 100, 20], 0.9),
        ];
        let actual = select_line_detections(&input, 2_000_000, 2_000_000);
        assert_eq!(actual.len(), 1);
        assert!(std::ptr::eq(actual[0], &input[1]));
    }

    /// 除外済み候補を橋渡しにして、別領域まで連鎖的に消さない。
    #[test]
    fn rejected_candidate_does_not_suppress_another_line() {
        let input = [
            line("line_main", [0, 0, 100, 20], 0.9),
            line("line_title", [5, 0, 105, 20], 0.7),
            line("line_ad", [10, 0, 110, 20], 0.8),
        ];
        let actual = select_line_detections(&input, 2_000_000, 2_000_000);
        assert_eq!(actual.len(), 2);
        assert!(std::ptr::eq(actual[0], &input[0]));
        assert!(std::ptr::eq(actual[1], &input[2]));
    }

    /// IoUの境界とi32全域の矩形面積を整数で正しく比較する。
    #[test]
    fn overlap_threshold_and_extreme_coordinates_are_exact() {
        assert!(nearly_same_box([0, 0, 100, 20], [0, 0, 90, 20]));
        assert!(!nearly_same_box([0, 0, 100, 20], [0, 0, 89, 20]));
        let extreme = [i32::MIN, i32::MIN, i32::MAX, i32::MAX];
        assert!(nearly_same_box(extreme, extreme));
        assert!(!nearly_same_box(extreme, [0, 0, 1, 1]));
    }

    proptest::proptest! {
        /// 拡大・平行移動・縦横方向・順序によらず強い候補の同一性を保つ。
        #[test]
        fn winning_detection_is_invariant(
            x in 0i32..1_000_000, y in 0i32..1_000_000,
            scale in 1i32..1000, vertical in proptest::bool::ANY, reverse in proptest::bool::ANY
        ) {
            let (w, h) = if vertical { (20 * scale, 100 * scale) } else { (100 * scale, 20 * scale) };
            let bounds = [x, y, x + w, y + h];
            let mut input = [line("line_main", bounds, 0.8), line("line_title", bounds, 0.9)];
            if reverse { input.reverse(); }
            let actual = select_line_detections(&input, 2_000_000, 2_000_000);
            proptest::prop_assert_eq!(actual.len(), 1);
            proptest::prop_assert_eq!(&actual[0].class_name, "line_title");
            proptest::prop_assert_eq!(actual[0].box_xyxy, bounds);
            proptest::prop_assert_eq!(actual[0].confidence, 0.9);
            proptest::prop_assert_eq!(actual[0].pred_char_count, 7.0);
        }
    }
}
