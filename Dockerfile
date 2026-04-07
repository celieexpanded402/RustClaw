# Build stage
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release && strip target/release/rustclaw

# Runtime stage
FROM scratch
COPY --from=builder /build/target/release/rustclaw /rustclaw
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/rustclaw", "gateway"]
