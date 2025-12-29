# Use Ubuntu base image
# For local builds: defaults to host architecture (arm64/amd64)
# For RISC-V: use --platform linux/riscv64 with docker buildx
FROM ubuntu:22.04 AS builder

# Install build dependencies
# Clear apt cache first to avoid space issues
RUN rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* && \
    apt-get -o Acquire::AllowInsecureRepositories=true \
    -o Acquire::AllowDowngradeToInsecureRepositories=true \
    update && \
    apt-get install -y --allow-unauthenticated --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    ca-certificates \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* /tmp/* /var/tmp/*

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

# Build the MatMul solver and API server
# When using --platform linux/riscv64, Docker buildx handles architecture automatically
# Build with api feature
# Try with OpenBLAS first, fallback to no OpenBLAS if build fails (u8i8 doesn't need it)
RUN (cargo build --release --bin matmul-solver --features api --bin matmul-api 2>&1) || \
    (echo "OpenBLAS build failed, building without it..." && \
     cargo build --release --bin matmul-solver --features api --no-default-features --bin matmul-api)

# Runtime stage
FROM ubuntu:22.04

# Install minimal runtime dependencies + OpenBLAS
# Clear apt cache first to avoid space issues
RUN rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* && \
    apt-get -o Acquire::AllowInsecureRepositories=true \
    -o Acquire::AllowDowngradeToInsecureRepositories=true \
    update && \
    apt-get install -y --allow-unauthenticated --no-install-recommends \
    ca-certificates \
    jq \
    python3 \
    libopenblas0 \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* /tmp/* /var/tmp/*

WORKDIR /app

# Copy the binaries from builder
COPY --from=builder /app/target/release/matmul-solver /usr/local/bin/matmul-solver
COPY --from=builder /app/target/release/matmul-api /usr/local/bin/matmul-api

# Copy benchmark script (optional, for benchmarking in container)
COPY benchmark.sh /app/benchmark.sh
RUN chmod +x /app/benchmark.sh

# Copy keep-alive wrapper script for Koyeb service deployment
COPY keep-alive.sh /app/keep-alive.sh
RUN chmod +x /app/keep-alive.sh

# Copy default input file into the container (optional - seed generation doesn't require it)
# This ensures 'input.json' is available at /app/input.json if needed
# Note: Seed-based generation (--seed flag) doesn't require input files
COPY inputs/input_fp32.json /app/input.json

# Expose port 8000 for health checks
EXPOSE 8000

# Set entrypoint to wrapper script (keeps container alive for Koyeb service)
ENTRYPOINT ["/app/keep-alive.sh"]
