use super::*;
use proptest::prelude::*;
fn sample() -> (Vec<u8>, Vec<LineChunk>, Vec<BlankGap>) {
    let mut rgb = vec![255; 720 * 40 * 3];
    for (xs, ys) in [(10..295, 4..36), (303..311, 28..36), (340..710, 4..36)] {
        for y in ys {
            for x in xs.clone() {
                rgb[(y * 720 + x) * 3..(y * 720 + x + 1) * 3].fill(0);
            }
        }
    }
    (
        rgb,
        vec![
            LineChunk {
                x0: 0,
                x1: 300,
                space_before: false,
                gap_before: 0,
            },
            LineChunk {
                x0: 300,
                x1: 330,
                space_before: false,
                gap_before: 8,
            },
            LineChunk {
                x0: 330,
                x1: 720,
                space_before: true,
                gap_before: 29,
            },
        ],
        vec![
            BlankGap {
                midpoint: 300,
                width: 8,
                word_space: false,
                preferred_split: false,
                internal_glyph_gap: false,
            },
            BlankGap {
                midpoint: 330,
                width: 29,
                word_space: true,
                preferred_split: true,
                internal_glyph_gap: false,
            },
        ],
    )
}
fn copy(c: &[LineChunk]) -> Vec<LineChunk> {
    c.iter()
        .map(|p| LineChunk {
            x0: p.x0,
            x1: p.x1,
            space_before: p.space_before,
            gap_before: p.gap_before,
        })
        .collect()
}
#[test]
fn lower_connected_mark_returns_to_preceding_body_without_changing_pixels_or_gaps() {
    let (rgb, mut chunks, gaps) = sample();
    let original = rgb.clone();
    let expected = vec![
        LineChunk {
            x0: 0,
            x1: 330,
            space_before: false,
            gap_before: 0,
        },
        LineChunk {
            x0: 330,
            x1: 720,
            space_before: true,
            gap_before: 29,
        },
    ];
    restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, (256, 24));
    assert_eq!(chunks, expected);
    assert_eq!(rgb, original);
}
#[test]
fn unsupported_marks_and_spaces_preserve_original_chunks() {
    for case in 0..10 {
        let (mut rgb, mut chunks, mut gaps) = sample();
        let mut input = (256, 24);
        match case {
            0 => chunks[1].space_before = true,
            1 => chunks[1].gap_before = 11,
            2 => input.0 = 197,
            3 => {
                for y in 0..40 {
                    rgb[(y * 720 + 300) * 3..(y * 720 + 330) * 3].fill(255);
                }
            }
            4 => {
                for y in 0..40 {
                    rgb[(y * 720 + 300) * 3..(y * 720 + 330) * 3].fill(255);
                }
                rgb[(35 * 720 + 305) * 3..(35 * 720 + 306) * 3].fill(0);
            }
            5 => {
                rgb[(35 * 720 + 315) * 3..(35 * 720 + 316) * 3].fill(0);
            }
            6 => {
                rgb[(10 * 720 + 305) * 3..(10 * 720 + 306) * 3].fill(0);
            }
            7 => {
                for y in 15..35 {
                    rgb[(y * 720 + 305) * 3..(y * 720 + 306) * 3].fill(0);
                }
            }
            8 => {
                gaps.extend((1..=20).map(|i| BlankGap {
                    midpoint: 10 * i,
                    width: 1,
                    word_space: false,
                    preferred_split: false,
                    internal_glyph_gap: false,
                }));
                gaps.sort_by_key(|g| g.midpoint);
            }
            9 => {
                for y in 0..40 {
                    rgb[(y * 720) * 3..(y * 720 + 300) * 3].fill(255);
                }
                for x in 10..295 {
                    rgb[(30 * 720 + x) * 3..(30 * 720 + x + 1) * 3].fill(0);
                }
            }
            _ => unreachable!(),
        }
        let original = copy(&chunks);
        restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, input);
        assert_eq!(chunks, original, "case={case}");
    }
}
#[test]
fn exact_model_capacity_is_allowed() {
    let (rgb, mut chunks, gaps) = sample();
    restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, (198, 24));
    assert_eq!(chunks.len(), 2);
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(24))]
 #[test]
 fn merging_preserves_coverage_under_scale_and_translation(scale in 1usize..4,offset in 0usize..20){
  let (source,base,gapbase)=sample();let(w,h)=(720*scale+offset,40*scale);let mut rgb=vec![255;w*h*3];
  for y in 0..h{for x in 0..720*scale{let p=(y*w+x+offset)*3;let s=((y/scale)*720+x/scale)*3;rgb[p..p+3].copy_from_slice(&source[s..s+3]);}}
  let mut chunks:Vec<_>=base.iter().map(|p|LineChunk{x0:p.x0*scale+offset,x1:p.x1*scale+offset,space_before:p.space_before,gap_before:p.gap_before*scale}).collect();chunks[0].x0=0;
  let gaps:Vec<_>=gapbase.iter().map(|g|BlankGap{midpoint:g.midpoint*scale+offset,width:g.width*scale,word_space:g.word_space,preferred_split:g.preferred_split,internal_glyph_gap:g.internal_glyph_gap}).collect();
  restore_trailing_lower_mark(&rgb,(w,h),220,&mut chunks,&gaps,(256,24));
  prop_assert_eq!(chunks.len(),2);prop_assert_eq!(chunks[0].x0,0);prop_assert_eq!(chunks[0].x1,330*scale+offset);prop_assert_eq!(chunks[1].x0,chunks[0].x1);prop_assert_eq!(chunks[1].x1,w);prop_assert!(chunks[1].space_before);prop_assert_eq!(chunks[1].gap_before,29*scale);
 }
}

