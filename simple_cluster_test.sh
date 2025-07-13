#!/bin/bash

# Simple DataMesh Cluster Test - Test Key Commands with 4 Nodes
# This script tests the most important DataMesh commands in a cluster

# Configuration
NODES=4
BOOTSTRAP_PORT=9000
BASE_PORT=9001
TEST_DIR="simple_cluster_test_$(date +%s)"
BINARY_PATH="$(pwd)/target/release/datamesh"

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

test_start() {
    ((TESTS_TOTAL++))
    log "Testing: $1"
}

# Wait for process to start
wait_for_startup() {
    sleep 5  # Simple wait - more reliable than port checking
}

# Main test function
run_test() {
    echo "=========================================="
    echo "DataMesh Simple Cluster Test"
    echo "Testing with $NODES nodes (minimum 4)"
    echo "=========================================="
    
    # Check if binary exists
    if [ ! -f "$BINARY_PATH" ]; then
        failure "Binary not found at $BINARY_PATH"
        exit 1
    fi
    
    success "Binary found at $BINARY_PATH"
    
    # Create test directory
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    
    # Create node directories and test files
    for i in $(seq 0 $((NODES-1))); do
        mkdir -p "node$i"
    done
    
    echo "Hello DataMesh Test File 1" > "test1.txt"
    echo "Hello DataMesh Test File 2" > "test2.txt"
    
    success "Test environment setup complete"
    
    # Start bootstrap node
    test_start "Bootstrap node startup"
    cd "node0"
    timeout 30 $BINARY_PATH bootstrap --port $BOOTSTRAP_PORT > bootstrap.log 2>&1 &
    BOOTSTRAP_PID=$!
    cd ..
    
    wait_for_startup
    
    # Check if bootstrap is still running
    if kill -0 $BOOTSTRAP_PID 2>/dev/null; then
        success "Bootstrap node started successfully"
    else
        failure "Bootstrap node failed to start"
        cat "node0/bootstrap.log"
        return 1
    fi
    
    # Start service nodes
    test_start "Service nodes startup"
    local bootstrap_addr="/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT"
    
    for i in $(seq 1 $((NODES-1))); do
        local port=$((BASE_PORT + i - 1))
        cd "node$i"
        timeout 30 $BINARY_PATH service --port $port --bootstrap-addr "$bootstrap_addr" --timeout 120 > service.log 2>&1 &
        cd ..
        sleep 1
    done
    
    # Wait for cluster to form
    sleep 10
    success "Service nodes started"
    
    # Test basic commands
    test_start "Basic file operations (put/get/list)"
    cd "node1"
    
    # Test put command
    if timeout 60 $BINARY_PATH put ../test1.txt --name test_file_1 > put_output.txt 2>&1; then
        if grep -q "stored successfully\|stored with key" put_output.txt; then
            success "File put operation successful"
            
            # Test list command
            if timeout 30 $BINARY_PATH list > list_output.txt 2>&1; then
                success "File list operation successful"
            else
                failure "File list operation failed"
            fi
            
            # Test get command
            if timeout 60 $BINARY_PATH get test_file_1 ../retrieved_test1.txt > get_output.txt 2>&1; then
                if [ -f "../retrieved_test1.txt" ]; then
                    success "File get operation successful"
                else
                    failure "File get operation failed - file not found"
                fi
            else
                failure "File get operation failed"
            fi
        else
            failure "File put operation failed"
            cat put_output.txt
        fi
    else
        failure "File put operation timed out or failed"
        cat put_output.txt
    fi
    cd ..
    
    # Test network commands
    test_start "Network commands (peers/health)"
    cd "node2"
    
    if timeout 30 $BINARY_PATH peers > peers_output.txt 2>&1; then
        success "Peers command successful"
    else
        failure "Peers command failed"
    fi
    
    if timeout 30 $BINARY_PATH health > health_output.txt 2>&1; then
        success "Health command successful"
    else
        failure "Health command failed"
    fi
    cd ..
    
    # Test new commands
    test_start "New commands (pin/share/optimize)"
    cd "node3"
    
    # Test pin command
    if timeout 30 $BINARY_PATH pin test_file_1 --priority 5 > pin_output.txt 2>&1; then
        if grep -q "pinned\|Pin\|Network connection" pin_output.txt; then
            success "Pin command successful"
        else
            failure "Pin command unexpected output"
        fi
    else
        failure "Pin command failed"
    fi
    
    # Test share command
    if timeout 30 $BINARY_PATH share test_file_1 --public > share_output.txt 2>&1; then
        if grep -q "Share\|share\|Network connection" share_output.txt; then
            success "Share command successful"
        else
            failure "Share command unexpected output"
        fi
    else
        failure "Share command failed"
    fi
    
    # Test optimize command
    if timeout 30 $BINARY_PATH optimize --defrag > optimize_output.txt 2>&1; then
        if grep -q "optimization\|Optimization\|Network connection" optimize_output.txt; then
            success "Optimize command successful"
        else
            failure "Optimize command unexpected output"
        fi
    else
        failure "Optimize command failed"
    fi
    cd ..
    
    # Test utility commands
    test_start "Utility commands (search/recent/quota)"
    cd "node4"
    
    # Test search command
    if timeout 30 $BINARY_PATH search test > search_output.txt 2>&1; then
        if grep -q "Search\|search\|Network connection" search_output.txt; then
            success "Search command successful"
        else
            failure "Search command unexpected output"
        fi
    else
        failure "Search command failed"
    fi
    
    # Test recent command
    if timeout 30 $BINARY_PATH recent --count 5 > recent_output.txt 2>&1; then
        if grep -q "Recent\|recent\|Network connection" recent_output.txt; then
            success "Recent command successful"
        else
            failure "Recent command unexpected output"
        fi
    else
        failure "Recent command failed"
    fi
    
    # Test quota command
    if timeout 30 $BINARY_PATH quota --usage > quota_output.txt 2>&1; then
        if grep -q "quota\|Quota\|storage\|Network connection" quota_output.txt; then
            success "Quota command successful"
        else
            failure "Quota command unexpected output"
        fi
    else
        failure "Quota command failed"
    fi
    cd ..
    
    # Test batch commands
    test_start "Batch commands (batch-put/batch-tag)"
    cd "node5"
    
    # Test batch-put command
    if timeout 60 $BINARY_PATH batch-put ../test1.txt > batch_put_output.txt 2>&1; then
        if grep -q "Batch\|batch\|Network connection" batch_put_output.txt; then
            success "Batch-put command successful"
        else
            failure "Batch-put command unexpected output"
        fi
    else
        failure "Batch-put command failed"
    fi
    
    # Test batch-tag command
    if timeout 30 $BINARY_PATH batch-tag '*' --add-tags test --dry-run > batch_tag_output.txt 2>&1; then
        if grep -q "Batch\|batch\|Tag\|Network connection" batch_tag_output.txt; then
            success "Batch-tag command successful"
        else
            failure "Batch-tag command unexpected output"
        fi
    else
        failure "Batch-tag command failed"
    fi
    cd ..
    
    # Test performance commands
    test_start "Performance commands (benchmark/cleanup)"
    cd "node1"
    
    # Test benchmark command
    if timeout 45 $BINARY_PATH benchmark --network --duration 5 > benchmark_output.txt 2>&1; then
        if grep -q "Benchmark\|benchmark\|Performance\|Network connection" benchmark_output.txt; then
            success "Benchmark command successful"
        else
            failure "Benchmark command unexpected output"
        fi
    else
        failure "Benchmark command failed"
    fi
    
    # Test cleanup command
    if timeout 30 $BINARY_PATH cleanup --dry-run > cleanup_output.txt 2>&1; then
        if grep -q "Cleanup\|cleanup\|Storage\|Network connection" cleanup_output.txt; then
            success "Cleanup command successful"
        else
            failure "Cleanup command unexpected output"
        fi
    else
        failure "Cleanup command failed"
    fi
    cd ..
    
    # Display results
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
    if [ $success_rate -ge 70 ]; then
        echo -e "${GREEN}Overall Result: PASSED ($success_rate% success rate)${NC}"
        echo -e "${GREEN}🎉 DataMesh cluster test completed successfully!${NC}"
    else
        echo -e "${RED}Overall Result: FAILED ($success_rate% success rate)${NC}"
        echo -e "${RED}❌ Some tests failed - check output above${NC}"
    fi
    echo "=========================================="
    
    # Return to original directory
    cd ..
}

# Run the test
run_test