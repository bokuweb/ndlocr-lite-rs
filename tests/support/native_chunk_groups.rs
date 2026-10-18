//! 実入力比率の文脈保持を比較する診断専用helper。

use ndlocr_lite_rs::pipeline::line_chunks::LineChunk;

/// 区間の被覆を変えず、モデル比率に収まる連続区間だけ結合する。
pub fn group_native_chunks(
    chunks: &[LineChunk],
    height: usize,
    input_width: usize,
    input_height: usize,
) -> Result<Vec<LineChunk>, String> {
    if height == 0 || input_width == 0 || input_height == 0 {
        return Err("invalid model/line dimensions".into());
    }
    if chunks.iter().any(|c| c.x0 >= c.x1) || chunks.windows(2).any(|pair| pair[0].x1 > pair[1].x0)
    {
        return Err("invalid or overlapping chunk bounds".into());
    }
    let mut grouped: Vec<LineChunk> = Vec::new();
    for chunk in chunks {
        if let Some(previous) = grouped.last_mut()
            && previous.x1 == chunk.x0
            && (chunk.x1 - previous.x0) as u128 * input_height as u128
                <= height as u128 * input_width as u128
        {
            previous.x1 = chunk.x1;
        } else {
            grouped.push(LineChunk {
                x0: chunk.x0,
                x1: chunk.x1,
                space_before: chunk.space_before,
                gap_before: chunk.gap_before,
            });
        }
    }
    Ok(grouped)
}
