#!/bin/bash
# Quick DataMesh Cluster Test - Simplified version for immediate verification

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
TEST_DIR="quick_test_$(date +%H%M%S)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} ⚠️ $1"; }

# Variables
CLUSTER_PIDS=()
BOOTSTRAP_PID=""
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    for pid in "${CLUSTER_PIDS[@]}"; do
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    rm -rf "$TEST_DIR" 2>/dev/null || true
    rm -f /tmp/quick_test_* 2>/dev/null || true
}

trap cleanup EXIT INT TERM

# Main test function
main() {
    echo "=================================================="
    echo "         DataMesh Quick Cluster Test"
    echo "=================================================="
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found. Run: cargo build --release"
        exit 1
    fi
    
    # Clean up any existing processes
    pkill -f datamesh 2>/dev/null || true
    sleep 1
    
    # Setup test directory
    mkdir -p "$TEST_DIR/logs"
    
    log_info "Starting bootstrap node..."
    
    # Start bootstrap node
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" \
        > "$TEST_DIR/logs/bootstrap.log" 2>&1 &
    BOOTSTRAP_PID=$!
    CLUSTER_PIDS+=("$BOOTSTRAP_PID")
    
    # Wait for bootstrap to start
    sleep 5
    
    # Extract bootstrap info
    if [[ -f "$TEST_DIR/logs/bootstrap.log" ]]; then
        BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/logs/bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
        BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/logs/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
    fi
    
    if [[ -z "$BOOTSTRAP_PEER_ID" || -z "$BOOTSTRAP_ADDR" ]]; then
        log_error "Failed to start bootstrap node or extract connection info"
        log_info "Bootstrap log contents:"
        cat "$TEST_DIR/logs/bootstrap.log" 2>/dev/null || echo "No log file found"
        exit 1
    fi
    
    log_success "Bootstrap node started successfully"
    log_info "Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "Address: $BOOTSTRAP_ADDR"
    
    # Start one regular node
    log_info "Starting regular node..."
    "$DATAMESH_BINARY" --non-interactive service \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port 40872 \
        > "$TEST_DIR/logs/node1.log" 2>&1 &
    
    local node_pid=$!
    CLUSTER_PIDS+=("$node_pid")
    
    sleep 3
    
    if kill -0 "$node_pid" 2>/dev/null; then
        log_success "Regular node started (PID: $node_pid)"
    else
        log_error "Regular node failed to start"
        exit 1
    fi
    
    # Wait for network to stabilize
    log_info "Waiting for network to stabilize..."
    sleep 5
    
    # Simple connectivity test
    log_info "Testing basic connectivity..."
    
    # Try to list files (should work even if empty)
    if timeout 10 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        list &>/dev/null; then
        log_success "Basic connectivity test PASSED"
    else
        log_error "Basic connectivity test FAILED"
        log_info "Showing recent logs..."
        echo "Bootstrap log (last 10 lines):"
        tail -10 "$TEST_DIR/logs/bootstrap.log" 2>/dev/null || echo "No bootstrap log"
        echo ""
        echo "Node log (last 10 lines):"
        tail -10 "$TEST_DIR/logs/node1.log" 2>/dev/null || echo "No node log"
        exit 1
    fi
    
    # Simple file test
    log_info "Testing file operations..."
    
    # Create a small test file
    local test_file="/tmp/quick_test_file.txt"
    echo "Quick test content - $(date)" > "$test_file"
    
    # Try to store the file with timeout
    log_info "Storing test file..."
    local store_output
    if store_output=$(timeout 15 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$test_file" 2>&1); then
        
        local key=$(echo "$store_output" | grep -E "(stored with key|File stored)" | grep -oE '[a-f0-9]{32,}' | head -1)
        if [[ -n "$key" ]]; then
            log_success "File stored successfully with key: ${key:0:32}..."
            
            # Try to retrieve the file
            log_info "Retrieving test file..."
            local retrieved_file="/tmp/quick_test_retrieved.txt"
            if timeout 15 "$DATAMESH_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                get "$key" "$retrieved_file" &>/dev/null; then
                
                if cmp -s "$test_file" "$retrieved_file"; then
                    log_success "File retrieval test PASSED"
                    log_success "🎉 All tests completed successfully!"
                else
                    log_error "Retrieved file differs from original"
                    exit 1
                fi
            else
                log_error "Failed to retrieve file"
                exit 1
            fi
        else
            log_error "Could not extract file key from store output"
            log_info "Store output: $store_output"
            exit 1
        fi
    else
        log_error "Failed to store file"
        log_info "Store output: $store_output"
        exit 1
    fi
    
    echo ""
    log_success "Quick cluster test completed successfully! 🎉"
    echo ""
    log_info "Cluster information:"
    echo "  Bootstrap Peer ID: $BOOTSTRAP_PEER_ID"
    echo "  Bootstrap Address: $BOOTSTRAP_ADDR"
    echo "  Log directory: $TEST_DIR/logs"
    echo ""
    log_info "Press Enter to stop the cluster..."
    read -r
}

main "$@"