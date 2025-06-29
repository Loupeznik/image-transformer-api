FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y clang libwebp-dev pkg-config

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM debian:12.11-slim

RUN apt-get update && apt-get install -y libwebp7 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/image_transformer_api /usr/local/bin/image_transformer_api

EXPOSE 3000

CMD ["image_transformer_api"]
