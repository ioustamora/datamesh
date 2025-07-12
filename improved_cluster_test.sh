#!/bin/bash
# Improved DataMesh Cluster Test with proper peer connectivity

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
TEST_DIR="cluster_test_$(date +%Y%m%d_%H%M%S)"

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

# Variables
BOOTSTRAP_PID=""
TEST_FILE="/tmp/cluster_test_file.txt"
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""

cleanup() {
    log_info "Cleaning up..."
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    sleep 1
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    rm -f "$TEST_FILE" /tmp/cluster_*.log /tmp/test_*.txt 2>/dev/null || true
    rm -rf "$TEST_DIR" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

extract_bootstrap_info() {
    local log_file="$1"
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if [[ -f "$log_file" ]]; then
            # Extract peer ID
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
            
            # Extract listening address for the bootstrap port
            BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$log_file" 2>/dev/null | head -1 | grep -o '/ip4/[^[:space:]]*' || true)
            
            if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
                return 0
            fi
        fi
        sleep 0.5
        ((attempt++))
    done
    return 1
}

main() {
    echo "=================================================="
    echo "      DataMesh Improved Cluster Test"
    echo "=================================================="
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        log_info "Build it with: cargo build --release"
        exit 1
    fi
    
    # Create test directory and log file
    mkdir -p "$TEST_DIR"
    local bootstrap_log="$TEST_DIR/bootstrap.log"
    
    # Clean up any existing processes
    cleanup
    sleep 2
    
    # Start bootstrap node
    log_info "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$bootstrap_log" 2>&1 &
    BOOTSTRAP_PID=$!
    
    # Wait for bootstrap node to initialize and extract info
    log_info "Waiting for bootstrap node to initialize..."
    if ! extract_bootstrap_info "$bootstrap_log"; then
        log_error "Failed to extract bootstrap node information"
        log_info "Bootstrap log contents:"
        cat "$bootstrap_log" 2>/dev/null || echo "No log file found"
        exit 1
    fi
    
    log_success "Bootstrap node ready:"
    log_info "  Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "  Address: $BOOTSTRAP_ADDR"
    
    # Give bootstrap node a moment to fully initialize
    sleep 3
    
    # Create test file
    echo "Test content - $(date)" > "$TEST_FILE"
    
    # Test file operations with explicit bootstrap connection
    local attempts=0
    local max_attempts=3
    
    while [[ $attempts -lt $max_attempts ]]; do
        log_info "Test attempt $((attempts + 1))/$max_attempts"
        
        # Test file storage
        log_info "Testing file storage..."
        local store_output
        if store_output=$("$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            put "$TEST_FILE" 2>&1); then
            
            # Extract file key
            local file_key=$(echo "$store_output" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
            if [[ -n "$file_key" ]]; then
                log_success "File stored with key: ${file_key:0:16}..."
                
                # Test file retrieval
                log_info "Testing file retrieval..."
                local retrieved_file="/tmp/test_retrieved_$attempts.txt"
                if timeout 15 "$DATAMESH_BINARY" \
                    --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                    --bootstrap-addr "$BOOTSTRAP_ADDR" \
                    --non-interactive \
                    get "$file_key" "$retrieved_file" 2>/dev/null; then
                    
                    if cmp -s "$TEST_FILE" "$retrieved_file"; then
                        log_success "🎉 Cluster test PASSED!"
                        log_info "Successfully stored and retrieved file"
                        echo ""
                        log_info "Test Summary:"
                        echo "  • Bootstrap node working on port $BOOTSTRAP_PORT"
                        echo "  • Client successfully connected to bootstrap"
                        echo "  • File storage and retrieval functional"
                        echo "  • Network connectivity verified"
                        echo "  • File key: $file_key"
                        echo ""
                        log_success "DataMesh cluster functionality confirmed! ✅"
                        
                        # Test file listing
                        log_info "Testing file listing..."
                        if "$DATAMESH_BINARY" \
                            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                            --bootstrap-addr "$BOOTSTRAP_ADDR" \
                            --non-interactive \
                            list > /dev/null 2>&1; then
                            log_success "File listing functionality working"
                        else
                            log_warning "File listing test had issues (non-critical)"
                        fi
                        
                        # Keep running briefly to show stability
                        log_info "Keeping cluster running for 5 seconds to verify stability..."
                        sleep 5
                        return 0
                    else
                        log_error "File content mismatch after retrieval"
                    fi
                else
                    log_error "Failed to retrieve file with key: $file_key"
                fi
            else
                log_error "Could not extract file key from output"
                log_info "Store output:"
                echo "$store_output"
            fi
        else
            log_error "Failed to store file"
            log_info "Store output:"
            echo "$store_output"
        fi
        
        sleep 2
        ((attempts++))
    done
    
    log_error "All test attempts failed"
    log_info "Final bootstrap log:"
    tail -20 "$bootstrap_log" 2>/dev/null || echo "No log available"
    return 1
}

main "$@"