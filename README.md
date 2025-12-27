# MatMul Solver - Amadeus Genesis Hack

Simple MatMul benchmark solver for the Hard Hack competition.

## Alignment with uPoW

This project is aligned with the **Hard Hack** requirements:
- ✅ **Uses real MatMul workloads** - Aligned with the live uPoW pipeline on Amadeus mainnet
- ✅ **Benchmarks reflect actual compute** - Same MatMul operations that miners run on mainnet today (not simulations)
- ✅ **Multiple precisions** - Supports fp32, fp16, int8 (matching uPoW compute requirements)
- ✅ **Performance metrics** - Latency, throughput, ops/sec for benchmarking
- ✅ **RISC-V platform** - Built for the target benchmarking platform

### Hard Hack

**What this project does (Hard Hack):**
- Takes matrices as JSON input (plain format for benchmarking)
- Computes MatMul efficiently
- Outputs performance metrics and correctness hashes
- Focuses on **benchmarking the compute workload**


## What It Does

Solves benchmark workloads for the Hard Hack competition:

**Currently Supported:**
- ✅ **Matrix Multiplication (MatMul)** - All precisions (fp32 (base optimisation), fp16, int8)

Takes JSON input, computes the workload, and outputs:
- Result data
- Performance metrics (latency, throughput, ops/sec)
- Result hash (for correctness verification)

## Quick Start

### Local Build & Run

```bash
# Build
cargo build --release --bin matmul-solver

# Run
cargo run --release --bin matmul-solver -- --input input.json --output output.json


# Run comprehensive correctness test script
./test_correctness.sh
```

### OpenBLAS Acceleration

By default, fp32/fp16/int8 matmul use OpenBLAS via the `openblas` feature for faster kernel performance.  
If you want to disable it (fallback to the Rust implementation), build with:

```bash
cargo build --release --no-default-features
```

### Docker Build & Run

#### Build Docker Image

```bash
# Build for local architecture (ARM64 on Mac, x86_64 on Linux/Intel Mac)
docker build -t matmul-solver .

# Build for specific architecture (e.g., linux/amd64 for deployment)
docker buildx build --platform linux/amd64 -t matmul-solver .
```

#### Run Single Computation

```bash
# Run with input/output files mounted
docker run --rm \
  -v $(pwd)/input_fp32.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json --output /app/output/output.json

# Run with verification
docker run --rm \
  -v $(pwd)/input_fp32.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json --output /app/output/output.json --verify
```

#### Run Benchmarking (Multiple Iterations)

```bash
# Run benchmark with 50 iterations (default)
docker run --rm \
  -v $(pwd)/input_fp32.json:/app/input.json \
  -v $(pwd):/app/output \
  --entrypoint /bin/bash \
  matmul-solver \
  /app/benchmark.sh /app/input.json 50

# Run benchmark with 100 iterations (more stable)
docker run --rm \
  -v $(pwd)/input_fp32.json:/app/input.json \
  -v $(pwd):/app/output \
  --entrypoint /bin/bash \
  matmul-solver \
  /app/benchmark.sh /app/input.json 100
```

#### Test Different Precisions

```bash
# Test fp32
docker run --rm \
  -v $(pwd)/input_fp32.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json --output /app/output/output_fp32.json

# Test fp16
docker run --rm \
  -v $(pwd)/input_fp16.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json --output /app/output/output_fp16.json

# Test int8
docker run --rm \
  -v $(pwd)/input_int8.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json --output /app/output/output_int8.json
```

**Note:** The benchmark script automatically detects Docker environment and uses the compiled binary directly (no cargo needed).

**Troubleshooting:** If you encounter disk space errors during build, clean Docker:
```bash
docker system prune -a --volumes
```

## Deployment Launch Command

For deployment platforms (like Koyeb), use:

**With verification:**
```bash
matmul-solver --input /app/input.json --output /app/output.json --verify
```

**Without verification:**
```bash
matmul-solver --input /app/input.json --output /app/output.json
```

**Using defaults (if files are in working directory):**
```bash
matmul-solver
```

The Dockerfile sets `ENTRYPOINT ["matmul-solver"]`, so the container automatically runs the solver. You only need to specify the command if you want to override the entrypoint or pass additional arguments.

## Input Format

```json
{
  "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
  "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
  "precision": "fp32",
  "metadata": {
    "compiler_flags": "-O3",
    "libraries": [],
    "cache_enabled": true
  }
}
```

**Supported precisions:** `fp32`, `fp16`, `int8`

## Output Format

```json
{
  "result_matrix": [[19.0, 22.0], [43.0, 50.0]],
  "result_hash": "abc123...",
  "metrics": {
    "latency_ms": 0.1234,
    "throughput_ops_per_sec": 1234567.89,
    "ops_per_second": 1234567.89,
    "memory_usage_mb": 0.001
  },
  "metadata": {
    "precision": "fp32",
    "matrix_a_shape": [2, 2],
    "matrix_b_shape": [2, 2],
    "result_shape": [2, 2]
  }
}
```


## Testing

### Running Tests

```bash
# Run all correctness tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_matmul_fp32_correctness
```

**Verifying correctness programmatically:**

```rust
use matmul_solver::{verify_correctness, types};

// After computing a result, verify it:
let is_correct = verify_correctness(
    &matrix_a,
    &matrix_b,
    "fp32",
    &output.result_hash
)?;
```

**Why latency varies between runs:**

For consistent benchmarking:
- Run multiple iterations and take average/median
- Use `--release` builds (optimized)
- Minimize system load
- Consider using `taskset` to pin CPU affinity


## Tracking Optimizations

For iterative optimization during the hackathon, track your changes:

### Quick Benchmark

The `benchmark.sh` script automatically follows these rules:
```bash
# Default: 50 runs with statistics
./benchmark.sh input.json

# 100 runs for more stable results
./benchmark.sh input.json 100

# Compare median results between optimizations
jq '.metrics.latency.median' benchmark_results.json
```

## Project Structure

```
.
├── Dockerfile          # RISC-V Docker build
├── Cargo.toml         # Rust dependencies
├── benchmark.sh       # Benchmark runner script
├── OPTIMIZATIONS.md   # Optimization tracking log
├── benchmark_results.json  # Latest benchmark results
├── src/
│   ├── main.rs        # CLI entry point
│   └── lib.rs         # MatMul implementation
└── README.md          # This file
```



by Akanimoh Osutuk
