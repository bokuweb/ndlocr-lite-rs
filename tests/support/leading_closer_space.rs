use super::*;
fn leading_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
) -> Option<String> {
    let closer = fragment.chars().next()?;
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let suffix = &fragment[closer.len_utf8()..];
    let body = suffix.strip_prefix(' ').unwrap_or(suffix);
    if body.chars().count() < 3
        || body.chars().any(|c| c.is_whitespace() || is_closer(c))
        || boxes.len() != body.chars().count() + 1
        || boxes.iter().any(|&(x0, x1, y0, y1)| x0 >= x1 || y0 >= y1)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let body_boxes = &boxes[1..];
    let h = body_boxes.iter().map(|g| g.3 - g.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    if gaps.contains(&0) || max_gap as u128 * 4 > h as u128 {
        return None;
    }
    let close = boxes.first()?;
    let gap = body_boxes.first()?.0 - close.1;
    let threshold = (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3));
    if close.1 - close.0 < h.div_ceil(4)
        || (close.3 - close.2) as u128 > h as u128 * 2
        || gap < threshold
        || gap > h
    {
        return None;
    }
    Some(format!("{closer} {body}"))
}

fn prototype_leading_space(
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
    leading_space_from_boxes(
        &fragment,
        &super::geometry_probe::frozen_glyph_boxes(rgb, w, h),
    )
    .unwrap_or(current)
}

#[test]
fn leading_space_uses_local_body_gaps_and_closed_boundaries() {
    let row = |gap: usize| {
        vec![
            (8, 16, 14, 30),
            (16 + gap, 28 + gap, 4, 28),
            (30 + gap, 42 + gap, 4, 28),
            (44 + gap, 56 + gap, 4, 28),
        ]
    };
    for text in ["』ABC", "』 ABC"] {
        assert_eq!(
            leading_space_from_boxes(text, &row(8)),
            Some("』 ABC".into())
        );
        assert_eq!(leading_space_from_boxes(text, &row(7)), None);
        assert_eq!(
            leading_space_from_boxes(text, &row(24)),
            Some("』 ABC".into())
        );
        assert_eq!(leading_space_from_boxes(text, &row(25)), None);
    }
    for text in [
        "ABC』",
        "))ABC",
        "』AB C",
        "』  ABC",
        "』AB",
        "』\u{3000}ABC",
    ] {
        assert_eq!(leading_space_from_boxes(text, &row(10)), None);
    }
    let mut thin = row(10);
    thin[0].0 = thin[0].1 - 5;
    assert_eq!(leading_space_from_boxes("』ABC", &thin), None);
    thin[0].0 -= 1;
    assert!(leading_space_from_boxes("』ABC", &thin).is_some());
    let mut tall = row(10);
    tall[0].2 = 4;
    tall[0].3 = 52;
    assert!(leading_space_from_boxes("』ABC", &tall).is_some());
    tall[0].3 += 1;
    assert_eq!(leading_space_from_boxes("』ABC", &tall), None);
    let mut joined = row(10);
    joined[2].0 = joined[1].1;
    assert_eq!(leading_space_from_boxes("』ABC", &joined), None);
    let mut wide = row(16);
    wide[2] = (50, 62, 4, 28);
    wide[3] = (68, 80, 4, 28);
    assert!(leading_space_from_boxes("』ABC", &wide).is_some());
    wide[3].0 += 1;
    wide[3].1 += 1;
    assert_eq!(leading_space_from_boxes("』ABC", &wide), None);
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn leading_space_keeps_scale_translation_tone_and_orientation_guards(
        scale in 1usize..4,dx in 0usize..8,dy in 0usize..8,tone in 0u8..180,
        closer in prop::sample::select(vec![')','）','」','』','】'])
    ) {
        let (w,h)=(100*scale+dx,150*scale+dy);let mut rgb=vec![255;w*h*3];
        for(x0,x1,y0,y1)in [(8,16,14,30),(26,38,4,28),(40,52,4,28),(54,66,4,28)] {
            for y in y0*scale+dy..y1*scale+dy{for x in x0*scale+dx..x1*scale+dx{rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);}}
        }
        let raw=format!("{closer}ABC");let expected=format!("{closer} ABC");
        prop_assert_eq!(prototype_leading_space(&raw,&rgb,w,h,ParseqOrientation::Horizontal),expected.clone());
        prop_assert_eq!(prototype_compact_leading_space(&raw,&rgb,w,h,ParseqOrientation::Horizontal),expected.clone());
        prop_assert_eq!(sanitize_visible_single_line(&raw,&rgb,w,h,ParseqOrientation::Horizontal),expected.clone());
        prop_assert_eq!(sanitize_visible_single_line(&raw,&rgb,w,h,ParseqOrientation::Auto),"ABC");
        prop_assert_eq!(prototype_compact_leading_space(&expected,&rgb,w,h,ParseqOrientation::Horizontal),expected.clone());
        prop_assert_eq!(prototype_leading_space(&expected,&rgb,w,h,ParseqOrientation::Horizontal),expected);
        prop_assert_eq!(prototype_leading_space(&raw,&rgb,w,h,ParseqOrientation::Auto),sanitize_visible_single_line(&raw,&rgb,w,h,ParseqOrientation::Auto));
        prop_assert_eq!(prototype_leading_space(&raw,&rgb[..rgb.len()-1],w,h,ParseqOrientation::Horizontal),sanitize_visible_single_line(&raw,&rgb[..rgb.len()-1],w,h,ParseqOrientation::Horizontal));
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen leading-space root"]
fn capture_leading_space_candidate() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_LEADING_SPACE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("frozen-probe.json");
    let frozen_actual = read("frozen-after-api.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1703);
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
                assert_eq!(current, record["prototype"].as_str().unwrap());
                let actual = frozen_actual["models"][index]["actual"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == id)
                    .unwrap();
                assert_eq!(current, actual["text"].as_str().unwrap());
                assert_eq!(
                    (record["mean_confidence"].as_f64().unwrap() as f32).to_bits(),
                    (actual["mean_confidence"].as_f64().unwrap() as f32).to_bits()
                );
            }
            let prototype = prototype_leading_space(
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
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"current":current,"prototype":prototype,"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 176);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":176,"fresh_processed_forwards":176,"reused_raw":1527}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

fn compact_leading_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
) -> Option<String> {
    let spaced = leading_space_from_boxes(fragment, boxes)?;
    let h = boxes[1..].iter().map(|g| g.3 - g.2).max()?;
    (boxes[1].0 - boxes[0].0 <= h).then_some(spaced)
}
#[test]
fn compact_leading_advance_excludes_whole_glyph_width_bearing() {
    let compact_leading_space_from_boxes = |text: &str, boxes: &[(usize, usize, usize, usize)]| {
        let reference = compact_leading_space_from_boxes(text, boxes);
        assert_eq!(
            super::compact_leading_space_from_boxes(text, boxes),
            reference
        );
        reference
    };
    let row = |gap: usize| {
        vec![
            (8, 16, 14, 30),
            (16 + gap, 28 + gap, 4, 28),
            (30 + gap, 42 + gap, 4, 28),
            (44 + gap, 56 + gap, 4, 28),
        ]
    };
    assert_eq!(
        compact_leading_space_from_boxes(")ABC", &row(10)),
        Some(") ABC".into())
    );
    assert_eq!(
        compact_leading_space_from_boxes(") ABC", &row(16)),
        Some(") ABC".into())
    );
    assert_eq!(compact_leading_space_from_boxes(")ABC", &row(17)), None);
    // Whole advance27 exceeds body height22 even though gap19 looks large.
    let fullwidth = [
        (9, 17, 8, 36),
        (36, 57, 11, 33),
        (59, 76, 11, 33),
        (78, 97, 11, 33),
    ];
    assert!(leading_space_from_boxes(") ABC", &fullwidth).is_some());
    assert_eq!(compact_leading_space_from_boxes(") ABC", &fullwidth), None);
}

