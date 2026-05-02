use anyhow::Result;

use crate::infer::deim::Detection;
use crate::pipeline::connect::{RecognizedLine, recognize_line_detections_with_cascade_detailed};
use crate::pipeline::crop::CroppedImage;
use crate::pipeline::reading_order::sort_lines_in_reading_order;

pub struct PageInput<'a> {
    pub rgb: &'a [u8],
    pub width: usize,
    pub height: usize,
    pub detections: &'a [Detection],
}

pub struct PageOutput {
    pub total_detection_count: usize,
    pub line_detection_count: usize,
    pub texts: Vec<String>,
    pub lines: Vec<RecognizedLine>,
}

pub fn run_page<F30, F50, F100>(
    input: PageInput<'_>,
    recognize30: F30,
    recognize50: F50,
    recognize100: F100,
) -> Result<PageOutput>
where
    F30: Fn(&CroppedImage) -> String,
    F50: Fn(&CroppedImage) -> String,
    F100: Fn(&CroppedImage) -> String,
{
    let line_detection_count = input
        .detections
        .iter()
        .filter(|d| d.class_name.starts_with("line_"))
        .count();
    let mut lines = recognize_line_detections_with_cascade_detailed(
        input.rgb,
        input.width,
        input.height,
        input.detections,
        recognize30,
        recognize50,
        recognize100,
    )?;
    sort_lines_in_reading_order(&mut lines);
    let texts = lines.iter().map(|l| l.text.clone()).collect();
    Ok(PageOutput {
        total_detection_count: input.detections.len(),
        line_detection_count,
        texts,
        lines,
    })
}
