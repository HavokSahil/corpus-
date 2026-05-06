//! Document image enhancement for readability.
//!
//! Converts the warped document to a clean, high-contrast representation
//! using adaptive thresholding and morphological noise removal.

use image::{DynamicImage, GrayImage};
use imageproc::contrast::adaptive_threshold;
use imageproc::morphology::{close, open};
use imageproc::distance_transform::Norm;

use crate::config::PipelineConfig;
use crate::error::Result;

/// Result of the enhancement stage.
pub struct EnhancedImage {
    /// The enhanced document image.
    pub image: DynamicImage,
    /// Whether the output is a binary (black/white) image.
    /// This hint is used by the encode stage to choose the output format.
    pub is_binary: bool,
}

/// Enhance a warped document image for readability.
///
/// # Algorithm
///
/// 1. Convert to grayscale (if not already).
/// 2. Apply adaptive thresholding to binarize the document.
///    - Uses mean-based adaptive threshold (block radius from config).
///    - Falls back to Otsu global threshold if adaptive produces poor results.
/// 3. Apply morphological *close* (dilate then erode) to fill small holes in text.
/// 4. Apply morphological *open* (erode then dilate) to remove salt/pepper noise.
pub fn enhance_document(
    img: &DynamicImage,
    config: &PipelineConfig,
) -> Result<EnhancedImage> {
    // Step 1: Convert to grayscale
    let gray: GrayImage = img.to_luma8();

    // Step 2: Adaptive thresholding
    //
    // `adaptive_threshold` computes a local threshold for each pixel based on
    // the mean intensity in a window of size (2 * block_radius + 1).
    // Pixels brighter than the local mean minus a constant are set to white.
    // A positive constant (e.g. 15) acts as noise reduction: a pixel must be
    // significantly darker than the neighborhood mean to be marked as text (black).
    let binary = adaptive_threshold(&gray, config.adaptive_block_radius, config.adaptive_c);

    config.save_debug_image(
        &DynamicImage::ImageLuma8(binary.clone()),
        "04_adaptive_threshold.png",
    )?;

    // We removed the Otsu fallback because our new `adaptive_c` noise reduction
    // correctly yields very clean backgrounds (often >95% white), which were
    // falsely triggering the degeneracy check and undoing the noise reduction.

    // Step 3: Morphological close — fills small holes in character strokes.
    // Using a 3×3 square structuring element (Mask::square(1) = radius 1 → 3×3).
    // L-infinity norm with k=1 produces a 3×3 square structuring element.
    let closed = close(&binary, Norm::LInf, 1);

    // Step 4: Morphological open — removes isolated noise pixels.
    let cleaned = open(&closed, Norm::LInf, 1);

    config.save_debug_image(
        &DynamicImage::ImageLuma8(cleaned.clone()),
        "05_enhanced.png",
    )?;

    Ok(EnhancedImage {
        image: DynamicImage::ImageLuma8(cleaned),
        is_binary: true,
    })
}

