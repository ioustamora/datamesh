#!/bin/bash

# Advanced DataMesh Cluster Test Suite
# 
# This script provides comprehensive testing of DataMesh distributed storage
# with real local network setup, multi-node operations, and advanced scenarios.

set -euo pipefail

# Configuration
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
readonly TEST_DIR="${PROJECT_ROOT}/cluster_test_$(date +%Y%m%d_%H%M%S)"
readonly BINARY_PATH="${PROJECT_ROOT}/target/release/datamesh"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Unicode symbols
readonly CHECKMARK="✅"
readonly CROSSMARK="❌"
readonly HOURGLASS="⏳"
readonly ROCKET="🚀"
readonly GEAR="⚙️"
readonly NETWORK="🌐"
readonly SHIELD="🛡️"
readonly LIGHTNING="⚡"

# Test configuration
declare -A TEST_CONFIG=(
    [NUM_NODES]=7
    [BASE_PORT]=40871
    [BOOTSTRAP_PORT]=40871
    [API_BASE_PORT]=8080
    [TEST_TIMEOUT]=300
    [FILE_SIZE_SMALL]=1024
    [FILE_SIZE_MEDIUM]=102400
    [FILE_SIZE_LARGE]=1048576
    [REPLICATION_FACTOR]=3
    [CONCURRENT_OPERATIONS]=20
)

# Global test state
declare -A TEST_RESULTS=(
    [TESTS_PASSED]=0
    [TESTS_FAILED]=0
    [PERFORMANCE_METRICS]=""
)

declare -a NODE_PIDS=()
declare -a NODE_PORTS=()
declare -a API_PORTS=()

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} ${CHECKMARK} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} ${CROSSMARK} $*"
}

log_progress() {
    echo -e "${PURPLE}[PROGRESS]${NC} ${HOURGLASS} $*"
}

log_test() {
    echo -e "${CYAN}[TEST]${NC} ${GEAR} $*"
}

# Setup functions
setup_test_environment() {
    log_info "Setting up test environment in: ${TEST_DIR}"
    
    mkdir -p "${TEST_DIR}"/{data,logs,results,configs}
    
    # Build the project
    log_progress "Building DataMesh..."
    cd "${PROJECT_ROOT}"
    cargo build --release --quiet || {
        log_error "Failed to build DataMesh"
        exit 1
    }
    
    if [[ ! -f "${BINARY_PATH}" ]]; then
        log_error "Binary not found at ${BINARY_PATH}"
        exit 1
    fi
    
    log_success "Test environment setup complete"
}

generate_node_config() {
    local node_id=$1
    local port=$2
    local api_port=$3
    local is_bootstrap=$4
    local config_file="${TEST_DIR}/configs/node_${node_id}.toml"
    
    cat > "${config_file}" << EOF
[network]
port = ${port}
discovery_enabled = true
$(if [[ "${is_bootstrap}" == "false" ]]; then
    echo "bootstrap_peers = [\"/ip4/127.0.0.1/tcp/${TEST_CONFIG[BOOTSTRAP_PORT]}\"]"
fi)

[storage]
data_dir = "${TEST_DIR}/data/node_${node_id}/storage"
max_file_size = 104857600  # 100MB
replication_factor = ${TEST_CONFIG[REPLICATION_FACTOR]}

[database]
db_path = "${TEST_DIR}/data/node_${node_id}/datamesh.db"

[security]
encryption_enabled = true
audit_enabled = true
audit_log_path = "${TEST_DIR}/logs/node_${node_id}_audit.log"

[api]
enabled = true
host = "127.0.0.1"
port = ${api_port}
max_request_size = 104857600

[monitoring]
enabled = true
metrics_interval_seconds = 5
health_check_interval_seconds = 10

[performance]
chunk_size = 65536
concurrent_uploads = 4
cache_size_mb = 64

[logging]
level = "info"
file_path = "${TEST_DIR}/logs/node_${node_id}.log"
EOF
    
    echo "${config_file}"
}

