use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 実空白後にモデルに収まる語を含む原寸見出しを読む。
fn fixture() -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/word_tail_context/performance.png",
    ))
    .unwrap()
}

/// 番号後の一語が入力比率に収まるなら、それ以上分割しない。
#[test]
fn fitting_word_tail_preserves_context() {
    let im = fixture();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    assert_eq!(chunks.len(), 2, "{chunks:?}");
    assert_eq!((chunks[1].x0, chunks[1].x1), (48, 411));
    assert!(chunks[1].space_before);
    assert_eq!(chunks[1].gap_before, 43);
}

/// 同じモデルに末尾の語全体を渡し、小書き仮名を認識する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn full_word_reaches_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let im = fixture();
    let s = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let mut actual = String::new();
    let (width, height) = s.input_dimensions();
    for c in split_line_for_input(&im.data, im.width, im.height, 220, width, height).unwrap() {
        let crop = crop_rgb_u8(
            &im.data,
            im.width,
            im.height,
            BBox::new(c.x0, 0, c.x1, im.height),
        )
        .unwrap();
        let text = s
            .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
            .unwrap()
            .text;
        if c.insert_space_before(&actual, &text, im.height) {
            actual.push(' ');
        }
        actual.push_str(&text);
    }
    assert_eq!(actual, "1. パフォーマンス");
}

/// 入力比率の等号境界を守り、1pxでも収まらない末尾は結合しない。
#[test]
fn word_tail_obeys_exact_model_capacity() {
    let im = fixture();
    for (input_width, expected) in [(242, 2), (241, 3)] {
        let chunks =
            split_line_for_input(&im.data, im.width, im.height, 220, input_width, 24).unwrap();
        assert_eq!(chunks.len(), expected, "{input_width}: {chunks:?}");
    }
}

/// 字形数上限に達した末尾と単語空白のない行では従来分割を維持する。
#[test]
fn dense_tails_and_absent_word_boundaries_keep_conservative_chunks() {
    use ndlocr_lite_rs::pipeline::line_chunks::split_line;
    for runs in [20, 21] {
        let mut columns = vec![0u8; 411];
        columns[12..56].fill(255);
        for i in 1..runs {
            columns[56 + i * 16] = 255;
        }
        let rgb: Vec<u8> = (0..36)
            .flat_map(|_| columns.iter().flat_map(|value| [*value; 3]))
            .collect();
        let chunks = split_line_for_input(&rgb, 411, 36, 220, 256, 24).unwrap();
        if runs == 20 {
            assert_eq!(chunks.len(), 2);
        } else {
            assert_eq!(chunks, split_line(&rgb, 411, 36, 220).unwrap());
        }
    }
    let rgb: Vec<u8> = (0..36)
        .flat_map(|_| (0..411).flat_map(|x| [if x % 40 == 39 { 255 } else { 0 }; 3]))
        .collect();
    assert_eq!(
        split_line_for_input(&rgb, 411, 36, 220, 256, 24).unwrap(),
        split_line(&rgb, 411, 36, 220).unwrap()
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大と水平移動でも末尾の文脈・空白属性・全画素被覆を維持する。
    #[test]
    fn word_tail_survives_scale_and_translation(scale in 1usize..4,padding in 0usize..10) {
        let im=fixture();
        let (width,height)=(im.width*scale+padding*2,im.height*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 0..height {for x in 0..im.width*scale {
            let source=((y/scale)*im.width+x/scale)*3;
            let target=(y*width+x+padding)*3;
            rgb[target..target+3].copy_from_slice(&im.data[source..source+3]);
        }}
        let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
        proptest::prop_assert_eq!(chunks.len(),2);
        proptest::prop_assert_eq!((chunks[0].x0,chunks[0].x1,chunks[1].x0,chunks[1].x1),(0,27*scale+(43*scale)/2+padding,27*scale+(43*scale)/2+padding,width));
        proptest::prop_assert!(chunks[1].space_before);
        proptest::prop_assert_eq!(chunks[1].gap_before,43*scale);
    }
}
