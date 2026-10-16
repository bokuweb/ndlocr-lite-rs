use ndlocr_lite_rs::pipeline::line_chunks::split_line;

/// 背景だけの短い行も画素を欠落させず一つの区間に保つ。
#[test]
fn public_chunking_covers_blank_line() {
    let chunks = split_line(&[255; 30 * 10 * 3], 30, 10, 220).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!((chunks[0].x0, chunks[0].x1), (0, 30));
}
