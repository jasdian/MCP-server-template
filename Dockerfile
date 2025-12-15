FROM rust:1.91-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.toml
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM cgr.dev/chainguard/static:latest
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mcp-server mcp-server
# 65532:65532
USER nonroot 
EXPOSE ${HTTP_PORT} ${HTTPS_PORT}

ENTRYPOINT ["/app/mcp-server"]