use anyhow::{bail, Context, Result};
use image::DynamicImage;
use std::path::Path;

use super::ProcessedOutput;
use crate::OutputFormat;

/// Export processed output to the specified format and path.
/// Returns the output dimensions (width, height).
pub fn export(
    output: &ProcessedOutput,
    path: &Path,
    format: OutputFormat,
    jpeg_quality: u8,
) -> Result<(u32, u32)> {
    match (output, format) {
        (ProcessedOutput::Raster(img), OutputFormat::Svg) => {
            // Raster output requested as SVG — this is unusual but we can vectorize on-the-fly
            tracing::warn!("raster output requested as SVG, auto-vectorizing");
            let svg = super::vectorize::vectorize(img)?;
            std::fs::write(path, &svg).context("failed to write SVG")?;
            Ok((img.width(), img.height()))
        }
        (ProcessedOutput::Raster(img), fmt) => {
            export_raster(img, path, fmt, jpeg_quality)
        }
        (ProcessedOutput::Vector(svg), OutputFormat::Svg) => {
            std::fs::write(path, svg).context("failed to write SVG")?;
            // SVG dimensions are not pixel-based, report 0x0
            Ok((0, 0))
        }
        (ProcessedOutput::Vector(_svg), _fmt) => {
            bail!(
                "cannot export vector output as {}; use --format svg for vectorized results",
                format
            );
        }
    }
}

fn export_raster(
    img: &DynamicImage,
    path: &Path,
    format: OutputFormat,
    jpeg_quality: u8,
) -> Result<(u32, u32)> {
    let dims = (img.width(), img.height());

    match format {
        OutputFormat::Png => {
            img.save_with_format(path, image::ImageFormat::Png)
                .context("failed to save PNG")?;
        }
        OutputFormat::Jpeg => {
            let rgb = img.to_rgb8();
            let mut file = std::fs::File::create(path).context("failed to create JPEG file")?;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut file,
                jpeg_quality,
            );
            image::ImageEncoder::write_image(
                encoder,
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
            .context("failed to encode JPEG")?;
        }
        OutputFormat::Webp => {
            // image crate supports WebP via the webp feature — save directly
            img.save_with_format(path, image::ImageFormat::WebP)
                .context("failed to save WebP")?;
        }
        OutputFormat::Svg => unreachable!("handled above"),
    }

    tracing::info!(
        path = %path.display(),
        width = dims.0,
        height = dims.1,
        format = %format,
        "exported image"
    );

    Ok(dims)
}
