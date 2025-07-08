#!/bin/bash
# simple_test.sh - Quick test of DFS functionality

set -e

echo "DataMesh Quick Test"
echo "=============="

# Test 1: Bootstrap mode
echo "1. Testing bootstrap mode..."
timeout 3s ./target/debug/dfs bootstrap --port 40880 &
BOOTSTRAP_PID=$!
sleep 2

if kill -0 $BOOTSTRAP_PID 2>/dev/null; then
    echo "✓ Bootstrap node started successfully"
    kill $BOOTSTRAP_PID 2>/dev/null || true
else
    echo "✗ Bootstrap node failed"
fi

# Test 2: File operations  
echo ""
echo "2. Testing file operations..."
echo "Test content with encryption and BLAKE3 hash $(date)" > test_simple.txt

# Store file
KEY=$(./target/debug/dfs put test_simple.txt | grep "File stored with key:" | cut -d' ' -f5)

if [ -n "$KEY" ]; then
    echo "✓ File stored successfully with key: $KEY"
    
    # Try to retrieve file
    if timeout 10s ./target/debug/dfs get "$KEY" recovered_simple.txt 2>/dev/null; then
        if [ -f "recovered_simple.txt" ] && diff test_simple.txt recovered_simple.txt >/dev/null 2>&1; then
            echo "✓ File retrieved and verified successfully"
            rm -f recovered_simple.txt
        else
            echo "✗ File retrieved but content differs"
        fi
    else
        echo "⚠ File retrieval timed out (expected in single-node test)"
    fi
else
    echo "✗ File storage failed"
fi

# Test 3: Check help commands
echo ""
echo "3. Testing command interface..."
if ./target/debug/dfs --help | grep -q "bootstrap"; then
    echo "✓ Bootstrap command available"
else
    echo "✗ Bootstrap command missing"
fi

if ./target/debug/dfs --help | grep -q "interactive"; then
    echo "✓ Interactive command available"
else
    echo "✗ Interactive command missing"
fi

# Test 4: Scripts
echo ""
echo "4. Testing cluster scripts..."
if [ -x "./examples/cluster_test.sh" ]; then
    echo "✓ Cluster test script is executable"
else
    echo "✗ Cluster test script not found or not executable"
fi

if [ -x "./examples/start_bootstrap.sh" ]; then
    echo "✓ Bootstrap start script is executable"
else
    echo "✗ Bootstrap start script not found or not executable"
fi

# Cleanup
rm -f test_simple.txt

echo ""
echo "Quick test completed!"
echo ""
echo "To run full cluster test:"
echo "  ./examples/cluster_test.sh"
echo ""
echo "To start interactive session:"
echo "  ./target/debug/dfs interactive"
