#!/bin/bash
# Test script for quorum fix verification

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
SERVICE_PORTS=(40872 40873 40874)
TEST_DIR="quorum_test_$(date +%Y%m%d_%H%M%S)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }

echo "=================================================================="
echo "    🔧 TESTING QUORUM FIX FOR DATAMESH CLUSTER"
echo "=================================================================="

# Rebuild with the fix
log_info "🔨 Building DataMesh with quorum fixes..."
if cargo build --release; then
    log_success "Build completed successfully"
else
    log_error "Build failed"
    exit 1
fi

# Clean up any existing processes
log_info "🧹 Cleaning up existing processes..."
pkill -f "datamesh.*bootstrap" 2>/dev/null || true
pkill -f "datamesh.*service" 2>/dev/null || true
sleep 2

mkdir -p "$TEST_DIR"

# Start bootstrap node
log_info "📡 Starting bootstrap node..."
"$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$TEST_DIR/bootstrap.log" 2>&1 &
BOOTSTRAP_PID=$!
sleep 5

# Extract bootstrap info
BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/bootstrap.log" | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' || true)

if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
    log_success "Bootstrap ready: $BOOTSTRAP_PEER_ID"
else
    log_error "Failed to start bootstrap"
    exit 1
fi

# Start service nodes
declare -A NODE_PIDS
for port in "${SERVICE_PORTS[@]}"; do
    log_info "🎯 Starting service node on port $port..."
    "$DATAMESH_BINARY" --non-interactive service \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$port" > "$TEST_DIR/service_$port.log" 2>&1 &
    NODE_PIDS[$port]=$!
    sleep 3
done

log_success "🎉 Cluster started: 1 bootstrap + ${#SERVICE_PORTS[@]} service nodes"

# Wait for network stabilization
log_info "⏳ Waiting for network stabilization (20 seconds)..."
sleep 20

# Test storage with quorum fix
log_info "🧪 Testing storage with intelligent quorum calculation..."

base_cmd="$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --non-interactive"

# Create test file
echo "Quorum fix test - $(date)" > /tmp/quorum_test.txt

log_info "📤 Testing PUT command with improved quorum logic..."
echo "Command: $base_cmd --port 41001 put /tmp/quorum_test.txt"

if store_output=$($base_cmd --port 41001 put /tmp/quorum_test.txt 2>&1); then
    log_success "✅ PUT command successful with quorum fix!"
    echo "$store_output"
    
    # Try to extract file key
    file_key=$(echo "$store_output" | grep -oE '(stored with key:|Key:) [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
    if [[ -n "$file_key" ]]; then
        log_success "📝 File key extracted: ${file_key:0:20}..."
        
        log_info "📥 Testing GET command..."
        if $base_cmd --port 41002 get "$file_key" /tmp/quorum_test_retrieved.txt 2>&1; then
            if cmp -s /tmp/quorum_test.txt /tmp/quorum_test_retrieved.txt; then
                log_success "✅ GET command successful - content matches!"
                
                echo ""
                echo "=================================================================="
                echo "    🎯 QUORUM FIX VERIFICATION SUCCESSFUL! 🎯"
                echo "=================================================================="
                echo "✅ PUT operations now work with intelligent quorum calculation"
                echo "✅ GET operations work correctly"
                echo "✅ File integrity maintained"
                echo "✅ Network mesh connectivity functioning"
                echo ""
                echo "📈 Improvement: QuorumFailed errors resolved!"
                echo "🔧 Fix: Dynamic quorum calculation based on network size"
                echo ""
                
            else
                log_error "❌ Content mismatch after retrieval"
            fi
        else
            log_error "❌ GET command failed"
        fi
    else
        log_error "❌ Could not extract file key"
    fi
else
    log_error "❌ PUT command still failing"
    echo "Output:"
    echo "$store_output"
    
    echo ""
    echo "=================================================================="
    echo "    ⚠️  QUORUM FIX NEEDS FURTHER INVESTIGATION"
    echo "=================================================================="
    echo "❌ PUT operations still experiencing issues"
    echo "🔍 Check logs in: $TEST_DIR/"
    echo ""
fi

# Show network connectivity analysis
log_info "📊 Network connectivity analysis:"
echo ""
echo "Connected peers in bootstrap:"
grep -c "Connected to peer" "$TEST_DIR/bootstrap.log" 2>/dev/null || echo "0"

for port in "${SERVICE_PORTS[@]}"; do
    echo "Connected peers in service $port:"
    grep -c "Connected to peer" "$TEST_DIR/service_$port.log" 2>/dev/null || echo "0"
done

# Cleanup
log_info "🧹 Cleaning up..."
kill $BOOTSTRAP_PID 2>/dev/null || true
for port in "${SERVICE_PORTS[@]}"; do
    kill ${NODE_PIDS[$port]} 2>/dev/null || true
done

sleep 3
pkill -f "datamesh" 2>/dev/null || true
rm -f /tmp/quorum_test* 2>/dev/null || true

log_success "🎯 Quorum fix test completed!"
echo "📂 Logs saved in: $TEST_DIR"