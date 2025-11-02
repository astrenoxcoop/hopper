# syntax=docker/dockerfile:1.4
FROM rust:1.90-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    g++ \
    && rm -rf /var/lib/apt/lists/*

ARG GIT_HASH

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY static ./static
COPY templates ./templates

RUN cargo build --release --bin hopper

FROM gcr.io/distroless/cc-debian12

LABEL org.opencontainers.image.title="hopper"
LABEL org.opencontainers.image.licenses="MIT"

WORKDIR /app

COPY --from=builder /app/target/release/hopper /app/hopper

COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

ENV HTTP_PORT=8080 \
    HTTP_STATIC_PATH=/app/static \
    RUST_LOG=hopper=info,warning \
    RUST_BACKTRACE=1

# Expose default port
EXPOSE 8080

# Run the application
ENTRYPOINT ["/app/hopper"]