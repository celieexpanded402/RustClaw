FROM rust:1.58 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release

# final image stage
FROM scratch
COPY --from=builder /app/target/release/my_app /
ENTRYPOINT ["/my_app"]