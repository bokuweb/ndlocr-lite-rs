use super::*;
#[test]
#[ignore = "requires pinned PARSeq assets"]
fn dense_pool_horizontal_preserves_auto_and_matches_session() {
    let model = Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx");
    let charset = Path::new("models/NDLmoji.yaml");
    let pool = ParseqPool::load(model, charset, 2).unwrap();
    let session = ParseqSession::load(model, charset).unwrap();
    assert_eq!(pool.input_dimensions(), session.input_dimensions());
    let image =
        crate::io::load_rgb_u8(Path::new("tests/fixtures/horizontal_digit/digit.png")).unwrap();
    let auto = session
        .recognize_rgb_u8(&image.data, image.width, image.height)
        .unwrap();
    let mut horizontal = session
        .recognize_rgb_u8_unprocessed_with_orientation(
            &image.data,
            image.width,
            image.height,
            ParseqOrientation::Horizontal,
        )
        .unwrap();
    horizontal.text = sanitize_recognized_text(&horizontal.text);
    assert_eq!(auto.text, "11");
    assert_eq!(horizontal.text, "1");
    for _ in 0..3 {
        let actual = pool
            .recognize_horizontal_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(actual.text, horizontal.text);
        assert!((actual.mean_confidence - horizontal.mean_confidence).abs() < 1e-5);
        let actual = pool
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(actual.text, auto.text);
        assert!((actual.mean_confidence - auto.mean_confidence).abs() < 1e-5);
    }
    assert!(pool.recognize_horizontal_rgb_u8(&[], 0, 0).is_err());
    assert_eq!(
        pool.recognize_horizontal_rgb_u8(&image.data, image.width, image.height)
            .unwrap()
            .text,
        "1"
    );
    let batch = pool
        .recognize_batch_single_session_rgb_u8(&[(&image.data, image.width, image.height)])
        .unwrap();
    assert_eq!(batch[0].text, auto.text);
    assert!((batch[0].mean_confidence - auto.mean_confidence).abs() < 1e-5);
}

#[test]
#[ignore = "requires pinned PARSeq assets"]
fn band_pool_batch_horizontal_keeps_auto_and_error_reuse() {
    let model = Path::new("models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx");
    let charset = Path::new("models/NDLmoji.yaml");
    let pool = ParseqPool::load(model, charset, 2).unwrap();
    let image =
        crate::io::load_rgb_u8(Path::new("tests/fixtures/horizontal_digit/digit.png")).unwrap();
    let items = [(image.data.as_slice(), image.width, image.height); 2];
    let horizontal = pool
        .recognize_batch_single_session_horizontal_rgb_u8(&items)
        .unwrap();
    assert_eq!(horizontal.len(), 2);
    assert!(horizontal.iter().all(|r| r.text == "1"));
    let direct = pool
        .recognize_horizontal_rgb_u8(&image.data, image.width, image.height)
        .unwrap();
    assert!(
        horizontal
            .iter()
            .all(|r| (r.mean_confidence - direct.mean_confidence).abs() < 1e-5)
    );
    let auto = pool.recognize_batch_single_session_rgb_u8(&items).unwrap();
    assert!(auto.iter().all(|r| r.text == "11"));
    assert!(
        pool.recognize_batch_single_session_horizontal_rgb_u8(&[])
            .unwrap()
            .is_empty()
    );
    assert!(
        pool.recognize_batch_single_session_horizontal_rgb_u8(&[(&[], 0, 0)])
            .is_err()
    );
    assert_eq!(
        pool.recognize_batch_single_session_horizontal_rgb_u8(&items)
            .unwrap()[0]
            .text,
        "1"
    );
}
