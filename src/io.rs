use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

pub struct RgbImageData {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

const SUPPORTED_IMAGE_EXTENSIONS: [&str; 7] = ["jpg", "jpeg", "png", "tiff", "tif", "jp2", "bmp"];

pub fn is_supported_image_extension(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    matches!(ext, Some(e) if SUPPORTED_IMAGE_EXTENSIONS.contains(&e.as_str()))
}

pub fn collect_input_images(
    sourcedir: Option<PathBuf>,
    sourceimg: Option<PathBuf>,
) -> Result<Vec<PathBuf>> {
    let mut input_paths = Vec::new();
    if let Some(dir) = sourcedir {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() && is_supported_image_extension(&path.to_string_lossy()) {
                input_paths.push(path);
            }
        }
    }
    if let Some(path) = sourceimg
        && path.is_file()
        && is_supported_image_extension(&path.to_string_lossy())
    {
        input_paths.push(path);
    }
    if input_paths.is_empty() {
        return Err(anyhow!(
            "No image found. Provide --sourceimg or --sourcedir."
        ));
    }
    Ok(input_paths)
}

pub fn load_rgb_u8(path: &Path) -> Result<RgbImageData> {
    let rgb = image::open(path)?.to_rgb8();
    Ok(RgbImageData {
        width: rgb.width() as usize,
        height: rgb.height() as usize,
        data: rgb.into_raw(),
    })
}
