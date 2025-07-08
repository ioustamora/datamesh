#!/bin/bash

# Simple test to debug retrieval issues
set -e

echo "=== DFS Retrieval Debug Test ==="

# Clean up any existing processes
pkill -f dfs || true
sleep 2

# Start bootstrap node
echo "Starting bootstrap node..."
./target/debug/dfs bootstrap --port 41001 > bootstrap_debug.log 2>&1 &
BOOTSTRAP_PID=$!
sleep 3

echo "Bootstrap node started (PID: $BOOTSTRAP_PID)"

# Get bootstrap peer info from log
BOOTSTRAP_PEER=$(grep "Peer ID:" bootstrap_debug.log | cut -d' ' -f3 || echo "")
if [ -z "$BOOTSTRAP_PEER" ]; then
    echo "Failed to get bootstrap peer ID"
    exit 1
fi
echo "Bootstrap peer ID: $BOOTSTRAP_PEER"

# Start a service node
echo "Starting service node..."
./target/debug/dfs service \
    --bootstrap-peer "$BOOTSTRAP_PEER" \
    --bootstrap-addr "/ip4/127.0.0.1/tcp/41001" \
    --port 41002 > service_debug.log 2>&1 &
SERVICE_PID=$!
sleep 5

echo "Service node started (PID: $SERVICE_PID)"

# Store a test file
echo "test content for debug" > test_debug.txt
echo "Storing test file..."
FILE_KEY=$(./target/debug/dfs \
    --bootstrap-peer "$BOOTSTRAP_PEER" \
    --bootstrap-addr "/ip4/127.0.0.1/tcp/41001" \
    --non-interactive \
    put test_debug.txt 2>&1 | grep "File stored with key:" | cut -d' ' -f5 || echo "")

if [ -z "$FILE_KEY" ]; then
    echo "Failed to store file"
    kill $BOOTSTRAP_PID $SERVICE_PID 2>/dev/null || true
    exit 1
fi

echo "File stored with key: $FILE_KEY"

# Wait for DHT propagation
echo "Waiting for DHT propagation..."
sleep 10

# Try to retrieve the file with verbose output
echo "Attempting to retrieve file..."
./target/debug/dfs \
    --bootstrap-peer "$BOOTSTRAP_PEER" \
    --bootstrap-addr "/ip4/127.0.0.1/tcp/41001" \
    --non-interactive \
    get "$FILE_KEY" retrieved_debug.txt

if [ -f "retrieved_debug.txt" ]; then
    echo "SUCCESS: File retrieved successfully"
    echo "Original content:"
    cat test_debug.txt
    echo "Retrieved content:"
    cat retrieved_debug.txt
    if diff test_debug.txt retrieved_debug.txt >/dev/null 2>&1; then
        echo "Content verification: PASS"
    else
        echo "Content verification: FAIL"
    fi
else
    echo "FAILED: File could not be retrieved"
fi

# Cleanup
echo "Cleaning up..."
kill $BOOTSTRAP_PID $SERVICE_PID 2>/dev/null || true
rm -f test_debug.txt retrieved_debug.txt

echo "Debug test complete"
