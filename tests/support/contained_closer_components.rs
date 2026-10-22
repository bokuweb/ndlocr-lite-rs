use super::*;
fn maximal_component_indices(boxes: &[(usize, usize, usize, usize)]) -> Vec<usize> {
    boxes
        .iter()
        .enumerate()
        .filter(|(_, child)| {
            !boxes.iter().any(|outer| {
                outer.0 < child.0 && child.1 < outer.1 && outer.2 < child.2 && child.3 < outer.3
            })
        })
        .map(|(i, _)| i)
        .collect()
}
fn collect_contained_view(rgb: &[u8], w: usize, h: usize) -> Option<(Vec<Component>, usize)> {
    let (components, height) = collect_components(rgb, w, h)?;
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    let indices = maximal_component_indices(&boxes);
    let roots = components
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| indices.contains(&i).then_some(c))
        .collect();
    Some((roots, height))
}
fn grouped_edge_space(
    fragment: &str,
    components: &[Component],
    height: usize,
    leading: bool,
) -> Option<String> {
    let closer = if leading {
        fragment.chars().next()?
    } else {
        fragment.chars().last()?
    };
    if !matches!(closer, ')' | '）' | '」' | '』' | '】') {
        return None;
    }
    let body = if leading {
        let tail = &fragment[closer.len_utf8()..];
        tail.strip_prefix(' ').unwrap_or(tail)
    } else {
        let head = &fragment[..fragment.len() - closer.len_utf8()];
        head.strip_suffix(' ').unwrap_or(head)
    };
    if body.chars().count() < 3
        || body
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ')' | '）' | '」' | '』' | '】'))
    {
        return None;
    }
    let compact = if leading {
        format!("{closer}{body}")
    } else {
        format!("{body}{closer}")
    };
    let n = compact.chars().count();
    if !(supports_collected(components, height, n, None)
        || supports_collected(components, height, n, Some(&compact)))
    {
        return None;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    if leading {
        compact_leading_space_from_boxes(fragment, &boxes)
    } else {
        trailing_space_from_boxes(fragment, &boxes)
    }
}
fn prototype_contained_closers(
    raw: &str,
    rgb: &[u8],
    w: usize,
    h: usize,
    orientation: ParseqOrientation,
) -> String {
    let current = sanitize_visible_single_line(raw, rgb, w, h, orientation);
    let original = sanitize_visible_with_bracket_policy(raw, rgb, w, h, orientation, true);
    let fragment = sanitize_visible_with_bracket_policy(raw, rgb, w, h, orientation, false);
    if !raw
        .chars()
        .any(|c| matches!(c, ')' | '）' | '」' | '』' | '】'))
        || fragment.chars().count() <= original.chars().count()
        || !(orientation == ParseqOrientation::Horizontal || h as u128 * 5 <= w as u128 * 4)
        || w == 0
        || h == 0
        || w.checked_mul(h).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return current;
    }
    let Some((components, height)) = collect_contained_view(rgb, w, h) else {
        return current;
    };
    if let Some(spaced) = grouped_edge_space(&fragment, &components, height, false)
        .or_else(|| grouped_edge_space(&fragment, &components, height, true))
    {
        return spaced;
    }
    if !fragment.chars().any(char::is_whitespace)
        && (supports_glyph_count(rgb, w, h, fragment.chars().count())
            || supports_collected(
                &components,
                height,
                fragment.chars().count(),
                Some(&fragment),
            ))
    {
        fragment
    } else {
        current
    }
}

