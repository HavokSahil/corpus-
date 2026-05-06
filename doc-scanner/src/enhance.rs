//! Document image enhancement for readability.
//!
//! Converts the warped document to a clean, high-contrast representation
//! using adaptive thresholding and morphological noise removal.

use image::{DynamicImage, GrayImage};
use imageproc::contrast::adaptive_threshold;
use imageproc::morphology::{close, open};
use imageproc::distance_transform::Norm;

use crate::config::{EnhanceMode, PipelineConfig};
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
pub fn enhance_document(
    img: &DynamicImage,
    config: &PipelineConfig,
) -> Result<EnhancedImage> {
    match config.enhance_mode {
        EnhanceMode::Binary => {
            let gray: GrayImage = img.to_luma8();
            let binary = adaptive_threshold(&gray, config.adaptive_block_radius, config.adaptive_c);
            
            let final_binary = if config.use_morphology {
                let closed = close(&binary, Norm::LInf, 1);
                open(&closed, Norm::LInf, 1)
            } else {
                binary
            };

            config.save_debug_image(
                &DynamicImage::ImageLuma8(final_binary.clone()),
                "05_enhanced.png",
            )?;

            Ok(EnhancedImage {
                image: DynamicImage::ImageLuma8(final_binary),
                is_binary: true,
            })
        }
        EnhanceMode::Grayscale => {
            Ok(EnhancedImage {
                image: DynamicImage::ImageLuma8(img.to_luma8()),
                is_binary: false,
            })
        }
        EnhanceMode::Color => {
            Ok(EnhancedImage {
                image: img.clone(),
                is_binary: false,
            })
        }
    }
}

