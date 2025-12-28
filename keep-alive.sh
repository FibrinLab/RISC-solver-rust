#!/bin/bash
# Wrapper script to run matmul-solver and keep container alive for Koyeb service deployment

# Start a simple HTTP server on port 8000 for health checks (runs in background)
python3 -m http.server 8000 > /dev/null 2>&1 &
HTTP_SERVER_PID=$!

# Run the solver with all arguments passed to this script
/usr/local/bin/matmul-solver "$@"

# Capture exit code
EXIT_CODE=$?

# Keep container alive after solver completes
# The HTTP server keeps health checks passing while container stays running
# This allows Koyeb to maintain the service and scale it
wait $HTTP_SERVER_PID

