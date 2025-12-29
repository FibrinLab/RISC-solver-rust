#!/bin/bash
# Wrapper script to run matmul-solver and keep container alive for Koyeb service deployment

# Check if API mode is requested via environment variable
if [ "$API_MODE" = "true" ] || [ "$API_MODE" = "1" ]; then
    # Run API server (keeps container alive and accepts matrix input via HTTP)
    echo "Starting API server on port 8000..."
    /usr/local/bin/matmul-api
else
    # Run solver once with provided arguments, then start API server for health checks
    if [ $# -gt 0 ]; then
        echo "Running solver with provided arguments..."
        /usr/local/bin/matmul-solver "$@"
        EXIT_CODE=$?
        echo "Solver completed with exit code: $EXIT_CODE"
    fi
    
    # Start API server for health checks and to accept new matrix submissions
    echo "Starting API server on port 8000 for health checks and matrix submissions..."
    /usr/local/bin/matmul-api
fi

