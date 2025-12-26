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

# Build the MatMul solver
# When using --platform linux/riscv64, Docker buildx handles architecture automatically
RUN cargo build --release --bin matmul-solver

# Runtime stage
FROM ubuntu:22.04

# Install minimal runtime dependencies
# Clear apt cache first to avoid space issues
RUN rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* && \
    apt-get -o Acquire::AllowInsecureRepositories=true \
    -o Acquire::AllowDowngradeToInsecureRepositories=true \
    update && \
    apt-get install -y --allow-unauthenticated --no-install-recommends \
    ca-certificates \
    jq \
    python3 \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* /tmp/* /var/tmp/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/matmul-solver /usr/local/bin/matmul-solver

# Copy benchmark script (optional, for benchmarking in container)
COPY benchmark.sh /app/benchmark.sh
RUN chmod +x /app/benchmark.sh

# Set entrypoint
ENTRYPOINT ["matmul-solver"]
