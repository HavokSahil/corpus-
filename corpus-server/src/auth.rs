//! Authentication module — password-based login with opaque session tokens.
//!
//! The password is read from the `CORPUS_PASSWORD` environment variable.
//! Session tokens are random 32-byte hex strings stored in a `DashMap`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use dashmap::DashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

// ── Configuration ───────────────────────────────────────────────

/// Auth configuration built from environment variables at startup.
#[derive(Clone)]
pub struct AuthConfig {
    /// Argon2 hash of the configured password.
    pub password_hash: String,
    /// How long a session remains valid.
    pub session_ttl: Duration,
}

impl AuthConfig {
    /// Build from environment variables.
    /// Panics if `CORPUS_PASSWORD` is not set.
    pub fn from_env() -> Self {
        let raw_password = std::env::var("CORPUS_PASSWORD")
            .unwrap_or_else(|_| "changeme".to_string());

        let ttl_secs: u64 = std::env::var("CORPUS_SESSION_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86400);

        // Hash the password at startup so we never compare plain-text.
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .expect("failed to hash password")
            .to_string();

        log::info!(
            "auth: password configured (hash generated), session TTL = {}s",
            ttl_secs
        );

        Self {
            password_hash: hash,
            session_ttl: Duration::from_secs(ttl_secs),
        }
    }
}

// ── Session Store ───────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Session {
    pub created_at: Instant,
    pub expires_at: Instant,
}

/// Thread-safe session store: token → Session.
pub type SessionStore = Arc<DashMap<String, Session>>;

/// Generate a cryptographically random 32-byte hex token.
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}

// ── Request / Response types ────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Serialize)]
pub struct CheckResponse {
    pub authenticated: bool,
    pub remaining_secs: u64,
}

// ── Handlers ────────────────────────────────────────────────────

/// `POST /api/auth/login` — validate password, return session token.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let parsed_hash =
        PasswordHash::new(&state.auth_config.password_hash).map_err(|_| {
            log::error!("auth: failed to parse stored password hash");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            log::warn!("auth: failed login attempt");
            StatusCode::UNAUTHORIZED
        })?;

    let token = generate_token();
    let now = Instant::now();
    let session = Session {
        created_at: now,
        expires_at: now + state.auth_config.session_ttl,
    };

    state.sessions.insert(token.clone(), session);
    log::info!("auth: login successful, token issued");

    Ok(Json(LoginResponse {
        token,
        expires_in: state.auth_config.session_ttl.as_secs(),
    }))
}

/// `POST /api/auth/logout` — revoke session token.
pub async fn logout(
    State(state): State<AppState>,
    req: Request<Body>,
) -> StatusCode {
    if let Some(token) = extract_bearer_token(&req) {
        state.sessions.remove(&token);
        log::info!("auth: session revoked");
    }
    StatusCode::OK
}

/// `GET /api/auth/check` — check if the current token is still valid.
pub async fn check(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Json<CheckResponse>, StatusCode> {
    let token = extract_bearer_token(&req).ok_or(StatusCode::UNAUTHORIZED)?;

    let session = state
        .sessions
        .get(&token)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Instant::now();
    if now >= session.expires_at {
        drop(session);
        state.sessions.remove(&token);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let remaining = session.expires_at.duration_since(now).as_secs();
    Ok(Json(CheckResponse {
        authenticated: true,
        remaining_secs: remaining,
    }))
}

// ── Middleware ───────────────────────────────────────────────────

/// Axum middleware that enforces authentication on all routes except
/// `/api/auth/login`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Allow unauthenticated access to the login endpoint.
    if path == "/api/auth/login" {
        return next.run(req).await;
    }

    let token = match extract_bearer_token(&req) {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "missing authorization token")
                .into_response();
        }
    };

    let session = match state.sessions.get(&token) {
        Some(s) => s,
        None => {
            return (StatusCode::UNAUTHORIZED, "invalid or expired token")
                .into_response();
        }
    };

    if Instant::now() >= session.expires_at {
        drop(session);
        state.sessions.remove(&token);
        return (StatusCode::UNAUTHORIZED, "session expired").into_response();
    }

    // Session is valid — proceed.
    drop(session);
    next.run(req).await
}

// ── Helpers ─────────────────────────────────────────────────────

/// Extract a bearer token from the `Authorization` header,
/// or from a `?token=` query parameter (for browser-initiated requests
/// like `<img src>` or download links).
fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    // Try Authorization header first.
    if let Some(val) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Fall back to ?token= query parameter.
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(token) = pair.strip_prefix("token=") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}
