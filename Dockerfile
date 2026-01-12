# Dockerfile for Claude Agent Rust SDK

# Multi-stage build for optimized image size
FROM rust:1.83-slim as builder

# Set working directory
WORKDIR /usr/src/claude-agent-rust

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY . .

# Build workspace in release mode
RUN cargo build --workspace --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s -u 9999 claude

# Set working directory
WORKDIR /home/claude

# Copy built artifacts from builder
COPY --from=builder /usr/src/claude-agent-rust/target/release /home/claude/

# Set environment variables
ENV RUST_LOG=info
ENV PATH=/home/claude:$PATH

# Switch to non-root user
USER claude

# Default command
CMD ["/home/claude/claude-agent-core"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD ["/home/claude/claude-agent-core", "--version"] || exit 1

# Metadata
LABEL description="Claude Agent Rust SDK"
LABEL version="0.1.0"
