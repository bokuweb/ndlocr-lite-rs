//! 実モデルの前後比較専用。真値を認識条件へ渡さない。
use super::*;
#[path = "../../examples/support/page_matrix_manifest.rs"]
mod manifest;

#[test]
#[ignore = "diagnostic; requires NDLOCR_LINE_MATRIX_MANIFEST and OUTPUT"]
fn capture_line_matrix() {
    let Some(input) = std::env::var_os("NDLOCR_LINE_MATRIX_MANIFEST") else {
        return;
    };
    let output = std::path::PathBuf::from(
        std::env::var_os("NDLOCR_LINE_MATRIX_OUTPUT").expect("output required"),
    );
    let model = std::path::PathBuf::from(
        std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").expect("model required"),
    );
    let charset = std::path::PathBuf::from(
        std::env::var_os("NDLOCR_TEST_CHARSET").expect("charset required"),
    );
    let rows = manifest::parse_manifest(&std::fs::read(input).unwrap()).unwrap();
    let images: Vec<_> = rows
        .iter()
        .map(|r| crate::io::load_rgb_u8(&r.image).unwrap())
        .collect();
    let items: Vec<_> = images
        .iter()
        .map(|im| (&im.data[..], im.width, im.height))
        .collect();
    std::fs::create_dir(&output).expect("new output directory required");
    let session = ParseqSession::load(&model, &charset).unwrap();
    let pool = ParseqPool::load(&model, &charset, 2).unwrap();
    let fragment = ParseqPool::load_for_fragments(&model, &charset, 2).unwrap();
    for route in [
        "raw_session",
        "session_single",
        "session_batch",
        "pool_single",
        "pool_batch",
        "pool_one_batch",
        "fragment_horizontal_batch",
        "standalone_score",
    ] {
        let mut results = Vec::new();
        for chunk in items.chunks(8) {
            let batch = match route {
                "session_batch" => session.recognize_batch_rgb_u8(chunk).unwrap(),
                "pool_batch" => pool.recognize_batch_rgb_u8(chunk).unwrap(),
                "pool_one_batch" => pool.recognize_batch_single_session_rgb_u8(chunk).unwrap(),
                "fragment_horizontal_batch" => fragment
                    .recognize_batch_single_session_horizontal_rgb_u8(chunk)
                    .unwrap(),
                _ => chunk
                    .iter()
                    .map(|&(p, w, h)| {
                        match route {
                            "raw_session" => session.recognize_rgb_u8_unprocessed(p, w, h),
                            "session_single" => session.recognize_rgb_u8(p, w, h),
                            "pool_single" => pool.recognize_rgb_u8(p, w, h),
                            "standalone_score" => {
                                super::super::parseq::recognize_rgb_u8_with_score(
                                    &model, p, w, h, &charset,
                                )
                            }
                            _ => unreachable!(),
                        }
                        .unwrap()
                    })
                    .collect(),
            };
            assert_eq!(batch.len(), chunk.len());
            results.extend(batch);
        }
        assert_eq!(results.len(), rows.len());
        let dir = output.join(route);
        std::fs::create_dir(&dir).unwrap();
        let mut records = Vec::new();
        for (row, result) in rows.iter().zip(results) {
            assert!(result.mean_confidence.is_finite());
            std::fs::write(dir.join(format!("{}.txt", row.id)), &result.text).unwrap();
            records.push(serde_json::json!({"id":row.id,"text":result.text,"mean_confidence":result.mean_confidence}));
        }
        std::fs::write(
            dir.join("results.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        println!("{route}: {} images", rows.len());
    }
}
