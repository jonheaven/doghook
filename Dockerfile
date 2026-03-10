# ─── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin doghook

# ─── Runtime stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/doghook /usr/local/bin/doghook
COPY doghook.toml /etc/doghook.toml

CMD ["doghook", "doginals", "service", "start", "--config-path", "/etc/doghook.toml"]