#[test]
fn strict_containment_removes_only_internal_components() {
    let maximal_component_indices = |boxes: &[(usize, usize, usize, usize)]| {
        let reference = maximal_component_indices(boxes);
        assert_eq!(
            super::contained_components::maximal_component_indices(boxes),
            reference
        );
        reference
    };
    let outer = (8, 32, 8, 36);
    let inner = (14, 22, 14, 20);
    let separate = (40, 60, 8, 36);
    assert_eq!(
        maximal_component_indices(&[outer, inner, separate]),
        vec![0, 2]
    );
    assert_eq!(
        maximal_component_indices(&[inner, (10, 30, 10, 32), outer]),
        vec![2]
    );
    assert_eq!(maximal_component_indices(&[]), Vec::<usize>::new());
    for touch in [
        (8, 22, 14, 20),
        (14, 32, 14, 20),
        (14, 22, 8, 20),
        (14, 22, 14, 36),
        (14, 35, 14, 20),
        (14, 22, 0, 4),
        outer,
    ] {
        assert_eq!(maximal_component_indices(&[outer, touch]), vec![0, 1]);
    }
    let partial = (24, 48, 8, 36);
    assert_eq!(
        maximal_component_indices(&[outer, partial, (27, 30, 14, 20)]),
        vec![0, 1]
    );
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn containment_is_invariant_to_component_order_scale_and_translation(scale in 1usize..4,dx in 0usize..20,dy in 0usize..20,reverse in any::<bool>()) {
        let mut boxes:Vec<_>=[(8,32,8,36),(14,22,14,20),(40,64,8,36),(47,53,16,22)].into_iter().map(|(a,b,c,d)|(a*scale+dx,b*scale+dx,c*scale+dy,d*scale+dy)).collect();
        if reverse{boxes.reverse();}let mut kept:Vec<_>=maximal_component_indices(&boxes).into_iter().map(|i|boxes[i]).collect();kept.sort();
        prop_assert_eq!(kept,vec![(8*scale+dx,32*scale+dx,8*scale+dy,36*scale+dy),(40*scale+dx,64*scale+dx,8*scale+dy,36*scale+dy)]);
    }
    #[test]
    fn contained_strokes_support_closers_without_changing_rgb_or_fragment(scale in 1usize..4,dx in 0usize..8,dy in 0usize..8,tone in 0u8..180,leading in any::<bool>()) {
        let (w,h)=(140*scale+dx,48*scale+dy);let mut rgb=vec![255;w*h*3];
        let body_start=if leading{30}else{8};
        let mut fill=|x0:usize,x1:usize,y0:usize,y1:usize|{for y in y0*scale+dy..y1*scale+dy{for x in x0*scale+dx..x1*scale+dx{rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);}}};
        for i in 0..3{let x=body_start+i*24;fill(x,x+20,8,9);fill(x,x+20,31,32);fill(x,x+1,8,32);fill(x+19,x+20,8,32);fill(x+6,x+10,15,19);}
        let close_x=if leading{8}else{body_start+2*24+20+14};fill(close_x,close_x+8,14,30);
        let original=rgb.clone();let raw=if leading{")ABC"}else{"ABC)"};let expected=if leading{") ABC"}else{"ABC )"};
        let (all,_)=collect_components(&rgb,w,h).unwrap();let (view,_)=collect_contained_view(&rgb,w,h).unwrap();prop_assert_eq!(all.len(),7);prop_assert_eq!(view.len(),4);
        prop_assert!(!supports_glyph_count(&rgb,w,h,4));prop_assert_eq!(prototype_contained_closers(raw,&rgb,w,h,ParseqOrientation::Auto),expected);
        prop_assert_eq!(sanitize_visible_repetitions(raw,&rgb,w,h,ParseqOrientation::Auto),raw);prop_assert_eq!(&rgb,&original);
    }
}

