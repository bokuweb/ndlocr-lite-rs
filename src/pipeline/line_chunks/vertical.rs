use super::{LineChunk, blank_gaps, split_line_at_preferred_gaps};

/// 回転済み縦列を白列で分割し、横書きの単語間隔による断片化・空白追加を避ける。
/// 最大20インク列群・幅/高さ比20・モデル入力比の2倍を開発上限とする。
/// 30文字PARSeq向けの経験的制約であり、全モデルの精度保証ではない。
pub fn split_rotated_vertical_column_for_input(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    input_width: usize,
    input_height: usize,
) -> Result<Vec<LineChunk>, String> {
    let length = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return Err("invalid RGB column dimensions".into());
    }
    if input_width == 0 || input_height == 0 || input_width.checked_mul(input_height).is_none() {
        return Err("invalid model input dimensions".into());
    }
    let model_width = height
        .checked_mul(input_width)
        .and_then(|n| n.checked_mul(2))
        .ok_or("column dimensions overflow")?
        / input_height;
    let max_width = height
        .checked_mul(20)
        .ok_or("column dimensions overflow")?
        .min(model_width);
    let mut gaps = blank_gaps(rgb, width, height, threshold);
    // 回転した縦書き字形の間隔は横書きの単語空白ではない。
    for gap in &mut gaps {
        gap.preferred_split = false;
        gap.word_space = false;
    }
    let mut chunks = split_line_at_preferred_gaps(&gaps, width, height, max_width)?;
    for chunk in &mut chunks {
        chunk.space_before = false;
    }
    Ok(chunks)
}
