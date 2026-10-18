#![cfg(feature = "onnx")]

/// 全20固定画像の、既定cropと明示的な四辺成分回復の生認識を保存する。
#[test]
#[ignore = "diagnostic; pinned DEIM, both PARSeq models and output directory required"]
fn compare_completed_components_on_frozen_images() {
    use ndlocr_lite_rs::{
        app::prepare_line_crops,
        infer::{cached::ParseqSession, deim_cached::DeimSession},
        io::load_rgb_u8,
        pipeline::{
            clipped_components::complete_clipped_band_components,
            crop::{BBox, crop_rgb_u8},
        },
    };
    let output = std::path::PathBuf::from(
        std::env::var_os("NDLOCR_COMPONENT_PROBE_DIR").expect("output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    let deim = DeimSession::load(std::path::Path::new("models/deim-s-1024x1024.onnx")).unwrap();
    let models: Vec<_> = [
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ]
    .into_iter()
    .map(|name| {
        (
            name,
            ParseqSession::load(
                &std::path::Path::new("models").join(name),
                std::path::Path::new("models/NDLmoji.yaml"),
            )
            .unwrap(),
        )
    })
    .collect();
    let mut paths = Vec::new();
    for directory in [
        "tests/fixtures/eval/images",
        "docs/benchmarks/deim-edge/quality/images",
    ] {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("png" | "jpg")
            ) {
                paths.push(path);
            }
        }
    }
    paths.sort();
    assert_eq!(paths.len(), 20);
    let mut pages = Vec::new();
    for path in paths {
        let image = load_rgb_u8(&path).unwrap();
        let detections = deim
            .detect_rgb_u8(&image.data, image.width, image.height, 0.25)
            .unwrap();
        let crops =
            prepare_line_crops(&image.data, image.width, image.height, detections, 0).unwrap();
        let original: Vec<_> = crops
            .iter()
            .map(|c| c.bbox_xyxy.map(|v| usize::try_from(v).unwrap()))
            .collect();
        let completed = complete_clipped_band_components(
            &image.data,
            image.width,
            image.height,
            &original,
            220,
            24,
        )
        .unwrap();
        assert_eq!(original.len(), completed.len());
        let mut changed = Vec::new();
        for (index, (&before, &after)) in original.iter().zip(&completed).enumerate() {
            if before == after {
                continue;
            }
            assert!(
                after[0] <= before[0]
                    && after[1] <= before[1]
                    && after[2] >= before[2]
                    && after[3] >= before[3]
            );
            let mut results = Vec::new();
            for (name, model) in &models {
                let mut outputs = Vec::new();
                for bounds in [before, after] {
                    let [x0, y0, x1, y1] = bounds;
                    let crop = crop_rgb_u8(
                        &image.data,
                        image.width,
                        image.height,
                        BBox::new(x0, y0, x1, y1),
                    )
                    .unwrap();
                    let result = model
                        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                        .unwrap();
                    outputs.push(
                        serde_json::json!({"text":result.text,"confidence":result.mean_confidence}),
                    );
                }
                results
                    .push(serde_json::json!({"model":name,"before":outputs[0],"after":outputs[1]}));
            }
            changed.push(
                serde_json::json!({"index":index,"before":before,"after":after,"models":results}),
            );
        }
        eprintln!(
            "{}: {}/{} changed crops",
            path.display(),
            changed.len(),
            original.len()
        );
        pages.push(
            serde_json::json!({"image":path,"total_regions":original.len(),"changed":changed}),
        );
    }
    std::fs::write(
        output.join("results.json"),
        serde_json::to_vec_pretty(&pages).unwrap(),
    )
    .unwrap();
}
