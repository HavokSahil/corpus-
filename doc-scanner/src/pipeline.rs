//! Pipeline orchestration.
//!
//! Wires together all processing stages into a single `run` function:
//!
//! ```text
//! Input → Detect → Warp → Enhance → Resize → Encode
//! ```

use std::path::Path;

use image::DynamicImage;

use crate::config::PipelineConfig;
use crate::error::Result;
use crate::{detect, encode, enhance, resize, transform};

/// Execute the full document scanning pipeline.
///
/// # Arguments
///
/// * `input`  — path to the input image (JPEG, PNG, etc.)
/// * `output` — path where the processed image will be saved
/// * `config` — tuneable pipeline parameters
///
/// # Pipeline stages
///
/// 1. **Load**: Read the input image from disk.
/// 2. **Detect**: Find the document quadrilateral via edge/contour detection.
/// 3. **Warp**: Apply perspective correction to flatten the document.
/// 4. **Enhance**: Binarize and clean up for readability.
/// 5. **Resize**: Normalize to target resolution.
/// 6. **Encode**: Write the output in the appropriate format.
pub fn run(input: &Path, output: &Path, config: &PipelineConfig) -> Result<()> {
    // Stage 0: Load
    log::info!("loading input image: {}", input.display());
    let img = image::open(input)?;
    log::info!(
        "loaded: {}×{} ({:?})",
        img.width(),
        img.height(),
        img.color()
    );

    config.save_debug_image(&img, "00_input.png")?;

    // Stage 1: Detect document corners
    log::info!("detecting document boundaries...");
    let corners = detect::find_document(&img, config)?;
    log::info!("detected corners: {:?}", corners);

    // Stage 2: Perspective warp
    log::info!("applying perspective correction...");
    let warped = transform::warp_perspective(&img, &corners, config)?;
    log::info!("warped to {}×{}", warped.width(), warped.height());

    // Stage 3: Enhance
    log::info!("enhancing document...");
    let enhanced = enhance::enhance_document(&warped, config)?;
    log::info!(
        "enhanced: {}×{}, binary={}",
        enhanced.image.width(),
        enhanced.image.height(),
        enhanced.is_binary
    );

    // Stage 4: Resize
    log::info!("normalizing resolution...");
    let resized = resize::normalize(&enhanced.image, config)?;

    // Wrap in EnhancedImage to preserve the is_binary flag for encoding
    let final_image = enhance::EnhancedImage {
        image: resized,
        is_binary: enhanced.is_binary,
    };

    // Stage 5: Encode
    log::info!("encoding output...");
    encode::save(&final_image, output, config)?;

    log::info!("pipeline complete: {}", output.display());
    Ok(())
}

/// Execute the pipeline but return the processed image instead of saving it.
/// Useful for library consumers who want to further process the result.
pub fn run_in_memory(
    img: &DynamicImage,
    config: &PipelineConfig,
) -> Result<enhance::EnhancedImage> {
    let corners = detect::find_document(img, config)?;
    let warped = transform::warp_perspective(img, &corners, config)?;
    let enhanced = enhance::enhance_document(&warped, config)?;
    let resized = resize::normalize(&enhanced.image, config)?;

    Ok(enhance::EnhancedImage {
        image: resized,
        is_binary: enhanced.is_binary,
    })
}
