FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# SQLx offline mode - uses .sqlx folder instead of live database
ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ai-merchant-backend /usr/local/bin/

EXPOSE 3000

CMD ["ai-merchant-backend"]
