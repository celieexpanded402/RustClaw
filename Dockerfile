FROM rust:1.77-slim AS builder
WORKDIR /app

# 安裝 musl 工具
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

# 先複製 Cargo.toml 做依賴緩存
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/main.rs

# 再複製真正的源碼編譯
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

# 最終 image 用 scratch（零 overhead）
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rustclaw /rustclaw
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
EXPOSE 18789
CMD ["/rustclaw", "gateway"]
