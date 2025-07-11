#!/bin/bash
# Improved DataMesh Cluster Test Script
# This script provides enhanced cluster testing with better error handling,
# diagnostics, and recovery mechanisms.

set -e

# Enhanced configuration
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/release/datamesh}"
BOOTSTRAP_PORT=${BOOTSTRAP_PORT:-40871}
API_PORT=${API_PORT:-8080}
NUM_NODES=${NUM_NODES:-7}
TEST_DIR="cluster_test_$(date +%Y%m%d_%H%M%S)"
LOG_LEVEL=${LOG_LEVEL:-info}
STARTUP_TIMEOUT=${STARTUP_TIMEOUT:-10}
TEST_TIMEOUT=${TEST_TIMEOUT:-30}

# Enhanced colors and symbols
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Enhanced logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1"; }
log_progress() { echo -e "${PURPLE}[PROGRESS]${NC} ⏳ $1"; }
log_debug() { if [[ "$LOG_LEVEL" == "debug" ]]; then echo -e "${CYAN}[DEBUG]${NC} 🔍 $1"; fi }

# Global variables
CLUSTER_PIDS=()
BOOTSTRAP_PID=""
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""
TEST_SUCCESS=true
CLEANUP_NEEDED=false

# Cleanup function
cleanup() {
    if [[ "$CLEANUP_NEEDED" == "true" ]]; then
        log_info "Cleaning up test environment..."
        
        # Stop cluster
        log_info "Stopping cluster..."
        for pid in "${CLUSTER_PIDS[@]}"; do
            if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
                log_debug "Killing process $pid"
                kill "$pid" 2>/dev/null || true
                sleep 1
                # Force kill if still running
                if kill -0 "$pid" 2>/dev/null; then
                    kill -9 "$pid" 2>/dev/null || true
                fi
            fi
        done
        
        # Clean up test directory
        if [[ -d "$TEST_DIR" ]]; then
            log_debug "Removing test directory $TEST_DIR"
            rm -rf "$TEST_DIR"
        fi
        
        # Clean up temporary files
        rm -f /tmp/dfs_test_* /tmp/dfs_retrieved_* 2>/dev/null || true
        
        log_info "Cleanup complete"
    fi
}

# Set up cleanup trap
trap cleanup EXIT INT TERM

# Enhanced error handling
handle_error() {
    local exit_code=$?
    local line_number=$1
    log_error "Script failed at line $line_number with exit code $exit_code"
    TEST_SUCCESS=false
    
    # Enhanced diagnostics
    show_diagnostics
    
    return $exit_code
}

trap 'handle_error $LINENO' ERR

