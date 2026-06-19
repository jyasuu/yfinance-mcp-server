FROM rust:1.96-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo fetch --locked && rm -rf src

COPY src/ src/

RUN cargo build --locked --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/yfinance-mcp /usr/local/bin/yfinance-mcp

ENV YFINANCE_CACHE_TTL=300
ENV YFINANCE_TIMEOUT=30
ENV YFINANCE_MAX_RETRIES=3

EXPOSE 8080

ENTRYPOINT ["yfinance-mcp"]
