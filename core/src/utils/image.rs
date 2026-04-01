use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

/// Load an image from disk, supporting common formats.
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    let img = image::open(path)
        .with_context(|| format!("failed to open image: {}", path.display()))?;
    Ok(img)
}

/// Get a human-readable description of image dimensions.
pub fn describe_dimensions(img: &DynamicImage) -> String {
    let (w, h) = (img.width(), img.height());
    let megapixels = (w as f64 * h as f64) / 1_000_000.0;
    format!("{w}x{h} ({megapixels:.1} MP)")
}

/// Estimate memory usage for an RGBA image in bytes.
pub fn estimate_memory(width: u32, height: u32) -> u64 {
    width as u64 * height as u64 * 4
}
