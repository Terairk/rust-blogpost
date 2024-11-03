# Builder stage
FROM rust:1.82-slim as builder

WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Install SQLx CLI
RUN cargo install sqlx-cli --no-default-features --features postgres

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release && \
  rm -rf src

# Copy source code and other necessary files
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

# Build real application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install required runtime dependencies
RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  curl \
  build-essential \
  && rm -rf /var/lib/apt/lists/*

# Install Rust and SQLx
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install sqlx-cli --no-default-features --features postgres

# Copy application files
COPY --from=builder /usr/src/app/target/release/blogpost ./blogpost
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/migrations ./migrations

## Create uploads directory with proper permissions

EXPOSE 8080

ENV RUST_LOG=info
ENV PATH="/usr/local/bin:${PATH}"

# Create uploads directory with proper permissions
RUN mkdir -p /app/uploads && \
  chmod 777 /app/uploads

CMD ["./blogpost"]
