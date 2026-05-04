#![cfg(feature = "onnx")]

#[test]
#[ignore = "requires ONNX model/image/charset fixtures via env vars"]
fn page_recognizer_processes_multiple_pages_in_order() {
    use ndlocr_lite_rs::infer::page_pool::{PageRecognizer, PageRecognizerOptions};
    use ndlocr_lite_rs::io as nd_io;

    let Some(det_model) = std::env::var_os("NDLOCR_TEST_DEIM_MODEL") else {
        eprintln!(
            "skip: set NDLOCR_TEST_DEIM_MODEL NDLOCR_TEST_PARSEQ_MODEL NDLOCR_TEST_CHARSET NDLOCR_TEST_IMAGE to run"
        );
        return;
    };
    let Some(parseq_model) = std::env::var_os("NDLOCR_TEST_PARSEQ_MODEL") else {
        eprintln!("skip: NDLOCR_TEST_PARSEQ_MODEL not set");
        return;
    };
    let Some(charset) = std::env::var_os("NDLOCR_TEST_CHARSET") else {
        eprintln!("skip: NDLOCR_TEST_CHARSET not set");
        return;
    };
    let Some(image) = std::env::var_os("NDLOCR_TEST_IMAGE") else {
        eprintln!("skip: NDLOCR_TEST_IMAGE not set");
        return;
    };

    let det_model = std::path::PathBuf::from(det_model);
    let parseq_model = std::path::PathBuf::from(parseq_model);
    let charset = std::path::PathBuf::from(charset);
    let image = std::path::PathBuf::from(image);

    let recognizer =
        PageRecognizer::load(&det_model, &parseq_model, &charset, 2).expect("page recognizer load");
    assert!(recognizer.parallelism() >= 1);

    let img = nd_io::load_rgb_u8(&image).expect("image load");
    let opts = PageRecognizerOptions::default();

    // 同じ画像を 4 ページぶん投げて並列処理。
    let items: Vec<(&[u8], usize, usize)> = (0..4)
        .map(|_| (img.data.as_slice(), img.width, img.height))
        .collect();
    let pages = recognizer
        .recognize_pages_rgb_u8(&items, opts)
        .expect("recognize pages");
    assert_eq!(pages.len(), 4);

    // page_index は入力順にきっちり 0..N-1 で並ぶ。
    for (i, page) in pages.iter().enumerate() {
        assert_eq!(page.page_index, i, "result must be ordered by input index");
    }

    // 同じ画像なので各ページの行数も一致する (atomic 配分のレースで
    // どこかが空になっていないことの確認)。
    let baseline = pages[0].lines.len();
    assert!(baseline > 0, "must detect at least one line");
    for page in &pages[1..] {
        assert_eq!(
            page.lines.len(),
            baseline,
            "all pages share the same image; line count must match"
        );
    }

    // 単一ページ API も同じ結果を返す。
    let single = recognizer
        .recognize_page_rgb_u8(&img.data, img.width, img.height, opts)
        .expect("recognize single");
    assert_eq!(single.lines.len(), baseline);
}
