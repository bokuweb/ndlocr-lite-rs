use ndlocr_lite_rs::{
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::split_line_for_input,
        line_segment::detect_textline_bands_naive,
    },
};

/// 斜体句読点後の実空白を、モデル入力の境界を変えず連結属性へ戻す。
#[test]
fn italic_punctuation_boundary_retains_word_space() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/chromatic_text_contrast/page.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let [x0, y0, x1, y1] = bands[6];
    let line = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
    let chunks = split_line_for_input(&line.data, line.width, line.height, 220, 256, 24).unwrap();
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| (chunk.x0, chunk.x1))
            .collect::<Vec<_>>(),
        [
            (0, 329),
            (329, 856),
            (856, 1324),
            (1324, 1608),
            (1608, 1913)
        ]
    );
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk.space_before)
            .collect::<Vec<_>>(),
        [false, true, false, true, true]
    );
}
