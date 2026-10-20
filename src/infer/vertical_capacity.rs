//! Conservative capacity recovery for very long vertical page fragments.

#[cfg(all(test, feature = "onnx"))]
mod capture;
#[cfg(all(test, feature = "onnx"))]
mod native;
mod otsu;
mod policy;
#[cfg(test)]
mod prefix;

struct PreparedColumn {
    original: Vec<u8>,
    support: Vec<u8>,
    width: usize,
    height: usize,
}

#[cfg(test)]
fn prepare(rgb: &[u8], width: usize, height: usize) -> Result<PreparedColumn, String> {
    let rotated =
        crate::pipeline::vertical_segment::rotate_vertical_column_rgb_u8(rgb, width, height)?;
    let support = crate::pipeline::background::normalize_chromatic_panels(
        &rotated.data,
        rotated.width,
        rotated.height,
    )?
    .into_owned();
    Ok(PreparedColumn {
        original: rotated.data,
        support,
        width: rotated.width,
        height: rotated.height,
    })
}

fn prepare_otsu(rgb: &[u8], width: usize, height: usize) -> Option<(PreparedColumn, u8)> {
    let rotated =
        crate::pipeline::vertical_segment::rotate_vertical_column_rgb_u8(rgb, width, height)
            .ok()?;
    let (support, threshold) = otsu::support_mask(&rotated.data)?;
    Some((
        PreparedColumn {
            original: rotated.data,
            support,
            width: rotated.width,
            height: rotated.height,
        },
        threshold,
    ))
}