# Pre-flight checks
preflight_checks() {
    log_info "🔍 Running pre-flight checks..."
    
    # Check if binary exists and is executable
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        log_error "Please run: cargo build --release"
        exit 1
    fi
    
    if [[ ! -x "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary is not executable"
        exit 1
    fi
    
    # Check for existing processes
    local existing_procs=$(ps aux | grep -E "datamesh.*bootstrap|datamesh.*service" | grep -v grep | wc -l)
    if [[ $existing_procs -gt 0 ]]; then
        log_warning "Found $existing_procs existing DataMesh processes"
        log_info "Cleaning up existing processes..."
        pkill -f datamesh || true
        sleep 2
    fi
    
    # Check port availability
    local ports_to_check=($BOOTSTRAP_PORT $API_PORT)
    for ((i=1; i<=NUM_NODES; i++)); do
        ports_to_check+=($((BOOTSTRAP_PORT + i)))
    done
    
    for port in "${ports_to_check[@]}"; do
        if ss -ln | grep -q ":$port "; then
            log_warning "Port $port is already in use"
            # Try to find and kill the process
            local pid=$(lsof -ti :$port 2>/dev/null || true)
            if [[ -n "$pid" ]]; then
                log_info "Killing process $pid using port $port"
                kill "$pid" 2>/dev/null || true
                sleep 1
            fi
        fi
    done
    
    # Check available disk space
    local available_space=$(df "$PWD" | awk 'NR==2 {print $4}')
    if [[ $available_space -lt 1000000 ]]; then  # Less than 1GB
        log_warning "Low disk space available: $(df -h "$PWD" | awk 'NR==2 {print $4}')"
    fi
    
    # Check system resources
    local available_memory=$(free -m | awk 'NR==2{print $7}')
    if [[ $available_memory -lt 512 ]]; then  # Less than 512MB
        log_warning "Low memory available: ${available_memory}MB"
    fi
    
    log_success "Pre-flight checks completed"
}

# Enhanced environment setup
setup_environment() {
    log_info "🏗️  Setting up test environment..."
    
    # Create test directory structure
    mkdir -p "$TEST_DIR"/{logs,keys,data,temp}
    
    # Set environment variables
    export RUST_LOG=${RUST_LOG:-$LOG_LEVEL}
    export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
    
    # Create test configuration
    cat > "$TEST_DIR/test_config.toml" << EOF
[network]
bootstrap_port = $BOOTSTRAP_PORT
max_peers = 50
connection_timeout = 30

[storage]
max_file_size = "100MB"
chunk_size = "64KB"

[performance]
concurrent_uploads = 4
concurrent_downloads = 4

[logging]
level = "$LOG_LEVEL"
target = "file"
EOF
    
    CLEANUP_NEEDED=true
    log_success "Test environment setup complete"
}

# Enhanced node startup with better error handling
start_bootstrap_node() {
    log_progress "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    
    # Start bootstrap node with enhanced logging
    "$DATAMESH_BINARY" \
        --non-interactive \
        bootstrap \
        --port "$BOOTSTRAP_PORT" \
        > "$TEST_DIR/logs/bootstrap.log" 2>&1 &
    
    BOOTSTRAP_PID=$!
    CLUSTER_PIDS+=("$BOOTSTRAP_PID")
    
    log_debug "Bootstrap node started with PID $BOOTSTRAP_PID"
    
    # Wait for bootstrap node to start with timeout
    local timeout_count=0
    while [[ $timeout_count -lt $STARTUP_TIMEOUT ]]; do
        if [[ -f "$TEST_DIR/logs/bootstrap.log" ]]; then
            # Extract peer ID and address from logs
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/logs/bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
            BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/logs/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
            
            if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
                break
            fi
        fi
        
        # Check if process is still running
        if ! kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
            log_error "Bootstrap node process died"
            show_bootstrap_logs
            return 1
        fi
        
        sleep 1
        ((timeout_count++))
    done
    
    if [[ -z "$BOOTSTRAP_PEER_ID" || -z "$BOOTSTRAP_ADDR" ]]; then
        log_error "Bootstrap node failed to start within $STARTUP_TIMEOUT seconds"
        show_bootstrap_logs
        return 1
    fi
    
    log_success "Bootstrap node started successfully"
    log_info "  Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "  Address: $BOOTSTRAP_ADDR"
    log_info "  PID: $BOOTSTRAP_PID"
    
    return 0
}

# Enhanced regular node startup
start_regular_nodes() {
    log_progress "Starting $NUM_NODES regular nodes..."
    
    local started_nodes=0
    local api_port=$API_PORT
    
    for ((i=1; i<=NUM_NODES; i++)); do
        local node_port=$((BOOTSTRAP_PORT + i))
        log_progress "Starting node $i on port $node_port (API: $api_port)"
        
        # Start node with enhanced configuration
        "$DATAMESH_BINARY" \
            --non-interactive \
            service \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --port "$node_port" \
            > "$TEST_DIR/logs/node_$i.log" 2>&1 &
        
        local node_pid=$!
        CLUSTER_PIDS+=("$node_pid")
        
        log_debug "Node $i started with PID $node_pid"
        
        # Wait a moment and check if node started successfully
        sleep 2
        if kill -0 "$node_pid" 2>/dev/null; then
            # Additional verification - check if node connected to bootstrap
            local connected=false
            for ((j=1; j<=5; j++)); do
                if grep -q "Connected to bootstrap" "$TEST_DIR/logs/node_$i.log" 2>/dev/null || \
                   grep -q "Successfully connected" "$TEST_DIR/logs/node_$i.log" 2>/dev/null; then
                    connected=true
                    break
                fi
                sleep 1
            done
            
            if [[ "$connected" == "true" ]]; then
                log_success "Node $i started and connected (PID $node_pid, Port $node_port)"
                ((started_nodes++))
            else
                log_warning "Node $i started but may not be fully connected (PID $node_pid)"
                ((started_nodes++))
            fi
        else
            log_error "Node $i failed to start (PID $node_pid)"
            show_node_logs $i
        fi
        
        ((api_port++))
    done
    
    log_info "Started $started_nodes/$NUM_NODES nodes successfully"
    
    # Wait for network stabilization
    log_progress "Waiting for network to stabilize..."
    sleep 5
    
    return 0
}

# Enhanced cluster startup
start_cluster() {
    log_info "🚀 Starting DataMesh cluster with $NUM_NODES nodes"
    
    if ! start_bootstrap_node; then
        log_error "Failed to start bootstrap node"
        return 1
    fi
    
    if ! start_regular_nodes; then
        log_error "Failed to start regular nodes"
        return 1
    fi
    
    log_success "Cluster startup completed"
    show_cluster_status
    
    return 0
}

# Enhanced cluster status display
show_cluster_status() {
    log_info "📊 Cluster Status Summary:"
    echo "  Bootstrap Node: $BOOTSTRAP_PEER_ID"
    echo "  Bootstrap Address: $BOOTSTRAP_ADDR"
    echo "  Total Processes: ${#CLUSTER_PIDS[@]}"
    
    local running_count=0
    for pid in "${CLUSTER_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            ((running_count++))
        fi
    done
    
    echo "  Running Processes: $running_count/${#CLUSTER_PIDS[@]}"
    echo "  Log Directory: $TEST_DIR/logs"
}

# Enhanced diagnostics
show_diagnostics() {
    log_info "🔬 System Diagnostics:"
    
    echo "System Information:"
    echo "  Date/Time: $(date)"
    echo "  User: $(whoami)"
    echo "  Working Directory: $(pwd)"
    echo "  DataMesh Binary: $DATAMESH_BINARY"
    echo "  Binary Size: $(ls -lh "$DATAMESH_BINARY" | awk '{print $5}')"
    echo "  Binary Modified: $(stat -c %y "$DATAMESH_BINARY")"
    
    echo ""
    echo "Process Information:"
    echo "  Bootstrap PID: ${BOOTSTRAP_PID:-'Not started'}"
    echo "  Total PIDs: ${#CLUSTER_PIDS[@]}"
    echo "  Running Processes:"
    for pid in "${CLUSTER_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "    PID $pid: RUNNING"
        else
            echo "    PID $pid: STOPPED"
        fi
    done
    
    echo ""
    echo "Network Information:"
    echo "  Bootstrap Port: $BOOTSTRAP_PORT"
    echo "  API Port: $API_PORT"
    echo "  Listening Ports:"
    ss -ln | grep -E ":408[0-9][0-9]" | head -10 || echo "    No DataMesh ports found"
    
    echo ""
    echo "Resource Usage:"
    echo "  Memory Usage:"
    ps aux | grep datamesh | grep -v grep | awk '{print "    PID " $2 ": " $4 "% memory, " $3 "% CPU"}' || echo "    No DataMesh processes found"
    
    echo ""
    echo "Recent Log Entries:"
    if [[ -f "$TEST_DIR/logs/bootstrap.log" ]]; then
        echo "  Bootstrap (last 3 lines):"
        tail -3 "$TEST_DIR/logs/bootstrap.log" | sed 's/^/    /'
    fi
    
    if [[ -f "$TEST_DIR/logs/node_1.log" ]]; then
        echo "  Node 1 (last 3 lines):"
        tail -3 "$TEST_DIR/logs/node_1.log" | sed 's/^/    /'
    fi
}

# Enhanced log display functions
show_bootstrap_logs() {
    if [[ -f "$TEST_DIR/logs/bootstrap.log" ]]; then
        log_info "Bootstrap node logs (last 20 lines):"
        tail -20 "$TEST_DIR/logs/bootstrap.log" | sed 's/^/  /'
    else
        log_warning "Bootstrap log file not found"
    fi
}

show_node_logs() {
    local node_num=$1
    if [[ -f "$TEST_DIR/logs/node_$node_num.log" ]]; then
        log_info "Node $node_num logs (last 10 lines):"
        tail -10 "$TEST_DIR/logs/node_$node_num.log" | sed 's/^/  /'
    else
        log_warning "Node $node_num log file not found"
    fi
}

# Enhanced cluster testing
run_cluster_tests() {
    log_info "🧪 Running cluster functionality tests..."
    
    local test_results=()
    
    # Test 1: Basic file storage and retrieval
    log_progress "Test 1: Basic file storage and retrieval"
    if run_basic_storage_test; then
        test_results+=("✅ Basic storage: PASS")
    else
        test_results+=("❌ Basic storage: FAIL")
        TEST_SUCCESS=false
    fi
    
    # Test 2: Network connectivity
    log_progress "Test 2: Network connectivity verification"
    if run_network_test; then
        test_results+=("✅ Network connectivity: PASS")
    else
        test_results+=("❌ Network connectivity: FAIL")
        TEST_SUCCESS=false
    fi
    
    # Test 3: Multiple file operations
    log_progress "Test 3: Multiple file operations"
    if run_multiple_files_test; then
        test_results+=("✅ Multiple files: PASS")
    else
        test_results+=("❌ Multiple files: FAIL")
        TEST_SUCCESS=false
    fi
    
    # Test 4: Large file handling
    log_progress "Test 4: Large file handling"
    if run_large_file_test; then
        test_results+=("✅ Large file: PASS")
    else
        test_results+=("❌ Large file: FAIL")
        TEST_SUCCESS=false
    fi
    
    # Display test results
    echo ""
    log_info "📋 Test Results Summary:"
    for result in "${test_results[@]}"; do
        echo "  $result"
    done
    
    echo ""
    if [[ "$TEST_SUCCESS" == "true" ]]; then
        log_success "All cluster tests PASSED! 🎉"
    else
        log_error "Some cluster tests FAILED! ❌"
        return 1
    fi
    
    return 0
}

# Basic storage test
run_basic_storage_test() {
    local test_file="$TEST_DIR/temp/basic_test.txt"
    local retrieved_file="$TEST_DIR/temp/basic_retrieved.txt"
    
    # Create test file
    echo "DataMesh basic test - $(date)" > "$test_file"
    echo "Random data: $RANDOM" >> "$test_file"
    echo "Test content line 3" >> "$test_file"
    
    # Store file
    log_debug "Storing test file..."
    local output
    if ! output=$("$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$test_file" 2>&1); then
        log_error "Failed to store test file"
        log_debug "Put command output: $output"
        return 1
    fi
    
    # Extract key from output
    local key=$(echo "$output" | grep -E "(File stored with key:|stored with key)" | grep -oE '[a-f0-9]{64}' | head -1)
    if [[ -z "$key" ]]; then
        log_error "Could not extract file key from output"
        log_debug "Put command output: $output"
        return 1
    fi
    
    log_debug "File stored with key: ${key:0:32}..."
    
    # Retrieve file
    log_debug "Retrieving test file..."
    if ! "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        get "$key" "$retrieved_file" &>/dev/null; then
        log_error "Failed to retrieve test file"
        return 1
    fi
    
    # Verify content
    if ! cmp -s "$test_file" "$retrieved_file"; then
        log_error "Retrieved file content differs from original"
        log_debug "Original file size: $(wc -c < "$test_file")"
        log_debug "Retrieved file size: $(wc -c < "$retrieved_file")"
        return 1
    fi
    
    log_debug "Basic storage test completed successfully"
    return 0
}

# Network connectivity test
run_network_test() {
    # Check if we can list files (tests network connectivity)
    log_debug "Testing network connectivity..."
    if ! "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        list &>/dev/null; then
        log_error "Network connectivity test failed"
        return 1
    fi
    
    log_debug "Network connectivity test completed successfully"
    return 0
}

# Multiple files test
run_multiple_files_test() {
    local test_files=()
    local keys=()
    
    # Create multiple test files
    for i in {1..3}; do
        local test_file="$TEST_DIR/temp/multi_test_$i.txt"
        echo "Multi-file test $i - $(date)" > "$test_file"
        echo "File number: $i" >> "$test_file"
        echo "Random data: $RANDOM" >> "$test_file"
        test_files+=("$test_file")
    done
    
    # Store all files
    for test_file in "${test_files[@]}"; do
        log_debug "Storing file: $(basename "$test_file")"
        local output
        if ! output=$("$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            put "$test_file" 2>&1); then
            log_error "Failed to store file: $(basename "$test_file")"
            return 1
        fi
        
        local key=$(echo "$output" | grep -E "(File stored with key:|stored with key)" | grep -oE '[a-f0-9]{64}' | head -1)
        if [[ -z "$key" ]]; then
            log_error "Could not extract key for file: $(basename "$test_file")"
            return 1
        fi
        keys+=("$key")
    done
    
    # Retrieve and verify all files
    for i in "${!keys[@]}"; do
        local key="${keys[$i]}"
        local original_file="${test_files[$i]}"
        local retrieved_file="$TEST_DIR/temp/multi_retrieved_$((i+1)).txt"
        
        log_debug "Retrieving file $((i+1)): ${key:0:16}..."
        if ! "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            get "$key" "$retrieved_file" &>/dev/null; then
            log_error "Failed to retrieve file $((i+1))"
            return 1
        fi
        
        if ! cmp -s "$original_file" "$retrieved_file"; then
            log_error "Retrieved file $((i+1)) content differs from original"
            return 1
        fi
    done
    
    log_debug "Multiple files test completed successfully"
    return 0
}

# Large file test
run_large_file_test() {
    local large_file="$TEST_DIR/temp/large_test.bin"
    local retrieved_file="$TEST_DIR/temp/large_retrieved.bin"
    
    # Create a larger test file (1MB)
    log_debug "Creating 1MB test file..."
    dd if=/dev/urandom of="$large_file" bs=1024 count=1024 &>/dev/null
    
    # Store large file
    log_debug "Storing large file..."
    local output
    if ! output=$("$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$large_file" 2>&1); then
        log_error "Failed to store large file"
        return 1
    fi
    
    local key=$(echo "$output" | grep -E "(File stored with key:|stored with key)" | grep -oE '[a-f0-9]{64}' | head -1)
    if [[ -z "$key" ]]; then
        log_error "Could not extract key for large file"
        return 1
    fi
    
    # Retrieve large file
    log_debug "Retrieving large file..."
    if ! "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        get "$key" "$retrieved_file" &>/dev/null; then
        log_error "Failed to retrieve large file"
        return 1
    fi
    
    # Verify content
    if ! cmp -s "$large_file" "$retrieved_file"; then
        log_error "Retrieved large file content differs from original"
        local orig_size=$(wc -c < "$large_file")
        local retr_size=$(wc -c < "$retrieved_file")
        log_debug "Original size: $orig_size bytes, Retrieved size: $retr_size bytes"
        return 1
    fi
    
    log_debug "Large file test completed successfully"
    return 0
}

# Main execution
main() {
    echo "=================================================="
    echo "       DataMesh Enhanced Cluster Test"
    echo "=================================================="
    echo ""
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nodes)
                NUM_NODES="$2"
                shift 2
                ;;
            --port)
                BOOTSTRAP_PORT="$2"
                shift 2
                ;;
            --debug)
                LOG_LEVEL="debug"
                shift
                ;;
            --timeout)
                STARTUP_TIMEOUT="$2"
                shift 2
                ;;
            -h|--help)
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --nodes N      Number of nodes to start (default: $NUM_NODES)"
                echo "  --port P       Bootstrap port (default: $BOOTSTRAP_PORT)"
                echo "  --debug        Enable debug logging"
                echo "  --timeout T    Startup timeout in seconds (default: $STARTUP_TIMEOUT)"
                echo "  -h, --help     Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log_info "Starting cluster test with $NUM_NODES nodes on port $BOOTSTRAP_PORT"
    log_info "Test directory: $TEST_DIR"
    log_info "Log level: $LOG_LEVEL"
    
    # Execute test phases
    preflight_checks
    setup_environment
    
    if start_cluster; then
        if run_cluster_tests; then
            log_success "🎉 All tests completed successfully!"
            echo ""
            echo "Cluster is running and fully functional."
            echo "You can connect to it using:"
            echo "  Bootstrap Peer: $BOOTSTRAP_PEER_ID"
            echo "  Bootstrap Address: $BOOTSTRAP_ADDR"
            echo ""
            echo "Press Enter to stop the cluster and exit..."
            read -r
        else
            log_error "Tests failed!"
            exit 1
        fi
    else
        log_error "Failed to start cluster!"
        exit 1
    fi
}

# Run main function
main "$@"