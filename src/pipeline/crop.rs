use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BBox {
    pub xmin: usize,
    pub ymin: usize,
    pub xmax: usize,
    pub ymax: usize,
}
impl BBox {
    pub fn new(xmin: usize, ymin: usize, xmax: usize, ymax: usize) -> Self {
        Self {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CroppedImage {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

/// 行 bbox を四方向に `pad` ピクセル広げ、画像範囲に収める（検出が本文ぎりぎりのときの欠けを減らす）。
pub fn expand_bbox_xyxy_clamped(
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    pad: usize,
    img_w: usize,
    img_h: usize,
) -> (usize, usize, usize, usize) {
    if pad == 0 {
        return (x0, y0, x1, y1);
    }
    let px0 = x0.saturating_sub(pad);
    let py0 = y0.saturating_sub(pad);
    let px1 = (x1 + pad).min(img_w);
    let py1 = (y1 + pad).min(img_h);
    if px0 >= px1 || py0 >= py1 {
        (x0, y0, x1, y1)
    } else {
        (px0, py0, px1, py1)
    }
}

pub fn crop_rgb_u8(rgb: &[u8], width: usize, height: usize, bbox: BBox) -> Result<CroppedImage> {
    let expected = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(3))
        .ok_or_else(|| anyhow::anyhow!("image size overflow"))?;
    if rgb.len() != expected {
        bail!("invalid RGB buffer length");
    }
    if bbox.xmin >= bbox.xmax || bbox.ymin >= bbox.ymax || bbox.xmax > width || bbox.ymax > height {
        bail!("invalid bbox");
    }
    let ow = bbox.xmax - bbox.xmin;
    let oh = bbox.ymax - bbox.ymin;
    let mut out = vec![0_u8; ow * oh * 3];
    for y in 0..oh {
        for x in 0..ow {
            let sx = bbox.xmin + x;
            let sy = bbox.ymin + y;
            let s = (sy * width + sx) * 3;
            let d = (y * ow + x) * 3;
            out[d..d + 3].copy_from_slice(&rgb[s..s + 3]);
        }
    }
    Ok(CroppedImage {
        data: out,
        width: ow,
        height: oh,
    })
}
