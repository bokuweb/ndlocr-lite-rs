use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn contained_closer_components_fixture_pins_glyphs_spaces_and_enclosure_controls() {
    let r = Path::new("tests/fixtures/contained_closer_components");
    let m: serde_json::Value = serde_json::from_slice(
        &std::fs::read(r.join("manifest.json")).expect("freeze containment controls before OCR"),
    )
    .unwrap();
    let rows = m["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 352);
    let mut zip = zip::ZipArchive::new(std::fs::File::open(r.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(zip.len(), 704);
    let mut ids = BTreeSet::new();
    let mut counts = [0; 3];
    for x in rows {
        assert!(ids.insert(x["id"].as_str().unwrap()));
        assert!(["gothic", "mincho"].contains(&x["font"].as_str().unwrap()));
        assert_eq!(x["size"], 32);
        assert!(["png", "jpeg75"].contains(&x["format"].as_str().unwrap()));
        let body = x["body"].as_str().unwrap();
        let expected = match x["kind"].as_str().unwrap() {
            "glyph" => {
                counts[0] += 1;
                assert!(["料金", "回路", "ijI", "2048"].contains(&body));
                let c = x["closer"].as_str().unwrap();
                assert!([")", "）", "」", "』", "】"].contains(&c));
                match x["position"].as_str().unwrap() {
                    "prefix" => format!("{c}{body}"),
                    "suffix" => format!("{body}{c}"),
                    "internal" => {
                        let first = body.chars().next().unwrap();
                        format!("{first}{c}{}", &body[first.len_utf8()..])
                    }
                    _ => panic!("position"),
                }
            }
            "space" => {
                counts[1] += 1;
                assert_eq!(body, "回路図");
                let c = x["closer"].as_str().unwrap();
                assert!([")", "）", "」", "』", "】"].contains(&c));
                let gap = if x["spaced"].as_bool().unwrap() {
                    " "
                } else {
                    ""
                };
                if x["prefix"].as_bool().unwrap() {
                    format!("{c}{gap}{body}")
                } else {
                    format!("{body}{gap}{c}")
                }
            }
            "enclosure" => {
                counts[2] += 1;
                assert!(["ABC", "回路"].contains(&body));
                assert!([1, 2].contains(&x["frames"].as_u64().unwrap()));
                assert!(x["outside_mark"].is_boolean());
                body.to_owned()
            }
            _ => panic!("kind"),
        };
        assert_eq!(x["truth"].as_str().unwrap(), expected);
        for key in ["image_member", "truth_member"] {
            let mut b = Vec::new();
            zip.by_name(x[key].as_str().unwrap())
                .unwrap()
                .read_to_end(&mut b)
                .unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&b).unwrap(), expected);
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
    assert_eq!(counts, [240, 80, 32]);
}
