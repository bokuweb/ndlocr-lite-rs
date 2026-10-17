use anyhow::{Context, Result, bail};

/// 元RGBと出力の寸法・要素数・Vecのbyte上限を、確保より先に検証する。
pub(super) fn checked_tensor_len(
    rgb: &[u8],
    width: usize,
    height: usize,
    input_width: usize,
    input_height: usize,
) -> Result<usize> {
    if width == 0 || height == 0 {
        bail!("invalid source image dimension");
    }
    let source_len = width
        .checked_mul(height)
        .and_then(|len| len.checked_mul(3))
        .context("image size overflow")?;
    if rgb.len() != source_len {
        bail!("invalid RGB buffer length");
    }
    if input_width == 0 || input_height == 0 {
        bail!("invalid resize dimension");
    }
    let len = input_width
        .checked_mul(input_height)
        .and_then(|len| len.checked_mul(3))
        .context("output size overflow")?;
    let bytes = len
        .checked_mul(size_of::<f32>())
        .context("output byte size overflow")?;
    if bytes > isize::MAX as usize {
        bail!("output exceeds tensor capacity");
    }
    Ok(len)
}

/// 検証済み要素数のテンソルを、確保エラーを伝播させて初期化する。
pub(super) fn zeroed_tensor(len: usize) -> Result<Vec<f32>> {
    let mut tensor = Vec::new();
    tensor
        .try_reserve_exact(len)
        .context("cannot allocate preprocessing tensor")?;
    tensor.resize(len, 0.0);
    Ok(tensor)
}
