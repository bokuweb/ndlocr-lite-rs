//! 実入力比率を超える横書き代替帯を、安全な区間で確認する。
use crate::infer::parseq::RecognizeResult;
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
