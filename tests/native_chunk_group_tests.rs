#[path = "support/native_chunk_groups.rs"]
mod native;
use native::group_native_chunks;
use ndlocr_lite_rs::pipeline::line_chunks::LineChunk;
use proptest::prelude::*;

fn c(x0: usize, x1: usize, space: bool, gap: usize) -> LineChunk {
    LineChunk {
        x0,
        x1,
        space_before: space,
        gap_before: gap,
    }
}
#[test]
fn merges_only_contiguous_chunks_up_to_actual_model_ratio() {
    let original = vec![
        c(0, 80, false, 0),
        c(80, 160, true, 10),
        c(160, 240, false, 2),
        c(240, 320, false, 2),
        c(320, 400, true, 15),
    ];
    let grouped = group_native_chunks(&original, 10, 768, 24).unwrap();
    assert_eq!(grouped, vec![c(0, 320, false, 0), c(320, 400, true, 15)]);
}
#[test]
fn preserves_removed_white_gaps_and_boundary_metadata() {
    let original = vec![
        c(3, 30, false, 0),
        c(30, 70, true, 14),
        c(90, 110, true, 20),
        c(110, 150, false, 1),
    ];
    assert_eq!(
        group_native_chunks(&original, 10, 768, 24).unwrap(),
        vec![c(3, 70, false, 0), c(90, 150, true, 20)]
    );
}
#[test]
fn handles_exact_boundaries_and_unsplittable_wide_originals() {
    let original = vec![c(0, 100, false, 0), c(100, 200, true, 3)];
    assert_eq!(
        group_native_chunks(&original, 10, 480, 24).unwrap(),
        vec![c(0, 200, false, 0)]
    );
    assert_eq!(
        group_native_chunks(&original, 10, 479, 24).unwrap(),
        original
    );
    let wide = vec![c(0, 400, false, 0), c(400, 420, false, 1)];
    assert_eq!(group_native_chunks(&wide, 10, 768, 24).unwrap(), wide);
}
#[test]
fn rejects_invalid_geometry_and_orders() {
    for (chunks, height, iw, ih) in [
        (vec![c(0, 10, false, 0)], 0, 768, 24),
        (vec![], 10, 0, 24),
        (vec![], 10, 768, 0),
        (vec![c(10, 10, false, 0)], 10, 768, 24),
        (vec![c(0, 20, false, 0), c(19, 40, false, 0)], 10, 768, 24),
    ] {
        assert!(group_native_chunks(&chunks, height, iw, ih).is_err());
    }
    assert!(group_native_chunks(&[], 10, 768, 24).unwrap().is_empty());
}
proptest! {
    #[test]
    fn groups_preserve_exact_pixel_column_coverage(widths in proptest::collection::vec(1usize..41,1..30), gaps in proptest::collection::vec(0usize..4,30)) {
        let mut x=0;let mut original=Vec::new();
        for (i,width) in widths.iter().enumerate() {x+=gaps[i];original.push(c(x,x+width,i%2==1,gaps[i]));x+=width;}
        let grouped=group_native_chunks(&original,10,240,24).unwrap();
        let columns=|chunks:&[LineChunk]|chunks.iter().flat_map(|c|c.x0..c.x1).collect::<Vec<_>>();
        prop_assert_eq!(columns(&grouped),columns(&original));
        prop_assert!(grouped.len()<=original.len());
        for group in grouped {prop_assert!(group.x1-group.x0<=100);prop_assert!(original.iter().any(|c|c.x0==group.x0 && c.space_before==group.space_before && c.gap_before==group.gap_before));}
    }
}
