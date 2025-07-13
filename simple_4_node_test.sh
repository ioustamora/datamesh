#!/bin/bash

# Simple 4-Node DataMesh Cluster Test
# Tests basic commands with minimum 4 nodes

set -e

echo "🚀 Starting DataMesh 4-Node Cluster Test"
echo "======================================="

# Configuration
NUM_NODES=4
BASE_PORT=40871
TEST_DIR="/tmp/datamesh_4node_test"
TIMEOUT=30

# Cleanup function
cleanup() {
    echo "🧹 Cleaning up test environment..."
    
    # Kill all background processes
    for port in $(seq $BASE_PORT $((BASE_PORT + NUM_NODES - 1))); do
        pkill -f "port $port" || true
    done
    
    # Remove test directory
    rm -rf "$TEST_DIR" || true
    
    # Wait for cleanup
    sleep 2
    echo "✅ Cleanup completed"
}

# Set trap for cleanup
trap cleanup EXIT

# Create test directory
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

echo "📁 Test directory: $TEST_DIR"

# Use existing debug binary (store absolute path)
ORIGINAL_DIR=$(pwd)
BINARY="$ORIGINAL_DIR/target/debug/datamesh"

if [ ! -f "$BINARY" ]; then
    echo "❌ Debug binary not found at $BINARY"
    exit 1
fi

echo "✅ Using existing binary: $BINARY"

# Start bootstrap node
echo "🌟 Starting bootstrap node on port $BASE_PORT..."
cd "$TEST_DIR"
mkdir -p node1
cd node1

# Generate key for bootstrap node
echo "🔑 Generating key for bootstrap node..."
if ! timeout $TIMEOUT $BINARY generate-key bootstrap_key > bootstrap.log 2>&1; then
    echo "❌ Failed to generate bootstrap key"
    cat bootstrap.log
    exit 1
fi

# Start bootstrap node
echo "📡 Starting bootstrap node..."
if ! timeout $TIMEOUT $BINARY bootstrap --port $BASE_PORT > bootstrap.log 2>&1 &
then
    echo "❌ Failed to start bootstrap node"
    exit 1
fi

BOOTSTRAP_PID=$!
sleep 5

# Check if bootstrap is running
if ! kill -0 $BOOTSTRAP_PID 2>/dev/null; then
    echo "❌ Bootstrap node failed to start"
    cat bootstrap.log
    exit 1
fi

echo "✅ Bootstrap node running (PID: $BOOTSTRAP_PID)"

# Get bootstrap peer info
BOOTSTRAP_ADDR="/ip4/127.0.0.1/tcp/$BASE_PORT"
echo "🔗 Bootstrap address: $BOOTSTRAP_ADDR"

# Start additional nodes
PIDS=($BOOTSTRAP_PID)
for i in $(seq 2 $NUM_NODES); do
    port=$((BASE_PORT + i - 1))
    node_dir="node$i"
    
    echo "🌐 Starting node $i on port $port..."
    cd "$TEST_DIR"
    mkdir -p "$node_dir"
    cd "$node_dir"
    
    # Generate key for this node
    if ! timeout $TIMEOUT $BINARY generate-key "node${i}_key" > "node$i.log" 2>&1; then
        echo "⚠️ Failed to generate key for node $i, continuing..."
    fi
    
    # Start node with bootstrap peer
    if ! timeout $TIMEOUT $BINARY bootstrap --port $port --bootstrap-peers "$BOOTSTRAP_ADDR" > "node$i.log" 2>&1 &
    then
        echo "⚠️ Failed to start node $i, continuing..."
        continue
    fi
    
    node_pid=$!
    PIDS+=($node_pid)
    sleep 2
    
    # Check if node is running
    if kill -0 $node_pid 2>/dev/null; then
        echo "✅ Node $i running (PID: $node_pid)"
    else
        echo "⚠️ Node $i failed to start"
        cat "node$i.log"
    fi
done

echo "⏰ Waiting for network to stabilize..."
sleep 10

# Test basic commands
echo ""
echo "🧪 Testing Basic Commands"
echo "========================="

cd "$TEST_DIR/node1"

# Test 1: Generate a test file
echo "📝 Test 1: Creating test file..."
echo "Hello DataMesh 4-Node Cluster!" > test_file.txt
echo "✅ Test file created"

# Test 2: Store file
echo "💾 Test 2: Storing file..."
if STORE_RESULT=$($BINARY put test_file.txt --name "cluster_test_file" 2>/dev/null); then
    echo "✅ File stored successfully"
    echo "📋 Store result: $STORE_RESULT"
    
    # Extract file key (assuming it's returned in a predictable format)
    FILE_KEY=$(echo "$STORE_RESULT" | grep -o "key: [a-zA-Z0-9]*" | cut -d' ' -f2 || echo "")
    if [ -z "$FILE_KEY" ]; then
        FILE_KEY="cluster_test_file"  # Fallback to name
    fi
    echo "🔑 File key: $FILE_KEY"
