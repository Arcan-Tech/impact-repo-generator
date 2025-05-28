FROM rust:1.83-bullseye AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {println!("hello");}' > src/main.rs
RUN cargo build --release && rm -r src

# Copy actual source and build
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/repo-generator /app/repo-generator

ENTRYPOINT ["/app/repo-generator"]
