use super::*;
use crate::pipeline::crop::{BBox, crop_rgb_u8};
use proptest::prelude::*;

fn white(w: usize, h: usize) -> Vec<u8> {
    vec![255; w * h * 3]
}
fn rect(rgb: &mut [u8], w: usize, b: [usize; 4]) {
    for y in b[1]..b[3] {
        for x in b[0]..b[2] {
            rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
        }
    }
}

#[test]
fn moves_an_ink_cut_to_the_nearby_blank_gap() {
    let mut rgb = white(200, 20);
    rect(&mut rgb, 200, [0, 0, 105, 20]);
    rect(&mut rgb, 200, [109, 0, 200, 20]);
    let chunks = split_long_line_near_center(&rgb, 200, 20, 220)
        .unwrap()
        .unwrap();
    assert_eq!(
        (chunks[0].x0, chunks[0].x1, chunks[1].x0, chunks[1].x1),
        (0, 107, 107, 200)
    );
    assert!(!chunks[1].space_before);
}
#[test]
fn preserves_a_safe_original_midpoint_and_real_word_space() {
    let mut rgb = white(200, 20);
    rect(&mut rgb, 200, [0, 0, 90, 20]);
    rect(&mut rgb, 200, [110, 0, 200, 20]);
    let chunks = split_long_line_near_center(&rgb, 200, 20, 220)
        .unwrap()
        .unwrap();
    assert_eq!(chunks[0].x1, 100);
    assert!(chunks[1].space_before);
    assert_eq!(chunks[1].gap_before, 20);
}
#[test]
fn refuses_distant_gaps_and_narrow_or_solid_lines() {
    let mut rgb = white(200, 20);
    rect(&mut rgb, 200, [0, 0, 35, 20]);
    rect(&mut rgb, 200, [45, 0, 200, 20]);
    assert!(
        split_long_line_near_center(&rgb, 200, 20, 220)
            .unwrap()
            .is_none()
    );
    assert!(
        split_long_line_near_center(&vec![0; 200 * 20 * 3], 200, 20, 220)
            .unwrap()
            .is_none()
    );
    assert!(
        split_long_line_near_center(&white(79, 20), 79, 20, 220)
            .unwrap()
            .is_none()
    );
}
#[test]
fn ignores_only_supported_underlines_for_the_split_decision() {
    let mut rgb = white(240, 24);
    for i in 0..13 {
        rect(&mut rgb, 240, [6 + i * 18, 6, 16 + i * 18, 22]);
    }
    rect(&mut rgb, 240, [0, 23, 240, 24]);
    let original = rgb.clone();
    let chunks = split_long_line_near_center(&rgb, 240, 24, 220)
        .unwrap()
        .unwrap();
    assert_eq!(chunks[0].x1, 128);
    assert_eq!(rgb, original);
    for y in 0..23 {
        let p = (y * 240 + 128) * 3;
        assert_eq!(&rgb[p..p + 3], &[255; 3]);
    }
    assert_eq!(&rgb[(23 * 240 + 128) * 3..(23 * 240 + 129) * 3], &[0; 3]);
    let mut few = white(240, 24);
    for x in [5, 100, 200] {
        rect(&mut few, 240, [x, 6, x + 10, 22]);
    }
    rect(&mut few, 240, [0, 23, 240, 24]);
    assert!(
        split_long_line_near_center(&few, 240, 24, 220)
            .unwrap()
            .is_none()
    );
}
#[test]
fn rejects_bad_dimensions_and_buffer_lengths() {
    for (rgb, w, h) in [
        (vec![], 0, 20),
        (vec![], 20, 0),
        (vec![255; 3], 200, 20),
        (vec![], usize::MAX, 2),
    ] {
        assert!(split_long_line_near_center(&rgb, w, h, 220).is_err());
    }
}
proptest! {
    #[test]
    fn accepted_cuts_preserve_every_source_pixel_and_do_not_cross_ink(columns in proptest::collection::vec(0u8..4,200)) {
        let mut rgb=white(200,20);
        for (x,v) in columns.iter().enumerate(){if *v==0{rect(&mut rgb,200,[x,0,x+1,20]);}}
        if let Some(chunks)=split_long_line_near_center(&rgb,200,20,220).unwrap(){
            let cut=chunks[0].x1;
            prop_assert_eq!(chunks[1].x0,cut);prop_assert!(cut.abs_diff(100)<=20);
            prop_assert!(columns[cut]!=0 || columns[cut-1]!=0);
            let left=crop_rgb_u8(&rgb,200,20,BBox::new(0,0,cut,20)).unwrap();
            let right=crop_rgb_u8(&rgb,200,20,BBox::new(cut,0,200,20)).unwrap();
            let mut recovered=Vec::new();
            for y in 0..20 {recovered.extend_from_slice(&left.data[y*cut*3..(y+1)*cut*3]);recovered.extend_from_slice(&right.data[y*(200-cut)*3..(y+1)*(200-cut)*3]);}
            prop_assert_eq!(recovered,rgb);
        }
    }
}

#[test]
fn avoids_a_narrow_gap_between_parts_of_one_glyph() {
    let mut rgb = white(200, 30);
    for b in [
        [0, 0, 1, 30],
        [96, 0, 102, 30],
        [103, 0, 108, 30],
        [199, 0, 200, 30],
    ] {
        rect(&mut rgb, 200, b);
    }
    assert!(
        split_long_line_near_center(&rgb, 200, 30, 220)
            .unwrap()
            .is_none()
    );
}
#[test]
fn ties_choose_the_smaller_coordinate() {
    let mut rgb = white(200, 20);
    for b in [[0, 0, 92, 20], [96, 0, 104, 20], [108, 0, 200, 20]] {
        rect(&mut rgb, 200, b);
    }
    assert_eq!(
        split_long_line_near_center(&rgb, 200, 20, 220)
            .unwrap()
            .unwrap()[0]
            .x1,
        94
    );
}
#[test]
fn honors_the_exact_ink_threshold() {
    let mut rgb = vec![0; 200 * 20 * 3];
    for y in 0..20 {
        for x in 98..102 {
            rgb[(y * 200 + x) * 3..(y * 200 + x + 1) * 3].fill(220);
        }
    }
    assert_eq!(
        split_long_line_near_center(&rgb, 200, 20, 220)
            .unwrap()
            .unwrap()[0]
            .x1,
        100
    );
    assert!(
        split_long_line_near_center(&rgb, 200, 20, 221)
            .unwrap()
            .is_none()
    );
}

#[test]
fn keeps_an_existing_blank_cut_even_when_the_gap_is_narrow() {
    let mut rgb = white(200, 30);
    for b in [
        [0, 0, 1, 30],
        [96, 0, 99, 30],
        [100, 0, 104, 30],
        [199, 0, 200, 30],
    ] {
        rect(&mut rgb, 200, b);
    }
    let chunks = split_long_line_near_center(&rgb, 200, 30, 220)
        .unwrap()
        .expect("safe legacy cut must not disable recognition");
    assert_eq!(chunks[0].x1, 100);
}
#[test]
fn preserves_nonconnecting_strokes_with_overlapping_column_projections() {
    let mut rgb = white(200, 30);
    rect(&mut rgb, 200, [0, 0, 100, 5]);
    rect(&mut rgb, 200, [100, 25, 200, 30]);
    let chunks = split_long_line_near_center(&rgb, 200, 30, 220)
        .unwrap()
        .expect("separate strokes are not cut");
    assert_eq!(chunks[0].x1, 100);
    assert!(!chunks[1].space_before);
}
