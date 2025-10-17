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

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/openapi-common ./crates/openapi-common
COPY crates/openapi-k8s-operator ./crates/openapi-k8s-operator
COPY crates/openapi-doc-server ./crates/openapi-doc-server

# Build the server
RUN cd crates/openapi-doc-server && cargo build --release

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
