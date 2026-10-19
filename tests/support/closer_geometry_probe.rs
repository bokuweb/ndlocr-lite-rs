//! Experimental policies only; no product caller uses these functions.
use super::*;
#[derive(Clone, Debug)]
struct Glyph {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
}
fn is_closer(c: char) -> bool {
    matches!(c, ')' | '）' | '」' | '』' | '】')
}
fn valid_glyphs(glyphs: &[Glyph], text: &str) -> bool {
    !glyphs.is_empty()
        && glyphs.len() == text.chars().count()
        && text.chars().any(is_closer)
        && !text.chars().any(char::is_whitespace)
        && glyphs.iter().all(|c| c.x1 > c.x0 && c.y1 > c.y0)
        && glyphs.windows(2).all(|p| p[0].x1 <= p[1].x0)
}
fn short_common(glyphs: &[Glyph], text: &str) -> bool {
    if !valid_glyphs(glyphs, text) {
        return false;
    }
    let max_h = glyphs.iter().map(|c| c.y1 - c.y0).max().unwrap();
    let min_h = glyphs.iter().map(|c| c.y1 - c.y0).min().unwrap();
    let top = glyphs.iter().map(|c| c.y0).max().unwrap();
    let bottom = glyphs.iter().map(|c| c.y1).min().unwrap();
    min_h as u128 * 2 >= max_h as u128
        && bottom.saturating_sub(top) as u128 * 4 >= min_h as u128 * 3
        && glyphs
            .iter()
            .all(|c| (c.x1 - c.x0) as u128 <= max_h as u128 * 2)
}
fn closer_body(glyphs: &[Glyph], text: &str) -> bool {
    if !valid_glyphs(glyphs, text) {
        return false;
    }
    let body: Vec<_> = glyphs
        .iter()
        .zip(text.chars())
        .filter(|(_, c)| !is_closer(*c))
        .map(|(g, _)| g)
        .collect();
    if body.len() < 2 {
        return false;
    }
    let body_h = body.iter().map(|c| c.y1 - c.y0).max().unwrap();
    let top = body.iter().map(|c| c.y0).max().unwrap();
    let bottom = body.iter().map(|c| c.y1).min().unwrap();
    if bottom.saturating_sub(top) as u128 * 4 < body_h as u128 * 3
        || glyphs
            .iter()
            .any(|c| (c.x1 - c.x0) as u128 > body_h as u128 * 2)
    {
        return false;
    }
    glyphs
        .iter()
        .zip(text.chars())
        .filter(|(_, c)| is_closer(*c))
        .all(|(g, _)| {
            let height = g.y1 - g.y0;
            height as u128 * 2 >= body_h as u128
                && g.y1.min(bottom).saturating_sub(g.y0.max(top)) as u128 * 4
                    >= height.min(body_h) as u128 * 3
        })
}
fn example_row() -> Vec<Glyph> {
    vec![
        Glyph {
            x0: 8,
            y0: 8,
            x1: 29,
            y1: 30,
        },
        Glyph {
            x0: 31,
            y0: 8,
            x1: 48,
            y1: 30,
        },
        Glyph {
            x0: 50,
            y0: 8,
            x1: 69,
            y1: 30,
        },
        Glyph {
            x0: 71,
            y0: 17,
            x1: 79,
            y1: 32,
        },
    ]
}
#[test]
fn short_common_can_explain_a_lower_closer_but_weakens_body_alignment() {
    let mut row = example_row();
    assert!(short_common(&row, "ABC」"));
    row[1].y0 += 6;
    row[1].y1 += 6;
    assert!(short_common(&row, "ABC」"));
}
#[test]
fn closer_body_keeps_the_body_alignment_and_checks_each_closer() {
    let mut row = example_row();
    for text in ["ABC)", "ABC）", "ABC」", "ABC』", "ABC】"] {
        assert!(closer_body(&row, text));
    }
    row[1].y0 += 6;
    row[1].y1 += 6;
    assert!(!closer_body(&row, "ABC」"));
}
#[test]
fn geometry_controls_reject_unsupported_count_spaces_overlap_small_and_separate_ink() {
    for policy in [short_common, closer_body] {
        let row = example_row();
        assert!(!policy(&row, "AB」"));
        assert!(!policy(&row, "AB 」"));
        let mut bad = row.clone();
        bad[3].x0 = 68;
        assert!(!policy(&bad, "ABC」"));
        let mut bad = row.clone();
        bad[3].y0 = 26;
        bad[3].y1 = 41;
        assert!(!policy(&bad, "ABC」"));
        let mut bad = row.clone();
        bad[3].y0 = 20;
        bad[3].y1 = 29;
        assert!(!policy(&bad, "ABC」"));
        let mut bad = row.clone();
        bad[1].y0 = 45;
        bad[1].y1 = 67;
        assert!(!policy(&bad, "ABC」"));
        assert!(!policy(&[], ""));
    }
    assert!(!closer_body(&example_row()[..2], "))"));
}

