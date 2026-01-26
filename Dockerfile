# Multi-stage build for minimal image size
# Stage 1: Build
FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --package ankit-mcp

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 ankit
USER ankit

# Copy binary from builder
COPY --from=builder /app/target/release/ankit-mcp /usr/local/bin/ankit-mcp

# Default to stdio transport for MCP
ENTRYPOINT ["ankit-mcp"]

# Default args (can be overridden)
CMD ["--host", "host.docker.internal"]
