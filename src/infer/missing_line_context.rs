//! 既存枠の外に画線が残る単一横書き行の、全体文脈による確認。
use crate::{infer::parseq::RecognizeResult, pipeline::line_context::ScoredRegion};

pub(crate) struct RecoveredContext {
    pub bounds: [i32; 4],
    pub recognized: RecognizeResult,
}

#[cfg(all(test, feature = "onnx"))]
static CONTEXT_TRACE: std::sync::Mutex<Vec<serde_json::Value>> = std::sync::Mutex::new(Vec::new());

#[cfg(all(test, feature = "onnx"))]
pub(crate) fn take_context_trace() -> Vec<serde_json::Value> {
    std::mem::take(&mut *CONTEXT_TRACE.lock().unwrap())
}

pub(crate) fn recover_missing_line_context<F>(
    rows: &[ScoredRegion<'_>],
    rgb: &[u8],
    width: usize,
    height: usize,
    native: (usize, usize),
    mut recognize: F,
) -> Option<RecoveredContext>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    if width == 0
        || height == 0
        || native.0 == 0
        || native.1 == 0
        || width.checked_mul(height)?.checked_mul(3)? != rgb.len()
    {
        return None;
    }
    let full = [
        0,
        0,
        i32::try_from(width).ok()?,
        i32::try_from(height).ok()?,
    ];
    let mut weight = 0usize;
    let mut weighted = 0f64;
    for row in rows {
        let [a, b, c, d] = row.bounds;
        let count = row.text.chars().filter(|c| !c.is_whitespace()).count();
        if a < 0
            || b < 0
            || c <= a
            || d <= b
            || c as usize > width
            || d as usize > height
            || count == 0
            || !row.mean_confidence.is_finite()
            || !(0.0..=1.0).contains(&row.mean_confidence)
        {
            return None;
        }
        weight = weight.checked_add(count)?;
        weighted += count as f64 * f64::from(row.mean_confidence);
    }
    if weight == 0 {
        return None;
    }
    let mean = weighted / weight as f64;
    if mean <= 0.0 {
        return None;
    }
    let bands = if width.saturating_mul(height) >= 1_000_000 {
        crate::pipeline::line_segment::detect_textline_bands_fast(rgb, width, height, 220)
    } else {
        crate::pipeline::line_segment::detect_textline_bands_naive(rgb, width, height, 220)
    };
    if bands.len() != 1 {
        return None;
    }
    let [x0, y0, x1, y1] = bands[0];
    if (x1 - x0) as u128 * 4 < (y1 - y0) as u128 * 5 {
        return None;
    }
    let band = [x0 as i32, y0 as i32, x1 as i32, y1 as i32];
    if !crate::pipeline::band_text_support::has_text_like_ink(rgb, width, height, band, 220) {
        return None;
    }
    let inside = |x: usize, y: usize, b: [i32; 4]| {
        x >= b[0] as usize && y >= b[1] as usize && x < (b[2] as usize) && y < (b[3] as usize)
    };
    let mut ink = Vec::new();
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
        if pixel.iter().map(|&c| u16::from(c)).sum::<u16>() < 660 {
            ink.push((index % width, index / width));
        }
    }
    if ink
        .iter()
        .all(|&(x, y)| rows.iter().any(|row| inside(x, y, row.bounds)))
    {
        return None;
    }
    let mut best: Option<RecoveredContext> = None;
    for bounds in [full, band]
        .into_iter()
        .enumerate()
        .filter_map(|(i, b)| if i == 1 && b == full { None } else { Some(b) })
    {
        let w = (bounds[2] - bounds[0]) as usize;
        let h = (bounds[3] - bounds[1]) as usize;
        if w as u128 * native.1 as u128 > h as u128 * native.0 as u128
            || !ink.iter().all(|&(x, y)| inside(x, y, bounds))
        {
            continue;
        }
        let result = if bounds == full {
            recognize(rgb, w, h)
        } else {
            let crop = crate::pipeline::crop::crop_rgb_u8(
                rgb,
                width,
                height,
                crate::pipeline::crop::BBox::new(
                    bounds[0] as usize,
                    bounds[1] as usize,
                    bounds[2] as usize,
                    bounds[3] as usize,
                ),
            )
            .ok()?;
            recognize(&crop.data, crop.width, crop.height)
        };
        #[cfg(all(test, feature = "onnx"))]
        if std::env::var_os("NDLOCR_CONTEXT_CAPTURE").is_some() {
            CONTEXT_TRACE.lock().unwrap().push(serde_json::json!({
                "bounds":bounds,"input_dimensions":[w,h],"native":[native.0,native.1],"baseline_mean":mean,
                "candidate":result.as_ref().ok().map(|x|serde_json::json!({"text":x.text,"confidence":x.mean_confidence})),
                "error":result.as_ref().err().map(ToString::to_string),
            }));
        }
        let Ok(recognized) = result else {
            continue;
        };
        let score = recognized.mean_confidence;
        if !score.is_finite()
            || score > 1.0
            || f64::from(score) <= mean
            || recognized.text.trim().is_empty()
        {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|b| score > b.recognized.mean_confidence)
        {
            best = Some(RecoveredContext { bounds, recognized });
        }
    }
    best
}

#[cfg(test)]
#[path = "../../tests/support/missing_line_context_policy.rs"]
mod tests;
