# ── Stage 1: Build ───────────────────────────────────────────────
FROM rust:1.88-bookworm AS builder

# Install OpenCV + build deps (needed by doc-scanner → opencv crate)
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake pkg-config clang llvm-dev libclang-dev \
    libopencv-dev libopencv-imgproc-dev libopencv-imgcodecs-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the workspace (server + doc-scanner)
COPY doc-scanner/ /app/doc-scanner/
COPY corpus-server/ /app/corpus-server/

WORKDIR /app/corpus-server
RUN cargo build --release

# ── Stage 2: Runtime ─────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopencv-imgproc406 libopencv-imgcodecs406 libopencv-core406 libopencv-dnn406 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/corpus-server/target/release/corpus-server /app/corpus-server

# Persistent corpus data lives here
VOLUME /app/corpus-data

EXPOSE 8081
ENV RUST_LOG=info

ENTRYPOINT ["/app/corpus-server", "--port", "8081", "--data-dir", "/app/corpus-data"]