#[test]
fn contained_body_parts_can_support_a_nonspaced_two_character_body() {
    let (w, h) = (90, 48);
    let mut rgb = vec![255; w * h * 3];
    let mut fill = |x0: usize, x1: usize, y0: usize, y1: usize| {
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    };
    for x in [8, 32] {
        fill(x, x + 20, 8, 9);
        fill(x, x + 20, 31, 32);
        fill(x, x + 1, 8, 32);
        fill(x + 19, x + 20, 8, 32);
        fill(x + 6, x + 10, 15, 19);
    }
    fill(56, 64, 14, 30);
    let (original, height) = collect_components(&rgb, w, h).unwrap();
    assert!(!supports_collected(&original, height, 3, Some("AB)")));
    assert_eq!(
        prototype_contained_closers("AB)", &rgb, w, h, ParseqOrientation::Auto),
        "AB)"
    );
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen leading-space root"]
fn capture_contained_closer_components() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_CONTAINED_CLOSER_ROOT") else {
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
    assert_eq!(inputs.len(), 2231);
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
            let prototype = prototype_contained_closers(
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
            if input["source"] != "fresh_holdout" {
                assert_eq!(fragment, record["fragment"].as_str().unwrap());
            }
            let (original_components, max_height) =
                collect_components(&image.data, image.width, image.height).unwrap_or_default();
            let (contained_components, contained_height) =
                collect_contained_view(&image.data, image.width, image.height).unwrap_or_default();
            assert_eq!(max_height, contained_height);
            let compact: String = fragment.chars().filter(|c| !c.is_whitespace()).collect();
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
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"current":current,"prototype":prototype,"component_details":original_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1,c.area]).collect::<Vec<_>>(),"contained_boxes":contained_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1]).collect::<Vec<_>>(),"contained_body_support":supports_collected(&contained_components,max_height,compact.chars().count(),Some(&compact)),"legacy_support":supports_glyph_count(&image.data,image.width,image.height,fragment.chars().count()),"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 352);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":352,"fresh_processed_forwards":352,"reused_raw":1879}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "verification; explicit frozen contained closer root"]
fn verify_contained_closer_actual_api_and_replay() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_CONTAINED_CLOSER_VERIFY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let d: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("probe.json")).unwrap()).unwrap();
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let offline = std::env::var_os("NDLOCR_CONTAINED_CLOSER_OFFLINE").is_some();
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
            let original_components = collect_components(&image.data, image.width, image.height)
                .unwrap_or_default()
                .0;
            let original_details: Vec<_> = original_components
                .iter()
                .map(|c| [c.x0, c.x1, c.y0, c.y1, c.area])
                .collect();
            assert_eq!(
                serde_json::json!(original_details),
                row["component_details"]
            );
            assert_eq!(
                supports_glyph_count(
                    &image.data,
                    image.width,
                    image.height,
                    row["fragment"].as_str().unwrap().chars().count()
                ),
                row["legacy_support"].as_bool().unwrap()
            );
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

fn wide_cell_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
    leading: bool,
) -> Option<String> {
    use unicode_width::UnicodeWidthChar;
    let closer = if leading {
        fragment.chars().next()?
    } else {
        fragment.chars().last()?
    };
    if !matches!(closer, ')' | '）' | '」' | '』' | '】') {
        return None;
    }
    let body = if leading {
        let tail = &fragment[closer.len_utf8()..];
        tail.strip_prefix(' ').unwrap_or(tail)
    } else {
        let head = &fragment[..fragment.len() - closer.len_utf8()];
        head.strip_suffix(' ').unwrap_or(head)
    };
    if body.chars().count() < 2
        || body
            .chars()
            .any(|c| c.width() != Some(2) || matches!(c, ')' | '）' | '」' | '』' | '】'))
        || boxes.len() != body.chars().count() + 1
        || boxes.iter().any(|&(a, b, c, d)| a >= b || c >= d)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let (close, body_boxes) = if leading {
        (&boxes[0], &boxes[1..])
    } else {
        (boxes.last()?, &boxes[..boxes.len() - 1])
    };
    let h = body_boxes.iter().map(|b| b.3 - b.2).max()?;
    let pitches: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].0).collect();
    let max_pitch = *pitches.iter().max()?;
    let min_pitch = *pitches.iter().min()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    if body_boxes.iter().any(|b| b.1 - b.0 < h.div_ceil(2))
        || gaps.contains(&0)
        || max_gap as u128 * 4 > h as u128
        || min_pitch as u128 * 4 < h as u128 * 3
        || max_pitch as u128 * 2 > h as u128 * 3
        || max_pitch - min_pitch > h.div_ceil(4)
        || close.1 - close.0 < h.div_ceil(4)
        || (close.3 - close.2) as u128 > h as u128 * 2
    {
        return None;
    }
    let gap = if leading {
        body_boxes[0].0 - close.1
    } else {
        close.0 - body_boxes.last()?.1
    };
    let advance = body_boxes[0].0.saturating_sub(close.0);
    let compact = gap >= (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3))
        && gap <= h
        && (!leading || advance <= h);
    let wide = leading && advance >= max_pitch + h.div_ceil(4) && advance <= max_pitch + h;
    if compact || wide {
        Some(if leading {
            format!("{closer} {body}")
        } else {
            format!("{body} {closer}")
        })
    } else {
        None
    }
}

