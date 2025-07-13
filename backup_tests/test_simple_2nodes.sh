#!/bin/bash
# Simple 2-Node Test for DataMesh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }

# Clean up function
cleanup() {
    log_info "Cleaning up..."
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    sleep 2
    rm -f /tmp/simple_test.txt /tmp/simple_retrieved.txt /tmp/bootstrap_simple.log
}

trap cleanup EXIT

main() {
    log_info "🧪 Simple 2-Node DataMesh Test"
    echo "=============================="
    
    cleanup
    
    # Start bootstrap node
    log_info "Starting bootstrap node on port 30000..."
    ./target/release/datamesh --non-interactive bootstrap --port 30000 > /tmp/bootstrap_simple.log 2>&1 &
    local bootstrap_pid=$!
    
    # Wait for bootstrap to start
    sleep 8
    
    # Extract peer ID
    local peer_id=""
    local attempts=0
    while [[ $attempts -lt 10 && -z "$peer_id" ]]; do
        peer_id=$(grep -o "Peer ID: [A-Za-z0-9]\+" /tmp/bootstrap_simple.log 2>/dev/null | sed 's/Peer ID: //' || echo "")
        if [[ -z "$peer_id" ]]; then
            sleep 1
            ((attempts++))
        fi
    done
    
    if [[ -z "$peer_id" ]]; then
        log_error "Failed to get bootstrap peer ID"
        cat /tmp/bootstrap_simple.log
        return 1
    fi
    
    log_info "Bootstrap peer ID: $peer_id"
    
    # Test basic storage with debug logs - use the actual connected peer ID from bootstrap logs
    echo "Simple test content $(date)" > /tmp/simple_test.txt
    
    # Wait for any peer connections to show up in bootstrap logs
    sleep 3
    
    # Try to extract actual connected peer ID from logs
    local actual_peer=""
    if grep -q "Connection established" /tmp/bootstrap_simple.log; then
        actual_peer=$(grep "Connection established" /tmp/bootstrap_simple.log | tail -1 | grep -oE '12D3[A-Za-z0-9]+')
    fi
    
    if [[ -n "$actual_peer" ]]; then
        log_info "Using actual connected peer: $actual_peer"
        peer_id="$actual_peer"
    fi
    
    log_info "Testing storage with debug logging..."
    if RUST_LOG=debug ./target/release/datamesh --non-interactive \
        --bootstrap-peers "${peer_id}@/ip4/127.0.0.1/tcp/30000" \
        put /tmp/simple_test.txt > /tmp/store_simple.txt 2>&1; then
        
        log_success "Storage command completed! 🎉"
        echo "Last 10 lines of output:"
        tail -10 /tmp/store_simple.txt
        
        # Extract key if available
        local key=$(grep -oE '[a-f0-9]{32,}' /tmp/store_simple.txt | head -1)
        if [[ -n "$key" ]]; then
            log_info "File key: ${key:0:20}..."
            
            # Try to retrieve
            log_info "Testing retrieval..."
            if ./target/release/datamesh --non-interactive \
                --bootstrap-peers "${peer_id}@/ip4/127.0.0.1/tcp/30000" \
                get "$key" /tmp/simple_retrieved.txt &>/dev/null; then
                
                if cmp -s /tmp/simple_test.txt /tmp/simple_retrieved.txt; then
                    log_success "✨ FULL TEST PASSED! Storage and retrieval working!"
                else
                    log_error "File content mismatch"
                fi
            else
                log_error "Retrieval failed"
            fi
        else
            log_info "No key found, but command completed"
        fi
    else
        log_error "Storage failed"
        echo "Error output:"
        cat /tmp/store_simple.txt
        return 1
    fi
    
    # Keep bootstrap running for a moment
    sleep 2
    kill $bootstrap_pid 2>/dev/null || true
}

main "$@"