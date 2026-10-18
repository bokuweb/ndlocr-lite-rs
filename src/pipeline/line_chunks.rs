/// 実空白を挟む斜体と直立の切替を分割する。
mod style_transitions;

/// 傾いた字形の投影から実単語空白を確認する。
mod slanted_spaces;

/// 画像内の間隔分布から単語空白の閾値を推定する。
mod word_spaces;

/// 一定の全角相当字送りから、字形の左右余白を推定する。
mod regular_advance;

/// 回転判定へ入る小区間を、モデルに収まる水平単語の文脈へ戻す。
mod word_fragments;

/// 行頭の小画線にモデル入力の上下余白を保持する。
mod leading_marks;

/// 孤立した行頭小記号の認識文脈を維持する。
mod leading_symbols;

/// 孤立した上下の小画線だけを前の単語へ戻す。
mod punctuation_tails;

/// 分割不能な長い行から、広い完全白余白だけを認識対象の外へ分離する。
mod wide_spaces;

/// 回転した縦書き字形の文脈を保つ分割。
mod vertical;
pub use vertical::split_rotated_vertical_column_for_input;

use slanted_spaces::restore_slanted_word_spaces;
use word_fragments::{merge_rotated_word_fragments, restore_short_word_tails};
use word_spaces::{strong_connected_gap_threshold, word_gap_threshold};

/// 横書き行から切り出す、非白画素の重複・欠落のない元座標の水平区間。
/// 認識用APIは広い完全白余白の中央を省く場合があり、区間が隣接するとは限らない。
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

/// 幅/高さ比5以上で、両端が単語境界または行端のchunkだけ純白の外側列を除く。
/// 短い/疎なchunk・字形間分割・未分割行・PARSeqの回転条件が変わる場合は元の余白を保持する。
/// 除去幅が高さの四分の一未満なら、小さな側部余白と元の入力文脈を保持する。
/// 単語境界に囲まれた水平区間の孤立した上部記号には、全画素を保って最小の上下白余白を加える。
pub fn crop_line_chunk_rgb_u8(
    rgb: &[u8],
    width: usize,
    height: usize,
    chunks: &[LineChunk],
    index: usize,
) -> anyhow::Result<super::crop::CroppedImage> {
    let image = crop_line_chunk_without_mark_context(rgb, width, height, chunks, index)?;
    let bounded = chunks.len() > 1
        && (index == 0 || chunks[index].space_before)
        && chunks.get(index + 1).is_none_or(|next| next.space_before);
    Ok(if bounded {
        leading_marks::preserve_leading_mark(image)
    } else {
        image
    })
}

/// 元の単語境界・回転分類を維持して外側白列だけを調整する。
fn crop_line_chunk_without_mark_context(
    rgb: &[u8],
    width: usize,
    height: usize,
    chunks: &[LineChunk],
    index: usize,
) -> anyhow::Result<super::crop::CroppedImage> {
    use super::crop::{BBox, crop_rgb_u8, crop_rgb_u8_tight_horizontal};
    let chunk = chunks
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("invalid chunk index"))?;
    let word_bounded = chunks.len() > 1
        && (index == 0 || chunk.space_before)
        && chunks.get(index + 1).is_none_or(|next| next.space_before);
    let wide =
        (chunk.x1.saturating_sub(chunk.x0) as u128) >= (height as u128) * MIN_MARGIN_CROP_ASPECT;
    let bbox = BBox::new(chunk.x0, 0, chunk.x1, height);
    if !word_bounded || !wide {
        return crop_rgb_u8(rgb, width, height, bbox);
    }
    let image = crop_rgb_u8_tight_horizontal(rgb, width, height, bbox)?;
    let removed = chunk.x1 - chunk.x0 - image.width;
    let original_rotated = (height as u128) * 5 > ((chunk.x1 - chunk.x0) as u128) * 4;
    let tight_rotated = (height as u128) * 5 > (image.width as u128) * 4;
    if (removed as u128) * 4 < height as u128
        || original_rotated != tight_rotated
        || horizontal_ink_runs(&image) < MIN_MARGIN_CROP_INK_RUNS
    {
        return crop_rgb_u8(rgb, width, height, BBox::new(chunk.x0, 0, chunk.x1, height));
    }
    Ok(image)
}

