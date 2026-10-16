use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 正解を先に固定した英語記事のコンマと単語を原寸で読む。
fn fixture() -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/latin_punctuation_spaces/comma-words.png",
    ))
    .unwrap()
}

/// コンマに続く実空白を、小さい字形のadvance推定で除去しない。
#[test]
fn comma_word_gaps_remain_word_spaces() {
    let im = fixture();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    for (start, end) in [(342, 359), (1048, 1065)] {
        assert!(
            chunks
                .iter()
                .any(|c| c.space_before && (start..end).contains(&c.x0)),
            "lost real word space: {chunks:?}"
        );
    }
}

/// 固定モデルの出力を変換せず、コンマの後の単語間隔を維持する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn comma_word_spaces_survive_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let im = fixture();
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
        "Amazon, Discord, Dropbox, Google (Alphabet), Meta, and Microsoft. In December 2022, it"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大と左右余白でも実空白と重複・欠落のない分割を維持する。
    #[test]
    fn comma_spaces_survive_scaling(scale in 1usize..4, padding in 0usize..12) {
        let im=fixture();
        let (width,height)=(im.width*scale+padding*2,im.height*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 0..height { for x in 0..im.width*scale {
            let source=((y/scale)*im.width+x/scale)*3;
            let target=(y*width+x+padding)*3;
            rgb[target..target+3].copy_from_slice(&im.data[source..source+3]);
        }}
        let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
        for (start,end) in [(342,359),(1048,1065)] {
            // 区間内の空白は画像に残る。ここで分割した場合だけ、空白属性を要求する。
            proptest::prop_assert!(!chunks.iter().any(|c| !c.space_before && (start*scale+padding..end*scale+padding).contains(&c.x0)));
        }
        proptest::prop_assert_eq!(chunks.first().unwrap().x0,0);
        proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
        proptest::prop_assert!(chunks.windows(2).all(|c| c[0].x1==c[1].x0));
    }
}
