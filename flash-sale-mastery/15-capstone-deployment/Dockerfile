# ==============================================================================
# Stage 1: Builder
# ==============================================================================
FROM rust:1.82-bookworm AS builder

WORKDIR /app

# Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY flash-sale-mastery/15-capstone-deployment/Cargo.toml flash-sale-mastery/15-capstone-deployment/

# Create a dummy main.rs to cache dependencies
RUN mkdir -p flash-sale-mastery/15-capstone-deployment/src && \
    echo "fn main() {}" > flash-sale-mastery/15-capstone-deployment/src/main.rs

# Cache dependencies — this layer only rebuilds when Cargo.toml changes
RUN cargo build --release --package capstone-deployment 2>/dev/null || true

# Copy the actual source code
COPY flash-sale-mastery/15-capstone-deployment/src/ flash-sale-mastery/15-capstone-deployment/src/

# Touch main.rs to force a rebuild with real source
RUN touch flash-sale-mastery/15-capstone-deployment/src/main.rs && \
    cargo build --release --package capstone-deployment

# ==============================================================================
# Stage 2: Runtime
# ==============================================================================
FROM debian:bookworm-slim AS runtime

# Install minimal runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r flashsale && \
    useradd -r -g flashsale -d /app -s /sbin/nologin flashsale

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/capstone-deployment /app/flash-sale-api

# Set ownership
RUN chown -R flashsale:flashsale /app

# Switch to non-root user
USER flashsale

# Expose the API port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/flash-sale-api", "--health-check"]

# Run the API server
ENTRYPOINT ["/app/flash-sale-api"]
CMD ["api"]
