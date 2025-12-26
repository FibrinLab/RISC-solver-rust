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
docker system prune -a --volumes
```

```bash
# Build for local architecture (ARM64 on Mac, x86_64 on Linux/Intel Mac)
docker build -t matmul-solver .

# Or explicitly build for AMD64 (slower due to emulation, but more compatible)
docker build --platform linux/amd64 -t matmul-solver .

# Run single computation
docker run --rm \
  -v $(pwd)/input.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver \
  --input /app/input.json \
  --output /app/output/output.json

# Run benchmark with 100 iterations (more stable)
docker run --rm \
  -v $(pwd)/input.json:/app/input.json \
  -v $(pwd):/app/output \
  --entrypoint /bin/bash \
  matmul-solver \
  /app/benchmark.sh /app/input.json 100
```

**Note:** The benchmark script automatically detects Docker environment and uses the compiled binary directly (no cargo needed).

**Troubleshooting:** If you encounter disk space errors during build, clean Docker:
```bash
docker system prune -a --volumes
```

## Deployment Launch Command

For deployment platforms (Koyeb, Railway, etc.):

**Basic launch command:**
```
matmul-solver --input /app/input.json --output /app/output.json
```

**With verification:**
```
matmul-solver --input /app/input.json --output /app/output.json --verify
```

**Using defaults (if files are in working directory):**
```
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

- **Base Image**: `ubuntu:22.04` (supports RISC-V via buildx)
- **Binary**: `matmul-solver` (compiled for RISC-V)
- **Entrypoint**: Takes `--input` and `--output` arguments
- **Architecture**: Built for `linux/riscv64` (cross-compiled from host if needed)

## Building and Pushing to Docker Hub

### Push to `fibrinlab/risc-rust` (Multi-Architecture)

**Build for both amd64 and riscv64 architectures:**

```bash
# Login to Docker Hub
docker login

# Set up buildx for multi-arch (one-time setup)
docker buildx create --use --name multiarch-builder
docker buildx inspect --bootstrap

# Build for BOTH architectures and push
docker buildx build \
  --platform linux/amd64,linux/riscv64 \
  --tag fibrinlab/risc-rust:latest \
  --push \
  .

# Verify both architectures are supported
docker buildx imagetools inspect fibrinlab/risc-rust:latest
```

This creates a single image that supports both amd64 (for testing) and riscv64 (for deployment on RISC-V platforms).

**Important:** The image must be built for RISC-V architecture. Use `docker buildx` to build for the correct platform:

## Alternative
```bash
docker build -t fibrinlab/risc-rust:latest .
docker push fibrinlab/risc-rust:latest
```

```bash
# Login to Docker Hub
docker login

# Set up buildx (if not already done)
docker buildx create --use --name riscv-builder

# Build for RISC-V architecture and push in one step
docker buildx build \
  --platform linux/riscv64 \
  --tag fibrinlab/risc-rust:latest \
  --push \
  .

# Optional: Tag with version
docker buildx build \
  --platform linux/riscv64 \
  --tag fibrinlab/risc-rust:v1.0.0 \
  --push \
  .
```

**Build only for RISC-V (if deployment platform is RISC-V only):**

```bash
docker buildx build \
  --platform linux/riscv64 \
  --tag fibrinlab/risc-rust:latest \
  --push \
  .
```

### Option 1: Build on Local Machine (Multi-Architecture)

```bash
# Login to Docker Hub
docker login

# Set up buildx for multi-platform builds
docker buildx create --use --name multiarch-builder
docker buildx inspect --bootstrap

# Build for both amd64 and riscv64 (recommended)
docker buildx build \
  --platform linux/amd64,linux/riscv64 \
  --tag fibrinlab/risc-rust:latest \
  --push \
  .

# Or build only for RISC-V
docker buildx build \
  --platform linux/riscv64 \
  --tag fibrinlab/risc-rust:latest \
  --push \
  .
```

**Note:** `docker buildx` is required for cross-platform builds. Regular `docker build` will build for your host architecture (amd64/arm64), not RISC-V.

### Option 2: Build on RISC-V Cloud (Recommended)

Building directly on a RISC-V instance is more efficient:

```bash
# SSH into RISC-V cloud instance
ssh user@your-riscv-instance

# Clone repo
git clone YOUR_REPO_URL
cd YOUR_REPO

# Build
docker build -t fibrinlab/risc-rust:latest .

# Push to Docker Hub
docker login
docker push fibrinlab/risc-rust:latest
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

## Tracking Optimizations

For iterative optimization during the hackathon, track your changes:

### Quick Benchmark

```bash
# Run benchmark with 50 iterations (default)
./benchmark.sh input.json

# Run with 100 iterations for more stable results
./benchmark.sh input.json 100

# Results saved to:
# - benchmark_results.json (machine-readable with statistics)
# - OPTIMIZATIONS.md (human-readable log)
```

**Benchmarking Rules:**
- Runs N iterations (default: 50, recommended: 50-100)
- Reports **median** (robust measure)
- Reports **min** (best-case performance)
- Reports **p90** (90th percentile, stability measure)
- Reports **max** (worst-case)
- Verifies correctness (all runs must produce identical hash)

**Why multiple runs?** Single-run timing lies due to system variance (CPU scheduling, cache effects, etc.). Multiple runs provide statistically meaningful results.

### Optimization Log Format

Document each optimization in `OPTIMIZATIONS.md`:
- **Date/Time**: When change was made
- **Change**: What was optimized
- **Before/After Metrics**: Performance comparison
- **Impact**: Improvement percentage

Example entry:
```markdown
### 2025-01-XX 12:00:00 UTC
- **Change**: Added cache-friendly matrix transpose
- **Latency**: 0.123 ms → 0.098 ms (20% improvement)
- **Throughput**: 1.2M → 1.5M ops/sec
- **Hash**: `abc123...` (correctness verified)
```

### Benchmarking Best Practices

1. **Use consistent test cases** - Same matrix sizes for fair comparison
2. **Run N iterations (50-100)** - Single-run timing lies, need statistical measures
3. **Report median, min, p90** - Robust, best-case, and stability metrics
4. **Document each change** - Track what improved performance
5. **Verify correctness** - All runs must produce identical hash
6. **Commit benchmarks** - Git history shows optimization progress

**Proper Benchmarking Rules:**
- **N = 50 or 100** iterations (default: 50)
- **Report median** (robust measure, not affected by outliers)
- **Report min** (best-case performance)
- **Report p90** (90th percentile, stability measure)
- **Verify hash consistency** (all runs must match)

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

That's it. Simple benchmark solver ready for submission.
