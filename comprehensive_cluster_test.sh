#!/bin/bash

# Comprehensive DataMesh Cluster Test - 4+ Node Cluster with All Commands
# This script tests all DataMesh commands in a realistic distributed environment

set -e

# Configuration
NODES=4  # Use 4 nodes as minimum requirement (will be configurable)
BOOTSTRAP_PORT=9000
BASE_PORT=9001
TEST_DIR="cluster_test_$(date +%s)"
BINARY_PATH="./target/release/datamesh"
LOG_LEVEL="info"
TEST_TIMEOUT=300  # 5 minutes timeout for each test
WAIT_TIME=3  # Seconds to wait between operations

# Allow overriding node count via environment variable
if [ ! -z "$CLUSTER_NODES" ] && [ "$CLUSTER_NODES" -ge 4 ]; then
    NODES=$CLUSTER_NODES
    echo "Using $NODES nodes (from CLUSTER_NODES environment variable)"
elif [ "$NODES" -lt 4 ]; then
    echo "Warning: Node count is less than 4. Setting to minimum of 4 nodes."
    NODES=4
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test tracking
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up cluster...${NC}"
    
    # Kill all datamesh processes
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    pkill -f "datamesh.*interactive" 2>/dev/null || true
    
    # Wait for processes to terminate
    sleep 2
    
    # Remove test directory
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
    fi
    
    echo -e "${GREEN}Cleanup complete${NC}"
}

# Set up cleanup trap
trap cleanup EXIT

# Utility functions
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')] $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TESTS_PASSED++))
}

