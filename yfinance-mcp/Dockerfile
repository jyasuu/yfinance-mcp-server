FROM rust:1.96-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends musl-tools && rm -rf /var/lib/apt/lists/* && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo fetch --locked && rm -rf src

COPY src/ src/

RUN cargo build --locked --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache ca-certificates-bundle

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/yfinance-mcp /usr/local/bin/yfinance-mcp

ENV YFINANCE_CACHE_TTL=300
ENV YFINANCE_TIMEOUT=30
ENV YFINANCE_MAX_RETRIES=3

EXPOSE 8080

ENTRYPOINT ["yfinance-mcp"]
