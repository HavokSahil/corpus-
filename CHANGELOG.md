# Changelog

All notable changes to Corpus+ will be documented in this file.

## [0.2.0] — 2026-05-27

### Added

- **Authentication wall** — Platform now requires a password to access. No more open access on the local network.
  - Password-based login with Argon2 hashing (password never stored or compared in plain text)
  - Opaque session tokens (32-byte random hex) with configurable TTL (default: 24 hours)
  - Axum middleware rejects all unauthenticated API requests (except `/api/auth/login`)
  - `?token=` query parameter fallback for browser-initiated loads (`<img>`, downloads)
- **Login page** — Glassmorphism UI with animated gradient orbs, shake-on-error, and loading spinner
- **Logout button** — Added to the sidebar footer with hover effects
- **Auth environment variables**:
  - `CORPUS_PASSWORD` — Set the access password (default: `changeme`)
  - `CORPUS_SESSION_TTL` — Session lifetime in seconds (default: `86400` / 24h)

### Changed

- **Backend port no longer published** — Removed `ports: 8081:8081` from `docker-compose.yml`. The backend is now only reachable via Docker's internal network through Nginx, closing the direct API bypass.
- **All API calls now carry auth tokens** — Frontend `fetch` calls include `Authorization: Bearer <token>` header; automatic token cleanup and redirect to login on `401`.

### Security

- Defence-in-depth: backend inaccessible from host network + token auth middleware
- Password hashed with Argon2 at server startup
- Sessions stored in-memory (`DashMap`) with automatic expiry
- Instant session revocation on logout

---

## [0.1.0] — 2026-05-11

### Added

- Initial release
- Fuzzy search for corpora and images (fuse.js)
- Management scripts (backup, restore, CLI wrapper)
- Document scanning pipeline with configurable image processing
- Reading mode and PDF export
- Docker Compose deployment
