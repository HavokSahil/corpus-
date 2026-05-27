//! Shared application state.

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::auth::{AuthConfig, SessionStore};
use crate::models::JobState;

#[derive(Clone)]
pub struct AppState {
    /// Root directory for all corpus data.
    pub data_dir: PathBuf,
    /// In-memory job registry.
    pub jobs: Arc<DashMap<Uuid, JobState>>,
    /// Authentication configuration (password hash, TTL).
    pub auth_config: AuthConfig,
    /// Active session tokens.
    pub sessions: SessionStore,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).expect("cannot create data directory");
        Self {
            data_dir,
            jobs: Arc::new(DashMap::new()),
            auth_config: AuthConfig::from_env(),
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Returns the directory for a specific corpus.
    pub fn corpus_dir(&self, corpus_id: Uuid) -> PathBuf {
        self.data_dir.join(corpus_id.to_string())
    }

    /// Returns the images subdirectory for a specific corpus.
    pub fn images_dir(&self, corpus_id: Uuid) -> PathBuf {
        self.corpus_dir(corpus_id).join("images")
    }

    /// Returns the path to the corpus metadata file.
    pub fn meta_path(&self, corpus_id: Uuid) -> PathBuf {
        self.corpus_dir(corpus_id).join("meta.json")
    }
}

