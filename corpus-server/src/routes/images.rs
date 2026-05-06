//! Image upload, serve, reorder, delete, and export handlers.

use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use axum::{
    body::Body,
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;
use printpdf::*;

use crate::jobs::{collect_from_zip, read_meta, spawn_processing_job, write_meta, PendingImage};
use crate::models::{JobState, ReorderImageRequest, UploadResponse};
use crate::state::AppState;

fn is_image_name(name: &str) -> bool {
    name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg")
}

/// POST /api/corpora/:id/images — accept multipart upload, queue processing job.
pub async fn upload_images(
    State(state): State<AppState>,
    AxumPath(corpus_id): AxumPath<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), (StatusCode, String)> {
    // Ensure corpus exists
    if read_meta(&state, corpus_id).is_none() {
        return Err((StatusCode::NOT_FOUND, "corpus not found".into()));
    }

    // Collect all uploaded files and config
    let mut pending: Vec<PendingImage> = Vec::new();
    let mut config = doc_scanner::config::PipelineConfig::default();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "config" {
            if let Ok(data) = field.bytes().await {
                if let Ok(c) = serde_json::from_slice::<doc_scanner::config::PipelineConfig>(&data) {
                    config = c;
                }
            }
            continue;
        }

        let filename = field
            .file_name()
            .unwrap_or("upload")
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
            .to_vec();

        let lower = filename.to_ascii_lowercase();
        if lower.ends_with(".zip") {
            // Expand zip
            match collect_from_zip(&data) {
                Ok(imgs) => pending.extend(imgs),
                Err(e) => {
                    return Err((StatusCode::BAD_REQUEST, format!("bad zip: {e}")));
                }
            }
        } else if is_image_name(&lower) {
            pending.push(PendingImage {
                original_name: filename,
                data,
            });
        }
    }

    if pending.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "no valid images found in upload".into()));
    }

    let total = pending.len();
    let job_id = Uuid::new_v4();

    // Register job in state
    state.jobs.insert(job_id, JobState::new(job_id, corpus_id, total));

    // Spawn background processing
    spawn_processing_job(state, job_id, corpus_id, pending, config);

    Ok((StatusCode::ACCEPTED, Json(UploadResponse { job_id, total })))
}

/// GET /api/jobs/:job_id — poll job status.
pub async fn get_job(
    State(state): State<AppState>,
    AxumPath(job_id): AxumPath<Uuid>,
) -> Result<Json<crate::models::JobState>, (StatusCode, String)> {
    state
        .jobs
        .get(&job_id)
        .map(|j| Json(j.clone()))
        .ok_or((StatusCode::NOT_FOUND, "job not found".into()))
}

/// GET /api/corpora/:id/images/:img_id — serve a processed image.
pub async fn serve_image(
    State(state): State<AppState>,
    AxumPath((corpus_id, img_id)): AxumPath<(Uuid, Uuid)>,
) -> Result<Response, (StatusCode, String)> {
    let corpus = read_meta(&state, corpus_id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;
    let img = corpus
        .images
        .iter()
        .find(|i| i.id == img_id)
        .ok_or((StatusCode::NOT_FOUND, "image not found".into()))?;
    let path = state.images_dir(corpus_id).join(&img.filename);
    let data = std::fs::read(&path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        [(header::CONTENT_TYPE, "image/png")],
        Body::from(data),
    ).into_response())
}

/// PATCH /api/corpora/:id/images/:img_id — reorder image (set index).
pub async fn reorder_image(
    State(state): State<AppState>,
    AxumPath((corpus_id, img_id)): AxumPath<(Uuid, Uuid)>,
    Json(req): Json<ReorderImageRequest>,
) -> Result<Json<crate::models::Corpus>, (StatusCode, String)> {
    let mut corpus = read_meta(&state, corpus_id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;

    // Find and update the target image index
    let img = corpus
        .images
        .iter_mut()
        .find(|i| i.id == img_id)
        .ok_or((StatusCode::NOT_FOUND, "image not found".into()))?;
    img.index = req.index;

    // Re-sort images by index and re-assign sequential indices
    corpus.images.sort_by_key(|i| i.index);
    for (pos, img) in corpus.images.iter_mut().enumerate() {
        img.index = pos as u32 + 1;
    }

    write_meta(&state, &corpus)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(corpus))
}