fn prototype_wide_contained_closers(
    raw: &str,
    rgb: &[u8],
    w: usize,
    h: usize,
    orientation: ParseqOrientation,
) -> String {
    let current = prototype_contained_closers(raw, rgb, w, h, orientation);
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
    let Some((components, height)) = collect_contained_view(rgb, w, h) else {
        return current;
    };
    let compact: String = fragment.chars().filter(|&c| c != ' ').collect();
    let n = compact.chars().count();
    if !(supports_collected(&components, height, n, None)
        || supports_collected(&components, height, n, Some(&compact)))
    {
        return current;
    }
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    wide_cell_space_from_boxes(&fragment, &boxes, true)
        .or_else(|| wide_cell_space_from_boxes(&fragment, &boxes, false))
        .unwrap_or(current)
}

#[test]
fn wide_cell_advance_distinguishes_a_full_cell_from_an_extra_word_space() {
    let row = |advance: usize| {
        vec![
            (8, 16, 14, 30),
            (8 + advance, 30 + advance, 8, 32),
            (34 + advance, 56 + advance, 8, 32),
            (60 + advance, 82 + advance, 8, 32),
        ]
    };
    assert_eq!(wide_cell_space_from_boxes("」回路図", &row(26), true), None);
    assert_eq!(wide_cell_space_from_boxes("」回路図", &row(31), true), None);
    for advance in [32, 50] {
        assert_eq!(
            wide_cell_space_from_boxes("」回路図", &row(advance), true),
            Some("」 回路図".into())
        );
    }
    assert_eq!(wide_cell_space_from_boxes("」回路図", &row(51), true), None);
    assert_eq!(
        wide_cell_space_from_boxes(")図面", &row(20)[..3], true),
        Some(") 図面".into())
    );
    for text in [")ABC", ")回A図", ")  回路図", ")\u{3000}回路図"] {
        assert_eq!(wide_cell_space_from_boxes(text, &row(32), true), None);
    }
    let mut variable = row(32);
    variable[3].0 += 7;
    variable[3].1 += 7;
    assert_eq!(
        wide_cell_space_from_boxes("」回路図", &variable, true),
        None
    );
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn wide_cells_support_two_and_three_glyph_word_spaces_across_scale_and_translation(scale in 1usize..4,dx in 0usize..8,dy in 0usize..8,tone in 0u8..180,leading in any::<bool>(),two in any::<bool>(),closer in prop::sample::select(vec![')','）','」','』','】'])) {
        let body=if two{"図面"}else{"回路図"};let n=body.chars().count();let(w,h)=(180*scale+dx,48*scale+dy);let mut rgb=vec![255;w*h*3];let start=if leading{42}else{8};
        let mut fill=|x0:usize,x1:usize,y0:usize,y1:usize|{for y in y0*scale+dy..y1*scale+dy{for x in x0*scale+dx..x1*scale+dx{rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);}}};
        for i in 0..n{let x=start+i*26;fill(x,x+22,8,9);fill(x,x+22,31,32);fill(x,x+1,8,32);fill(x+21,x+22,8,32);fill(x+6,x+10,15,19);}
        let cx=if leading{8}else{start+(n-1)*26+22+14};fill(cx,cx+8,14,30);
        let raw=if leading{format!("{closer}{body}")}else{format!("{body}{closer}")};let expected=if leading{format!("{closer} {body}")}else{format!("{body} {closer}")};
        prop_assert_eq!(prototype_wide_contained_closers(&raw,&rgb,w,h,ParseqOrientation::Auto),expected.clone());
        prop_assert_eq!(prototype_wide_contained_closers(&expected,&rgb,w,h,ParseqOrientation::Auto),expected);
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen leading-space root"]
fn capture_wide_contained_closer_components() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_CONTAINED_WIDE_CLOSER_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("frozen-probe.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2551);
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
            let prototype = prototype_wide_contained_closers(
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
            if input["source"] != "fresh_holdout" {
                assert_eq!(fragment, record["fragment"].as_str().unwrap());
            }
            let (original_components, max_height) =
                collect_components(&image.data, image.width, image.height).unwrap_or_default();
            let (contained_components, contained_height) =
                collect_contained_view(&image.data, image.width, image.height).unwrap_or_default();
            assert_eq!(max_height, contained_height);
            let compact: String = fragment.chars().filter(|c| !c.is_whitespace()).collect();
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
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"current":current,"prototype":prototype,"contained_only":prototype_contained_closers(raw,&image.data,image.width,image.height,ParseqOrientation::Auto),"component_details":original_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1,c.area]).collect::<Vec<_>>(),"contained_boxes":contained_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1]).collect::<Vec<_>>(),"contained_body_support":supports_collected(&contained_components,max_height,compact.chars().count(),Some(&compact)),"legacy_support":supports_glyph_count(&image.data,image.width,image.height,fragment.chars().count()),"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 320);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":320,"fresh_processed_forwards":320,"reused_raw":2231}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

