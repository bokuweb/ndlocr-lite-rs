use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};
#[test]
fn closer_geometry_fixture_covers_new_fonts_sizes_spaces_and_nontext_controls() {
    let mut expected = BTreeMap::new();
    for (family, open, close) in [
        ("round", "(", ")"),
        ("full_round", "（", "）"),
        ("corner", "「", "」"),
        ("double_corner", "『", "』"),
        ("lenticular", "【", "】"),
    ] {
        for (body_id, body) in [("latin", "Agjp"), ("digits", "2026"), ("japanese", "情報")] {
            let first = body.chars().next().unwrap();
            let tail: String = body.chars().skip(1).collect();
            for (position, text) in [
                ("prefix", format!("{close}{body}")),
                ("suffix", format!("{body}{close}")),
                ("internal", format!("{first}{close}{tail}")),
            ] {
                expected.insert(format!("{family}_{body_id}_{position}"), text);
            }
        }
        expected.insert(format!("{family}_balanced"), format!("{open}情報{close}"));
        expected.insert(format!("{family}_alone"), close.into());
        expected.insert(format!("{family}_double"), format!("Agjp{close}{close}"));
    }
    for (i, text) in [
        "Agjp",
        "情報",
        "2026",
        "Ag jp",
        "情報 処理",
        "A.B",
        "i j",
        "j i",
        "一 二",
        "三 川",
        "Agjp.",
        "Agjp,",
        "Agjp!",
        "Agjp?",
        "情報。",
        "情報、",
        "A) gjp",
        "A） gjp",
        "A』 gjp",
        "） 情報",
    ]
    .into_iter()
    .enumerate()
    {
        expected.insert(format!("control_{i:02}"), text.into());
    }
    assert_eq!(expected.len(), 80);
    for name in [
        "horizontal",
        "vertical",
        "rectangle",
        "grid",
        "diagonal",
        "low_frame",
        "dots",
        "two_rows",
    ] {
        expected.insert(format!("figure_{name}"), String::new());
    }
    let root = Path::new("tests/fixtures/closer_geometry");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).expect("frozen geometry fixture before OCR"),
    )
    .unwrap();
    let mut archive =
        zip::ZipArchive::new(std::fs::File::open(root.join("corpus.zip")).unwrap()).unwrap();
    assert_eq!(archive.len(), 1312);
    let rows = manifest["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 656);
    let mut combinations = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut cases = BTreeSet::new();
    for row in rows {
        let case = row["case"].as_str().unwrap();
        let text = row["truth"].as_str().unwrap();
        assert_eq!(text, expected[case]);
        let font = row["font"].as_str().unwrap();
        let size = row["size"].as_u64().unwrap();
        let format = row["format"].as_str().unwrap();
        assert!(["png", "jpeg75"].contains(&format));
        if case.starts_with("figure_") {
            assert_eq!((font, size), ("none", 0));
        } else {
            assert!(["maru", "mincho"].contains(&font));
            assert!([20, 40].contains(&size));
        }
        assert!(combinations.insert((case, font, size, format)));
        cases.insert(case);
        for key in ["image_member", "truth_member"] {
            let name = row[key].as_str().unwrap();
            assert!(!name.starts_with('/') && !name.split('/').any(|p| p == ".."));
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
    assert_eq!(cases.len(), 88);
    assert_eq!(members.len(), 1312);
}
