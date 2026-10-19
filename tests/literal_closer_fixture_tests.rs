use std::{collections::BTreeSet, io::Read, path::Path};
#[test]
fn literal_closer_fixture_covers_frozen_positions_and_all_controls() {
    let mut expected = std::collections::BTreeMap::new();
    for (family, open, close) in [
        ("round", "(", ")"),
        ("full_round", "（", "）"),
        ("corner", "「", "」"),
        ("double_corner", "『", "』"),
        ("lenticular", "【", "】"),
    ] {
        for (body_id, body) in [("latin", "ABC"), ("digits", "123456"), ("japanese", "資料")] {
            let first = body.chars().next().unwrap();
            let tail: String = body.chars().skip(1).collect();
            for (position, text) in [
                ("prefix", format!("{close}{body}")),
                ("suffix", format!("{body}{close}")),
                ("internal", format!("{first}{close}{tail}")),
                ("balanced", format!("{open}{body}{close}")),
            ] {
                expected.insert(format!("{family}_{body_id}_{position}"), text);
            }
        }
        expected.insert(format!("{family}_alone"), close.into());
        expected.insert(format!("{family}_double"), format!("ABC{close}{close}"));
    }
    for (id, text) in [
        ("plain_latin", "ABC"),
        ("plain_digits", "123456"),
        ("plain_japanese", "資料"),
        ("square_suffix", "ABC]"),
        ("square_prefix", "]ABC"),
        ("curly_suffix", "ABC}"),
        ("curly_prefix", "}ABC"),
        ("period", "ABC."),
        ("japanese_period", "資料。"),
        ("question", "ABC?"),
    ] {
        expected.insert(format!("control_{id}"), text.into());
    }
    assert_eq!(expected.len(), 80);
    let root = Path::new("tests/fixtures/literal_closers");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).expect("frozen literal closer manifest"),
    )
    .unwrap();
    let mut archive =
        zip::ZipArchive::new(std::fs::File::open(root.join("corpus.zip")).unwrap()).unwrap();
    let rows = manifest["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 320);
    assert_eq!(archive.len(), 640);
    let mut combinations = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut cases = BTreeSet::new();
    for row in rows {
        let id = row["case"].as_str().unwrap();
        let text = row["truth"].as_str().unwrap();
        assert_eq!(text, expected[id]);
        let font = row["font"].as_str().unwrap();
        let format = row["format"].as_str().unwrap();
        assert!(["gothic", "mincho"].contains(&font));
        assert!(["png", "jpeg75"].contains(&format));
        assert!(combinations.insert((id, font, format)));
        cases.insert(id);
        for key in ["image_member", "truth_member"] {
            let name = row[key].as_str().unwrap();
            assert!(!name.starts_with('/') && !name.split('/').any(|part| part == ".."));
            assert!(members.insert(name));
            let mut bytes = Vec::new();
            archive
                .by_name(name)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            if key == "truth_member" {
                assert_eq!(std::str::from_utf8(&bytes).unwrap(), text);
            } else {
                let image = image::load_from_memory(&bytes).unwrap();
                assert_eq!(
                    (image.width() as u64, image.height() as u64),
                    (
                        row["width"].as_u64().unwrap(),
                        row["height"].as_u64().unwrap()
                    )
                );
            }
        }
    }
    assert_eq!(cases.len(), 80);
    assert_eq!(members.len(), 640);
    for family in [
        "round",
        "full_round",
        "corner",
        "double_corner",
        "lenticular",
    ] {
        for body in ["latin", "digits", "japanese"] {
            for position in ["prefix", "suffix", "internal", "balanced"] {
                assert!(cases.contains(format!("{family}_{body}_{position}").as_str()));
            }
        }
        assert!(cases.contains(format!("{family}_alone").as_str()));
        assert!(cases.contains(format!("{family}_double").as_str()));
    }
}
