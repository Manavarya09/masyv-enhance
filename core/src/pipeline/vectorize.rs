use anyhow::Result;
use image::DynamicImage;

/// Convert a raster image to SVG using vtracer.
///
/// Best results on clean, high-contrast images (logos, icons, line art).
/// Photos will produce large, noisy SVGs — the analyzer should route photos
/// away from this pipeline.
pub fn vectorize(img: &DynamicImage) -> Result<String> {
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width() as usize, rgba.height() as usize);

    tracing::info!(width, height, "vectorizing image");

    // vtracer::convert_image_to_svg works with file paths, so we use the
    // lower-level vtracer::convert which works with in-memory ColorImage.
    let config = build_config();

    let color_image = vtracer::ColorImage {
        pixels: rgba.into_raw(),
        width,
        height,
    };

    let svg_file = vtracer::convert(color_image, config)
        .map_err(|e| anyhow::anyhow!("vtracer conversion failed: {e}"))?;

    let svg = svg_file.to_string();
    let svg_bytes = svg.len();
    tracing::info!(svg_bytes, "vectorization complete");

    Ok(svg)
}

fn build_config() -> vtracer::Config {
    let mut config = vtracer::Config::default();

    // Tuned for clean output on logos/icons:
    config.color_precision = 8;
    config.filter_speckle = 4; // Remove tiny noise artifacts
    config.corner_threshold = 60; // Sharper corners for geometric shapes
    config.length_threshold = 4.0; // Minimum segment length
    config.splice_threshold = 45; // Angle threshold for path joining
    config.color_mode = vtracer::ColorMode::Color; // Full color, not binary

    config
}
