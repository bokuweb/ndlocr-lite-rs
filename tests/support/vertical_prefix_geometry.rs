//! Geometry-only preflight; dummy text opens the count gate without image NN.
use super::*;

#[test]
#[ignore = "requires frozen 114-input manifest and new directory; no models or recognition"]
fn record_two_partition_geometry_without_nn() {
    let Some(path) = std::env::var_os("NDLOCR_PREFIX_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("NDLOCR_PREFIX_METADATA").unwrap()).unwrap(),
    )
    .unwrap();
    let output = std::path::PathBuf::from(std::env::var_os("NDLOCR_PREFIX_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 114);
    let mut records = Vec::new();
    let mut cuts = Vec::new();
    for input in inputs {
        let id = input["id"].as_str().unwrap();
        let rgb = std::fs::read(input["rgb"].as_str().unwrap()).unwrap();
        let dimensions = (
            input["width"].as_u64().unwrap() as usize,
            input["height"].as_u64().unwrap() as usize,
        );
        for model in metadata.as_array().unwrap() {
            let label = model["label"].as_str().unwrap();
            let dims = model["inputs"][0]["dimensions"].as_array().unwrap();
            let iw = dims[3].as_u64().unwrap() as usize;
            let ih = dims[2].as_u64().unwrap() as usize;
            let limit = model["outputs"][0]["max_raw_scalars_from_static_shape"]
                .as_u64()
                .unwrap() as usize;
            let base = RecognizeResult {
                text: "A".repeat(limit - 1),
                mean_confidence: 1.0,
            };
            let Some(plan) = plans(&base, &rgb, dimensions, (iw, ih), Some(limit)) else {
                records.push(serde_json::json!({"id":id,"model":label,"plan":false}));
                continue;
            };
            let mut routes = Vec::new();
            for (route, chunks) in plan.routes.iter().enumerate() {
                let mut parts = Vec::new();
                for (index, chunk) in chunks.iter().enumerate() {
                    let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                        &plan.column.original,
                        plan.column.width,
                        plan.column.height,
                        chunks,
                        index,
                    )
                    .unwrap();
                    let file = format!("{id}-{label}-{route}-{index}.rgb");
                    std::fs::write(output.join(&file), &crop.data).unwrap();
                    parts.push(serde_json::json!({"x_range":[chunk.x0,chunk.x1],"width":crop.width,"height":crop.height,"file":file,"space_before":chunk.space_before}));
                    if let Some(glyphs) = input["glyph_boxes"].as_array() {
                        for (glyph, box_) in glyphs.iter().enumerate() {
                            if let Some(box_) = box_.as_array() {
                                let y0 = box_[1].as_u64().unwrap() as usize;
                                let y1 = box_[3].as_u64().unwrap() as usize;
                                for boundary in [chunk.x0, chunk.x1] {
                                    if y0 < boundary && boundary < y1 {
                                        cuts.push(serde_json::json!({"id":id,"model":label,"route":route,"glyph":glyph,"bbox":box_,"boundary":boundary}));
                                    }
                                }
                            }
                        }
                    }
                }
                routes.push(parts);
            }
            records.push(serde_json::json!({"id":id,"model":label,"plan":true,"routes":routes}));
        }
    }
    std::fs::write(
        output.join("records.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    std::fs::write(
        output.join("glyph-cuts.json"),
        serde_json::to_vec_pretty(&cuts).unwrap(),
    )
    .unwrap();
    println!(
        "{} geometry pairs; {} interior glyph cuts; model loads and NN calls zero",
        records.len(),
        cuts.len()
    );
    assert!(
        cuts.is_empty(),
        "reject the whole proposal before NN, not selected inputs"
    );
}