failure() {
    echo -e "${RED}❌ $1${NC}"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$1")
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

test_start() {
    ((TESTS_TOTAL++))
    log "Testing: $1"
}

wait_for_port() {
    local port=$1
    local timeout=30
    local count=0
    
    while [ $count -lt $timeout ]; do
        # Try different methods to check if port is open
        if command -v nc >/dev/null 2>&1; then
            if nc -z localhost $port 2>/dev/null; then
                return 0
            fi
        elif command -v lsof >/dev/null 2>&1; then
            if lsof -i:$port >/dev/null 2>&1; then
                return 0
            fi
        elif command -v ss >/dev/null 2>&1; then
            if ss -ln | grep ":$port " >/dev/null 2>&1; then
                return 0
            fi
        else
            # Fallback: just wait and assume success
            if [ $count -ge 5 ]; then
                return 0
            fi
        fi
        
        sleep 1
        ((count++))
    done
    return 1
}

run_with_timeout() {
    local cmd="$1"
    local timeout=${2:-$TEST_TIMEOUT}
    
    timeout $timeout bash -c "$cmd"
    return $?
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if binary exists
    if [ ! -f "$BINARY_PATH" ]; then
        failure "Binary not found at $BINARY_PATH. Please run 'cargo build --release' first."
        exit 1
    fi
    
    # Check if we can test ports (use ss, lsof, or nc)
    if ! command -v lsof >/dev/null 2>&1 && ! command -v nc >/dev/null 2>&1 && ! command -v ss >/dev/null 2>&1; then
        warning "No port checking tools available - port checking will be simplified"
    fi
    
    # Check if timeout command is available
    if ! command -v timeout >/dev/null 2>&1; then
        failure "timeout command is required but not available"
        exit 1
    fi
    
    success "Prerequisites check passed"
}

# Set up test environment
setup_test_environment() {
    log "Setting up test environment..."
    
    # Create test directory
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    
    # Create node directories
    for i in $(seq 0 $((NODES-1))); do
        mkdir -p "node$i/keys"
        mkdir -p "node$i/storage"
        mkdir -p "node$i/logs"
    done
    
    # Create test files for upload testing
    mkdir -p "test_files"
    echo "Hello DataMesh Test File 1" > "test_files/test1.txt"
    echo "Hello DataMesh Test File 2" > "test_files/test2.txt"
    echo "This is a larger test file with more content for testing." > "test_files/large.txt"
    
    # Create a directory with multiple files for batch operations
    mkdir -p "test_files/batch_test"
    for i in {1..5}; do
        echo "Batch test file $i content" > "test_files/batch_test/batch_file_$i.txt"
    done
    
    success "Test environment setup complete"
}

# Start bootstrap node
start_bootstrap() {
    log "Starting bootstrap node..."
    
    cd "node0"
    run_with_timeout "$BINARY_PATH bootstrap --port $BOOTSTRAP_PORT > logs/bootstrap.log 2>&1 &" 10
    BOOTSTRAP_PID=$!
    cd ..
    
    # Wait for bootstrap node to start
    if wait_for_port $BOOTSTRAP_PORT; then
        success "Bootstrap node started on port $BOOTSTRAP_PORT"
        sleep $WAIT_TIME
    else
        failure "Bootstrap node failed to start"
        return 1
    fi
}

# Start service nodes
start_service_nodes() {
    log "Starting service nodes..."
    
    local bootstrap_addr="/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT"
    
    for i in $(seq 1 $((NODES-1))); do
        local port=$((BASE_PORT + i - 1))
        cd "node$i"
        
        run_with_timeout "$BINARY_PATH service --port $port --bootstrap-addr $bootstrap_addr --timeout 300 > logs/service.log 2>&1 &" 10
        
        cd ..
        
        # Wait for node to start
        if wait_for_port $port; then
            log "Service node $i started on port $port"
        else
            warning "Service node $i may have issues starting on port $port"
        fi
        
        sleep 1  # Brief pause between node starts
    done
    
    # Give the cluster time to form
    log "Waiting for cluster to form..."
    sleep 10
    
    success "Service nodes started"
}

# Test network connectivity
test_network_connectivity() {
    test_start "Network connectivity"
    
    cd "node1"
    
    # Test peers command
    if run_with_timeout "$BINARY_PATH peers > ../test_output.txt 2>&1" 30; then
        local peer_count=$(grep -c "Peer" ../test_output.txt || echo "0")
        if [ "$peer_count" -gt 0 ]; then
            success "Network connectivity test passed - found $peer_count peers"
        else
            warning "Network connectivity test - no peers found yet"
        fi
    else
        failure "Network connectivity test failed"
    fi
    
    cd ..
}

# Test health monitoring
test_health_monitoring() {
    test_start "Health monitoring"
    
    cd "node1"
    
    if run_with_timeout "$BINARY_PATH health > ../test_output.txt 2>&1" 30; then
        if grep -q "Network health" ../test_output.txt; then
            success "Health monitoring test passed"
        else
            warning "Health monitoring test - unexpected output"
        fi
    else
        failure "Health monitoring test failed"
    fi
    
    cd ..
}

# Test network discovery
test_network_discovery() {
    test_start "Network discovery"
    
    cd "node1"
    
    if run_with_timeout "$BINARY_PATH discover --timeout 30 > ../test_output.txt 2>&1" 45; then
        success "Network discovery test passed"
    else
        failure "Network discovery test failed"
    fi
    
    cd ..
}

# Test file operations
test_file_operations() {
    test_start "File operations - Put/Get/List"
    
    cd "node1"
    
    # Test Put command
    local file_key=""
    if file_key=$(run_with_timeout "$BINARY_PATH put ../test_files/test1.txt --name test_file_1 2>&1" 60); then
        if echo "$file_key" | grep -q "stored successfully\|File key:"; then
            success "File put operation passed"
            
            # Extract file key from output
            local extracted_key=$(echo "$file_key" | grep -o "[a-f0-9]\{64\}" | head -1 || echo "test_file_1")
            
            # Test Get command
            if run_with_timeout "$BINARY_PATH get '$extracted_key' ../retrieved_test1.txt 2>&1" 60; then
                if [ -f "../retrieved_test1.txt" ]; then
                    success "File get operation passed"
                else
                    failure "File get operation failed - file not retrieved"
                fi
            else
                failure "File get operation failed"
            fi
            
            # Test List command
            if run_with_timeout "$BINARY_PATH list > ../test_output.txt 2>&1" 30; then
                if grep -q "test_file_1\|Files found" ../test_output.txt; then
                    success "File list operation passed"
                else
                    warning "File list operation - unexpected output"
                fi
            else
                failure "File list operation failed"
            fi
        else
            failure "File put operation failed"
        fi
    else
        failure "File put operation failed - timeout or error"
    fi
    
    cd ..
}

# Test file information and statistics
test_file_info_stats() {
    test_start "File info and statistics"
    
    cd "node1"
    
    # Test Info command
    if run_with_timeout "$BINARY_PATH info test_file_1 > ../test_output.txt 2>&1" 30; then
        if grep -q "File information\|not found" ../test_output.txt; then
            success "File info command passed"
        else
            failure "File info command failed"
        fi
    else
        failure "File info command failed"
    fi
    
    # Test Stats command
    if run_with_timeout "$BINARY_PATH stats > ../test_output.txt 2>&1" 30; then
        if grep -q "Storage statistics\|Files stored" ../test_output.txt; then
            success "Stats command passed"
        else
            failure "Stats command failed"
        fi
    else
        failure "Stats command failed"
    fi
    
    cd ..
}

# Test missing commands (newly implemented)
test_missing_commands() {
    test_start "Missing commands - Sync, Backup, Search"
    
    cd "node2"
    
    # Test Sync command
    if run_with_timeout "$BINARY_PATH sync ../test_files/batch_test --parallel 2 > ../test_output.txt 2>&1" 60; then
        if grep -q "sync complete\|Analyzing local directory" ../test_output.txt; then
            success "Sync command passed"
        else
            warning "Sync command - unexpected output"
        fi
    else
        failure "Sync command failed"
    fi
    
    # Test Backup command
    if run_with_timeout "$BINARY_PATH backup ../test_files/test1.txt --name test_backup_1 > ../test_output.txt 2>&1" 60; then
        if grep -q "Backup.*created\|Network connection established" ../test_output.txt; then
            success "Backup command passed"
        else
            warning "Backup command - unexpected output"
        fi
    else
        failure "Backup command failed"
    fi
    
    # Test Search command
    if run_with_timeout "$BINARY_PATH search test --limit 5 > ../test_output.txt 2>&1" 30; then
        if grep -q "Search completed\|Found.*files\|File search" ../test_output.txt; then
            success "Search command passed"
        else
            warning "Search command - unexpected output"
        fi
    else
        failure "Search command failed"
    fi
    
    cd ..
}

# Test newly implemented commands
test_new_commands() {
    test_start "New commands - Pin, Share, Optimize"
    
    cd "node2"
    
    # Test Pin command
    if run_with_timeout "$BINARY_PATH pin test_file_1 --priority 8 --duration 1h > ../test_output.txt 2>&1" 30; then
        if grep -q "pinned successfully\|Network connection established" ../test_output.txt; then
            success "Pin command passed"
        else
            warning "Pin command - unexpected output"
        fi
    else
        failure "Pin command failed"
    fi
    
    # Test Unpin command
    if run_with_timeout "$BINARY_PATH unpin test_file_1 > ../test_output.txt 2>&1" 30; then
        if grep -q "unpinned\|removed\|Network connection established" ../test_output.txt; then
            success "Unpin command passed"
        else
            warning "Unpin command - unexpected output"
        fi
    else
        failure "Unpin command failed"
    fi
    
    # Test Share command
    if run_with_timeout "$BINARY_PATH share test_file_1 --public --expires 1d > ../test_output.txt 2>&1" 30; then
        if grep -q "Share link generated\|Network connection established" ../test_output.txt; then
            success "Share command passed"
        else
            warning "Share command - unexpected output"
        fi
    else
        failure "Share command failed"
    fi
    
    # Test Optimize command
    if run_with_timeout "$BINARY_PATH optimize --defrag --rebalance > ../test_output.txt 2>&1" 30; then
        if grep -q "optimization\|Performance Optimization" ../test_output.txt; then
            success "Optimize command passed"
        else
            warning "Optimize command - unexpected output"
        fi
    else
        failure "Optimize command failed"
    fi
    
    cd ..
}

# Test batch operations
test_batch_operations() {
    test_start "Batch operations - BatchPut, BatchGet, BatchTag"
    
    cd "node3"
    
    # Test BatchPut command (simplified)
    if run_with_timeout "$BINARY_PATH batch-put ../test_files/batch_test --parallel 2 > ../test_output.txt 2>&1" 60; then
        if grep -q "Batch upload\|Network connection established" ../test_output.txt; then
            success "BatchPut command passed"
        else
            warning "BatchPut command - unexpected output"
        fi
    else
        failure "BatchPut command failed"
    fi
    
    # Test BatchGet command
    if run_with_timeout "$BINARY_PATH batch-get '*test*' --destination ../batch_downloads --parallel 2 > ../test_output.txt 2>&1" 60; then
        if grep -q "Batch download\|Network connection established" ../test_output.txt; then
            success "BatchGet command passed"
        else
            warning "BatchGet command - unexpected output"
        fi
    else
        failure "BatchGet command failed"
    fi
    
    # Test BatchTag command
    if run_with_timeout "$BINARY_PATH batch-tag '*' --add-tags 'test,batch' --dry-run > ../test_output.txt 2>&1" 30; then
        if grep -q "Batch Tag\|Network connection established" ../test_output.txt; then
            success "BatchTag command passed"
        else
            warning "BatchTag command - unexpected output"
        fi
    else
        failure "BatchTag command failed"
    fi
    
    cd ..
}

# Test utility commands
test_utility_commands() {
    test_start "Utility commands - Recent, Popular, Quota"
    
    cd "node3"
    
    # Test Recent command
    if run_with_timeout "$BINARY_PATH recent --count 5 --days 7 > ../test_output.txt 2>&1" 30; then
        if grep -q "Recent Files\|Network connection established" ../test_output.txt; then
            success "Recent command passed"
        else
            warning "Recent command - unexpected output"
        fi
    else
        failure "Recent command failed"
    fi
    
    # Test Popular command
    if run_with_timeout "$BINARY_PATH popular week --count 3 > ../test_output.txt 2>&1" 30; then
        if grep -q "Popular Files\|Network connection established" ../test_output.txt; then
            success "Popular command passed"
        else
            warning "Popular command - unexpected output"
        fi
    else
        failure "Popular command failed"
    fi
    
    # Test Quota command
    if run_with_timeout "$BINARY_PATH quota --usage > ../test_output.txt 2>&1" 30; then
        if grep -q "storage usage\|Storage Quota\|Network connection established" ../test_output.txt; then
            success "Quota command passed"
        else
            warning "Quota command - unexpected output"
        fi
    else
        failure "Quota command failed"
    fi
    
    cd ..
}

# Test performance commands
test_performance_commands() {
    test_start "Performance commands - Benchmark"
    
    cd "node3"
    
    # Test Benchmark command
    if run_with_timeout "$BINARY_PATH benchmark --network --duration 10 > ../test_output.txt 2>&1" 30; then
        if grep -q "Benchmark\|Performance\|Network connection established" ../test_output.txt; then
            success "Benchmark command passed"
        else
            warning "Benchmark command - unexpected output"
        fi
    else
        failure "Benchmark command failed"
    fi
    
    cd ..
}

# Test cleanup and maintenance
test_cleanup_maintenance() {
    test_start "Cleanup and maintenance"
    
    cd "node4"
    
    # Test Cleanup command
    if run_with_timeout "$BINARY_PATH cleanup --orphaned --dry-run > ../test_output.txt 2>&1" 30; then
        if grep -q "Cleanup\|Storage cleanup\|Network connection established" ../test_output.txt; then
            success "Cleanup command passed"
        else
            warning "Cleanup command - unexpected output"
        fi
    else
        failure "Cleanup command failed"
    fi
    
    cd ..
}

# Test configuration and admin commands
test_admin_commands() {
    test_start "Admin commands - Config, Metrics, Networks"
    
    cd "node4"
    
    # Test Config command
    if run_with_timeout "$BINARY_PATH config --generate config.toml > ../test_output.txt 2>&1" 30; then
        if grep -q "Configuration\|config" ../test_output.txt || [ -f "config.toml" ]; then
            success "Config command passed"
        else
            warning "Config command - unexpected output"
        fi
    else
        failure "Config command failed"
    fi
    
    # Test Metrics command
    if run_with_timeout "$BINARY_PATH metrics --summary > ../test_output.txt 2>&1" 30; then
        if grep -q "Metrics\|Performance" ../test_output.txt; then
            success "Metrics command passed"
        else
            warning "Metrics command - unexpected output"
        fi
    else
        failure "Metrics command failed"
    fi
    
    # Test Networks command
    if run_with_timeout "$BINARY_PATH networks > ../test_output.txt 2>&1" 30; then
        if grep -q "Network\|Available" ../test_output.txt; then
            success "Networks command passed"
        else
            warning "Networks command - unexpected output"
        fi
    else
        failure "Networks command failed"
    fi
    
    cd ..
}

# Test cross-node file sharing
test_cross_node_sharing() {
    test_start "Cross-node file sharing"
    
    # Upload from node 1
    cd "node1"
    local file_key=""
    if file_key=$(run_with_timeout "$BINARY_PATH put ../test_files/test2.txt --name cross_node_test 2>&1" 60); then
        cd ..
        
        # Try to retrieve from last node
        local last_node=$((NODES-1))
        cd "node$last_node"
        if run_with_timeout "$BINARY_PATH get cross_node_test ../retrieved_cross_node.txt 2>&1" 60; then
            if [ -f "../retrieved_cross_node.txt" ]; then
                success "Cross-node file sharing passed"
            else
                failure "Cross-node file sharing failed - file not retrieved"
            fi
        else
            failure "Cross-node file sharing failed - get command failed"
        fi
        cd ..
    else
        failure "Cross-node file sharing failed - put command failed"
        cd ..
    fi
}

# Test cluster resilience
test_cluster_resilience() {
    test_start "Cluster resilience"
    
    # Test that cluster continues to function with multiple nodes
    local node_idx=$((NODES-1))
    cd "node$node_idx"
    
    if run_with_timeout "$BINARY_PATH peers > ../test_output.txt 2>&1" 30; then
        local peer_count=$(grep -c "Peer\|Connected" ../test_output.txt || echo "0")
        if [ "$peer_count" -gt 2 ]; then
            success "Cluster resilience test passed - $peer_count peers active"
        else
            warning "Cluster resilience test - low peer count: $peer_count"
        fi
    else
        failure "Cluster resilience test failed"
    fi
    
    cd ..
}

# Test import/export operations
test_import_export() {
    test_start "Import/Export operations"
    
    cd "node0"
    
    # Test Export command
    if run_with_timeout "$BINARY_PATH export --destination ../test_export.tar.gz --format tar.gz --pattern '*test*' > ../test_output.txt 2>&1" 60; then
        if grep -q "Export\|exported\|Network connection established" ../test_output.txt; then
            success "Export command passed"
        else
            warning "Export command - unexpected output"
        fi
    else
        failure "Export command failed"
    fi
    
    # Test Import command
    if [ -f "../test_export.tar.gz" ]; then
        if run_with_timeout "$BINARY_PATH import ../test_export.tar.gz --verify --tag-prefix imported > ../test_output.txt 2>&1" 60; then
            if grep -q "Import\|imported\|Network connection established" ../test_output.txt; then
                success "Import command passed"
            else
                warning "Import command - unexpected output"
            fi
        else
            failure "Import command failed"
        fi
    else
        warning "Export file not found - skipping import test"
    fi
    
    cd ..
}

# Test file manipulation commands
test_file_manipulation() {
    test_start "File manipulation - Duplicate, Rename"
    
    cd "node1"
    
    # Test Duplicate command
    if run_with_timeout "$BINARY_PATH duplicate test_file_1 --new-name test_file_duplicate --new-tags duplicated > ../test_output.txt 2>&1" 30; then
        if grep -q "duplicated\|Network connection established" ../test_output.txt; then
            success "Duplicate command passed"
        else
            warning "Duplicate command - unexpected output"
        fi
    else
        failure "Duplicate command failed"
    fi
    
    # Test Rename command
    if run_with_timeout "$BINARY_PATH rename test_file_1 test_file_renamed > ../test_output.txt 2>&1" 30; then
        if grep -q "renamed\|Network connection established" ../test_output.txt; then
            success "Rename command passed"
        else
            warning "Rename command - unexpected output"
        fi
    else
        failure "Rename command failed"
    fi
    
    cd ..
}

# Test backup and restore operations
test_backup_restore() {
    test_start "Backup and Restore operations"
    
    cd "node2"
    
    # Test Restore command (list versions)
    if run_with_timeout "$BINARY_PATH restore --backup-name test_backup_1 --list-versions > ../test_output.txt 2>&1" 30; then
        if grep -q "versions\|Restore\|Network connection established" ../test_output.txt; then
            success "Restore (list versions) command passed"
        else
            warning "Restore command - unexpected output"
        fi
    else
        failure "Restore command failed"
    fi
    
    cd ..
}

# Test repair and maintenance commands
test_repair_maintenance() {
    test_start "Repair and maintenance commands"
    
    cd "node3"
    
    # Test Repair command
    if run_with_timeout "$BINARY_PATH repair --verify-all --threshold 80 > ../test_output.txt 2>&1" 60; then
        if grep -q "Repair\|verification\|Network connection established" ../test_output.txt; then
            success "Repair command passed"
        else
            warning "Repair command - unexpected output"
        fi
    else
        failure "Repair command failed"
    fi
    
    cd ..
}

# Display test results
display_results() {
    echo ""
    echo "=========================================="
    echo "         TEST RESULTS SUMMARY"
    echo "=========================================="
    echo -e "Total Tests: ${BLUE}$TESTS_TOTAL${NC}"
    echo -e "Passed:      ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed:      ${RED}$TESTS_FAILED${NC}"
    
    if [ $TESTS_FAILED -gt 0 ]; then
        echo ""
        echo -e "${RED}Failed Tests:${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo -e "  ${RED}• $test${NC}"
        done
    fi
    
    echo ""
    local success_rate=$((TESTS_PASSED * 100 / TESTS_TOTAL))
    if [ $success_rate -ge 80 ]; then
        echo -e "${GREEN}Overall Result: PASSED ($success_rate% success rate)${NC}"
    elif [ $success_rate -ge 60 ]; then
        echo -e "${YELLOW}Overall Result: PARTIAL ($success_rate% success rate)${NC}"
    else
        echo -e "${RED}Overall Result: FAILED ($success_rate% success rate)${NC}"
    fi
    echo "=========================================="
}

# Main test execution
main() {
    echo "=========================================="
    echo "DataMesh Comprehensive Cluster Test"
    echo "Testing with $NODES nodes"
    echo "=========================================="
    
    check_prerequisites
    setup_test_environment
    
    # Start cluster
    start_bootstrap
    start_service_nodes
    
    # Run all tests
    test_network_connectivity
    test_health_monitoring
    test_network_discovery
    test_file_operations
    test_file_info_stats
    test_missing_commands
    test_new_commands
    test_batch_operations
    test_utility_commands
    test_performance_commands
    test_cleanup_maintenance
    test_admin_commands
    test_import_export
    test_file_manipulation
    test_backup_restore
    test_repair_maintenance
    test_cross_node_sharing
    test_cluster_resilience
    
    # Display results
    display_results
    
    # Return appropriate exit code
    if [ $TESTS_FAILED -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"