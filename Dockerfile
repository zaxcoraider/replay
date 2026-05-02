# Multi-stage build for replay-api
# Stage 1: build the binary
FROM rust:1.79-slim AS builder

WORKDIR /app

# Cache dependencies before copying source
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/replay-core/Cargo.toml crates/replay-core/Cargo.toml
COPY crates/replay-api/Cargo.toml crates/replay-api/Cargo.toml
COPY crates/replay-cli/Cargo.toml crates/replay-cli/Cargo.toml
COPY crates/replay-sdk/Cargo.toml crates/replay-sdk/Cargo.toml

# Create stub lib.rs files so cargo can resolve deps without the full source
RUN mkdir -p crates/replay-core/src crates/replay-api/src \
             crates/replay-cli/src crates/replay-sdk/src && \
    echo "fn main() {}" > crates/replay-api/src/main.rs && \
    touch crates/replay-core/src/lib.rs \
          crates/replay-cli/src/main.rs \
          crates/replay-sdk/src/lib.rs

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo fetch

# Now copy real source and build
COPY crates/ crates/
RUN cargo build --release -p replay-api

# Stage 2: minimal runtime image
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/replay-api /usr/local/bin/replay-api

# Copy bundled IDLs — path is baked in at compile time as
# /app/crates/replay-core/assets/idls (matches CARGO_MANIFEST_DIR in builder)
COPY --from=builder /app/crates/replay-core/assets /app/crates/replay-core/assets

# Disk-cache dir for on-chain IDL fetches
RUN mkdir -p /app/.replay/idl-cache

ENV REPLAY_IDL_CACHE_DIR=/app/.replay/idl-cache
ENV RUST_LOG=replay_api=info,replay_core=info

EXPOSE 8080
CMD ["/usr/local/bin/replay-api"]
