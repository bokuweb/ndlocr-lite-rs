use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 引用符に囲まれた英字の原寸行を読む。
fn fixture(name: &str) -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&format!(
        "tests/fixtures/punctuation_bearings/{name}.png"
    )))
    .unwrap()
}

/// 実モデルでも単語内と句読点直前へ空白を足さない。引用符のUnicode誤読は残る。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn quoted_words_preserve_spacing_in_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let im = fixture("quoted-tight");
    let (width, height) = session.input_dimensions();
    let mut actual = String::new();
    for c in split_line_for_input(&im.data, im.width, im.height, 220, width, height).unwrap() {
        let crop = crop_rgb_u8(
            &im.data,
            im.width,
            im.height,
            BBox::new(c.x0, 0, c.x1, im.height),
        )
        .unwrap();
        let text = session
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text;
        if c.insert_space_before(&actual, &text, im.height) {
            actual.push(' ');
        }
        actual.push_str(&text);
    }
    assert_eq!(
        actual,
        "\"test\" \"test\". \"test\", [\"test\"] (\"test\") {\"test\"} \"test\": \"test\"; \"test\"-\"test\""
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 整数倍拡大・左右余白でも英単語内の空白属性と全画素被覆を維持する。
    #[test]
    fn quoted_word_gaps_survive_scaling(scale in 1usize..4, padding in 0usize..12) {
        let im=fixture("quoted-tight");
        let (width,height)=(im.width*scale+padding*2,im.height*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 0..height { for x in 0..im.width*scale {
            let source=((y/scale)*im.width+x/scale)*3;
            let target=(y*width+x+padding)*3;
            rgb[target..target+3].copy_from_slice(&im.data[source..source+3]);
        }}
        let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
        for (start,end) in [(626,722),(1479,1575),(1680,1775)] {
            proptest::prop_assert!(!chunks.iter().any(|c| c.space_before && (start*scale+padding..end*scale+padding).contains(&c.x0)));
        }
        proptest::prop_assert!(chunks.iter().any(|c| c.space_before && c.gap_before>=30*scale));
        proptest::prop_assert_eq!(chunks.first().unwrap().x0,0);
        proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
        proptest::prop_assert!(chunks.windows(2).all(|c| c[0].x1==c[1].x0));
    }
}

/// 本体の字間へ空白を挿入せず、明示的な広い空白で分割する。
#[test]
fn quoted_words_are_not_split_as_separate_words() {
    for (name, word_ranges) in [
        ("quoted", vec![(1298, 1393)]),
        ("quoted-tight", vec![(626, 722), (1479, 1575), (1680, 1775)]),
    ] {
        let im = fixture(name);
        let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
        for (start, end) in word_ranges {
            assert!(
                !chunks
                    .iter()
                    .any(|c| c.space_before && (start..end).contains(&c.x0)),
                "{name}: {chunks:?}"
            );
        }
        assert!(chunks.iter().any(|c| c.space_before && c.gap_before >= 30));
        assert!(
            chunks.iter().all(|c| !c.space_before || c.gap_before >= 30),
            "{name}: {chunks:?}"
        );
        assert_eq!(chunks.first().unwrap().x0, 0);
        assert_eq!(chunks.last().unwrap().x1, im.width);
        assert!(chunks.windows(2).all(|c| c[0].x1 == c[1].x0));
    }
}
