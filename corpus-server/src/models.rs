//! Shared data models for corpus-server.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub images: Vec<ImageMeta>,
}

impl Corpus {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            images: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ImageMeta
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMeta {
    pub id: Uuid,
    /// 1-based display index (determines sort order)
    pub index: u32,
    /// Original file name before processing
    pub original_name: String,
    /// Stored file name on disk (under corpus dir/images/)
    pub filename: String,
}

impl ImageMeta {
    pub fn new(index: u32, original_name: String, filename: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            index,
            original_name,
            filename,
        }
    }
}

// ---------------------------------------------------------------------------
// Job system
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub id: Uuid,
    pub corpus_id: Uuid,
    pub status: JobStatus,
    pub total: usize,
    pub done: usize,
    pub errors: Vec<String>,
}

impl JobState {
    pub fn new(id: Uuid, corpus_id: Uuid, total: usize) -> Self {
        Self {
            id,
            corpus_id,
            status: JobStatus::Pending,
            total,
            done: 0,
            errors: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Request/response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCorpusRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameCorpusRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ReorderImageRequest {
    pub index: u32,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub job_id: Uuid,
    pub total: usize,
}
