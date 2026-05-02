use ndlocr_lite_rs::pipeline::cascade::{LineCandidate, run_cascade};

#[test]
fn cascade_routes_and_fallbacks() {
    let lines = vec![
        LineCandidate::new(0, 3.0),
        LineCandidate::new(1, 2.0),
        LineCandidate::new(2, 100.0),
    ];
    let out = run_cascade(
        lines,
        |_| "x".repeat(25),
        |_| "y".repeat(45),
        |_| "ok".into(),
    );
    assert_eq!(out, vec!["ok", "ok", "ok"]);
}
