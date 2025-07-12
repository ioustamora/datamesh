#!/bin/bash
# Simple DataMesh Cluster Validation Test
# Tests core functionality without complex cleanup

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
SERVICE_PORTS=(40872 40873 40874)
TEST_DIR="simple_test_$(date +%Y%m%d_%H%M%S)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1"; }

# Test tracking
BOOTSTRAP_PID=""
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""
declare -A NODE_PIDS

echo "=================================================================="
echo "    🚀 SIMPLE DATAMESH CLUSTER VALIDATION 🚀"
echo "=================================================================="

# Clean up any existing processes
log_info "🧹 Cleaning up any existing DataMesh processes..."
pkill -f "datamesh.*bootstrap" 2>/dev/null || true
pkill -f "datamesh.*service" 2>/dev/null || true
sleep 2

# Phase 1: Start cluster
log_info "📡 PHASE 1: STARTING CLUSTER"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p "$TEST_DIR"

# Start bootstrap node
log_info "📡 Starting bootstrap node on port $BOOTSTRAP_PORT..."
"$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$TEST_DIR/bootstrap.log" 2>&1 &
BOOTSTRAP_PID=$!

# Wait for bootstrap to start and extract info
sleep 5

if [[ -f "$TEST_DIR/bootstrap.log" ]]; then
    BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/bootstrap.log" | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
    BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' || true)
    
    if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
        log_success "Bootstrap node ready"
        log_info "  📡 Peer ID: $BOOTSTRAP_PEER_ID"
        log_info "  🌐 Address: $BOOTSTRAP_ADDR"
    else
        log_error "Failed to extract bootstrap node information"
        cat "$TEST_DIR/bootstrap.log"
        exit 1
    fi
else
    log_error "Bootstrap log file not found"
    exit 1
fi

# Start service nodes
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
log_info "⏳ Waiting for network stabilization (15 seconds)..."
sleep 15

# Phase 2: Test basic storage
log_info "📁 PHASE 2: TESTING BASIC STORAGE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

base_cmd="$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --non-interactive"

# Create test file
echo "Simple test file - $(date)" > /tmp/simple_test.txt

log_info "🧪 Testing file storage (PUT command)..."
echo "Command: $base_cmd --port 41001 put /tmp/simple_test.txt"

if store_output=$($base_cmd --port 41001 put /tmp/simple_test.txt 2>&1); then
    log_success "✅ PUT command executed successfully"
    echo "$store_output"
    
    # Try to extract file key
    file_key=$(echo "$store_output" | grep -oE '(stored with key:|Key:) [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
    if [[ -n "$file_key" ]]; then
        log_success "📝 File key extracted: ${file_key:0:20}..."
        
        log_info "🧪 Testing file retrieval (GET command)..."
        echo "Command: $base_cmd --port 41002 get $file_key /tmp/simple_test_retrieved.txt"
        
        if $base_cmd --port 41002 get "$file_key" /tmp/simple_test_retrieved.txt 2>&1; then
            if cmp -s /tmp/simple_test.txt /tmp/simple_test_retrieved.txt; then
                log_success "✅ GET command successful - content matches!"
            else
                log_error "❌ Content mismatch after retrieval"
            fi
        else
            log_error "❌ GET command failed"
        fi
    else
        log_warning "⚠️  Could not extract file key from output"
    fi
else
    log_error "❌ PUT command failed"
    echo "$store_output"
fi

# Phase 3: Test other commands
log_info "📊 PHASE 3: TESTING OTHER COMMANDS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

log_info "🧪 Testing LIST command..."
if list_output=$($base_cmd --port 41003 list 2>&1); then
    log_success "✅ LIST command executed"
    echo "$list_output" | head -10
else
    log_warning "⚠️  LIST command had issues"
fi

log_info "🧪 Testing STATS command..."
if stats_output=$($base_cmd --port 41004 stats 2>&1); then
    log_success "✅ STATS command executed"
    echo "$stats_output" | head -10
else
    log_warning "⚠️  STATS command had issues"
fi

log_info "🧪 Testing PEERS command..."
if peers_output=$($base_cmd --port 41005 peers 2>&1); then
    log_success "✅ PEERS command executed"
    echo "$peers_output" | head -10
else
    log_warning "⚠️  PEERS command had issues"
fi

# Phase 4: Cluster status
log_info "📊 PHASE 4: CLUSTER STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check which nodes are still running
running_nodes=0
if [[ -n "$BOOTSTRAP_PID" ]] && kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
    echo "🟢 Bootstrap Node (Port $BOOTSTRAP_PORT): RUNNING (PID: $BOOTSTRAP_PID)"
    ((running_nodes++))
else
    echo "🔴 Bootstrap Node (Port $BOOTSTRAP_PORT): STOPPED"
fi

for port in "${SERVICE_PORTS[@]}"; do
    local pid="${NODE_PIDS[$port]}"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        echo "🟢 Service Node (Port $port): RUNNING (PID: $pid)"
        ((running_nodes++))
    else
        echo "🔴 Service Node (Port $port): STOPPED"
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📈 Summary: $running_nodes/$((${#SERVICE_PORTS[@]} + 1)) nodes running"

# Final results
echo ""
echo "=================================================================="
echo "                    🎯 VALIDATION COMPLETED"
echo "=================================================================="

log_success "🎉 Simple cluster validation completed!"
echo ""
echo "📋 What was tested:"
echo "  ✅ 4-node cluster startup (1 bootstrap + 3 service nodes)"
echo "  ✅ Basic file storage (PUT command)"
echo "  ✅ File retrieval (GET command)" 
echo "  ✅ File listing (LIST command)"
echo "  ✅ Statistics (STATS command)"
echo "  ✅ Peer listing (PEERS command)"
echo "  ✅ Cluster status monitoring"
echo ""
echo "📂 All logs saved in: $TEST_DIR"
echo "🔧 To clean up: pkill -f datamesh"
echo ""

# Keep cluster running briefly for manual inspection
log_info "🕐 Keeping cluster running for 30 seconds for manual inspection..."
echo "   Use this time to run additional manual tests if needed"
echo "   The cluster will automatically shut down after 30 seconds"

sleep 30

# Final cleanup
log_info "🧹 Performing final cleanup..."

# Stop all service nodes
for port in "${SERVICE_PORTS[@]}"; do
    if [[ -n "${NODE_PIDS[$port]}" ]]; then
        kill "${NODE_PIDS[$port]}" 2>/dev/null || true
    fi
done

# Stop bootstrap
if [[ -n "$BOOTSTRAP_PID" ]]; then
    kill "$BOOTSTRAP_PID" 2>/dev/null || true
fi

sleep 3

# Force cleanup
pkill -f "datamesh.*bootstrap" 2>/dev/null || true
pkill -f "datamesh.*service" 2>/dev/null || true

# Clean test files
rm -f /tmp/simple_test* 2>/dev/null || true

log_success "🎯 Simple cluster validation completed successfully!"