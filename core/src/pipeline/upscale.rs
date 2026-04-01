use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

use crate::models::onnx::OnnxUpscaler;

/// Upscale an image using AI (Real-ESRGAN via ONNX) or fallback to Lanczos3.
///
/// The `model_dir` should contain `realesrgan_x4plus.onnx`.
/// If the model is not available, falls back to high-quality Lanczos3 interpolation.
pub fn upscale(img: &DynamicImage, scale: u32, model_dir: &Path) -> Result<DynamicImage> {
    let scale = match scale {
        2 | 4 | 8 => scale,
        _ => {
            tracing::warn!(scale, "unsupported scale factor, defaulting to 4");
            4
        }
    };

    // Try AI upscaling first
    let model_path = model_dir.join("realesrgan_x4plus.onnx");
    if model_path.exists() {
        tracing::info!("using Real-ESRGAN AI upscaler");
        return ai_upscale(img, scale, &model_path);
    }

    // Fallback to traditional upscaling
    tracing::warn!("ONNX model not found at {}, using Lanczos3 fallback", model_path.display());
    Ok(lanczos_upscale(img, scale))
}

/// AI-powered upscale using Real-ESRGAN ONNX model.
fn ai_upscale(img: &DynamicImage, scale: u32, model_path: &Path) -> Result<DynamicImage> {
    let upscaler = OnnxUpscaler::load(model_path)
        .context("failed to load Real-ESRGAN model")?;

    // Real-ESRGAN is natively 4x. For other scales, we compose.
    match scale {
        2 => {
            // Upscale 4x then downscale 2x
            let upscaled_4x = upscaler.infer(img)?;
            let (w, h) = (img.width() * 2, img.height() * 2);
            Ok(DynamicImage::ImageRgb8(image::imageops::resize(
                &upscaled_4x.to_rgb8(),
                w,
                h,
                image::imageops::FilterType::Lanczos3,
            )))
        }
        4 => upscaler.infer(img),
        8 => {
            // Two-pass 4x upscale
            let first = upscaler.infer(img)?;
            upscaler.infer(&first)
        }
        _ => upscaler.infer(img),
    }
}

/// High-quality Lanczos3 interpolation fallback.
fn lanczos_upscale(img: &DynamicImage, scale: u32) -> DynamicImage {
    let (w, h) = (img.width() * scale, img.height() * scale);
    tracing::info!(target_w = w, target_h = h, "Lanczos3 upscale");
    DynamicImage::ImageRgba8(image::imageops::resize(
        img,
        w,
        h,
        image::imageops::FilterType::Lanczos3,
    ))
}
