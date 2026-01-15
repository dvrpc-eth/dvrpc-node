# Build stage
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/dvrpc-node /usr/local/bin/

# Copy default config
COPY config.example.toml /app/config.example.toml

# Create data directory
RUN mkdir -p /app/data

# Expose RPC port
EXPOSE 8545

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8545/health || exit 1

ENTRYPOINT ["dvrpc-node"]
CMD ["--config", "/app/config.toml"]
