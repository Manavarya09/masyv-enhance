pub mod analyze;
pub mod enhance;
pub mod export;
pub mod upscale;
pub mod vectorize;

/// Intermediate result from pipeline processing — either a raster image or SVG string.
pub enum ProcessedOutput {
    Raster(image::DynamicImage),
    Vector(String),
}
