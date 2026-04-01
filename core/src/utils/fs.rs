use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// List all image files in a directory (non-recursive).
pub fn list_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let extensions = ["png", "jpg", "jpeg", "webp", "bmp", "tiff", "tif", "gif"];

    let mut images = Vec::new();
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    images.push(path);
                }
            }
        }
    }

    images.sort();
    Ok(images)
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Generate an output path from an input path, suffix, and extension.
pub fn output_path(input: &Path, suffix: &str, extension: &str) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let dir = input.parent().unwrap_or(Path::new("."));
    dir.join(format!("{stem}{suffix}.{extension}"))
}
