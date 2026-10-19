use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn leading_closer_space_fixture_pins_new_words_sizes_and_rule_controls() {
    let r = Path::new("tests/fixtures/leading_closer_space");
    let m: serde_json::Value = serde_json::from_slice(
        &std::fs::read(r.join("manifest.json"))
            .expect("freeze additional spacing controls before OCR"),
    )
    .unwrap();
    let mut z = zip::ZipArchive::new(std::fs::File::open(r.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(z.len(), 352);
    let rows = m["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 176);
    let mut seen = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut text_n = 0;
    let mut rule_n = 0;
    for x in rows {
        let fmt = x["format"].as_str().unwrap();
        assert!(["png", "jpeg75"].contains(&fmt));
        let expected = if x["kind"] == "text" {
            text_n += 1;
            let c = x["closer"].as_str().unwrap();
            assert!([")", "）", "」", "』", "】"].contains(&c));
            let p = x["prefix"].as_bool().unwrap();
            let s = x["spaced"].as_bool().unwrap();
            let font = x["font"].as_str().unwrap();
            let size = x["size"].as_u64().unwrap();
            assert!(["maru", "mincho"].contains(&font));
            assert!([24, 36].contains(&size));
            assert!(seen.insert(format!("{c}/{p}/{s}/{font}/{size}/{fmt}")));
            let g = if s { " " } else { "" };
            if p {
                format!("{c}{g}KLM")
            } else {
                format!("KLM{g}{c}")
            }
        } else {
            rule_n += 1;
            assert_eq!(x["kind"], "rule");
            let w = x["rule_width"].as_u64().unwrap();
            let h = x["rule_height"].as_u64().unwrap();
            let g = x["rule_gap"].as_u64().unwrap();
            assert!([1, 2].contains(&w) && [22, 66].contains(&h) && [2, 10].contains(&g));
            assert!(seen.insert(format!("rule/{w}/{h}/{g}/{fmt}")));
            "KLM".to_string()
        };
        assert_eq!(x["truth"].as_str().unwrap(), expected);
        for key in ["image_member", "truth_member"] {
            let name = x[key].as_str().unwrap();
            assert!(!name.starts_with('/') && !name.split('/').any(|p| p == ".."));
            assert!(members.insert(name));
            let mut b = Vec::new();
            z.by_name(name).unwrap().read_to_end(&mut b).unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&b).unwrap(), expected);
            } else {
                let im = image::load_from_memory(&b).unwrap();
                assert!(im.width() >= im.height() * 2);
                assert_eq!(
                    (im.width() as u64, im.height() as u64),
                    (x["width"].as_u64().unwrap(), x["height"].as_u64().unwrap())
                );
            }
        }
    }
    assert_eq!((text_n, rule_n), (160, 16));
    assert_eq!(members.len(), 352);
}
