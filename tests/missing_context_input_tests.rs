#[test]
#[ignore = "diagnostic; requires frozen raw probe and explicit manifest"]
fn frozen_context_crops_equal_original_rgb_with_the_same_rust_decoder() {
    use ndlocr_lite_rs::{
        io::load_rgb_u8,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    use std::{collections::BTreeSet, path::Path};
    let Ok(manifest_path) = std::env::var("NDLOCR_CONTEXT_INPUT_MANIFEST") else {
        return;
    };
    let probe_path = std::env::var("NDLOCR_CONTEXT_INPUT_PROBE").unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest_path).unwrap()).unwrap();
    let probe: serde_json::Value =
        serde_json::from_slice(&std::fs::read(probe_path).unwrap()).unwrap();
    let mut seen = BTreeSet::new();
    for model in probe["models"].as_array().unwrap() {
        for sample in model["images"].as_array().unwrap() {
            let source = manifest["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == sample["id"])
                .unwrap();
            let original = load_rgb_u8(Path::new(source["image"].as_str().unwrap())).unwrap();
            for input in sample["inputs"].as_array().unwrap() {
                let path = input["png"].as_str().unwrap();
                if !seen.insert(path.to_string()) {
                    continue;
                }
                let image = load_rgb_u8(Path::new(path)).unwrap();
                let bounds: [usize; 4] =
                    std::array::from_fn(|i| input["bbox"][i].as_u64().unwrap() as usize);
                let crop = crop_rgb_u8(
                    &original.data,
                    original.width,
                    original.height,
                    BBox::new(bounds[0], bounds[1], bounds[2], bounds[3]),
                )
                .unwrap();
                assert_eq!(
                    (crop.width, crop.height),
                    (image.width, image.height),
                    "{path}"
                );
                assert_eq!(crop.data, image.data, "{path}");
            }
        }
    }
    assert_eq!(seen.len(), 342);
}
