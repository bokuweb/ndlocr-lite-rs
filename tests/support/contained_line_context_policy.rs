use super::*;
use proptest::prelude::*;
fn sample() -> Vec<ScoredRegion<'static>> {
    vec![
        ScoredRegion {
            bounds: [0, 0, 160, 24],
            text: "ABCABC",
            mean_confidence: 0.95,
        },
        ScoredRegion {
            bounds: [0, 0, 50, 24],
            text: "AB",
            mean_confidence: 0.8,
        },
        ScoredRegion {
            bounds: [55, 0, 105, 24],
            text: "CA",
            mean_confidence: 0.8,
        },
        ScoredRegion {
            bounds: [110, 0, 160, 24],
            text: "BC",
            mean_confidence: 0.8,
        },
    ]
}
#[test]
fn reliable_complete_context_removes_only_its_same_line_fragments() {
    let mut rows = sample();
    rows.push(ScoredRegion {
        bounds: [0, 40, 160, 64],
        text: "another row",
        mean_confidence: 0.99,
    });
    assert_eq!(select_contexts(&rows, (256, 24)), [0, 4]);
    assert_eq!(rows[0].text, "ABCABC");
    assert_eq!(rows[0].bounds, [0, 0, 160, 24]);
    assert_eq!(rows[0].mean_confidence, 0.95);
}
#[test]
fn uncertain_sparse_or_unexplained_parents_keep_all_results() {
    for kind in 0..5 {
        let mut rows = sample();
        match kind {
            0 => rows[0].mean_confidence = 0.7,
            1 => rows[0].text = "A",
            2 => rows.truncate(2),
            3 => {
                rows[2].bounds = [100, 0, 120, 24];
                rows[3].bounds = [145, 0, 160, 24];
            }
            _ => rows[0].bounds = [0, 0, 800, 24],
        };
        assert_eq!(
            select_contexts(&rows, (256, 24)),
            (0..rows.len()).collect::<Vec<_>>(),
            "kind {kind}"
        );
    }
}
#[test]
fn other_baselines_partial_overlap_and_duplicate_supports_are_not_evidence() {
    for kind in 0..3 {
        let mut rows = sample();
        for child in &mut rows[1..] {
            match kind {
                0 => {
                    child.bounds[1] += 8;
                    child.bounds[3] += 8;
                }
                1 => {
                    child.bounds[0] += 150;
                    child.bounds[2] += 150;
                }
                _ => child.bounds = [10, 0, 30, 24],
            }
        }
        assert_eq!(select_contexts(&rows, (256, 24)), [0, 1, 2, 3]);
    }
}
#[test]
fn invalid_values_and_empty_text_do_not_suppress_other_results() {
    for score in [0., -0.1, 1.1, f32::NAN, f32::INFINITY] {
        let mut rows = sample();
        rows[0].mean_confidence = score;
        assert_eq!(select_contexts(&rows, (256, 24)), [0, 1, 2, 3]);
    }
    let mut rows = sample();
    rows[0].text = " ";
    assert_eq!(select_contexts(&rows, (256, 24)), [0, 1, 2, 3]);
    assert_eq!(select_contexts(&sample(), (0, 24)), [0, 1, 2, 3]);
}
#[test]
fn score_native_and_span_boundaries_are_inclusive() {
    let mut rows = sample();
    rows[0].mean_confidence = 0.8;
    assert_eq!(select_contexts(&rows, (160, 24)), [0]);
    rows[0].mean_confidence = f32::from_bits(0.8f32.to_bits() - 1);
    assert_eq!(select_contexts(&rows, (160, 24)), [0, 1, 2, 3]);
    rows[0].mean_confidence = 0.95;
    assert_eq!(select_contexts(&rows, (159, 24)), [0, 1, 2, 3]);
    for left in [40, 41] {
        let mut rows = sample();
        rows[1].bounds = [left, 0, 70, 24];
        rows[2].bounds = [75, 0, 105, 24];
        assert_eq!(
            select_contexts(&rows, (256, 24)),
            if left == 40 {
                vec![0]
            } else {
                vec![0, 1, 2, 3]
            }
        );
    }
}
proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(48))]
 #[test]
 fn context_choice_is_invariant_under_scale_translation_axis_and_order(scale in 1i32..8,dx in 0i32..40,dy in 0i32..40,vertical in any::<bool>(),reverse in any::<bool>()){
  let mut rows=sample();for r in &mut rows{let[x0,y0,x1,y1]=r.bounds;r.bounds=if vertical{[y0*scale+dx,x0*scale+dy,y1*scale+dx,x1*scale+dy]}else{[x0*scale+dx,y0*scale+dy,x1*scale+dx,y1*scale+dy]};}if reverse{rows.reverse();}let chosen=select_contexts(&rows,(256,24));prop_assert_eq!(chosen,vec![if reverse{3}else{0}]);
 }
}

