//! Aspect-ratio-preserving image resizing.
//!
//! Normalizes the document image to a target maximum width while
//! maintaining the original aspect ratio. Uses appropriate interpolation
//! filters for downscaling vs upscaling.

use image::imageops::FilterType;
use image::DynamicImage;

use crate::config::PipelineConfig;
use crate::error::Result;

/// Resize the image so its width does not exceed `config.max_width`.
///
/// - If the image is already narrower, it is returned unchanged.
/// - Downscaling uses `Triangle` (area-based) interpolation for sharpness.
/// - Upscaling uses `CatmullRom` (cubic) for smooth results.
pub fn normalize(img: &DynamicImage, config: &PipelineConfig) -> Result<DynamicImage> {
    let (w, h) = (img.width(), img.height());

    if w <= config.max_width {
        log::info!("image width ({w}) ≤ max_width ({}), skipping resize", config.max_width);
        return Ok(img.clone());
    }

    // Compute new dimensions preserving aspect ratio.
    let scale = config.max_width as f64 / w as f64;
    let new_w = config.max_width;
    let new_h = (h as f64 * scale).round() as u32;

    // Choose interpolation filter based on scale direction.
    let filter = if scale < 1.0 {
        FilterType::Triangle // good for downscaling (area-like averaging)
    } else {
        FilterType::CatmullRom // good for upscaling (smooth cubic)
    };

    log::info!("resizing: {w}×{h} → {new_w}×{new_h} (scale={scale:.3}, filter={filter:?})");

    let resized = img.resize_exact(new_w, new_h, filter);

    Ok(resized)
}
