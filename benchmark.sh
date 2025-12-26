#!/bin/bash
# Benchmark script to track optimization progress
# Follows proper benchmarking rules: N runs, report median, min, p90
# Works both locally (with cargo) and in Docker (with binary)

set -e

BENCHMARK_INPUT="${1:-input.json}"
NUM_RUNS="${2:-50}"  # Default to 50 runs, can override with second arg
RESULTS_FILE="benchmark_results.json"
LOG_FILE="OPTIMIZATIONS.md"
TEMP_DIR="/tmp/bench_$$"

# Detect if running in Docker or locally
if [ -f "/usr/local/bin/matmul-solver" ] || [ -f "/app/target/release/matmul-solver" ]; then
  # Running in Docker - use binary directly
  if [ -f "/usr/local/bin/matmul-solver" ]; then
    SOLVER_BIN="/usr/local/bin/matmul-solver"
  else
    SOLVER_BIN="/app/target/release/matmul-solver"
  fi
  USE_CARGO=false
else
  # Running locally - use cargo
  SOLVER_BIN=""
  USE_CARGO=true
fi

mkdir -p "$TEMP_DIR"

echo "=== Running Benchmark (N=$NUM_RUNS runs) ==="
echo "Input: $BENCHMARK_INPUT"
echo "Iterations: $NUM_RUNS"
echo ""

# Run N iterations
echo "Running $NUM_RUNS iterations..."
LATENCIES=()
THROUGHPUTS=()
OPS_PER_SECS=()

for i in $(seq 1 $NUM_RUNS); do
  if [ $((i % 10)) -eq 0 ] || [ $i -eq 1 ]; then
    echo -ne "\r  Progress: $i/$NUM_RUNS"
  fi
  
  # Run solver (either via cargo or binary)
  if [ "$USE_CARGO" = true ]; then
    if ! cargo run --release --bin matmul-solver -- \
      --input "$BENCHMARK_INPUT" \
      --output "$TEMP_DIR/run_$i.json" \
      --verify > /dev/null 2>&1; then
      echo ""
      echo "❌ Error: Failed to run iteration $i"
      rm -rf "$TEMP_DIR"
      exit 1
    fi
  else
    if ! "$SOLVER_BIN" \
      --input "$BENCHMARK_INPUT" \
      --output "$TEMP_DIR/run_$i.json" \
      --verify > /dev/null 2>&1; then
      echo ""
      echo "❌ Error: Failed to run iteration $i"
      rm -rf "$TEMP_DIR"
      exit 1
    fi
  fi
  
  if [ ! -f "$TEMP_DIR/run_$i.json" ]; then
    echo ""
    echo "❌ Error: Output file not created for iteration $i"
    rm -rf "$TEMP_DIR"
    exit 1
  fi
  
  LATENCY=$(jq -r '.metrics.latency_ms' "$TEMP_DIR/run_$i.json")
  THROUGHPUT=$(jq -r '.metrics.throughput_ops_per_sec' "$TEMP_DIR/run_$i.json")
  OPS_PER_SEC=$(jq -r '.metrics.ops_per_second' "$TEMP_DIR/run_$i.json")
  
  # Validate values are numeric
  if ! echo "$LATENCY" | grep -qE '^[0-9]+\.?[0-9]*$'; then
    echo ""
    echo "❌ Error: Invalid latency value: $LATENCY"
    rm -rf "$TEMP_DIR"
    exit 1
  fi
  
  LATENCIES+=("$LATENCY")
  THROUGHPUTS+=("$THROUGHPUT")
  OPS_PER_SECS+=("$OPS_PER_SEC")
done

echo -e "\r  Progress: $NUM_RUNS/$NUM_RUNS - Complete!"
echo ""

# Get metadata from last run (should be identical across runs)
LAST_RUN="$TEMP_DIR/run_$NUM_RUNS.json"
HASH=$(jq -r '.result_hash' "$LAST_RUN")
PRECISION=$(jq -r '.metadata.precision' "$LAST_RUN")
SHAPE=$(jq -r '.metadata.result_shape' "$LAST_RUN")
MATRIX_A_SHAPE=$(jq -r '.metadata.matrix_a_shape' "$LAST_RUN")
MATRIX_B_SHAPE=$(jq -r '.metadata.matrix_b_shape' "$LAST_RUN")
MEMORY=$(jq -r '.metrics.memory_usage_mb // "N/A"' "$LAST_RUN")

