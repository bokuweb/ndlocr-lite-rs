#![cfg(feature = "onnx")]

use ndlocr_lite_rs::{
    infer::{cached::ParseqSession, parseq::sanitize_recognized_text},
    io::load_rgb_u8,
    pipeline::crop::{BBox, crop_rgb_u8},
};

/// 同じDEIM cropには閉じ括弧があり、従来の単行後処理でだけ削除される。
#[test]
#[ignore = "requires pinned PARSeq assets"]
fn raw_continuation_contains_the_closing_bracket() {
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let crop = crop_rgb_u8(
        &image.data,
        image.width,
        image.height,
        BBox::new(48, 558, 134, 581),
    )
    .unwrap();
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let raw = session
        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
        .unwrap();
    let legacy = sanitize_recognized_text(&raw.text);
    eprintln!("RAW {:?} LEGACY {:?}", raw.text, legacy);
    assert!(raw.text.contains(')'));
    assert!(!legacy.contains(')'));
}

/// 既定のページCLIは、前行で開かれた括弧を継続行から削除しない。
#[test]
#[ignore = "requires pinned DEIM and all PARSeq assets"]
fn page_cli_preserves_the_literal_continuation_closer() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ndlocr-lite-rs"))
        .args([
            "recognize-page",
            "--image",
            "tests/fixtures/eval/images/jma_kobe_notice.png",
            "--charset",
            "models/NDLmoji.yaml",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.lines().any(|line| line == "り1.2)。"),
        "continuation closer missing: {text}"
    );
}

/// 両方のページruntime構築経路でも同じ断片方針を使う。
#[test]
#[ignore = "requires pinned DEIM/PARSeq assets"]
fn page_runtime_constructors_preserve_closers_and_confidence() {
    use ndlocr_lite_rs::infer::{
        cached::ParseqPool,
        deim_cached::DeimPool,
        page_pool::{PageRecognizer, PageRecognizerOptions},
        parseq::sanitize_recognized_fragment,
    };
    let det = std::path::PathBuf::from(std::env::var("NDLOCR_TEST_DEIM_MODEL").unwrap());
    let model = std::path::PathBuf::from(std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = std::path::PathBuf::from(std::env::var("NDLOCR_TEST_CHARSET").unwrap());
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let crop = crop_rgb_u8(
        &image.data,
        image.width,
        image.height,
        BBox::new(48, 558, 134, 581),
    )
    .unwrap();
    let expected = ParseqSession::load(&model, &charset)
        .unwrap()
        .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
        .unwrap();
    let load = PageRecognizer::load(&det, &model, &charset, 1).unwrap();
    let from_pools = PageRecognizer::from_pools(
        DeimPool::load(&det, 1).unwrap(),
        ParseqPool::load(&model, &charset, 1).unwrap(),
    );
    for runtime in [load, from_pools] {
        let page = runtime
            .recognize_page_rgb_u8(
                &image.data,
                image.width,
                image.height,
                PageRecognizerOptions {
                    det_conf_threshold: 0.25,
                    min_line_confidence: 0.0,
                    drop_empty_text: false,
                },
            )
            .unwrap();
        let target = page
            .lines
            .iter()
            .find(|line| line.bbox_xyxy == [48, 558, 134, 581])
            .unwrap();
        assert_eq!(target.text, sanitize_recognized_fragment(&expected.text));
        assert!((target.confidence - expected.mean_confidence).abs() < 1e-5);
    }
}

/// 公開の単行Poolと両batch APIは従来の本文・順序・信頼度を保持する。
#[test]
#[ignore = "requires pinned PARSeq assets"]
fn legacy_pool_batch_paths_keep_their_existing_text_policy() {
    use ndlocr_lite_rs::infer::cached::ParseqPool;
    let model = std::path::PathBuf::from(std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = std::path::PathBuf::from(std::env::var("NDLOCR_TEST_CHARSET").unwrap());
    let image = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/eval/images/jma_kobe_notice.png",
    ))
    .unwrap();
    let crop = crop_rgb_u8(
        &image.data,
        image.width,
        image.height,
        BBox::new(48, 558, 134, 581),
    )
    .unwrap();
    let repeated = load_rgb_u8(std::path::Path::new(
        "tests/fixtures/parseq_unprocessed/repeated_digits.png",
    ))
    .unwrap();
    let images = [
        (&crop.data[..], crop.width, crop.height),
        (&repeated.data[..], repeated.width, repeated.height),
        (&crop.data[..], crop.width, crop.height),
    ];
    let session = ParseqSession::load(&model, &charset).unwrap();
    let expected: Vec<_> = images
        .iter()
        .map(|(rgb, w, h)| session.recognize_rgb_u8(rgb, *w, *h).unwrap())
        .collect();
    assert_eq!(expected[0].text, "り1.2。");
    assert_eq!(expected[1].text, "1111");
    let pool = ParseqPool::load(&model, &charset, 2).unwrap();
    for actual in [
        images
            .iter()
            .map(|(rgb, w, h)| pool.recognize_rgb_u8(rgb, *w, *h).unwrap())
            .collect::<Vec<_>>(),
        pool.recognize_batch_rgb_u8(&images).unwrap(),
        pool.recognize_batch_single_session_rgb_u8(&images).unwrap(),
    ] {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(&expected) {
            assert_eq!(a.text, b.text);
            assert!((a.mean_confidence - b.mean_confidence).abs() < 1e-5);
        }
    }
    assert!(pool.recognize_batch_rgb_u8(&[(&[], 0, 0)]).is_err());
    assert_eq!(
        pool.recognize_rgb_u8(&crop.data, crop.width, crop.height)
            .unwrap()
            .text,
        expected[0].text
    );
}
