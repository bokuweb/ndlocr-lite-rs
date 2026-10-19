#[derive(Debug, PartialEq, Eq)]
struct ComponentGeometry {
    body_height: usize,
    max_blank_gap: usize,
    components: usize,
}

impl ComponentGeometry {
    fn supported(&self) -> bool {
        self.max_blank_gap > 0 && self.max_blank_gap as u128 * 4 <= self.body_height as u128 * 3
    }
}

fn measure_component_geometry(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Option<ComponentGeometry> {
    let (components, body_height) =
        super::visible_repetitions::probe_space_components(rgb, width, height)?;
    let mut columns = vec![false; width];
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
    Some(ComponentGeometry {
        body_height,
        max_blank_gap,
        components: components.len(),
    })
}

fn punctuation_boundary(text: &str) -> bool {
    let Some((left, right)) = text.split_once(' ') else {
        return false;
    };
    matches!(left.chars().next_back(), Some('、' | '。'))
        && !right.contains(' ')
        && right.chars().next().is_some_and(|c| {
            matches!(c, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{4dbf}'
                | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_dots_do_not_make_a_real_gap_look_like_punctuation_bearing() {
        let (w, h) = (32, 24);
        let mut rgb = vec![255; w * h * 3];
        for y in 6..18 {
            for x in (4..10).chain(22..28) {
                rgb[(y * w + x) * 3..(y * w + x + 1) * 3].fill(0);
            }
        }
        let base = measure_component_geometry(&rgb, w, h).unwrap();
        assert_eq!(base.body_height, 12);
        assert_eq!(base.max_blank_gap, 12);
        assert!(!base.supported());
        for y in [0, h - 1] {
            rgb[(y * w + 16) * 3..(y * w + 17) * 3].fill(0);
        }
        assert_eq!(measure_component_geometry(&rgb, w, h).unwrap(), base);
    }

    #[test]
    fn decoded_boundary_requires_japanese_punctuation_followed_by_body() {
        for text in ["予定時刻、 担当者", "実験完了。 記録"] {
            assert!(punctuation_boundary(text), "{text}");
        }
        for text in [
            "登録内容 保存先",
            "資料 、記録",
            "、 ",
            "、  次",
            "、 、次",
            "予定、 A",
        ] {
            assert!(!punctuation_boundary(text), "{text}");
        }
    }

    #[test]
    fn component_support_requires_valid_rgb_and_a_real_internal_gap() {
        assert!(measure_component_geometry(&[], 0, 4).is_none());
        assert!(measure_component_geometry(&[0; 3], usize::MAX, 2).is_none());
        assert!(measure_component_geometry(&[0; 5], 2, 1).is_none());
        assert!(measure_component_geometry(&[255; 36], 3, 4).is_none());
        assert!(
            !measure_component_geometry(&[0; 8 * 12 * 3], 8, 12)
                .unwrap()
                .supported()
        );
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        /// 純白外周と、本文から分離したarea4以下の点で支持を強めない。
        #[test]
        fn component_support_preserves_scale_and_small_isolated_noise(
            scale in 1usize..5, gap in 6usize..25, ink_height in 12usize..25,
            mx in 4usize..12,my in 4usize..12,tone in 0u8..220,
        ) {
            let w=(12+gap)*scale+2*mx;let h=ink_height*scale+2*my;
            let mut rgb=vec![255;w*h*3];
            for y in my..my+ink_height*scale {
                for x in (mx..mx+6*scale).chain(mx+(6+gap)*scale..mx+(12+gap)*scale) {
                    rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);
                }
            }
            let base=measure_component_geometry(&rgb,w,h).unwrap();
            proptest::prop_assert_eq!(base.body_height,ink_height*scale);
            proptest::prop_assert_eq!(base.max_blank_gap,gap*scale);
            proptest::prop_assert_eq!(base.supported(),4*gap<=3*ink_height);
            for y in [0,1,h-2,h-1] {
                for x in [mx+(6+gap/2)*scale,mx+(6+gap/2)*scale+1] {
                    rgb[(y*w+x)*3..(y*w+x+1)*3].fill(0);
                }
            }
            proptest::prop_assert_eq!(measure_component_geometry(&rgb,w,h).unwrap(),base);
        }
    }
}

#[test]
#[ignore = "diagnostic; requires frozen RGB/native matrix root"]
fn record_punctuation_space_component_geometry() {
    let Ok(root) = std::env::var("NDLOCR_PUNCT_SPACE_ROOT") else {
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
        let g = measure_component_geometry(&rgb, w, h).unwrap();
        rows.push(
            serde_json::json!({"id":row["id"],"width":w,"height":h,"body_height":g.body_height,
            "max_blank_gap":g.max_blank_gap,"components":g.components,"supported":g.supported()}),
        );
    }
    assert!(!rows.is_empty());
    std::fs::write(
        root.join("geometry.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    if root.join("native-rows.json").exists() {
        let native: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(root.join("native-rows.json")).unwrap()).unwrap();
        let boundaries: Vec<_> = native
            .iter()
            .map(|r| {
                serde_json::json!({"id":r["id"],"model":r["model"],
            "boundary":punctuation_boundary(r["base"]["text"].as_str().unwrap())})
            })
            .collect();
        std::fs::write(
            root.join("boundaries.json"),
            serde_json::to_vec_pretty(&boundaries).unwrap(),
        )
        .unwrap();
    }
}
