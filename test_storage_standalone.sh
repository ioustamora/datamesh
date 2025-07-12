#!/bin/bash
# Standalone Storage Test - Test storage on a single node

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
    pkill -f "datamesh" 2>/dev/null || true
    sleep 2
    rm -f /tmp/standalone_*.txt /tmp/bootstrap_standalone.log
}

trap cleanup EXIT

main() {
    log_info "🧪 Standalone Storage Test"
    echo "=========================="
    
    cleanup
    
    # Start bootstrap node and keep it running
    log_info "Starting bootstrap node on port 31000..."
    ./target/release/datamesh --non-interactive bootstrap --port 31000 > /tmp/bootstrap_standalone.log 2>&1 &
    local bootstrap_pid=$!
    
    # Wait longer for full initialization
    sleep 15
    
    # Extract peer ID
    local peer_id=""
    local attempts=0
    while [[ $attempts -lt 15 && -z "$peer_id" ]]; do
        peer_id=$(grep -o "Peer ID: [A-Za-z0-9]\+" /tmp/bootstrap_standalone.log 2>/dev/null | sed 's/Peer ID: //' || echo "")
        if [[ -z "$peer_id" ]]; then
            sleep 1
            ((attempts++))
        fi
    done
    
    if [[ -z "$peer_id" ]]; then
        log_error "Failed to get bootstrap peer ID"
        cat /tmp/bootstrap_standalone.log
        return 1
    fi
    
    log_info "Bootstrap peer ID: $peer_id"
    
    # Create test files
    echo "Standalone test content $(date)" > /tmp/standalone_test.txt
    dd if=/dev/urandom of=/tmp/standalone_large.txt bs=1024 count=10 2>/dev/null
    
    # Test 1: Connect and check stats first
    log_info "Testing network connectivity..."
    if ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/31000" \
        stats > /tmp/stats_test.txt 2>&1; then
        
        log_success "Stats command worked!"
        local connected_peers=$(grep -o "Connected peers: [0-9]\+" /tmp/stats_test.txt | grep -o "[0-9]\+" || echo "0")
        log_info "Connected peers: $connected_peers"
    else
        log_error "Stats command failed"
        cat /tmp/stats_test.txt
    fi
    
    # Test 2: Try storage with timeout and more waiting
    log_info "Testing file storage (with network warm-up)..."
    
    # Give the network time to discover peers
    sleep 5
    
    if timeout 30s ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/31000" \
        put /tmp/standalone_test.txt > /tmp/store_standalone.txt 2>&1; then
        
        log_success "Storage command completed! 🎉"
        
        # Extract key if available
        local key=$(grep -oE '[a-f0-9]{32,}' /tmp/store_standalone.txt | head -1)
        if [[ -n "$key" ]]; then
            log_info "File key: ${key:0:20}..."
            
            # Try to retrieve
            log_info "Testing retrieval..."
            if timeout 20s ./target/release/datamesh --non-interactive \
                --bootstrap-peer "$peer_id" \
                --bootstrap-addr "/ip4/127.0.0.1/tcp/31000" \
                get "$key" /tmp/standalone_retrieved.txt 2>&1; then
                
                if cmp -s /tmp/standalone_test.txt /tmp/standalone_retrieved.txt; then
                    log_success "✨ FULL TEST PASSED! Storage and retrieval working!"
                    
                    # Test large file
                    log_info "Testing large file storage..."
                    if timeout 60s ./target/release/datamesh --non-interactive \
                        --bootstrap-peer "$peer_id" \
                        --bootstrap-addr "/ip4/127.0.0.1/tcp/31000" \
                        put /tmp/standalone_large.txt > /tmp/store_large_standalone.txt 2>&1; then
                        
                        local large_key=$(grep -oE '[a-f0-9]{32,}' /tmp/store_large_standalone.txt | head -1)
                        if [[ -n "$large_key" ]]; then
                            log_success "Large file stored! Key: ${large_key:0:20}..."
                        fi
                    fi
                    
                else
                    log_error "File content mismatch"
                fi
            else
                log_error "Retrieval failed or timed out"
            fi
        else
            log_info "No key found in output, checking for errors..."
            echo "Store output:"
            cat /tmp/store_standalone.txt
        fi
        
    else
        log_error "Storage failed or timed out"
        echo "Error output:"
        cat /tmp/store_standalone.txt
        return 1
    fi
    
    # Keep bootstrap running for stability test
    log_info "Testing network stability (keeping node running for 10 seconds)..."
    sleep 10
    
    # Final stats
    if ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/31000" \
        stats > /tmp/final_stats.txt 2>&1; then
        
        log_info "Final network stats:"
        cat /tmp/final_stats.txt
    fi
    
    kill $bootstrap_pid 2>/dev/null || true
    log_success "Test completed!"
}

main "$@"