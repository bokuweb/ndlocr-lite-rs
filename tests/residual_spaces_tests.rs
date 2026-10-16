use ndlocr_lite_rs::{io::load_rgb_u8, pipeline::line_chunks::split_line_for_input};
use std::path::Path;

/// コンマ後の実単語空白を、左右の細い字形の余白として消さない。
#[test]
fn paragraph_comma_word_gap_survives_chunk_boundary() {
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/residual_spaces/paragraph-normalized.png",
    ))
    .unwrap();
    let chunks =
        split_line_for_input(&image.data, image.width, image.height, 220, 256, 24).unwrap();
    // 空白の再判定でモデルへ渡す区間そのものを変えない。
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| (chunk.x0, chunk.x1))
            .collect::<Vec<_>>(),
        vec![
            (0, 130),
            (130, 433),
            (433, 776),
            (776, 1071),
            (1071, 1289),
            (1289, 1490),
            (1490, 1747),
            (1747, 1873)
        ]
    );
    assert!(
        !chunks
            .iter()
            .any(|chunk| (427..440).contains(&chunk.x0) && !chunk.space_before),
        "{chunks:?}"
    );
}

/// 字幅より狭い見出しの実単語間隔を、字間と取り違えない。
#[test]
fn heading_word_gap_survives_chunk_boundary() {
    let image = load_rgb_u8(Path::new(
        "tests/fixtures/residual_spaces/heading-normalized.png",
    ))
    .unwrap();
    let chunks =
        split_line_for_input(&image.data, image.width, image.height, 220, 256, 24).unwrap();
    // 空白の再判定でモデルへ渡す区間そのものを変えない。
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| (chunk.x0, chunk.x1))
            .collect::<Vec<_>>(),
        vec![(0, 250), (250, 451), (451, 580), (580, 829)]
    );
    assert!(
        !chunks
            .iter()
            .any(|chunk| (244..256).contains(&chunk.x0) && !chunk.space_before),
        "{chunks:?}"
    );
}

/// 固定モデルが認識した単語を、画像上の単語空白を保持して連結する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn recognized_residual_lines_preserve_original_word_spaces() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession, pipeline::line_chunks::crop_line_chunk_rgb_u8,
    };
    let model = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL").unwrap());
    let charset = std::path::PathBuf::from(std::env::var_os("NDLOCR_TEST_CHARSET").unwrap());
    let session = ParseqSession::load(&model, &charset).unwrap();
    for (name, expected) in [
        (
            "paragraph",
            "turpis. Volutpat nibh, nunc magna non diam euismod sed dolore non diam nibh tempus donec",
        ),
        ("heading", "Introduction The Importance of Wetlands"),
    ] {
        let image = load_rgb_u8(Path::new(&format!(
            "tests/fixtures/residual_spaces/{name}-normalized.png"
        )))
        .unwrap();
        let chunks =
            split_line_for_input(&image.data, image.width, image.height, 220, 256, 24).unwrap();
        let mut actual = String::new();
        for (index, chunk) in chunks.iter().enumerate() {
            let crop =
                crop_line_chunk_rgb_u8(&image.data, image.width, image.height, &chunks, index)
                    .unwrap();
            let result = session
                .recognize_rgb_u8_unprocessed(&crop.data, crop.width, crop.height)
                .unwrap();
            if chunk.insert_space_before(&actual, &result.text, image.height) {
                actual.push(' ');
            }
            actual.push_str(&result.text);
        }
        assert_eq!(actual, expected, "{name}");
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大・平行移動でも、実空白で切る場合は空白属性を保持し全画素を覆う。
    #[test]
    fn residual_spaces_survive_scale_and_translation(scale in 1usize..4, padding in 0usize..12, heading in proptest::bool::ANY) {
        let (name, start, end) = if heading { ("heading",244,256) } else { ("paragraph",427,440) };
        let image=load_rgb_u8(Path::new(&format!("tests/fixtures/residual_spaces/{name}-normalized.png"))).unwrap();
        let (width,height)=(image.width*scale+2*padding,image.height*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 0..height { for x in 0..image.width*scale {
            let source=((y/scale)*image.width+x/scale)*3;
            let target=(y*width+x+padding)*3;
            rgb[target..target+3].copy_from_slice(&image.data[source..source+3]);
        }}
        let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
        proptest::prop_assert!(!chunks.iter().any(|chunk| !chunk.space_before && (start*scale+padding..end*scale+padding).contains(&chunk.x0)));
        proptest::prop_assert_eq!(chunks.first().unwrap().x0,0);
        proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
        proptest::prop_assert!(chunks.windows(2).all(|chunks|chunks[0].x1==chunks[1].x0));
    }
}
