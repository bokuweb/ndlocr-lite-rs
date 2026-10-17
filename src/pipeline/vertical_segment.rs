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

/// RGB8の縦書きページから、段は上→下・各段の列は右→左の半開矩形を検出する。
///
/// 縦書きが既知のページで明示的に使用する。紙面の陰影補正は行わず、
/// 全幅の広い白帯の前後に十分な高さと複数列がある場合に段を分ける。
/// 単列の段・汚れや罫線で白帯が途切れる紙面は段分けの対象外とする。
/// 返した矩形は元画像から切り出し、縦列回転と縦列用分割を経て認識できる。
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
    let columns = detect_block(rgb, width, height, 0, height, threshold)?;
    // 単列だけでは字間・段落空白と段間の区別を支持できない。
    if columns.len() < 2 {
        return Ok(columns);
    }
    let mut widths: Vec<_> = columns
        .iter()
        .map(|b| (b[2] - b[0]).saturating_sub(2).max(1))
        .collect();
    widths.sort_unstable();
    let glyph_width = widths[widths.len() / 2];
    let mut cuts = vec![0];
    let mut blank_start = None;
    let mut first_ink = None;
    let mut last_ink = 0;
    let mut gaps = Vec::new();
    for y in 0..height {
        let has_ink = rgb[y * width * 3..(y + 1) * width * 3]
            .chunks_exact(3)
            .any(|pixel| {
                (pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3 < threshold as u16
            });
        if has_ink {
            first_ink.get_or_insert(y);
            if let Some(start) = blank_start.take()
                && y - start >= glyph_width.saturating_mul(2)
            {
                gaps.push((start, y));
            }
            last_ink = y + 1;
        } else {
            blank_start.get_or_insert(y);
        }
    }
    let Some(first_ink) = first_ink else {
        return Ok(columns);
    };
    // 余白や短い孤立記号を段にしない。前後に本文の縦方向の長さを要求する。
    let min_extent = glyph_width.saturating_mul(4);
    let mut block_start = first_ink;
    for (start, end) in gaps {
        if start.saturating_sub(block_start) >= min_extent
            && last_ink.saturating_sub(end) >= min_extent
        {
            cuts.push(start + (end - start) / 2);
            block_start = end;
        }
    }
    if cuts.len() == 1 {
        return Ok(columns);
    }
    cuts.push(height);
    let mut blocks = Vec::new();
    for span in cuts.windows(2) {
        let block = detect_block(rgb, width, height, span[0], span[1], threshold)?;
        // 両段で複数の列が検出される場合だけ、全体の列を置き換える。
        if block.len() < 2 {
            return Ok(columns);
        }
        blocks.extend(block);
    }
    Ok(blocks)
}

/// 元画像内の横帯を転置して列検出し、元画像の座標へ戻す。
fn detect_block(
    rgb: &[u8],
    width: usize,
    height: usize,
    y0: usize,
    y1: usize,
    threshold: u8,
) -> Result<Vec<[usize; 4]>, String> {
    debug_assert!(y0 < y1 && y1 <= height);
    let block_height = y1 - y0;
    let length = width * block_height * 3;
    let mut transposed = Vec::new();
    transposed
        .try_reserve_exact(length)
        .map_err(|error| format!("cannot allocate vertical page: {error}"))?;
    transposed.resize(length, 0);
    // (x, y) -> (y, x)。列末の右側の句読点は、変換後の行末の下側になる。
    for y in y0..y1 {
        for x in 0..width {
            let source = (y * width + x) * 3;
            let target = (x * block_height + y - y0) * 3;
            transposed[target..target + 3].copy_from_slice(&rgb[source..source + 3]);
        }
    }
    Ok(super::line_segment::detect_textline_bands_naive(
        &transposed,
        block_height,
        width,
        threshold,
    )
    .into_iter()
    .rev()
    .map(|[x0, x_start, x1, x_end]| [x_start, x0 + y0, x_end, x1 + y0])
    .collect())
}
