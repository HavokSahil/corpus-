//! Async job execution for image processing.
//!
//! When images are uploaded, a job is created and processing happens in a
//! background Tokio task. Callers poll `/api/jobs/:id` for progress.

use std::io::{Cursor, Read};
use std::path::PathBuf;

use uuid::Uuid;

use doc_scanner::config::PipelineConfig;
use doc_scanner::pipeline::run_in_memory;

use crate::models::{ImageMeta, JobStatus};
use crate::state::AppState;

/// An image to be processed by the pipeline.
pub struct PendingImage {
    pub original_name: String,
    pub data: Vec<u8>,
}

/// Spawn a background task to process a batch of images into a corpus.
pub fn spawn_processing_job(
    state: AppState,
    job_id: Uuid,
    corpus_id: Uuid,
    images: Vec<PendingImage>,
    config: PipelineConfig,
) {
    tokio::spawn(async move {
        process_images(state, job_id, corpus_id, images, config).await;
    });
}

async fn process_images(
    state: AppState,
    job_id: Uuid,
    corpus_id: Uuid,
    images: Vec<PendingImage>,
    mut config: PipelineConfig,
) {
    // Mark job as running.
    if let Some(mut job) = state.jobs.get_mut(&job_id) {
        job.status = JobStatus::Running;
    }

    let images_dir = state.images_dir(corpus_id);
    std::fs::create_dir_all(&images_dir).ok();

    config.debug_dir = images_dir.join(".debug");

    for pending in images {
        let name = pending.original_name.clone();

        // Decode bytes
        let img = match image::load_from_memory(&pending.data) {
            Ok(i) => i,
            Err(e) => {
                log_error(&state, job_id, format!("{name}: failed to decode: {e}"));
                continue;
            }
        };

        // Run pipeline (blocking; use spawn_blocking in a real high-throughput server)
        let config_clone = config.clone();
        let result = tokio::task::spawn_blocking(move || run_in_memory(&img, &config_clone)).await;

        let enhanced = match result {
            Ok(Ok(e)) => e,
            Ok(Err(e)) => {
                log_error(&state, job_id, format!("{name}: pipeline error: {e}"));
                continue;
            }
            Err(e) => {
                log_error(&state, job_id, format!("{name}: task panic: {e}"));
                continue;
            }
        };

        // Encode to PNG bytes
        let mut buf = Cursor::new(Vec::new());
        let encode_result = {
            let encoder = image::codecs::png::PngEncoder::new(&mut buf);
            use image::ImageEncoder;
            let img_ref = &enhanced.image;
            match img_ref {
                image::DynamicImage::ImageLuma8(g) => {
                    encoder.write_image(g.as_raw(), g.width(), g.height(), image::ExtendedColorType::L8)
                }
                other => {
                    let rgb = other.to_rgb8();
                    encoder.write_image(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
                }
            }
        };

        if let Err(e) = encode_result {
            log_error(&state, job_id, format!("{name}: encode error: {e}"));
            continue;
        }

        // Determine next index and filename
        let (next_index, filename) = {
            let meta = read_meta(&state, corpus_id);
            let idx = meta.map(|m| m.images.len() as u32 + 1).unwrap_or(1);
            (idx, format!("{idx:04}.png"))
        };

        // Write image to disk
        let out_path = images_dir.join(&filename);
        if let Err(e) = std::fs::write(&out_path, buf.into_inner()) {
            log_error(&state, job_id, format!("{name}: write error: {e}"));
            continue;
        }

        // Update meta.json
        let image_meta = ImageMeta::new(next_index, name.clone(), filename);
        if let Err(e) = append_image_meta(&state, corpus_id, image_meta) {
            log_error(&state, job_id, format!("{name}: meta update error: {e}"));
            // Don't skip incrementing done — image was written
        }

        // Increment done counter
        if let Some(mut job) = state.jobs.get_mut(&job_id) {
            job.done += 1;
        }
    }

    // Mark job as done
    if let Some(mut job) = state.jobs.get_mut(&job_id) {
        job.status = JobStatus::Done;
    }
}

fn log_error(state: &AppState, job_id: Uuid, msg: String) {
    log::error!("{msg}");
    if let Some(mut job) = state.jobs.get_mut(&job_id) {
        job.errors.push(msg);
        job.done += 1;
    }
}

// ---------------------------------------------------------------------------
// Meta helpers
// ---------------------------------------------------------------------------

pub fn read_meta(state: &AppState, corpus_id: Uuid) -> Option<crate::models::Corpus> {
    let path = state.meta_path(corpus_id);
    let data = std::fs::read(&path).ok()?;
    serde_json::from_slice(&data).ok()
}

pub fn write_meta(state: &AppState, corpus: &crate::models::Corpus) -> anyhow::Result<()> {
    let path = state.meta_path(corpus.id);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let data = serde_json::to_vec_pretty(corpus)?;
    std::fs::write(path, data)?;
    Ok(())
}

fn append_image_meta(
    state: &AppState,
    corpus_id: Uuid,
    image: ImageMeta,
) -> anyhow::Result<()> {
    let mut corpus = read_meta(state, corpus_id)
        .ok_or_else(|| anyhow::anyhow!("corpus meta not found"))?;
    corpus.images.push(image);
    write_meta(state, &corpus)
}

/// Collect images from a zip file into PendingImages.
pub fn collect_from_zip(data: &[u8]) -> anyhow::Result<Vec<PendingImage>> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if !is_image_name(&name) {
            continue;
        }
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        out.push(PendingImage { original_name: name, data: buf });
    }
    Ok(out)
}

fn is_image_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ["jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}