start_node() {
    local node_id=$1
    local is_bootstrap=$2
    local port=$((TEST_CONFIG[BASE_PORT] + node_id))
    local api_port=$((TEST_CONFIG[API_BASE_PORT] + node_id))
    
    NODE_PORTS[node_id]=${port}
    API_PORTS[node_id]=${api_port}
    
    mkdir -p "${TEST_DIR}/data/node_${node_id}"/{storage,keys}
    
    local config_file
    config_file=$(generate_node_config "${node_id}" "${port}" "${api_port}" "${is_bootstrap}")
    
    log_progress "Starting node ${node_id} on port ${port} (API: ${api_port})"
    
    local cmd_args=(
        "--config" "${config_file}"
        "--data-dir" "${TEST_DIR}/data/node_${node_id}"
        "--non-interactive"
    )
    
    if [[ "${is_bootstrap}" == "true" ]]; then
        cmd_args+=("bootstrap")
    else
        cmd_args+=("service")
    fi
    
    # Start node in background
    RUST_LOG=info "${BINARY_PATH}" "${cmd_args[@]}" \
        > "${TEST_DIR}/logs/node_${node_id}.log" 2>&1 &
    
    local pid=$!
    NODE_PIDS[node_id]=${pid}
    
    # Wait for node to start
    local max_attempts=30
    local attempt=0
    
    while [[ ${attempt} -lt ${max_attempts} ]]; do
        if kill -0 "${pid}" 2>/dev/null; then
            # Check if API is responding (for non-bootstrap nodes)
            if [[ "${is_bootstrap}" == "false" ]] && command -v curl >/dev/null; then
                if curl -s "http://127.0.0.1:${api_port}/api/v1/health" >/dev/null 2>&1; then
                    break
                fi
            else
                # For bootstrap nodes, just check if process is running
                sleep 2
                if kill -0 "${pid}" 2>/dev/null; then
                    break
                fi
            fi
        else
            log_error "Node ${node_id} failed to start (PID ${pid})"
            return 1
        fi
        
        ((attempt++))
        sleep 1
    done
    
    if [[ ${attempt} -eq ${max_attempts} ]]; then
        log_error "Node ${node_id} failed to become ready within ${max_attempts} seconds"
        return 1
    fi
    
    log_success "Node ${node_id} started successfully (PID: ${pid})"
    return 0
}

start_cluster() {
    log_info "${ROCKET} Starting DataMesh cluster with ${TEST_CONFIG[NUM_NODES]} nodes"
    
    # Start bootstrap node
    if ! start_node 0 true; then
        log_error "Failed to start bootstrap node"
        return 1
    fi
    
    # Wait for bootstrap node to stabilize
    sleep 3
    
    # Start regular nodes
    for ((i=1; i<TEST_CONFIG[NUM_NODES]; i++)); do
        if ! start_node ${i} false; then
            log_error "Failed to start node ${i}"
            return 1
        fi
        sleep 1  # Stagger node startup
    done
    
    # Wait for network formation
    log_progress "Waiting for network formation..."
    sleep 5
    
    log_success "Cluster started successfully"
    return 0
}

stop_cluster() {
    log_info "Stopping cluster..."
    
    for i in "${!NODE_PIDS[@]}"; do
        local pid=${NODE_PIDS[i]}
        if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
            log_progress "Stopping node ${i} (PID: ${pid})"
            kill -TERM "${pid}" 2>/dev/null || true
            
            # Wait for graceful shutdown
            local attempts=0
            while [[ ${attempts} -lt 10 ]] && kill -0 "${pid}" 2>/dev/null; do
                sleep 1
                ((attempts++))
            done
            
            # Force kill if still running
            if kill -0 "${pid}" 2>/dev/null; then
                kill -KILL "${pid}" 2>/dev/null || true
                log_warning "Force killed node ${i}"
            else
                log_success "Node ${i} stopped gracefully"
            fi
        fi
    done
    
    # Save PIDs for reference
    printf '%s\n' "${NODE_PIDS[@]}" > "${TEST_DIR}/pids.txt"
}

