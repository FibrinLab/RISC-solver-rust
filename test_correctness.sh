#!/bin/bash
# Test script to verify correctness and hash consistency

set -e

echo "=== MatMul Solver Correctness Tests ==="
echo ""

# Create a simple test input
mkdir -p inputs outputs
cat > inputs/test_input.json << 'EOF'
{
  "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
  "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
  "precision": "fp32"
}
EOF

echo "1. Running MatMul computation..."
cargo run --release --bin matmul-solver -- --input inputs/test_input.json --output outputs/test_output.json

echo ""
echo "2. Checking result correctness..."
EXPECTED_RESULT="[[19.0,22.0],[43.0,50.0]]"
ACTUAL_RESULT=$(jq -c '.result_matrix' outputs/test_output.json)

if [ "$ACTUAL_RESULT" = "$EXPECTED_RESULT" ]; then
    echo "   ✅ Result is correct!"
else
    echo "   ❌ Result mismatch!"
    echo "   Expected: $EXPECTED_RESULT"
    echo "   Got:      $ACTUAL_RESULT"
    exit 1
fi

echo ""
echo "3. Testing hash consistency (running 5 times)..."
HASHES=()
for i in {1..5}; do
    cargo run --release --bin matmul-solver -- --input inputs/test_input.json --output "outputs/test_output_$i.json" > /dev/null 2>&1
    HASH=$(jq -r '.result_hash' "outputs/test_output_$i.json")
    HASHES+=("$HASH")
    echo "   Run $i: $HASH"
done

echo ""
UNIQUE_HASHES=$(printf '%s\n' "${HASHES[@]}" | sort -u | wc -l)
if [ "$UNIQUE_HASHES" -eq 1 ]; then
    echo "   ✅ All hashes are identical (correctness verified)"
else
    echo "   ❌ Hashes differ between runs!"
    exit 1
fi

echo ""
echo "4. Running Rust unit tests..."
cargo test --lib

echo ""
echo "=== All tests passed! ==="

# Cleanup
rm -f inputs/test_input.json outputs/test_output*.json

