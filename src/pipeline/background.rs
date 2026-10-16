use std::borrow::Cow;

/// 色背景とみなすRGB成分の最小差。無彩色の本文には作用させない。
const MIN_CHROMA: u8 = 24;

/// 広い色背景の白文字・黒文字を、同じ座標の暗い字形として準備する。
///
/// 横長の色パネルだけを対象にし、普通の無彩色ページは借用のまま返す。
/// 背景は列ごとの多数派から推定するため、写真や任意の図版には適用しない。
/// 検出・分割・認識には返された同じ画素列を使用すること。
pub fn normalize_chromatic_panels(
    rgb: &[u8],
    width: usize,
    height: usize,
) -> Result<Cow<'_, [u8]>, String> {
    let expected = width.checked_mul(height).and_then(|n| n.checked_mul(3));
    if width == 0 || height == 0 || expected != Some(rgb.len()) {
        return Err("invalid RGB page dimensions".into());
    }
    let min_coverage = (width / 8).max(32);
    let min_solid_run = (width / 4).max(64);
    let mut output = Cow::Borrowed(rgb);
    let mut start = None;
    let mut seeded = false;
    let mut background = None;
    for y in 0..=height {
        let mut count = 0;
        let mut longest = 0;
        let mut run = 0;
        let mut histogram = [0usize; 256];
        if y < height {
            for pixel in rgb[y * width * 3..(y + 1) * width * 3].chunks_exact(3) {
                if chromatic(pixel) {
                    count += 1;
                    histogram[usize::from(brightness(pixel))] += 1;
                    run += 1;
                    longest = longest.max(run);
                } else {
                    run = 0;
                }
            }
        }
        if count >= min_coverage {
            let mut cumulative = 0;
            let median = histogram
                .iter()
                .position(|n| {
                    cumulative += n;
                    cumulative > count / 2
                })
                .expect("chromatic row has samples") as u8;
            // 字形がgradientの一部を隠すだけの中央値変化では分割しない。
            if longest >= min_solid_run
                && background.is_some_and(|previous| {
                    normalized_intensity(median, previous) < 220
                        || normalized_intensity(previous, median) < 220
                })
            {
                if let Some(y0) = start.take() {
                    if seeded && y - y0 >= 8 {
                        normalize_panel(rgb, width, y0, y, &mut output);
                    }
                }
                seeded = false;
                background = None;
            }
            start.get_or_insert(y);
            background.get_or_insert(median);
            seeded |= longest >= min_solid_run;
        } else if let Some(y0) = start.take() {
            if seeded && y - y0 >= 8 {
                normalize_panel(rgb, width, y0, y, &mut output);
            }
            seeded = false;
            background = None;
        }
    }
    Ok(output)
}

/// 無彩色の字形・白紙を除外し、色のある背景候補を選ぶ。
fn chromatic(pixel: &[u8]) -> bool {
    let max = pixel[0].max(pixel[1]).max(pixel[2]);
    let min = pixel[0].min(pixel[1]).min(pixel[2]);
    max - min >= MIN_CHROMA
}

/// 同じ重みのRGB平均を、背景との明暗差の比較にだけ使う。
fn brightness(pixel: &[u8]) -> u8 {
    ((u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])) / 3) as u8
}

/// 各列で多数派の色背景を推定し、明暗両極性の字形を白地へ写す。
fn normalize_panel(rgb: &[u8], width: usize, y0: usize, y1: usize, output: &mut Cow<'_, [u8]>) {
    // 短い末尾行の白紙を、上の行の背景内にある白文字と取り違えない。
    let row_bounds: Vec<_> = (y0..y1)
        .map(|y| {
            let row = &rgb[y * width * 3..(y + 1) * width * 3];
            row.chunks_exact(3)
                .position(chromatic)
                .zip(row.chunks_exact(3).rposition(chromatic))
        })
        .collect();
    let mut samples = Vec::with_capacity(y1 - y0);
    for x in 0..width {
        samples.clear();
        for y in y0..y1 {
            let i = (y * width + x) * 3;
            let pixel = &rgb[i..i + 3];
            if chromatic(pixel) {
                samples.push(brightness(pixel));
            }
        }
        if samples.is_empty() {
            continue;
        }
        let middle = samples.len() / 2;
        let (_, median, _) = samples.select_nth_unstable(middle);
        let background = *median;
        for y in y0..y1 {
            if !row_bounds[y - y0].is_some_and(|(x0, x1)| (x0..=x1).contains(&x)) {
                continue;
            }
            let i = (y * width + x) * 3;
            let value = brightness(&rgb[i..i + 3]);
            output.to_mut()[i..i + 3].fill(normalized_intensity(value, background));
        }
    }
}

/// 背景との明暗差を、白地に対する字形の暗さへ変換する。
fn normalized_intensity(value: u8, background: u8) -> u8 {
    let value = u16::from(value);
    let background = u16::from(background);
    let contrast = if value >= background {
        (value - background) * 255 / (255 - background).max(1)
    } else {
        (background - value) * 255 / background.max(1)
    };
    (255 - contrast) as u8
}
