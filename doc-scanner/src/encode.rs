//! Format-aware image encoding and output.
//!
//! Chooses the optimal output format based on image content:
//! - Binary/thresholded images → PNG (lossless, very compact for bi-level)
//! - Grayscale/color images → JPEG (configurable quality)
//!
//! The output format can also be overridden via config or file extension.

use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ImageEncoder};

use crate::config::{OutputFormat, PipelineConfig};
use crate::enhance::EnhancedImage;
use crate::error::{PipelineError, Result};

/// Save the processed image to disk with format-aware encoding.
///
/// # Format selection priority
///
/// 1. If `config.output_format` is `Png` or `Jpeg`, use that.
/// 2. If `Auto`, infer from the output file extension.
/// 3. If the extension is ambiguous, use the `is_binary` hint:
///    - binary → PNG
///    - grayscale/color → JPEG
pub fn save(
    enhanced: &EnhancedImage,
    output_path: &Path,
    config: &PipelineConfig,
) -> Result<()> {
    let format = resolve_format(output_path, config.output_format, enhanced.is_binary);

    log::info!("encoding output as {format:?} → {}", output_path.display());

    match format {
        ResolvedFormat::Png => save_png(&enhanced.image, output_path),
        ResolvedFormat::Jpeg => save_jpeg(&enhanced.image, output_path, config.jpeg_quality),
    }
}

// ---------------------------------------------------------------------------
// Internal format resolution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum ResolvedFormat {
    Png,
    Jpeg,
}

/// Determine the final output format.
fn resolve_format(
    path: &Path,
    configured: OutputFormat,
    is_binary: bool,
) -> ResolvedFormat {
    match configured {
        OutputFormat::Png => ResolvedFormat::Png,
        OutputFormat::Jpeg => ResolvedFormat::Jpeg,
        OutputFormat::Auto => {
            // Try to infer from file extension first.
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_ascii_lowercase().as_str() {
                    "png" => return ResolvedFormat::Png,
                    "jpg" | "jpeg" => return ResolvedFormat::Jpeg,
                    _ => {}
                }
            }
            // Fall back to content-based decision.
            if is_binary {
                ResolvedFormat::Png
            } else {
                ResolvedFormat::Jpeg
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Encoder wrappers
// ---------------------------------------------------------------------------

fn save_png(img: &DynamicImage, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let encoder = PngEncoder::new(writer);

    let rgba = img.to_rgba8();
    encoder
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| PipelineError::EncodingError(e.to_string()))?;

    log::info!("saved PNG: {}", path.display());
    Ok(())
}

fn save_jpeg(img: &DynamicImage, path: &Path, quality: u8) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let encoder = JpegEncoder::new_with_quality(writer, quality);

    let rgb = img.to_rgb8();
    encoder
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| PipelineError::EncodingError(e.to_string()))?;

    log::info!("saved JPEG (quality={quality}): {}", path.display());
    Ok(())
}
