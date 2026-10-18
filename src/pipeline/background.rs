use std::borrow::Cow;

mod antialias;
mod exterior;
mod lateral;

/// 色背景とみなすRGB成分の最小差。無彩色の本文には作用させない。
const MIN_CHROMA: u8 = 24;

/// 広い色背景の白文字・黒文字を、同じ座標の暗い字形として準備する。
///
/// 横長の色パネルだけを対象にし、普通の無彩色ページは借用のまま返す。
/// 背景は列ごとの多数派から推定するため、写真や任意の図版には適用しない。
/// ページ外周へ4近傍で接続する近白（RGB各成分250以上）は元RGBのまま保持する。
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
    let mut exterior = None;
    for y in 0..=height {
        let mut count = 0;
        let mut histogram = [0usize; 256];
        let mut has_background_run = false;
        if y < height {
            let row = &rgb[y * width * 3..(y + 1) * width * 3];
            for pixel in row.chunks_exact(3).filter(|pixel| chromatic(pixel)) {
                count += 1;
                histogram[usize::from(brightness(pixel))] += 1;
            }
            // 密な色文字しかない行では、背景が変わったと判断しない。
            has_background_run =
                count >= min_coverage && !background_sample_ranges(row, min_solid_run).is_empty();
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
            if has_background_run
                && background.is_some_and(|previous| {
                    normalized_intensity(median, previous) < 220
                        || normalized_intensity(previous, median) < 220
                })
            {
                if let Some(y0) = start.take() {
                    if seeded && y - y0 >= 8 {
                        normalize_panel(rgb, width, y0, y, &mut output, &mut exterior);
                    }
                }
                seeded = false;
                background = None;
            }
            start.get_or_insert(y);
            if has_background_run {
                background.get_or_insert(median);
                seeded = true;
            }
        } else if let Some(y0) = start.take() {
            if seeded && y - y0 >= 8 {
                normalize_panel(rgb, width, y0, y, &mut output, &mut exterior);
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

/// 横長の連続色領域だけを、背景推定の標本範囲として列挙する。
fn background_sample_ranges(row: &[u8], min_run: usize) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = None;
    let width = row.len() / 3;
    for x in 0..=width {
        let pixel = (x < width).then(|| &row[x * 3..x * 3 + 3]);
        let colored = pixel.is_some_and(chromatic);
        // 色文字も有彩色なので、字形の色境界で背景の連続標本を分ける。
        // 隣接画素の差だけを見ることで、緩やかな横方向gradientは保持する。
        let continuous = colored && start.is_some() && {
            let previous = &row[(x - 1) * 3..x * 3];
            let current = pixel.unwrap();
            current
                .iter()
                .zip(previous)
                .all(|(&a, &b)| a.abs_diff(b) < MIN_CHROMA)
        };
        if !continuous {
            if let Some(x0) = start.take()
                && x - x0 >= min_run
            {
                ranges.push(x0..x);
            }
            if colored {
                start = Some(x);
            }
        }
    }
    ranges
}

/// 各列のRGB背景を推定し、背景内の明暗・異色の字形を白地へ写す。
fn normalize_panel(
    rgb: &[u8],
    width: usize,
    y0: usize,
    y1: usize,
    output: &mut Cow<'_, [u8]>,
    exterior: &mut Option<Vec<bool>>,
) {
    let exterior = exterior.get_or_insert_with(|| exterior::white_paper(rgb, width));
    let seed_ranges: Vec<_> = (y0..y1)
        .map(|y| {
            background_sample_ranges(
                &rgb[y * width * 3..(y + 1) * width * 3],
                (width / 4).max(64),
            )
        })
        .collect();
    let mut backgrounds = vec![None; width];
    let mut samples = [
        Vec::with_capacity(y1 - y0),
        Vec::with_capacity(y1 - y0),
        Vec::with_capacity(y1 - y0),
    ];
    for (x, background) in backgrounds.iter_mut().enumerate() {
        for channel in &mut samples {
            channel.clear();
        }
        for (dy, ranges) in seed_ranges.iter().enumerate() {
            if !ranges.iter().any(|range| range.contains(&x)) {
                continue;
            }
            let i = ((y0 + dy) * width + x) * 3;
            for (channel, values) in samples.iter_mut().enumerate() {
                values.push(rgb[i + channel]);
            }
        }
        if samples[0].is_empty() {
            continue;
        }
        let middle = samples[0].len() / 2;
        *background = Some(std::array::from_fn(|channel| {
            *samples[channel].select_nth_unstable(middle).1
        }));
    }
    extend_solid_panel_edges(rgb, width, y0, y1, &mut backgrounds);
    for y in y0..y1 {
        // 同じ行でも色文字の外側白紙を背景内の白文字として反転しない。
        let is_background = |x: &usize| {
            let i = (y * width + *x) * 3;
            chromatic(&rgb[i..i + 3])
                && backgrounds[*x].is_some_and(|background| {
                    normalized_color_intensity(&rgb[i..i + 3], background) >= 220
                })
        };
        let Some(x0) = (0..width).find(is_background) else {
            continue;
        };
        let x1 = (x0..width).rfind(is_background).unwrap();
        for (x, background) in backgrounds.iter().enumerate().take(x1 + 1).skip(x0) {
            let Some(background) = background else {
                continue;
            };
            if exterior[y * width + x] {
                continue;
            }
            let i = (y * width + x) * 3;
            output.to_mut()[i..i + 3].fill(normalized_color_intensity(&rgb[i..i + 3], *background));
        }
    }
    antialias::remove_panel_fringe(rgb, width, y0, y1, &backgrounds, exterior, output);
    lateral::remove_fringe(rgb, width, y0, y1, &backgrounds, exterior, output);
}

/// 広い標本の外縁に接し、行帯の全高で色が不変な細い枠だけを背景へ延長する。
fn extend_solid_panel_edges(
    rgb: &[u8],
    width: usize,
    y0: usize,
    y1: usize,
    backgrounds: &mut [Option<[u8; 3]>],
) {
    let Some(left) = backgrounds.iter().position(Option::is_some) else {
        return;
    };
    let right = backgrounds.iter().rposition(Option::is_some).unwrap();
    for columns in [
        (0..left).rev().collect::<Vec<_>>(),
        (right + 1..width).collect(),
    ] {
        for x in columns {
            let i = (y0 * width + x) * 3;
            let color: [u8; 3] = rgb[i..i + 3].try_into().unwrap();
            if !chromatic(&color)
                || !(y0 + 1..y1).all(|y| {
                    let i = (y * width + x) * 3;
                    rgb[i..i + 3] == color
                })
            {
                break;
            }
            backgrounds[x] = Some(color);
        }
    }
}

/// 平均明度が同じ異色の字形も保持するため、最も強い成分差を用いる。
fn normalized_color_intensity(value: &[u8], background: [u8; 3]) -> u8 {
    (0..3)
        .map(|channel| normalized_intensity(value[channel], background[channel]))
        .min()
        .unwrap()
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
