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
COPY crates/ ./crates/
# Copy templates directory (needed for Askama)
COPY crates/openapi-doc-server/templates/ ./crates/openapi-doc-server/templates/

# Build arguments for features
ARG FEATURES=scalar,redoc

# Build the server
RUN cargo build --release --package openapi-doc-server --features ${FEATURES}

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
