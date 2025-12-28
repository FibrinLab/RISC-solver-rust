#!/bin/bash
# Wrapper script to run matmul-solver and keep container alive for Koyeb service deployment

# Run the solver with all arguments passed to this script
/usr/local/bin/matmul-solver "$@"

# Capture exit code
EXIT_CODE=$?

# Keep container alive after solver completes
# This allows Koyeb to maintain the service and scale it
# The service will show as "running" even after computation completes
exec sleep infinity

