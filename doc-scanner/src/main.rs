//! CLI entry point for the document scanning pipeline.
//!
//! ```bash
//! # Basic usage
//! cargo run -- input.jpg output.png
//!
//! # With debug output
//! cargo run -- input.jpg output.png --debug
//!
//! # Customize parameters
//! cargo run -- photo.jpg scan.jpeg --quality 90 --max-width 1600
//! ```

use std::path::PathBuf;

use clap::Parser;

use doc_scanner::config::{OutputFormat, PipelineConfig};
use doc_scanner::pipeline;

/// Document image processing pipeline — detect, warp, enhance, resize, encode.
#[derive(Parser, Debug)]
#[command(
    name = "doc-scanner",
    version,
    about = "Process a photo of a document into a clean, flat scan."
)]
struct Cli {
    /// Path to the input image (JPEG, PNG, etc.)
    input: PathBuf,

    /// Path for the output image
    output: PathBuf,

    /// Target maximum width in pixels (aspect ratio is preserved)
    #[arg(long, default_value_t = 1200)]
    max_width: u32,

    /// JPEG output quality (1–100)
    #[arg(long, default_value_t = 80)]
    quality: u8,

    /// Output format: auto, png, or jpeg
    #[arg(long, default_value = "auto")]
    format: String,

    /// Save intermediate debug images (edges, contours, warped, thresholded)
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Directory for debug output images
    #[arg(long, default_value = "debug")]
    debug_dir: PathBuf,

    /// Canny edge detector low threshold
    #[arg(long, default_value_t = 50.0)]
    canny_low: f32,

    /// Canny edge detector high threshold
    #[arg(long, default_value_t = 150.0)]
    canny_high: f32,

    /// Block radius for adaptive thresholding (window = 2×radius + 1)
    #[arg(long, default_value_t = 15)]
    adaptive_block_radius: u32,

    /// Noise reduction constant for adaptive thresholding (higher = less noise)
    #[arg(long, default_value_t = 15)]
    adaptive_c: i32,
}

fn main() {
    // Initialize the logger. Set RUST_LOG=debug for verbose output.
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp(None)
    .init();

    let cli = Cli::parse();

    // Parse the output format string.
    let output_format: OutputFormat = cli
        .format
        .parse()
        .unwrap_or_else(|e: String| {
            log::error!("{e}");
            std::process::exit(1);
        });

    // Build pipeline configuration from CLI arguments.
    let config = PipelineConfig {
        max_width: cli.max_width,
        jpeg_quality: cli.quality,
        output_format,
        canny_low: cli.canny_low,
        canny_high: cli.canny_high,
        adaptive_block_radius: cli.adaptive_block_radius,
        adaptive_c: cli.adaptive_c,
        debug: cli.debug,
        debug_dir: cli.debug_dir,
    };

    // Run the pipeline.
    if let Err(e) = pipeline::run(&cli.input, &cli.output, &config) {
        log::error!("pipeline failed: {e}");
        std::process::exit(1);
    }
}
