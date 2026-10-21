use crate::infer::parseq::RecognizeResult;

#[derive(Clone, Copy)]
pub(super) struct Options {
    pub(super) minimum_confidence: f32,
    pub(super) padding: usize,
}

pub(super) fn try_if_empty<F>(
    has_detections: bool,
    rgb: &[u8],
    dimensions: (usize, usize),
    options: Options,
    recognize: F,
) -> Option<([i32; 4], RecognizeResult)>
where
    F: FnOnce(&[u8], usize, usize) -> anyhow::Result<RecognizeResult>,
{
    if has_detections {
        return None;
    }
    crate::infer::vertical_fallback::try_recognize_with_padding(
        rgb,
        dimensions.0,
        dimensions.1,
        options.minimum_confidence,
        options.padding,
        recognize,
    )
}

#[cfg(test)]
#[path = "../../tests/support/vertical_cli_fallback.rs"]
mod tests;

pub(super) fn vertical_bucket(
    (width, height): (usize, usize),
    (short, medium): (usize, usize),
) -> f32 {
    crate::cascade::estimate_pred_char_bucket(height, width, short, medium)
}

pub(super) fn try_prepared_if_empty<F>(
    has_detections: bool,
    rgb: &[u8],
    dimensions: (usize, usize),
    options: Options,
    thresholds: (usize, usize),
    recognize: F,
) -> Option<(super::PreparedLineCrop, RecognizeResult)>
where
    F: FnOnce(&super::PreparedLineCrop) -> anyhow::Result<RecognizeResult>,
{
    let mut prepared = None;
    let (bounds, recognized) = try_if_empty(
        has_detections,
        rgb,
        dimensions,
        options,
        |pixels, width, height| {
            let line = super::PreparedLineCrop {
                crop: crate::pipeline::crop::CroppedImage {
                    data: pixels.to_vec(),
                    width,
                    height,
                },
                bbox_xyxy: [0, 0, width as i32, height as i32],
                confidence: 1.0,
                pred_char_count: vertical_bucket((width, height), thresholds),
                is_vertical: true,
            };
            let result = recognize(&line);
            prepared = Some(line);
            result
        },
    )?;
    let mut line = prepared?;
    line.bbox_xyxy = bounds;
    Some((line, recognized))
}

pub(super) fn reuse_or_legacy<F, P>(
    accepted: Option<(super::PreparedLineCrop, RecognizeResult)>,
    finish: P,
    legacy: F,
) -> anyhow::Result<(Vec<super::PreparedLineCrop>, Vec<RecognizeResult>)>
where
    F: FnOnce() -> anyhow::Result<(Vec<super::PreparedLineCrop>, Vec<RecognizeResult>)>,
    P: FnOnce(
        &[super::PreparedLineCrop],
        Vec<RecognizeResult>,
    ) -> anyhow::Result<Vec<RecognizeResult>>,
{
    match accepted {
        Some((crop, result)) => {
            let crops = vec![crop];
            let results = finish(&crops, vec![result])?;
            Ok((crops, results))
        }
        None => legacy(),
    }
}