fn prototype_compact_leading_space(
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
    compact_leading_space_from_boxes(
        &fragment,
        &super::geometry_probe::frozen_glyph_boxes(rgb, w, h),
    )
    .unwrap_or(current)
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen leading-space root"]
fn capture_compact_leading_space_candidate() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_COMPACT_LEADING_SPACE_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("frozen-probe.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1879);
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
            let prototype = prototype_compact_leading_space(
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
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"current":current,"prototype":prototype,"unbounded":prototype_leading_space(raw,&image.data,image.width,image.height,ParseqOrientation::Auto),"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 176);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":176,"fresh_processed_forwards":176,"reused_raw":1703}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "verification; explicit frozen compact leading-space root"]
fn verify_compact_leading_space_actual_api_and_frozen_replay() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_COMPACT_LEADING_SPACE_VERIFY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let d: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("probe.json")).unwrap()).unwrap();
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let offline = std::env::var_os("NDLOCR_COMPACT_LEADING_SPACE_OFFLINE").is_some();
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
        eprintln!(
            "verified {} rows for {} ({})",
            model["rows"].as_array().unwrap().len(),
            model["label"],
            if offline { "offline" } else { "actual" }
        );
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
fn product_recovers_a_compact_leading_closer_with_one_space() {
    let (w, h) = (100, 48);
    let mut rgb = vec![255; w * h * 3];
    for (x0, x1, y0, y1) in [
        (8, 16, 14, 30),
        (26, 38, 4, 28),
        (40, 52, 4, 28),
        (54, 66, 4, 28),
    ] {
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    for raw in [")ABC", ") ABC"] {
        assert_eq!(
            sanitize_visible_single_line(raw, &rgb, w, h, ParseqOrientation::Auto),
            ") ABC"
        );
    }
    assert_eq!(sanitize_recognized_text(") ABC"), " ABC");
    assert_eq!(
        sanitize_visible_repetitions(")ABC", &rgb, w, h, ParseqOrientation::Auto),
        ")ABC"
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn leading_space_recovers_public_session_result() {
    use std::{io::Read, path::Path};
    let mut z = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/closer_word_spaces/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut b = Vec::new();
    z.by_name("images/round_prefix_space_gothic_png.png")
        .unwrap()
        .read_to_end(&mut b)
        .unwrap();
    let im = image::load_from_memory(&b).unwrap().to_rgb8();
    let s = crate::infer::cached::ParseqSession::load(
        Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx"),
        Path::new("models/NDLmoji.yaml"),
    )
    .unwrap();
    let raw = s
        .recognize_rgb_u8_unprocessed(im.as_raw(), im.width() as usize, im.height() as usize)
        .unwrap();
    assert_eq!(raw.text, ") ABC");
    let out = s
        .recognize_rgb_u8(im.as_raw(), im.width() as usize, im.height() as usize)
        .unwrap();
    assert_eq!(out.text, ") ABC");
    assert_eq!(out.mean_confidence.to_bits(), raw.mean_confidence.to_bits());
}

#[test]
fn leading_space_counts_toward_cascade_boundaries_without_repetition_shortening() {
    for count in [22, 23, 24, 42, 43, 44] {
        let (w, h) = (count * 14 + 48, 48);
        let mut rgb = vec![255; w * h * 3];
        for (x0, x1, y0, y1) in std::iter::once((8, 16, 14, 30))
            .chain((0..count).map(|i| (26 + i * 14, 38 + i * 14, 4, 28)))
        {
            for y in y0..y1 {
                for x in x0..x1 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
        let body: String = (0..count)
            .map(|i| char::from(b'A' + (i % 26) as u8))
            .collect();
        let out =
            sanitize_visible_single_line(&format!("){body}"), &rgb, w, h, ParseqOrientation::Auto);
        assert_eq!(out, format!(") {body}"));
        assert_eq!(out.chars().count(), count + 2);
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn compact_leading_space_reaches_public_paths_and_keeps_fullwidth_bearing_control() {
    use crate::infer::cached::{ParseqCascadePool, ParseqPool, ParseqSession};
    use std::{io::Read, path::Path};
    let mut z = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/closer_word_spaces/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut images = Vec::new();
    for file in [
        "round_prefix_space_gothic_png.png",
        "full_round_prefix_joined_gothic_png.png",
    ] {
        let mut b = Vec::new();
        z.by_name(&format!("images/{file}"))
            .unwrap()
            .read_to_end(&mut b)
            .unwrap();
        images.push(image::load_from_memory(&b).unwrap().to_rgb8());
    }
    let inputs: Vec<_> = images
        .iter()
        .map(|im| {
            (
                im.as_raw().as_slice(),
                im.width() as usize,
                im.height() as usize,
            )
        })
        .collect();
    let files = [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ];
    let charset = Path::new("models/NDLmoji.yaml");
    let expected = [") ABC", " ABC"];
    for file in files {
        let model = Path::new("models").join(file);
        let s = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 1).unwrap();
        let fragment = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
        for (i, &(rgb, w, h)) in inputs.iter().enumerate() {
            let saved = rgb.to_vec();
            let raw = s.recognize_rgb_u8_unprocessed(rgb, w, h).unwrap();
            assert_eq!(raw.text, ") ABC");
            let out = s.recognize_rgb_u8(rgb, w, h).unwrap();
            assert_eq!(out.text, expected[i]);
            assert_eq!(out.mean_confidence.to_bits(), raw.mean_confidence.to_bits());
            assert_eq!(pool.recognize_rgb_u8(rgb, w, h).unwrap().text, expected[i]);
            assert_eq!(
                crate::infer::parseq::recognize_rgb_u8_with_score(&model, rgb, w, h, charset)
                    .unwrap()
                    .text,
                expected[i]
            );
            assert_eq!(fragment.recognize_rgb_u8(rgb, w, h).unwrap().text, raw.text);
            assert_eq!(
                s.recognize_rgb_u8_candidates_with_orientation(
                    rgb,
                    w,
                    h,
                    ParseqOrientation::Auto,
                    2
                )
                .unwrap()
                .recognition
                .text,
                raw.text
            );
            assert_eq!(rgb, saved);
        }
        for out in [
            s.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_single_session_rgb_u8(&inputs).unwrap(),
        ] {
            assert_eq!(
                out.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                expected
            );
        }
        assert!(s.recognize_rgb_u8(&[], 2, 2).is_err());
        assert_eq!(
            s.recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            expected[0]
        );
    }
    let paths: Vec<_> = files.iter().map(|f| Path::new("models").join(f)).collect();
    let c = ParseqCascadePool::load(&paths[0], &paths[1], &paths[2], charset, 1).unwrap();
    let items: Vec<_> = [3.0, 2.0, 1.0]
        .into_iter()
        .flat_map(|bucket| {
            inputs
                .iter()
                .map(move |&(rgb, w, h)| (rgb, w, h, Some(bucket)))
        })
        .collect();
    let out = c.recognize_batch_with_buckets_rgb_u8(&items).unwrap();
    assert_eq!(
        out.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        expected.repeat(3)
    );
}
