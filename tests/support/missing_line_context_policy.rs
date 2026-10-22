use super::*;
use crate::{infer::parseq::RecognizeResult, pipeline::line_context::ScoredRegion};
use proptest::prelude::*;
fn fixture() -> (Vec<u8>, usize, usize) {
    let (w, h) = (64, 24);
    let mut rgb = vec![255; w * h * 3];
    for i in 0..6 {
        for y in 6..18 {
            for x in 3 + i * 9..8 + i * 9 {
                if x < 5 + i * 9 || y < 8 || y >= 16 {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
    }
    (rgb, w, h)
}
fn rows() -> Vec<ScoredRegion<'static>> {
    vec![
        ScoredRegion {
            bounds: [0, 4, 20, 20],
            text: "AA",
            mean_confidence: 0.6,
        },
        ScoredRegion {
            bounds: [37, 4, 64, 20],
            text: "BB",
            mean_confidence: 0.6,
        },
    ]
}
fn result(text: &str, score: f32) -> RecognizeResult {
    RecognizeResult {
        text: text.into(),
        mean_confidence: score,
    }
}
#[test]
fn missing_ink_accepts_the_strongest_context_without_changing_its_pixels_or_text() {
    let (rgb, w, h) = fixture();
    let original = rgb.clone();
    eprintln!(
        "fixture bands: {:?}",
        crate::pipeline::line_segment::detect_textline_bands_naive(&rgb, w, h, 220)
    );
    let mut calls = 0;
    let out = recover_missing_line_context(&rows(), &rgb, w, h, (256, 24), |data, cw, ch| {
        calls += 1;
        assert_eq!(data.len(), cw * ch * 3);
        if calls == 1 {
            assert_eq!(data, rgb);
            Ok(result("ABCABC", 0.9))
        } else {
            Ok(result("ABCA", 0.8))
        }
    })
    .unwrap();
    assert_eq!(out.bounds, [0, 0, 64, 24]);
    assert_eq!(out.recognized.text, "ABCABC");
    assert_eq!(out.recognized.mean_confidence, 0.9);
    assert_eq!(calls, 2);
    assert_eq!(rgb, original);
}
#[test]
fn complete_coverage_empty_invalid_or_zero_confidence_rows_do_not_call_models() {
    let (rgb, w, h) = fixture();
    for kind in 0..7 {
        let mut rs = rows();
        match kind {
            0 => rs[0].bounds = [0, 0, w as i32, h as i32],
            1 => rs.clear(),
            2 => rs[0].text = " ",
            3 => rs[0].mean_confidence = f32::NAN,
            4 => {
                rs[0].mean_confidence = 0.;
                rs[1].mean_confidence = 0.;
            }
            5 => rs[0].bounds = [-1, 4, 20, 20],
            _ => rs[0].mean_confidence = 1.1,
        };
        let out = recover_missing_line_context(&rs, &rgb, w, h, (256, 24), |_, _, _| {
            panic!("kind {kind}")
        });
        assert!(out.is_none());
    }
}
#[test]
fn blank_solid_two_rows_invalid_rgb_and_insufficient_native_capacity_do_not_call_models() {
    let (rgb, w, h) = fixture();
    for kind in 0..5 {
        let mut data = rgb.clone();
        let mut ih = h;
        let mut native = (256, 24);
        match kind {
            0 => data.fill(255),
            1 => {
                data.fill(255);
                for y in 6..18 {
                    for x in 3..53 {
                        data[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                    }
                }
            }
            2 => {
                data.extend_from_slice(&rgb);
                ih *= 2;
            }
            3 => {
                data.pop();
            }
            _ => native = (8, 24),
        };
        assert!(
            recover_missing_line_context(&rows(), &data, w, ih, native, |_, _, _| panic!(
                "kind {kind}"
            ))
            .is_none()
        );
    }
}
#[test]
fn failed_low_equal_empty_nonfinite_and_out_of_range_candidates_keep_the_original() {
    let (rgb, w, h) = fixture();
    for kind in 0..7 {
        let mut calls = 0;
        let out = recover_missing_line_context(&rows(), &rgb, w, h, (256, 24), |_, _, _| {
            calls += 1;
            match kind {
                0 => Err(anyhow::anyhow!("failed context")),
                1 => Ok(result("ABCABC", 0.5)),
                2 => Ok(result("ABCABC", 0.6)),
                3 => Ok(result(" ", 0.99)),
                4 => Ok(result("ABCABC", f32::NAN)),
                5 => Ok(result("ABCABC", 1.1)),
                _ => Ok(result("ABCABC", f32::INFINITY)),
            }
        });
        assert!(out.is_none());
        assert_eq!(calls, 2);
    }
    let score = f32::from_bits(0.6f32.to_bits() + 1);
    assert!(
        recover_missing_line_context(&rows(), &rgb, w, h, (256, 24), |_, _, _| Ok(result(
            "ABCABC", score
        )))
        .is_some()
    );
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(40))]
 #[test]
 fn scale_and_translation_keep_the_supported_context(scale in 1usize..4,dx in 0usize..8,dy in 0usize..8){
 let(rgb,w,h)=fixture();let(nw,nh)=(w*scale+dx,h*scale+dy);let mut data=vec![255;nw*nh*3];for y in 0..h{for x in 0..w{for yy in 0..scale{for xx in 0..scale{let dest=((y*scale+dy+yy)*nw+x*scale+dx+xx)*3;data[dest..dest+3].copy_from_slice(&rgb[(y*w+x)*3..(y*w+x+1)*3]);}}}}
 let mut rs=rows();for row in &mut rs{let[a,b,c,d]=row.bounds;row.bounds=[a*scale as i32+dx as i32,b*scale as i32+dy as i32,c*scale as i32+dx as i32,d*scale as i32+dy as i32];}
 let out=recover_missing_line_context(&rs,&data,nw,nh,(256,24),|_,_,_|Ok(result("ABCABC",0.9))).unwrap();prop_assert_eq!(out.recognized.text,"ABCABC");prop_assert_eq!(out.recognized.mean_confidence,0.9);
 }
}

#[test]
fn comparison_uses_nonspace_unicode_character_weights_and_native_limits_are_closed() {
    let (rgb, w, h) = fixture();
    let mut rs = rows();
    rs[0].text = "一";
    rs[0].mean_confidence = 0.5;
    rs[1].text = "Ｂ Ｂ Ｂ Ｂ Ｂ Ｂ Ｂ Ｂ Ｂ";
    rs[1].mean_confidence = 0.9;
    assert!(
        recover_missing_line_context(&rs, &rgb, w, h, (256, 24), |_, _, _| Ok(result(
            "ABCABC", 0.8
        )))
        .is_none()
    );
    assert!(
        recover_missing_line_context(&rs, &rgb, w, h, (256, 24), |_, _, _| Ok(result(
            "ABCABC", 0.87
        )))
        .is_some()
    );
    for native in [(64, 24), (63, 24), (0, 24), (256, 0)] {
        let mut calls = 0;
        let chosen = recover_missing_line_context(&rows(), &rgb, w, h, native, |_, _, _| {
            calls += 1;
            Ok(result("ABCABC", 1.0))
        });
        assert_eq!(chosen.is_some(), native == (64, 24));
        assert_eq!(calls, usize::from(native == (64, 24)));
    }
}

#[test]
#[ignore = "diagnostic; requires explicit frozen before/after matrix root"]
fn replay_missing_context_choices_from_recorded_model_outputs() {
    let Ok(root) = std::env::var("NDLOCR_MISSING_CONTEXT_REPLAY_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let mut ledger = Vec::new();
    for (mode, native) in [
        ("runtime30_eval", (256, 24)),
        ("runtime30_default", (256, 24)),
        ("runtime100_eval", (768, 24)),
        ("runtime100_default", (768, 24)),
    ] {
        let before: Vec<serde_json::Value> = serde_json::from_slice(
            &std::fs::read(root.join("before").join(mode).join("results.json")).unwrap(),
        )
        .unwrap();
        let after: Vec<serde_json::Value> = serde_json::from_slice(
            &std::fs::read(root.join("after").join(mode).join("results.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(before.len(), 287);
        assert_eq!(after.len(), 287);
        for old in &before {
            let id = old["id"].as_str().unwrap();
            let source = manifest["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap();
            let image =
                crate::io::load_rgb_u8(std::path::Path::new(source["image"].as_str().unwrap()))
                    .unwrap();
            let expected = after.iter().find(|x| x["id"] == id).unwrap();
            let traces = expected["context_attempts"].as_array().unwrap();
            let lines = old["lines"].as_array().unwrap();
            let rows: Vec<_> = lines
                .iter()
                .map(|line| ScoredRegion {
                    bounds: std::array::from_fn(|i| line["bbox"][i].as_i64().unwrap() as i32),
                    text: line["text"].as_str().unwrap(),
                    mean_confidence: line["confidence"].as_f64().unwrap() as f32,
                })
                .collect();
            let prepared = crate::pipeline::background::normalize_chromatic_panels(
                &image.data,
                image.width,
                image.height,
            )
            .unwrap();
            let variants = if prepared.as_ref() == image.data.as_slice() {
                vec![image.data.as_slice()]
            } else {
                vec![image.data.as_slice(), prepared.as_ref()]
            };
            for rgb in variants {
                let mut calls = 0;
                let recovered = recover_missing_line_context(
                    &rows,
                    rgb,
                    image.width,
                    image.height,
                    native,
                    |data, w, h| {
                        let trace = traces
                            .get(calls)
                            .unwrap_or_else(|| panic!("unexpected call {mode}/{id}"));
                        calls += 1;
                        assert_eq!(
                            trace["input_dimensions"],
                            serde_json::json!([w, h]),
                            "{mode}/{id}"
                        );
                        let bounds: [usize; 4] =
                            std::array::from_fn(|i| trace["bounds"][i].as_u64().unwrap() as usize);
                        let crop = crate::pipeline::crop::crop_rgb_u8(
                            rgb,
                            image.width,
                            image.height,
                            crate::pipeline::crop::BBox::new(
                                bounds[0], bounds[1], bounds[2], bounds[3],
                            ),
                        )
                        .unwrap();
                        assert_eq!(data, crop.data, "{mode}/{id}");
                        if trace["candidate"].is_null() {
                            Err(anyhow::anyhow!("recorded candidate error"))
                        } else {
                            Ok(result(
                                trace["candidate"]["text"].as_str().unwrap(),
                                trace["candidate"]["confidence"].as_f64().unwrap() as f32,
                            ))
                        }
                    },
                );
                assert_eq!(calls, traces.len(), "{mode}/{id}");
                let actual: Vec<_> = if let Some(context) = recovered {
                    vec![
                        serde_json::json!({"bbox":context.bounds,"text":context.recognized.text,"confidence":context.recognized.mean_confidence}),
                    ]
                } else {
                    lines.iter().map(|line|serde_json::json!({"bbox":line["bbox"],"text":line["text"],"confidence":line["confidence"].as_f64().unwrap()as f32})).collect()
                };
                let wanted:Vec<_>=expected["lines"].as_array().unwrap().iter().map(|line|serde_json::json!({"bbox":line["bbox"],"text":line["text"],"confidence":line["confidence"].as_f64().unwrap()as f32})).collect();
                assert_eq!(actual, wanted, "{mode}/{id}");
            }
            ledger.push(
                serde_json::json!({"mode":mode,"id":id,"model_calls":traces.len(),"matches":true}),
            );
        }
    }
    assert_eq!(ledger.len(), 1148);
    std::fs::write(
        root.join("policy-replay.json"),
        serde_json::to_vec_pretty(&ledger).unwrap(),
    )
    .unwrap();
}
