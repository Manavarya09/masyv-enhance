use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;

use masyv_core::{Engine, EnhanceMode, EnhanceRequest, OutputFormat};

#[derive(Parser, Debug)]
#[command(
    name = "masyv-core",
    about = "MASYV Enhance Engine — AI-powered image enhancement",
    version
)]
struct Cli {
    /// Input image path (or directory with --batch)
    input: PathBuf,

    /// Processing mode
    #[arg(long, default_value = "smart", value_parser = parse_mode)]
    mode: EnhanceMode,

    /// Upscale factor
    #[arg(long, default_value = "4")]
    scale: u32,

    /// Output format
    #[arg(long, default_value = "png", value_parser = parse_format)]
    format: OutputFormat,

    /// Output path (auto-generated if omitted)
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// JPEG quality (1-100)
    #[arg(long, default_value = "90")]
    quality: u8,

    /// Directory containing ONNX models
    #[arg(long)]
    model_dir: Option<PathBuf>,

    /// Process all images in directory
    #[arg(long)]
    batch: bool,

    /// Output as JSON (for programmatic use)
    #[arg(long)]
    json: bool,

    /// Verbose logging
    #[arg(long, short)]
    verbose: bool,
}

fn parse_mode(s: &str) -> Result<EnhanceMode, String> {
    match s.to_lowercase().as_str() {
        "smart" => Ok(EnhanceMode::Smart),
        "upscale" => Ok(EnhanceMode::Upscale),
        "vectorize" => Ok(EnhanceMode::Vectorize),
        "enhance" => Ok(EnhanceMode::Enhance),
        _ => Err(format!("unknown mode: {s} (expected: smart, upscale, vectorize, enhance)")),
    }
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "png" => Ok(OutputFormat::Png),
        "jpg" | "jpeg" => Ok(OutputFormat::Jpeg),
        "webp" => Ok(OutputFormat::Webp),
        "svg" => Ok(OutputFormat::Svg),
        _ => Err(format!("unknown format: {s} (expected: png, jpeg, webp, svg)")),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with_target(false)
        .init();

    // Resolve model directory
    let model_dir = cli.model_dir.clone().unwrap_or_else(|| {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default();
        // Check next to binary, then in ../models/
        let beside = exe_dir.join("models");
        if beside.exists() {
            beside
        } else {
            exe_dir.join("../models")
        }
    });

    let engine = Engine::new(model_dir);

    if cli.batch {
        return run_batch(&engine, &cli);
    }

    // Single file processing
    if !cli.input.exists() {
        bail!("input file not found: {}", cli.input.display());
    }

    let request = EnhanceRequest {
        input_path: cli.input,
        output_path: cli.output,
        mode: cli.mode,
        scale: cli.scale,
        format: cli.format,
        jpeg_quality: cli.quality,
        model_dir: None,
    };

    let result = engine.process(&request)?;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Enhanced: {}", result.output_path.display());
        if let Some(t) = result.detected_type {
            println!("  Detected type: {t}");
        }
        println!("  Mode: {}", result.mode_used);
        println!("  Input:  {}x{}", result.input_dimensions.0, result.input_dimensions.1);
        println!("  Output: {}x{}", result.output_dimensions.0, result.output_dimensions.1);
        println!("  Format: {}", result.format);
        println!("  Time: {}ms", result.processing_time_ms);
    }

    Ok(())
}

fn run_batch(engine: &Engine, cli: &Cli) -> Result<()> {
    let images = masyv_core::utils::fs::list_images(&cli.input)?;
    if images.is_empty() {
        bail!("no images found in: {}", cli.input.display());
    }

    tracing::info!(count = images.len(), "batch processing images");

    let results: Vec<_> = images
        .iter()
        .map(|path| {
            let request = EnhanceRequest {
                input_path: path.clone(),
                output_path: None,
                mode: cli.mode,
                scale: cli.scale,
                format: cli.format,
                jpeg_quality: cli.quality,
                model_dir: None,
            };
            (path, engine.process(&request))
        })
        .collect();

    let mut success = 0;
    let mut failed = 0;

    for (path, result) in &results {
        match result {
            Ok(r) => {
                success += 1;
                if cli.json {
                    println!("{}", serde_json::to_string(r).unwrap_or_default());
                } else {
                    println!("  OK: {} → {}", path.display(), r.output_path.display());
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("  FAIL: {} — {e:#}", path.display());
            }
        }
    }

    println!("\nBatch complete: {success} succeeded, {failed} failed");
    Ok(())
}