# Verify all hashes are identical (correctness check)
FIRST_HASH=$(jq -r '.result_hash' "$TEMP_DIR/run_1.json")
ALL_SAME=true
for i in $(seq 2 $NUM_RUNS); do
  RUN_HASH=$(jq -r '.result_hash' "$TEMP_DIR/run_$i.json")
  if [ "$RUN_HASH" != "$FIRST_HASH" ]; then
    echo "⚠️  WARNING: Hash mismatch detected! Run 1: $FIRST_HASH, Run $i: $RUN_HASH"
    ALL_SAME=false
  fi
done

if [ "$ALL_SAME" = true ]; then
  echo "✅ Correctness verified: All $NUM_RUNS runs produced identical hash"
else
  echo "❌ Correctness check failed: Hashes differ between runs!"
fi

echo ""

# Calculate statistics using Python (more reliable than bash for math)
# Write arrays to temp file to avoid command-line length issues
# Format arrays properly as JSON - convert to float first
LATENCIES_JSON=$(printf '%s\n' "${LATENCIES[@]}" | jq -R 'tonumber' | jq -s .)
THROUGHPUTS_JSON=$(printf '%s\n' "${THROUGHPUTS[@]}" | jq -R 'tonumber' | jq -s .)
OPS_PER_SECS_JSON=$(printf '%s\n' "${OPS_PER_SECS[@]}" | jq -R 'tonumber' | jq -s .)

