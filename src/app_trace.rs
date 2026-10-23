//! Optional page diagnostics. Recording never calls inference or changes selection.
use crate::infer::deim::Detection;
use crate::pipeline::connect::RecognizedLine;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

pub(crate) struct PageTrace(Option<Value>);

impl PageTrace {
    pub(crate) fn new(
        enabled: bool,
        width: usize,
        height: usize,
        settings: impl FnOnce() -> Value,
    ) -> Self {
        Self(enabled.then(|| {
            json!({
                "schema_version": 1, "width": width, "height": height,
                "settings": settings(), "stages": {}
            })
        }))
    }

    pub(crate) fn record(&mut self, stage: &str, value: impl FnOnce() -> Value) {
        if let Some(trace) = &mut self.0 {
            trace["stages"][stage] = value();
        }
    }

    pub(crate) fn regions(
        &mut self,
        detections: &[Detection],
        rgb: &[u8],
        width: usize,
        height: usize,
        padding: usize,
    ) {
        self.record("region_selection", || {
            let coarse =
                crate::pipeline::line_detections::select_line_detections(detections, width, height);
            let selected = crate::pipeline::line_detections::line_recognition_regions(
                detections, rgb, width, height,
            );
            json!(
                detections
                    .iter()
                    .enumerate()
                    .map(|(index, d)| {
                        let mut value = detection(d);
                        value["candidate_index"] = json!(index);
                        let region = selected
                            .iter()
                            .find(|(candidate, _)| std::ptr::eq(*candidate, d));
                        let decision = if let Some((_, bounds)) = region {
                            let [x0, y0, x1, y1] = bounds.map(|v| v.max(0) as usize);
                            if x0 >= x1 || y0 >= y1 || x1 > width || y1 > height {
                                "invalid_bbox"
                            } else {
                                let (x0, y0, x1, y1) =
                                    crate::pipeline::crop::expand_bbox_xyxy_clamped(
                                        x0, y0, x1, y1, padding, width, height,
                                    );
                                value["recognition_bbox"] = json!([x0, y0, x1, y1]);
                                "prepared"
                            }
                        } else if !d.class_name.starts_with("line_") {
                            "non_line_class"
                        } else if coarse.iter().any(|candidate| std::ptr::eq(*candidate, d)) {
                            "covered_by_two_lines"
                        } else {
                            "near_identical_box"
                        };
                        value["decision"] = json!(decision);
                        value
                    })
                    .collect::<Vec<_>>()
            )
        });
    }

    pub(crate) fn lines(&mut self, stage: &str, lines: &[RecognizedLine]) {
        self.record(stage, || {
            json!(
                lines
                    .iter()
                    .map(|line| json!({
                        "bbox": line.bbox_xyxy, "text": line.text,
                        "confidence": line.confidence, "is_vertical": line.is_vertical,
                    }))
                    .collect::<Vec<_>>()
            )
        });
    }

    pub(crate) fn save(self, path: Option<&Path>, body: &str) -> Result<()> {
        if let (Some(path), Some(mut trace)) = (path, self.0) {
            trace["body"] = json!(body);
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, serde_json::to_vec_pretty(&trace)?)?;
        }
        Ok(())
    }
}

pub(crate) fn detection(d: &Detection) -> Value {
    json!({"bbox": d.box_xyxy, "class": d.class_name, "confidence": d.confidence,
           "pred_char_count": d.pred_char_count})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_decisions_account_for_duplicates_and_invalid_boxes() {
        let line = |bounds, confidence| Detection {
            class_index: 1,
            confidence,
            box_xyxy: bounds,
            pred_char_count: 3.0,
            class_name: "line_main".into(),
        };
        let detections = [
            line([2, 2, 90, 25], 0.9),
            line([2, 2, 90, 25], 0.8),
            line([-2, 2, 0, 25], 0.7),
        ];
        let mut trace = PageTrace::new(true, 100, 40, || json!({}));
        trace.regions(&detections, &vec![255; 100 * 40 * 3], 100, 40, 0);
        let value = trace.0.unwrap();
        let decisions: Vec<_> = value["stages"]["region_selection"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["decision"].as_str().unwrap())
            .collect();
        assert_eq!(
            decisions,
            ["prepared", "near_identical_box", "invalid_bbox"]
        );
    }
}
