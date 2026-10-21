//! Image-supported direction selection only when detector regions are absent.
use crate::pipeline::{
    crop::{BBox, crop_rgb_u8},
    vertical_segment::{detect_vertical_textline_columns, rotate_vertical_column_rgb_u8},
};

fn supported_column(rgb: &[u8], width: usize, height: usize) -> Option<([i32; 4], usize)> {
    let expected = width.checked_mul(height)?.checked_mul(3)?;
    if width < 12
        || (height as u128) < width as u128 * 8
        || expected != rgb.len()
        || width > i32::MAX as usize
        || height > i32::MAX as usize
    {
        return None;
    }
    let columns = detect_vertical_textline_columns(rgb, width, height, 220).ok()?;
    let [column] = columns.as_slice() else {
        return None;
    };
    let detected_width = column[2] - column[0];
    if detected_width < 12 || (detected_width as u128) * 2 < width as u128 {
        return None;
    }
    let bounds = nonwhite_envelope(rgb, width, height)?;
    let (w, h) = (bounds.xmax - bounds.xmin, bounds.ymax - bounds.ymin);
    if w < 12 || (h as u128) < (w as u128) * 8 {
        return None;
    }
    let crop = crop_rgb_u8(rgb, width, height, bounds).ok()?;
    let rotated = rotate_vertical_column_rgb_u8(&crop.data, w, h).ok()?;
    let anchors = crate::pipeline::line_chunks::fallback_column_anchor_count(
        &rotated.data,
        rotated.width,
        rotated.height,
    )?;
    if !crate::pipeline::band_text_support::has_text_like_ink(
        &rotated.data,
        rotated.width,
        rotated.height,
        [0, 0, rotated.width as i32, rotated.height as i32],
        220,
    ) {
        return None;
    }
    Some((
        [
            bounds.xmin as i32,
            bounds.ymin as i32,
            bounds.xmax as i32,
            bounds.ymax as i32,
        ],
        anchors,
    ))
}

#[cfg(test)]
pub(super) fn single_column(rgb: &[u8], width: usize, height: usize) -> Option<[i32; 4]> {
    supported_column(rgb, width, height).map(|(bounds, _)| bounds)
}

fn nonwhite_envelope(rgb: &[u8], width: usize, height: usize) -> Option<BBox> {
    let (mut x0, mut y0, mut x1, mut y1) = (width, height, 0usize, 0usize);
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
        if pixel.iter().map(|&x| u16::from(x)).sum::<u16>() < 765 {
            let (x, y) = (index % width, index / width);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some(BBox::new(
        x0.saturating_sub(1),
        y0.saturating_sub(1),
        (x1 + 1).min(width),
        (y1 + 1).min(height),
    ))
}

#[cfg(test)]
#[path = "../../tests/support/vertical_fallback_geometry.rs"]
mod tests;

/// Try the supported direction once; None leaves the legacy path available.
pub(super) fn try_recognize<F>(
    rgb: &[u8],
    width: usize,
    height: usize,
    minimum_confidence: f32,
    recognize: F,
) -> Option<([i32; 4], crate::infer::parseq::RecognizeResult)>
where
    F: FnOnce(&[u8], usize, usize) -> anyhow::Result<crate::infer::parseq::RecognizeResult>,
{
    try_recognize_with_padding(rgb, width, height, minimum_confidence, 0, recognize)
}

pub(crate) fn try_recognize_with_padding<F>(
    rgb: &[u8],
    width: usize,
    height: usize,
    minimum_confidence: f32,
    padding: usize,
    recognize: F,
) -> Option<([i32; 4], crate::infer::parseq::RecognizeResult)>
where
    F: FnOnce(&[u8], usize, usize) -> anyhow::Result<crate::infer::parseq::RecognizeResult>,
{
    let (bounds, anchors) = supported_column(rgb, width, height)?;
    let (x0, y0, x1, y1) = crate::pipeline::crop::expand_bbox_xyxy_clamped(
        bounds[0] as usize,
        bounds[1] as usize,
        bounds[2] as usize,
        bounds[3] as usize,
        padding.min(width.max(height)),
        width,
        height,
    );
    let bounds = [x0 as i32, y0 as i32, x1 as i32, y1 as i32];
    let crop = crop_rgb_u8(
        rgb,
        width,
        height,
        BBox::new(
            bounds[0] as usize,
            bounds[1] as usize,
            bounds[2] as usize,
            bounds[3] as usize,
        ),
    )
    .ok()?;
    let result = recognize(&crop.data, crop.width, crop.height).ok()?;
    if result.text.chars().filter(|c| !c.is_whitespace()).count() < anchors.div_ceil(2)
        || result
            .text
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            < 4
        || result.text.trim().is_empty()
        || !result.mean_confidence.is_finite()
        || !(0.9..=1.0).contains(&result.mean_confidence)
        || minimum_confidence.is_nan()
        || result.mean_confidence < minimum_confidence
    {
        return None;
    }
    Some((bounds, result))
}

#[cfg(test)]
#[path = "../../tests/support/vertical_fallback_padding.rs"]
mod padding_tests;
