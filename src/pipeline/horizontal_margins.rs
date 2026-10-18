use super::crop::CroppedImage;
use anyhow::Result;
/// 純白列だけを置換した画像と、保持区間の出典。
#[derive(Debug)]
pub struct HorizontalMargins {
    /// 正規化したRGB画像。
    pub image: CroppedImage,
    /// 入力上の保持区間の開始列。
    pub source_x: usize,
    /// 保持区間の列数。
    pub content_width: usize,
    /// 出力上の保持区間の開始列。
    pub output_x: usize,
}
/// 純白の外側列だけを左右margin列へ置き換える。白紙は恒等。
pub fn normalize_horizontal_margins(
    rgb: &[u8],
    width: usize,
    height: usize,
    margin: usize,
) -> Result<HorizontalMargins> {
    prepare_margins(rgb, width, height, margin, false)
}
/// 縮小限定と無条件正規化で共通の入力検証・出典計算・RGB転送を行う。
fn prepare_margins(
    rgb: &[u8],
    width: usize,
    height: usize,
    margin: usize,
    only_contract: bool,
) -> Result<HorizontalMargins> {
    anyhow::ensure!(width > 0 && height > 0, "empty image");
    let expected = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("image size overflow"))?;
    anyhow::ensure!(rgb.len() == expected, "invalid RGB buffer length");
    anyhow::ensure!(margin <= width, "margin exceeds source width");
    let mut left = width;
    let mut right = 0;
    for (i, pixel) in rgb.chunks_exact(3).enumerate() {
        if pixel != [255; 3] {
            left = left.min(i % width);
            right = right.max(i % width + 1);
        }
    }
    if left >= right {
        left = 0;
        right = width;
    }
    let blank = left == 0 && right == width && rgb.iter().all(|&v| v == 255);
    let output_x = if blank { 0 } else { margin };
    let content_width = right - left;
    let output_width = output_x
        .checked_mul(2)
        .and_then(|m| m.checked_add(content_width))
        .ok_or_else(|| anyhow::anyhow!("output width overflow"))?;
    let (left, content_width, output_x, output_width) = if only_contract && output_width >= width {
        (0, width, 0, width)
    } else {
        (left, content_width, output_x, output_width)
    };
    let len = output_width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("output size overflow"))?;
    let mut data = Vec::new();
    data.try_reserve_exact(len)?;
    data.resize(len, 255);
    for y in 0..height {
        let from = (y * width + left) * 3;
        let to = (y * output_width + output_x) * 3;
        data[to..to + content_width * 3].copy_from_slice(&rgb[from..from + content_width * 3]);
    }
    Ok(HorizontalMargins {
        image: CroppedImage {
            data,
            width: output_width,
            height,
        },
        source_x: left,
        content_width,
        output_x,
    })
}

/// 正規化後に幅が厳密に縮む場合だけ白余白を置換し、それ以外は入力を維持する。
pub fn compact_horizontal_margins(
    rgb: &[u8],
    width: usize,
    height: usize,
    margin: usize,
) -> Result<HorizontalMargins> {
    prepare_margins(rgb, width, height, margin, true)
}
