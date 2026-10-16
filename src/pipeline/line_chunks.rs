/// 横書き行から切り出す、重複・欠落のない水平区間。
#[derive(Debug, PartialEq, Eq)]
pub struct LineChunk {
    /// 区間の先頭列（含む）。
    pub x0: usize,
    /// 区間の末尾列（含まない）。
    pub x1: usize,
    /// 直前の区間との間に単語空白を補うか。
    pub space_before: bool,
    /// 直前の白列群の実幅。句読点の余白と広い空白を区別する。
    pub gap_before: usize,
}

impl LineChunk {
    /// 既存の認識文字は変えず、狭い和文句読点余白だけ追加空白の対象外にする。
    pub fn insert_space_before(&self, previous: &str, next: &str, height: usize) -> bool {
        let japanese_bearing = self.gap_before < height
            && matches!(previous.chars().next_back(), Some('、' | '。'))
            && next.chars().next().is_some_and(is_japanese_character);
        self.space_before && !japanese_bearing
    }
}

/// 和文句読点の直後を対象に、仮名・漢字のUnicode範囲を判定する。
fn is_japanese_character(value: char) -> bool {
    matches!(value, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{4dbf}'
        | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}' | '\u{20000}'..='\u{323af}')
}

/// 24x256 の固定モデルへ極端に圧縮しないための保守的な幅/高さ比。
pub const MAX_CHUNK_ASPECT: usize = 8;

/// 30文字モデルに対し、密集した字形を余裕を持って分割するインク列群数。
const MAX_CHUNK_INK_RUNS: usize = 20;

/// モデルの入力寸法を指定し、収まる短い行の文脈を保持する。
pub fn split_line_for_input(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    input_width: usize,
    input_height: usize,
) -> Result<Vec<LineChunk>, String> {
    let expected = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || expected != Some(rgb.len()) {
        return Err("invalid RGB line dimensions".into());
    }
    if input_width == 0 || input_height == 0 || input_width.checked_mul(input_height).is_none() {
        return Err("invalid model input dimensions".into());
    }
    let scaled_width = width
        .checked_mul(input_height)
        .ok_or("line dimensions overflow")?;
    let scaled_input = height
        .checked_mul(input_width)
        .ok_or("line dimensions overflow")?;
    if scaled_width <= scaled_input
        && blank_gaps(rgb, width, height, threshold).len() < MAX_CHUNK_INK_RUNS
    {
        return Ok(vec![LineChunk {
            x0: 0,
            x1: width,
            space_before: false,
            gap_before: 0,
        }]);
    }
    split_line(rgb, width, height, threshold)
}

