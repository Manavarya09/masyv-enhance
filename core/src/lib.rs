pub mod models;
pub mod pipeline;
pub mod utils;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Detected image content type for smart routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    Photo,
    Logo,
    Text,
    Illustration,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Photo => write!(f, "photo"),
            Self::Logo => write!(f, "logo"),
            Self::Text => write!(f, "text"),
            Self::Illustration => write!(f, "illustration"),
        }
    }
}

/// Processing mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnhanceMode {
    Smart,
    Upscale,
    Vectorize,
    Enhance,
}

impl std::fmt::Display for EnhanceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smart => write!(f, "smart"),
            Self::Upscale => write!(f, "upscale"),
            Self::Vectorize => write!(f, "vectorize"),
            Self::Enhance => write!(f, "enhance"),
        }
    }
}

/// Output format for processed images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
    Svg,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Svg => "svg",
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// A request to the enhance engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhanceRequest {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub mode: EnhanceMode,
    pub scale: u32,
    pub format: OutputFormat,
    pub jpeg_quality: u8,
    pub model_dir: Option<PathBuf>,
}

impl Default for EnhanceRequest {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            output_path: None,
            mode: EnhanceMode::Smart,
            scale: 4,
            format: OutputFormat::Png,
            jpeg_quality: 90,
            model_dir: None,
        }
    }
}

/// Result returned after processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhanceResult {
    pub output_path: PathBuf,
    pub detected_type: Option<ImageType>,
    pub mode_used: EnhanceMode,
    pub input_dimensions: (u32, u32),
    pub output_dimensions: (u32, u32),
    pub processing_time_ms: u64,
    pub format: OutputFormat,
}

/// Top-level engine that orchestrates the full pipeline.
pub struct Engine {
    model_dir: PathBuf,
}

impl Engine {
    pub fn new(model_dir: PathBuf) -> Self {
        Self { model_dir }
    }

    /// Run the full enhancement pipeline.
    pub fn process(&self, request: &EnhanceRequest) -> anyhow::Result<EnhanceResult> {
        let start = std::time::Instant::now();
        tracing::info!(input = %request.input_path.display(), mode = %request.mode, "starting enhancement");

        // Load input image
        let img = utils::image::load_image(&request.input_path)?;
        let input_dimensions = (img.width(), img.height());
        tracing::info!(width = input_dimensions.0, height = input_dimensions.1, "loaded input image");

        // Determine image type (for smart mode)
        let detected_type = if request.mode == EnhanceMode::Smart {
            let t = pipeline::analyze::detect_image_type(&img);
            tracing::info!(detected = %t, "image type detected");
            Some(t)
        } else {
            None
        };

        // Route through pipeline
        let (processed, effective_mode) = match request.mode {
            EnhanceMode::Smart => {
                let image_type = detected_type.unwrap();
                self.smart_pipeline(img, image_type, request)?
            }
            EnhanceMode::Upscale => {
                let result = pipeline::upscale::upscale(
                    &img,
                    request.scale,
                    &self.model_dir,
                )?;
                (pipeline::ProcessedOutput::Raster(result), EnhanceMode::Upscale)
            }
            EnhanceMode::Vectorize => {
                let svg = pipeline::vectorize::vectorize(&img)?;
                (pipeline::ProcessedOutput::Vector(svg), EnhanceMode::Vectorize)
            }
            EnhanceMode::Enhance => {
                let result = pipeline::enhance::enhance_photo(&img);
                (pipeline::ProcessedOutput::Raster(result), EnhanceMode::Enhance)
            }
        };

        // Determine output path
        let output_path = request.output_path.clone().unwrap_or_else(|| {
            let stem = request.input_path.file_stem().unwrap().to_string_lossy();
            let ext = request.format.extension();
            request.input_path.with_file_name(format!("{stem}_enhanced.{ext}"))
        });

        // Export
        let output_dimensions = pipeline::export::export(
            &processed,
            &output_path,
            request.format,
            request.jpeg_quality,
        )?;

        let elapsed = start.elapsed();
        tracing::info!(elapsed_ms = elapsed.as_millis(), output = %output_path.display(), "enhancement complete");

        Ok(EnhanceResult {
            output_path,
            detected_type,
            mode_used: effective_mode,
            input_dimensions,
            output_dimensions,
            processing_time_ms: elapsed.as_millis() as u64,
            format: request.format,
        })
    }

    fn smart_pipeline(
        &self,
        img: image::DynamicImage,
        image_type: ImageType,
        request: &EnhanceRequest,
    ) -> anyhow::Result<(pipeline::ProcessedOutput, EnhanceMode)> {
        match image_type {
            ImageType::Photo => {
                // Photo pipeline: denoise → AI upscale → sharpen
                let denoised = pipeline::enhance::denoise(&img);
                let upscaled = pipeline::upscale::upscale(
                    &denoised,
                    request.scale,
                    &self.model_dir,
                )?;
                let sharpened = pipeline::enhance::sharpen(&upscaled);
                Ok((pipeline::ProcessedOutput::Raster(sharpened), EnhanceMode::Smart))
            }
            ImageType::Logo => {
                if request.format == OutputFormat::Svg {
                    // Logo → vectorize pipeline
                    let cleaned = pipeline::enhance::threshold_cleanup(&img);
                    let svg = pipeline::vectorize::vectorize(&cleaned)?;
                    Ok((pipeline::ProcessedOutput::Vector(svg), EnhanceMode::Smart))
                } else {
                    // Logo but raster output requested — clean up and upscale
                    let cleaned = pipeline::enhance::threshold_cleanup(&img);
                    let upscaled = pipeline::upscale::upscale(
                        &cleaned,
                        request.scale,
                        &self.model_dir,
                    )?;
                    Ok((pipeline::ProcessedOutput::Raster(upscaled), EnhanceMode::Smart))
                }
            }
            ImageType::Text => {
                // Text pipeline: contrast boost → edge sharpen
                let boosted = pipeline::enhance::boost_contrast(&img);
                let sharpened = pipeline::enhance::sharpen(&boosted);
                Ok((pipeline::ProcessedOutput::Raster(sharpened), EnhanceMode::Smart))
            }
            ImageType::Illustration => {
                // Illustration: gentle denoise → upscale
                let denoised = pipeline::enhance::denoise(&img);
                let upscaled = pipeline::upscale::upscale(
                    &denoised,
                    request.scale,
                    &self.model_dir,
                )?;
                Ok((pipeline::ProcessedOutput::Raster(upscaled), EnhanceMode::Smart))
            }
        }
    }
}
