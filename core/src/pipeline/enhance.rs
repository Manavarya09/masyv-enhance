use image::{DynamicImage, Rgb, RgbImage};

/// Full photo enhancement pipeline: denoise → contrast → sharpen.
pub fn enhance_photo(img: &DynamicImage) -> DynamicImage {
    let denoised = denoise(img);
    let contrasted = boost_contrast(&denoised);
    sharpen(&contrasted)
}

/// Apply a mild Gaussian-style blur for denoising.
/// Uses a 3x3 box blur approximation (fast, good enough for preprocessing).
pub fn denoise(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    if w < 3 || h < 3 {
        return img.clone();
    }

    let mut out = RgbImage::new(w, h);

    // 3x3 box blur (uniform kernel)
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;

            for dy in 0..3u32 {
                for dx in 0..3u32 {
                    let px = rgb.get_pixel(x + dx - 1, y + dy - 1);
                    r_sum += px.0[0] as u32;
                    g_sum += px.0[1] as u32;
                    b_sum += px.0[2] as u32;
                }
            }

            out.put_pixel(
                x,
                y,
                Rgb([
                    (r_sum / 9) as u8,
                    (g_sum / 9) as u8,
                    (b_sum / 9) as u8,
                ]),
            );
        }
    }

    // Copy border pixels
    for x in 0..w {
        out.put_pixel(x, 0, *rgb.get_pixel(x, 0));
        out.put_pixel(x, h - 1, *rgb.get_pixel(x, h - 1));
    }
    for y in 0..h {
        out.put_pixel(0, y, *rgb.get_pixel(0, y));
        out.put_pixel(w - 1, y, *rgb.get_pixel(w - 1, y));
    }

    DynamicImage::ImageRgb8(out)
}

/// Boost image contrast using histogram stretching.
pub fn boost_contrast(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    // Find min/max luminance
    let mut min_lum = 255u8;
    let mut max_lum = 0u8;

    for px in rgb.pixels() {
        let lum = ((px.0[0] as u32 * 299 + px.0[1] as u32 * 587 + px.0[2] as u32 * 114) / 1000)
            as u8;
        min_lum = min_lum.min(lum);
        max_lum = max_lum.max(lum);
    }

    if max_lum <= min_lum {
        return img.clone();
    }

    // Stretch to full range with a slight clip (1% on each end for robustness)
    let range = (max_lum - min_lum) as f64;
    let low = min_lum as f64 + range * 0.01;
    let high = max_lum as f64 - range * 0.01;
    let span = (high - low).max(1.0);

    let mut out = RgbImage::new(w, h);
    for (x, y, px) in rgb.enumerate_pixels() {
        let r = stretch(px.0[0], low, span);
        let g = stretch(px.0[1], low, span);
        let b = stretch(px.0[2], low, span);
        out.put_pixel(x, y, Rgb([r, g, b]));
    }

    DynamicImage::ImageRgb8(out)
}

fn stretch(val: u8, low: f64, span: f64) -> u8 {
    let v = ((val as f64 - low) / span * 255.0).clamp(0.0, 255.0);
    v as u8
}

/// Sharpen using unsharp mask: original + strength * (original - blurred).
pub fn sharpen(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let blurred = denoise(img).to_rgb8();
    let (w, h) = rgb.dimensions();

    let strength = 0.8f64;
    let mut out = RgbImage::new(w, h);

    for (x, y, px) in rgb.enumerate_pixels() {
        let bl = blurred.get_pixel(x, y);
        let r = (px.0[0] as f64 + strength * (px.0[0] as f64 - bl.0[0] as f64)).clamp(0.0, 255.0);
        let g = (px.0[1] as f64 + strength * (px.0[1] as f64 - bl.0[1] as f64)).clamp(0.0, 255.0);
        let b = (px.0[2] as f64 + strength * (px.0[2] as f64 - bl.0[2] as f64)).clamp(0.0, 255.0);
        out.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
    }

    DynamicImage::ImageRgb8(out)
}

/// Threshold + cleanup for logos: convert to high-contrast binary-ish image.
pub fn threshold_cleanup(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (w, h) = gray.dimensions();

    // Otsu-like threshold: use mean as simple threshold
    let sum: u64 = gray.pixels().map(|p| p.0[0] as u64).sum();
    let count = (w * h) as u64;
    let threshold = (sum / count.max(1)) as u8;

    let mut out = RgbImage::new(w, h);
    for (x, y, px) in gray.enumerate_pixels() {
        let val = if px.0[0] > threshold { 255u8 } else { 0u8 };
        out.put_pixel(x, y, Rgb([val, val, val]));
    }

    DynamicImage::ImageRgb8(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denoise_preserves_dimensions() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(50, 50));
        let result = denoise(&img);
        assert_eq!(result.dimensions(), (50, 50));
    }

    #[test]
    fn sharpen_preserves_dimensions() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(50, 50));
        let result = sharpen(&img);
        assert_eq!(result.dimensions(), (50, 50));
    }
}
