//! Native diagnostic only: no GT is loaded and every attempted raw chunk is retained.
use crate::infer::{cached::ParseqSession, parseq::ParseqOrientation};

#[test]
#[ignore = "diagnostic; requires frozen rows/baseline raw results, model and new output"]
fn capture_capacity_candidates() {
    let Some(input) = std::env::var_os("NDLOCR_CAPACITY_CAPTURE_MANIFEST") else {
        return;
    };
    let out = std::path::PathBuf::from(std::env::var_os("NDLOCR_CAPACITY_CAPTURE_OUTPUT").unwrap());
    std::fs::create_dir(&out).unwrap();
    let model = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let runtime = ParseqSession::load(&model, &charset).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(input).unwrap()).unwrap();
    let mut rows = Vec::new();
    let mut errors = 0;
    for row in manifest["inputs"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(
            !id.is_empty()
                && id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
        );
        let rgb = std::fs::read(row["rgb"].as_str().unwrap()).unwrap();
        let w = row["width"].as_u64().unwrap() as usize;
        let h = row["height"].as_u64().unwrap() as usize;
        assert_eq!(rgb.len(), w * h * 3);
        let base = crate::infer::parseq::RecognizeResult {
            text: row["base"]["text"].as_str().unwrap().to_owned(),
            mean_confidence: f32::from_bits(row["base"]["mean_bits"].as_u64().unwrap() as u32),
        };
        assert!(base.mean_confidence.is_finite());
        let geometrical_chunks = super::prepare_otsu(&rgb, w, h)
            .and_then(|(column, _)| {
                let (iw, ih) = runtime.input_dimensions();
                crate::pipeline::line_chunks::split_rotated_vertical_column_for_input(
                    &column.support,
                    column.width,
                    column.height,
                    220,
                    iw,
                    ih,
                )
                .ok()
            })
            .unwrap_or_default();
        let mut chunks = Vec::new();
        let candidate = super::policy::recover(
            &base,
            &rgb,
            (w, h),
            runtime.input_dimensions(),
            |pixels, cw, ch| {
                let geometry = &geometrical_chunks[chunks.len()];
                assert_eq!((cw, ch), (geometry.x1 - geometry.x0, w));
                let range = [geometry.x0, geometry.x1];
                let name = format!("{id}-chunk{}.rgb", chunks.len());
                std::fs::write(out.join(&name), pixels).unwrap();
                let result = runtime.recognize_rgb_u8_unprocessed_with_orientation(
                    pixels,
                    cw,
                    ch,
                    ParseqOrientation::Horizontal,
                );
                match &result {
                Ok(read) => chunks.push(serde_json::json!({"file":name,"x_range":range,"width":cw,"height":ch,"text":read.text,"mean_confidence":read.mean_confidence,"mean_bits":read.mean_confidence.to_bits(),"error":null})),
                Err(error) => { errors+=1; chunks.push(serde_json::json!({"file":name,"x_range":range,"width":cw,"height":ch,"error":error.to_string()})); }
            }
                result
            },
        );
        let selected = candidate.as_ref().unwrap_or(&base);
        rows.push(serde_json::json!({"id":id,"base":{"text":base.text,"mean_confidence":base.mean_confidence,"mean_bits":base.mean_confidence.to_bits()},"selected":{"text":selected.text,"mean_confidence":selected.mean_confidence,"mean_bits":selected.mean_confidence.to_bits()},"accepted":candidate.is_some(),"chunks":chunks}));
        std::fs::write(out.join(format!("{id}.txt")), &selected.text).unwrap();
    }
    std::fs::write(
        out.join("results.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    println!(
        "{} inputs; {} raw Horizontal chunk API calls; {} errors",
        rows.len(),
        rows.iter()
            .map(|x| x["chunks"].as_array().unwrap().len())
            .sum::<usize>(),
        errors
    );
    assert_eq!(
        errors, 0,
        "all failed inputs and attempted chunks were retained"
    );
}