# Test functions
test_network_formation() {
    log_test "Testing network formation and peer discovery"
    
    local test_passed=true
    
    # Test that all nodes are running
    for i in "${!NODE_PIDS[@]}"; do
        local pid=${NODE_PIDS[i]}
        if ! kill -0 "${pid}" 2>/dev/null; then
            log_error "Node ${i} is not running"
            test_passed=false
        fi
    done
    
    # Test API health endpoints
    for i in $(seq 1 $((TEST_CONFIG[NUM_NODES] - 1))); do
        local api_port=${API_PORTS[i]}
        if command -v curl >/dev/null; then
            if ! curl -s -f "http://127.0.0.1:${api_port}/api/v1/health" >/dev/null; then
                log_error "Node ${i} API health check failed"
                test_passed=false
            fi
        fi
    done
    
    if [[ "${test_passed}" == "true" ]]; then
        log_success "Network formation test passed"
        ((TEST_RESULTS[TESTS_PASSED]++))
        return 0
    else
        log_error "Network formation test failed"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

test_basic_file_operations() {
    log_test "Testing basic file operations across cluster"
    
    local test_passed=true
    local test_file="${TEST_DIR}/test_files/basic_test.txt"
    mkdir -p "$(dirname "${test_file}")"
    
    # Create test file
    echo "Basic test content - $(date)" > "${test_file}"
    
    # Test file upload using CLI
    log_progress "Testing file upload..."
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_1.toml" \
        --non-interactive \
        put "${test_file}" basic_test \
        > "${TEST_DIR}/logs/basic_put.log" 2>&1; then
        log_success "File upload successful"
    else
        log_error "File upload failed"
        test_passed=false
    fi
    
    # Test file download
    log_progress "Testing file download..."
    local download_file="${TEST_DIR}/test_files/basic_test_downloaded.txt"
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_2.toml" \
        --non-interactive \
        get basic_test "${download_file}" \
        > "${TEST_DIR}/logs/basic_get.log" 2>&1; then
        
        # Verify file content
        if cmp -s "${test_file}" "${download_file}"; then
            log_success "File download and verification successful"
        else
            log_error "Downloaded file content mismatch"
            test_passed=false
        fi
    else
        log_error "File download failed"
        test_passed=false
    fi
    
    # Test file listing
    log_progress "Testing file listing..."
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_3.toml" \
        --non-interactive \
        list \
        > "${TEST_DIR}/logs/basic_list.log" 2>&1; then
        
        if grep -q "basic_test" "${TEST_DIR}/logs/basic_list.log"; then
            log_success "File listing successful"
        else
            log_error "File not found in listing"
            test_passed=false
        fi
    else
        log_error "File listing failed"
        test_passed=false
    fi
    
    if [[ "${test_passed}" == "true" ]]; then
        log_success "Basic file operations test passed"
        ((TEST_RESULTS[TESTS_PASSED]++))
        return 0
    else
        log_error "Basic file operations test failed"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

test_concurrent_operations() {
    log_test "Testing concurrent file operations"
    
    local test_passed=true
    local concurrent_ops=${TEST_CONFIG[CONCURRENT_OPERATIONS]}
    
    log_progress "Starting ${concurrent_ops} concurrent operations..."
    
    # Create test files
    local test_files_dir="${TEST_DIR}/test_files/concurrent"
    mkdir -p "${test_files_dir}"
    
    local pids=()
    
    for ((i=0; i<concurrent_ops; i++)); do
        local test_file="${test_files_dir}/concurrent_${i}.txt"
        local node_config="${TEST_DIR}/configs/node_$((i % (TEST_CONFIG[NUM_NODES] - 1) + 1)).toml"
        
        # Create unique test content
        echo "Concurrent test content ${i} - $(date) - ${RANDOM}" > "${test_file}"
        
        # Start concurrent upload
        (
            if "${BINARY_PATH}" \
                --config "${node_config}" \
                --non-interactive \
                put "${test_file}" "concurrent_${i}" \
                > "${TEST_DIR}/logs/concurrent_put_${i}.log" 2>&1; then
                echo "SUCCESS:${i}"
            else
                echo "FAILURE:${i}"
            fi
        ) &
        
        pids+=($!)
    done
    
    # Wait for all operations to complete
    local successful_ops=0
    local failed_ops=0
    
    for pid in "${pids[@]}"; do
        if wait "${pid}"; then
            ((successful_ops++))
        else
            ((failed_ops++))
        fi
    done
    
    log_info "Concurrent operations completed: ${successful_ops} successful, ${failed_ops} failed"
    
    # Test concurrent downloads
    log_progress "Testing concurrent downloads..."
    
    pids=()
    for ((i=0; i<concurrent_ops/2; i++)); do
        local download_file="${test_files_dir}/concurrent_downloaded_${i}.txt"
        local node_config="${TEST_DIR}/configs/node_$((i % (TEST_CONFIG[NUM_NODES] - 1) + 1)).toml"
        
        (
            if "${BINARY_PATH}" \
                --config "${node_config}" \
                --non-interactive \
                get "concurrent_${i}" "${download_file}" \
                > "${TEST_DIR}/logs/concurrent_get_${i}.log" 2>&1; then
                echo "SUCCESS:${i}"
            else
                echo "FAILURE:${i}"
            fi
        ) &
        
        pids+=($!)
    done
    
    # Wait for downloads
    local successful_downloads=0
    for pid in "${pids[@]}"; do
        if wait "${pid}"; then
            ((successful_downloads++))
        fi
    done
    
    # Evaluate test results
    local success_rate=$((successful_ops * 100 / concurrent_ops))
    local download_rate=$((successful_downloads * 100 / (concurrent_ops / 2)))
    
    if [[ ${success_rate} -ge 80 && ${download_rate} -ge 80 ]]; then
        log_success "Concurrent operations test passed (${success_rate}% upload, ${download_rate}% download success)"
        ((TEST_RESULTS[TESTS_PASSED]++))
        TEST_RESULTS[PERFORMANCE_METRICS]+="concurrent_upload_success_rate:${success_rate}% "
        TEST_RESULTS[PERFORMANCE_METRICS]+="concurrent_download_success_rate:${download_rate}% "
        return 0
    else
        log_error "Concurrent operations test failed (${success_rate}% upload, ${download_rate}% download success)"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

test_fault_tolerance() {
    log_test "Testing fault tolerance with node failures"
    
    local test_passed=true
    
    # Store a test file first
    local test_file="${TEST_DIR}/test_files/fault_test.txt"
    mkdir -p "$(dirname "${test_file}")"
    echo "Fault tolerance test content - $(date)" > "${test_file}"
    
    if ! "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_1.toml" \
        --non-interactive \
        put "${test_file}" fault_test \
        > "${TEST_DIR}/logs/fault_put.log" 2>&1; then
        log_error "Failed to store test file for fault tolerance test"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
    
    # Stop some nodes (but not bootstrap)
    local nodes_to_stop=2
    local stopped_nodes=()
    
    for ((i=1; i<=nodes_to_stop; i++)); do
        local node_idx=$((TEST_CONFIG[NUM_NODES] - i))
        local pid=${NODE_PIDS[node_idx]}
        
        if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
            log_progress "Stopping node ${node_idx} for fault tolerance test"
            kill -TERM "${pid}"
            stopped_nodes+=(${node_idx})
        fi
    done
    
    # Wait for nodes to stop
    sleep 3
    
    # Try to retrieve the file from remaining nodes
    log_progress "Testing file retrieval with ${nodes_to_stop} nodes down..."
    local download_file="${TEST_DIR}/test_files/fault_test_downloaded.txt"
    
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_1.toml" \
        --non-interactive \
        get fault_test "${download_file}" \
        > "${TEST_DIR}/logs/fault_get.log" 2>&1; then
        
        if cmp -s "${test_file}" "${download_file}"; then
            log_success "File retrieval successful with nodes down"
        else
            log_error "Retrieved file content mismatch during fault test"
            test_passed=false
        fi
    else
        log_error "File retrieval failed with nodes down"
        test_passed=false
    fi
    
    # Restart stopped nodes
    for node_idx in "${stopped_nodes[@]}"; do
        log_progress "Restarting node ${node_idx}"
        start_node "${node_idx}" false
    done
    
    # Wait for network to stabilize
    sleep 5
    
    if [[ "${test_passed}" == "true" ]]; then
        log_success "Fault tolerance test passed"
        ((TEST_RESULTS[TESTS_PASSED]++))
        return 0
    else
        log_error "Fault tolerance test failed"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

test_large_file_operations() {
    log_test "Testing large file operations"
    
    local test_passed=true
    local large_file="${TEST_DIR}/test_files/large_test.bin"
    mkdir -p "$(dirname "${large_file}")"
    
    # Create large test file (10MB)
    log_progress "Creating large test file (10MB)..."
    dd if=/dev/urandom of="${large_file}" bs=1M count=10 2>/dev/null
    
    local start_time=$(date +%s)
    
    # Upload large file
    log_progress "Uploading large file..."
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_1.toml" \
        --non-interactive \
        put "${large_file}" large_test \
        > "${TEST_DIR}/logs/large_put.log" 2>&1; then
        log_success "Large file upload successful"
    else
        log_error "Large file upload failed"
        test_passed=false
    fi
    
    local upload_time=$(($(date +%s) - start_time))
    
    # Download large file
    log_progress "Downloading large file..."
    local download_file="${TEST_DIR}/test_files/large_test_downloaded.bin"
    start_time=$(date +%s)
    
    if "${BINARY_PATH}" \
        --config "${TEST_DIR}/configs/node_2.toml" \
        --non-interactive \
        get large_test "${download_file}" \
        > "${TEST_DIR}/logs/large_get.log" 2>&1; then
        
        local download_time=$(($(date +%s) - start_time))
        
        # Verify file integrity
        if cmp -s "${large_file}" "${download_file}"; then
            log_success "Large file download and verification successful"
            log_info "Upload time: ${upload_time}s, Download time: ${download_time}s"
        else
            log_error "Large file content mismatch"
            test_passed=false
        fi
    else
        log_error "Large file download failed"
        test_passed=false
    fi
    
    if [[ "${test_passed}" == "true" ]]; then
        log_success "Large file operations test passed"
        ((TEST_RESULTS[TESTS_PASSED]++))
        TEST_RESULTS[PERFORMANCE_METRICS]+="large_file_upload_time:${upload_time}s "
        TEST_RESULTS[PERFORMANCE_METRICS]+="large_file_download_time:${download_time}s "
        return 0
    else
        log_error "Large file operations test failed"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

test_api_endpoints() {
    log_test "Testing REST API endpoints"
    
    if ! command -v curl >/dev/null; then
        log_warning "curl not available, skipping API tests"
        return 0
    fi
    
    local test_passed=true
    local api_port=${API_PORTS[1]}
    local base_url="http://127.0.0.1:${api_port}/api/v1"
    
    # Test health endpoint
    log_progress "Testing health endpoint..."
    if curl -s -f "${base_url}/health" > "${TEST_DIR}/logs/api_health.log"; then
        log_success "Health endpoint test passed"
    else
        log_error "Health endpoint test failed"
        test_passed=false
    fi
    
    # Test stats endpoint (may require auth)
    log_progress "Testing stats endpoint..."
    curl -s "${base_url}/stats" > "${TEST_DIR}/logs/api_stats.log" 2>&1 || true
    
    if [[ "${test_passed}" == "true" ]]; then
        log_success "API endpoints test passed"
        ((TEST_RESULTS[TESTS_PASSED]++))
        return 0
    else
        log_error "API endpoints test failed"
        ((TEST_RESULTS[TESTS_FAILED]++))
        return 1
    fi
}

run_performance_benchmarks() {
    log_test "Running performance benchmarks"
    
    if ! command -v hyperfine >/dev/null; then
        log_warning "hyperfine not available, running basic performance test"
        
        # Basic timing test
        local test_file="${TEST_DIR}/test_files/perf_test.txt"
        echo "Performance test content" > "${test_file}"
        
        local start_time=$(date +%s%N)
        "${BINARY_PATH}" \
            --config "${TEST_DIR}/configs/node_1.toml" \
            --non-interactive \
            put "${test_file}" perf_test \
            > "${TEST_DIR}/logs/perf_basic.log" 2>&1
        local end_time=$(date +%s%N)
        
        local duration_ms=$(((end_time - start_time) / 1000000))
        log_info "Basic operation took ${duration_ms}ms"
        TEST_RESULTS[PERFORMANCE_METRICS]+="basic_operation_time:${duration_ms}ms "
        
        ((TEST_RESULTS[TESTS_PASSED]++))
        return 0
    fi
    
    # Advanced benchmarking with hyperfine
    local test_file="${TEST_DIR}/test_files/bench_test.txt"
    echo "Benchmark test content" > "${test_file}"
    
    log_progress "Running PUT operation benchmark..."
    hyperfine \
        --warmup 2 \
        --runs 10 \
        --export-json "${TEST_DIR}/results/put_benchmark.json" \
        "${BINARY_PATH} --config ${TEST_DIR}/configs/node_1.toml --non-interactive put ${test_file} bench_test_{}" \
        > "${TEST_DIR}/logs/put_benchmark.log" 2>&1 || true
    
    log_progress "Running GET operation benchmark..."
    hyperfine \
        --warmup 2 \
        --runs 10 \
        --export-json "${TEST_DIR}/results/get_benchmark.json" \
        "${BINARY_PATH} --config ${TEST_DIR}/configs/node_2.toml --non-interactive get bench_test_1 ${TEST_DIR}/test_files/bench_downloaded_{}.txt" \
        > "${TEST_DIR}/logs/get_benchmark.log" 2>&1 || true
    
    log_success "Performance benchmarks completed"
    ((TEST_RESULTS[TESTS_PASSED]++))
    return 0
}

# Report generation
generate_test_report() {
    local report_file="${TEST_DIR}/results/test_report.md"
    
    cat > "${report_file}" << EOF
# DataMesh Cluster Test Report

**Test Date:** $(date)
**Test Duration:** ${SECONDS} seconds
**Cluster Size:** ${TEST_CONFIG[NUM_NODES]} nodes

## Test Results Summary

- **Tests Passed:** ${TEST_RESULTS[TESTS_PASSED]}
- **Tests Failed:** ${TEST_RESULTS[TESTS_FAILED]}
- **Success Rate:** $(( TEST_RESULTS[TESTS_PASSED] * 100 / (TEST_RESULTS[TESTS_PASSED] + TEST_RESULTS[TESTS_FAILED]) ))%

## Performance Metrics

${TEST_RESULTS[PERFORMANCE_METRICS]}

## Node Information

| Node ID | Port | API Port | PID | Status |
|---------|------|----------|-----|--------|
EOF

    for i in "${!NODE_PIDS[@]}"; do
        local status="Stopped"
        if kill -0 "${NODE_PIDS[i]}" 2>/dev/null; then
            status="Running"
        fi
        
        echo "| ${i} | ${NODE_PORTS[i]} | ${API_PORTS[i]} | ${NODE_PIDS[i]} | ${status} |" >> "${report_file}"
    done
    
    cat >> "${report_file}" << EOF

## Log Files

- Cluster logs: \`${TEST_DIR}/logs/\`
- Configuration files: \`${TEST_DIR}/configs/\`
- Test results: \`${TEST_DIR}/results/\`

## Test Environment

- **Test Directory:** ${TEST_DIR}
- **Binary Path:** ${BINARY_PATH}
- **Replication Factor:** ${TEST_CONFIG[REPLICATION_FACTOR]}
- **Base Port:** ${TEST_CONFIG[BASE_PORT]}

EOF

    log_info "Test report generated: ${report_file}"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    stop_cluster
    
    # Archive logs if tests completed
    if [[ ${TEST_RESULTS[TESTS_PASSED]} -gt 0 || ${TEST_RESULTS[TESTS_FAILED]} -gt 0 ]]; then
        local archive_name="cluster_test_results_$(date +%Y%m%d_%H%M%S).tar.gz"
        tar -czf "${archive_name}" -C "$(dirname "${TEST_DIR}")" "$(basename "${TEST_DIR}")" 2>/dev/null || true
        log_info "Test results archived: ${archive_name}"
    fi
}

# Main test execution
main() {
    echo -e "${ROCKET} ${CYAN}DataMesh Advanced Cluster Test Suite${NC}"
    echo -e "${NETWORK} Testing distributed storage with ${TEST_CONFIG[NUM_NODES]} nodes"
    echo

    # Setup
    trap cleanup EXIT
    setup_test_environment
    
    # Start cluster
    if ! start_cluster; then
        log_error "Failed to start cluster"
        exit 1
    fi
    
    log_info "${LIGHTNING} Running comprehensive test suite..."
    echo
    
    # Run all tests
    test_network_formation
    test_basic_file_operations
    test_concurrent_operations
    test_large_file_operations
    test_fault_tolerance
    test_api_endpoints
    run_performance_benchmarks
    
    # Generate report
    generate_test_report
    
    # Final results
    echo
    log_info "${GEAR} Test Suite Complete"
    echo -e "${GREEN}Tests Passed: ${TEST_RESULTS[TESTS_PASSED]}${NC}"
    echo -e "${RED}Tests Failed: ${TEST_RESULTS[TESTS_FAILED]}${NC}"
    
    if [[ ${TEST_RESULTS[TESTS_FAILED]} -eq 0 ]]; then
        echo -e "\n${GREEN}${CHECKMARK} All tests passed! DataMesh cluster is working correctly.${NC}"
        exit 0
    else
        echo -e "\n${RED}${CROSSMARK} Some tests failed. Check logs for details.${NC}"
        exit 1
    fi
}

# Script entry point
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi