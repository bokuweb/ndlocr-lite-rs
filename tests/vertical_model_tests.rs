#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::ParseqOrientation},
    io::RgbImageData,
    pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::{
            crop_line_chunk_rgb_u8, split_line_for_input, split_rotated_vertical_column_for_input,
        },
        line_segment::detect_textline_bands_naive,
        paper_background::normalize_paper_shading,
        vertical_segment::{detect_vertical_textline_columns, rotate_vertical_column_rgb_u8},
    },
};

/// 空白・句読点・列区切りを維持するUnicodeスカラー単位の距離。
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

/// 現在の横書き検出・分割・Horizontal推論を同じ画素で評価する。
fn horizontal_baseline(session: &ParseqSession, im: &RgbImageData) -> Result<String, String> {
    let (iw, ih) = session.input_dimensions();
    let mut lines = Vec::new();
    for [x0, y0, x1, y1] in detect_textline_bands_naive(&im.data, im.width, im.height, 220) {
        let crop = crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks = split_line_for_input(&crop.data, crop.width, crop.height, 220, iw, ih)?;
        let mut text = String::new();
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
            if chunk.insert_space_before(&text, &result.text, crop.height) {
                text.push(' ');
            }
            text.push_str(&result.text);
        }
        lines.push(text);
    }
    Ok(lines.join("\n"))
}

/// 先行固定した6列と原文で、横書き経路からの全文改善を検証する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn explicit_vertical_columns_improve_frozen_full_page_truth() {
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let mut im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/vertical_handwriting/page.png",
    ))
    .unwrap();
    let truth = include_str!("fixtures/vertical_handwriting/truth.txt")
        .strip_suffix('\n')
        .unwrap();
    assert_eq!((truth.lines().count(), truth.chars().count()), (6, 83));
    let raw = horizontal_baseline(&session, &im).unwrap();
    let raw_errors = edit_distance(truth, &raw);
    eprintln!("RAW HORIZONTAL {raw_errors}/83: {raw:?}");
    im.data = normalize_paper_shading(&im.data, im.width, im.height)
        .unwrap()
        .into_owned();
    let horizontal = horizontal_baseline(&session, &im);
    let horizontal_errors = horizontal
        .as_ref()
        .ok()
        .map(|text| edit_distance(truth, text));
    eprintln!("PREPARED HORIZONTAL {horizontal_errors:?}/83: {horizontal:?}");
    let columns = detect_vertical_textline_columns(&im.data, im.width, im.height, 220).unwrap();
    let mut lines = Vec::new();
    for &[x0, y0, x1, y1] in &columns {
        let crop = crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let result = session
            .recognize_rgb_u8_unprocessed_with_orientation(
                &crop.data,
                crop.width,
                crop.height,
                ParseqOrientation::Auto,
            )
            .unwrap();
        let rotated = rotate_vertical_column_rgb_u8(&crop.data, crop.width, crop.height).unwrap();
        let (iw, ih) = session.input_dimensions();
        let chunks = split_rotated_vertical_column_for_input(
            &rotated.data,
            rotated.width,
            rotated.height,
            220,
            iw,
            ih,
        )
        .unwrap();
        let mut chunked = String::new();
        for i in 0..chunks.len() {
            let part =
                crop_line_chunk_rgb_u8(&rotated.data, rotated.width, rotated.height, &chunks, i)
                    .unwrap();
            chunked.push_str(
                &session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &part.data,
                        part.width,
                        part.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap()
                    .text,
            );
        }
        assert_eq!(
            chunked, result.text,
            "bounded vertical path must preserve full-column context"
        );
        lines.push(result.text);
    }
    let actual = lines.join("\n");
    let errors = edit_distance(truth, &actual);
    eprintln!("PREPARED VERTICAL {errors}/83: {actual:?}");
    assert_eq!(columns.len(), 6);
    assert!(errors < raw_errors);
    if let Some(horizontal_errors) = horizontal_errors {
        assert!(errors < horizontal_errors);
    }
    assert!(
        lines[0].ends_with('。'),
        "must retain the first column period"
    );
    assert!(errors <= 4, "must retain the fixed 4/83 vertical baseline");
}

/// 全画像と検出後の切り出しの両方で、二・三の内部白帯を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn long_vertical_columns_preserve_disconnected_numerals() {
    assert_long_vertical_columns(false, "long_vertical");
}

/// omoと同じ列検出・切り出し経路でも全文を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn detected_long_columns_preserve_disconnected_numerals() {
    assert_long_vertical_columns(true, "long_vertical");
}

