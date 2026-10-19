#[derive(Debug, PartialEq, Eq)]
struct Geometry {
    body_height: usize,
    max_blank_gap: usize,
    body_width: Option<usize>,
    anchors: usize,
}
impl Geometry {
    fn supported(&self) -> bool {
        self.max_blank_gap > 0
            && self.max_blank_gap as u128 * 4 <= self.body_height as u128 * 3
            && self
                .body_width
                .is_some_and(|w| self.max_blank_gap as u128 * 4 <= w as u128 * 3)
    }
}
fn measure_width_geometry(rgb: &[u8], w: usize, h: usize) -> Option<Geometry> {
    let (components, body_height) = super::visible_repetitions::probe_space_components(rgb, w, h)?;
    let mut columns = vec![false; w];
    for c in &components {
        columns[c[0]..c[2]].fill(true);
    }
    let left = columns.iter().position(|&v| v)?;
    let right = columns.iter().rposition(|&v| v)? + 1;
    let max_blank_gap = columns[left..right]
        .split(|&v| v)
        .map(<[bool]>::len)
        .max()
        .unwrap_or(0);
    let mut widths: Vec<_> = components
        .iter()
        .filter(|c| {
            (c[3] - c[1]) as u128 * 4 >= body_height as u128 * 3 && c[2] - c[0] <= body_height
        })
        .map(|c| c[2] - c[0])
        .collect();
    widths.sort_unstable_by(|a, b| b.cmp(a));
    Some(Geometry {
        body_height,
        max_blank_gap,
        body_width: widths.get(1).copied(),
        anchors: widths.len(),
    })
}

#[test]
#[ignore = "diagnostic; requires frozen WIDTH_SPACE RGB matrix"]
fn record_punctuation_space_width_geometry() {
    let Ok(root) = std::env::var("NDLOCR_WIDTH_SPACE_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let mut rows = Vec::new();
    for row in manifest["inputs"].as_array().unwrap() {
        let rgb = std::fs::read(row["rgb"].as_str().unwrap()).unwrap();
        let w = row["width"].as_u64().unwrap() as usize;
        let h = row["height"].as_u64().unwrap() as usize;
        let g = measure_width_geometry(&rgb, w, h).unwrap();
        rows.push(serde_json::json!({"id":row["id"],"width":w,"height":h,"body_height":g.body_height,
            "max_blank_gap":g.max_blank_gap,"body_width":g.body_width,"anchors":g.anchors,"supported":g.supported()}));
    }
    assert!(!rows.is_empty());
    std::fs::write(
        root.join("geometry-width.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rectangles(
        widths: &[usize],
        height: usize,
        gap: usize,
        margin: usize,
        tone: u8,
    ) -> (Vec<u8>, usize, usize) {
        let w = widths.iter().sum::<usize>() + (widths.len() - 1) * gap + margin * 2;
        let h = height + margin * 2;
        let mut rgb = vec![255; w * h * 3];
        let mut left = margin;
        for &cw in widths {
            for y in margin..margin + height {
                for x in left..left + cw {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(tone);
                }
            }
            left += cw + gap;
        }
        (rgb, w, h)
    }

    #[test]
    fn narrow_components_cannot_support_a_full_component_wide_real_space() {
        let (rgb, w, h) = rectangles(&[6, 6], 12, 6, 4, 0);
        let g = measure_width_geometry(&rgb, w, h).unwrap();
        assert_eq!(g.body_width, Some(6));
        assert_eq!(g.max_blank_gap, 6);
        assert!(!g.supported());
        let (rgb, w, h) = rectangles(&[12, 12], 12, 6, 4, 0);
        let g = measure_width_geometry(&rgb, w, h).unwrap();
        assert_eq!(g.body_width, Some(12));
        assert!(g.supported());
    }

    #[test]
    fn a_single_or_too_wide_component_does_not_supply_width_quorum() {
        for widths in [&[12][..], &[20, 12][..], &[20, 20][..]] {
            let (rgb, w, h) = rectangles(widths, 12, 4, 4, 0);
            let g = measure_width_geometry(&rgb, w, h).unwrap();
            assert!(g.anchors < 2);
            assert_eq!(g.body_width, None);
            assert!(!g.supported());
        }
    }

    #[test]
    fn width_and_anchor_height_thresholds_are_closed_and_invalid_rgb_is_rejected() {
        assert!(measure_width_geometry(&[], 0, 1).is_none());
        assert!(measure_width_geometry(&[0; 3], usize::MAX, 2).is_none());
        assert!(measure_width_geometry(&[0; 5], 2, 1).is_none());
        assert!(measure_width_geometry(&[255; 48], 4, 4).is_none());
        let (rgb, w, h) = rectangles(&[12, 12], 12, 9, 4, 0);
        assert!(measure_width_geometry(&rgb, w, h).unwrap().supported());
        let (rgb, w, h) = rectangles(&[12, 12], 12, 10, 4, 0);
        assert!(!measure_width_geometry(&rgb, w, h).unwrap().supported());
        let (w, h) = (50, 24);
        let mut rgb = vec![255; w * h * 3];
        for (x0, cw, ch) in [(4, 12, 12), (20, 10, 9), (34, 10, 8)] {
            for y in 4..4 + ch {
                for x in x0..x0 + cw {
                    rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
                }
            }
        }
        let g = measure_width_geometry(&rgb, w, h).unwrap();
        assert_eq!(g.anchors, 2);
        assert_eq!(g.body_width, Some(10));
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases:64,
            failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
                "proptest-regressions/infer/punctuation_space_width.txt"
            ))),
            ..Default::default()
        })]
        #[test]
        fn width_support_preserves_horizontal_scale_while_anchor_conditions_hold(
            bw in 4usize..17,gap in 1usize..21,scale in 1usize..5,margin in 3usize..10,tone in 0u8..220,
        ) {
            let(rgb,w,h)=rectangles(&[bw*scale,bw*scale],bw*4,gap*scale,margin,tone);
            let g=measure_width_geometry(&rgb,w,h).unwrap();
            proptest::prop_assert_eq!(g.body_width,Some(bw*scale));
            proptest::prop_assert_eq!(g.max_blank_gap,gap*scale);
            proptest::prop_assert_eq!(g.supported(),4*gap<=3*bw);
        }
    }
}
