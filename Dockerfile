FROM rust:trixie AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
COPY --from=builder /app/target/release/silicate /usr/local/bin/silicate
ENTRYPOINT ["silicate"]