use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn cell_aware_spacing_fixture_pins_wide_mixed_and_ascii_bodies_before_ocr() {
    let r = Path::new("tests/fixtures/cell_aware_closer_spaces");
    let m: serde_json::Value = serde_json::from_slice(
        &std::fs::read(r.join("manifest.json"))
            .expect("freeze wide-cell spacing controls before candidate evaluation"),
    )
    .unwrap();
    let rows = m["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 320);
    let mut z = zip::ZipArchive::new(std::fs::File::open(r.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(z.len(), 640);
    let mut seen = BTreeSet::new();
    for x in rows {
        assert!(seen.insert(x["id"].as_str().unwrap()));
        let body = x["body"].as_str().unwrap();
        let c = x["closer"].as_str().unwrap();
        assert!(["経路図", "図画", "回B図", "UVW"].contains(&body));
        assert!([")", "）", "」", "』", "】"].contains(&c));
        assert!(["gothic", "mincho"].contains(&x["font"].as_str().unwrap()));
        assert_eq!(x["size"], 40);
        assert!(["png", "jpeg75"].contains(&x["format"].as_str().unwrap()));
        let gap = if x["spaced"].as_bool().unwrap() {
            " "
        } else {
            ""
        };
        let truth = if x["prefix"].as_bool().unwrap() {
            format!("{c}{gap}{body}")
        } else {
            format!("{body}{gap}{c}")
        };
        assert_eq!(x["truth"].as_str().unwrap(), truth);
        for key in ["image_member", "truth_member"] {
            let mut b = Vec::new();
            z.by_name(x[key].as_str().unwrap())
                .unwrap()
                .read_to_end(&mut b)
                .unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&b).unwrap(), truth);
            } else {
                let im = image::load_from_memory(&b).unwrap();
                assert_eq!(
                    (im.width() as u64, im.height() as u64),
                    (x["width"].as_u64().unwrap(), x["height"].as_u64().unwrap())
                );
                assert!(im.width() >= im.height() * 2);
            }
        }
    }
}
