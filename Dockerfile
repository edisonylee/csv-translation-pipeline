# ---------- Stage 1: Build ----------
FROM rust:1.83-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace metadata
COPY Cargo.toml Cargo.lock ./

# Copy crates
COPY translator ./translator
COPY translator-server ./translator-server

# Build only the server binary
RUN cargo build --release -p translator-server

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/translator-server /usr/local/bin/translator-server

RUN useradd -m -u 1000 translator
USER translator

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["translator-server"]
