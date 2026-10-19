//! 実入力比率を超える横書き代替帯を、安全な区間で確認する。
use crate::infer::parseq::RecognizeResult;

/// 既存の安全区間をまたぐ、短い不確かな語尾の原画素文脈。
struct TailPair<'a> {
    rgb: &'a [u8],
    size: (usize, usize),
    input: (usize, usize),
    chunks: &'a [crate::pipeline::line_chunks::LineChunk],
    index: usize,
}

impl TailPair<'_> {
    fn recheck<F>(
        &self,
        previous: &RecognizeResult,
        current: &RecognizeResult,
        recognize: &mut F,
    ) -> Option<RecognizeResult>
    where
        F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
    {
        let (width, height) = self.size;
        let (iw, ih) = self.input;
        let left = self.chunks.get(self.index.checked_sub(1)?)?;
        let right = self.chunks.get(self.index)?;
        let previous_count = previous.text.chars().count();
        let current_count = current.text.chars().count();
        if height == 0
            || iw == 0
            || ih == 0
            || width.checked_mul(height)?.checked_mul(3)? != self.rgb.len()
            || left.x0 >= left.x1
            || left.x1 != right.x0
            || right.x0 >= right.x1
            || right.x1 > width
            || right.space_before
            || self
                .chunks
                .get(self.index + 1)
                .is_some_and(|next| !next.space_before)
            || right.x1 - right.x0 > height.saturating_mul(2)
            || left.x1 - left.x0 < height.saturating_mul(3)
            || previous_count < 2
            || !(1..=2).contains(&current_count)
            || current.text.chars().any(char::is_whitespace)
            || !(0.98..=1.0).contains(&previous.mean_confidence)
            || !(0.9..0.95).contains(&current.mean_confidence)
            || (right.x1 - left.x0) as u128 * ih as u128 > height as u128 * iw as u128
        {
            return None;
        }
        let crop = crate::pipeline::crop::crop_rgb_u8(
            self.rgb,
            width,
            height,
            crate::pipeline::crop::BBox::new(left.x0, 0, right.x1, height),
        )
        .ok()?;
        let candidate = recognize(&crop.data, crop.width, crop.height).ok()?;
        let suffix = candidate.text.strip_prefix(&previous.text)?;
        let mean = (f64::from(previous.mean_confidence) * previous_count as f64
            + f64::from(current.mean_confidence) * current_count as f64)
            / (previous_count + current_count) as f64;
        ((0.98..=1.0).contains(&candidate.mean_confidence)
            && f64::from(candidate.mean_confidence) > mean
            && suffix.chars().count() == current_count
            && suffix != current.text
            && !suffix.chars().any(char::is_whitespace))
        .then_some(candidate)
    }
}

pub(crate) fn recover_overwide_band<F>(
    rgb: &[u8],
    w: usize,
    h: usize,
    input: (usize, usize),
    mut recognize: F,
) -> Option<RecognizeResult>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    use crate::pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input};
    let (iw, ih) = input;
    if w == 0
        || h == 0
        || iw == 0
        || ih == 0
        || w.checked_mul(h)?.checked_mul(3)? != rgb.len()
        || w as u128 * ih as u128 <= h as u128 * iw as u128
    {
        return None;
    }
    let chunks = split_line_for_input(rgb, w, h, 220, iw, ih).ok()?;
    if chunks.len() < 2 {
        return None;
    }
    let separated = chunks.windows(2).any(|pair| pair[0].x1 < pair[1].x0);
    let mut pieces = Vec::new();
    let mut words = Vec::new();
    let mut scores = Vec::new();
    let mut all_small = true;
    let mut needs_context = false;
    let mut count = 0;
    let mut confidence_sum = 0.0f64;
    for index in 0..chunks.len() {
        let crop = crop_line_chunk_rgb_u8(rgb, w, h, &chunks, index).ok()?;
        let mut small = false;
        let crop = if separated && crop.width <= crop.height.saturating_mul(2) {
            let compact = crate::pipeline::horizontal_margins::compact_horizontal_margins(
                &crop.data,
                crop.width,
                crop.height,
                crop.height / 16,
            )
            .ok()?;
            if compact.content_width <= crop.height {
                small = true;
                compact.image
            } else {
                crop
            }
        } else {
            crop
        };
        let result = recognize(&crop.data, crop.width, crop.height).ok()?;
        if result.text.trim().is_empty() || !(0.0..=1.0).contains(&result.mean_confidence) {
            return None;
        }
        all_small &= small;
        needs_context |= result.mean_confidence < 0.9;
        let chars = result.text.chars().count();
        count += chars;
        confidence_sum += f64::from(result.mean_confidence) * chars as f64;
        scores.push(result.mean_confidence);
        words.push(result.text);
        pieces.push(crop);
    }
    let join = |words: &[String]| {
        let mut text = String::new();
        for (chunk, word) in chunks.iter().zip(words) {
            if chunk.insert_space_before(&text, word, h) {
                text.push(' ');
            }
            text.push_str(word);
        }
        text
    };
    if !needs_context {
        let mut recovered = false;
        for index in 1..chunks.len() {
            let pair = TailPair {
                rgb,
                size: (w, h),
                input,
                chunks: &chunks,
                index,
            };
            let previous = RecognizeResult {
                text: words[index - 1].clone(),
                mean_confidence: scores[index - 1],
            };
            let current = RecognizeResult {
                text: words[index].clone(),
                mean_confidence: scores[index],
            };
            if let Some(candidate) = pair.recheck(&previous, &current, &mut recognize) {
                words[index] = candidate.text[previous.text.len()..].to_owned();
                scores[index - 1] = candidate.mean_confidence;
                scores[index] = candidate.mean_confidence;
                recovered = true;
                // 次区間はspace_before=trueなので、採用した組とは重ならない。
            }
        }
        if recovered {
            confidence_sum = words
                .iter()
                .zip(&scores)
                .map(|(text, &score)| text.chars().count() as f64 * f64::from(score))
                .sum();
        }
        return Some(RecognizeResult {
            text: join(&words),
            mean_confidence: (confidence_sum / count as f64) as f32,
        });
    }
    let context_h = pieces[0].height;
    if pieces.iter().any(|p| p.height != context_h) {
        return None;
    }
    let context_w = pieces
        .iter()
        .try_fold(0usize, |sum, p| sum.checked_add(p.width))?;
    if context_w as u128 * ih as u128 > context_h as u128 * iw as u128 {
        return None;
    }
    let mut data = vec![255; context_w.checked_mul(context_h)?.checked_mul(3)?];
    let mut x = 0;
    for piece in &pieces {
        for y in 0..context_h {
            let from = y * piece.width * 3;
            let to = (y * context_w + x) * 3;
            data[to..to + piece.width * 3]
                .copy_from_slice(&piece.data[from..from + piece.width * 3]);
        }
        x += piece.width;
    }
    let context = recognize(&data, context_w, context_h).ok()?;
    if !(0.98..=1.0).contains(&context.mean_confidence) {
        return None;
    }
    let chars: Vec<_> = context
        .text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let individual: Vec<_> = words
        .iter()
        .flat_map(|word| word.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    let text = if chars == individual {
        join(&words)
    } else if all_small && chars.len() == words.len() {
        let mapped: Vec<_> = chars.iter().map(char::to_string).collect();
        join(&mapped)
    } else {
        return None;
    };
    Some(RecognizeResult {
        text,
        mean_confidence: context.mean_confidence,
    })
}

#[cfg(test)]
#[path = "../../tests/support/band_line_recovery_policy.rs"]
mod tests;
