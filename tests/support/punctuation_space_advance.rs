#[derive(Debug, PartialEq, Eq)]
struct Geometry {
    body_height: usize,
    max_blank_gap: usize,
    foreground_width: usize,
}
impl Geometry {
    fn supported_for(&self, count: usize) -> bool {
        (6..=12).contains(&count)
            && self.max_blank_gap > 0
            && self.max_blank_gap as u128 * 4 <= self.body_height as u128 * 3
            && self.max_blank_gap as u128 * 4 * count as u128 <= self.foreground_width as u128 * 3
    }
}
fn measure_advance_geometry(rgb: &[u8], w: usize, h: usize) -> Option<Geometry> {
    let (components, body_height) = super::visible_repetitions::probe_space_components(rgb, w, h)?;
    let mut columns = vec![false; w];
    for c in components {
        columns[c[0]..c[2]].fill(true);
    }
    let left = columns.iter().position(|&v| v)?;
    let right = columns.iter().rposition(|&v| v)? + 1;
    let max_blank_gap = columns[left..right]
        .split(|&v| v)
        .map(<[bool]>::len)
        .max()
        .unwrap_or(0);
    Some(Geometry {
        body_height,
        max_blank_gap,
        foreground_width: right - left,
    })
}

#[test]
#[ignore = "diagnostic; requires frozen ADVANCE_SPACE RGB matrix"]
fn record_punctuation_space_advance_geometry() {
    let Ok(root) = std::env::var("NDLOCR_ADVANCE_SPACE_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let mut rows = Vec::new();
    for row in m["inputs"].as_array().unwrap() {
        let rgb = std::fs::read(row["rgb"].as_str().unwrap()).unwrap();
        let w = row["width"].as_u64().unwrap() as usize;
        let h = row["height"].as_u64().unwrap() as usize;
        let g = measure_advance_geometry(&rgb, w, h).unwrap();
        let support: std::collections::BTreeMap<_, _> = (6..=12)
            .map(|n| (n.to_string(), g.supported_for(n)))
            .collect();
        rows.push(serde_json::json!({"id":row["id"],"width":w,"height":h,"body_height":g.body_height,"max_blank_gap":g.max_blank_gap,
            "foreground_width":g.foreground_width,"support_by_length":support}));
    }
    assert!(!rows.is_empty());
    std::fs::write(
        root.join("geometry-advance.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn image(
        bw: usize,
        gap: usize,
        height: usize,
        scale: usize,
        margin: usize,
        tone: u8,
    ) -> (Vec<u8>, usize, usize) {
        let w = (bw * 2 + gap) * scale + margin * 2;
        let h = height + margin * 2;
        let mut rgb = vec![255; w * h * 3];
        for y in margin..margin + height {
            for x in (margin..margin + bw * scale)
                .chain(margin + (bw + gap) * scale..margin + (bw * 2 + gap) * scale)
            {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(tone);
            }
        }
        (rgb, w, h)
    }

    #[test]
    fn text_count_distinguishes_bearing_from_a_wide_real_space() {
        let (rgb, w, h) = image(12, 6, 12, 1, 4, 0);
        let g = measure_advance_geometry(&rgb, w, h).unwrap();
        assert_eq!(g.foreground_width, 30);
        assert!(!g.supported_for(8));
        let (rgb, w, h) = image(24, 3, 12, 1, 4, 0);
        let g = measure_advance_geometry(&rgb, w, h).unwrap();
        assert_eq!(g.foreground_width, 51);
        assert!(g.supported_for(8));
    }

    #[test]
    fn closed_threshold_and_count_range_are_checked_before_multiplication() {
        let g = Geometry {
            body_height: 12,
            max_blank_gap: 3,
            foreground_width: 32,
        };
        assert!(g.supported_for(8));
        assert!(
            !Geometry {
                foreground_width: 31,
                ..g
            }
            .supported_for(8)
        );
        for n in [0, 5, 13, usize::MAX] {
            assert!(!g.supported_for(n));
        }
        let g = Geometry {
            body_height: usize::MAX,
            max_blank_gap: usize::MAX,
            foreground_width: usize::MAX,
        };
        assert!(!g.supported_for(usize::MAX));
        assert!(!g.supported_for(12));
        assert!(
            !Geometry {
                body_height: 12,
                max_blank_gap: 0,
                foreground_width: 32
            }
            .supported_for(8)
        );
        assert!(measure_advance_geometry(&[], 0, 1).is_none());
        assert!(measure_advance_geometry(&[0; 3], usize::MAX, 2).is_none());
        assert!(measure_advance_geometry(&[255; 48], 4, 4).is_none());
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases:64,failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
                "proptest-regressions/infer/punctuation_space_advance.txt"
            ))),..Default::default()
        })]
        #[test]
        fn span_and_gap_scale_horizontally_and_support_never_exceeds_d(
            bw in 8usize..25,gap in 1usize..15,scale in 1usize..5,n in 6usize..13,
            margin in 3usize..10,tone in 0u8..220,
        ) {
            let(rgb,w,h)=image(bw,gap,80,scale,margin,tone);let g=measure_advance_geometry(&rgb,w,h).unwrap();
            proptest::prop_assert_eq!(g.foreground_width,(2*bw+gap)*scale);
            proptest::prop_assert_eq!(g.max_blank_gap,gap*scale);
            let width_supported=4*gap*n<=3*(2*bw+gap);
            proptest::prop_assert_eq!(g.supported_for(n),width_supported&&4*gap*scale<=3*80);
            proptest::prop_assert!(!g.supported_for(n)||4*g.max_blank_gap<=3*g.body_height);
        }
    }
}
