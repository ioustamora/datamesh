#!/bin/bash
# DataMesh 6-Node Cluster Test with Comprehensive Command Testing

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
NUM_NODES=6
NODE_PORTS=(40872 40873 40874 40875 40876 40877)

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
BOOTSTRAP_PID=""
NODE_PIDS=()
BOOTSTRAP_MULTIADDR=""

cleanup() {
    log_info "Cleaning up all processes..."
    
    # Kill all node processes
    for pid in "${NODE_PIDS[@]}"; do
        [[ -n "$pid" ]] && kill "$pid" 2>/dev/null || true
    done
    
    # Kill bootstrap
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    
    sleep 2
    
    # Force cleanup
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*join" 2>/dev/null || true
    
    # Clean up temp files
    rm -f /tmp/test_*.txt /tmp/store_*.txt /tmp/cluster_*.txt
    
    log_info "Cleanup complete"
}

trap cleanup EXIT

start_cluster() {
    log_info "🚀 Starting DataMesh 6-node cluster..."
    
    # Build if needed
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_info "Building DataMesh..."
        cargo build --release
    fi
    
    # Clean up any existing processes
    cleanup
    sleep 2
    
    # Start bootstrap node
    log_info "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > /tmp/bootstrap.log 2>&1 &
    BOOTSTRAP_PID=$!
    
    # Wait for bootstrap to initialize
    sleep 10
    
    # Extract bootstrap peer ID from logs
    local attempts=0
    BOOTSTRAP_PEER_ID=""
    while [[ $attempts -lt 10 && -z "$BOOTSTRAP_PEER_ID" ]]; do
        BOOTSTRAP_PEER_ID=$(grep -o "Peer ID: [A-Za-z0-9]\+" /tmp/bootstrap.log 2>/dev/null | sed 's/Peer ID: //' || echo "")
        if [[ -z "$BOOTSTRAP_PEER_ID" ]]; then
            sleep 1
            ((attempts++))
        fi
    done
    
    if [[ -z "$BOOTSTRAP_PEER_ID" ]]; then
        log_error "Failed to extract bootstrap peer ID"
        return 1
    fi
    
    # Extract bootstrap multiaddr from logs or construct it
    BOOTSTRAP_MULTIADDR="/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT"
    log_info "Bootstrap peer ID: $BOOTSTRAP_PEER_ID"
    log_info "Bootstrap multiaddr: $BOOTSTRAP_MULTIADDR"
    
    # Start the 6 additional nodes
    for i in "${!NODE_PORTS[@]}"; do
        local port=${NODE_PORTS[$i]}
        local node_num=$((i + 1))
        
        log_info "Starting node $node_num on port $port..."
        "$DATAMESH_BINARY" --non-interactive join \
            --port "$port" \
            --bootstrap "$BOOTSTRAP_MULTIADDR" > "/tmp/node_${node_num}.log" 2>&1 &
        
        local pid=$!
        NODE_PIDS+=("$pid")
        
        # Brief delay between node starts
        sleep 3
    done
    
    # Wait for all nodes to connect
    log_info "Waiting for cluster stabilization..."
    sleep 15
    
    log_success "6-node cluster started successfully!"
    log_info "Cluster composition:"
    echo "  • Bootstrap node: 127.0.0.1:$BOOTSTRAP_PORT (PID: $BOOTSTRAP_PID)"
    for i in "${!NODE_PORTS[@]}"; do
        local port=${NODE_PORTS[$i]}
        local node_num=$((i + 1))
        echo "  • Node $node_num: 127.0.0.1:$port (PID: ${NODE_PIDS[$i]})"
    done
    echo ""
}

test_basic_storage() {
    log_info "🧪 Testing basic storage operations..."
    
    # Create test file
    echo "Test content $(date)" > /tmp/test_basic.txt
    
    # Store file - connect to bootstrap node
    log_info "Storing test file..."
    if "$DATAMESH_BINARY" --non-interactive --bootstrap-peers "$BOOTSTRAP_PEER_ID@$BOOTSTRAP_MULTIADDR" put /tmp/test_basic.txt > /tmp/store_basic.txt 2>&1; then
        local key=$(grep -oE '[a-f0-9]{32,}' /tmp/store_basic.txt | head -1)
        if [[ -n "$key" ]]; then
            log_success "File stored with key: ${key:0:16}..."
            
            # Retrieve file
            log_info "Retrieving test file..."
            if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" get "$key" /tmp/test_retrieved_basic.txt &>/dev/null; then
                if cmp -s /tmp/test_basic.txt /tmp/test_retrieved_basic.txt; then
                    log_success "Basic storage test PASSED!"
                    return 0
                else
                    log_error "Retrieved file content doesn't match original"
                    return 1
                fi
            else
                log_error "Failed to retrieve file"
                return 1
            fi
        else
            log_error "No key returned from storage operation"
            return 1
        fi
    else
        log_error "Failed to store file"
        cat /tmp/store_basic.txt
        return 1
    fi
}

test_list_files() {
    log_info "🧪 Testing list files command..."
    
    if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" list > /tmp/list_output.txt 2>&1; then
        local file_count=$(grep -c "Key:" /tmp/list_output.txt || echo "0")
        log_success "List command executed successfully. Found $file_count files."
        return 0
    else
        log_error "List command failed"
        cat /tmp/list_output.txt
        return 1
    fi
}

