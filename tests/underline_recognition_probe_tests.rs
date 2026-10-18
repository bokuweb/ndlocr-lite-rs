#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::ParseqOrientation},
    pipeline::{
        crop::{BBox, crop_rgb_u8},
        line_chunks::{crop_line_chunk_rgb_u8, split_line_for_input},
        line_segment::detect_textline_bands_naive,
    },
};

/// 同じ座標・寸法の原RGBで残るURL誤認を先に再現する。
#[test]
#[ignore = "diagnostic only; requires fixed PARSeq assets"]
fn underlined_urls_preserve_literal_punctuation() {
    let im = image::open("tests/fixtures/eval/images/jma_kobe_notice.png")
        .unwrap()
        .to_rgb8();
    let (w, h) = (im.width() as usize, im.height() as usize);
    let bands = detect_textline_bands_naive(im.as_raw(), w, h, 220);
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let truth: Vec<_> = include_str!("fixtures/eval/truth/jma_kobe_notice.txt")
        .lines()
        .collect();
    let mut outputs = Vec::new();
    for row in 28..32 {
        let [x0, y0, x1, y1] = bands[row];
        let band = crop_rgb_u8(im.as_raw(), w, h, BBox::new(x0, y0, x1, y1)).unwrap();
        let chunks =
            split_line_for_input(&band.data, band.width, band.height, 220, 256, 24).unwrap();
        let mut candidate = band.clone();
        if std::env::var_os("NDLOCR_UNDERLINE_PROBE_REMOVE_RULE").is_some() {
            for y in band.height - band.height / 4..band.height {
                let mut start = 0;
                for x in 0..=band.width {
                    let ink = x < band.width
                        && band.data[(y * band.width + x) * 3..(y * band.width + x + 1) * 3]
                            .iter()
                            .map(|&v| u16::from(v))
                            .sum::<u16>()
                            < 660;
                    if ink {
                        continue;
                    }
                    if x - start >= 8 * band.height {
                        candidate.data[(y * band.width + start) * 3..(y * band.width + x) * 3]
                            .fill(255);
                    }
                    start = x + 1;
                }
            }
        }
        let mut actual = String::new();
        for (index, chunk) in chunks.iter().enumerate() {
            let crop = crop_line_chunk_rgb_u8(&band.data, band.width, band.height, &chunks, index)
                .unwrap();
            let source = crop;
            let crop = crop_line_chunk_rgb_u8(
                &candidate.data,
                candidate.width,
                candidate.height,
                &chunks,
                index,
            )
            .unwrap();
            assert_eq!(
                (crop.width, crop.height),
                (source.width, source.height),
                "fixed model geometry"
            );
            let result = session
                .recognize_rgb_u8_unprocessed_with_orientation(
                    &crop.data,
                    crop.width,
                    crop.height,
                    ParseqOrientation::Horizontal,
                )
                .unwrap();
            if chunk.insert_space_before(&actual, &result.text, band.height) {
                actual.push(' ');
            }
            actual.push_str(&result.text);
        }
        // 真値の全角footnote番号だけを表示幅の差として揃える。
        let expected = truth[row].replace('１', "1").replace('２', "2");
        eprintln!("URL {row}: {actual:?} EXPECTED {expected:?}");
        outputs.push((actual, expected));
    }
    if let Some(path) = std::env::var_os("NDLOCR_UNDERLINE_PROBE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&outputs).unwrap()).unwrap();
    }
    // 完全一致の探索Redは明示時だけ実行し、既定の実モデル回帰と区別する。
    if std::env::var_os("NDLOCR_UNDERLINE_PROBE_REQUIRE_EXACT").is_some() {
        assert!(outputs.iter().all(|(actual, expected)| actual == expected));
    }
    assert_eq!(outputs.len(), 4);
    if std::env::var_os("NDLOCR_UNDERLINE_PROBE_REMOVE_RULE").is_some() {
        assert_eq!(
            outputs[1].0, outputs[1].1,
            "the long path recovers underscores"
        );
        assert_ne!(
            outputs[0].0, outputs[0].1,
            "the correct short URL regresses"
        );
    } else {
        assert_eq!(outputs[0].0, outputs[0].1, "the raw short URL is correct");
        assert_ne!(
            outputs[1].0, outputs[1].1,
            "the raw long path loses underscores"
        );
    }
}
