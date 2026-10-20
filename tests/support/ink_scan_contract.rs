use super::*;
use proptest::prelude::*;

fn fixture() -> Vec<u8> {
    let mut rgb = vec![255; 8 * 5 * 3];
    for (x, y, color) in [
        (1, 0, [219, 220, 220]),
        (1, 4, [0, 0, 0]),
        (2, 2, [0, 0, 0]),
        (5, 1, [0, 0, 0]),
        (5, 2, [0, 0, 0]),
        (5, 3, [0, 0, 0]),
        (6, 2, [0, 0, 0]),
    ] {
        rgb[(y * 8 + x) * 3..(y * 8 + x + 1) * 3].copy_from_slice(&color);
    }
    rgb
}

#[test]
fn scan_preserves_strict_rgb_threshold_peak_counts_and_full_envelopes() {
    let result = scan_ink_runs(&fixture(), 8, 5, 220);
    assert_eq!(result.widths, [2, 2]);
    assert_eq!(result.heights, [2, 3]);
    assert_eq!(result.extents, [5, 3]);
    assert_eq!(result.gaps.len(), 1);
    let gap = &result.gaps[0];
    assert_eq!((gap.midpoint, gap.width), (4, 2));
    assert!(!gap.word_space && !gap.preferred_split && !gap.internal_glyph_gap);
}

#[test]
fn terminal_ink_flushes_the_last_run_without_a_trailing_blank() {
    let mut rgb = fixture();
    rgb[(4 * 8 + 7) * 3..(4 * 8 + 8) * 3].fill(0);
    let result = scan_ink_runs(&rgb, 8, 5, 220);
    assert_eq!(result.widths, [2, 3]);
    assert_eq!(result.heights, [2, 3]);
    assert_eq!(result.extents, [5, 4]);
    assert_eq!(
        result
            .gaps
            .iter()
            .map(|g| (g.midpoint, g.width))
            .collect::<Vec<_>>(),
        [(4, 2)]
    );
}

#[test]
fn white_pixels_at_the_threshold_do_not_create_a_run() {
    let result = scan_ink_runs(&[220; 3 * 3 * 3], 3, 3, 220);
    assert!(
        result.gaps.is_empty()
            && result.widths.is_empty()
            && result.heights.is_empty()
            && result.extents.is_empty()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn scan_is_invariant_under_scale_translation_reflection_and_valid_palettes(
        scale in 1usize..5,padding in 0usize..15,reflected in any::<bool>(),
        foreground in prop::array::uniform3(0u8..121),background in prop::array::uniform3(230u8..=255),
    ) {
        let width=8*scale+2*padding;let height=5*scale;let mut rgb=background.repeat(width*height);
        for (x,y)in [(1,0),(1,4),(2,2),(5,1),(5,2),(5,3),(6,2)] {
            for yy in y*scale..(y+1)*scale {for xx in padding+x*scale..padding+(x+1)*scale {
                let xx=if reflected {width-1-xx}else{xx};rgb[(yy*width+xx)*3..(yy*width+xx+1)*3].copy_from_slice(&foreground);
            }}
        }
        let result=scan_ink_runs(&rgb,width,height,220);
        prop_assert_eq!(result.widths,vec![2*scale,2*scale]);
        prop_assert_eq!(result.heights,if reflected {vec![3*scale,2*scale]}else{vec![2*scale,3*scale]});
        prop_assert_eq!(result.extents,if reflected {vec![3*scale,5*scale]}else{vec![5*scale,3*scale]});
        prop_assert_eq!(result.gaps.len(),1);prop_assert_eq!((result.gaps[0].midpoint,result.gaps[0].width),(padding+4*scale,2*scale));
    }
}