/// 開発コーパスで短い区間の拡大誤読を避けた、白余白調整の最小幅/高さ比。
const MIN_MARGIN_CROP_ASPECT: u128 = 5;

/// 疎な記号の文脈を保持するために必要な独立インク列群数。
const MIN_MARGIN_CROP_INK_RUNS: usize = 9;

/// RGB全チャンネルの非白画素を用い、独立したインク列群の支持数を数える。
fn horizontal_ink_runs(image: &super::crop::CroppedImage) -> usize {
    let mut runs = 0;
    let mut previous = false;
    for x in 0..image.width {
        let ink = (0..image.height).any(|y| {
            image.data[(y * image.width + x) * 3..(y * image.width + x + 1) * 3] != [255; 3]
        });
        runs += usize::from(ink && !previous);
        previous = ink;
    }
    runs
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
/// 通常分割が失敗したときは、広い完全白余白だけを省いて元座標の認識区間を返す。
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
    let mut gaps = blank_gaps(rgb, width, height, threshold);
    if scaled_width <= scaled_input && gaps.len() < MAX_CHUNK_INK_RUNS {
        return Ok(vec![LineChunk {
            x0: 0,
            x1: width,
            space_before: false,
            gap_before: 0,
        }]);
    }
    let max_width = height
        .checked_mul(MAX_CHUNK_ASPECT)
        .ok_or("line dimensions overflow")?;
    let mut chunks = match split_line_at_preferred_gaps(&gaps, width, height, max_width) {
        Ok(chunks) => chunks,
        Err(error) => {
            return wide_spaces::split_separated_groups(
                rgb,
                width,
                height,
                threshold,
                input_width,
                input_height,
                max_width,
            )
            .unwrap_or(Err(error));
        }
    };
    // 最後の単語空白の後に残る語は、実入力比率と字形数に収まれば文脈を保つ。
    if let Some(first) = chunks.iter().rposition(|chunk| chunk.space_before) {
        let x0 = chunks[first].x0;
        let tail_runs = gaps.len() - gaps.partition_point(|gap| gap.midpoint <= x0);
        if first + 1 < chunks.len()
            && (width - x0) * input_height <= scaled_input
            && tail_runs < MAX_CHUNK_INK_RUNS
        {
            chunks[first].x1 = width;
            chunks.truncate(first + 1);
        }
    }
    restore_slanted_word_spaces(rgb, width, height, threshold, &mut gaps);
    restore_word_spaces(&mut chunks, &gaps);
    restore_short_word_tails(&mut chunks, &gaps, height, input_width, input_height);
    punctuation_tails::restore_separated_marks(
        rgb,
        (width, height),
        threshold,
        &mut chunks,
        &gaps,
        (input_width, input_height),
    );
    merge_rotated_word_fragments(&mut chunks, &gaps, height, input_width, input_height);
    style_transitions::split_style_transitions(rgb, width, height, threshold, &mut chunks, &gaps);
    leading_symbols::restore_leading_symbol(
        rgb,
        (width, height),
        threshold,
        &mut chunks,
        &gaps,
        (input_width, input_height),
    );
    Ok(chunks)
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
    let mut gaps = blank_gaps(rgb, width, height, threshold);
    let mut chunks = split_line_at_preferred_gaps(
        &gaps,
        width,
        height,
        height
            .checked_mul(MAX_CHUNK_ASPECT)
            .ok_or("line dimensions overflow")?,
    )?;
    restore_slanted_word_spaces(rgb, width, height, threshold, &mut gaps);
    restore_word_spaces(&mut chunks, &gaps);
    Ok(chunks)
}

