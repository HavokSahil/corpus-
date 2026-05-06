//! Batch document scanner — process entire directories or zip archives.
//!
//! ```bash
//! # Directory → Directory
//! batch-scanner ./photos/ ./scanned/
//!
//! # Zip → Zip
//! batch-scanner documents.zip scanned.zip
//!
//! # Directory → Zip
//! batch-scanner ./photos/ scanned.zip
//!
//! # Zip → Directory
//! batch-scanner documents.zip ./scanned/
//! ```

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use doc_scanner::config::{OutputFormat, PipelineConfig};
use doc_scanner::pipeline;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Batch document scanner — process directories or zip archives of images.
#[derive(Parser, Debug)]
#[command(
    name = "batch-scanner",
    version,
    about = "Batch-process document photos from a directory or zip archive."
)]
struct Cli {
    /// Input source: a directory path or a .zip file
    input: PathBuf,

    /// Output destination: a directory path or a .zip file
    output: PathBuf,

    /// Target maximum width in pixels (aspect ratio preserved)
    #[arg(long, default_value_t = 1200)]
    max_width: u32,

    /// JPEG output quality (1–100)
    #[arg(long, default_value_t = 80)]
    quality: u8,

    /// Output image format: auto, png, or jpeg
    #[arg(long, default_value = "auto")]
    format: String,

    /// Save intermediate debug images alongside outputs
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Canny edge detector low threshold
    #[arg(long, default_value_t = 40.0)]
    canny_low: f32,

    /// Canny edge detector high threshold
    #[arg(long, default_value_t = 120.0)]
    canny_high: f32,

    /// Block radius for adaptive thresholding (window = 2×radius + 1)
    #[arg(long, default_value_t = 20)]
    adaptive_block_radius: u32,

    /// Noise reduction constant for adaptive thresholding (higher = less noise)
    #[arg(long, default_value_t = 8)]
    adaptive_c: i32,

    /// Enhancement mode: binary, grayscale, or color
    #[arg(long, default_value = "binary")]
    enhance_mode: String,

    /// Show a visual progress bar instead of per-image logs
    #[arg(long, default_value_t = false)]
    progress: bool,
}

// ---------------------------------------------------------------------------
// Source / Sink abstractions
// ---------------------------------------------------------------------------

/// Recognised image extensions.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp"];

fn is_image_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn is_zip_path(p: &Path) -> bool {
    p.extension()
        .and_then(OsStr::to_str)
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

/// One input image to process.
struct InputImage {
    /// Relative name used for the output (e.g. "page01.jpg").
    name: String,
    /// Raw image bytes.
    data: Vec<u8>,
}

/// Collect all images from the input source.
fn collect_inputs(src: &Path) -> io::Result<Vec<InputImage>> {
    if is_zip_path(src) {
        collect_from_zip(src)
    } else {
        collect_from_dir(src)
    }
}

fn collect_from_dir(dir: &Path) -> io::Result<Vec<InputImage>> {
    let mut images = Vec::new();
    for entry in WalkDir::new(dir).sort_by_file_name().into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry
            .path()
            .strip_prefix(dir)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        if !is_image_name(&name) {
            continue;
        }

        let data = fs::read(entry.path())?;
        images.push(InputImage { name, data });
    }
    Ok(images)
}

fn collect_from_zip(zip_path: &Path) -> io::Result<Vec<InputImage>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut images = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        if !is_image_name(&name) {
            continue;
        }

        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data)?;
        images.push(InputImage { name, data });
    }
    Ok(images)
}

// ---------------------------------------------------------------------------
// Output writing
// ---------------------------------------------------------------------------

/// Derive an output filename from the input name, honouring the chosen format.
fn output_name(input_name: &str, format: &OutputFormat) -> String {
    let stem = Path::new(input_name)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("output");

    let ext = match format {
        OutputFormat::Png => "png",
        OutputFormat::Jpeg => "jpeg",
        OutputFormat::Auto => "png", // binary images default to PNG
    };

    format!("{stem}.{ext}")
}

/// Write all results to a directory.
fn write_to_dir(
    dir: &Path,
    results: &[(String, Vec<u8>)],
) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for (name, data) in results {
        let out_path = dir.join(name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, data)?;
    }
    Ok(())
}

