//! 固定した実NN出力を使う選択/画像支持のreplay。新規NN推論とは区別する。
use super::select;
use crate::infer::parseq::{ParseqOrientation, RecognizeResult};
use crate::infer::visible_repetitions::{
    sanitize_visible_repetitions, sanitize_visible_single_line,
};
use std::{collections::HashMap, path::Path};

fn recognition(v: &serde_json::Value) -> RecognizeResult {
    RecognizeResult {
        text: v["text"].as_str().unwrap().to_owned(),
        mean_confidence: v["mean_confidence"].as_f64().unwrap() as f32,
    }
}

#[test]
#[ignore = "diagnostic; requires NDLOCR_ADVANCE_REPLAY_ROOT with frozen real neural outputs"]
fn replay_fixed_neural_results() {
    let Ok(root) = std::env::var("NDLOCR_ADVANCE_REPLAY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let manifest = read("evaluation-manifest.json");
    let inputs = manifest["inputs"].as_array().unwrap();
    let mut images = HashMap::new();
    for x in inputs {
        let id = x["id"].as_str().unwrap();
        assert!(
            !id.is_empty()
                && id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
        );
        let w = x["width"].as_u64().unwrap() as usize;
        let h = x["height"].as_u64().unwrap() as usize;
        assert!(
            h as u128 * 5 <= w as u128 * 4,
            "the frozen Horizontal outputs must also be the original Auto inputs"
        );
        let rgb = std::fs::read(x["rgb"].as_str().unwrap()).unwrap();
        assert_eq!(rgb.len(), w * h * 3);
        assert!(images.insert(id.to_owned(), (rgb, w, h)).is_none());
    }
    let expected = read("decisions-advance.json");
    let expected: HashMap<_, _> = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            (
                (
                    x["id"].as_str().unwrap().to_owned(),
                    x["model"].as_str().unwrap().to_owned(),
                ),
                x["selected"].as_bool().unwrap(),
            )
        })
        .collect();
    let native = read("native-rows.json");
    let native = native.as_array().unwrap();
    assert_eq!(native.len(), images.len() * 3);
    assert_eq!(expected.len(), native.len());
    let output = root.join("production-replay");
    std::fs::create_dir(&output).expect("new replay directory required");
    let mut records = Vec::new();
    for row in native {
        let id = row["id"].as_str().unwrap();
        let model = row["model"].as_str().unwrap();
        let dims = match model {
            "30" => (256, 24),
            "50" => (384, 24),
            "100" => (768, 24),
            _ => panic!("unknown pinned model"),
        };
        let (rgb, w, h) = &images[id];
        let (w, h) = (*w, *h);
        let raw = recognition(&row["base"]);
        let before = raw.clone();
        let candidates: Vec<_> = row["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(recognition)
            .collect();
        assert_eq!(candidates.len(), 3);
        let mut calls = 0;
        let selected = select(
            &raw,
            rgb,
            w,
            h,
            dims,
            ParseqOrientation::Auto,
            |p, pw, ph| {
                let yp = calls;
                calls += 1;
                assert!(calls <= 3);
                assert_eq!((pw, ph), (w + 4, h + yp * 2));
                for y in 0..ph {
                    let line = &p[y * pw * 3..(y + 1) * pw * 3];
                    if (yp..h + yp).contains(&y) {
                        assert!(
                            line[..6]
                                .iter()
                                .chain(&line[(w + 2) * 3..])
                                .all(|&v| v == 255)
                        );
                        assert_eq!(
                            &line[6..(w + 2) * 3],
                            &rgb[(y - yp) * w * 3..(y - yp + 1) * w * 3]
                        );
                    } else {
                        assert!(line.iter().all(|&v| v == 255));
                    }
                }
                Ok(candidates[yp].clone())
            },
        );
        let expected_selected = expected[&(id.to_owned(), model.to_owned())]
            && (w + 4) as u128 * dims.1 as u128 <= h as u128 * dims.0 as u128;
        assert_eq!(selected.is_some(), expected_selected, "{model}/{id}");
        assert_eq!(raw.text, before.text);
        assert_eq!(raw.mean_confidence, before.mean_confidence);
        if let Some(ref x) = selected {
            assert_eq!(calls, 3);
            assert_eq!(x.text, candidates[0].text);
            assert_eq!(x.mean_confidence, candidates[0].mean_confidence);
        }
        let result = selected.as_ref().unwrap_or(&raw);
        let mut policies = serde_json::Map::new();
        for policy in ["single", "fragment"] {
            let sanitize = if policy == "single" {
                sanitize_visible_single_line
            } else {
                sanitize_visible_repetitions
            };
            let old = sanitize(&raw.text, rgb, w, h, ParseqOrientation::Auto);
            let new = sanitize(&result.text, rgb, w, h, ParseqOrientation::Auto);
            for (phase, text) in [("before", &old), ("after", &new)] {
                let dir = output.join(model).join(policy).join(phase);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(format!("{id}.txt")), text).unwrap();
            }
            policies.insert(
                policy.to_owned(),
                serde_json::json!({"before":old,"after":new}),
            );
        }
        records.push(serde_json::json!({"id":id,"model":model,"selected":selected.is_some(),"calls":calls,
            "raw":{"text":raw.text,"mean_confidence":raw.mean_confidence},
            "result":{"text":result.text,"mean_confidence":result.mean_confidence},"policies":policies}));
    }
    std::fs::write(
        output.join("records.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    println!(
        "{} fixed neural pairs replayed through production selector and both policies",
        records.len()
    );
}