fn cell_aware_space_from_boxes(
    fragment: &str,
    boxes: &[(usize, usize, usize, usize)],
    leading: bool,
) -> Option<String> {
    use unicode_width::UnicodeWidthChar;
    let closer = if leading {
        fragment.chars().next()?
    } else {
        fragment.chars().last()?
    };
    let is_closer = |c| matches!(c, ')' | '）' | '」' | '』' | '】');
    if !is_closer(closer) {
        return None;
    }
    let body = if leading {
        let tail = &fragment[closer.len_utf8()..];
        tail.strip_prefix(' ').unwrap_or(tail)
    } else {
        let head = &fragment[..fragment.len() - closer.len_utf8()];
        head.strip_suffix(' ').unwrap_or(head)
    };
    let widths: Vec<_> = body.chars().map(|c| c.width().unwrap_or(0)).collect();
    if body.chars().any(|c| c.is_whitespace() || is_closer(c))
        || widths.iter().any(|w| !matches!(w, 1 | 2))
        || widths.iter().filter(|&&w| w == 2).count() < 2
        || boxes.len() != widths.len() + 1
        || boxes.iter().any(|&(a, b, c, d)| a >= b || c >= d)
        || boxes.windows(2).any(|p| p[0].1 > p[1].0)
    {
        return None;
    }
    let (close, body_boxes) = if leading {
        (&boxes[0], &boxes[1..])
    } else {
        (boxes.last()?, &boxes[..boxes.len() - 1])
    };
    let h = body_boxes.iter().map(|b| b.3 - b.2).max()?;
    let gaps: Vec<_> = body_boxes.windows(2).map(|p| p[1].0 - p[0].1).collect();
    let max_gap = *gaps.iter().max()?;
    let wide_widths: Vec<_> = body_boxes
        .iter()
        .zip(&widths)
        .filter(|(_, w)| **w == 2)
        .map(|(b, _)| b.1 - b.0)
        .collect();
    let max_width = *wide_widths.iter().max()?;
    let min_width = *wide_widths.iter().min()?;
    let unit = max_width + max_gap;
    if gaps.contains(&0)
        || max_gap as u128 * 4 > h as u128
        || min_width < h.div_ceil(2)
        || max_width - min_width > h.div_ceil(4)
        || unit as u128 * 4 < h as u128 * 3
        || unit as u128 * 2 > h as u128 * 3
        || close.1 - close.0 < h.div_ceil(5)
        || (close.3 - close.2) as u128 > h as u128 * 2
    {
        return None;
    }
    for (p, &width) in body_boxes.windows(2).zip(&widths) {
        let pitch = p[1].0 - p[0].0;
        if (width == 2 && pitch.abs_diff(unit) > h.div_ceil(4))
            || (width == 1 && (pitch < unit.div_ceil(4) || pitch > unit + h.div_ceil(4)))
        {
            return None;
        }
    }
    let gap = if leading {
        body_boxes[0].0 - close.1
    } else {
        close.0 - body_boxes.last()?.1
    };
    let advance = body_boxes[0].0.saturating_sub(close.0);
    let narrow = gap >= (max_gap * 2 + h.div_ceil(8)).max(h.div_ceil(3))
        && gap <= h
        && (!leading || (advance <= h && advance + h.div_ceil(8) <= unit));
    let wide = leading && advance >= unit + h.div_ceil(4) && advance <= unit + h;
    if narrow || wide {
        Some(if leading {
            format!("{closer} {body}")
        } else {
            format!("{body} {closer}")
        })
    } else {
        None
    }
}
fn prototype_cell_aware_contained_closers(
    raw: &str,
    rgb: &[u8],
    w: usize,
    h: usize,
    orientation: ParseqOrientation,
) -> String {
    use unicode_width::UnicodeWidthChar;
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
    let Some((all, _)) = collect_components(rgb, w, h) else {
        return current;
    };
    let Some((components, height)) = collect_contained_view(rgb, w, h) else {
        return current;
    };
    if all.len() == components.len() {
        return current;
    }
    let compact: String = fragment.chars().filter(|&c| c != ' ').collect();
    let n = compact.chars().count();
    let support = supports_collected(&components, height, n, None)
        || supports_collected(&components, height, n, Some(&compact));
    let boxes: Vec<_> = components
        .iter()
        .map(|c| (c.x0, c.x1, c.y0, c.y1))
        .collect();
    if support
        && let Some(spaced) = cell_aware_space_from_boxes(&fragment, &boxes, true)
            .or_else(|| cell_aware_space_from_boxes(&fragment, &boxes, false))
    {
        return spaced;
    }
    let wide_count = fragment
        .chars()
        .filter(|&c| c.width() == Some(2) && !matches!(c, ')' | '）' | '」' | '』' | '】'))
        .count();
    if wide_count < 2
        && let Some(spaced) = grouped_edge_space(&fragment, &components, height, false)
            .or_else(|| grouped_edge_space(&fragment, &components, height, true))
    {
        return spaced;
    }
    if !fragment.chars().any(char::is_whitespace)
        && (supports_glyph_count(rgb, w, h, fragment.chars().count())
            || supports_collected(
                &components,
                height,
                fragment.chars().count(),
                Some(&fragment),
            ))
    {
        fragment
    } else {
        current
    }
}

