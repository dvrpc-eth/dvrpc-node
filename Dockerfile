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

# Create data directory
RUN mkdir -p /app/data

# Environment variables for configuration
# Required:
ENV DVRPC_EXECUTION_RPC=""
# Optional (with defaults):
ENV DVRPC_HOST="0.0.0.0"
ENV DVRPC_PORT="8545"
ENV DVRPC_NETWORK="mainnet"
ENV DVRPC_CONSENSUS_ENABLED="false"
ENV DVRPC_CONSENSUS_RPC=""
ENV DVRPC_LOG_LEVEL="info"

# Expose RPC port
EXPOSE 8545

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8545/health || exit 1

# Run without config file - uses environment variables
ENTRYPOINT ["dvrpc-node"]
CMD []
