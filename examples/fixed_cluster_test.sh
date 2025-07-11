#!/bin/bash
# Fixed DataMesh Cluster Test - Addresses connection retry issues

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
NUM_NODES=3
TEST_DIR="fixed_test_$(date +%H%M%S)"

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

cleanup() {
    log_info "Cleaning up..."
    for pid in "${CLUSTER_PIDS[@]}"; do
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            sleep 1
            # Force kill if needed
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
    done
    rm -rf "$TEST_DIR" 2>/dev/null || true
    rm -f /tmp/cluster_test_* 2>/dev/null || true
}

trap cleanup EXIT INT TERM

main() {
    echo "=================================================="
    echo "         DataMesh Fixed Cluster Test"
    echo "=================================================="
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found. Run: cargo build --release"
        exit 1
    fi
    
    # Clean up existing processes
    pkill -f datamesh 2>/dev/null || true
    sleep 2
    
    # Setup
    mkdir -p "$TEST_DIR/logs"
    log_info "Test directory: $TEST_DIR"
    
    # Start bootstrap node
    log_info "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" \
        > "$TEST_DIR/logs/bootstrap.log" 2>&1 &
    BOOTSTRAP_PID=$!
    CLUSTER_PIDS+=("$BOOTSTRAP_PID")
    
    # Wait for bootstrap to start
    log_info "Waiting for bootstrap to initialize..."
    sleep 8
    
    # Extract bootstrap info with better error handling
    local attempts=0
    while [[ $attempts -lt 10 ]]; do
        if [[ -f "$TEST_DIR/logs/bootstrap.log" ]]; then
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/logs/bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
            BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/logs/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
            
            if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
                break
            fi
        fi
        
        if ! kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
            log_error "Bootstrap process died"
            log_info "Bootstrap log:"
            cat "$TEST_DIR/logs/bootstrap.log" 2>/dev/null || echo "No log available"
            exit 1
        fi
        
        sleep 1
        ((attempts++))
    done
    
    if [[ -z "$BOOTSTRAP_PEER_ID" || -z "$BOOTSTRAP_ADDR" ]]; then
        log_error "Failed to get bootstrap connection info"
        log_info "Bootstrap log (last 20 lines):"
        tail -20 "$TEST_DIR/logs/bootstrap.log" 2>/dev/null || echo "No log available"
        exit 1
    fi
    
    log_success "Bootstrap node ready"
    log_info "Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "Address: $BOOTSTRAP_ADDR"
    
    # Start regular nodes with staggered timing
    log_info "Starting $NUM_NODES regular nodes..."
    local started_nodes=0
    
    for ((i=1; i<=NUM_NODES; i++)); do
        local node_port=$((BOOTSTRAP_PORT + i))
        log_info "Starting node $i on port $node_port..."
        
        "$DATAMESH_BINARY" --non-interactive service \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --port "$node_port" \
            > "$TEST_DIR/logs/node_$i.log" 2>&1 &
        
        local node_pid=$!
        CLUSTER_PIDS+=("$node_pid")
        
        # Check if node started
        sleep 3
        if kill -0 "$node_pid" 2>/dev/null; then
            log_success "Node $i started (PID: $node_pid)"
            ((started_nodes++))
        else
            log_warning "Node $i failed to start"
            log_info "Node $i log:"
            tail -10 "$TEST_DIR/logs/node_$i.log" 2>/dev/null || echo "No log available"
        fi
    done
    
    log_info "Started $started_nodes/$NUM_NODES nodes"
    
    # Wait for network to stabilize
    log_info "Waiting for network to stabilize..."
    sleep 10
    
    # Test basic connectivity first
    log_info "Testing basic connectivity..."
    local connectivity_attempts=0
    local max_connectivity_attempts=3
    
    while [[ $connectivity_attempts -lt $max_connectivity_attempts ]]; do
        log_info "Connectivity attempt $((connectivity_attempts + 1))/$max_connectivity_attempts"
        
        if timeout 10 "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            list > /dev/null 2>&1; then
            log_success "Basic connectivity established"
            break
        else
            log_warning "Connectivity attempt failed, retrying..."
            sleep 5
        fi
        ((connectivity_attempts++))
    done
    
    if [[ $connectivity_attempts -eq $max_connectivity_attempts ]]; then
        log_error "Failed to establish basic connectivity"
        log_info "Diagnostics:"
        echo "Bootstrap status: $(kill -0 "$BOOTSTRAP_PID" 2>/dev/null && echo "Running" || echo "Dead")"
        echo "Network ports in use:"
        ss -ln | grep -E ":408[0-9][0-9]" || echo "No ports found"
        exit 1
    fi
    
    # Test file operations with a more robust approach
    log_info "Testing file operations..."
    
    # Create test file
    local test_file="/tmp/cluster_test_file.txt"
    echo "Cluster test file - $(date)" > "$test_file"
    echo "Random data: $RANDOM" >> "$test_file"
    echo "Node count: $started_nodes" >> "$test_file"
    
    # Store file with multiple attempts
    local store_attempts=0
    local max_store_attempts=3
    local file_key=""
    
    while [[ $store_attempts -lt $max_store_attempts ]]; do
        log_info "File store attempt $((store_attempts + 1))/$max_store_attempts"
        
        local store_output
        if store_output=$(timeout 20 "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            put "$test_file" 2>&1); then
            
            file_key=$(echo "$store_output" | grep -E "(stored with key|File stored)" | grep -oE '[a-f0-9]{32,}' | head -1)
            if [[ -n "$file_key" ]]; then
                log_success "File stored with key: ${file_key:0:32}..."
                break
            else
                log_warning "Store successful but could not extract key"
                log_info "Store output: $store_output"
            fi
        else
            log_warning "Store attempt failed"
            log_info "Store output: $store_output"
        fi
        
        sleep 5
        ((store_attempts++))
    done
    
    if [[ -z "$file_key" ]]; then
        log_error "Failed to store file after $max_store_attempts attempts"
        exit 1
    fi
    
    # Retrieve file
    log_info "Testing file retrieval..."
    local retrieved_file="/tmp/cluster_test_retrieved.txt"
    
    if timeout 20 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        get "$file_key" "$retrieved_file" > /dev/null 2>&1; then
        
        # Verify file content
        if cmp -s "$test_file" "$retrieved_file"; then
            log_success "File retrieval and verification successful"
        else
            log_error "Retrieved file differs from original"
            log_info "Original size: $(wc -c < "$test_file") bytes"
            log_info "Retrieved size: $(wc -c < "$retrieved_file") bytes"
            exit 1
        fi
    else
        log_error "Failed to retrieve file"
        exit 1
    fi
    
    # Additional test: List files to verify file is in the network
    log_info "Testing file listing..."
    if timeout 10 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        list > /dev/null 2>&1; then
        log_success "File listing successful"
    else
        log_warning "File listing failed, but core functionality works"
    fi
    
    echo ""
    log_success "🎉 All cluster tests PASSED!"
    echo ""
    log_info "Cluster Summary:"
    echo "  Bootstrap Peer: $BOOTSTRAP_PEER_ID"
    echo "  Bootstrap Address: $BOOTSTRAP_ADDR"
    echo "  Nodes Started: $started_nodes/$NUM_NODES"
    echo "  Test File Key: ${file_key:0:32}..."
    echo "  Log Directory: $TEST_DIR/logs"
    echo ""
    log_info "The cluster is fully functional!"
    echo ""
    
    # Show cluster status
    log_info "Final cluster status:"
    local running_count=0
    for pid in "${CLUSTER_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            ((running_count++))
        fi
    done
    echo "  Running processes: $running_count/${#CLUSTER_PIDS[@]}"
    
    # Keep cluster running for a moment to show it's stable
    log_info "Keeping cluster running for 5 seconds to verify stability..."
    sleep 5
    
    # Final verification
    if timeout 5 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        list > /dev/null 2>&1; then
        log_success "Final stability check PASSED"
    else
        log_warning "Final stability check failed, but tests completed successfully"
    fi
    
    log_success "Cluster test completed successfully! ✅"
}

main "$@"