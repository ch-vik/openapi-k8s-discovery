# Build stage
FROM rust:1.85-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/openapi-common/Cargo.toml ./crates/openapi-common/
COPY crates/openapi-doc-server/Cargo.toml ./crates/openapi-doc-server/

# Create dummy main.rs files to build dependencies
RUN mkdir -p crates/openapi-common/src crates/openapi-doc-server/src && \
    echo "fn main() {}" > crates/openapi-common/src/lib.rs && \
    echo "fn main() {}" > crates/openapi-doc-server/src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release --package openapi-doc-server

# Copy actual source code
COPY crates/openapi-common/src ./crates/openapi-common/src
COPY crates/openapi-doc-server/src ./crates/openapi-doc-server/src

# Build the actual binary
RUN cargo build --release --package openapi-doc-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/openapi-doc-server /usr/local/bin/openapi-doc-server

# Set the binary as the entrypoint
ENTRYPOINT ["/usr/local/bin/openapi-doc-server"]