/// 固定画像の原文を推論へ渡さず、指定経路の全文認識を検証する。
fn assert_long_vertical_columns(detect_column: bool, fixture: &str) {
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("tests/fixtures/{fixture}/manifest.json")).unwrap(),
    )
    .unwrap();
    let (iw, ih) = session.input_dimensions();
    let mut failures = Vec::new();
    for item in manifest["items"].as_array().unwrap() {
        let path = std::path::Path::new("tests/fixtures")
            .join(fixture)
            .join(item["image"].as_str().unwrap());
        let im = ndlocr_lite_rs::io::load_rgb_u8(&path).unwrap();
        let cropped = if detect_column {
            let columns =
                detect_vertical_textline_columns(&im.data, im.width, im.height, 220).unwrap();
            assert_eq!(columns.len(), 1);
            let [x0, y0, x1, y1] = columns[0];
            crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap()
        } else {
            crop_rgb_u8(
                &im.data,
                im.width,
                im.height,
                BBox::new(0, 0, im.width, im.height),
            )
            .unwrap()
        };
        let r =
            rotate_vertical_column_rgb_u8(&cropped.data, cropped.width, cropped.height).unwrap();
        let chunks =
            split_rotated_vertical_column_for_input(&r.data, r.width, r.height, 220, iw, ih)
                .unwrap();
        let mut actual = String::new();
        for i in 0..chunks.len() {
            let crop = crop_line_chunk_rgb_u8(&r.data, r.width, r.height, &chunks, i).unwrap();
            actual.push_str(
                &session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &crop.data,
                        crop.width,
                        crop.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap()
                    .text,
            );
        }
        let truth = item["truth"].as_str().unwrap();
        eprintln!(
            "{} detected={detect_column}: {}/{}, {actual:?}",
            item["id"],
            edit_distance(truth, &actual),
            truth.chars().count()
        );
        if actual != truth {
            failures.push(item["id"].clone());
        }
    }
    assert!(failures.is_empty(), "long-column regressions: {failures:?}");
}

/// 太字と明朝体の二・三も、全画像と列切り出し後の双方で全文を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn alternate_fonts_preserve_disconnected_numerals() {
    assert_long_vertical_columns(false, "vertical_fonts");
}

/// 検出後に列幅が狭くなっても、別書体の全文を保持する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn detected_alternate_fonts_preserve_disconnected_numerals() {
    assert_long_vertical_columns(true, "vertical_fonts");
}

/// 推論前に固定した印刷本文全体を、検出・切り出し・縦列分割から認識する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn stacked_printed_body_preserves_full_text_and_reading_order() {
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/stacked_vertical/body.png",
    ))
    .unwrap();
    let truth = include_str!("fixtures/stacked_vertical/truth.txt")
        .strip_suffix('\n')
        .unwrap();
    assert_eq!((truth.lines().count(), truth.chars().count()), (47, 1044));
    let start = std::time::Instant::now();
    let columns = detect_vertical_textline_columns(&im.data, im.width, im.height, 220).unwrap();
    let (iw, ih) = session.input_dimensions();
    let mut lines = Vec::new();
    let mut calls = 0;
    for &[x0, y0, x1, y1] in &columns {
        let crop = crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let r = rotate_vertical_column_rgb_u8(&crop.data, crop.width, crop.height).unwrap();
        let chunks =
            split_rotated_vertical_column_for_input(&r.data, r.width, r.height, 220, iw, ih)
                .unwrap();
        let mut text = String::new();
        for i in 0..chunks.len() {
            let part = crop_line_chunk_rgb_u8(&r.data, r.width, r.height, &chunks, i).unwrap();
            text.push_str(
                &session
                    .recognize_rgb_u8_unprocessed_with_orientation(
                        &part.data,
                        part.width,
                        part.height,
                        ParseqOrientation::Horizontal,
                    )
                    .unwrap()
                    .text,
            );
            calls += 1;
        }
        lines.push(text);
    }
    let actual = lines.join("\n");
    let errors = edit_distance(truth, &actual);
    eprintln!(
        "STACKED FULL {errors}/1044 columns={} calls={calls} elapsed_ms={} text={actual:?}",
        columns.len(),
        start.elapsed().as_millis()
    );
    assert_eq!(columns.len(), 47);
    assert!(
        errors * 10 <= 1044,
        "fixed whole-body error rate must be <=10%"
    );
}