/// インクを横切らずに行を短い区間へ分割し、分割不能なら拒否する。
pub fn split_line(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Result<Vec<LineChunk>, String> {
    let expected = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || expected != Some(rgb.len()) {
        return Err("invalid RGB line dimensions".into());
    }
    let max_width = height
        .checked_mul(MAX_CHUNK_ASPECT)
        .ok_or("line dimensions overflow")?;
    let gaps = blank_gaps(rgb, width, height, threshold);
    let minimum = minimum_chunk_counts(&gaps, width, max_width);
    let mut chunks = Vec::new();
    let mut x0 = 0;
    let mut space_before = false;
    let mut gap_before = 0;
    loop {
        let first = gaps.partition_point(|gap| gap.midpoint <= x0);
        if width - x0 <= max_width && gaps.len() - first < MAX_CHUNK_INK_RUNS {
            break;
        }
        let last = gaps
            .partition_point(|gap| gap.midpoint <= x0.saturating_add(max_width))
            .min(first.saturating_add(MAX_CHUNK_INK_RUNS));
        // 後続も幅・字形数制限内に収まる切り方だけを候補にする。
        // 幅だけから区間数を見積もると、境界の手前で漢字1字が孤立する。
        let count = minimum[first];
        if count == usize::MAX {
            return Err(format!(
                "long OCR line has no safe blank-column boundary after column {x0}"
            ));
        }
        let candidates: Vec<_> = gaps[first..last]
            .iter()
            .enumerate()
            .filter(|(offset, _)| minimum[first + offset + 1] != usize::MAX)
            .map(|(_, gap)| gap)
            .collect();
        let target = x0 + (width - x0).div_ceil(count.max(1));
        let gap = candidates
            .iter()
            .copied()
            .filter(|gap| gap.word_space)
            .min_by_key(|gap| gap.midpoint.abs_diff(target))
            .or_else(|| {
                let nearest = candidates
                    .iter()
                    .copied()
                    .min_by_key(|gap| gap.midpoint.abs_diff(target))?;
                // 行高の1/20以下の白列は漢字の部品間にもある。
                // 通常の字間は維持し、近傍により広い字間があるときだけ避ける。
                if nearest.width <= height / 20 {
                    candidates
                        .iter()
                        .copied()
                        .filter(|gap| {
                            gap.width >= height.div_ceil(10)
                                && gap.midpoint.abs_diff(target) <= height
                        })
                        .min_by_key(|gap| gap.midpoint.abs_diff(target))
                        .or(Some(nearest))
                } else {
                    Some(nearest)
                }
            })
            .ok_or_else(|| {
                format!("long OCR line has no safe blank-column boundary after column {x0}")
            })?;
        chunks.push(LineChunk {
            x0,
            x1: gap.midpoint,
            space_before,
            gap_before,
        });
        x0 = gap.midpoint;
        space_before = gap.word_space;
        gap_before = gap.width;
    }
    chunks.push(LineChunk {
        x0,
        x1: width,
        space_before,
        gap_before,
    });
    Ok(chunks)
}

/// 各白列境界から末尾までを安全に覆う最小区間数を後ろから求める。
/// 1区間の候補は字形数制限で最大20個なので、空白数に比例する。
fn minimum_chunk_counts(gaps: &[BlankGap], width: usize, max_width: usize) -> Vec<usize> {
    let mut counts = vec![usize::MAX; gaps.len() + 1];
    for first in (0..=gaps.len()).rev() {
        let x0 = if first == 0 {
            0
        } else {
            gaps[first - 1].midpoint
        };
        if width - x0 <= max_width && gaps.len() - first < MAX_CHUNK_INK_RUNS {
            counts[first] = 1;
        }
        for next in first..gaps.len().min(first.saturating_add(MAX_CHUNK_INK_RUNS)) {
            if gaps[next].midpoint - x0 > max_width {
                break;
            }
            counts[first] = counts[first].min(counts[next + 1].saturating_add(1));
        }
    }
    counts
}

/// 前後にインクがある空白の中央と、単語間隔の推定を保持する。
struct BlankGap {
    /// 空白内の分割列。
    midpoint: usize,
    /// 空白の列数。
    width: usize,
    /// 代表的な字形の幅から単語間隔と推定したか。
    word_space: bool,
}

/// 行全高で閾値未満の画素がない内側の列群を列挙する。
fn blank_gaps(rgb: &[u8], width: usize, height: usize, threshold: u8) -> Vec<BlankGap> {
    let mut gaps = Vec::new();
    let mut blank_start = None;
    let mut seen_ink = false;
    let mut ink_start = None;
    let mut ink_widths = Vec::new();
    let mut ink_heights = Vec::new();
    let mut peak_height = 0;
    for x in 0..width {
        let ink_height = (0..height)
            .filter(|&y| {
                let i = (y * width + x) * 3;
                u16::from(rgb[i]) + u16::from(rgb[i + 1]) + u16::from(rgb[i + 2])
                    < u16::from(threshold) * 3
            })
            .count();
        let blank = ink_height == 0;
        if blank {
            if let Some(start) = ink_start.take() {
                ink_widths.push(x - start);
                ink_heights.push(peak_height);
                peak_height = 0;
            }
            if seen_ink && blank_start.is_none() {
                blank_start = Some(x);
            }
        } else {
            ink_start.get_or_insert(x);
            peak_height = peak_height.max(ink_height);
            seen_ink = true;
            if let Some(start) = blank_start.take() {
                gaps.push(BlankGap {
                    midpoint: start + (x - start) / 2,
                    width: x - start,
                    word_space: false,
                });
            }
        }
    }
    if let Some(start) = ink_start {
        ink_widths.push(width - start);
        ink_heights.push(peak_height);
    }
    if !ink_widths.is_empty() {
        let middle = ink_widths.len() / 2;
        let mut sorted_widths = ink_widths.clone();
        let (_, typical_width, _) = sorted_widths.select_nth_unstable(middle);
        // 字幅の3/5以上を単語空白と推定する。極端な横長インクは行高で頭打ちにする。
        let typical = (*typical_width).min(height);
        let word_gap = typical.saturating_mul(3).div_ceil(5);
        for (i, gap) in gaps.iter_mut().enumerate() {
            let (left, right) = (ink_widths[i], ink_widths[i + 1]);
            let narrow_stem = (left < typical.div_ceil(2)
                && ink_heights[i] >= height.saturating_mul(2).div_ceil(3))
                || (right < typical.div_ceil(2)
                    && ink_heights[i + 1] >= height.saturating_mul(2).div_ceil(3));
            gap.word_space = if narrow_stem {
                // 細い字形の左右余白を差し引き、文字間隔を単語空白と誤判定しない。
                let bearings = typical.saturating_sub(left).div_ceil(2)
                    + typical.saturating_sub(right).div_ceil(2);
                gap.width.saturating_sub(bearings) > word_gap
            } else {
                gap.width >= word_gap
            };
        }
    }
    gaps
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// 列単位のインク配置をRGB画像へ変換する。
    fn image(columns: &[bool], height: usize) -> Vec<u8> {
        (0..height)
            .flat_map(|_| {
                columns
                    .iter()
                    .flat_map(|ink| [if *ink { 0 } else { 255 }; 3])
            })
            .collect()
    }

    /// 細い文字の左右余白を単語空白と取り違えず、本当の広い空白は保持する。
    #[test]
    fn narrow_side_bearings_are_not_word_spaces() {
        for (gap, expected) in [(26, false), (50, true)] {
            let mut columns = vec![true; 30];
            columns.extend(vec![false; gap]);
            columns.extend([true; 14]);
            columns.extend(vec![false; gap]);
            columns.extend([true; 30]);
            let gaps = blank_gaps(&image(&columns, 48), columns.len(), 48, 220);
            assert_eq!(gaps.len(), 2);
            assert!(gaps.iter().all(|g| g.word_space == expected), "gap={gap}");
        }
        let mut normal = vec![true; 30];
        normal.extend([false; 18]);
        normal.extend([true; 30]);
        assert!(blank_gaps(&image(&normal, 48), normal.len(), 48, 220)[0].word_space);
    }

    /// 曲線括弧の狭さだけで本当の単語空白を消さない。
    #[test]
    fn curved_narrow_glyph_keeps_word_space() {
        let (width, height) = (120, 48);
        let mut columns = vec![false; width];
        columns[..30].fill(true);
        columns[56..70].fill(true);
        columns[90..].fill(true);
        let mut rgb = image(&columns, height);
        for y in 0..height {
            for x in 56..70 {
                if x != 56 + y % 14 {
                    rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(255);
                }
            }
        }
        assert!(blank_gaps(&rgb, width, height, 220)[0].word_space);
    }

    /// 単語空白を優先し、短い最終区間まで元の画素範囲を覆う。
    #[test]
    fn splits_at_word_spaces_without_losing_pixels() {
        let mut columns = vec![true; 210];
        columns[70..78].fill(false);
        columns[145..153].fill(false);
        assert_eq!(
            split_line(&image(&columns, 12), 210, 12, 220).unwrap(),
            [
                LineChunk {
                    x0: 0,
                    x1: 74,
                    space_before: false,
                    gap_before: 0
                },
                LineChunk {
                    x0: 74,
                    x1: 149,
                    space_before: true,
                    gap_before: 8
                },
                LineChunk {
                    x0: 149,
                    x1: 210,
                    space_before: true,
                    gap_before: 8
                },
            ]
        );
    }

    /// 必要区間数が増えても、後続を安全に分割できる単語空白を維持する。
    #[test]
    fn word_space_can_require_an_additional_feasible_chunk() {
        let mut columns = vec![true; 300];
        columns[50..70].fill(false);
        columns[145..149].fill(false);
        columns[220..224].fill(false);
        let chunks = split_line(&image(&columns, 20), 300, 20, 220).unwrap();
        assert_eq!(chunks[0].x1, 60);
        assert!(chunks[1].space_before);
    }

    /// 字間の狭い白列には単語空白を挿入しない。
    #[test]
    fn narrow_glyph_gap_does_not_invent_a_space() {
        let mut columns = vec![true; 150];
        columns[90..92].fill(false);
        let chunks = split_line(&image(&columns, 12), 150, 12, 220).unwrap();
        assert_eq!(chunks[0].x1, 91);
        assert!(!chunks[1].space_before);
    }

    /// 数字1の余白程度の字間を単語空白と取り違えない。
    #[test]
    fn digit_bearing_is_not_a_word_space() {
        let mut columns = vec![true; 400];
        columns[200..208].fill(false);
        columns[280..294].fill(false);
        let chunks = split_line(&image(&columns, 32), 400, 32, 220).unwrap();
        assert_eq!(chunks[0].x1, 204);
        assert!(!chunks[1].space_before);
    }

    /// 背の高い密集文字も分割し、単語順と単語空白を保つ。
    #[test]
    fn splits_dense_glyphs_even_below_aspect_limit() {
        let mut columns = Vec::new();
        for word in 0..8 {
            if word > 0 {
                columns.extend([false; 10]);
            }
            for glyph in 0..4 {
                if glyph > 0 {
                    columns.extend([false; 2]);
                }
                columns.extend([true; 12]);
            }
        }
        let chunks = split_line(&image(&columns, 120), columns.len(), 120, 220).unwrap();
        assert!(chunks.len() > 1);
        assert!(chunks.iter().skip(1).all(|chunk| chunk.space_before));
        for chunk in chunks {
            let ink_runs = columns[chunk.x0..chunk.x1]
                .split(|ink| !ink)
                .filter(|run| !run.is_empty())
                .count();
            assert!(ink_runs <= 20, "{ink_runs}");
        }
    }

    /// 日本語程度の字幅と字間から単語スペースを捏造しない。
    #[test]
    fn japanese_glyph_bearings_do_not_create_word_spaces() {
        let mut columns = Vec::new();
        for glyph in 0..33 {
            if glyph > 0 {
                columns.extend([false; 15]);
            }
            columns.extend([true; 34]);
        }
        let chunks = split_line(&image(&columns, 41), columns.len(), 41, 220).unwrap();
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| !chunk.space_before));
    }

    /// 末尾の句点だけを独立区間にせず、必要な区間数に合わせて幅を均す。
    #[test]
    fn balances_chunks_instead_of_leaving_orphan_punctuation() {
        let mut columns = vec![true; 170];
        columns[70..74].fill(false);
        columns[150..154].fill(false);
        let chunks = split_line(&image(&columns, 20), 170, 20, 220).unwrap();
        assert_eq!(chunks[0].x1, 72);
        assert_eq!(chunks[1].x0, 72);
        assert_eq!(chunks[1].x1, 170);
    }

    /// 漢字の左右部品間の細い白列より、隣の字間を分割位置に選ぶ。
    #[test]
    fn avoids_narrow_component_gap_near_balanced_boundary() {
        let mut columns = vec![true; 843];
        columns[259..262].fill(false);
        columns[283..285].fill(false);
        columns[303..308].fill(false);
        columns[566..573].fill(false);
        let chunks = split_line(&image(&columns, 43), 843, 43, 220).unwrap();
        assert_eq!(chunks[0].x1, 305);
        assert_eq!(chunks.last().unwrap().x1, 843);
    }

    /// 行高の1/20を超える字間や遠くの空白では、均等分割を変更しない。
    #[test]
    fn keeps_normal_bearings_and_does_not_seek_distant_gaps() {
        for (height, width, narrow, wide, expected) in [
            (21, 300, 149..151, 158..162, 150),
            (43, 500, 249..251, 300..306, 250),
        ] {
            let mut columns = vec![true; width];
            columns[narrow].fill(false);
            columns[wide].fill(false);
            let chunks = split_line(&image(&columns, height), width, height, 220).unwrap();
            assert_eq!(chunks[0].x1, expected);
        }
    }

    /// 白列がない長い区間を推測で途中切断しない。
    #[test]
    fn rejects_wide_connected_ink_and_invalid_buffers() {
        assert!(split_line(&image(&[true; 150], 12), 150, 12, 220).is_err());
        assert!(split_line(&[], 1, 1, 220).is_err());
        assert!(split_line(&[], 0, 1, 220).is_err());
        assert!(split_line(&[], usize::MAX, 2, 220).is_err());
    }

    proptest! {
        /// 字形の余白判定は左右反転と行外余白の追加に依存しない。
        #[test]
        fn word_spaces_are_reflection_and_translation_invariant(
            columns in prop::collection::vec(any::<bool>(), 1..300),
            height in 6usize..50, padding in 0usize..80,
        ) {
            let expected: Vec<_> = blank_gaps(&image(&columns, height), columns.len(), height, 220)
                .into_iter().map(|gap| gap.word_space).collect();
            let mut translated = vec![false; padding];
            translated.extend_from_slice(&columns);
            translated.extend(vec![false; padding]);
            let actual: Vec<_> = blank_gaps(&image(&translated, height), translated.len(), height, 220)
                .into_iter().map(|gap| gap.word_space).collect();
            prop_assert_eq!(&expected, &actual);
            translated.reverse();
            let reflected: Vec<_> = blank_gaps(&image(&translated, height), translated.len(), height, 220)
                .into_iter().rev().map(|gap| gap.word_space).collect();
            prop_assert_eq!(expected, reflected);
        }

        /// 成功した分割は全画素を一度ずつ覆い、白列以外を横切らない。
        #[test]
        fn chunks_tile_the_image_without_cutting_ink(
            columns in prop::collection::vec(any::<bool>(), 1..700), height in 6usize..50,
        ) {
            if let Ok(chunks) = split_line(&image(&columns, height), columns.len(), height, 220) {
                prop_assert!(!chunks.is_empty());
                let mut end = 0;
                for chunk in chunks {
                    prop_assert_eq!(chunk.x0, end);
                    prop_assert!(chunk.x1 > chunk.x0);
                    prop_assert!(chunk.x1 - chunk.x0 <= height * MAX_CHUNK_ASPECT);
                    let ink_runs = columns[chunk.x0..chunk.x1]
                        .split(|ink| !ink)
                        .filter(|run| !run.is_empty())
                        .count();
                    prop_assert!(ink_runs <= MAX_CHUNK_INK_RUNS);
                    if chunk.x1 < columns.len() {
                        prop_assert!(!columns[chunk.x1]);
                    }
                    end = chunk.x1;
                }
                prop_assert_eq!(end, columns.len());
            }
        }
    }
}