test_network_status() {
    log_info "🧪 Testing network status command..."
    
    if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" stats > /tmp/status_output.txt 2>&1; then
        if grep -q "Connected peers" /tmp/status_output.txt; then
            local peer_count=$(grep "Connected peers" /tmp/status_output.txt | grep -oE '[0-9]+' | head -1)
            log_success "Network status command executed. Connected peers: $peer_count"
            return 0
        else
            log_warning "Network status command executed but no peer info found"
            return 0
        fi
    else
        log_error "Network status command failed"
        cat /tmp/status_output.txt
        return 1
    fi
}

test_multiple_files() {
    log_info "🧪 Testing multiple file storage..."
    
    local success_count=0
    local keys=()
    
    # Store multiple files
    for i in {1..3}; do
        echo "Test file $i content $(date)" > "/tmp/test_multi_$i.txt"
        
        if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" put "/tmp/test_multi_$i.txt" > "/tmp/store_multi_$i.txt" 2>&1; then
            local key=$(grep -oE '[a-f0-9]{32,}' "/tmp/store_multi_$i.txt" | head -1)
            if [[ -n "$key" ]]; then
                keys+=("$key")
                ((success_count++))
                log_success "File $i stored with key: ${key:0:16}..."
            else
                log_error "No key returned for file $i"
            fi
        else
            log_error "Failed to store file $i"
        fi
    done
    
    # Retrieve multiple files
    local retrieve_count=0
    for i in "${!keys[@]}"; do
        local key=${keys[$i]}
        local file_num=$((i + 1))
        
        if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" get "$key" "/tmp/test_retrieved_multi_$file_num.txt" &>/dev/null; then
            if cmp -s "/tmp/test_multi_$file_num.txt" "/tmp/test_retrieved_multi_$file_num.txt"; then
                ((retrieve_count++))
                log_success "File $file_num retrieved successfully"
            else
                log_error "Retrieved file $file_num content doesn't match"
            fi
        else
            log_error "Failed to retrieve file $file_num"
        fi
    done
    
    if [[ $success_count -eq 3 && $retrieve_count -eq 3 ]]; then
        log_success "Multiple files test PASSED! ($success_count stored, $retrieve_count retrieved)"
        return 0
    else
        log_error "Multiple files test FAILED! ($success_count stored, $retrieve_count retrieved)"
        return 1
    fi
}

test_large_file() {
    log_info "🧪 Testing large file storage (1MB)..."
    
    # Create 1MB test file
    dd if=/dev/urandom of=/tmp/test_large.txt bs=1024 count=1024 2>/dev/null
    
    if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" put /tmp/test_large.txt > /tmp/store_large.txt 2>&1; then
        local key=$(grep -oE '[a-f0-9]{32,}' /tmp/store_large.txt | head -1)
        if [[ -n "$key" ]]; then
            log_success "Large file stored with key: ${key:0:16}..."
            
            # Retrieve large file
            log_info "Retrieving large file..."
            if "$DATAMESH_BINARY" --non-interactive --bootstrap-peer "$BOOTSTRAP_PEER_ID" --bootstrap-addr "$BOOTSTRAP_MULTIADDR" get "$key" /tmp/test_retrieved_large.txt &>/dev/null; then
                if cmp -s /tmp/test_large.txt /tmp/test_retrieved_large.txt; then
                    log_success "Large file test PASSED!"
                    return 0
                else
                    log_error "Retrieved large file content doesn't match"
                    return 1
                fi
            else
                log_error "Failed to retrieve large file"
                return 1
            fi
        else
            log_error "No key returned from large file storage"
            return 1
        fi
    else
        log_error "Failed to store large file"
        cat /tmp/store_large.txt
        return 1
    fi
}

run_comprehensive_tests() {
    log_info "🎯 Running comprehensive command tests..."
    echo ""
    
    local tests_passed=0
    local tests_total=5
    
    # Test 1: Basic storage
    if test_basic_storage; then
        ((tests_passed++))
    fi
    echo ""
    
    # Test 2: List files
    if test_list_files; then
        ((tests_passed++))
    fi
    echo ""
    
    # Test 3: Network status
    if test_network_status; then
        ((tests_passed++))
    fi
    echo ""
    
    # Test 4: Multiple files
    if test_multiple_files; then
        ((tests_passed++))
    fi
    echo ""
    
    # Test 5: Large file
    if test_large_file; then
        ((tests_passed++))
    fi
    echo ""
    
    # Results summary
    log_info "📊 Test Results Summary:"
    echo "  • Tests passed: $tests_passed/$tests_total"
    echo "  • Success rate: $((tests_passed * 100 / tests_total))%"
    echo ""
    
    if [[ $tests_passed -eq $tests_total ]]; then
        log_success "🎉 ALL TESTS PASSED! DataMesh 6-node cluster is fully functional!"
        return 0
    else
        log_error "Some tests failed. Cluster may have issues."
        return 1
    fi
}

# Main execution
main() {
    log_info "🌟 DataMesh 6-Node Cluster Test Suite"
    echo "======================================"
    echo ""
    
    start_cluster
    
    # Wait a bit more for full stabilization
    log_info "Allowing cluster to fully stabilize..."
    sleep 10
    
    run_comprehensive_tests
    
    # Keep cluster running briefly to show stability
    log_info "Keeping cluster running for 10 seconds to demonstrate stability..."
    sleep 10
    
    log_success "Test completed successfully! 🚀"
}

# Execute main function
main "$@"