#[test]
fn cell_reference_keeps_joined_cell_bearing_and_supports_mixed_width_spaces() {
    let cell_aware_space_from_boxes =
        |text: &str, boxes: &[(usize, usize, usize, usize)], leading: bool| {
            let reference = cell_aware_space_from_boxes(text, boxes, leading);
            assert_eq!(
                super::contained_components::cell_aware_space_from_boxes(text, boxes, leading),
                reference
            );
            reference
        };
    let joined = [(10, 18, 20, 36), (37, 63, 9, 36), (65, 92, 9, 36)];
    assert_eq!(cell_aware_space_from_boxes("」資料", &joined, true), None);
    let prefix = [
        (9, 16, 11, 41),
        (32, 60, 10, 39),
        (63, 94, 9, 40),
        (96, 124, 11, 39),
    ];
    assert_eq!(
        cell_aware_space_from_boxes(")回路図", &prefix, true),
        Some(") 回路図".into())
    );
    let mixed = [
        (9, 14, 10, 34),
        (26, 47, 9, 32),
        (50, 66, 13, 32),
        (68, 89, 9, 32),
    ];
    assert_eq!(
        cell_aware_space_from_boxes(")回A図", &mixed, true),
        Some(") 回A図".into())
    );
    assert_eq!(cell_aware_space_from_boxes(")XYZ", &mixed, true), None);
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn cell_aware_layout_keeps_thin_closers_mixed_widths_and_scale(scale in 1usize..4,dx in 0usize..8,dy in 0usize..8,tone in 0u8..220,leading in any::<bool>(),mixed in any::<bool>()) {
        let body=if mixed{"回A図"}else{"回路図"};let (w,h)=(180*scale+dx,48*scale+dy);let mut rgb=vec![255;w*h*3];let start=if leading{42}else{8};
        let mut fill=|x0:usize,x1:usize,y0:usize,y1:usize|{for y in y0*scale+dy..y1*scale+dy{for x in x0*scale+dx..x1*scale+dx{rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);}}};
        for i in 0..3{let x=start+i*26;fill(x,x+22,8,9);fill(x,x+22,31,32);fill(x,x+1,8,32);fill(x+21,x+22,8,32);fill(x+6,x+10,15,19);}
        let cx=if leading{8}else{start+2*26+22+14};fill(cx,cx+5,14,30);
        let raw=if leading{format!("){body}")}else{format!("{body})")};let expected=if leading{format!(") {body}")}else{format!("{body} )")};
        prop_assert_eq!(sanitize_visible_single_line(&raw,&rgb,w,h,ParseqOrientation::Auto),expected.clone());
        prop_assert_eq!(prototype_cell_aware_contained_closers(&raw,&rgb,w,h,ParseqOrientation::Auto),expected);
    }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; explicit frozen leading-space root"]
