# Submission Guide - Amadeus Genesis Hack

## Submission Workflow (from Hackathon Docs)

Based on the hackathon documentation, the submission process follows this workflow:

### 1. Request API Key
- Contact the hackathon organizers to get your API key
- This will be used to authenticate your submissions

### 2. Receive Workload Spec
- The organizers will provide workload specifications
- These will include matrix sizes, precision requirements, and expected formats

### 3. Run Locally / Optimize
- Test and optimize your implementation
- Use the provided workload specs to validate your solution

### 4. Submit via JSON or Upload Container
You have two submission options:

#### Option A: Submit via JSON API
```bash
# Submit benchmark results via API
curl -X POST https://api.hackathon-endpoint.com/submit \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d @output.json
```

#### Option B: Upload Docker Container
- Push your Docker image to a registry (Docker Hub, GitHub Container Registry, etc.)
- Or provide the Dockerfile and source code for them to build

### 5. Receive Score
- You'll get a score based on:
  - Latency (primary)
  - Throughput
  - Correctness
  - Resource usage

### 6. Optional Validation Run
- Request a re-run if your score seems incorrect
- Manual review available

### 7. Score Locked to Leaderboard
- Final scores appear on the real-time leaderboard

## What to Submit

### Required Files:
1. **Dockerfile** - Your container definition
2. **Source Code** - All Rust source files (`src/`)
3. **Cargo.toml** - Project dependencies
4. **README.md** - Documentation (optional but recommended)

### Optional but Recommended:
- **Docker Image** - Pre-built image pushed to a registry
- **Test Results** - Sample output.json showing your implementation works
- **Documentation** - Explanation of optimizations

## Submission Methods

### Method 1: GitHub Repository (Most Common)
1. Create a GitHub repository
2. Push your code:
   ```bash
   git init
   git add .
   git commit -m "MatMul solver submission"
   git remote add origin YOUR_REPO_URL
   git push -u origin main
   ```
3. Share the repository link with organizers

### Method 2: Docker Image Registry
1. Build and tag your image:
   ```bash
   docker build -t matmul-solver-riscv .
   docker tag matmul-solver-riscv your-username/matmul-solver-riscv:latest
   ```
2. Push to Docker Hub or GitHub Container Registry:
   ```bash
   docker push your-username/matmul-solver-riscv:latest
   ```
3. Share the image name with organizers

### Method 3: Direct File Upload
- If organizers provide a file upload portal, zip your project:
   ```bash
   zip -r submission.zip . -x "target/*" ".git/*" "*.log" "output.json"
   ```

## Pre-Submission Checklist

- [ ] Dockerfile uses the correct RISC-V base image
- [ ] Code compiles without errors
- [ ] All required dependencies are in Cargo.toml
- [ ] Input/output format matches specifications
- [ ] Result hash is computed correctly
- [ ] Metrics are calculated accurately
- [ ] Code is tested locally (even if on different architecture)
- [ ] README includes usage instructions
- [ ] No hardcoded solutions (only optimizations allowed)

## Contact Information

Check the hackathon documentation or Discord/Slack for:
- API endpoint URLs
- Submission portal links
- API key request process
- Deadline information
- Technical support contacts

## Important Notes

- **Unlimited submissions per day** - You can iterate and improve
- **Caching allowed** - As long as workload input isn't modified
- **Optimized libraries allowed** - BLIS, OpenBLAS, TVM, custom kernels
- **No workload modification** - Changing workload shapes/data = instant disqualification

## Example Submission Package

Your submission directory should look like:
```
ama/
├── Dockerfile
├── Cargo.toml
├── src/
│   └── main.rs
├── README.md
├── .dockerignore
└── .gitignore
```

Good luck with your submission! 🚀

