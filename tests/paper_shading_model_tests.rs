#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::ParseqOrientation},
    pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
        line_segment::detect_textline_bands_naive,
        paper_background::normalize_paper_shading,
    },
};

/// 空白・句読点・改行を維持したUnicodeスカラー単位の厳密編集距離。
fn edit_distance(expected: &str, actual: &str) -> usize {
    let actual: Vec<_> = actual.chars().collect();
    let mut row: Vec<_> = (0..=actual.len()).collect();
    for (i, a) in expected.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &b) in actual.iter().enumerate() {
            let previous = row[j + 1];
            row[j + 1] = (row[j] + 1)
                .min(previous + 1)
                .min(diagonal + usize::from(a != b));
            diagonal = previous;
        }
    }
    row[actual.len()]
}

/// 初回推論前の原文を保ち、先行Redで固定した56編集の半減と5行分離を満たす。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn paper_shading_separates_five_rows_and_halves_initial_errors() {
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/paper_shading/page.png",
    ))
    .unwrap();
    let rgb = normalize_paper_shading(&im.data, im.width, im.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, im.width, im.height, 220);
    assert_eq!(bands.len(), 5);
    let (input_width, input_height) = session.input_dimensions();
    let mut lines = Vec::new();
    for [x0, y0, x1, y1] in bands {
        let line = crop_rgb_u8(&rgb, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks = split_line_for_input(
            &line.data,
            line.width,
            line.height,
            220,
            input_width,
            input_height,
        )
        .unwrap();
        let mut text = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let crop =
                crop_line_chunk_rgb_u8(&line.data, line.width, line.height, &chunks, i).unwrap();
            let part = session
                .recognize_rgb_u8_unprocessed_with_orientation(
                    &crop.data,
                    crop.width,
                    crop.height,
                    ParseqOrientation::Horizontal,
                )
                .unwrap()
                .text;
            if chunk.insert_space_before(&text, &part, line.height) {
                text.push(' ');
            }
            text.push_str(&part);
        }
        lines.push(text);
    }
    let actual = lines.join("\n");
    let truth = include_str!("fixtures/paper_shading/truth.txt")
        .strip_suffix('\n')
        .unwrap();
    assert_eq!(truth.chars().count(), 68);
    let errors = edit_distance(truth, &actual);
    eprintln!("PAPER SHADING {errors}/68: {actual:?}");
    // 完全一致は未達。この開発画像の改善ゲートは本番精度の受入基準ではない。
    assert!(errors <= 28, "must halve initial 56/68: {actual:?}");
    assert!(
        actual.ends_with('。'),
        "must recognize the detached final period: {actual:?}"
    );
    assert!(
        errors < 18,
        "must improve the fixed 18/68 baseline: {actual:?}"
    );
}
