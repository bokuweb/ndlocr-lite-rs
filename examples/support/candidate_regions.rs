//! 診断で、通常の分割と明示的な単一ROIを選ぶ。
use anyhow::Result;
use ndlocr_lite_rs::pipeline::line_chunks::LineChunk;

pub fn choose_regions(
    whole_roi: bool,
    width: usize,
    split: impl FnOnce() -> Result<Vec<LineChunk>>,
) -> Result<Vec<LineChunk>> {
    if whole_roi {
        anyhow::ensure!(width > 0, "whole ROI width must be positive");
        Ok(vec![LineChunk {
            x0: 0,
            x1: width,
            space_before: false,
            gap_before: 0,
        }])
    } else {
        split()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_roi_preserves_the_entire_supplied_range_without_splitting() {
        let actual = choose_regions(true, 400, || {
            panic!("whole ROI must not invoke the splitter")
        })
        .unwrap();
        assert_eq!(
            actual,
            vec![LineChunk {
                x0: 0,
                x1: 400,
                space_before: false,
                gap_before: 0
            }]
        );
    }

    #[test]
    fn default_mode_keeps_the_splitters_regions_and_errors() {
        let expected = vec![
            LineChunk {
                x0: 2,
                x1: 20,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 30,
                x1: 40,
                space_before: true,
                gap_before: 10,
            },
        ];
        let actual = choose_regions(false, 40, || {
            Ok(vec![
                LineChunk {
                    x0: 2,
                    x1: 20,
                    space_before: false,
                    gap_before: 0,
                },
                LineChunk {
                    x0: 30,
                    x1: 40,
                    space_before: true,
                    gap_before: 10,
                },
            ])
        })
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            choose_regions(false, 40, || Err(anyhow::anyhow!("original split error")))
                .unwrap_err()
                .to_string(),
            "original split error"
        );
    }

    #[test]
    fn whole_roi_rejects_zero_width_before_calling_the_splitter() {
        assert!(
            choose_regions(true, 0, || panic!(
                "invalid ROI must not invoke the splitter"
            ))
            .is_err()
        );
    }
}
