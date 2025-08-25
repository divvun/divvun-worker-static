# Multi-stage build for divvun-worker-static
FROM rust:slim-trixie AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code and build the actual application
COPY src ./src
COPY languages.toml index.html ./
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy binary and required files from builder
COPY --from=builder /app/target/release/divvun-worker-static ./
COPY --from=builder /app/languages.toml ./
COPY --from=builder /app/index.html ./


# Expose the default port
EXPOSE 4000

# Run the server
CMD ["./divvun-worker-static", "serve", "--host", "0.0.0.0", "--port", "4000"]