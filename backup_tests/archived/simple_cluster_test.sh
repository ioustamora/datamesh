#!/bin/bash
# Simple Working DataMesh Cluster Test

set -e

# Configuration
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
NODE_PID=""

cleanup() {
    log_info "Cleaning up..."
    [[ -n "$NODE_PID" ]] && kill "$NODE_PID" 2>/dev/null || true
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    sleep 1
    pkill -f datamesh 2>/dev/null || true
    rm -f /tmp/simple_test_* 2>/dev/null || true
}

trap cleanup EXIT INT TERM

main() {
    echo "=================================================="
    echo "      DataMesh Simple Cluster Test"
    echo "=================================================="
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found"
        exit 1
    fi
    
    # Clean up any existing processes
    cleanup
    sleep 2
    
    # Start bootstrap node
    log_info "Starting bootstrap node..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" &
    BOOTSTRAP_PID=$!
    
    # Wait for bootstrap to start
    sleep 8
    
    # Check if bootstrap is running
    if ! kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
        log_error "Bootstrap node failed to start"
        exit 1
    fi
    
    log_success "Bootstrap node is running (PID: $BOOTSTRAP_PID)"
    
    # Start a regular node using correct syntax
    log_info "Starting service node..."
    
    # Use correct command syntax: service subcommand with bootstrap peers
    "$DATAMESH_BINARY" --non-interactive \
        --bootstrap-peers "12D3KooWDummyPeerIdForTesting@/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT" \
        service --port $((BOOTSTRAP_PORT + 1)) &
    NODE_PID=$!
    
    sleep 5
    
    # Check if node is running
    if ! kill -0 "$NODE_PID" 2>/dev/null; then
        log_error "Service node failed to start"
        exit 1
    fi
    
    log_success "Service node is running (PID: $NODE_PID)"
    
    # Test basic functionality
    log_info "Testing cluster functionality..."
    
    # Create test file
    echo "Simple cluster test - $(date)" > /tmp/simple_test_file.txt
    echo "Random: $RANDOM" >> /tmp/simple_test_file.txt
    
    # Test with the running cluster
    log_info "Attempting file storage..."
    
    # Try basic storage using the local network preset
    if timeout 15 "$DATAMESH_BINARY" --non-interactive --network local put /tmp/simple_test_file.txt > /tmp/simple_store_output.txt 2>&1; then
        
        # Extract key
        local key=$(grep -oE '[a-f0-9]{32,}' /tmp/simple_store_output.txt | head -1)
        if [[ -n "$key" ]]; then
            log_success "File stored with key: ${key:0:16}..."
            
            # Try to retrieve
            if timeout 15 "$DATAMESH_BINARY" --non-interactive --network local get "$key" /tmp/simple_test_retrieved.txt &>/dev/null; then
                
                if cmp -s /tmp/simple_test_file.txt /tmp/simple_test_retrieved.txt; then
                    log_success "🎉 CLUSTER TEST PASSED!"
                    echo ""
                    log_info "Summary:"
                    echo "  • Bootstrap node: Running on port $BOOTSTRAP_PORT"
                    echo "  • Service node: Running on port $((BOOTSTRAP_PORT + 1))"
                    echo "  • File operations: Working"
                    echo "  • File key: ${key:0:32}..."
                    echo ""
                    log_success "DataMesh cluster is functional! ✅"
                    
                    # Keep running briefly to show stability
                    log_info "Keeping cluster running for 3 seconds..."
                    sleep 3
                    return 0
                else
                    log_error "File content mismatch"
                fi
            else
                log_error "Failed to retrieve file"
            fi
        else
            log_error "Could not extract file key"
            log_info "Store output:"
            cat /tmp/simple_store_output.txt
        fi
    else
        log_error "Failed to store file"
        log_info "Store output:"
        cat /tmp/simple_store_output.txt 2>/dev/null || echo "No output"
    fi
    
    log_error "Cluster test failed"
    return 1
}

main "$@"