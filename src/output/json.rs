use crate::pipeline::connect::RecognizedLine;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct OcrJson {
    pub contents: Vec<Vec<LineJson>>,
    pub imginfo: ImgInfo,
}

#[derive(Debug, Serialize)]
pub struct ImgInfo {
    pub img_width: usize,
    pub img_height: usize,
    pub img_path: String,
    pub img_name: String,
}

#[derive(Debug, Serialize)]
pub struct LineJson {
    #[serde(rename = "boundingBox")]
    pub bounding_box: [[i32; 2]; 4],
    pub id: usize,
    #[serde(rename = "isVertical")]
    pub is_vertical: String,
    pub text: String,
    #[serde(rename = "isTextline")]
    pub is_textline: String,
    pub confidence: f32,
}

pub fn build_ocr_json(
    lines: &[RecognizedLine],
    img_width: usize,
    img_height: usize,
    img_path: &str,
    img_name: &str,
) -> OcrJson {
    let items = lines
        .iter()
        .enumerate()
        .map(|(id, l)| {
            let [xmin, ymin, xmax, ymax] = l.bbox_xyxy;
            LineJson {
                bounding_box: [[xmin, ymin], [xmin, ymax], [xmax, ymin], [xmax, ymax]],
                id,
                is_vertical: if l.is_vertical { "true" } else { "false" }.into(),
                text: l.text.clone(),
                is_textline: "true".into(),
                confidence: l.confidence,
            }
        })
        .collect();
    OcrJson {
        contents: vec![items],
        imginfo: ImgInfo {
            img_width,
            img_height,
            img_path: img_path.to_string(),
            img_name: img_name.to_string(),
        },
    }
}

pub fn save_ocr_json(data: &OcrJson, path: &Path) -> anyhow::Result<()> {
    let body = serde_json::to_string_pretty(data)?;
    std::fs::write(path, body)?;
    Ok(())
}
