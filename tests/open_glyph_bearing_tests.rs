use ndlocr_lite_rs::pipeline::line_chunks::split_line_for_input;

/// 低密度の全高字形を含む原寸行を読み込む。
fn fixture(name: &str) -> ndlocr_lite_rs::io::RgbImageData {
    ndlocr_lite_rs::io::load_rgb_u8(std::path::Path::new(&format!(
        "tests/fixtures/open_glyph_bearings/{name}.png"
    )))
    .unwrap()
}

/// 和文の字間余白へ単語空白を補わず、元の全画素を保持する。
#[test]
fn open_glyph_bearings_do_not_insert_word_spaces() {
    for (name, ranges) in [
        ("runtimes", vec![(1079, 1103)]),
        ("possible", vec![(332, 349), (382, 401)]),
        ("docs", vec![(591, 612), (782, 803)]),
    ] {
        let im = fixture(name);
        let chunks = split_line_for_input(&im.data, im.width, im.height, 220, 256, 24).unwrap();
        for (start, end) in ranges {
            assert!(
                !chunks
                    .iter()
                    .any(|c| c.space_before && (start..=end).contains(&c.x0)),
                "{name}: {chunks:?}"
            );
        }
        assert_eq!(chunks.first().unwrap().x0, 0);
        assert_eq!(chunks.last().unwrap().x1, im.width);
        assert!(chunks.windows(2).all(|c| c[0].x1 == c[1].x0));
    }
}

/// 固定モデルの本文へ不要な空白を追加せず、英字前後の実空白を維持する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn open_glyph_context_reaches_real_recognition() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::crop::{BBox, crop_rgb_u8},
    };
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    for (name, expected) in [
        (
            "runtimes",
            "(ア) Rust は非常に高速でメモリ効率が高くランタイムやガベージコ",
        ),
        ("possible", "ル時に排除することが可能です。"),
        (
            "docs",
            "(ア) Rust には優れたドキュメント、有用なエラーメッセージを備え",
        ),
    ] {
        let im = fixture(name);
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
            eprintln!("{name} {c:?}: {text}");
            if c.insert_space_before(&actual, &text, im.height) {
                actual.push(' ');
            }
            actual.push_str(&text);
        }
        assert_eq!(actual, expected, "{name}");
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(24))]
    /// 拡大・移動・反転でも疎な字形の余白属性と全画素被覆を維持する。
    #[test]
    fn open_bearings_survive_geometry_changes(scale in 1usize..4, padding in 0usize..10, reflect in proptest::bool::ANY) {
        for (name, ranges) in [
            ("runtimes", vec![(1079,1103)]),
            ("possible", vec![(332,349),(382,401)]),
            ("docs", vec![(591,612),(782,803)]),
        ] {
            let im=fixture(name);
            let (width,height)=(im.width*scale+padding*2,im.height*scale);
            let mut rgb=vec![255; width*height*3];
            for y in 0..height { for x in 0..im.width*scale {
                let column=if reflect {im.width-1-x/scale} else {x/scale};
                let source=((y/scale)*im.width+column)*3;
                let target=(y*width+x+padding)*3;
                rgb[target..target+3].copy_from_slice(&im.data[source..source+3]);
            }}
            let chunks=split_line_for_input(&rgb,width,height,220,256,24).unwrap();
            for (start,end) in ranges {
                let (start,end)=if reflect {(im.width-end,im.width-start)} else {(start,end)};
                proptest::prop_assert!(!chunks.iter().any(|c| c.space_before && (start*scale+padding..=end*scale+padding).contains(&c.x0)));
            }
            proptest::prop_assert_eq!(chunks.first().unwrap().x0,0);
            proptest::prop_assert_eq!(chunks.last().unwrap().x1,width);
            proptest::prop_assert!(chunks.windows(2).all(|c| c[0].x1==c[1].x0));
        }
    }
}

/// 英字の実空白とスキャン句読点を保持する。
#[cfg(all(feature = "onnx", unix))]
#[test]
#[ignore = "requires fixed PARSeq assets"]
fn open_bearings_preserve_latin_spaces_and_scan_punctuation() {
    use ndlocr_lite_rs::{
        infer::cached::ParseqSession,
        pipeline::{
            crop::{BBox, crop_rgb_u8},
            line_segment::detect_textline_bands_naive,
        },
    };
    let session = ParseqSession::load(
        std::path::Path::new(&std::env::var("NDLOCR_TEST_PARSEQ_MODEL").unwrap()),
        std::path::Path::new(&std::env::var("NDLOCR_TEST_CHARSET").unwrap()),
    )
    .unwrap();
    for (name, expected) in [
        ("scan", "し、その複製物は、原本と同等の保管、管理をする。"),
        ("latin", "OCR accuracy test 1234567890"),
    ] {
        let im = fixture(name);
        let bands = detect_textline_bands_naive(&im.data, im.width, im.height, 220);
        assert_eq!(bands.len(), 1);
        let [x0, y0, x1, y1] = bands[0];
        let crop = crop_rgb_u8(&im.data, im.width, im.height, BBox::new(x0, y0, x1, y1)).unwrap();
        let (width, height) = session.input_dimensions();
        let chunks =
            split_line_for_input(&crop.data, crop.width, crop.height, 220, width, height).unwrap();
        eprintln!("NEGATIVE {name} band {:?} chunks {chunks:?}", bands[0]);
        let mut actual = String::new();
        for c in chunks {
            let piece = crop_rgb_u8(
                &crop.data,
                crop.width,
                crop.height,
                BBox::new(c.x0, 0, c.x1, crop.height),
            )
            .unwrap();
            let text = session
                .recognize_rgb_u8_unprocessed(&piece.data, piece.width, piece.height)
                .unwrap()
                .text;
            if c.insert_space_before(&actual, &text, crop.height) {
                actual.push(' ');
            }
            actual.push_str(&text);
        }
        assert_eq!(actual, expected, "{name}");
    }
}
