/// 既知の縦書き列を無補間で反時計回り90度回し、横書き分割・Horizontal推論へ渡す。
/// 上→下を左→右へ移し、寸法比による自動判定を行わない。
pub fn rotate_vertical_column_rgb_u8(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Result<super::crop::CroppedImage, String> {
    let length = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return Err("invalid RGB column dimensions".into());
    }
    let mut data = Vec::new();
    data.try_reserve_exact(rgb.len())
        .map_err(|error| format!("cannot allocate rotated column: {error}"))?;
    data.resize(rgb.len(), 0);
    for y in 0..height {
        for x in 0..width {
            let source = (y * width + x) * 3;
            let target = ((width - 1 - x) * height + y) * 3;
            data[target..target + 3].copy_from_slice(&rgb[source..source + 3]);
        }
    }
    Ok(super::crop::CroppedImage {
        data,
        width: height,
        height: width,
    })
}

/// RGB8の縦書きページから右→左順の半開矩形を検出する。
///
/// 縦書きが既知のページで明示的に使用する。紙面の陰影補正は行わず、
/// 返した矩形の元画素をPARSeqのAuto方向で認識する。
/// 空画像・長さ不一致・寸法の積のoverflowはエラーにする。
pub fn detect_vertical_textline_columns(
    rgb: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Result<Vec<[usize; 4]>, String> {
    let length = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || length != Some(rgb.len()) {
        return Err("invalid RGB page dimensions".into());
    }
    let mut transposed = Vec::new();
    transposed
        .try_reserve_exact(rgb.len())
        .map_err(|error| format!("cannot allocate vertical page: {error}"))?;
    transposed.resize(rgb.len(), 0);
    // (x, y) -> (y, x)。列末の右側の句読点は、変換後の行末の下側になる。
    for y in 0..height {
        for x in 0..width {
            let source = (y * width + x) * 3;
            let target = (x * height + y) * 3;
            transposed[target..target + 3].copy_from_slice(&rgb[source..source + 3]);
        }
    }
    Ok(
        super::line_segment::detect_textline_bands_naive(&transposed, height, width, threshold)
            .into_iter()
            .rev()
            .map(|[x0, y0, x1, y1]| [y0, x0, y1, x1])
            .collect(),
    )
}
