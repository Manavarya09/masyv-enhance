use image::{DynamicImage, Pixel};
use serde::Serialize;
use std::collections::HashSet;

use crate::ImageType;

/// Full analysis result returned by `analyze_image`.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    pub detected_type: ImageType,
    pub dimensions: (u32, u32),
    pub unique_colors: usize,
    pub edge_density: f64,
    pub saturation_variance: f64,
    pub avg_saturation: f64,
    pub confidence: f64,
}

/// Analyze an image and return detailed stats + detected type without running the pipeline.
pub fn analyze_image(img: &DynamicImage) -> AnalysisResult {
    let (width, height) = (img.width(), img.height());
    let stats = compute_image_stats(img);
    let detected_type = classify(&stats);
    let confidence = compute_confidence(&stats, detected_type);

    AnalysisResult {
        detected_type,
        dimensions: (width, height),
        unique_colors: stats.unique_colors,
        edge_density: stats.edge_density,
        saturation_variance: stats.saturation_variance,
        avg_saturation: stats.avg_saturation,
        confidence,
    }
}

/// Analyze an image and detect its content type using heuristics.
///
/// Examines:
/// - unique color count (low = logo/text, high = photo)
/// - edge density via Sobel-like gradient magnitude
/// - color variance (saturation spread)
/// - aspect ratio hints
pub fn detect_image_type(img: &DynamicImage) -> ImageType {
    let stats = compute_image_stats(img);

    tracing::debug!(
        unique_colors = stats.unique_colors,
        edge_density = stats.edge_density,
        saturation_variance = stats.saturation_variance,
        avg_saturation = stats.avg_saturation,
        "image analysis stats"
    );

    classify(&stats)
}

fn classify(stats: &ImageStats) -> ImageType {
    // Decision tree:
    //
    // Very few unique colors + high edge density → Logo
    // Low saturation + high edge density → Text
    // High unique colors + moderate-high saturation → Photo
    // Everything else → Illustration

    if stats.unique_colors < 64 && stats.edge_density > 0.05 {
        ImageType::Logo
    } else if stats.avg_saturation < 0.1 && stats.edge_density > 0.08 {
        ImageType::Text
    } else if stats.unique_colors > 1000 && stats.saturation_variance > 0.01 {
        ImageType::Photo
    } else {
        ImageType::Illustration
    }
}

/// Rough confidence score based on how strongly the stats match the detected type.
fn compute_confidence(stats: &ImageStats, detected: ImageType) -> f64 {
    match detected {
        ImageType::Logo => {
            // Fewer colors + higher edge density → more confident
            let color_score = 1.0 - (stats.unique_colors as f64 / 64.0).min(1.0);
            let edge_score = ((stats.edge_density - 0.05) / 0.15).clamp(0.0, 1.0);
            0.5 + 0.5 * (color_score * 0.6 + edge_score * 0.4)
        }
        ImageType::Text => {
            let sat_score = 1.0 - (stats.avg_saturation / 0.1).min(1.0);
            let edge_score = ((stats.edge_density - 0.08) / 0.12).clamp(0.0, 1.0);
            0.5 + 0.5 * (sat_score * 0.5 + edge_score * 0.5)
        }
        ImageType::Photo => {
            let color_score = ((stats.unique_colors as f64 - 1000.0) / 5000.0).clamp(0.0, 1.0);
            let var_score = ((stats.saturation_variance - 0.01) / 0.05).clamp(0.0, 1.0);
            0.5 + 0.5 * (color_score * 0.5 + var_score * 0.5)
        }
        ImageType::Illustration => {
            // Fallback category — lower base confidence
            0.4
        }
    }
}

struct ImageStats {
    unique_colors: usize,
    edge_density: f64,
    saturation_variance: f64,
    avg_saturation: f64,
}

fn compute_image_stats(img: &DynamicImage) -> ImageStats {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Sample at most ~100k pixels for performance on large images
    let step = ((width * height) as f64 / 100_000.0).max(1.0).sqrt() as u32;
    let step = step.max(1);

    let mut color_set = HashSet::new();
    let mut saturations = Vec::new();

    for y in (0..height).step_by(step as usize) {
        for x in (0..width).step_by(step as usize) {
            let px = rgba.get_pixel(x, y).to_rgb();
            let [r, g, b] = px.0;

            // Quantize to 5-bit per channel for unique color counting
            let quantized = ((r >> 3) as u32) << 10 | ((g >> 3) as u32) << 5 | (b >> 3) as u32;
            color_set.insert(quantized);

            // Compute saturation (HSL-style)
            let rf = r as f64 / 255.0;
            let gf = g as f64 / 255.0;
            let bf = b as f64 / 255.0;
            let max = rf.max(gf).max(bf);
            let min = rf.min(gf).min(bf);
            let l = (max + min) / 2.0;
            let s = if (max - min).abs() < f64::EPSILON {
                0.0
            } else if l <= 0.5 {
                (max - min) / (max + min)
            } else {
                (max - min) / (2.0 - max - min)
            };
            saturations.push(s);
        }
    }

    let avg_saturation = if saturations.is_empty() {
        0.0
    } else {
        saturations.iter().sum::<f64>() / saturations.len() as f64
    };

    let saturation_variance = if saturations.len() < 2 {
        0.0
    } else {
        let n = saturations.len() as f64;
        saturations.iter().map(|s| (s - avg_saturation).powi(2)).sum::<f64>() / n
    };

    // Edge density via simple gradient magnitude on grayscale
    let gray = img.to_luma8();
    let edge_density = compute_edge_density(&gray, step);

    ImageStats {
        unique_colors: color_set.len(),
        edge_density,
        saturation_variance,
        avg_saturation,
    }
}

fn compute_edge_density(gray: &image::GrayImage, step: u32) -> f64 {
    let (width, height) = gray.dimensions();
    if width < 3 || height < 3 {
        return 0.0;
    }

    let mut edge_count = 0u64;
    let mut total = 0u64;
    let threshold = 30.0; // gradient magnitude threshold for "edge"

    for y in (1..height - 1).step_by(step as usize) {
        for x in (1..width - 1).step_by(step as usize) {
            let gx = gray.get_pixel(x + 1, y).0[0] as f64
                - gray.get_pixel(x - 1, y).0[0] as f64;
            let gy = gray.get_pixel(x, y + 1).0[0] as f64
                - gray.get_pixel(x, y - 1).0[0] as f64;
            let mag = (gx * gx + gy * gy).sqrt();

            if mag > threshold {
                edge_count += 1;
            }
            total += 1;
        }
    }

    if total == 0 {
        0.0
    } else {
        edge_count as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_color_is_logo_or_illustration() {
        // A solid-color image should have very few unique colors
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            100,
            100,
            image::Rgb([255, 0, 0]),
        ));
        let t = detect_image_type(&img);
        // Solid color, no edges → likely illustration (fallback)
        assert!(t == ImageType::Logo || t == ImageType::Illustration);
    }
}
