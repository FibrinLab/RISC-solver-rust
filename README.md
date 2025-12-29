# MatMul Solver - Amadeus Genesis Hack

Simple MatMul benchmark solver for the Hard Hack competition.

## Alignment with uPoW

This project is aligned with the **Hard Hack** requirements:
- ✅ **Uses real MatMul workloads** - Aligned with the live uPoW pipeline on Amadeus mainnet
- ✅ **Benchmarks reflect actual compute** - Same MatMul operations that miners run on mainnet today (not simulations)
- ✅ **Multiple precisions** - Supports fp32, fp16, int8, **u8i8** (matching uPoW compute requirements)
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
- ✅ **Matrix Multiplication (MatMul)** - All precisions (fp32, fp16, int8, **u8i8**)
- ✅ **u8i8 (unsigned × signed)** - Optimized for seed dimensions (16×50240 × 50240×16)

## Quick Start

### Local Build & Run

```bash
# Build
cargo build --release --bin matmul-solver

# Run with seed (recommended - no JSON file needed!)
cargo run --release --bin matmul-solver -- --seed "deadbeef1234..." --precision "u8i8"

# Run with JSON input file
cargo run --release --bin matmul-solver -- --input inputs/input.json --output outputs/output.json

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

To run with verification add the --verify flag

**Using seed (recommended - no file mounting needed):**
```bash
# Generate matrices from seed (deterministic)
docker run --rm \
  -v $(pwd)/outputs:/app/outputs \
  matmul-solver \
  --seed "deadbeef1234567890abcdef1234567890abcdef1234567890abcdef1234567890" \
  --precision "u8i8" \
  --output /app/outputs/output.json
```

**Using JSON input file:**
```bash
# Run with input/output files mounted
docker run --rm \
  -v $(pwd)/inputs/input_fp32.json:/app/input.json \
  -v $(pwd)/outputs:/app/outputs \
  matmul-solver \
  --input /app/input.json --output /app/outputs/output.json
```

#### Run Benchmarking (Multiple Iterations)

```bash
# Run benchmark with seed (recommended - no file mounting needed) with 100 iterations
docker run --rm \
  -v $(pwd)/outputs:/app/outputs \
  --entrypoint /bin/bash \
  matmul-solver \
  -c "/app/benchmark.sh 'deadbeef1234567890abcdef1234567890abcdef1234567890abcdef1234567890' 100 u8i8"

# Run benchmark with JSON input file
docker run --rm \
  -v $(pwd)/inputs/input_fp32.json:/app/input.json \
  -v $(pwd)/outputs:/app/outputs \
  --entrypoint /bin/bash \
  matmul-solver \
  /app/benchmark.sh /app/input.json 50
```

**Troubleshooting:** If you encounter disk space errors during build, clean Docker:
```bash
docker system prune -a --volumes
```

## Deployment Launch Command

For deployment platforms (like Koyeb), the container runs in **API mode** by default:

**API Mode (Default for Koyeb):**
- The container starts an HTTP API server on port 8000
- Judges can submit matrices via `POST /compute` endpoint (with `seed` or `matrix_a`/`matrix_b`)
- Health checks use `GET /health` endpoint
- Container stays alive and accepts multiple requests

**CLI Mode (One-shot execution):**
If you want to run the solver once and exit, override the entrypoint:

**Using seed (recommended - no input file needed):**
```bash
matmul-solver --seed "deadbeef1234..." --precision "u8i8" --output /app/outputs/output.json
```

**Using JSON input file:**
```bash
matmul-solver --input /app/input.json --output /app/outputs/output.json --verify
```

## API Endpoints (for Judges)

When deployed to Koyeb, the service exposes HTTP endpoints:

**POST /compute**
- Submit matrix computation request
- Request body: JSON with `matrix_a`, `matrix_b`, `precision` (e.g., "u8i8", "fp32", "fp16", "int8")
- Response: JSON with `result_matrix`, `result_hash`, `metrics` (including throughput)

**GET /health**
- Health check endpoint
- Returns: "OK"

**Example API Request (with seed - recommended):**
```bash
curl -X POST http://your-koyeb-url/compute \
  -H "Content-Type: application/json" \
  -d '{
    "seed": "deadbeef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    "precision": "u8i8"
  }'
```

**Example API Request (with matrices):**
```bash
curl -X POST http://your-koyeb-url/compute \
  -H "Content-Type: application/json" \
  -d '{
    "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
    "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
    "precision": "u8i8"
  }'
```

**For u8i8 seed dimensions (16×50240 × 50240×16):**
- Use `seed` field instead of `matrix_a`/`matrix_b` - matrices are generated deterministically from the seed
- The solver uses Blake3 XOF to generate matrices (matches PoW specification)
- No need to send large JSON files - just provide a hex seed string

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

**Supported precisions:** `fp32`, `fp16`, `int8`, `u8i8`

**Note:** `u8i8` is optimized for the seed workload dimensions (16×50240 × 50240×16 = 16×16 result). This matches the PoW specification where matrices come from raw binary (u8 for matrix_a, i8 for matrix_b).

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


## Tracking Optimizations

For iterative optimization during the hackathon, track your changes:

### Quick Benchmark

The `benchmark.sh` script automatically follows these rules:
```bash
# Using JSON input file (default: 50 runs)
./benchmark.sh inputs/input.json

# 100 runs for more stable results
./benchmark.sh inputs/input.json 100
# Or with seed:
./benchmark.sh "deadbeef1234567890abcdef1234567890abcdef1234567890abcdef1234567890" 100 u8i8

# Compare median results between optimizations
jq '.metrics.latency.median' outputs/benchmark_results.json
```

**Note:** The benchmark script automatically detects if the first argument is a seed (hex string) or a file path. For seed mode, provide: `seed_hex [num_runs] [precision]`
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
