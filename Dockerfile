# Build stage
FROM rust:1.95-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY database ./database

RUN cargo build --release

# Run stage
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/threev_twitch_chatbot /usr/local/bin/threev_twitch_chatbot

CMD ["threev_twitch_chatbot"]
