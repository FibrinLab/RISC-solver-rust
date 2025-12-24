# MatMul Solver - Amadeus Genesis Hack

A high-performance matrix multiplication solver for the Amadeus Genesis Hack Hard Hack competition, supporting multiple precision formats (fp32, fp16, int8) with comprehensive benchmarking and correctness verification.

## Features

- **Multiple Precision Support**: fp32, fp16, and int8
- **Optimized Implementations**: Cache-friendly algorithms for better performance
- **Comprehensive Metrics**: Latency, throughput, ops/sec, memory usage
- **Correctness Verification**: SHA-256 hash of results
- **Docker Support**: Fully containerized for reproducibility
- **JSON I/O**: Standardized input/output format

## Building

### Local Build

```bash
cargo build --release
```

### Docker Build

The Docker image is built for RISC-V architecture using the TensTorrent base image provided by the hackathon organizers:

```bash
docker build -t matmul-solver .
```

**Note**: This uses the RISC-V base image `ghcr.io/tenstorrent/tt-xla/tt-xla-ird-ubuntu-22-04:latest` as required for the benchmarking competition.

## Usage

### Local Execution

```bash
# Using default input.json and output.json
cargo run --release

# Specify custom input/output files
cargo run --release -- --input custom_input.json --output custom_output.json
```

### Docker Execution

**Note**: This image is built for RISC-V architecture. It will run on the hackathon's RISC-V benchmarking infrastructure. On non-RISC-V systems, you may need to use emulation or wait for the official benchmarking environment.

#### On RISC-V Hardware/Emulation:

```bash
# Mount input file and output directory
docker run --rm \
  -v $(pwd)/input.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver-riscv \
  --input /app/input.json \
  --output /app/output/output.json
```

#### For Hackathon Submission:

The hackathon organizers will run your container on their RISC-V infrastructure. Make sure your `input.json` follows the expected format, and the container will produce `output.json` with metrics and results.

#### Testing Locally (x86/ARM):

To test the logic locally before submission, you can build and run the Rust code directly:

```bash
# Build locally
cargo build --release

# Run with your input
cargo run --release -- --input input.json --output output.json
```

This will help verify your implementation works correctly, though performance metrics will differ from RISC-V hardware.

## Input Format

The input JSON file should follow this structure:

```json
{
  "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
  "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
  "precision": "fp32",
  "metadata": {
    "compiler_flags": "-O3 -march=native",
    "libraries": ["rayon"],
    "cache_enabled": true
  }
}
```

### Fields

- `matrix_a`: First matrix (2D array of floats)
- `matrix_b`: Second matrix (2D array of floats)
- `precision`: One of "fp32", "fp16", or "int8"
- `metadata` (optional): Additional metadata about the run

## Output Format

The output JSON file contains:

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
    "result_shape": [2, 2],
    "compiler_flags": "-O3 -march=native",
    "libraries": ["rayon"]
  }
}
```

### Output Fields

- `result_matrix`: The computed matrix multiplication result
- `result_hash`: SHA-256 hash of the result for correctness verification
- `metrics`: Performance metrics
  - `latency_ms`: Execution time in milliseconds
  - `throughput_ops_per_sec`: Operations per second
  - `ops_per_second`: Same as throughput (for compatibility)
  - `memory_usage_mb`: Estimated memory usage in MB
- `metadata`: Information about the computation

## Implementation Details

### Precision Implementations

- **fp32**: Optimized implementation with cache-friendly memory access (transposed B matrix)
- **fp16**: Uses half-precision floating point with conversion to/from fp32
- **int8**: Quantized implementation with dynamic scaling

### Performance Optimizations

- Cache-friendly memory access patterns
- Transposed matrix for better cache locality (fp32)
- Efficient data type conversions (fp16, int8)

## Submission Requirements

This implementation includes:

- ✅ Raw metrics (latency, throughput, ops/sec)
- ✅ Correctness proof (result hash)
- ✅ Docker container for reproducibility
- ✅ Source code
- ✅ Benchmark metadata

## Testing

Create a test input file:

```bash
cat > input.json << EOF
{
  "matrix_a": [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
  "matrix_b": [[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]],
  "precision": "fp32"
}
EOF
```

Run the solver:

```bash
cargo run --release
```

Check the output:

```bash
cat output.json
```

## License

This project is submitted for the Amadeus Genesis Hack competition.

