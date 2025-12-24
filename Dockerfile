# Use TensTorrent RISC-V base image provided by hackathon organizers
FROM ghcr.io/tenstorrent/tt-xla/tt-xla-ird-ubuntu-22-04:latest AS builder

# Install build dependencies and Rust toolchain
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set up Rust environment
ENV PATH="/root/.cargo/bin:${PATH}"

# Set default Rust toolchain
RUN rustup default stable

# Set working directory
WORKDIR /app

# Copy everything
COPY . .

# Build the MatMul solver
RUN cargo build --release --bin matmul-solver

# Runtime stage - use the same base image
FROM ghcr.io/tenstorrent/tt-xla/tt-xla-ird-ubuntu-22-04:latest

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/matmul-solver /usr/local/bin/matmul-solver

# Set entrypoint
ENTRYPOINT ["matmul-solver"]
