FROM rust:1.75-bookworm AS builder

WORKDIR /app

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() { println!(\"stub\"); }" > src/main.rs
RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust_pg_api /usr/local/bin/rust_pg_api

ENV RUST_LOG=info
EXPOSE 3000

CMD ["rust_pg_api"]