/// 従来の字幅閾値による優先境界を使い、モデル入力の文脈を維持する。
fn split_line_at_preferred_gaps(
    gaps: &[BlankGap],
    width: usize,
    height: usize,
    max_width: usize,
) -> Result<Vec<LineChunk>, String> {
    let minimum = minimum_chunk_counts(gaps, width, max_width);
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
            .filter(|gap| gap.preferred_split)
            .min_by_key(|gap| gap.midpoint.abs_diff(target))
            .or_else(|| {
                let nearest = candidates
                    .iter()
                    .copied()
                    .min_by_key(|gap| gap.midpoint.abs_diff(target))?;
                // 一字幅に収まる仮名の左右画線間にも細い白列がある。
                // 通常の字間は維持し、近傍により広い字間があるときだけ避ける。
                if nearest.width <= height / 20 || nearest.internal_glyph_gap {
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
        space_before = gap.preferred_split;
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

/// 区間選択・末尾統合の完了後に、実単語間隔の連結空白を補う。
fn restore_word_spaces(chunks: &mut [LineChunk], gaps: &[BlankGap]) {
    for chunk in chunks.iter_mut().skip(1) {
        let index = gaps.partition_point(|gap| gap.midpoint < chunk.x0);
        if let Some(gap) = gaps.get(index).filter(|gap| gap.midpoint == chunk.x0) {
            chunk.space_before = gap.word_space;
        }
    }
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
    /// 実間隔の支持も含めて単語空白と推定したか。
    word_space: bool,
    /// 従来の字幅閾値で区間選択に優先する境界か。
    preferred_split: bool,
    /// 前後の画線が一字幅に収まる狭い部品間の候補か。
    internal_glyph_gap: bool,
}

/// 行全高で閾値未満の画素がない内側の列群を列挙する。
fn blank_gaps(rgb: &[u8], width: usize, height: usize, threshold: u8) -> Vec<BlankGap> {
    blank_gaps_with_word_gap_limit(rgb, width, height, threshold, usize::MAX)
}

/// 支持のある元画像の単語幅を閾値の上限とし、字形余白の保護を維持する。
fn blank_gaps_with_word_gap_limit(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    word_gap_limit: usize,
) -> Vec<BlankGap> {
    blank_gaps_with_geometry(rgb, width, height, threshold, word_gap_limit, false)
}

/// 書字方向ごとの分類を行い、一度の画素走査でインク群と白帯を調べる。
fn blank_gaps_with_geometry(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    word_gap_limit: usize,
    vertical: bool,
) -> Vec<BlankGap> {
    let mut gaps = Vec::new();
    let mut blank_start = None;
    let mut seen_ink = false;
    let mut ink_start = None;
    let mut ink_widths = Vec::new();
    let mut ink_heights = Vec::new();
    let mut ink_extents = Vec::new();
    let mut peak_height = 0;
    let mut ink_top = height;
    let mut ink_bottom = 0;
    for x in 0..width {
        let ink_height = (0..height)
            .filter(|&y| {
                let i = (y * width + x) * 3;
                let ink = u16::from(rgb[i]) + u16::from(rgb[i + 1]) + u16::from(rgb[i + 2])
                    < u16::from(threshold) * 3;
                if ink {
                    ink_top = ink_top.min(y);
                    ink_bottom = ink_bottom.max(y + 1);
                }
                ink
            })
            .count();
        let blank = ink_height == 0;
        if blank {
            if let Some(start) = ink_start.take() {
                ink_widths.push(x - start);
                ink_heights.push(peak_height);
                ink_extents.push(ink_bottom - ink_top);
                peak_height = 0;
                ink_top = height;
                ink_bottom = 0;
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
                    preferred_split: false,
                    internal_glyph_gap: false,
                });
            }
        }
    }
    if let Some(start) = ink_start {
        ink_widths.push(width - start);
        ink_heights.push(peak_height);
        ink_extents.push(ink_bottom - ink_top);
    }
    if vertical {
        return vertical::retain_glyph_boundaries(gaps, &ink_widths, &ink_extents, height);
    }
    if !ink_widths.is_empty() {
        // 字幅の3/5以上を単語空白と推定する。極端な横長インクは行高で頭打ちにする。
        let typical = representative_width(&ink_widths, &ink_extents, height).min(height);
        // 斜体などで複数字が連結した成分が複数あれば、全角相当の中央値だけで除外しない。
        let connected_runs = ink_widths
            .iter()
            .filter(|&&width| width >= height.saturating_mul(2))
            .count();
        let word_gap =
            word_gap_threshold(&gaps, typical, height, connected_runs).min(word_gap_limit);
        // 連結字形の強い二群に限り、単語内部より支持済みの空白を優先する。
        let preferred_gap = if typical >= height.saturating_mul(2).div_ceil(3) && connected_runs < 3
        {
            strong_connected_gap_threshold(&gaps, height, connected_runs)
                .map_or(typical.saturating_mul(3).div_ceil(5), |gap| {
                    gap.min(word_gap)
                })
        } else {
            typical.saturating_mul(3).div_ceil(5)
        };
        // 幅の中央値が近い英字行を除くため、全角相当の字形も十分あることを要求する。
        let wide_glyphs = ink_widths
            .iter()
            .filter(|&&width| width >= height.saturating_mul(2).div_ceil(3))
            .count();
        let wide_row = wide_glyphs >= ink_widths.len().div_ceil(3);
        let regular_advance = wide_row
            .then(|| {
                regular_advance::supported_glyph_advance(&ink_widths, &ink_extents, &gaps, height)
            })
            .flatten();
        for (i, gap) in gaps.iter_mut().enumerate() {
            let (left, right) = (ink_widths[i], ink_widths[i + 1]);
            gap.internal_glyph_gap = gap.width < height.div_ceil(10)
                && left.saturating_add(gap.width).saturating_add(right) <= height;
            let narrow_stem = (left < typical.div_ceil(2)
                && ink_heights[i] >= height.saturating_mul(2).div_ceil(3))
                || (right < typical.div_ceil(2)
                    && ink_heights[i + 1] >= height.saturating_mul(2).div_ceil(3));
            let full_neighbors = [i, i + 1].into_iter().all(|index| {
                ink_widths[index] >= height.div_ceil(2)
                    && ink_extents[index] >= height.saturating_mul(2).div_ceil(3)
            });
            let compact_left = compact_glyph_bearing(left, ink_extents[i], typical, height);
            let compact_right = compact_glyph_bearing(right, ink_extents[i + 1], typical, height);
            let paired_compact = compact_left.is_some() && compact_right.is_some();
            let bearing = |index, compact: Option<usize>| {
                // 英字の小画単独ではadvanceを推定せず、隣接する小画同士だけ補正する。
                compact
                    .filter(|_| typical >= height.saturating_mul(2).div_ceil(3) || paired_compact)
                    .or_else(|| {
                        if wide_row && full_neighbors {
                            open_glyph_bearing(
                                ink_widths[index],
                                ink_extents[index],
                                ink_heights[index],
                                typical,
                                height,
                            )
                        } else {
                            None
                        }
                    })
            };
            let (bearing_left, bearing_right) =
                (bearing(i, compact_left), bearing(i + 1, compact_right));
            let tall_narrow =
                tall_narrow_glyph(left, ink_extents[i], ink_heights[i], typical, height)
                    || tall_narrow_glyph(
                        right,
                        ink_extents[i + 1],
                        ink_heights[i + 1],
                        typical,
                        height,
                    );
            let is_word_space = |threshold| {
                if narrow_stem || tall_narrow || bearing_left.is_some() || bearing_right.is_some() {
                    // 細い字形・小字形の左右余白を差し引き、文字間隔を単語空白と誤判定しない。
                    let bearings = bearing_left
                        .unwrap_or_else(|| typical.saturating_sub(left).div_ceil(2))
                        + bearing_right
                            .unwrap_or_else(|| typical.saturating_sub(right).div_ceil(2));
                    gap.width.saturating_sub(bearings) > threshold
                } else {
                    gap.width >= threshold
                }
            };
            gap.preferred_split = is_word_space(preferred_gap);
            // 優先分割は維持し、一定の全角字送りに含まれる左右余白だけを連結空白から除く。
            let regular_bearing = regular_advance
                .filter(|&advance| {
                    // 字送りの許容幅を超す中心距離は実空白として保護する。
                    let center_twice = left as u128 + gap.width as u128 * 2 + right as u128;
                    center_twice * 4 <= advance as u128 * 8 + height as u128
                        && full_neighbors
                        && left <= height
                        && right <= height
                })
                .map(|advance| {
                    advance.saturating_sub(left).div_ceil(2)
                        + advance.saturating_sub(right).div_ceil(2)
                });
            gap.word_space = is_word_space(word_gap)
                && regular_bearing
                    .is_none_or(|bearing| gap.width.saturating_sub(bearing) > word_gap);
        }
    }
    gaps
}

/// 細い成分が過半数かつ低い成分も多ければ、十分な本体から代表字幅を推定する。
fn representative_width(widths: &[usize], extents: &[usize], height: usize) -> usize {
    let narrow = widths
        .iter()
        .filter(|&&width| width < height.div_ceil(3))
        .count();
    let small = widths
        .iter()
        .zip(extents)
        .filter(|&(width, extent)| *width < height.div_ceil(3) && *extent < height.div_ceil(2))
        .count();
    let body: Vec<_> = widths
        .iter()
        .zip(extents)
        .filter(|&(width, extent)| *width >= height.div_ceil(3) && *extent >= height.div_ceil(2))
        .map(|(&width, _)| width)
        .collect();
    let mut candidates = if narrow > widths.len() / 2
        && small >= widths.len().div_ceil(3)
        && body.len() >= widths.len().div_ceil(4)
    {
        body
    } else {
        widths.to_vec()
    };
    let middle = candidates.len() / 2;
    *candidates.select_nth_unstable(middle).1
}

/// 和文行の半字幅以上の全高字形を、細い括弧と区別する。
fn tall_narrow_glyph(
    width: usize,
    extent: usize,
    peak: usize,
    typical: usize,
    height: usize,
) -> bool {
    typical >= height.saturating_mul(2).div_ceil(3)
        && width >= typical.div_ceil(2)
        && width.saturating_mul(4) < typical.saturating_mul(3)
        && extent >= height.saturating_mul(2).div_ceil(3)
        && peak >= height.div_ceil(2)
        && peak < height.saturating_mul(2).div_ceil(3)
}

/// 背の低い小字形の背景余白を、和文では行高・英字では代表幅から推定する。
fn compact_glyph_bearing(
    width: usize,
    extent: usize,
    typical: usize,
    height: usize,
) -> Option<usize> {
    if typical < height.saturating_mul(2).div_ceil(3)
        && width < typical.div_ceil(2)
        && extent < height.div_ceil(2)
    {
        return Some(typical.saturating_sub(width).div_ceil(2));
    }
    (typical >= height.saturating_mul(2).div_ceil(3)
        && width >= typical.saturating_mul(3).div_ceil(4)
        && width <= height.saturating_mul(3) / 4
        && extent <= height.saturating_mul(2) / 3)
        .then(|| height.saturating_sub(width).div_ceil(2))
}

/// 半行高以上の幅を持つ疎な全高字形に限り、字送りの左右余白を推定する。
fn open_glyph_bearing(
    width: usize,
    extent: usize,
    peak: usize,
    typical: usize,
    height: usize,
) -> Option<usize> {
    (typical >= height.saturating_mul(3) / 5
        && width >= height.div_ceil(2)
        && width <= height
        && extent >= height.saturating_mul(2).div_ceil(3)
        && peak < height.div_ceil(2))
    .then(|| height.saturating_sub(width).div_ceil(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// 小画と本文の間は通常閾値、小画同士だけは両側の余白を考慮する。
    #[test]
    fn latin_single_and_paired_marks_obey_gap_boundaries() {
        for (paired, gap, expected) in [
            (false, 11, false),
            (false, 12, true),
            (false, 17, true),
            (true, 17, false),
            (true, 24, false),
            (true, 25, true),
        ] {
            let height = 46;
            let right_width = if paired { 7 } else { 19 };
            let width = 19 * 3 + 7 + right_width + 3 * 3 + gap;
            let mut rgb = vec![255; width * height * 3];
            let mut x = 0;
            for (ink_width, extent, following_gap) in [
                (19, 35, 3),
                (19, 35, 3),
                (19, 35, 3),
                (7, 11, gap),
                (right_width, if paired { 11 } else { 35 }, 0),
            ] {
                for y in height - extent..height {
                    for col in x..x + ink_width {
                        rgb[(y * width + col) * 3..(y * width + col) * 3 + 3].fill(0);
                    }
                }
                x += ink_width + following_gap;
            }
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(gaps.last().unwrap().word_space, expected, "{paired}/{gap}");
        }
    }

    /// 低い成分と本体の最低支持数を満たす場合だけ細い成分の偏りを除く。
    #[test]
    fn punctuation_width_requires_short_and_body_support() {
        for (short, body, expected) in [(4, 3, 20), (3, 3, 8), (4, 2, 8)] {
            let widths = [vec![8; 7], vec![20; 5]].concat();
            let extents = [
                vec![12; short],
                vec![40; 7 - short],
                vec![30; body],
                vec![12; 5 - body],
            ]
            .concat();
            assert_eq!(representative_width(&widths, &extents, 48), expected);
        }
        // 幅1/3・高さ1/2の等号は本体、直下は対象外とする。
        assert_eq!(representative_width(&[8, 8, 16], &[12, 12, 24], 48), 16);
        assert_eq!(representative_width(&[8, 8, 15], &[12, 12, 24], 48), 8);
        assert_eq!(representative_width(&[8, 8, 16], &[12, 12, 23], 48), 8);
        assert_eq!(representative_width(&[8, 8, 8], &[40, 40, 40], 48), 8);
    }

    /// 英字句読点の幅・高さの上限と全高の細字の除外を固定する。
    #[test]
    fn latin_punctuation_bearing_excludes_full_height_stems() {
        assert_eq!(compact_glyph_bearing(9, 23, 20, 48), Some(6));
        assert_eq!(compact_glyph_bearing(10, 23, 20, 48), None);
        assert_eq!(compact_glyph_bearing(9, 24, 20, 48), None);
        assert_eq!(compact_glyph_bearing(9, 40, 20, 48), None);
    }

    /// 全高で疎な字形の左右余白と広い実空白を区別する。
    #[test]
    fn open_glyph_spaces_obey_shape_boundaries() {
        for (normal, middle, extent, peak, gap, expected) in [
            (34, 30, 40, 16, 24, false),
            (34, 30, 40, 16, 48, true),
            (34, 24, 32, 23, 24, false),
            (34, 23, 40, 16, 24, true),
            (34, 40, 31, 16, 24, true),
            (34, 30, 40, 24, 24, true),
            (28, 30, 40, 16, 22, true),
            (27, 30, 40, 16, 22, true),
            (34, 49, 40, 16, 24, true),
        ] {
            let height = 48;
            let width = normal * 2 + middle + gap * 2;
            let mut rgb = vec![255; width * height * 3];
            for x in 0..width {
                let rows = if x < normal || x >= normal + gap * 2 + middle {
                    0..height
                } else if (normal + gap..normal + gap + middle).contains(&x) {
                    let top = x % 2 * (extent - peak);
                    top..top + peak
                } else {
                    0..0
                };
                for y in rows {
                    rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].fill(0);
                }
            }
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(gaps.len(), 2);
            assert!(
                gaps.iter().all(|g| g.word_space == expected),
                "normal={normal} middle={middle} extent={extent} peak={peak} gap={gap}"
            );
        }
    }

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

    /// 全高仮名の余白は除き、字幅・密度・行種別の境界と実空白を保持する。
    #[test]
    fn tall_kana_side_bearings_obey_shape_and_space_boundaries() {
        // (代表字幅、対象字幅、縦extent、列peak、空白幅、単語空白か)。
        for (normal, middle, extent, peak, gap, expected) in [
            (34, 16, 41, 26, 21, true),
            (34, 17, 41, 26, 21, false),
            (34, 23, 41, 26, 21, false),
            (34, 23, 41, 26, 27, false),
            (34, 23, 41, 31, 21, false),
            (34, 23, 41, 32, 21, true),
            (42, 27, 45, 42, 27, true),
            (34, 23, 41, 26, 28, true),
            (34, 23, 41, 26, 50, true),
            (31, 23, 41, 26, 21, true),
            (32, 23, 41, 26, 21, false),
            (34, 25, 41, 26, 21, false),
            (34, 26, 41, 26, 21, true),
            (34, 23, 41, 23, 21, true),
            (34, 23, 41, 24, 21, false),
            (34, 23, 31, 24, 21, true),
            (34, 23, 32, 24, 21, false),
            (34, 14, 48, 4, 26, true),
            (20, 14, 41, 26, 21, true),
        ] {
            let (width, height) = (normal * 2 + middle + gap * 2, 48);
            let mut rgb = vec![255; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    let offset = (x % 2) * (extent - peak);
                    if x < normal
                        || x >= width - normal
                        || ((normal + gap..normal + gap + middle).contains(&x)
                            && (offset..offset + peak).contains(&y))
                    {
                        rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    }
                }
            }
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert!(
                gaps.iter().all(|g| g.word_space == expected),
                "normal={normal} middle={middle} extent={extent} peak={peak} gap={gap}"
            );
        }
    }

    /// 小さな正方形に近い字形の左右余白と、明確な広い空白を区別する。
    #[test]
    fn compact_glyph_side_bearings_are_not_word_spaces() {
        for (gap, expected) in [(25, false), (50, true)] {
            let (width, height) = (35 + gap + 32 + gap + 35, 47);
            let mut rgb = vec![255; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    if x < 35
                        || x >= width - 35
                        || ((16..45).contains(&y) && (35 + gap..67 + gap).contains(&x))
                    {
                        rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    }
                }
            }
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(gaps.len(), 2);
            assert!(gaps.iter().all(|g| g.word_space == expected), "gap={gap}");
        }
    }

    /// 小文字相当の狭い字幅や全高の括弧では、通常の単語空白を保持する。
    #[test]
    fn compact_bearings_keep_latin_and_full_height_spaces() {
        for (normal, middle, extent, gap) in [(20, 18, 29, 13), (35, 32, 47, 25)] {
            let (width, height) = (normal * 2 + middle + gap * 2, 47);
            let mut rgb = vec![255; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    if x < normal
                        || x >= width - normal
                        || (y < extent && (normal + gap..normal + gap + middle).contains(&x))
                    {
                        rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                    }
                }
            }
            let gaps = blank_gaps(&rgb, width, height, 220);
            assert_eq!(gaps.len(), 2);
            assert!(gaps.iter().all(|gap| gap.word_space));
        }
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

    /// 行高1/20より広くても、一字幅に収まる左右画線間では分割しない。
    #[test]
    fn avoids_internal_kana_gap_within_one_glyph_width() {
        let mut columns = vec![true; 1000];
        columns[310..315].fill(false);
        columns[338..342].fill(false);
        columns[355..364].fill(false);
        columns[650..657].fill(false);
        let chunks = split_line(&image(&columns, 47), 1000, 47, 220).unwrap();
        assert!(
            chunks
                .iter()
                .any(|chunk| chunk.x0 <= 315 && chunk.x1 >= 355),
            "{chunks:?}"
        );
    }

    /// 1/10以上の字間や一字幅に収まらない画線では、新しい部品間判定を適用しない。
    #[test]
    fn keeps_wider_gaps_and_separate_glyphs() {
        for (left_start, gap_end) in [(315, 343), (298, 342)] {
            let mut columns = vec![true; 1000];
            columns[left_start - 5..left_start].fill(false);
            columns[338..gap_end].fill(false);
            columns[355..364].fill(false);
            columns[650..657].fill(false);
            let chunks = split_line(&image(&columns, 47), 1000, 47, 220).unwrap();
            assert_eq!(chunks[0].x1, 340);
        }
    }

    /// 部品間らしい白列でも、近傍に広い候補がなければ安全な白列へ戻す。
    #[test]
    fn internal_gap_without_wider_neighbor_keeps_safe_fallback() {
        let mut columns = vec![true; 1000];
        for gap in [310..314, 338..342, 355..359, 650..654] {
            columns[gap].fill(false);
        }
        let chunks = split_line(&image(&columns, 47), 1000, 47, 220).unwrap();
        assert_eq!(chunks[0].x1, 340);
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
        #![proptest_config(ProptestConfig::with_cases(32))]
        /// 一字内の部品間保護を、拡大・左右反転・水平移動でも維持する。
        #[test]
        fn internal_kana_gap_survives_scale_reflection_and_translation(
            scale in 1usize..5, padding in 0usize..11, reflected in any::<bool>(),
        ) {
            let mut columns = vec![true; 1000];
            for gap in [310..315, 338..342, 355..364, 650..657] {
                columns[gap].fill(false);
            }
            let (mut start,mut end) = (315,355);
            if reflected { columns.reverse(); (start,end) = (1000-end,1000-start); }
            let mut scaled = vec![false; padding];
            for ink in columns { scaled.extend(std::iter::repeat_n(ink,scale)); }
            scaled.extend(vec![false; padding]);
            let chunks = split_line(&image(&scaled,47*scale),scaled.len(),47*scale,220).unwrap();
            prop_assert!(chunks.iter().any(|c| c.x0 <= start*scale+padding && c.x1 >= end*scale+padding));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]
        /// 実字形の左右反転と背景の追加でも、小書き仮名と本当の空白の区別を保つ。
        #[test]
        fn real_compact_bearings_are_reflection_and_translation_invariant(
            padding in 0usize..80, reflected in any::<bool>(),
        ) {
            let im = crate::io::load_rgb_u8(std::path::Path::new(
                "tests/fixtures/small_kana_bearings/thread.png",
            )).unwrap();
            let expected: Vec<_> = blank_gaps(&im.data, im.width, im.height, 220)
                .into_iter().map(|gap| gap.word_space).collect();
            let width = im.width + padding * 2;
            let mut rgb = vec![255; width * im.height * 3];
            for y in 0..im.height {
                for x in 0..im.width {
                    let target = if reflected { im.width - 1 - x } else { x } + padding;
                    rgb[(y * width + target) * 3..(y * width + target + 1) * 3]
                        .copy_from_slice(&im.data[(y * im.width + x) * 3..(y * im.width + x + 1) * 3]);
                }
            }
            let mut actual: Vec<_> = blank_gaps(&rgb, width, im.height, 220)
                .into_iter().map(|gap| gap.word_space).collect();
            if reflected { actual.reverse(); }
            prop_assert_eq!(expected, actual);
        }
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