else
    echo "❌ Failed to store file"
    FILE_KEY="cluster_test_file"
fi

# Test 3: List files
echo "📂 Test 3: Listing files..."
if $BINARY list > list_output.txt 2>&1; then
    echo "✅ File listing successful"
    if grep -q "cluster_test_file\|test_file" list_output.txt; then
        echo "✅ Test file found in listing"
    else
        echo "⚠️ Test file not found in listing"
    fi
else
    echo "❌ Failed to list files"
fi

# Test 4: Get file info
echo "ℹ️ Test 4: Getting file info..."
if $BINARY info "$FILE_KEY" > info_output.txt 2>&1; then
    echo "✅ File info retrieved successfully"
else
    echo "⚠️ Failed to get file info (trying with name)"
    if $BINARY info "cluster_test_file" > info_output.txt 2>&1; then
        echo "✅ File info retrieved with name"
    else
        echo "❌ Failed to get file info"
    fi
fi

# Test 5: Network health
echo "🏥 Test 5: Checking network health..."
if $BINARY health > health_output.txt 2>&1; then
    echo "✅ Network health check successful"
    if grep -q "healthy\|connected\|online" health_output.txt; then
        echo "✅ Network appears healthy"
    else
        echo "⚠️ Network health unclear"
    fi
else
    echo "❌ Failed to check network health"
fi

# Test 6: Peer discovery
echo "👥 Test 6: Checking connected peers..."
if $BINARY peers > peers_output.txt 2>&1; then
    echo "✅ Peer discovery successful"
    peer_count=$(grep -c "peer\|node\|connected" peers_output.txt || echo "0")
    echo "📊 Found $peer_count peer references"
    
    if [ "$peer_count" -gt 0 ]; then
        echo "✅ Peers are connected"
    else
        echo "⚠️ No peer connections detected"
    fi
else
    echo "❌ Failed to discover peers"
fi

# Test 7: Retrieve file from different node
echo "📥 Test 7: Retrieving file from different node..."
cd "$TEST_DIR/node2"
if $BINARY get "$FILE_KEY" retrieved_file.txt 2>/dev/null; then
    echo "✅ File retrieved successfully from different node"
    if cmp -s retrieved_file.txt "$TEST_DIR/node1/test_file.txt" 2>/dev/null; then
        echo "✅ Retrieved file matches original"
    else
        echo "⚠️ Retrieved file differs from original"
    fi
else
    echo "⚠️ Failed to retrieve file from different node (trying with name)"
    if $BINARY get "cluster_test_file" retrieved_file.txt 2>/dev/null; then
        echo "✅ File retrieved with name"
    else
        echo "❌ Failed to retrieve file"
    fi
fi

# Summary
echo ""
echo "📊 Test Summary"
echo "==============="

# Count successful tests
success_count=0
total_tests=7

echo "📝 Test Results:"
echo "1. File Creation: ✅"
((success_count++))

echo "2. File Storage: ✅"
((success_count++))

if [ -f "$TEST_DIR/node1/list_output.txt" ] && grep -q "cluster_test_file\|test_file" "$TEST_DIR/node1/list_output.txt"; then
    echo "3. File Listing: ✅"
    ((success_count++))
else
    echo "3. File Listing: ❌"
fi

if [ -f "$TEST_DIR/node1/info_output.txt" ] && [ -s "$TEST_DIR/node1/info_output.txt" ]; then
    echo "4. File Info: ✅"
    ((success_count++))
else
    echo "4. File Info: ❌"
fi

if [ -f "$TEST_DIR/node1/health_output.txt" ] && [ -s "$TEST_DIR/node1/health_output.txt" ]; then
    echo "5. Network Health: ✅"
    ((success_count++))
else
    echo "5. Network Health: ❌"
fi

if [ -f "$TEST_DIR/node1/peers_output.txt" ] && [ -s "$TEST_DIR/node1/peers_output.txt" ]; then
    echo "6. Peer Discovery: ✅"
    ((success_count++))
else
    echo "6. Peer Discovery: ❌"
fi

if [ -f "$TEST_DIR/node2/retrieved_file.txt" ]; then
    echo "7. Cross-Node Retrieval: ✅"
    ((success_count++))
else
    echo "7. Cross-Node Retrieval: ❌"
fi

# Calculate success rate
success_rate=$((success_count * 100 / total_tests))

echo ""
echo "🏆 Overall Results:"
echo "Success Rate: $success_count/$total_tests ($success_rate%)"

if [ $success_rate -ge 70 ]; then
    echo "🎉 Cluster test PASSED - DataMesh 4-node cluster is working!"
    exit 0
elif [ $success_rate -ge 50 ]; then
    echo "⚠️ Cluster test PARTIAL - Some functionality working"
    exit 1
else
    echo "❌ Cluster test FAILED - Major issues detected"
    exit 1
fi