//! 極端に圧縮された横長行の短い初回結果を、同じモデルで再確認する。
use crate::infer::parseq::RecognizeResult;

pub(crate) fn recover_dense_horizontal_line<F>(
    base: &RecognizeResult,
    rgb: &[u8],
    width: usize,
    height: usize,
    input: (usize, usize),
    mut recognize: F,
) -> Option<RecognizeResult>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    use crate::pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input};
    let count = base.text.chars().count();
    let (iw, ih) = input;
    if height < 12
        || iw == 0
        || ih == 0
        || count > 16
        || width as u128 * ih as u128 <= (height as u128 * iw as u128).checked_mul(2)?
        || width.checked_mul(height)?.checked_mul(3)? != rgb.len()
    {
        return None;
    }
    let mut ink_columns = 0;
    let mut groups = 0;
    let mut top = height;
    let mut bottom = 0;
    let mut group_start = 0;
    let mut group_top = height;
    let mut group_bottom = 0;
    let mut in_group = false;
    for x in 0..=width {
        let mut column_top = height;
        let mut column_bottom = 0;
        if x < width {
            for y in 0..height {
                let p = (y * width + x) * 3;
                if rgb[p..p + 3].iter().map(|&v| u16::from(v)).sum::<u16>() < 660 {
                    column_top = column_top.min(y);
                    column_bottom = column_bottom.max(y + 1);
                }
            }
        }
        if column_bottom > 0 {
            ink_columns += 1;
            top = top.min(column_top);
            bottom = bottom.max(column_bottom);
            if !in_group {
                group_start = x;
                group_top = height;
                group_bottom = 0;
            }
            group_top = group_top.min(column_top);
            group_bottom = group_bottom.max(column_bottom);
            in_group = true;
        } else if in_group {
            if x - group_start <= height.saturating_mul(2)
                && group_bottom - group_top >= height.div_ceil(3)
            {
                groups += 1;
            }
            in_group = false;
        }
    }
    if ink_columns < width.div_ceil(3)
        || bottom.saturating_sub(top) < height.div_ceil(2)
        || groups < 40
        || groups < count * 4
    {
        return None;
    }
    let chunks = split_line_for_input(rgb, width, height, 220, iw, ih).ok()?;
    if chunks.len() < 2 {
        return None;
    }
    let mut text = String::new();
    let mut emitted = 0;
    let mut confidence_sum = 0.0f64;
    for (index, chunk) in chunks.iter().enumerate() {
        let crop = crop_line_chunk_rgb_u8(rgb, width, height, &chunks, index).ok()?;
        let result = recognize(&crop.data, crop.width, crop.height).ok()?;
        if result.text.trim().is_empty() || !(0.9..=1.0).contains(&result.mean_confidence) {
            return None;
        }
        let chars = result.text.chars().count();
        emitted += chars;
        confidence_sum += f64::from(result.mean_confidence) * chars as f64;
        if chunk.insert_space_before(&text, &result.text, height) {
            text.push(' ');
        }
        text.push_str(&result.text);
    }
    if emitted < (count * 2).max(groups.div_ceil(2))
        || text
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<std::collections::HashSet<_>>()
            .len()
            < 4
    {
        return None;
    }
    Some(RecognizeResult {
        text,
        mean_confidence: (confidence_sum / emitted as f64) as f32,
    })
}

/// 別の選択領域と大きく重なる行は、追加認識で重複を増幅しない。
pub(crate) fn isolated_recovery_regions(bounds: &[[i32; 4]]) -> Vec<bool> {
    let area = |b: [i32; 4]| -> u128 {
        let w = (i64::from(b[2]) - i64::from(b[0])).max(0) as u128;
        let h = (i64::from(b[3]) - i64::from(b[1])).max(0) as u128;
        w * h
    };
    bounds
        .iter()
        .enumerate()
        .map(|(i, &a)| {
            let aa = area(a);
            aa > 0
                && !bounds.iter().enumerate().any(|(j, &b)| {
                    if i == j {
                        return false;
                    }
                    let ba = area(b);
                    ba > 0
                        && area([
                            a[0].max(b[0]),
                            a[1].max(b[1]),
                            a[2].min(b[2]),
                            a[3].min(b[3]),
                        ]) * 2
                            >= aa.min(ba)
                })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/support/dense_line_recovery_policy.rs"]
mod tests;
