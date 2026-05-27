//! Corpus-server entry point.
//!
//! Usage:
//! ```bash
//! corpus-server --port 8080 --data-dir ./corpus-data
//! ```

mod auth;
mod jobs;
mod models;
mod routes;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};

use routes::{corpus as corpus_routes, images as image_routes};
use state::AppState;

#[derive(Parser, Debug)]
#[command(name = "corpus-server", about = "Corpus+ document corpus management server")]
struct Cli {
    /// Port to listen on
    #[arg(long, default_value_t = 8081)]
    port: u16,

    /// Directory to store corpus data
    #[arg(long, default_value = "./corpus-data")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp(None)
    .init();

    let cli = Cli::parse();
    let state = AppState::new(cli.data_dir.clone());

    log::info!("data directory: {}", cli.data_dir.display());
    log::info!("starting corpus-server on port {}", cli.port);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // ── Auth routes (login is unprotected, others go through middleware) ──
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/check", get(auth::check))
        // Corpus CRUD
        .route("/api/corpora", get(corpus_routes::list_corpora))
        .route("/api/corpora", post(corpus_routes::create_corpus))
        .route("/api/corpora/:id", get(corpus_routes::get_corpus))
        .route("/api/corpora/:id", patch(corpus_routes::rename_corpus))
        .route("/api/corpora/:id", delete(corpus_routes::delete_corpus))
        // Image operations
        .route("/api/corpora/:id/images", post(image_routes::upload_images))
        .route("/api/corpora/:id/images/:img_id", get(image_routes::serve_image))
        .route("/api/corpora/:id/images/:img_id", patch(image_routes::reorder_image))
        .route("/api/corpora/:id/images/:img_id", delete(image_routes::delete_image))
        // Export
        .route("/api/corpora/:id/export", get(image_routes::export_corpus))
        .route("/api/corpora/:id/export/pdf", get(image_routes::export_corpus_pdf))
        // Job polling
        .route("/api/jobs/:job_id", get(image_routes::get_job))
        // ── Auth middleware protects all routes above ──
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        .layer(DefaultBodyLimit::disable())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    log::info!("listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

