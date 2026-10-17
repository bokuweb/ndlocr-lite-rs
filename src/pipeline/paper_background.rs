use std::borrow::Cow;

/// 明るい無彩色の紙面だけを対象に、緩い影を除いた同寸法RGBを返す。
///
/// 白い紙・色パネル・急な濃淡境界・密な画像は借用のまま維持する。
/// 検出と認識には返された同じ画素を使用すること。
pub fn normalize_paper_shading(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Result<Cow<'_, [u8]>, String> {
    let expected = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || expected != Some(rgb.len()) {
        return Err("invalid RGB page dimensions".into());
    }
    let step_x = (width / 128).max(2);
    let step_y = (height / 16).max(8);
    let columns = width.div_ceil(step_x);
    let rows = height.div_ceil(step_y);
    let mut backgrounds = vec![None; columns * rows];
    for row in 0..rows {
        for column in 0..columns {
            backgrounds[row * columns + column] = sample_background(
                rgb,
                width,
                [
                    column * step_x,
                    row * step_y,
                    ((column + 1) * step_x).min(width),
                    ((row + 1) * step_y).min(height),
                ],
            );
        }
    }
    let supported = backgrounds.iter().flatten().count();
    let shaded = backgrounds
        .iter()
        .flatten()
        .filter(|value| gray(*value) < 232)
        .count();
    if supported * 4 < backgrounds.len() * 3 || shaded * 16 < backgrounds.len() {
        return Ok(Cow::Borrowed(rgb));
    }
    fill_narrow_gaps(&mut backgrounds, columns);
    // 急なパネル境界を、緩い紙面の照明変化として扱わない。
    for row in 0..rows {
        for column in 0..columns {
            let Some(value) = backgrounds[row * columns + column] else {
                continue;
            };
            for neighbor in [
                (column + 1 < columns).then_some(row * columns + column + 1),
                (row + 1 < rows).then_some((row + 1) * columns + column),
            ]
            .into_iter()
            .flatten()
            {
                if backgrounds[neighbor]
                    .is_some_and(|other| value.iter().zip(other).any(|(&a, b)| a.abs_diff(b) > 24))
                {
                    return Ok(Cow::Borrowed(rgb));
                }
            }
        }
    }
    let mut output = rgb.to_vec();
    for y in 0..height {
        for x in 0..width {
            let Some(background) = backgrounds[(y / step_y) * columns + x / step_x] else {
                continue;
            };
            let i = (y * width + x) * 3;
            let contrast = (0..3)
                .map(|channel| background[channel].abs_diff(rgb[i + channel]))
                .max()
                .unwrap();
            output[i..i + 3].fill(255u8.saturating_sub(contrast.saturating_mul(2)));
        }
    }
    Ok(Cow::Owned(output))
}

/// 細い列と局所行帯の明るい多数派を背景候補とし、密な画像を除く。
fn sample_background(rgb: &[u8], width: usize, bounds: [usize; 4]) -> Option<[u8; 3]> {
    let [x0, y0, x1, y1] = bounds;
    let area = (x1 - x0) * (y1 - y0);
    let mut histogram = [0usize; 256];
    let mut candidates = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width + x) * 3;
            let pixel = &rgb[i..i + 3];
            let value = gray(pixel);
            if value >= 160 && pixel.iter().max().unwrap() - pixel.iter().min().unwrap() <= 40 {
                histogram[usize::from(value)] += 1;
                candidates += 1;
            }
        }
    }
    if candidates * 4 < area * 3 {
        return None;
    }
    let mut cumulative = 0;
    let background = histogram.iter().position(|&count| {
        cumulative += count;
        cumulative * 10 > candidates * 9
    })?;
    let supported: usize = histogram[background.saturating_sub(18)..=background]
        .iter()
        .sum();
    if supported * 4 < area * 3 {
        return None;
    }
    // 同明度の色文字に汚染されない多数派の色を、背景明度に対応させる。
    let mut colors = [[0usize; 256]; 3];
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width + x) * 3;
            let pixel = &rgb[i..i + 3];
            let value = usize::from(gray(pixel));
            if (background.saturating_sub(18)..=background).contains(&value)
                && pixel.iter().max().unwrap() - pixel.iter().min().unwrap() <= 40
            {
                for channel in 0..3 {
                    colors[channel][usize::from(pixel[channel])] += 1;
                }
            }
        }
    }
    let median: [u8; 3] = std::array::from_fn(|channel| {
        let mut cumulative = 0;
        colors[channel]
            .iter()
            .position(|&count| {
                cumulative += count;
                cumulative > supported / 2
            })
            .unwrap() as u8
    });
    let correction = background as i16 - i16::from(gray(&median));
    Some(median.map(|channel| (i16::from(channel) + correction).clamp(0, 255) as u8))
}

/// 行検出と同じRGB平均明度を使う。
fn gray(pixel: &[u8]) -> u8 {
    ((u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])) / 3) as u8
}

/// 細い縦画線が標本を占めた最大2列の欠落だけを、左右の背景一致で補う。
fn fill_narrow_gaps(backgrounds: &mut [Option<[u8; 3]>], columns: usize) {
    for row in backgrounds.chunks_exact_mut(columns) {
        let mut x = 0;
        while x < columns {
            if row[x].is_some() {
                x += 1;
                continue;
            }
            let start = x;
            while x < columns && row[x].is_none() {
                x += 1;
            }
            if start == 0 || x == columns || x - start > 2 {
                continue;
            }
            let (left, right) = (row[start - 1].unwrap(), row[x].unwrap());
            if left.iter().zip(right).all(|(&a, b)| a.abs_diff(b) <= 18) {
                for value in &mut row[start..x] {
                    *value = Some(std::array::from_fn(|channel| {
                        ((u16::from(left[channel]) + u16::from(right[channel])) / 2) as u8
                    }));
                }
            }
        }
    }
}
