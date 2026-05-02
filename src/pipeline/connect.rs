use anyhow::Result;
use std::collections::HashMap;

use crate::infer::deim::Detection;
use crate::pipeline::cascade::{LineCandidate, run_cascade_with_idx};
use crate::pipeline::crop::{BBox, CroppedImage, crop_rgb_u8};

#[derive(Clone, Debug, PartialEq)]
pub struct RecognizedLine {
    pub bbox_xyxy: [i32; 4],
    pub text: String,
    pub confidence: f32,
    pub is_vertical: bool,
}

pub fn recognize_line_detections_with_cascade<F30, F50, F100>(
    rgb: &[u8],
    width: usize,
    height: usize,
    detections: &[Detection],
    recognize30: F30,
    recognize50: F50,
    recognize100: F100,
) -> Result<Vec<String>>
where
    F30: Fn(&CroppedImage) -> String,
    F50: Fn(&CroppedImage) -> String,
    F100: Fn(&CroppedImage) -> String,
{
    Ok(recognize_line_detections_with_cascade_detailed(
        rgb,
        width,
        height,
        detections,
        recognize30,
        recognize50,
        recognize100,
    )?
    .into_iter()
    .map(|l| l.text)
    .collect())
}

pub fn recognize_line_detections_with_cascade_detailed<F30, F50, F100>(
    rgb: &[u8],
    width: usize,
    height: usize,
    detections: &[Detection],
    recognize30: F30,
    recognize50: F50,
    recognize100: F100,
) -> Result<Vec<RecognizedLine>>
where
    F30: Fn(&CroppedImage) -> String,
    F50: Fn(&CroppedImage) -> String,
    F100: Fn(&CroppedImage) -> String,
{
    let mut lines = Vec::new();
    let mut crop_map = HashMap::new();
    let mut bbox_map = HashMap::new();
    let mut conf_map = HashMap::new();
    for det in detections
        .iter()
        .filter(|d| d.class_name.starts_with("line_"))
    {
        let idx = lines.len();
        let bbox = BBox::new(
            det.box_xyxy[0] as usize,
            det.box_xyxy[1] as usize,
            det.box_xyxy[2] as usize,
            det.box_xyxy[3] as usize,
        );
        let crop = crop_rgb_u8(rgb, width, height, bbox)?;
        lines.push(LineCandidate::new(idx, det.pred_char_count));
        crop_map.insert(idx, crop);
        bbox_map.insert(idx, det.box_xyxy);
        conf_map.insert(idx, det.confidence);
    }
    let res = run_cascade_with_idx(
        lines,
        |l| recognize30(crop_map.get(&l.idx).expect("crop")),
        |l| recognize50(crop_map.get(&l.idx).expect("crop")),
        |l| recognize100(crop_map.get(&l.idx).expect("crop")),
    );
    Ok(res
        .into_iter()
        .map(|(idx, text)| RecognizedLine {
            bbox_xyxy: *bbox_map.get(&idx).expect("bbox"),
            text,
            confidence: *conf_map.get(&idx).expect("conf"),
            is_vertical: {
                let b = *bbox_map.get(&idx).expect("bbox");
                (b[3] - b[1]) > (b[2] - b[0])
            },
        })
        .collect())
}
