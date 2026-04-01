use anyhow::Result;
use image::{DynamicImage, RgbImage};
use ndarray::Array4;
use ort::value::Tensor;
use std::path::Path;
use std::sync::Mutex;

const TILE_SIZE: u32 = 512;
const TILE_PAD: u32 = 16;
const NATIVE_SCALE: u32 = 4;

/// ONNX-based image upscaler wrapping Real-ESRGAN.
pub struct OnnxUpscaler {
    session: Mutex<ort::session::Session>,
}

/// Global model cache — load once per process.
static MODEL_CACHE: std::sync::OnceLock<Result<OnnxUpscaler, String>> = std::sync::OnceLock::new();

impl OnnxUpscaler {
    /// Load an ONNX model from disk. Uses a global cache so the model is loaded only once.
    pub fn load(model_path: &Path) -> Result<&'static OnnxUpscaler> {
        let result = MODEL_CACHE.get_or_init(|| {
            tracing::info!(path = %model_path.display(), "loading ONNX model");
            match Self::load_inner(model_path) {
                Ok(m) => Ok(m),
                Err(e) => Err(format!("{e:#}")),
            }
        });

        match result {
            Ok(m) => Ok(m),
            Err(e) => anyhow::bail!("failed to load model: {e}"),
        }
    }

    fn load_inner(model_path: &Path) -> Result<OnnxUpscaler> {
        let session = ort::session::Session::builder()
            .map_err(|e| anyhow::anyhow!("failed to create session builder: {e}"))?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("failed to set optimization level: {e}"))?
            .with_intra_threads(
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4),
            )
            .map_err(|e| anyhow::anyhow!("failed to set thread count: {e}"))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("failed to load ONNX model: {e}"))?;

        tracing::info!("ONNX model loaded successfully");
        Ok(OnnxUpscaler {
            session: Mutex::new(session),
        })
    }

    /// Run inference on an image. Handles tile-based processing for large images.
    pub fn infer(&self, img: &DynamicImage) -> Result<DynamicImage> {
        let rgb = img.to_rgb8();
        let (w, h) = (rgb.width(), rgb.height());

        if w <= TILE_SIZE && h <= TILE_SIZE {
            self.infer_single(&rgb)
        } else {
            tracing::info!(
                width = w,
                height = h,
                tile_size = TILE_SIZE,
                "image too large for single-pass, using tiled inference"
            );
            self.infer_tiled(&rgb)
        }
    }

    /// Process a single image tile through the model.
    fn infer_single(&self, rgb: &RgbImage) -> Result<DynamicImage> {
        let (w, h) = (rgb.width(), rgb.height());

        // Build NCHW f32 tensor normalized to [0, 1]
        let tensor_data = image_to_nchw(rgb);
        let input_tensor = Tensor::from_array(tensor_data)
            .map_err(|e| anyhow::anyhow!("failed to create input tensor: {e}"))?;

        // Run inference
        let mut session = self.session.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let outputs = session
            .run(ort::inputs![input_tensor])
            .map_err(|e| anyhow::anyhow!("ONNX inference failed: {e}"))?;

        // Extract output — Real-ESRGAN outputs [1, 3, H*4, W*4]
        let output_value = &outputs[0];
        let (shape, data) = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow::anyhow!("failed to extract output tensor: {e}"))?;

        let out_w = w * NATIVE_SCALE;
        let out_h = h * NATIVE_SCALE;

        // Shape derefs to &[i64]
        let shape_dims: Vec<i64> = shape.iter().copied().collect();
        let result = nchw_to_image(data, &shape_dims, out_w, out_h);

        Ok(DynamicImage::ImageRgb8(result))
    }

    /// Tile-based inference: split image into overlapping tiles, process each, stitch.
    fn infer_tiled(&self, rgb: &RgbImage) -> Result<DynamicImage> {
        let (w, h) = (rgb.width(), rgb.height());
        let out_w = w * NATIVE_SCALE;
        let out_h = h * NATIVE_SCALE;
        let mut output = RgbImage::new(out_w, out_h);

        let tiles_x = (w + TILE_SIZE - 1) / TILE_SIZE;
        let tiles_y = (h + TILE_SIZE - 1) / TILE_SIZE;

        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let x0 = (tx * TILE_SIZE).saturating_sub(TILE_PAD);
                let y0 = (ty * TILE_SIZE).saturating_sub(TILE_PAD);
                let x1 = ((tx + 1) * TILE_SIZE + TILE_PAD).min(w);
                let y1 = ((ty + 1) * TILE_SIZE + TILE_PAD).min(h);

                let tile =
                    image::imageops::crop_imm(rgb, x0, y0, x1 - x0, y1 - y0).to_image();
                let upscaled_tile = self.infer_single(&tile)?;
                let upscaled_rgb = upscaled_tile.to_rgb8();

                let pad_left = (tx * TILE_SIZE - x0) * NATIVE_SCALE;
                let pad_top = (ty * TILE_SIZE - y0) * NATIVE_SCALE;
                let dst_x = tx * TILE_SIZE * NATIVE_SCALE;
                let dst_y = ty * TILE_SIZE * NATIVE_SCALE;

                let copy_w = (TILE_SIZE * NATIVE_SCALE).min(out_w - dst_x);
                let copy_h = (TILE_SIZE * NATIVE_SCALE).min(out_h - dst_y);

                for row in 0..copy_h {
                    for col in 0..copy_w {
                        let src_x = pad_left + col;
                        let src_y = pad_top + row;
                        if src_x < upscaled_rgb.width() && src_y < upscaled_rgb.height() {
                            let px = *upscaled_rgb.get_pixel(src_x, src_y);
                            let out_x = dst_x + col;
                            let out_y = dst_y + row;
                            if out_x < out_w && out_y < out_h {
                                output.put_pixel(out_x, out_y, px);
                            }
                        }
                    }
                }

                tracing::debug!(tile_x = tx, tile_y = ty, "processed tile");
            }
        }

        Ok(DynamicImage::ImageRgb8(output))
    }
}

