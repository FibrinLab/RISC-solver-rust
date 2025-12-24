# Koyeb Deployment Guide

## Overview

This guide explains how to deploy the MatMul Solver API to Koyeb on RISC-V infrastructure.

## Prerequisites

- Koyeb account with API key
- Docker image built for RISC-V
- GitHub repository (optional, for Git-based deployment)

## Deployment Options

### Option 1: Deploy via Docker Image

1. **Build the API Docker image:**
   ```bash
   docker build -f Dockerfile.api -t matmul-solver-api:latest .
   ```

2. **Tag and push to a registry:**
   ```bash
   # Tag for your registry
   docker tag matmul-solver-api:latest your-registry/matmul-solver-api:latest
   
   # Push to registry (Docker Hub, GHCR, etc.)
   docker push your-registry/matmul-solver-api:latest
   ```

3. **Deploy on Koyeb:**
   - Go to Koyeb dashboard
   - Create new app
   - Select "Docker" as source
   - Enter your image: `your-registry/matmul-solver-api:latest`
   - Set environment variables if needed
   - Deploy!

### Option 2: Deploy via GitHub (Recommended)

1. **Push code to GitHub:**
   ```bash
   git init
   git add .
   git commit -m "MatMul Solver API"
   git remote add origin YOUR_GITHUB_REPO
   git push -u origin main
   ```

2. **Deploy on Koyeb:**
   - Go to Koyeb dashboard
   - Create new app
   - Connect your GitHub repository
   - Set build command: `cargo build --release --bin matmul-api`
   - Set run command: `./target/release/matmul-api`
   - Set Dockerfile path: `Dockerfile.api` (if using Docker build)
   - Deploy!

## Environment Variables

The API server uses the `PORT` environment variable (defaults to 8080). Koyeb will automatically set this.

Optional environment variables:
- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Logging level (e.g., `info`, `debug`)

## API Endpoints

Once deployed, your API will have these endpoints:

### Health Check
```bash
GET https://your-app.koyeb.app/health
```

Response:
```json
{
  "status": "ok"
}
```

### Solve Matrix Multiplication
```bash
POST https://your-app.koyeb.app/solve
Content-Type: application/json

{
  "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
  "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
  "precision": "fp32"
}
```

Response:
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

## Testing Your Deployment

```bash
# Health check
curl https://your-app.koyeb.app/health

# Test solve endpoint
curl -X POST https://your-app.koyeb.app/solve \
  -H "Content-Type: application/json" \
  -d '{
    "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
    "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
    "precision": "fp32"
  }'
```

## For Hackathon Submission

When submitting to the hackathon organizers, provide:

1. **API Endpoint URL**: `https://your-app.koyeb.app/solve`
2. **Health Check URL**: `https://your-app.koyeb.app/health`
3. **Documentation**: This file or a link to your README
4. **Source Code**: GitHub repository link (if applicable)

The organizers can then call your API endpoint directly with their test workloads.

## Troubleshooting

### Build Fails
- Ensure you're using the RISC-V base image
- Check that all dependencies are in Cargo.toml
- Verify Rust toolchain installs correctly

### API Not Responding
- Check Koyeb logs: `koyeb logs your-app-name`
- Verify PORT environment variable is set
- Ensure the binary is built correctly

### Performance Issues
- RISC-V architecture may have different performance characteristics
- Check Koyeb resource allocation
- Monitor logs for errors

## Next Steps

1. Deploy to Koyeb
2. Test all endpoints
3. Share API URL with hackathon organizers
4. Monitor performance during benchmarking

Good luck! 🚀

