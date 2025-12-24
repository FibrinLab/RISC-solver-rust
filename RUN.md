# How to Run the MatMul Solver

## Quick Start

### Option 1: Run Locally (for Testing)

Test your implementation on your local machine:

```bash
# Build the project
cargo build --release

# Run with default input.json
cargo run --release

# Or specify custom input/output files
cargo run --release -- --input input.json --output output.json
```

### Option 2: Run in Docker (RISC-V)

**Important**: The Docker image is built for RISC-V architecture. It's designed to run on the hackathon's benchmarking infrastructure.

```bash
# Build the RISC-V Docker image
docker build -t matmul-solver-riscv .

# Run the container
docker run --rm \
  -v $(pwd)/input.json:/app/input.json \
  -v $(pwd):/app/output \
  matmul-solver-riscv \
  --input /app/input.json \
  --output /app/output/output.json
```

**Note**: If you're on a non-RISC-V system (like x86_64 or ARM64 Mac), Docker may not be able to run this directly. The image is meant to be submitted to the hackathon organizers who will run it on their RISC-V hardware.

## Input Format

Create an `input.json` file with your matrices:

```json
{
  "matrix_a": [
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0]
  ],
  "matrix_b": [
    [7.0, 8.0],
    [9.0, 10.0],
    [11.0, 12.0]
  ],
  "precision": "fp32",
  "metadata": {
    "compiler_flags": "-O3",
    "libraries": [],
    "cache_enabled": true
  }
}
```

## Output

The solver will create an `output.json` file with:
- `result_matrix`: The computed matrix multiplication result
- `result_hash`: SHA-256 hash for correctness verification
- `metrics`: Performance metrics (latency, throughput, ops/sec)
- `metadata`: Information about the computation

## For Hackathon Submission

1. **Build the Docker image**: `docker build -t matmul-solver-riscv .`
2. **Test locally** (optional): `cargo run --release` to verify logic
3. **Submit the Docker image** to the hackathon organizers
4. They will run it on their RISC-V benchmarking infrastructure with their test workloads

## Example Run

```bash
# 1. Create test input
cat > input.json << EOF
{
  "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
  "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
  "precision": "fp32"
}
EOF

# 2. Run locally to test
cargo run --release

# 3. Check output
cat output.json
```

