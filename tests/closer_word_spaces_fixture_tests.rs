use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn closer_word_spaces_fixture_pins_explicit_spaces_and_unspaced_controls_before_ocr() {
    let root = Path::new("tests/fixtures/closer_word_spaces");
    let m: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).expect("frozen closer/word spacing corpus"),
    )
    .unwrap();
    let mut zip =
        zip::ZipArchive::new(std::fs::File::open(root.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(zip.len(), 160);
    let rows = m["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 80);
    let mut combinations = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut literals = BTreeSet::new();
    for x in rows {
        let close = x["closer"].as_str().unwrap();
        assert!([")", "）", "」", "』", "】"].contains(&close));
        let prefix = x["prefix"].as_bool().unwrap();
        let spaced = x["spaced"].as_bool().unwrap();
        let gap = if spaced { " " } else { "" };
        let expected = if prefix {
            format!("{close}{gap}ABC")
        } else {
            format!("ABC{gap}{close}")
        };
        assert_eq!(x["truth"].as_str().unwrap(), expected);
        assert_eq!(
            expected.chars().filter(|c| *c == ' ').count(),
            usize::from(spaced)
        );
        let font = x["font"].as_str().unwrap();
        let fmt = x["format"].as_str().unwrap();
        assert!(["gothic", "mincho"].contains(&font));
        assert!(["png", "jpeg75"].contains(&fmt));
        assert!(combinations.insert((close, prefix, spaced, font, fmt)));
        literals.insert(expected.clone());
        for key in ["image_member", "truth_member"] {
            let name = x[key].as_str().unwrap();
            assert!(!name.starts_with('/') && !name.split('/').any(|p| p == ".."));
            assert!(members.insert(name));
            let mut b = Vec::new();
            zip.by_name(name).unwrap().read_to_end(&mut b).unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&b).unwrap(), expected);
            } else {
                let im = image::load_from_memory(&b).unwrap();
                assert_eq!(
                    (im.width() as u64, im.height() as u64),
                    (x["width"].as_u64().unwrap(), x["height"].as_u64().unwrap())
                );
            }
        }
    }
    assert_eq!(members.len(), 160);
    assert_eq!(literals.len(), 20);
}
