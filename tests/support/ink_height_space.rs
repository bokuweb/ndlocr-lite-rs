#[derive(Debug, PartialEq, Eq)]
struct Geometry {
    foreground_height: usize,
    max_blank_gap: usize,
}

impl Geometry {
    fn supported(&self) -> bool {
        self.max_blank_gap > 0
            && self.max_blank_gap as u128 * 4 <= self.foreground_height as u128 * 3
    }
}

fn measure_geometry(rgb: &[u8], width: usize, height: usize) -> Option<Geometry> {
    if width == 0 || height == 0 || width.checked_mul(height)?.checked_mul(3)? != rgb.len() {
        return None;
    }
    let mut columns = vec![false; width];
    let (mut top, mut bottom) = (height, 0);
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
        if pixel.iter().map(|&v| u16::from(v)).sum::<u16>() < 660 {
            columns[index % width] = true;
            top = top.min(index / width);
            bottom = bottom.max(index / width + 1);
        }
    }
    let left = columns.iter().position(|&ink| ink)?;
    let right = columns.iter().rposition(|&ink| ink)? + 1;
    let max_blank_gap = columns[left..right]
        .split(|&ink| ink)
        .map(<[bool]>::len)
        .max()
        .unwrap_or(0);
    Some(Geometry {
        foreground_height: bottom - top,
        max_blank_gap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(gap: usize, height: usize, margin: usize) -> (Vec<u8>, usize, usize) {
        let width = 12 + gap + 2 * margin;
        let canvas_height = height + 2 * margin;
        let mut rgb = vec![255; width * canvas_height * 3];
        for y in margin..margin + height {
            for x in (margin..margin + 6).chain(margin + 6 + gap..margin + 12 + gap) {
                rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
            }
        }
        (rgb, width, canvas_height)
    }

    /// 紙で画像高が増えても、実空白12px/画線12pxを狭い字間にしない。
    #[test]
    fn paper_margin_does_not_turn_real_space_into_glyph_bearing() {
        for margin in [0, 2, 8] {
            let (rgb, w, h) = sample(12, 12, margin);
            let geometry = measure_geometry(&rgb, w, h).unwrap();
            assert_eq!(geometry.foreground_height, 12);
            assert_eq!(geometry.max_blank_gap, 12);
            assert!(!geometry.supported());
        }
        let (rgb, w, h) = sample(8, 13, 4);
        assert!(measure_geometry(&rgb, w, h).unwrap().supported());
    }

    #[test]
    fn geometry_requires_valid_ink_and_respects_closed_gap_and_tone_boundaries() {
        assert!(measure_geometry(&[], 0, 3).is_none());
        assert!(measure_geometry(&[0; 3], usize::MAX, 2).is_none());
        assert!(measure_geometry(&[0; 5], 2, 1).is_none());
        assert!(measure_geometry(&[255; 36], 3, 4).is_none());
        let contiguous = measure_geometry(&[0; 9 * 8 * 3], 9, 8).unwrap();
        assert_eq!(contiguous.max_blank_gap, 0);
        assert!(!contiguous.supported());
        for (gap, expected) in [(12, true), (13, false)] {
            let (rgb, w, h) = sample(gap, 16, 7);
            assert_eq!(measure_geometry(&rgb, w, h).unwrap().supported(), expected);
        }
        let (rgb, w, h) = sample(8, 13, 3);
        for (color, expected) in [([219, 220, 220], true), ([220; 3], false)] {
            let mut image = rgb.clone();
            for pixel in image.as_chunks_mut::<3>().0 {
                if *pixel == [0; 3] {
                    *pixel = color;
                }
            }
            assert_eq!(measure_geometry(&image, w, h).is_some(), expected);
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        /// 独立した閉形式の矩形を使い、整数拡大・上下左右移動・濃度で支持を変えない。
        #[test]
        fn geometry_is_invariant_to_white_margins_and_integer_scale(
            scale in 1usize..5, gap in 1usize..25, ink_height in 8usize..25,
            left in 0usize..15,right in 0usize..15,top in 0usize..15,bottom in 0usize..15,
            tone in 0u8..220,
        ) {
            let w=(12+gap)*scale+left+right;let h=ink_height*scale+top+bottom;
            let mut rgb=vec![255;w*h*3];
            for y in top..top+ink_height*scale {
                for x in (left..left+6*scale).chain(left+(6+gap)*scale..left+(12+gap)*scale) {
                    rgb[(y*w+x)*3..(y*w+x+1)*3].fill(tone);
                }
            }
            let actual=measure_geometry(&rgb,w,h).unwrap();
            proptest::prop_assert_eq!(actual.foreground_height,ink_height*scale);
            proptest::prop_assert_eq!(actual.max_blank_gap,gap*scale);
            proptest::prop_assert_eq!(actual.supported(),gap*4<=ink_height*3);
        }
    }
}

/// 認識済みRGBそのものを測り、JPEG再decodeや真値を判定に取り込まない。
#[test]
#[ignore = "diagnostic; requires an explicit frozen RGB matrix root"]
fn record_ink_height_space_geometry() {
    let Ok(root) = std::env::var("NDLOCR_INK_SPACE_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let m: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("evaluation-manifest.json")).unwrap())
            .unwrap();
    let mut rows = Vec::new();
    for input in m["inputs"].as_array().unwrap() {
        let rgb = std::fs::read(input["rgb"].as_str().unwrap()).unwrap();
        let w = input["width"].as_u64().unwrap() as usize;
        let h = input["height"].as_u64().unwrap() as usize;
        let g = measure_geometry(&rgb, w, h).unwrap();
        rows.push(serde_json::json!({"id":input["id"],"width":w,"height":h,
            "foreground_height":g.foreground_height,"max_blank_gap":g.max_blank_gap,
            "supported":g.supported()}));
    }
    assert!(!rows.is_empty());
    std::fs::write(
        root.join("geometry.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
