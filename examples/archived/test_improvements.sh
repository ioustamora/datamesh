#!/bin/bash
# Test script to validate DataMesh improvements

set -e

DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }

# Variables
BOOTSTRAP_PID=""
BOOTSTRAP_LOG="/tmp/bootstrap_test.log"
TEST_FILE="/tmp/test_improvements.txt"

cleanup() {
    log_info "Cleaning up..."
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    sleep 1
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    rm -f "$BOOTSTRAP_LOG" "$TEST_FILE" /tmp/test_*.txt 2>/dev/null || true
}

trap cleanup EXIT INT TERM

main() {
    echo "============================================"
    echo "     DataMesh Improvements Test"
    echo "============================================"
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found"
        exit 1
    fi
    
    cleanup
    sleep 1
    
    # Start bootstrap node with logging
    log_info "Starting bootstrap node..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$BOOTSTRAP_LOG" 2>&1 &
    BOOTSTRAP_PID=$!
    
    # Wait for bootstrap to start
    sleep 5
    
    # Check if bootstrap is running
    if ! kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
        log_error "Bootstrap node failed to start"
        cat "$BOOTSTRAP_LOG" 2>/dev/null || echo "No log available"
        exit 1
    fi
    
    # Extract bootstrap info
    local peer_id=""
    local addr=""
    
    if [[ -f "$BOOTSTRAP_LOG" ]]; then
        peer_id=$(grep -E "(Peer ID:|Local peer id:)" "$BOOTSTRAP_LOG" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
        addr=$(grep "Listening on.*$BOOTSTRAP_PORT" "$BOOTSTRAP_LOG" 2>/dev/null | head -1 | grep -o '/ip4/[^[:space:]]*' || true)
    fi
    
    if [[ -n "$peer_id" && -n "$addr" ]]; then
        log_success "Bootstrap node started successfully"
        log_info "Peer ID: $peer_id"
        log_info "Address: $addr"
    else
        log_error "Could not extract bootstrap information"
        log_info "Bootstrap log:"
        cat "$BOOTSTRAP_LOG" 2>/dev/null || echo "No log available"
        exit 1
    fi
    
    # Test file operations
    echo "Test file content - $(date)" > "$TEST_FILE"
    
    log_info "Testing file storage..."
    local store_result
    if store_result=$("$DATAMESH_BINARY" \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "$addr" \
        --non-interactive \
        put "$TEST_FILE" 2>&1); then
        
        local file_key=$(echo "$store_result" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        if [[ -n "$file_key" ]]; then
            log_success "File stored successfully with key: ${file_key:0:16}..."
            
            # Test retrieval
            log_info "Testing file retrieval..."
            if "$DATAMESH_BINARY" \
                --bootstrap-peer "$peer_id" \
                --bootstrap-addr "$addr" \
                --non-interactive \
                get "$file_key" "/tmp/retrieved_test.txt" 2>/dev/null; then
                
                if cmp -s "$TEST_FILE" "/tmp/retrieved_test.txt"; then
                    log_success "🎉 File retrieval successful!"
                    log_success "✅ DataMesh improvements working correctly!"
                    return 0
                else
                    log_error "File content mismatch"
                fi
            else
                log_error "File retrieval failed"
            fi
        else
            log_error "Could not extract file key"
        fi
    else
        log_error "File storage failed"
        echo "$store_result"
    fi
    
    return 1
}

main "$@"