#[test]
#[ignore = "diagnostic; requires explicit frozen-result root"]
fn replay_context_selection_on_frozen_page_results() {
    let Ok(root) = std::env::var("NDLOCR_CONTEXT_REPLAY_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let mut ledger = Vec::new();
    for (mode, native) in [
        ("runtime30_eval", (256, 24)),
        ("runtime30_default", (256, 24)),
        ("runtime100_eval", (768, 24)),
        ("runtime100_default", (768, 24)),
    ] {
        let source = root
            .join("count-only/after")
            .join(mode)
            .join("results.json");
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
        assert_eq!(rows.len(), 287);
        let dir = root.join("after").join(mode);
        std::fs::create_dir_all(&dir).unwrap();
        let mut outputs = Vec::new();
        for mut row in rows {
            let lines = row["lines"].as_array().unwrap();
            let scored: Vec<_> = lines
                .iter()
                .map(|line| ScoredRegion {
                    bounds: std::array::from_fn(|i| {
                        line["bbox"][i].as_i64().unwrap().try_into().unwrap()
                    }),
                    text: line["text"].as_str().unwrap(),
                    mean_confidence: line["confidence"].as_f64().unwrap() as f32,
                })
                .collect();
            let keep = select_contexts(&scored, native);
            let chosen: Vec<_> = keep.iter().map(|&i| lines[i].clone()).collect();
            ledger.push(serde_json::json!({"mode":mode,"id":row["id"],"before_lines":lines.len(),"kept_indices":keep}));
            let text = chosen
                .iter()
                .map(|line| line["text"].as_str().unwrap())
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(
                dir.join(format!("{}.txt", row["id"].as_str().unwrap())),
                &text,
            )
            .unwrap();
            row["text"] = text.into();
            row["lines"] = chosen.into();
            outputs.push(row);
        }
        std::fs::write(
            dir.join("results.json"),
            serde_json::to_vec_pretty(&outputs).unwrap(),
        )
        .unwrap();
    }
    std::fs::write(
        root.join("context-replay-ledger.json"),
        serde_json::to_vec_pretty(&ledger).unwrap(),
    )
    .unwrap();
}

#[test]
fn baseline_gap_quantity_and_invalid_box_boundaries_are_explicit() {
    for offset in [3, 4] {
        let mut rows = sample();
        for row in &mut rows[1..] {
            row.bounds[1] += offset;
            row.bounds[3] += offset;
        }
        assert_eq!(
            select_contexts(&rows, (256, 24)),
            if offset == 3 {
                vec![0]
            } else {
                vec![0, 1, 2, 3]
            }
        );
    }
    for gap in [24, 25] {
        let mut rows = sample();
        rows[1].bounds = [0, 0, 40, 24];
        rows[2].bounds = [40 + gap, 0, 80 + gap, 24];
        rows[3].bounds = [128, 0, 160, 24];
        assert_eq!(
            select_contexts(&rows, (256, 24)),
            if gap == 24 { vec![0] } else { vec![0, 1, 2, 3] }
        );
    }
    for text in ["ABC", "AB"] {
        let mut rows = sample();
        for row in &mut rows {
            row.mean_confidence = 1.;
        }
        rows[0].text = text;
        assert_eq!(
            select_contexts(&rows, (256, 24)),
            if text == "ABC" {
                vec![0]
            } else {
                vec![0, 1, 2, 3]
            }
        );
    }
    for bounds in [
        [-1, 0, 160, 24],
        [0, -1, 160, 24],
        [0, 0, 0, 24],
        [160, 0, 0, 24],
        [0, 24, 160, 0],
    ] {
        let mut rows = sample();
        rows[0].bounds = bounds;
        assert_eq!(select_contexts(&rows, (256, 24)), [0, 1, 2, 3]);
    }
}