/// Write all results into a zip archive.
fn write_to_zip(
    zip_path: &Path,
    results: &[(String, Vec<u8>)],
) -> io::Result<()> {
    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (name, data) in results {
        zip.start_file(name, options)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        zip.write_all(data)?;
    }

    zip.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.progress && !cli.debug {
        "warn"
    } else {
        "info"
    };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level),
    )
    .format_timestamp(None)
    .init();

    let output_format: OutputFormat = cli
        .format
        .parse()
        .unwrap_or_else(|e: String| {
            log::error!("{e}");
            std::process::exit(1);
        });

    // ---- Collect inputs ----
    log::info!("collecting images from: {}", cli.input.display());
    let inputs = match collect_inputs(&cli.input) {
        Ok(v) => v,
        Err(e) => {
            log::error!("failed to read input: {e}");
            std::process::exit(1);
        }
    };

    if inputs.is_empty() {
        log::warn!("no images found in input source");
        std::process::exit(0);
    }

    log::info!("found {} image(s) to process", inputs.len());

    // ---- Process each image ----
    let total = inputs.len();
    let mut results: Vec<(String, Vec<u8>)> = Vec::with_capacity(total);
    let mut success = 0usize;
    let mut failed = 0usize;

    let pb = if cli.progress {
        let p = ProgressBar::new(total as u64);
        p.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) ETA: {eta} - {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        Some(p)
    } else {
        None
    };

    for (idx, input) in inputs.iter().enumerate() {
        let num = idx + 1;
        if let Some(ref p) = pb {
            p.set_message(input.name.clone());
        } else {
            log::info!("[{num}/{total}] processing: {}", input.name);
        }

        // Build a per-image config (debug dir is per-image)
        let debug_dir = if cli.debug {
            let stem = Path::new(&input.name)
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or("img");
            cli.output
                .parent()
                .unwrap_or(Path::new("."))
                .join(format!("debug_{stem}"))
        } else {
            PathBuf::from("debug")
        };

        // Parse enhance mode
        let enhance_mode = match cli.enhance_mode.to_lowercase().as_str() {
            "binary" => doc_scanner::config::EnhanceMode::Binary,
            "grayscale" => doc_scanner::config::EnhanceMode::Grayscale,
            "color" => doc_scanner::config::EnhanceMode::Color,
            _ => doc_scanner::config::EnhanceMode::Binary,
        };

        let config = PipelineConfig {
            enhance_mode,
            max_width: cli.max_width,
            jpeg_quality: cli.quality,
            output_format,
            canny_low: cli.canny_low,
            canny_high: cli.canny_high,
            adaptive_block_radius: cli.adaptive_block_radius,
            adaptive_c: cli.adaptive_c,
            debug: cli.debug,
            debug_dir,
        };

        // Decode input bytes into a DynamicImage
        let img = match image::load_from_memory(&input.data) {
            Ok(img) => img,
            Err(e) => {
                log::error!("[{num}/{total}] failed to decode {}: {e}", input.name);
                failed += 1;
                continue;
            }
        };

        // Run the pipeline in memory
        let enhanced = match pipeline::run_in_memory(&img, &config) {
            Ok(e) => e,
            Err(e) => {
                log::error!("[{num}/{total}] pipeline failed for {}: {e}", input.name);
                failed += 1;
                continue;
            }
        };

        // Encode to bytes
        let out_name = output_name(&input.name, &output_format);
        let mut buf = Cursor::new(Vec::new());
        let encode_result = match output_format {
            OutputFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buf,
                    cli.quality,
                );
                enhanced.image.to_rgb8().write_with_encoder(encoder)
            }
            _ => {
                // PNG for Auto/Png
                let encoder = image::codecs::png::PngEncoder::new(&mut buf);
                enhanced.image.to_luma8().write_with_encoder(encoder)
            }
        };

        match encode_result {
            Ok(()) => {
                results.push((out_name.clone(), buf.into_inner()));
                success += 1;
                if pb.is_none() {
                    log::info!("[{num}/{total}] ✓ {}", out_name);
                }
            }
            Err(e) => {
                let err_msg = format!("[{num}/{total}] encoding failed for {}: {e}", input.name);
                if let Some(ref p) = pb {
                    p.println(&err_msg);
                } else {
                    log::error!("{}", err_msg);
                }
                failed += 1;
            }
        }

        if let Some(ref p) = pb {
            p.inc(1);
        }
    }

    if let Some(ref p) = pb {
        p.finish_with_message("Done");
    }

    // ---- Write outputs ----
    log::info!("writing {} result(s) to: {}", results.len(), cli.output.display());

    let write_result = if is_zip_path(&cli.output) {
        write_to_zip(&cli.output, &results)
    } else {
        write_to_dir(&cli.output, &results)
    };

    if let Err(e) = write_result {
        log::error!("failed to write output: {e}");
        std::process::exit(1);
    }

    // ---- Summary ----
    log::info!("batch complete: {success} succeeded, {failed} failed, {total} total");
    if failed > 0 {
        std::process::exit(1);
    }
}
