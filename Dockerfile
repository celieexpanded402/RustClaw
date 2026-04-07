FROM rust:1.77-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rustclaw /usr/local/bin/rustclaw
EXPOSE 18789
CMD ["rustclaw", "gateway"]
