# MatMul Solver - Amadeus Genesis Hack

Simple MatMul benchmark solver for the Hard Hack competition.

## Alignment with uPoW

This project is aligned with the **Hard Hack** requirements:
- ✅ **Uses real MatMul workloads** - Aligned with the live uPoW pipeline on Amadeus mainnet
- ✅ **Benchmarks reflect actual compute** - Same MatMul operations that miners run on mainnet today (not simulations)
- ✅ **Multiple precisions** - Supports fp32, fp16, int8 (matching uPoW compute requirements)
- ✅ **Performance metrics** - Latency, throughput, ops/sec for benchmarking
- ✅ **RISC-V platform** - Built for the target benchmarking platform

### Hard Hack vs. Full Validator PoW

**What this project does (Hard Hack):**
- Takes matrices as JSON input (plain format for benchmarking)
- Computes MatMul efficiently
- Outputs performance metrics and correctness hashes
- Focuses on **benchmarking the compute workload**

**What validators do (Full PoW protocol):**
- Build seed from epoch, segment_vr_hash, node_pk, node_pop, solver_pk, nonce
- Use Blake3 XOF to generate matrices from seed
- Compute MatMul (same operation we benchmark)
- Hash solution and check difficulty (leading zeros)
- Broadcast solution to network

**Key difference:** The Hard Hack benchmarks the **MatMul computation itself** (the core compute workload), not the full PoW protocol. The matrices are provided directly as JSON input rather than generated via Blake3 XOF, which allows for controlled benchmarking of the compute performance.

## What It Does

Solves benchmark workloads for the Hard Hack competition:

**Currently Supported:**
- ✅ **Matrix Multiplication (MatMul)** - All precisions (fp32, fp16, int8)

**Ready for Future Workloads** (when schemas are provided):
- Convolution kernels
- Attention-style workloads  
- Small model inference microbenchmarks

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
```

### Docker Build & Run

```bash
# Build
docker build -t matmul-solver .

# Run
docker run --rm \
  -v $(pwd)/input.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json \
  --output /app/output/output.json
```

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

## Features

- ✅ Matrix Multiplication workload (MatMul)
- ✅ Multiple precision support (fp32, fp16, int8)
- ✅ Optimized MatMul implementation
- ✅ Performance metrics (latency, throughput, ops/sec)
- ✅ Correctness verification (SHA-256 hash)
- ✅ Docker container for RISC-V
- ✅ JSON I/O format
- ✅ Extensible architecture for future workloads (convolution, attention, inference)

## Docker Details

- **Base Image**: `ghcr.io/tenstorrent/tt-xla/tt-xla-ird-ubuntu-22-04:latest` (RISC-V)
- **Binary**: `matmul-solver`
- **Entrypoint**: Takes `--input` and `--output` arguments

## Building and Pushing to Docker Hub

### Option 1: Build on Local Machine (Cross-compilation)

```bash
# Login to Docker Hub
docker login

# Build (replace 'yourusername' with your Docker Hub username)
docker build -t yourusername/matmul-solver:latest .

# Push
docker push yourusername/matmul-solver:latest
```

### Option 2: Build on RISC-V Cloud (Recommended)

Building directly on a RISC-V instance is more efficient:

```bash
# SSH into RISC-V cloud instance
ssh user@your-riscv-instance

# Clone repo
git clone YOUR_REPO_URL
cd YOUR_REPO

# Build
docker build -t matmul-solver .

# Tag and push to Docker Hub
docker login
docker tag matmul-solver yourusername/matmul-solver:latest
docker push yourusername/matmul-solver:latest
```

**Note**: Since the image targets RISC-V, building natively on RISC-V hardware ensures proper compatibility and faster builds.

## Submission Requirements

This implementation includes:
- ✅ Raw metrics (latency, throughput, ops/sec)
- ✅ Correctness proof (result hash)
- ✅ Docker container for reproducibility
- ✅ Source code
- ✅ Benchmark metadata

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

### Correctness Verification

The project includes comprehensive tests for:
- ✅ **MatMul correctness** - Verifies results match expected values
- ✅ **Hash consistency** - Same input always produces same hash
- ✅ **Precision accuracy** - Tests for fp32, fp16, int8
- ✅ **Integration tests** - End-to-end workflow verification
- ✅ **Error handling** - Invalid matrix dimensions

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

Latency measurements can vary due to:
- **System load** - Other processes using CPU/memory
- **CPU scheduling** - OS task switching
- **Cache effects** - First run vs. warm cache
- **CPU frequency scaling** - Dynamic clock speed
- **Memory allocation** - Heap fragmentation

For consistent benchmarking:
- Run multiple iterations and take average/median
- Use `--release` builds (optimized)
- Minimize system load
- Consider using `taskset` to pin CPU affinity

### Manual Testing

```bash
# Test with sample input
cargo run --release --bin matmul-solver -- --input input.json --output output.json

# Check output
cat output.json

# Run comprehensive correctness test script
./test_correctness.sh

# Verify hash consistency (run multiple times, hash should be identical)
for i in {1..5}; do
  cargo run --release --bin matmul-solver -- --input input.json --output output_$i.json
  echo "Run $i hash: $(jq -r '.result_hash' output_$i.json)"
done
```

## Project Structure

```
.
├── Dockerfile          # RISC-V Docker build
├── Cargo.toml         # Rust dependencies
├── src/
│   ├── main.rs        # CLI entry point
│   └── lib.rs         # MatMul implementation
└── README.md          # This file
```

That's it. Simple benchmark solver ready for submission.