# Verify we have data
if [ ${#LATENCIES[@]} -eq 0 ]; then
  echo "❌ Error: No latency data collected"
  rm -rf "$TEMP_DIR"
  exit 1
fi

jq -n \
  --argjson latencies "$LATENCIES_JSON" \
  --argjson throughputs "$THROUGHPUTS_JSON" \
  --argjson ops_per_secs "$OPS_PER_SECS_JSON" \
  '{
    latencies: $latencies,
    throughputs: $throughputs,
    ops_per_secs: $ops_per_secs
  }' > "$TEMP_DIR/arrays.json"

if [ ! -s "$TEMP_DIR/arrays.json" ]; then
  echo "❌ Error: Failed to create arrays.json"
  rm -rf "$TEMP_DIR"
  exit 1
fi

# Call Python with the arrays file as argument
STATS=$(python3 - "$TEMP_DIR/arrays.json" << 'PYEOF'
import json
import sys

if len(sys.argv) < 2:
    print('{"error": "No input file provided"}', file=sys.stderr)
    sys.exit(1)

try:
    with open(sys.argv[1], 'r') as f:
        data = json.load(f)
except FileNotFoundError:
    print('{"error": "Arrays file not found"}', file=sys.stderr)
    sys.exit(1)
except json.JSONDecodeError as e:
    print(f'{{"error": "Invalid JSON: {e}"}}', file=sys.stderr)
    sys.exit(1)

latencies = data.get('latencies', [])
throughputs = data.get('throughputs', [])
ops_per_secs = data.get('ops_per_secs', [])

if not latencies or not throughputs or not ops_per_secs:
    print('{"error": "Empty arrays"}', file=sys.stderr)
    sys.exit(1)

latencies.sort()
throughputs.sort()
ops_per_secs.sort()

n = len(latencies)

if n == 0:
    print('{"error": "No data"}', file=sys.stderr)
    sys.exit(1)

# Median
lat_median = latencies[n//2] if n % 2 == 1 else (latencies[n//2-1] + latencies[n//2]) / 2
thr_median = throughputs[n//2] if n % 2 == 1 else (throughputs[n//2-1] + throughputs[n//2]) / 2
ops_median = ops_per_secs[n//2] if n % 2 == 1 else (ops_per_secs[n//2-1] + ops_per_secs[n//2]) / 2

# Min (best-case)
lat_min = latencies[0]
thr_min = throughputs[0]
ops_min = ops_per_secs[0]

# P90 (90th percentile)
p90_idx = int(n * 0.9)
lat_p90 = latencies[p90_idx] if p90_idx < n else latencies[-1]
thr_p90 = throughputs[p90_idx] if p90_idx < n else throughputs[-1]
ops_p90 = ops_per_secs[p90_idx] if p90_idx < n else ops_per_secs[-1]

# Max (worst-case)
lat_max = latencies[-1]
thr_max = throughputs[-1]
ops_max = ops_per_secs[-1]

result = {
    "latency": {
        "min": lat_min,
        "median": lat_median,
        "p90": lat_p90,
        "max": lat_max
    },
    "throughput": {
        "min": thr_min,
        "median": thr_median,
        "p90": thr_p90,
        "max": thr_max
    },
    "ops_per_second": {
        "min": ops_min,
        "median": ops_median,
        "p90": ops_p90,
        "max": ops_max
    }
}

print(json.dumps(result))
PYEOF
)

# Check if Python succeeded
if [ $? -ne 0 ] || [ -z "$STATS" ] || echo "$STATS" | jq -e '.error' > /dev/null 2>&1; then
  echo "❌ Error: Failed to calculate statistics"
  echo "Python output: $STATS"
  rm -rf "$TEMP_DIR"
  exit 1
fi

# Extract stats
LAT_MIN=$(echo "$STATS" | jq -r '.latency.min')
LAT_MEDIAN=$(echo "$STATS" | jq -r '.latency.median')
LAT_P90=$(echo "$STATS" | jq -r '.latency.p90')
LAT_MAX=$(echo "$STATS" | jq -r '.latency.max')

THR_MIN=$(echo "$STATS" | jq -r '.throughput.min')
THR_MEDIAN=$(echo "$STATS" | jq -r '.throughput.median')
THR_P90=$(echo "$STATS" | jq -r '.throughput.p90')
THR_MAX=$(echo "$STATS" | jq -r '.throughput.max')

OPS_MIN=$(echo "$STATS" | jq -r '.ops_per_second.min')
OPS_MEDIAN=$(echo "$STATS" | jq -r '.ops_per_second.median')
OPS_P90=$(echo "$STATS" | jq -r '.ops_per_second.p90')
OPS_MAX=$(echo "$STATS" | jq -r '.ops_per_second.max')

# Display results
echo "=== Benchmark Results (N=$NUM_RUNS) ==="
echo "Precision: $PRECISION"
echo "Matrix A: $MATRIX_A_SHAPE"
echo "Matrix B: $MATRIX_B_SHAPE"
echo "Result Shape: $SHAPE"
echo "Memory: ${MEMORY} MB"
echo "Hash: $HASH"
echo ""
echo "Latency (ms):"
echo "  Min (best-case):    ${LAT_MIN}"
echo "  Median (robust):    ${LAT_MEDIAN}"
echo "  P90 (stability):    ${LAT_P90}"
echo "  Max (worst-case):   ${LAT_MAX}"
echo ""
echo "Throughput (ops/sec):"
echo "  Min:                ${THR_MIN}"
echo "  Median:              ${THR_MEDIAN}"
echo "  P90:                 ${THR_P90}"
echo "  Max:                 ${THR_MAX}"
echo ""

# Save to JSON results file
jq -n \
  --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg precision "$PRECISION" \
  --arg shape "$SHAPE" \
  --arg memory "$MEMORY" \
  --arg hash "$HASH" \
  --argjson stats "$STATS" \
  --arg num_runs "$NUM_RUNS" \
  '{
    timestamp: $date,
    num_runs: ($num_runs | tonumber),
    precision: $precision,
    shape: $shape,
    memory_usage_mb: (if $memory == "N/A" then null else ($memory | tonumber) end),
    result_hash: $hash,
    metrics: $stats
  }' > "$RESULTS_FILE"

echo "Results saved to: $RESULTS_FILE"

# Append to optimization log
if [ -f "$LOG_FILE" ]; then
  echo "" >> "$LOG_FILE"
  echo "### $(date -u +%Y-%m-%d\ %H:%M:%S\ UTC) (N=$NUM_RUNS runs)" >> "$LOG_FILE"
  echo "- **Precision**: $PRECISION" >> "$LOG_FILE"
  echo "- **Matrix Size**: A=$MATRIX_A_SHAPE, B=$MATRIX_B_SHAPE" >> "$LOG_FILE"
  echo "- **Latency (ms)**: min=${LAT_MIN}, median=${LAT_MEDIAN}, p90=${LAT_P90}, max=${LAT_MAX}" >> "$LOG_FILE"
  echo "- **Throughput (ops/sec)**: min=${THR_MIN}, median=${THR_MEDIAN}, p90=${THR_P90}, max=${THR_MAX}" >> "$LOG_FILE"
  echo "- **Memory**: ${MEMORY} MB" >> "$LOG_FILE"
  echo "- **Hash**: \`$HASH\`" >> "$LOG_FILE"
  echo "" >> "$LOG_FILE"
  echo "Results appended to: $LOG_FILE"
fi

# Cleanup
rm -rf "$TEMP_DIR"

