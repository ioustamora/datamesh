#!/bin/bash
# DHT Diagnostic Test - Understand what's happening with peer discovery

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }

cleanup() {
    log_info "Cleaning up..."
    pkill -f "datamesh" 2>/dev/null || true
    sleep 2
}

trap cleanup EXIT

main() {
    log_info "🔍 DHT Diagnostic Test"
    echo "====================="
    
    cleanup
    
    # Start bootstrap node and keep logs
    log_info "Starting bootstrap node..."
    ./target/release/datamesh --non-interactive bootstrap --port 33000 > bootstrap_diag.log 2>&1 &
    local bootstrap_pid=$!
    
    sleep 15
    
    # Extract peer ID
    local peer_id=$(grep -o "Peer ID: [A-Za-z0-9]\+" bootstrap_diag.log | sed 's/Peer ID: //')
    log_info "Bootstrap peer ID: $peer_id"
    
    # Show bootstrap logs
    log_info "Bootstrap node logs (last 10 lines):"
    tail -10 bootstrap_diag.log
    echo ""
    
    # Start one additional node
    log_info "Starting additional node..."
    ./target/release/datamesh --non-interactive join --port 33001 --bootstrap "/ip4/127.0.0.1/tcp/33000" > node_diag.log 2>&1 &
    local node_pid=$!
    
    sleep 10
    
    # Show node logs
    log_info "Additional node logs (last 10 lines):"
    tail -10 node_diag.log
    echo ""
    
    # Test network stats
    log_info "Testing network stats..."
    ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/33000" \
        stats > stats_diag.txt 2>&1
    
    log_info "Network stats output:"
    cat stats_diag.txt
    echo ""
    
    # Test peers command
    log_info "Testing peers command..."
    ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/33000" \
        peers > peers_diag.txt 2>&1
    
    log_info "Peers command output:"
    cat peers_diag.txt
    echo ""
    
    # Try simple storage with debug
    log_info "Attempting storage with debug output..."
    echo "diagnostic test" > test_diag.txt
    
    RUST_LOG=debug ./target/release/datamesh --non-interactive \
        --bootstrap-peer "$peer_id" \
        --bootstrap-addr "/ip4/127.0.0.1/tcp/33000" \
        put test_diag.txt > storage_diag.txt 2>&1 || true
    
    log_info "Storage attempt output (last 20 lines):"
    tail -20 storage_diag.txt
    echo ""
    
    # Keep running for a bit to show what logs accumulate
    log_info "Letting cluster run for 20 seconds to see ongoing activity..."
    sleep 20
    
    log_info "Final bootstrap logs (last 15 lines):"
    tail -15 bootstrap_diag.log
    echo ""
    
    log_info "Final node logs (last 15 lines):"
    tail -15 node_diag.log
    
    kill $bootstrap_pid $node_pid 2>/dev/null || true
    
    log_info "Diagnostic complete - log files preserved for analysis"
}

main "$@"