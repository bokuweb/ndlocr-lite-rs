use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 全高で細い「り」を含む原寸行を読み込む。
fn fixture() -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(
        "tests/fixtures/tall_kana_bearings/ownership.png",
    ))
    .unwrap()
}

/// 仮名の字幅余白に空白属性を付けず、元の全画素を保持する。
#[test]
fn tall_kana_bearing_does_not_insert_word_space() {
    let im = fixture();
    let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
    assert!(
        !chunks
            .iter()
            .any(|c| c.space_before && (1237..=1258).contains(&c.x0)),
        "{chunks:?}"
    );
    assert_eq!(chunks.first().unwrap().x0, 0);
    assert_eq!(chunks.last().unwrap().x1, im.width);
    assert!(chunks.windows(2).all(|c| c[0].x1 == c[1].x0));
}

/// 固定モデルで「により」の間に存在しない空白を追加しない。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn tall_kana_context_reaches_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let im = fixture();
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    let mut actual = String::new();
    let (width, height) = session.input_dimensions();
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
        "(ア) Rust の豊かな型システムと所有権モデルによりメモリ安全性と"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大・平行移動・左右反転でも仮名の字間に空白属性を付けない。
    #[test]
    fn kana_bearing_survives_geometry_changes(scale in 1usize..4, padding in 0usize..10, reflect in proptest::bool::ANY) {
        let im=fixture();
        let (width,height)=(im.width*scale+padding*2,im.height*scale);
        let mut rgb=vec![255; width*height*3];
        for y in 0..height { for x in 0..im.width*scale {
            let column=if reflect { im.width-1-x/scale } else { x/scale };
            let source=((y/scale)*im.width+column)*3;
            let target=(y*width+x+padding)*3;
            rgb[target..target+3].copy_from_slice(&im.data[source..source+3]);
        }}
        let (start,end)=if reflect {(im.width-1258,im.width-1237)} else {(1237,1258)};
        let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
        proptest::prop_assert!(!chunks.iter().any(|c| c.space_before && (start*scale+padding..=end*scale+padding).contains(&c.x0)));
        proptest::prop_assert_eq!(chunks.first().unwrap().x0,0);
        proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
        proptest::prop_assert!(chunks.windows(2).all(|c| c[0].x1==c[1].x0));
    }
}
