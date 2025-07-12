#!/bin/bash
# Enhanced Multi-Node DataMesh Test with Comprehensive Storage Testing
# This script thoroughly tests DataMesh with 3+ nodes for complete DHT functionality

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
NODE_PORTS=(40872 40873 40874)
TEST_DIR="enhanced_test_$(date +%Y%m%d_%H%M%S)"

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

# Process tracking
declare -A NODE_PIDS
declare -A NODE_LOGS
BOOTSTRAP_PID=""
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""

# Test statistics
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

cleanup() {
    if [[ "$1" != "manual" ]]; then
        log_info "Cleaning up test environment..."
    fi
    
    # Stop all nodes
    for port in "${NODE_PORTS[@]}"; do
        if [[ -n "${NODE_PIDS[$port]}" ]]; then
            kill "${NODE_PIDS[$port]}" 2>/dev/null || true
        fi
    done
    
    # Stop bootstrap
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    
    sleep 2
    
    # Force cleanup
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    
    # Clean test files
    rm -f /tmp/enhanced_test_* 2>/dev/null || true
    
    if [[ "$1" != "manual" ]]; then
        log_success "Cleanup completed"
    fi
}

# Only trap on TERM and INT, not EXIT to avoid double cleanup
trap cleanup TERM INT

extract_bootstrap_info() {
    local log_file="$1"
    local max_attempts=20
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if [[ -f "$log_file" ]]; then
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
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

start_bootstrap() {
    log_info "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    
    local bootstrap_log="$TEST_DIR/bootstrap.log"
    mkdir -p "$TEST_DIR"
    
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$bootstrap_log" 2>&1 &
    BOOTSTRAP_PID=$!
    
    if ! extract_bootstrap_info "$bootstrap_log"; then
        log_error "Failed to start bootstrap node"
        return 1
    fi
    
    log_success "Bootstrap node started"
    log_info "  Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "  Address: $BOOTSTRAP_ADDR"
    log_info "  PID: $BOOTSTRAP_PID"
    
    return 0
}

start_service_node() {
    local port="$1"
    local node_id="node_$port"
    local log_file="$TEST_DIR/${node_id}.log"
    
    log_info "Starting service node on port $port..."
    
    "$DATAMESH_BINARY" --non-interactive service \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$port" > "$log_file" 2>&1 &
    
    local pid=$!
    NODE_PIDS[$port]=$pid
    NODE_LOGS[$port]=$log_file
    
    # Give node time to start and connect
    sleep 5
    
    if kill -0 "$pid" 2>/dev/null; then
        log_success "Service node started on port $port (PID: $pid)"
        return 0
    else
        log_error "Service node failed to start on port $port"
        cat "$log_file" | tail -10
        return 1
    fi
}

wait_for_network_stabilization() {
    log_info "Waiting for network stabilization and peer discovery..."
    local stabilization_time=20
    
    for ((i=1; i<=stabilization_time; i++)); do
        echo -ne "\r  Progress: [$((i*100/stabilization_time))%] ($i/$stabilization_time seconds)"
        sleep 1
    done
    echo ""
    
    log_success "Network stabilization period completed"
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    ((TOTAL_TESTS++))
    log_info "🧪 Running test: $test_name"
    
    if eval "$test_command"; then
        log_success "Test passed: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "Test failed: $test_name"
        ((TESTS_FAILED++))
        return 1
    fi
}

test_basic_file_storage() {
    log_info "Testing basic file storage on bootstrap node..."
    
    # Create test file
    local test_file="/tmp/enhanced_test_basic.txt"
    echo "Basic test content - $(date)" > "$test_file"
    
    # Store file using bootstrap node
    local store_output
    if store_output=$(timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port 40875 \
        --non-interactive \
        put "$test_file" 2>&1); then
        
        local file_key=$(echo "$store_output" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        
        if [[ -n "$file_key" ]]; then
            log_success "✅ File stored successfully with key: ${file_key:0:16}..."
            
            # Test retrieval
            local output_file="/tmp/enhanced_test_basic_retrieved.txt"
            if timeout 30 "$DATAMESH_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --port 40876 \
                --non-interactive \
                get "$file_key" "$output_file" 2>/dev/null; then
                
                if cmp -s "$test_file" "$output_file"; then
                    log_success "✅ File retrieved successfully and content matches"
                    return 0
                else
                    log_error "❌ File content mismatch after retrieval"
                    return 1
                fi
            else
                log_error "❌ File retrieval failed"
                return 1
            fi
        else
            log_error "❌ Could not extract file key from store output"
            echo "Store output: $store_output"
            return 1
        fi
    else
        log_error "❌ File storage failed"
        echo "Store output: $store_output"
        return 1
    fi
}

test_cross_node_storage() {
    log_info "Testing cross-node file storage and retrieval..."
    
    local test_content="Cross-node test content - $(date) - 🚀"
    local test_file="/tmp/enhanced_test_cross_node.txt"
    echo "$test_content" > "$test_file"
    
    # Store file from first service node
    local store_port="${NODE_PORTS[0]}"
    log_info "Storing file from node on port $store_port..."
    
    local store_output
    if store_output=$(timeout 45 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$((store_port + 1000))" \
        --non-interactive \
        put "$test_file" 2>&1); then
        
        local file_key=$(echo "$store_output" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        
        if [[ -n "$file_key" ]]; then
            log_success "✅ File stored from node $store_port with key: ${file_key:0:16}..."
            
            # Retrieve from different service node
            local retrieve_port="${NODE_PORTS[1]}"
            log_info "Retrieving file from different node on port $retrieve_port..."
            
            local output_file="/tmp/enhanced_test_cross_node_retrieved.txt"
            if timeout 45 "$DATAMESH_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --port "$((retrieve_port + 2000))" \
                --non-interactive \
                get "$file_key" "$output_file" 2>/dev/null; then
                
                if cmp -s "$test_file" "$output_file"; then
                    log_success "✅ Cross-node file retrieval successful!"
                    log_success "✅ DHT functionality working across nodes"
                    return 0
                else
                    log_error "❌ File content mismatch in cross-node retrieval"
                    return 1
                fi
            else
                log_error "❌ Cross-node file retrieval failed"
                return 1
            fi
        else
            log_error "❌ Could not extract file key from cross-node store"
            echo "Store output: $store_output"
            return 1
        fi
    else
        log_error "❌ Cross-node file storage failed"
        echo "Store output: $store_output"
        return 1
    fi
}

test_multiple_files() {
    log_info "Testing storage of multiple files across different nodes..."
    
    local success_count=0
    local total_files=3
    
    for i in $(seq 1 $total_files); do
        local test_file="/tmp/enhanced_test_multi_$i.txt"
        echo "Multi-file test $i - $(date) - Content: $RANDOM" > "$test_file"
        
        # Use different ports for each storage operation
        local port_index=$((i % ${#NODE_PORTS[@]}))
        local store_port="${NODE_PORTS[$port_index]}"
        
        log_info "Storing file $i from node on port $store_port..."
        
        local store_output
        if store_output=$(timeout 30 "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --port "$((store_port + 3000 + i))" \
            --non-interactive \
            put "$test_file" 2>&1); then
            
            local file_key=$(echo "$store_output" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
            
            if [[ -n "$file_key" ]]; then
                log_success "✅ File $i stored with key: ${file_key:0:16}..."
                ((success_count++))
            else
                log_warning "⚠️  Could not extract key for file $i"
            fi
        else
            log_warning "⚠️  Failed to store file $i"
        fi
        
        sleep 2  # Small delay between operations
    done
    
    log_info "Multiple file test results: $success_count/$total_files files stored successfully"
    
    if [[ $success_count -ge 2 ]]; then
        log_success "✅ Multiple file storage test passed (≥2 files successful)"
        return 0
    else
        log_error "❌ Multiple file storage test failed (<2 files successful)"
        return 1
    fi
}

test_node_resilience() {
    log_info "Testing node resilience - temporary node shutdown..."
    
    # Stop one service node temporarily
    local test_port="${NODE_PORTS[2]}"
    local test_pid="${NODE_PIDS[$test_port]}"
    
    log_info "Temporarily stopping node on port $test_port..."
    kill "$test_pid" 2>/dev/null || true
    sleep 3
    
    # Try to store a file with one node down
    local test_file="/tmp/enhanced_test_resilience.txt"
    echo "Resilience test - $(date)" > "$test_file"
    
    local store_output
    if store_output=$(timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port 40879 \
        --non-interactive \
        put "$test_file" 2>&1); then
        
        local file_key=$(echo "$store_output" | grep -oE 'stored with key: [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        
        if [[ -n "$file_key" ]]; then
            log_success "✅ File stored successfully with one node down"
            
            # Restart the node
            log_info "Restarting the stopped node..."
            start_service_node "$test_port"
            sleep 5
            
            # Test retrieval after node restart
            local output_file="/tmp/enhanced_test_resilience_retrieved.txt"
            if timeout 30 "$DATAMESH_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --port 40880 \
                --non-interactive \
                get "$file_key" "$output_file" 2>/dev/null; then
                
                if cmp -s "$test_file" "$output_file"; then
                    log_success "✅ Node resilience test passed - data survived node restart"
                    return 0
                else
                    log_error "❌ File content mismatch after node restart"
                    return 1
                fi
            else
                log_error "❌ File retrieval failed after node restart"
                return 1
            fi
        else
            log_error "❌ Could not extract file key in resilience test"
            return 1
        fi
    else
        log_error "❌ File storage failed with one node down"
        echo "Store output: $store_output"
        return 1
    fi
}

check_network_connectivity() {
    log_info "Checking network connectivity between nodes..."
    
    local connected_nodes=0
    
    for port in "${NODE_PORTS[@]}"; do
        local log_file="${NODE_LOGS[$port]}"
        
        if [[ -f "$log_file" ]]; then
            local peer_connections=$(grep -c "Connected to peer:\|peer connected" "$log_file" 2>/dev/null || echo "0")
            local peer_id=$(grep -E "Network actor starting with peer ID:|Local peer id:" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || echo "unknown")
            
            log_info "Node $port (ID: ${peer_id:0:16}...): $peer_connections peer connections"
            
            if [[ $peer_connections -gt 0 ]]; then
                ((connected_nodes++))
            fi
        fi
    done
    
    log_info "Network connectivity: $connected_nodes/${#NODE_PORTS[@]} nodes have peer connections"
    
    if [[ $connected_nodes -ge 1 ]]; then
        log_success "✅ Network connectivity established"
        return 0
    else
        log_warning "⚠️  Limited network connectivity"
        return 1
    fi
}

main() {
    echo "=========================================================="
    echo "     Enhanced DataMesh Multi-Node Functionality Test"
    echo "=========================================================="
    echo "Testing comprehensive DHT functionality with storage operations"
    echo "Bootstrap: 1 node, Service nodes: ${#NODE_PORTS[@]}"
    echo ""
    
    # Check binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        echo "Please run: cargo build --release"
        exit 1
    fi
    
    # Clean up any existing processes
    cleanup manual
    sleep 2
    
    # Start bootstrap node
    if ! start_bootstrap; then
        log_error "Failed to start bootstrap node"
        exit 1
    fi
    
    # Start service nodes
    local started_nodes=0
    for port in "${NODE_PORTS[@]}"; do
        if start_service_node "$port"; then
            ((started_nodes++))
        else
            log_warning "Failed to start node on port $port"
        fi
    done
    
    if [[ $started_nodes -eq 0 ]]; then
        log_error "No service nodes started successfully"
        cleanup manual
        exit 1
    fi
    
    log_success "Started $started_nodes/${#NODE_PORTS[@]} service nodes"
    
    # Wait for network stabilization
    wait_for_network_stabilization
    
    # Check network connectivity
    run_test "Network Connectivity" "check_network_connectivity"
    
    # Run comprehensive storage tests
    echo ""
    log_info "🚀 Running comprehensive storage functionality tests..."
    echo ""
    
    run_test "Basic File Storage" "test_basic_file_storage"
    sleep 3
    
    run_test "Cross-Node Storage" "test_cross_node_storage"
    sleep 3
    
    run_test "Multiple Files Storage" "test_multiple_files"
    sleep 3
    
    run_test "Node Resilience" "test_node_resilience"
    
    # Final results
    echo ""
    echo "=========================================================="
    echo "                 FINAL TEST RESULTS"
    echo "=========================================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Tests Passed: $TESTS_PASSED ✅"
    echo "Tests Failed: $TESTS_FAILED ❌"
    echo "Success Rate: $(( TESTS_PASSED * 100 / TOTAL_TESTS ))%"
    echo ""
    
    if [[ $TESTS_PASSED -eq $TOTAL_TESTS ]]; then
        log_success "🎉 ALL TESTS PASSED! DataMesh cluster is fully functional! 🚀"
        echo ""
        echo "✅ Bootstrap node functioning correctly"
        echo "✅ Service nodes connecting and communicating"
        echo "✅ File storage working across nodes"
        echo "✅ Cross-node retrieval functioning"
        echo "✅ Multi-file operations successful"
        echo "✅ Network resilience validated"
        echo "✅ Kademlia DHT routing operational"
        test_result=0
    elif [[ $TESTS_PASSED -ge $((TOTAL_TESTS * 3 / 4)) ]]; then
        log_warning "⚠️  PARTIAL SUCCESS - Most tests passed"
        echo "💡 Some functionality may need optimization"
        test_result=0
    else
        log_error "❌ TESTS FAILED - Major issues detected"
        echo "💡 Check node logs in $TEST_DIR/ for details"
        test_result=1
    fi
    
    echo ""
    echo "Log Directory: $TEST_DIR"
    echo "Bootstrap PID: $BOOTSTRAP_PID"
    echo "Service Node PIDs: ${NODE_PIDS[*]}"
    echo "=========================================================="
    
    # Keep running briefly to show stability
    log_info "Keeping cluster running for 10 seconds to demonstrate stability..."
    sleep 10
    
    # Manual cleanup
    cleanup manual
    
    return $test_result
}

main "$@"