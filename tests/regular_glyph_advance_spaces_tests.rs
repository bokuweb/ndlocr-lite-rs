use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 凍結済み金融画像から原寸で切り出した横書き行を読む。
fn fixture(name: &str) -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(
        &std::path::Path::new("tests/fixtures/regular_glyph_advance").join(format!("{name}.png")),
    )
    .unwrap()
}

/// モデル出力にない仮名内の空白を、字送りの支持で連結時に追加しない。
#[test]
fn regular_advance_does_not_insert_spaces_inside_japanese_words() {
    for (name, unwanted, real_space) in [
        ("announcement", vec![819], Some(1241)),
        ("schedule", vec![615, 875], None),
        ("company", vec![], Some(301)),
    ] {
        let im = fixture(name);
        let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
        for x in unwanted {
            let chunk = chunks
                .iter()
                .find(|chunk| chunk.x0 == x)
                .expect("same boundary");
            assert!(!chunk.space_before, "{name}/{x}: {chunks:?}");
        }
        // 幅が半行高未満の字形は、和英境界の実空白を守るため今回の対象外。
        let protected = match name {
            "schedule" => Some(1283),
            "company" => Some(1315),
            _ => None,
        };
        if let Some(x) = protected {
            assert!(
                chunks
                    .iter()
                    .any(|chunk| chunk.x0 == x && chunk.space_before)
            );
        }
        if let Some(x) = real_space {
            assert!(
                chunks
                    .iter()
                    .any(|chunk| chunk.x0 == x && chunk.space_before)
            );
        }
    }
}

/// 原画素・区間・モデル本文を維持し、追加された連結空白だけを減らす。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn regular_advance_spaces_survive_real_recognition() {
    use ndlocr_lite_rs::{
        infer::{cached::ParseqSession, parseq::ParseqOrientation},
        pipeline::line_chunks::crop_line_chunk_rgb_u8,
    };
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    for (name, expected) in [
        (
            "announcement",
            "当社は、本年1月 1 日に開示いたしました2023年3月期 決算短信にて、第00期有価証券",
        ),
        (
            "schedule",
            "報告書の提出を本日予定としておりましたが、12月30日 の提出予定に変更となりました",
        ),
        (
            "company",
            "本年1月20 日に適時開示しました当社海外子会社のテス トインターナショナルコ",
        ),
    ] {
        let im = fixture(name);
        let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
        // 連結空白だけの改善であり、既存の区間とモデル本文を維持する。
        let expected_chunks: &[&str] = match name {
            "announcement" => &[
                "当社は、本年1月",
                "1",
                "日に開示いた",
                "しま",
                "した2023年3",
                "月期",
                "決算短信にて、",
                "第00期有価証券",
            ],
            "schedule" => &[
                "報告書の提出を",
                "本日予定と",
                "しておりま",
                "したが、12月",
                "30日",
                "の提出予定に変",
                "更となりました",
            ],
            "company" => &[
                "本年1月20",
                "日に適時開示",
                "しました当社海",
                "外子会社のテス",
                "トインターナ",
                "ショナルコ",
            ],
            _ => unreachable!(),
        };
        assert_eq!(chunks.len(), expected_chunks.len(), "{name}");
        let mut actual = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let crop = crop_line_chunk_rgb_u8(&im.data, im.width, im.height, &chunks, i).unwrap();
            let text = session
                .recognize_rgb_u8_unprocessed_with_orientation(
                    &crop.data,
                    crop.width,
                    crop.height,
                    ParseqOrientation::Horizontal,
                )
                .unwrap()
                .text;
            assert_eq!(text, expected_chunks[i], "{name}/{i}: model text");
            if chunk.insert_space_before(&actual, &text, im.height) {
                actual.push(' ');
            }
            actual.push_str(&text);
        }
        assert_eq!(actual, expected, "{name}");
    }
}

/// 規則的な本文でも、章番号の直後の実空白を残す。
#[test]
fn chapter_heading_keeps_its_real_space() {
    let im = fixture("chapter_space");
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    assert!(
        chunks
            .iter()
            .any(|chunk| chunk.x0 == 154 && chunk.space_before),
        "{chunks:?}"
    );
}
