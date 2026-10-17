use ndlocr_lite_rs::pipeline::{
    crop::{BBox, crop_rgb_u8},
    line_chunks::split_rotated_vertical_column_for_input,
    vertical_segment::{detect_vertical_textline_columns, rotate_vertical_column_rgb_u8},
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// 太い長画線も拡大・移動にかかわらず一字幅の内側でまとめる。
    #[test]
    fn thick_strokes_preserve_glyph_boundaries(scale in 1usize..4, padding in 0usize..16, thickness in 4usize..7) {
        let height = 40 * scale;
        let width = 31 * 42 * scale + padding * 2;
        let mut rgb = vec![255; width * height * 3];
        for n in 0..31 {
            for dx in 0..34 * scale {
                if n == 15 && !(dx < thickness * scale || (12 * scale..(12 + thickness) * scale).contains(&dx) || (24 * scale..(24 + thickness) * scale).contains(&dx)) {
                    continue;
                }
                let x = padding + n * 42 * scale + dx;
                for y in 4 * scale..36 * scale {
                    rgb[(y * width + x) * 3..(y * width + x + 1) * 3].fill(0);
                }
            }
        }
        let start = padding + 15 * 42 * scale;
        let end = start + (24 + thickness) * scale;
        let chunks = split_rotated_vertical_column_for_input(&rgb, width, height, 220, 256, 24).unwrap();
        for chunk in chunks.iter().skip(1) {
            prop_assert!(chunk.x0 <= start || chunk.x0 >= end, "split {} inside [{}, {})", chunk.x0, start, end);
        }
    }
}

/// 太字と明朝体の二・三の描画矩形を分割しない。矩形は採点だけに使う。
#[test]
fn alternate_fonts_do_not_split_inside_glyphs() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/vertical_fonts/manifest.json")).unwrap();
    let mut failures = Vec::new();
    for item in manifest["items"].as_array().unwrap() {
        let path = std::path::Path::new("tests/fixtures/vertical_fonts")
            .join(item["image"].as_str().unwrap());
        let im = ndlocr_lite_rs::io::load_rgb_u8(&path).unwrap();
        for detected in [false, true] {
            let bbox = if detected {
                let columns =
                    detect_vertical_textline_columns(&im.data, im.width, im.height, 220).unwrap();
                assert_eq!(columns.len(), 1);
                columns[0]
            } else {
                [0, 0, im.width, im.height]
            };
            let [x0, y0, x1, y1] = bbox;
            let crop =
                crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
            let r = rotate_vertical_column_rgb_u8(&crop.data, crop.width, crop.height).unwrap();
            let chunks =
                split_rotated_vertical_column_for_input(&r.data, r.width, r.height, 220, 256, 24)
                    .unwrap();
            for chunk in chunks.iter().skip(1) {
                let boundary = chunk.x0 + y0;
                for bbox in item["glyph_boxes"].as_array().unwrap() {
                    let top = bbox[1].as_u64().unwrap() as usize;
                    let bottom = bbox[3].as_u64().unwrap() as usize;
                    if boundary > top && boundary < bottom {
                        failures.push(format!(
                            "{} detected={detected}: {boundary} inside [{top}, {bottom})",
                            item["id"]
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
