use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 全角の細いＩの余白を単語境界と誤判定し、ＩＣの文脈を切らない。
#[test]
fn fullwidth_narrow_stem_keeps_neighbor_context() {
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/narrow_bearings/line.png",
    ))
    .unwrap();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    assert!(
        chunks.iter().any(|c| c.x0 <= 1419 && c.x1 >= 1492),
        "I/C split: {chunks:?}"
    );
}

/// 固定モデルでもＩＣの文脈を保ち、途中分割による反復を防ぐ。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn narrow_stem_context_reaches_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    use std::path::PathBuf;
    let im = ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/narrow_bearings/line.png",
    ))
    .unwrap();
    let session = ParseqSession::load(
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        &PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let (width, height) = session.input_dimensions();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, width, height).unwrap();
    let mut actual = String::new();
    for chunk in chunks {
        let crop = crop_rgb_u8(
            &im.data,
            im.width,
            im.height,
            BBox::new(chunk.x0, 0, chunk.x1, im.height),
        )
        .unwrap();
        if chunk.space_before {
            actual.push(' ');
        }
        actual.push_str(
            &session
                .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                .unwrap()
                .text,
        );
    }
    eprintln!("NARROW CONTEXT: {actual}");
    // 全角・句読点・字間の誤りは残る。元の正解はsource.jsonと全文CERで評価する。
    assert_eq!(
        actual,
        "という。) にて発行した電子証明書を格納したカード. (以下) 「ICカー ド」という。)"
    );
}
