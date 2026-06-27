# Multi-stage Docker build for rust-assistant
# Produces a minimal image with the binary + models
#
# Build:    docker build -t rust-assistant .
# Run:      docker run --rm -it \
#             --device /dev/snd \
#             -e OLLAMA_SERVER=host.docker.internal:11434 \
#             rust-assistant
#
# For GPU acceleration (NVIDIA):
#   docker run --rm -it --gpus all \
#     --device /dev/snd \
#     rust-assistant

# ── Stage 1: Build ──
FROM rust:1.84-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libasound2-dev libssl-dev cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./

# Build dependencies first (for Docker layer caching)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Build the actual binary
COPY . .
RUN cargo build --release

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies: ALSA for audio, ca-certificates for HTTPS
RUN apt-get update && apt-get install -y \
    libasound2 libpulse0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and config
COPY --from=builder /app/target/release/rust-assistant /usr/local/bin/rust-assistant

# Create default directories
RUN mkdir -p /app/models /app/records
WORKDIR /app

# Default command
ENTRYPOINT ["rust-assistant"]
CMD ["run"]

# Metadata
LABEL org.opencontainers.image.title="rust-assistant"
LABEL org.opencontainers.image.description="Local-first voice assistant — 100% Rust, zero Python"
LABEL org.opencontainers.image.licenses="MIT"