fn capture_cell_aware_contained_closer_components() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_CONTAINED_CELL_AWARE_CLOSER_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let m = read("evaluation-manifest.json");
    let frozen = read("frozen-probe.json");
    let inputs = m["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2871);
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
            let prototype = prototype_cell_aware_contained_closers(
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
            if input["source"] != "fresh_holdout" {
                assert_eq!(fragment, record["fragment"].as_str().unwrap());
            }
            let (original_components, max_height) =
                collect_components(&image.data, image.width, image.height).unwrap_or_default();
            let (contained_components, contained_height) =
                collect_contained_view(&image.data, image.width, image.height).unwrap_or_default();
            assert_eq!(max_height, contained_height);
            if original_components.len() == contained_components.len() {
                assert_eq!(prototype, current, "unchanged view/{id}");
            }
            let compact: String = fragment.chars().filter(|c| !c.is_whitespace()).collect();
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
            rows.push(serde_json::json!({"id":id,"source":input["source"],"cohort":input["cohort"],"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"current":current,"prototype":prototype,"wide_only":prototype_wide_contained_closers(raw,&image.data,image.width,image.height,ParseqOrientation::Auto),"contained_only":prototype_contained_closers(raw,&image.data,image.width,image.height,ParseqOrientation::Auto),"component_details":original_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1,c.area]).collect::<Vec<_>>(),"contained_boxes":contained_components.iter().map(|c|[c.x0,c.x1,c.y0,c.y1]).collect::<Vec<_>>(),"contained_body_support":supports_collected(&contained_components,max_height,compact.chars().count(),Some(&compact)),"legacy_support":supports_glyph_count(&image.data,image.width,image.height,fragment.chars().count()),"fragment":fragment,"text_only":sanitize_recognized_text(raw),"boxes":boxes,"width":image.width,"height":image.height}));
        }
        assert_eq!(fresh, 320);
        all.push(serde_json::json!({"label":format!("single{}",[30,50,100][index]),"model":file,"rows":rows,"fresh_raw_forwards":320,"fresh_processed_forwards":320,"reused_raw":2551}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":all})).unwrap(),
    )
    .unwrap();
}

#[test]
fn product_counts_enclosed_body_strokes_as_one_glyph_without_weakening_legacy_gate() {
    let (w, h) = (90, 48);
    let mut rgb = vec![255; w * h * 3];
    let mut fill = |x0: usize, x1: usize, y0: usize, y1: usize| {
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    };
    for x in [8, 32] {
        fill(x, x + 20, 8, 9);
        fill(x, x + 20, 31, 32);
        fill(x, x + 1, 8, 32);
        fill(x + 19, x + 20, 8, 32);
        fill(x + 6, x + 10, 15, 19);
    }
    fill(56, 64, 14, 30);
    assert!(!supports_glyph_count(&rgb, w, h, 3));
    assert_eq!(
        sanitize_visible_single_line("AB)", &rgb, w, h, ParseqOrientation::Auto),
        "AB)"
    );
    assert_eq!(
        sanitize_visible_repetitions("AB)", &rgb, w, h, ParseqOrientation::Auto),
        "AB)"
    );
    assert_eq!(sanitize_recognized_text("AB)"), "AB");
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn contained_glyph_strokes_recover_a_public_session_closer() {
    use std::{io::Read, path::Path};
    let mut z = zip::ZipArchive::new(
        std::fs::File::open("tests/fixtures/literal_closers/corpus.zip").unwrap(),
    )
    .unwrap();
    let mut b = Vec::new();
    z.by_name("images/round_japanese_suffix_gothic_png.png")
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
    assert_eq!(raw.text, "資料)");
    let out = s
        .recognize_rgb_u8(im.as_raw(), im.width() as usize, im.height() as usize)
        .unwrap();
    assert_eq!(out.text, "資料)");
    assert_eq!(out.mean_confidence.to_bits(), raw.mean_confidence.to_bits());
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn contained_and_cell_layout_support_reaches_public_paths_with_controls() {
    use crate::infer::cached::{ParseqCascadePool, ParseqPool, ParseqSession};
    use std::{io::Read, path::Path};
    let mut images = Vec::new();
    for (folder, file) in [
        ("literal_closers", "round_japanese_suffix_gothic_png.png"),
        (
            "contained_closer_components",
            "space_round_prefix_space_gothic_png.png",
        ),
        ("literal_closers", "corner_japanese_prefix_mincho_png.png"),
        (
            "contained_closer_components",
            "enclosure_latin_f2_mark1_gothic_png.png",
        ),
    ] {
        let mut z = zip::ZipArchive::new(
            std::fs::File::open(format!("tests/fixtures/{folder}/corpus.zip")).unwrap(),
        )
        .unwrap();
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
    let mut cascade_expected = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let expected = [
            "資料)",
            ") 回路図",
            if index == 0 { "」資料" } else { "「資料" },
            if index == 0 { "ABC" } else { "ABCII" },
        ];
        cascade_expected.extend(expected.iter().map(|s| s.to_string()));
        let model = Path::new("models").join(file);
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 1).unwrap();
        let fragment = ParseqPool::load_for_fragments(&model, charset, 1).unwrap();
        for (i, &(rgb, w, h)) in inputs.iter().enumerate() {
            let saved = rgb.to_vec();
            let raw = session.recognize_rgb_u8_unprocessed(rgb, w, h).unwrap();
            let out = session.recognize_rgb_u8(rgb, w, h).unwrap();
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
                session
                    .recognize_rgb_u8_candidates_with_orientation(
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
            session.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_rgb_u8(&inputs).unwrap(),
            pool.recognize_batch_single_session_rgb_u8(&inputs).unwrap(),
        ] {
            assert_eq!(
                out.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                expected
            );
        }
        assert!(session.recognize_rgb_u8(&[], 2, 2).is_err());
        assert_eq!(
            session
                .recognize_rgb_u8(inputs[0].0, inputs[0].1, inputs[0].2)
                .unwrap()
                .text,
            expected[0]
        );
    }
    let paths: Vec<_> = files.iter().map(|f| Path::new("models").join(f)).collect();
    let cascade = ParseqCascadePool::load(&paths[0], &paths[1], &paths[2], charset, 1).unwrap();
    let items: Vec<_> = [3.0, 2.0, 1.0]
        .into_iter()
        .flat_map(|bucket| {
            inputs
                .iter()
                .map(move |&(rgb, w, h)| (rgb, w, h, Some(bucket)))
        })
        .collect();
    assert_eq!(
        cascade
            .recognize_batch_with_buckets_rgb_u8(&items)
            .unwrap()
            .iter()
            .map(|r| r.text.clone())
            .collect::<Vec<_>>(),
        cascade_expected
    );
}

#[test]
fn cell_space_boundaries_separate_narrow_and_full_cell_gaps() {
    let cell_aware_space_from_boxes =
        |text: &str, boxes: &[(usize, usize, usize, usize)], leading: bool| {
            let reference = cell_aware_space_from_boxes(text, boxes, leading);
            assert_eq!(
                super::contained_components::cell_aware_space_from_boxes(text, boxes, leading),
                reference
            );
            reference
        };
    let row = |advance: usize| {
        vec![
            (8, 13, 14, 30),
            (8 + advance, 30 + advance, 8, 32),
            (34 + advance, 56 + advance, 8, 32),
        ]
    };
    for a in [16, 23, 32, 50] {
        assert_eq!(
            cell_aware_space_from_boxes(")図面", &row(a), true),
            Some(") 図面".into())
        );
    }
    for a in [15, 24, 31, 51] {
        assert_eq!(cell_aware_space_from_boxes(")図面", &row(a), true), None);
    }
    let mut thin = row(32);
    thin[0].1 -= 1;
    assert_eq!(cell_aware_space_from_boxes(")図面", &thin, true), None);
    let mut tall = row(32);
    tall[0].2 = 8;
    tall[0].3 = 56;
    assert!(cell_aware_space_from_boxes(")図面", &tall, true).is_some());
    tall[0].3 += 1;
    assert_eq!(cell_aware_space_from_boxes(")図面", &tall, true), None);
    let mut varied = row(32);
    varied[2].1 -= 6;
    assert!(cell_aware_space_from_boxes(")図面", &varied, true).is_some());
    varied[2].1 -= 1;
    assert_eq!(cell_aware_space_from_boxes(")図面", &varied, true), None);
}
