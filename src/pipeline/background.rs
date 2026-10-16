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
    for y in 0..=height {
        let mut count = 0;
        let mut longest = 0;
        let mut run = 0;
        if y < height {
            for pixel in rgb[y * width * 3..(y + 1) * width * 3].chunks_exact(3) {
                if chromatic(pixel) {
                    count += 1;
                    run += 1;
                    longest = longest.max(run);
                } else {
                    run = 0;
                }
            }
        }
        if count >= min_coverage {
            start.get_or_insert(y);
            seeded |= longest >= min_solid_run;
        } else if let Some(y0) = start.take() {
            if seeded && y - y0 >= 8 {
                normalize_panel(rgb, width, y0, y, &mut output);
            }
            seeded = false;
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
        if samples.len() <= (y1 - y0) / 2 {
            continue;
        }
        let middle = samples.len() / 2;
        let (_, median, _) = samples.select_nth_unstable(middle);
        let background = u16::from(*median);
        for y in y0..y1 {
            let i = (y * width + x) * 3;
            let value = u16::from(brightness(&rgb[i..i + 3]));
            let contrast = if value >= background {
                (value - background) * 255 / (255 - background).max(1)
            } else {
                (background - value) * 255 / background.max(1)
            };
            output.to_mut()[i..i + 3].fill((255 - contrast) as u8);
        }
    }
}