#[cfg(feature = "onnx")]
pub(super) fn recover<F>(
    base: &crate::infer::parseq::RecognizeResult,
    rgb: &[u8],
    dimensions: (usize, usize),
    input: (usize, usize),
    recognize: F,
) -> Option<crate::infer::parseq::RecognizeResult>
where
    F: FnMut(&[u8], usize, usize) -> anyhow::Result<crate::infer::parseq::RecognizeResult>,
{
    policy::recover(base, rgb, dimensions, input, recognize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    #[ignore = "diagnostic; requires frozen real-column manifest and new output directory"]
    fn record_prepared_columns_and_existing_splits() {
        record_prepared(false);
    }

    #[test]
    #[ignore = "diagnostic; requires frozen real-column manifest and new output directory"]
    fn record_otsu_prepared_columns_and_existing_splits() {
        record_prepared(true);
    }

    fn record_prepared(otsu: bool) {
        let Ok(manifest) = std::env::var("NDLOCR_VERTICAL_CAPACITY_MANIFEST") else {
            return;
        };
        let output =
            std::path::PathBuf::from(std::env::var("NDLOCR_VERTICAL_CAPACITY_OUTPUT").unwrap());
        std::fs::create_dir(&output).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
        let inputs = manifest["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 9);
        let mut records = Vec::new();
        for x in inputs {
            let id = x["id"].as_str().unwrap();
            assert!(id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'));
            let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
            let before = rgb.clone();
            let (width, height) = (
                x["width"].as_u64().unwrap() as usize,
                x["height"].as_u64().unwrap() as usize,
            );
            let (prepared, threshold) = if otsu {
                let (prepared, threshold) = prepare_otsu(&rgb, width, height)
                    .expect("fixed real crop has two supported gray levels");
                (prepared, Some(threshold))
            } else {
                (prepare(&rgb, width, height).unwrap(), None)
            };
            std::fs::write(
                output.join(format!("{id}-original.rgb")),
                &prepared.original,
            )
            .unwrap();
            std::fs::write(output.join(format!("{id}-support.rgb")), &prepared.support).unwrap();
            let mut models = Vec::new();
            for (label, iw) in [("30", 256), ("50", 384), ("100", 768)] {
                let result = crate::pipeline::line_chunks::split_rotated_vertical_column_for_input(
                    &prepared.support,
                    prepared.width,
                    prepared.height,
                    220,
                    iw,
                    24,
                );
                match result {
                    Ok(chunks) => {
                        let mut parts = Vec::new();
                        for (index, c) in chunks.iter().enumerate() {
                            let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                                &prepared.original,
                                prepared.width,
                                prepared.height,
                                &chunks,
                                index,
                            )
                            .unwrap();
                            let file = format!("{id}-{label}-{index}.rgb");
                            std::fs::write(output.join(&file), &crop.data).unwrap();
                            parts.push(serde_json::json!({"index":index,"x_range":[c.x0,c.x1],"space_before":c.space_before,"width":crop.width,"height":crop.height,"file":file}));
                        }
                        models.push(serde_json::json!({"model":label,"chunks":parts,"error":null}));
                    }
                    Err(error) => {
                        models.push(serde_json::json!({"model":label,"chunks":[],"error":error}))
                    }
                }
            }
            assert_eq!(rgb, before);
            let mut record = serde_json::json!({"id":id,"width":prepared.width,"height":prepared.height,"models":models});
            if let Some(threshold) = threshold {
                record["threshold"] = serde_json::json!(threshold);
            }
            records.push(record);
        }
        std::fs::write(
            output.join("records.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        println!("9 original rotations and support masks; existing splits; no NN");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn recognition_retains_every_rgb_value_for_varied_dimensions_and_palettes(
            width in 1usize..48,
            height in 1usize..160,
            palette in proptest::collection::vec(any::<[u8; 3]>(), 1..17),
        ) {
            let rgb: Vec<_> = (0..width * height)
                .flat_map(|i| palette[i % palette.len()])
                .collect();
            let before = rgb.clone();
            let otsu = prepare_otsu(&rgb, width, height);
            let levels: std::collections::BTreeSet<_> = rgb.as_chunks::<3>().0.iter()
                .map(|p| p.iter().map(|&v| u16::from(v)).sum::<u16>() / 3)
                .collect();
            prop_assert_eq!(otsu.is_some(), levels.len() > 1);
            let mut choices = vec![prepare(&rgb, width, height).unwrap()];
            if let Some((prepared, _)) = otsu {
                choices.push(prepared);
            }
            for prepared in choices {
                let mut expected = rgb.as_chunks::<3>().0.to_vec();
                let mut actual = prepared.original.as_chunks::<3>().0.to_vec();
                expected.sort_unstable();
                actual.sort_unstable();
                prop_assert_eq!(actual, expected);
                prop_assert_eq!((prepared.width, prepared.height), (height, width));
                prop_assert_eq!(prepared.support.len(), prepared.original.len());
            }
            prop_assert_eq!(rgb, before);
        }
    }

    #[test]
    fn otsu_support_keeps_the_lossless_original_and_exposes_the_threshold() {
        let rgb = [63, 64, 65, 199, 200, 201];
        let (prepared, threshold) = prepare_otsu(&rgb, 2, 1).unwrap();
        assert_eq!(threshold, 64);
        assert_eq!((prepared.width, prepared.height), (1, 2));
        assert_eq!(prepared.original, [199, 200, 201, 63, 64, 65]);
        assert_eq!(prepared.support, [255, 255, 255, 0, 0, 0]);
        assert!(prepare_otsu(&[240, 220, 170], 1, 1).is_none());
        assert!(prepare_otsu(&[0, 0], 1, 1).is_none());
        assert!(prepare_otsu(&[], 0, 1).is_none());
    }

    #[test]
    fn support_can_whiten_warm_paper_without_recoloring_the_recognition_pixels() {
        let (width, height) = (24, 43 * 22 + 4);
        let paper = [240, 220, 170];
        let mut rgb: Vec<u8> = (0..width * height).flat_map(|_| paper).collect();
        for glyph in 0..43 {
            for y in 3 + glyph * 22..17 + glyph * 22 {
                for x in 5..18 {
                    rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[0, 0, 0]);
                }
            }
        }
        let before = rgb.clone();
        let prepared = prepare(&rgb, width, height).unwrap();
        assert_eq!(rgb, before);
        assert_eq!((prepared.width, prepared.height), (height, width));
        assert_eq!(prepared.original.len(), rgb.len());
        assert_eq!(prepared.support.len(), rgb.len());
        // Independent coordinate mapping for a lossless counterclockwise turn.
        for y in 0..height {
            for x in 0..width {
                let from = (y * width + x) * 3;
                let to = ((width - 1 - x) * height + y) * 3;
                assert_eq!(&prepared.original[to..to + 3], &rgb[from..from + 3]);
            }
        }
        assert_eq!(&prepared.original[..3], &paper);
        assert_eq!(&prepared.support[..3], &[255, 255, 255]);
        assert_ne!(prepared.original, prepared.support);
    }
}