#[test]
fn geometry_thresholds_have_closed_boundaries() {
    for policy in [short_common, closer_body] {
        let base = vec![
            Glyph {
                x0: 0,
                y0: 0,
                x1: 6,
                y1: 24,
            },
            Glyph {
                x0: 10,
                y0: 0,
                x1: 16,
                y1: 24,
            },
            Glyph {
                x0: 20,
                y0: 12,
                x1: 26,
                y1: 28,
            },
        ];
        assert!(policy(&base, "AB」"));
        let mut bad = base.clone();
        bad[2].y0 += 1;
        bad[2].y1 += 1;
        assert!(!policy(&bad, "AB」"));
        let mut floor = base.clone();
        floor[2].y1 = 24;
        assert!(policy(&floor, "AB」"));
        floor[2].y0 = 13;
        assert!(!policy(&floor, "AB」"));
    }
    for (shift, expected) in [(6, true), (7, false)] {
        let row = vec![
            Glyph {
                x0: 0,
                y0: 0,
                x1: 6,
                y1: 24,
            },
            Glyph {
                x0: 10,
                y0: shift,
                x1: 16,
                y1: 24 + shift,
            },
            Glyph {
                x0: 20,
                y0: 6,
                x1: 26,
                y1: 22,
            },
        ];
        assert_eq!(closer_body(&row, "AB」"), expected);
    }
}
use proptest::prelude::*;
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn supported_lower_closers_are_invariant_under_scale_translation_and_position(n in 2usize..9, at in 0usize..10, kind in 0usize..5, scale in 1usize..5, dx in 0usize..40, dy in 0usize..40) {
        let at=at%(n+1);let close=[')','）','」','』','】'][kind];let mut text=String::new();let mut glyphs=Vec::new();
        for i in 0..=n {let is_close=i==at;text.push(if is_close{close}else{'A'});let (y0,y1,w)=if is_close{(17,32,5)}else{(8,30,12)};glyphs.push(Glyph{x0:(i*20+8)*scale+dx,x1:(i*20+8+w)*scale+dx,y0:y0*scale+dy,y1:y1*scale+dy});}
        prop_assert!(short_common(&glyphs,&text));prop_assert!(closer_body(&glyphs,&text));
        let body=(0..glyphs.len()).find(|i|*i!=at).unwrap();glyphs[body].y0+=6*scale;glyphs[body].y1+=6*scale;
        prop_assert!(!closer_body(&glyphs,&text));
    }
}
fn collect_glyphs(rgb: &[u8], width: usize, height: usize) -> Vec<Glyph> {
    if width == 0
        || height == 0
        || width.checked_mul(height).and_then(|n| n.checked_mul(3)) != Some(rgb.len())
    {
        return Vec::new();
    }
    struct Component {
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        area: usize,
    }
    let mut mask: Vec<bool> = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| p.iter().map(|&v| u16::from(v)).sum::<u16>() < 660)
        .collect();
    let mut components = Vec::new();
    for seed in 0..mask.len() {
        if !mask[seed] {
            continue;
        }
        mask[seed] = false;
        let mut pending = vec![(seed % width, seed / width)];
        let mut c = Component {
            x0: width,
            y0: height,
            x1: 0,
            y1: 0,
            area: 0,
        };
        while let Some((x, y)) = pending.pop() {
            c.x0 = c.x0.min(x);
            c.y0 = c.y0.min(y);
            c.x1 = c.x1.max(x + 1);
            c.y1 = c.y1.max(y + 1);
            c.area += 1;
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let p = ny * width + nx;
                    if mask[p] {
                        mask[p] = false;
                        pending.push((nx, ny));
                    }
                }
            }
        }
        if c.area >= 3 {
            components.push(c);
        }
    }
    let Some(body_height) = components.iter().map(|c| c.y1 - c.y0).max() else {
        return Vec::new();
    };
    let min_area = body_height.div_ceil(2).max(3);
    components.retain(|c| c.area >= min_area);
    components.sort_by_key(|c| c.x0);
    components
        .into_iter()
        .map(|c| Glyph {
            x0: c.x0,
            y0: c.y0,
            x1: c.x1,
            y1: c.y1,
        })
        .collect()
}
#[test]
fn diagnostic_collection_matches_the_existing_pixel_gate_and_ignores_small_noise() {
    let (w, h) = (120, 50);
    let mut rgb = vec![255; w * h * 3];
    for (x0, x1, y0, y1) in [
        (10, 22, 8, 30),
        (40, 52, 8, 30),
        (70, 82, 8, 30),
        (100, 105, 17, 32),
    ] {
        for y in y0..y1 {
            for x in x0..x1 {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
    }
    rgb[(42 * w + 2) * 3..(42 * w + 4) * 3].fill(0);
    let glyphs = collect_glyphs(&rgb, w, h);
    assert_eq!(glyphs.len(), 4);
    assert!(!supports_glyph_count(&rgb, w, h, 4));
    assert!(short_common(&glyphs, "ABC」"));
    assert!(closer_body(&glyphs, "ABC」"));
    assert!(collect_glyphs(&rgb[..rgb.len() - 1], w, h).is_empty());
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "diagnostic; requires frozen geometry root and pinned models"]
fn capture_closer_geometry_matrix() {
    use crate::infer::cached::ParseqSession;
    use std::path::Path;
    let Ok(root) = std::env::var("NDLOCR_GEOMETRY_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let old: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("reused-raw.json")).unwrap()).unwrap();
    let inputs = manifest["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1239);
    std::fs::create_dir_all(root.join("inputs")).unwrap();
    let mut models = Vec::new();
    for (index, (label, model)) in [
        (
            "single30",
            "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "single50",
            "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        ),
        (
            "single100",
            "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
        ),
    ]
    .iter()
    .enumerate()
    {
        let session = ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let old_model = &old["models"][index];
        assert_eq!(old_model["label"].as_str().unwrap(), *label);
        let mut rows = Vec::new();
        let mut fresh = 0;
        for input in inputs {
            let id = input["id"].as_str().unwrap();
            let source =
                crate::io::load_rgb_u8(Path::new(input["image"].as_str().unwrap())).unwrap();
            let reused = input["raw_source"] == "reused_raw";
            let record = if reused {
                old_model["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["id"] == id)
                    .unwrap()
                    .clone()
            } else {
                fresh += 1;
                let out = session
                    .recognize_rgb_u8_candidates_with_orientation(
                        &source.data,
                        source.width,
                        source.height,
                        ParseqOrientation::Auto,
                        2,
                    )
                    .unwrap();
                serde_json::json!({"raw":out.recognition.text,"mean_confidence":out.recognition.mean_confidence,"stop_reason":format!("{:?}",out.stop_reason),"steps":out.steps.iter().map(|s|serde_json::json!({"timestep":s.timestep,"range":s.text_range.as_ref().map(|r|[r.start,r.end]),"candidates":s.candidates.iter().map(|c|serde_json::json!({"token_id":c.token_id,"char":c.character,"probability":c.probability})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
            };
            let raw = record["raw"].as_str().unwrap();
            let baseline = sanitize_visible_single_line(
                raw,
                &source.data,
                source.width,
                source.height,
                ParseqOrientation::Auto,
            );
            let fragment = sanitize_visible_repetitions(
                raw,
                &source.data,
                source.width,
                source.height,
                ParseqOrientation::Auto,
            );
            if reused {
                let expected = std::fs::read_to_string(
                    root.join("expected").join(label).join(format!("{id}.txt")),
                )
                .unwrap();
                assert_eq!(baseline, expected, "{label}/{id}");
                assert_eq!(fragment, record["fragment"].as_str().unwrap());
            }
            let glyphs = collect_glyphs(&source.data, source.width, source.height);
            let count = fragment.chars().count();
            let strict = supports_glyph_count(&source.data, source.width, source.height, count);
            let replay_strict = if glyphs.len() != count || glyphs.is_empty() {
                false
            } else {
                let max_h = glyphs.iter().map(|g| g.y1 - g.y0).max().unwrap();
                let top = glyphs.iter().map(|g| g.y0).max().unwrap();
                let bottom = glyphs.iter().map(|g| g.y1).min().unwrap();
                bottom.saturating_sub(top) as u128 * 4 >= max_h as u128 * 3
                    && glyphs
                        .iter()
                        .all(|g| (g.x1 - g.x0) as u128 <= max_h as u128 * 2)
                    && glyphs.windows(2).all(|p| p[0].x1 <= p[1].x0)
            };
            assert_eq!(
                strict, replay_strict,
                "component collection parity: {label}/{id}"
            );
            if reused {
                assert_eq!(strict, record["count_supported"].as_bool().unwrap());
            }
            let eligible = fragment.chars().count() > baseline.chars().count()
                && !fragment.chars().any(char::is_whitespace)
                && source.height as u128 * 5 <= source.width as u128 * 4;
            let short = short_common(&glyphs, &fragment);
            let body = closer_body(&glyphs, &fragment);
            let choose = |supported: bool| {
                if eligible && supported {
                    fragment.clone()
                } else {
                    baseline.clone()
                }
            };
            let png = root.join("inputs").join(format!("{id}.png"));
            if !png.exists() {
                image::RgbImage::from_raw(
                    source.width as u32,
                    source.height as u32,
                    source.data.clone(),
                )
                .unwrap()
                .save(&png)
                .unwrap();
            }
            let reloaded = crate::io::load_rgb_u8(&png).unwrap();
            assert_eq!(reloaded.data, source.data);
            let truth =
                std::fs::read_to_string(Path::new(input["truth"].as_str().unwrap())).unwrap();
            rows.push(serde_json::json!({"id":id,"cohort":input["cohort"],"raw_source":input["raw_source"],"width":source.width,"height":source.height,"truth":truth,"raw":raw,"mean_confidence":record["mean_confidence"],"stop_reason":record["stop_reason"],"steps":record["steps"],"baseline":baseline,"fragment":fragment,"eligible":eligible,"legacy_count_supported":strict,"short_common_supported":short,"closer_body_supported":body,"no_space":choose(true),"short_common":choose(short),"closer_body":choose(body),"glyphs":glyphs.iter().map(|g|serde_json::json!({"x0":g.x0,"x1":g.x1,"y0":g.y0,"y1":g.y1})).collect::<Vec<_>>()}));
        }
        assert_eq!(fresh, 656);
        assert_eq!(rows.len(), 1239);
        models.push(serde_json::json!({"label":label,"model":model,"native":session.input_dimensions(),"fresh_forward_count":fresh,"reused_forward_count":583,"rows":rows}));
    }
    std::fs::write(
        root.join("probe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"models":models})).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "diagnostic; verifies archived choices without models"]
fn replay_closer_geometry_from_frozen_pixels() {
    let Ok(root) = std::env::var("NDLOCR_GEOMETRY_REPLAY_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let probe: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("probe.json")).unwrap()).unwrap();
    let mut total = 0;
    for model in probe["models"].as_array().unwrap() {
        let label = model["label"].as_str().unwrap();
        for row in model["rows"].as_array().unwrap() {
            let id = row["id"].as_str().unwrap();
            let image =
                crate::io::load_rgb_u8(&root.join("inputs").join(format!("{id}.png"))).unwrap();
            let raw = row["raw"].as_str().unwrap();
            let baseline = sanitize_visible_single_line(
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
            assert_eq!(baseline, row["baseline"].as_str().unwrap(), "{label}/{id}");
            assert_eq!(fragment, row["fragment"].as_str().unwrap());
            let glyphs = collect_glyphs(&image.data, image.width, image.height);
            let actual: Vec<_> = glyphs
                .iter()
                .map(|g| serde_json::json!({"x0":g.x0,"x1":g.x1,"y0":g.y0,"y1":g.y1}))
                .collect();
            assert_eq!(serde_json::json!(actual), row["glyphs"]);
            let eligible = fragment.chars().count() > baseline.chars().count()
                && !fragment.chars().any(char::is_whitespace)
                && image.height as u128 * 5 <= image.width as u128 * 4;
            for (name, supported) in [
                ("no_space", true),
                ("short_common", short_common(&glyphs, &fragment)),
                ("closer_body", closer_body(&glyphs, &fragment)),
            ] {
                let candidate = if eligible && supported {
                    &fragment
                } else {
                    &baseline
                };
                assert_eq!(
                    candidate,
                    row[name].as_str().unwrap(),
                    "{label}/{id}/{name}"
                );
            }
            total += 1;
        }
    }
    assert_eq!(total, 3717);
    std::fs::write(
        root.join("replay.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"rows":total,"component_boxes_and_all_choices_equal":true}),
        )
        .unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "onnx")]
pub(super) fn supports_frozen_closer_body(
    rgb: &[u8],
    width: usize,
    height: usize,
    text: &str,
) -> bool {
    closer_body(&collect_glyphs(rgb, width, height), text)
}

#[cfg(feature = "onnx")]
pub(super) fn frozen_glyph_boxes(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Vec<(usize, usize, usize, usize)> {
    collect_glyphs(rgb, width, height)
        .into_iter()
        .map(|g| (g.x0, g.x1, g.y0, g.y1))
        .collect()
}
