use std::{collections::BTreeSet, io::Read, path::Path};
fn fixture() -> (serde_json::Value, zip::ZipArchive<std::fs::File>) {
    let root = Path::new("tests/fixtures/repeated_glyph");
    let manifest = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).expect("frozen repeated-glyph manifest"),
    )
    .unwrap();
    let archive = zip::ZipArchive::new(
        std::fs::File::open(root.join("corpus.zip")).expect("frozen repeated-glyph corpus"),
    )
    .unwrap();
    (manifest, archive)
}
#[test]
fn repeated_glyph_fixture_covers_every_frozen_combination_and_decodes() {
    let (manifest, mut archive) = fixture();
    let rows = manifest["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 160);
    assert_eq!(archive.len(), 320);
    let mut combinations = BTreeSet::new();
    let mut members = BTreeSet::new();
    for row in rows {
        let unit = row["unit"].as_str().unwrap();
        let count = row["repeat_count"].as_u64().unwrap() as usize;
        let font = row["font"].as_str().unwrap();
        let format = row["format"].as_str().unwrap();
        assert!(["1", "0", "8", "I", "l", "O", "一", "A1"].contains(&unit));
        assert!([2, 4, 6, 8, 12].contains(&count));
        assert!(["gothic", "mincho"].contains(&font));
        assert!(["png", "jpeg75"].contains(&format));
        assert!(combinations.insert((unit, count, font, format)));
        assert_eq!(row["truth"].as_str().unwrap(), unit.repeat(count));
        let mut data = Vec::new();
        for key in ["truth_member", "image_member"] {
            let member = row[key].as_str().unwrap();
            assert!(!member.starts_with('/') && !member.split('/').any(|p| p == ".."));
            assert!(members.insert(member.to_string()));
            data.clear();
            archive
                .by_name(member)
                .unwrap()
                .read_to_end(&mut data)
                .unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&data).unwrap(), unit.repeat(count));
            } else {
                let im = image::load_from_memory(&data).unwrap();
                assert_eq!(
                    (im.width() as u64, im.height() as u64),
                    (
                        row["width"].as_u64().unwrap(),
                        row["height"].as_u64().unwrap()
                    )
                );
            }
        }
    }
    assert_eq!(combinations.len(), 160);
    assert_eq!(members.len(), 320);
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires pinned models and NDLOCR_REPEAT_PROBE_OUTPUT"]
fn capture_repeated_glyph_raw_and_fragment_outputs() {
    let root =
        std::env::var("NDLOCR_REPEAT_PROBE_OUTPUT").expect("explicit diagnostic output path");
    let data = record_repeated_glyph_raw_and_fragment_outputs(Path::new(&root));
    assert_eq!(data["models"].as_array().unwrap().len(), 3);
    for model in data["models"].as_array().unwrap() {
        assert_eq!(model["images"].as_array().unwrap().len(), 184);
    }
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        Path::new(&root).join("probe.json"),
        serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "onnx")]
fn record_repeated_glyph_raw_and_fragment_outputs(root: &Path) -> serde_json::Value {
    use ndlocr_lite_rs::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{
            cached::ParseqSession,
            parseq::{ParseqOrientation, sanitize_recognized_fragment},
        },
    };
    struct Input {
        kind: String,
        data: Vec<u8>,
        width: usize,
        height: usize,
        bbox: [i32; 4],
        full_ink: bool,
        png: String,
    }
    struct Sample {
        id: String,
        cohort: &'static str,
        truth: String,
        inputs: Vec<Input>,
    }
    std::fs::create_dir_all(root.join("inputs")).unwrap();
    let (manifest, mut archive) = fixture();
    let mut sources = Vec::new();
    for row in manifest["rows"].as_array().unwrap() {
        let mut data = Vec::new();
        archive
            .by_name(row["image_member"].as_str().unwrap())
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let image = image::load_from_memory(&data).unwrap().to_rgb8();
        sources.push((
            format!("repeat_{}", row["id"].as_str().unwrap()),
            "repeated_glyph",
            row["truth"].as_str().unwrap().to_string(),
            image.width() as usize,
            image.height() as usize,
            image.into_raw(),
        ));
    }
    let old: serde_json::Value = serde_json::from_slice(
        &std::fs::read("tests/fixtures/page_band_fallback/manifest.json").unwrap(),
    )
    .unwrap();
    for row in old["rows"].as_array().unwrap() {
        let im =
            ndlocr_lite_rs::io::load_rgb_u8(Path::new(row["image"].as_str().unwrap())).unwrap();
        sources.push((
            format!("fallback_{}", row["id"].as_str().unwrap()),
            "page_band_fallback",
            row["truth"].as_str().unwrap().to_string(),
            im.width,
            im.height,
            im.data,
        ));
    }
    assert_eq!(sources.len(), 184);
    let mut samples = Vec::new();
    for (id, cohort, truth, w, h, rgb) in sources {
        let mut inputs = vec![Input {
            kind: "original".into(),
            data: rgb.clone(),
            width: w,
            height: h,
            bbox: [0, 0, w as i32, h as i32],
            full_ink: true,
            png: String::new(),
        }];
        let bands = fallback_line_detections_from_bands(&rgb, w, h, 220, 25, 45);
        let crops = prepare_line_crops(&rgb, w, h, bands, 0).unwrap();
        for (i, c) in crops.into_iter().enumerate() {
            let [x0, y0, x1, y1] = c.bbox_xyxy;
            let full_ink = (0..h).all(|y| {
                (0..w).all(|x| {
                    let inside =
                        x >= x0 as usize && x < x1 as usize && y >= y0 as usize && y < y1 as usize;
                    inside
                        || rgb[(y * w + x) * 3..(y * w + x + 1) * 3]
                            .iter()
                            .map(|&p| u16::from(p))
                            .sum::<u16>()
                            >= 660
                })
            });
            inputs.push(Input {
                kind: format!("bands-{i}"),
                data: c.crop.data,
                width: c.crop.width,
                height: c.crop.height,
                bbox: c.bbox_xyxy,
                full_ink,
                png: String::new(),
            });
        }
        for input in &mut inputs {
            let p = root.join("inputs").join(format!("{id}-{}.png", input.kind));
            image::RgbImage::from_raw(input.width as u32, input.height as u32, input.data.clone())
                .unwrap()
                .save(&p)
                .unwrap();
            input.png = p.to_string_lossy().into_owned();
        }
        samples.push(Sample {
            id,
            cohort,
            truth,
            inputs,
        });
    }
    let mut models = Vec::new();
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let session = ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let mut images = Vec::new();
        for sample in &samples {
            let mut inputs = Vec::new();
            for input in &sample.inputs {
                let rec = session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &input.data,
                        input.width,
                        input.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                inputs.push(serde_json::json!({"kind":input.kind,"size":[input.width,input.height],"bbox":input.bbox,"covers_all_220_ink":input.full_ink,"png":input.png,"raw_text":rec.text,"raw_confidence":rec.mean_confidence,"fragment_text":sanitize_recognized_fragment(&rec.text)}));
            }
            images.push(serde_json::json!({"id":sample.id,"cohort":sample.cohort,"truth":sample.truth,"inputs":inputs}));
        }
        models.push(serde_json::json!({"model":model,"input_dimensions":session.input_dimensions(),"orientation":"Horizontal","images":images}));
    }
    serde_json::json!({"models":models})
}
