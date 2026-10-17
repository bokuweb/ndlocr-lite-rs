/// 任意採点に渡す追加領域の資源上限。ゼロ件/ゼロ画素は候補生成を無効にする。
#[derive(Clone, Copy, Debug)]
pub struct MergeBudget {
    /// 検証前に受理する元の帯数の上限。
    pub max_source_bands: usize,
    /// 出力する追加候補数の上限。
    pub max_candidates: usize,
    /// 出力する和集合矩形の面積合計の上限（画素数）。
    pub max_total_pixels: usize,
}

/// 隣接2帯の原領域を置換せずに提示する矩形候補。
#[derive(Debug, PartialEq, Eq)]
pub struct LineMergeCandidate {
    /// 入力スライス中の隣接2帯の添字。別候補と共有する場合がある。
    pub source_indices: [usize; 2],
    /// 2帯を完全に含む半開矩形 `[x0, y0, x1, y1]`。
    pub bounds: [usize; 4],
}

/// 元の帯とは別に採点する候補と、予算による未列挙の情報。
#[derive(Debug, PartialEq, Eq)]
pub struct LineMergeHypotheses {
    /// 元の帯順に並ぶ追加候補。同時採用可能とは限らず、結合採否は決めない。
    pub candidates: Vec<LineMergeCandidate>,
    /// 候補矩形の面積合計。認識時のchunk数や推論メモリの上限ではない。
    pub total_pixels: usize,
    /// 件数または画素予算により出力しなかった隣接ペア数。
    pub omitted_pairs: usize,
}

/// 上から並ぶ非重複の横書き帯から、隣接2帯の和集合を予算内で列挙する。
///
/// 元領域・検出・認識結果を変更せず、画像の読取りや推論もしない。
/// 候補があることは、結合する根拠を意味しない。原読を保持して別途採否を決めること。
/// 入力帯数超過、無効寸法、空/画像外/逆順/上下重複の帯、確保失敗はエラー。
/// 時間は元の帯数、追加メモリは出力数に比例する。
///
/// ```
/// use ndlocr_lite_rs::pipeline::line_hypotheses::{MergeBudget, adjacent_line_merge_hypotheses};
/// let original = [[0, 0, 20, 5], [0, 10, 20, 15], [0, 20, 20, 25]];
/// let budget = MergeBudget { max_source_bands: 10, max_candidates: 1, max_total_pixels: 300 };
/// let alternatives = adjacent_line_merge_hypotheses(20, 25, &original, budget)?;
/// assert_eq!(original.len(), 3);
/// assert_eq!(alternatives.candidates[0].source_indices, [0, 1]);
/// assert_eq!(alternatives.omitted_pairs, 1);
/// # Ok::<(), String>(())
/// ```
pub fn adjacent_line_merge_hypotheses(
    width: usize,
    height: usize,
    bands: &[[usize; 4]],
    budget: MergeBudget,
) -> Result<LineMergeHypotheses, String> {
    if bands.len() > budget.max_source_bands {
        return Err("source band budget exceeded".into());
    }
    if width == 0 || height == 0 || width.checked_mul(height).is_none() {
        return Err("invalid page dimensions".into());
    }
    let mut previous_end = 0;
    for &[x0, y0, x1, y1] in bands {
        if x0 >= x1 || y0 >= y1 || x1 > width || y1 > height || y0 < previous_end {
            return Err("invalid or unordered horizontal band".into());
        }
        previous_end = y1;
    }
    let mut result = LineMergeHypotheses {
        candidates: Vec::new(),
        total_pixels: 0,
        omitted_pairs: 0,
    };
    for (i, pair) in bands.windows(2).enumerate() {
        let bounds = [
            pair[0][0].min(pair[1][0]),
            pair[0][1],
            pair[0][2].max(pair[1][2]),
            pair[1][3],
        ];
        // 検証済みのページ内矩形なので面積はページ面積以下。
        let area = (bounds[2] - bounds[0]) * (bounds[3] - bounds[1]);
        if result.candidates.len() >= budget.max_candidates
            || area > budget.max_total_pixels - result.total_pixels
        {
            result.omitted_pairs += 1;
            continue;
        }
        result
            .candidates
            .try_reserve(1)
            .map_err(|_| "candidate allocation failed")?;
        result.candidates.push(LineMergeCandidate {
            source_indices: [i, i + 1],
            bounds,
        });
        result.total_pixels += area;
    }
    Ok(result)
}
