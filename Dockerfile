FROM rust:1.85-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/va-link-server /usr/local/bin/
EXPOSE 8080
CMD ["va-link-server"]
