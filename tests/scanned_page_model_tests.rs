#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::ParseqOrientation},
    pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
        line_segment::detect_textline_bands_naive,
    },
};

/// 改行以外の文字を保持したUnicode編集距離を求める。
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

/// 固定スキャンの物理行を分け、固定モデルの容量に収まる白列区間で認識する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn scanned_page_bounded_ocr_preserves_frozen_text() {
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let image = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/scaned0.png",
    ))
    .unwrap();
    let boxes = detect_textline_bands_naive(&image.data, image.width, image.height, 220);
    let (iw, ih) = session.input_dimensions();
    let mut lines = Vec::new();
    for &[x0, y0, x1, y1] in &boxes {
        let crop = crop_rgb_u8(
            &image.data,
            image.width,
            image.height,
            BBox::new(x0, y0, x1, y1),
        )
        .unwrap();
        let chunks =
            split_line_for_input(&crop.data, crop.width, crop.height, 220, iw, ih).unwrap();
        let mut line = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let part =
                crop_line_chunk_rgb_u8(&crop.data, crop.width, crop.height, &chunks, i).unwrap();
            let result = session
                .recognize_rgb_u8_unprocessed_with_orientation(
                    &part.data,
                    part.width,
                    part.height,
                    ParseqOrientation::Horizontal,
                )
                .unwrap();
            if chunk.insert_space_before(&line, &result.text, crop.height) {
                line.push(' ');
            }
            line.push_str(&result.text);
        }
        lines.push(line);
    }
    let actual = lines.join("\n");
    let expected = include_str!("fixtures/eval/truth/scaned0.txt").replace('\n', "");
    let errors = edit_distance(&expected, &actual.replace('\n', ""));
    eprintln!(
        "SCANNED PAGE {errors}/{} bands={boxes:?} text={actual:?}",
        expected.chars().count()
    );
    assert_eq!(boxes.len(), 7);
    assert!(errors <= 20, "{errors} edits: {actual}");
    assert!(actual.contains("第3条") && actual.contains("対馬"));
}
