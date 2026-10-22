#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::ParseqOrientation},
    io::load_rgb_u8,
    pipeline::{
        background::normalize_chromatic_panels,
        crop::{BBox, crop_rgb_u8},
        line_chunks::{LineChunk, crop_line_chunk_rgb_u8, split_line_for_input},
        line_segment::detect_textline_bands_naive,
    },
};

/// 新規資料の下線付きURLを、原画素と固定モデルで回帰検査する。
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn kobe_notice_recovers_underlined_url_context() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let rgb = normalize_chromatic_panels(&image.data, image.width, image.height).unwrap();
    let bands = detect_textline_bands_naive(&rgb, image.width, image.height, 220);
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let (iw, ih) = session.input_dimensions();
    let mut lines = Vec::new();
    let mut calls = 0;
    let mut unsplit = 0;
    for (row, &[x0, y0, x1, y1]) in bands.iter().enumerate() {
        let band = crop_rgb_u8(&rgb, image.width, image.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks = split_line_for_input(&band.data, band.width, band.height, 220, iw, ih)
            .unwrap_or_else(|error| {
                unsplit += 1;
                eprintln!("UNSPLIT ROW {row} BOUNDS {:?} {error}", [x0, y0, x1, y1]);
                vec![LineChunk {
                    x0: 0,
                    x1: band.width,
                    space_before: false,
                    gap_before: 0,
                }]
            });
        let mut text = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let crop =
                crop_line_chunk_rgb_u8(&band.data, band.width, band.height, &chunks, i).unwrap();
            let result = session
                .recognize_rgb_u8_unprocessed_with_orientation(
                    &crop.data,
                    crop.width,
                    crop.height,
                    ParseqOrientation::Horizontal,
                )
                .unwrap();
            eprintln!(
                "ROW {row} CHUNK {i} {chunk:?} {}x{} {:?}",
                crop.width, crop.height, result.text
            );
            if chunk.insert_space_before(&text, &result.text, band.height) {
                text.push(' ');
            }
            text.push_str(&result.text);
            calls += 1;
        }
        eprintln!("ROW {row} BOUNDS {:?} TEXT {text:?}", [x0, y0, x1, y1]);
        lines.push(text);
    }
    let actual = lines.join("\n") + "\n";
    if let Some(output) = std::env::var_os("NDLOCR_FOLLOWUP_OUTPUT") {
        std::fs::write(output, &actual).unwrap();
    }
    eprintln!("CALLS {calls} BANDS {}", bands.len());
    if std::env::var_os("NDLOCR_FOLLOWUP_REQUIRE_EXACT").is_some() {
        assert_eq!(
            actual,
            include_str!("fixtures/eval/truth/jma_kobe_notice.txt")
        );
    }
    assert_eq!(unsplit, 0, "no unbounded fallback inference");
    assert_eq!(lines.len(), 32);
    assert!(lines[5].starts_with("令和8年3月17日("), "{}", lines[5]);
    assert!(lines[7].ends_with("気象警報・注意"), "{}", lines[7]);
    assert_eq!(lines[28], "1 https://www.jma-net.go.jp/kobe-");
    for row in [29, 31] {
        assert!(
            lines[row].starts_with("c/press/2025/2025122401/"),
            "{}",
            lines[row]
        );
        assert!(lines[row].ends_with("henkou.pdf"), "{}", lines[row]);
    }
}
