use super::*;
fn strict_image_line(
    raw: &str,
    rgb: &[u8],
    width: usize,
    height: usize,
    orientation: ParseqOrientation,
) -> String {
    let current = sanitize_visible_with_bracket_policy(raw, rgb, width, height, orientation, true);
    if !raw
        .chars()
        .any(|c| matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return current;
    }
    let fragment =
        sanitize_visible_with_bracket_policy(raw, rgb, width, height, orientation, false);
    if fragment.chars().count() > current.chars().count()
        && !fragment.chars().any(char::is_whitespace)
        && (orientation == ParseqOrientation::Horizontal || height as u128 * 5 <= width as u128 * 4)
        && width > 0
        && height > 0
        && width.checked_mul(height).and_then(|n| n.checked_mul(3)) == Some(rgb.len())
        && supports_glyph_count(rgb, width, height, fragment.chars().count())
    {
        fragment
    } else {
        current
    }
}

#[test]
fn historical_baseline_excludes_the_new_body_fallback_and_preserves_remaining_spaces() {
    let (w, h) = (90, 48);
    let mut rgb = vec![255; w * h * 3];
    for (x0, cw, y0, ch) in [
        (8, 12, 8, 22),
        (28, 12, 8, 22),
        (48, 12, 8, 22),
        (68, 5, 17, 15),
    ] {
        for y in y0..y0 + ch {
            for x in x0..x0 + cw {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    assert_eq!(
        strict_image_line("ABC」", &rgb, w, h, ParseqOrientation::Auto),
        "ABC"
    );
    assert_eq!(
        sanitize_visible_single_line("ABC」", &rgb, w, h, ParseqOrientation::Auto),
        "ABC」"
    );
    assert_eq!(
        strict_image_line("」 ABC", &rgb, w, h, ParseqOrientation::Auto),
        " ABC"
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen word-space root"]
fn capture_closer_word_spaces_matrix() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_WORD_SPACE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let old = read("frozen-before-records.json");
    let old_after = read("frozen-after-api.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1351);
    let mut all = Vec::new();
    for (index, file) in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ]
    .iter()
    .enumerate()
    {
        let label = format!("single{}", [30, 50, 100][index]);
        let session = ParseqSession::load(
            &Path::new("models").join(file),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let mut rows = Vec::new();
        let mut fresh = 0;
        for input in inputs {
            let id = input["id"].as_str().unwrap();
            let image =
                crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
            let is_new = input["source"] == "fresh";
            let record = if is_new {
                fresh += 1;
                let d = session
                    .recognize_rgb_u8_candidates_with_orientation(
                        &image.data,
                        image.width,
                        image.height,
                        ParseqOrientation::Auto,
                        2,
                    )
                    .unwrap();
                serde_json::json!({"raw":d.recognition.text,"mean_confidence":d.recognition.mean_confidence,"stop_reason":format!("{:?}",d.stop_reason),"steps":d.steps.iter().map(|s|serde_json::json!({"timestep":s.timestep,"range":s.text_range.as_ref().map(|r|[r.start,r.end]),"candidates":s.candidates.iter().map(|c|serde_json::json!({"token_id":c.token_id,"char":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
            } else {
                old["models"][index]["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == id)
                    .unwrap()
                    .clone()
            };
            let raw = record["raw"].as_str().unwrap();
            let strict = strict_image_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let current = sanitize_visible_single_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let fragment = sanitize_visible_repetitions(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let text_only = sanitize_recognized_text(raw);
            if is_new {
                let actual = session
                    .recognize_rgb_u8(&image.data, image.width, image.height)
                    .unwrap();
                assert_eq!(actual.text, current);
                assert_eq!(
                    actual.mean_confidence.to_bits(),
                    (record["mean_confidence"].as_f64().unwrap() as f32).to_bits()
                );
            } else {
                assert_eq!(
                    strict,
                    record["baseline"].as_str().unwrap(),
                    "old strict {label}/{id}"
                );
                assert_eq!(
                    current,
                    record["closer_body"].as_str().unwrap(),
                    "old current {label}/{id}"
                );
                let actual = old_after["models"][index]["actual"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == id)
                    .unwrap();
                assert_eq!(current, actual["text"].as_str().unwrap());
                assert_eq!(fragment, record["fragment"].as_str().unwrap());
            }
            if let Some(duplicate) = input["duplicate_of"].as_str() {
                let original = inputs.iter().find(|x| x["id"] == duplicate).unwrap();
                let other =
                    crate::io::load_rgb_u8(Path::new(original["image"].as_str().unwrap())).unwrap();
                assert_eq!(image.data, other.data, "paired pixels {id}");
                let original_raw = old["models"][index]["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == duplicate)
                    .unwrap();
                assert_eq!(raw, original_raw["raw"].as_str().unwrap());
            }

            let boxes =
                super::geometry_probe::frozen_glyph_boxes(&image.data, image.width, image.height);
            let truth =
                std::fs::read_to_string(Path::new(input["truth"].as_str().unwrap())).unwrap();
            let png = root.join("decoded").join(format!("{id}.png"));
            if !png.exists() {
                std::fs::create_dir_all(png.parent().unwrap()).unwrap();
                image::RgbImage::from_raw(
                    image.width as u32,
                    image.height as u32,
                    image.data.clone(),
                )
                .unwrap()
                .save(&png)
                .unwrap();
            }
            assert_eq!(crate::io::load_rgb_u8(&png).unwrap().data, image.data);
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"closer":input["closer"],"prefix":input["prefix"],"spaced":input["spaced"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"strict":strict,"current":current,"fragment":fragment,"text_only":text_only,"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 80);
        all.push(serde_json::json!({"label":label,"model":file,"rows":rows,"fresh_raw_forwards":80,"fresh_processed_forwards":80,"reused_raw":1271}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

fn trailing_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
) -> Option<String> {
    let closer = fragment.chars().last()?;
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let prefix = &fragment[..fragment.len() - closer.len_utf8()];
    let body = prefix.strip_suffix(' ').unwrap_or(prefix);
    if body.chars().count() < 3
        || body.chars().any(|c| char::is_whitespace(c) || is_closer(c))
        || boxes.len() != body.chars().count() + 1
        || boxes.iter().any(|&(x0, x1, y0, y1)| x0 >= x1 || y0 >= y1)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let body_boxes = &boxes[..boxes.len() - 1];
    let h = body_boxes.iter().map(|g| g.3 - g.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    if gaps.contains(&0) || max_gap as u128 * 4 > h as u128 {
        return None;
    }
    let close = boxes.last()?;
    let gap = close.0 - body_boxes.last()?.1;
    let threshold = (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3));
    if close.1 - close.0 < h.div_ceil(4)
        || (close.3 - close.2) as u128 > h as u128 * 2
        || gap < threshold
        || gap > h
    {
        return None;
    }
    Some(format!("{body} {closer}"))
}

fn prototype_trailing_space(
    raw: &str,
    rgb: &[u8],
    w: usize,
    h: usize,
    orientation: ParseqOrientation,
) -> String {
    let current = sanitize_visible_single_line(raw, rgb, w, h, orientation);
    let original = sanitize_visible_with_bracket_policy(raw, rgb, w, h, orientation, true);
    let fragment = sanitize_visible_with_bracket_policy(raw, rgb, w, h, orientation, false);
    if fragment.chars().count() <= original.chars().count()
        || !(orientation == ParseqOrientation::Horizontal || h as u128 * 5 <= w as u128 * 4)
        || w == 0
        || h == 0
        || w.checked_mul(h).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return current;
    }
    let compact: String = fragment.chars().filter(|&c| c != ' ').collect();
    if !(supports_glyph_count(rgb, w, h, compact.chars().count())
        || supports_closer_body(rgb, w, h, &compact))
    {
        return current;
    }
    trailing_space_from_boxes(
        &fragment,
        &super::geometry_probe::frozen_glyph_boxes(rgb, w, h),
    )
    .unwrap_or(current)
}

#[test]
fn trailing_space_has_separated_references_and_closed_geometric_boundaries() {
    let trailing_space_from_boxes = |text: &str, boxes: &[(usize, usize, usize, usize)]| {
        let frozen = trailing_space_from_boxes(text, boxes);
        assert_eq!(super::trailing_space_from_boxes(text, boxes), frozen);
        frozen
    };
    let row = |gap: usize| {
        vec![
            (8, 20, 4, 28),
            (22, 34, 4, 28),
            (36, 48, 4, 28),
            (48 + gap, 56 + gap, 14, 30),
        ]
    };
    assert_eq!(
        trailing_space_from_boxes("ABC』", &row(8)),
        Some("ABC 』".into())
    );
    assert_eq!(
        trailing_space_from_boxes("ABC 』", &row(8)),
        Some("ABC 』".into())
    );
    assert_eq!(trailing_space_from_boxes("ABC』", &row(7)), None);
    assert_eq!(
        trailing_space_from_boxes("ABC』", &row(24)),
        Some("ABC 』".into())
    );
    assert_eq!(trailing_space_from_boxes("ABC』", &row(25)), None);
    for text in ["』ABC", "ABC))", "AB C』", "ABC  』", "AB』"] {
        assert_eq!(trailing_space_from_boxes(text, &row(10)), None);
    }
    let mut thin = row(10);
    thin[3].1 = thin[3].0 + 5;
    assert_eq!(trailing_space_from_boxes("ABC』", &thin), None);
    thin[3].1 += 1;
    assert!(trailing_space_from_boxes("ABC』", &thin).is_some());
    let mut tall = row(10);
    tall[3].2 = 4;
    tall[3].3 = 52;
    assert!(trailing_space_from_boxes("ABC』", &tall).is_some());
    tall[3].3 += 1;
    assert_eq!(trailing_space_from_boxes("ABC』", &tall), None);
    let mut wide_ref = row(16);
    wide_ref[1] = (26, 38, 4, 28);
    wide_ref[2] = (44, 56, 4, 28);
    wide_ref[3] = (72, 80, 14, 30);
    assert!(trailing_space_from_boxes("ABC』", &wide_ref).is_some());
    wide_ref[2].0 += 1;
    wide_ref[2].1 += 1;
    assert_eq!(trailing_space_from_boxes("ABC』", &wide_ref), None);
    let mut joined = row(10);
    joined[1].0 = joined[0].1;
    assert_eq!(trailing_space_from_boxes("ABC』", &joined), None);
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn trailing_space_is_invariant_to_integer_scale_translation_and_dark_tone(
        scale in 1usize..4, dx in 0usize..8, dy in 0usize..8, tone in 0u8..180,
        closer in prop::sample::select(vec![')', '）', '」', '』', '】'])
    ) {
        let (w,h)=(100*scale+dx,48*scale+dy);
        let mut rgb=vec![255;w*h*3];
        for (x0,x1,y0,y1) in [(8,20,4,28),(22,34,4,28),(36,48,4,28),(58,66,14,30)] {
            for y in y0*scale+dy..y1*scale+dy { for x in x0*scale+dx..x1*scale+dx {
                rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);
            }}
        }
        let raw=format!("ABC{closer}");
        let expected=format!("ABC {closer}");
        prop_assert_eq!(prototype_trailing_space(&raw,&rgb,w,h,ParseqOrientation::Auto),expected.clone());
        prop_assert_eq!(sanitize_visible_single_line(&raw,&rgb,w,h,ParseqOrientation::Auto),expected.clone());
        prop_assert_eq!(prototype_trailing_space(&expected,&rgb,w,h,ParseqOrientation::Auto),expected);
        prop_assert_eq!(prototype_trailing_space(&raw,&rgb[..rgb.len()-1],w,h,ParseqOrientation::Auto),sanitize_visible_single_line(&raw,&rgb[..rgb.len()-1],w,h,ParseqOrientation::Auto));
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen trailing-space root"]
fn capture_trailing_space_candidate() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_TRAILING_SPACE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("frozen-probe.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1527);
    let mut all = Vec::new();
    for (index, file) in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ]
    .iter()
    .enumerate()
    {
        let session = ParseqSession::load(
            &Path::new("models").join(file),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let mut rows = Vec::new();
        let mut fresh = 0;
        for input in inputs {
            let id = input["id"].as_str().unwrap();
            let image =
                crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
            let record = if input["source"] == "fresh_holdout" {
                fresh += 1;
                let d = session
                    .recognize_rgb_u8_candidates_with_orientation(
                        &image.data,
                        image.width,
                        image.height,
                        ParseqOrientation::Auto,
                        2,
                    )
                    .unwrap();
                serde_json::json!({"raw":d.recognition.text,"mean_confidence":d.recognition.mean_confidence,"stop_reason":format!("{:?}",d.stop_reason),"steps":d.steps.iter().map(|s|serde_json::json!({"timestep":s.timestep,"range":s.text_range.as_ref().map(|r|[r.start,r.end]),"candidates":s.candidates.iter().map(|c|serde_json::json!({"token_id":c.token_id,"char":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
            } else {
                frozen["models"][index]["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == id)
                    .unwrap()
                    .clone()
            };
            let raw = record["raw"].as_str().unwrap();
            let current = sanitize_visible_single_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            if input["source"] == "fresh_holdout" {
                let actual = session
                    .recognize_rgb_u8(&image.data, image.width, image.height)
                    .unwrap();
                assert_eq!(actual.text, current);
                assert_eq!(
                    actual.mean_confidence.to_bits(),
                    (record["mean_confidence"].as_f64().unwrap() as f32).to_bits()
                );
            } else {
                assert_eq!(current, record["current"].as_str().unwrap());
            }
            let prototype = prototype_trailing_space(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let strict = strict_image_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let fragment = sanitize_visible_repetitions(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            let boxes =
                super::geometry_probe::frozen_glyph_boxes(&image.data, image.width, image.height);
            let png = root.join("decoded").join(format!("{id}.png"));
            if !png.exists() {
                std::fs::create_dir_all(png.parent().unwrap()).unwrap();
                image::RgbImage::from_raw(
                    image.width as u32,
                    image.height as u32,
                    image.data.clone(),
                )
                .unwrap()
                .save(&png)
                .unwrap();
            }
            assert_eq!(crate::io::load_rgb_u8(&png).unwrap().data, image.data);
            let truth =
                std::fs::read_to_string(Path::new(input["truth"].as_str().unwrap())).unwrap();
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"strict":strict,"current":current,"prototype":prototype,"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 176);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":176,"fresh_processed_forwards":176,"reused_raw":1351}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "verification; explicit frozen trailing-space root"]
fn verify_trailing_space_actual_api_and_frozen_replay() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_TRAILING_SPACE_VERIFY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let d: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("probe.json")).unwrap()).unwrap();
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let offline = std::env::var_os("NDLOCR_TRAILING_SPACE_OFFLINE").is_some();
    let mut models = Vec::new();
    for model in d["models"].as_array().unwrap() {
        let session = if offline {
            None
        } else {
            Some(
                ParseqSession::load(
                    &Path::new("models").join(model["model"].as_str().unwrap()),
                    Path::new("models/NDLmoji.yaml"),
                )
                .unwrap(),
            )
        };
        let mut actual = Vec::new();
        for row in model["rows"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let input = m["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap();
            let image =
                crate::io::load_rgb_u8(&root.join("decoded").join(format!("{id}.png"))).unwrap();
            let raw = row["raw"].as_str().unwrap();
            let expected = row["prototype"].as_str().unwrap();
            let replay = sanitize_visible_single_line(
                raw,
                &image.data,
                image.width,
                image.height,
                ParseqOrientation::Auto,
            );
            assert_eq!(replay, expected, "replay/{id}");
            assert_eq!(
                sanitize_visible_repetitions(
                    raw,
                    &image.data,
                    image.width,
                    image.height,
                    ParseqOrientation::Auto
                ),
                row["fragment"].as_str().unwrap()
            );
            if let Some(ref s) = session {
                let original =
                    crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
                assert_eq!(image.data, original.data);
                let out = s
                    .recognize_rgb_u8(&original.data, original.width, original.height)
                    .unwrap();
                assert_eq!(out.text, expected, "actual/{id}");
                assert_eq!(
                    out.mean_confidence.to_bits(),
                    (row["mean_confidence"].as_f64().unwrap() as f32).to_bits()
                );
                actual.push(serde_json::json!({"id":id,"text":out.text,"mean_confidence":out.mean_confidence}));
            }
        }
        models.push(serde_json::json!({"label":model["label"],"actual":actual}));
    }
    if !offline {
        std::fs::write(
            root.join("after-api.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"models":models})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn single_line_product_keeps_the_supported_space_before_a_trailing_closer() {
    let (w, h) = (100, 48);
    let mut rgb = vec![255; w * h * 3];
    for (x0, x1, y0, y1) in [
        (8, 20, 4, 28),
        (22, 34, 4, 28),
        (36, 48, 4, 28),
        (58, 66, 14, 30),
    ] {
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    for raw in ["ABC』", "ABC 』"] {
        assert_eq!(
            sanitize_visible_single_line(raw, &rgb, w, h, ParseqOrientation::Auto),
            "ABC 』"
        );
    }
    assert_eq!(sanitize_recognized_text("ABC』"), "ABC");
    assert_eq!(
        sanitize_visible_repetitions("ABC』", &rgb, w, h, ParseqOrientation::Auto),
        "ABC』"
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn trailing_space_recovers_the_public_session_result() {
    use std::{io::Read, path::Path};
    let mut z = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/closer_word_spaces/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut b = Vec::new();
    z.by_name("images/double_corner_suffix_space_gothic_png.png")
        .unwrap()
        .read_to_end(&mut b)
        .unwrap();
    let image = image::load_from_memory(&b).unwrap().to_rgb8();
    let s = crate::infer::cached::ParseqSession::load(
        Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let raw = s
        .recognize_rgb_u8_unprocessed(
            image.as_raw(),
            image.width() as usize,
            image.height() as usize,
        )
        .unwrap();
    assert_eq!(raw.text, "ABC』");
    let out = s
        .recognize_rgb_u8(
            image.as_raw(),
            image.width() as usize,
            image.height() as usize,
        )
        .unwrap();
    assert_eq!(out.text, "ABC 』");
    assert_eq!(out.mean_confidence.to_bits(), raw.mean_confidence.to_bits());
}

#[test]
fn supported_trailing_space_counts_toward_cascade_character_boundaries() {
    for count in [22, 23, 24, 42, 43, 44] {
        let (w, h) = (count * 14 + 36, 48);
        let mut rgb = vec![255; w * h * 3];
        for i in 0..count + 1 {
            let x0 = if i == count {
                8 + (count - 1) * 14 + 12 + 10
            } else {
                8 + i * 14
            };
            let (y0, y1, cw) = if i == count { (14, 30, 8) } else { (4, 28, 12) };
            for y in y0..y1 {
                for x in x0..x0 + cw {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
        let body: String = (0..count)
            .map(|i| char::from(b'A' + (i % 26) as u8))
            .collect();
        let out =
            sanitize_visible_single_line(&format!("{body}』"), &rgb, w, h, ParseqOrientation::Auto);
        assert_eq!(out, format!("{body} 』"));
        assert_eq!(out.chars().count(), count + 2);
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn supported_trailing_space_reaches_pool_batch_stateless_and_cascade() {
    use crate::infer::cached::{ParseqCascadePool, ParseqPool, ParseqSession};
    use std::{io::Read, path::Path};
    let mut z = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/closer_word_spaces/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut b = Vec::new();
    z.by_name("images/round_suffix_space_gothic_png.png")
        .unwrap()
        .read_to_end(&mut b)
        .unwrap();
    let image = image::load_from_memory(&b).unwrap().to_rgb8();
    let (w, h) = (image.width() as usize, image.height() as usize);
    let rgb = image.as_raw();
    let saved = rgb.clone();
    let inputs = [(rgb.as_slice(), w, h)];
    let files = [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ];
    let charset = Path::new("models/NDLmoji.yaml");
    for file in files {
        let model = Path::new("models").join(file);
        let s = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 1).unwrap();
        let raw = s.recognize_rgb_u8_unprocessed(rgb, w, h).unwrap();
        assert_eq!(raw.text, "ABC)");
        let out = s.recognize_rgb_u8(rgb, w, h).unwrap();
        assert_eq!(out.text, "ABC )");
        assert_eq!(out.mean_confidence.to_bits(), raw.mean_confidence.to_bits());
        assert_eq!(
            crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, w, h, charset)
                .unwrap()
                .text,
            "ABC )"
        );
        assert_eq!(pool.recognize_rgb_u8(rgb, w, h).unwrap().text, "ABC )");
        for out in [
            s.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_single_session_rgb_u8(&inputs).unwrap(),
        ] {
            assert_eq!(out[0].text, "ABC )");
            assert_eq!(
                out[0].mean_confidence.to_bits(),
                raw.mean_confidence.to_bits()
            );
        }
        let fragment = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
        assert_eq!(fragment.recognize_rgb_u8(rgb, w, h).unwrap().text, raw.text);
        assert!(s.recognize_rgb_u8(&[], 2, 2).is_err());
        assert_eq!(s.recognize_rgb_u8(rgb, w, h).unwrap().text, "ABC )");
        assert_eq!(*rgb, saved);
    }
    let paths: Vec<_> = files.iter().map(|f| Path::new("models").join(f)).collect();
    let c = ParseqCascadePool::load(&paths[0], &paths[1], &paths[2], charset, 1).unwrap();
    let items: Vec<_> = [3.0, 2.0, 1.0]
        .into_iter()
        .map(|v| (rgb.as_slice(), w, h, Some(v)))
        .collect();
    assert!(
        c.recognize_batch_with_buckets_rgb_u8(&items)
            .unwrap()
            .iter()
            .all(|v| v.text == "ABC )")
    );
}
