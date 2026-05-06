//! Corpus CRUD route handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::jobs::{read_meta, write_meta};
use crate::models::{Corpus, CreateCorpusRequest, RenameCorpusRequest};
use crate::state::AppState;

/// GET /api/corpora — list all corpora.
pub async fn list_corpora(State(state): State<AppState>) -> impl IntoResponse {
    let mut corpora: Vec<Corpus> = Vec::new();
    let Ok(entries) = std::fs::read_dir(&state.data_dir) else {
        return Json(corpora);
    };
    for entry in entries.flatten() {
        let meta_path = entry.path().join("meta.json");
        if let Ok(data) = std::fs::read(&meta_path) {
            if let Ok(corpus) = serde_json::from_slice::<Corpus>(&data) {
                corpora.push(corpus);
            }
        }
    }
    corpora.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(corpora)
}

/// POST /api/corpora — create a new corpus.
pub async fn create_corpus(
    State(state): State<AppState>,
    Json(req): Json<CreateCorpusRequest>,
) -> Result<(StatusCode, Json<Corpus>), (StatusCode, String)> {
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty".into()));
    }
    let corpus = Corpus::new(req.name.trim().to_string());
    let dir = state.corpus_dir(corpus.id);
    std::fs::create_dir_all(dir.join("images"))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    write_meta(&state, &corpus)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(corpus)))
}

/// GET /api/corpora/:id — get corpus metadata.
pub async fn get_corpus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Corpus>, (StatusCode, String)> {
    read_meta(&state, id)
        .map(Json)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))
}

/// PATCH /api/corpora/:id — rename corpus.
pub async fn rename_corpus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RenameCorpusRequest>,
) -> Result<Json<Corpus>, (StatusCode, String)> {
    let mut corpus = read_meta(&state, id)
        .ok_or((StatusCode::NOT_FOUND, "corpus not found".into()))?;
    corpus.name = req.name.trim().to_string();
    write_meta(&state, &corpus)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(corpus))
}

/// DELETE /api/corpora/:id — delete corpus + all images.
pub async fn delete_corpus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let dir = state.corpus_dir(id);
    if !dir.exists() {
        return Err((StatusCode::NOT_FOUND, "corpus not found".into()));
    }
    std::fs::remove_dir_all(&dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