/// Convert an RGB image to an NCHW f32 Array4 normalized to [0, 1].
fn image_to_nchw(rgb: &RgbImage) -> Array4<f32> {
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let mut tensor = Array4::<f32>::zeros((1, 3, h, w));

    for y in 0..h {
        for x in 0..w {
            let px = rgb.get_pixel(x as u32, y as u32);
            tensor[[0, 0, y, x]] = px.0[0] as f32 / 255.0;
            tensor[[0, 1, y, x]] = px.0[1] as f32 / 255.0;
            tensor[[0, 2, y, x]] = px.0[2] as f32 / 255.0;
        }
    }

    tensor
}

/// Convert NCHW f32 flat slice back to an RGB image.
fn nchw_to_image(data: &[f32], shape: &[i64], width: u32, height: u32) -> RgbImage {
    let mut img = RgbImage::new(width, height);

    // shape is [1, 3, H, W] or [3, H, W]
    let (c_stride, h_stride) = if shape.len() == 4 {
        let h = shape[2] as usize;
        let w = shape[3] as usize;
        (h * w, w)
    } else {
        let h = shape[1] as usize;
        let w = shape[2] as usize;
        (h * w, w)
    };

    let base = if shape.len() == 4 {
        shape[1] as usize * shape[2] as usize * shape[3] as usize * 0 // batch 0
    } else {
        0
    };

    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx_r = base + 0 * c_stride + y * h_stride + x;
            let idx_g = base + 1 * c_stride + y * h_stride + x;
            let idx_b = base + 2 * c_stride + y * h_stride + x;

            if idx_b < data.len() {
                let r = (data[idx_r] * 255.0).clamp(0.0, 255.0) as u8;
                let g = (data[idx_g] * 255.0).clamp(0.0, 255.0) as u8;
                let b = (data[idx_b] * 255.0).clamp(0.0, 255.0) as u8;
                img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }
    }

    img
}
