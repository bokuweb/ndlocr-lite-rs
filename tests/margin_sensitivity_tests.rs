use ndlocr_lite_rs::{
    infer::parseq::{ParseqOrientation, preprocess_rgb_u8_with_orientation},
    io::load_rgb_u8,
    pipeline::horizontal_margins::compact_horizontal_margins,
};
use std::path::Path;

/// 先に固定した原寸と縮小寸法を全fixtureで検査する。
#[test]
fn original_chunks_preserve_rgb_but_change_resampling_only_when_contracting() {
    for (name, width, height, output_width) in [
        ("modern_200", 246, 43, 234),
        ("modern_150", 185, 32, 177),
        ("comments", 203, 34, 200),
        ("japan_we", 341, 43, 340),
        ("culture", 226, 34, 226),
        ("sources", 393, 43, 393),
    ] {
        let original = load_rgb_u8(Path::new(&format!(
            "tests/fixtures/margin_sensitivity/{name}.png"
        )))
        .unwrap();
        assert_eq!((original.width, original.height), (width, height), "{name}");
        let compact =
            compact_horizontal_margins(&original.data, width, height, height / 4).unwrap();
        assert_eq!(
            (compact.image.width, compact.image.height),
            (output_width, height),
            "{name}"
        );
        for y in 0..height {
            for x in 0..width {
                let source = &original.data[(y * width + x) * 3..(y * width + x + 1) * 3];
                if (compact.source_x..compact.source_x + compact.content_width).contains(&x) {
                    let target = x - compact.source_x + compact.output_x;
                    assert_eq!(
                        source,
                        &compact.image.data
                            [(y * output_width + target) * 3..(y * output_width + target + 1) * 3]
                    );
                } else {
                    assert_eq!(source, [255; 3], "{name}: discarded nonwhite");
                }
            }
        }
        let tensor = |rgb: &[u8], w| {
            preprocess_rgb_u8_with_orientation(
                rgb,
                w,
                height,
                256,
                24,
                ParseqOrientation::Horizontal,
            )
            .unwrap()
        };
        assert_eq!(
            tensor(&original.data, width) == tensor(&compact.image.data, output_width),
            width == output_width,
            "{name}"
        );
    }
}