/// DELETE /api/corpora/:id/images/:img_id — delete an image.
pub async fn delete_image(
    State(state): State<AppState>,
    AxumPath((corpus_id, img_id)): AxumPath<(Uuid, Uuid)>,
) -> Result<Json<crate::models::Corpus>, (StatusCode, String)> {
    let mut corpus = read_meta(&state, corpus_id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;

    let pos = corpus
        .images
        .iter()
        .position(|i| i.id == img_id)
        .ok_or((StatusCode::NOT_FOUND, "image not found".into()))?;

    let removed = corpus.images.remove(pos);

    // Delete file from disk
    let file_path = state.images_dir(corpus_id).join(&removed.filename);
    let _ = std::fs::remove_file(&file_path);

    // Re-index remaining images
    for (idx, img) in corpus.images.iter_mut().enumerate() {
        img.index = idx as u32 + 1;
    }

    write_meta(&state, &corpus)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(corpus))
}

/// GET /api/corpora/:id/export — download corpus as zip archive.
pub async fn export_corpus(
    State(state): State<AppState>,
    AxumPath(corpus_id): AxumPath<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    let corpus = read_meta(&state, corpus_id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;

    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    // Sort by index before zipping
    let mut images = corpus.images.clone();
    images.sort_by_key(|i| i.index);

    for img in &images {
        let path = state.images_dir(corpus_id).join(&img.filename);
        let data = std::fs::read(&path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let zip_name = format!("{:04}_{}", img.index, img.original_name.replace('/', "_"));
        zip.start_file(&zip_name, options)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        zip.write_all(&data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let buf = zip
        .finish()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let zip_name = format!("{}.zip", corpus.name.replace(' ', "_"));
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{zip_name}\""),
            ),
        ],
        Body::from(buf.into_inner()),
    ).into_response())
}

pub async fn export_corpus_pdf(
    State(state): State<AppState>,
    AxumPath(corpus_id): AxumPath<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    let corpus = read_meta(&state, corpus_id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;

    let mut images = corpus.images.clone();
    images.sort_by_key(|i| i.index);

    if images.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "corpus is empty".into()));
    }

    let mut doc = PdfDocument::new(&corpus.name);
    let mut pages = Vec::new();
    let mut warnings = Vec::new();

    for img_meta in images.iter() {
        let path = state.images_dir(corpus_id).join(&img_meta.filename);
        let data = std::fs::read(&path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Use `image` crate to get dimensions
        let dynamic_image = ::image::load_from_memory(&data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let img_w = dynamic_image.width() as f32;
        let img_h = dynamic_image.height() as f32;

        let raw_img = RawImage::decode_from_bytes(&data, &mut warnings)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let image_id = doc.add_image(&raw_img);

        let target_w_mm = 210.0_f32;
        let target_h_mm = 297.0_f32;

        let ratio_w = target_w_mm / img_w;
        let ratio_h = target_h_mm / img_h;
        let scale = ratio_w.min(ratio_h);

        let final_w_mm = img_w * scale;
        let final_h_mm = img_h * scale;

        let translate_x = (target_w_mm - final_w_mm) / 2.0;
        let translate_y = (target_h_mm - final_h_mm) / 2.0;

        let dpi = img_w / final_w_mm * 25.4;

        let transform = XObjectTransform {
            translate_x: Some(Pt(translate_x * 2.83465)),
            translate_y: Some(Pt(translate_y * 2.83465)),
            scale_x: None,
            scale_y: None,
            rotate: None,
            dpi: Some(dpi as f32),
        };

        let ops = vec![Op::UseXobject {
            id: image_id,
            transform,
        }];

        pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));
    }

    let bytes: Vec<u8> = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    
    let pdf_name = format!("{}.pdf", corpus.name.replace(' ', "_"));
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{pdf_name}\""),
            ),
        ],
        Body::from(bytes),
    ).into_response())
}
