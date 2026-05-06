//! Pipeline configuration.
//!
//! All tuneable parameters live here so that modules remain pure functions
//! of `(image, config) → image`.

use std::path::PathBuf;

/// Supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Automatically choose based on image content:
    /// binary/thresholded → PNG, grayscale/photo → JPEG.
    Auto,
    Png,
    Jpeg,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(OutputFormat::Auto),
            "png" => Ok(OutputFormat::Png),
            "jpeg" | "jpg" => Ok(OutputFormat::Jpeg),
            other => Err(format!("unknown output format: `{other}` (expected auto|png|jpeg)")),
        }
    }
}

/// Complete set of tuneable parameters for the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    // -- Resize --
    /// Maximum output width in pixels. Images narrower than this are untouched.
    pub max_width: u32,

    // -- Encode --
    /// JPEG quality (1–100). Only used when output format is JPEG.
    pub jpeg_quality: u8,
    /// Desired output format.
    pub output_format: OutputFormat,

    // -- Detect (Canny) --
    /// Canny edge detector low threshold.
    pub canny_low: f32,
    /// Canny edge detector high threshold.
    pub canny_high: f32,

    // -- Enhance (adaptive threshold) --
    /// Block radius for adaptive thresholding (actual window = 2*radius + 1).
    pub adaptive_block_radius: u32,
    /// Constant added/subtracted for adaptive thresholding to remove background noise.
    pub adaptive_c: i32,

    // -- Debug --
    /// When true, intermediate images are written to `debug_dir`.
    pub debug: bool,
    /// Directory for debug output images.
    pub debug_dir: PathBuf,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_width: 1200,
            jpeg_quality: 80,
            output_format: OutputFormat::Auto,
            canny_low: 50.0,
            canny_high: 150.0,
            adaptive_block_radius: 15,
            adaptive_c: 15,
            debug: false,
            debug_dir: PathBuf::from("debug"),
        }
    }
}

impl PipelineConfig {
    /// Save a debug image if debug mode is enabled.
    /// `name` should be a simple filename like `"01_edges.png"`.
    pub fn save_debug_image(
        &self,
        img: &image::DynamicImage,
        name: &str,
    ) -> crate::error::Result<()> {
        if !self.debug {
            return Ok(());
        }
        std::fs::create_dir_all(&self.debug_dir)?;
        let path = self.debug_dir.join(name);
        img.save(&path)?;
        log::info!("debug: saved {}", path.display());
        Ok(())
    }
}
