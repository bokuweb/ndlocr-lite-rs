use super::*;

#[test]
#[ignore = "requires pinned PARSeq models"]
fn latin_pair_keeps_the_owned_round_closer_in_session_and_horizontal_pool() {
    let image = crate::io::load_rgb_u8(Path::new(
        "tests/fixtures/latin_pair_bridge/klm_round_suffix_mincho24.png",
    ))
    .unwrap();
    let before = image.data.clone();
    let charset = Path::new("models/NDLmoji.yaml");
    let mut original_reads = Vec::new();
    let mut session_reads = Vec::new();
    let mut horizontal_reads = Vec::new();
    for file in [
        "parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx",
        "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    ] {
        let model = Path::new("models").join(file);
        let session = ParseqSession::load(&model, charset).unwrap();
        let pool = ParseqPool::load(&model, charset, 1).unwrap();
        let raw = session
            .recognize_rgb_u8_unprocessed(&image.data, image.width, image.height)
            .unwrap();
        let actual = session
            .recognize_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        let horizontal = pool
            .recognize_horizontal_rgb_u8(&image.data, image.width, image.height)
            .unwrap();
        assert_eq!(raw.mean_confidence.to_bits(), actual.mean_confidence.to_bits());
        assert_eq!(
            raw.mean_confidence.to_bits(),
            horizontal.mean_confidence.to_bits()
        );
        original_reads.push(raw.text);
        session_reads.push(actual.text);
        horizontal_reads.push(horizontal.text);
    }
    assert_eq!(image.data, before);
    assert_eq!(original_reads, ["KLM)"; 3]);
    assert_eq!(session_reads, ["KLM)"; 3]);
    assert_eq!(horizontal_reads, ["KLM)"; 3]);
}