#[cfg(feature = "onnx")]
#[test]
#[ignore = "requires pinned PARSeq models"]
fn real_isolated_lower_marks_keep_horizontal_sentence_context() {
    use crate::{
        app::{fallback_line_detections_from_bands, prepare_line_crops},
        infer::{
            cached::ParseqSession,
            parseq::{ParseqOrientation, sanitize_recognized_fragment},
        },
        io::load_rgb_u8,
        pipeline::line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
    };
    use std::path::Path;
    for model in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let session = ParseqSession::load(
            &Path::new("models").join(model),
            Path::new("models/NDLmoji.yaml"),
        )
        .unwrap();
        let (iw, ih) = session.input_dimensions();
        for id in ["japanese_gothic36", "japanese_mincho36"] {
            let image = load_rgb_u8(
                &Path::new("tests/fixtures/long_line_split/images").join(format!("{id}.png")),
            )
            .unwrap();
            let ds = fallback_line_detections_from_bands(
                &image.data,
                image.width,
                image.height,
                220,
                25,
                45,
            );
            let crops = prepare_line_crops(&image.data, image.width, image.height, ds, 0).unwrap();
            assert_eq!(crops.len(), 1);
            let c = &crops[0].crop;
            let chunks = split_line_for_input(&c.data, c.width, c.height, 220, iw, ih).unwrap();
            let mut text = String::new();
            for (i, p) in chunks.iter().enumerate() {
                let crop = crop_line_chunk_rgb_u8(&c.data, c.width, c.height, &chunks, i).unwrap();
                let result = session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &crop.data,
                        crop.width,
                        crop.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap();
                let next = sanitize_recognized_fragment(&result.text);
                if p.insert_space_before(&text, &next, c.height) {
                    text.push(' ');
                }
                text.push_str(&next);
            }
            let truth = std::fs::read_to_string(
                Path::new("tests/fixtures/long_line_split/truth").join(format!("{id}.txt")),
            )
            .unwrap();
            assert_eq!(text, truth, "{model}: {id}");
        }
    }
}

#[test]
fn lower_mark_gap_and_half_height_limits_are_inclusive() {
    let (rgb, mut chunks, gaps) = sample();
    chunks[1].gap_before = 10;
    restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, (256, 24));
    assert_eq!(chunks.len(), 2);
    for edge in [20, 21] {
        let (mut rgb, mut chunks, gaps) = sample();
        for y in 0..40 {
            rgb[(y * 720 + 300) * 3..(y * 720 + 330) * 3].fill(255);
        }
        for y in 20..40 {
            for x in 302..302 + edge {
                rgb[(y * 720 + x) * 3..(y * 720 + x + 1) * 3].fill(0);
            }
        }
        restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, (256, 24));
        assert_eq!(chunks.len(), if edge == 20 { 2 } else { 3 });
    }
}

#[test]
fn diagonal_lower_mark_connectivity_uses_all_eight_neighbors() {
    for connected in [true, false] {
        let (mut rgb, mut chunks, gaps) = sample();
        for y in 0..40 {
            rgb[(y * 720 + 300) * 3..(y * 720 + 330) * 3].fill(255);
        }
        for i in 0..8 {
            if connected || i != 4 {
                let p = ((28 + i) * 720 + 303 + i) * 3;
                rgb[p..p + 3].fill(0);
            }
        }
        restore_trailing_lower_mark(&rgb, (720, 40), 220, &mut chunks, &gaps, (256, 24));
        assert_eq!(chunks.len(), if connected { 2 } else { 3 });
    }
}
