use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn body_closer_boundaries_cover_both_cascade_thresholds_before_ocr() {
    let root = Path::new("tests/fixtures/body_closer_boundary");
    let m: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).expect("frozen 25/45 boundary corpus"),
    )
    .unwrap();
    let mut zip =
        zip::ZipArchive::new(std::fs::File::open(root.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(zip.len(), 64);
    let rows = m["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 32);
    let mut seen = BTreeSet::new();
    let mut members = BTreeSet::new();
    for x in rows {
        let n = x["body_len"].as_u64().unwrap() as usize;
        assert!([23, 24, 43, 44].contains(&n));
        let close = x["closer"].as_str().unwrap();
        assert!(["」", "』"].contains(&close));
        let font = x["font"].as_str().unwrap();
        assert!(["gothic", "mincho"].contains(&font));
        let fmt = x["format"].as_str().unwrap();
        assert!(["png", "jpeg75"].contains(&fmt));
        assert!(seen.insert((n, close, font, fmt)));
        let body: String = (0..n).map(|i| (b'A' + (i % 26) as u8) as char).collect();
        let expected = format!("{body}{close}");
        assert_eq!(x["truth"].as_str().unwrap(), expected);
        assert_eq!(expected.chars().count(), n + 1);
        for key in ["image_member", "truth_member"] {
            let name = x[key].as_str().unwrap();
            assert!(members.insert(name));
            assert!(!name.starts_with('/') && !name.split('/').any(|p| p == ".."));
            let mut bytes = Vec::new();
            zip.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&bytes).unwrap(), expected);
            } else {
                let im = image::load_from_memory(&bytes).unwrap();
                assert_eq!(
                    (im.width() as u64, im.height() as u64),
                    (x["width"].as_u64().unwrap(), x["height"].as_u64().unwrap())
                );
            }
        }
    }
    assert_eq!(members.len(), 64);
}
