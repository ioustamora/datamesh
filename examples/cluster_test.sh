#!/bin/bash
# cluster_test.sh - Enhanced DFS cluster testing with comprehensive functionality verification

set -e

# Configuration
BOOTSTRAP_PORT=40871
NODE_PORTS=(40872 40873 40874 40875 40876)
DFS_BINARY="${DFS_BINARY:-./target/release/dfs}"
TEST_DIR="cluster_test_$(date +%Y%m%d_%H%M%S)"
LOG_DIR="$TEST_DIR/logs"
DATA_DIR="$TEST_DIR/data"
RESULTS_DIR="$TEST_DIR/results"

# Test configuration
MIN_NODES_RUNNING=3
DHT_PROPAGATION_TIME=8
TEST_FILES_COUNT=5
LARGE_FILE_SIZE_KB=100
RETRIEVAL_TIMEOUT=30
BOOTSTRAP_STARTUP_TIMEOUT=15
SERVICE_NODE_TIMEOUT=300  # 5 minutes for service nodes in testing

# Check if DFS binary exists
if [ ! -f "$DFS_BINARY" ]; then
    echo "Error: DFS binary not found at $DFS_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

# Ensure we have jq for JSON processing
if ! command -v jq &> /dev/null; then
    echo "Installing jq for JSON processing..."
    sudo apt-get update && sudo apt-get install -y jq || {
        echo "Warning: jq not available, using basic reporting"
        USE_JSON=false
    }
else
    USE_JSON=true
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

section() {
    echo ""
    echo -e "${PURPLE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${PURPLE}  $1${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Test result tracking
TEST_RESULTS="$RESULTS_DIR/test_results.json"

record_test() {
    local test_name="$1"
    local status="$2"
    local details="$3"
    local duration="$4"
    
    if [ "$USE_JSON" = true ]; then
        # Update results JSON
        temp_file=$(mktemp)
        jq --arg name "$test_name" --arg status "$status" --arg details "$details" --arg duration "$duration" '
            .tests += [{
                "name": $name,
                "status": $status,
                "details": $details,
                "duration": $duration,
                "timestamp": now
            }] |
            .summary.total += 1 |
            if $status == "PASS" then .summary.passed += 1 else .summary.failed += 1 end
        ' "$TEST_RESULTS" > "$temp_file" && mv "$temp_file" "$TEST_RESULTS"
    fi
    
    # Also log to text file
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$status] $test_name: $details (${duration}s)" >> "$RESULTS_DIR/test_log.txt"
}

# Cleanup function
cleanup() {
    log "Cleaning up processes..."
    
    # Kill all background processes
    if [ -f "$TEST_DIR/pids.txt" ]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                kill "$pid" 2>/dev/null || true
                log "Killed process $pid"
            fi
        done < "$TEST_DIR/pids.txt"
        rm -f "$TEST_DIR/pids.txt"
    fi
    
    log "Cleanup complete"
}

# Create test directories
mkdir -p "$LOG_DIR" "$DATA_DIR" "$RESULTS_DIR"

# Initialize test results
if [ "$USE_JSON" = true ]; then
    echo '{"tests": [], "summary": {"total": 0, "passed": 0, "failed": 0}}' > "$TEST_RESULTS"
fi

section "DataMesh Enhanced Cluster Test Starting"
log "Test directory: $TEST_DIR"
log "Bootstrap port: $BOOTSTRAP_PORT"
log "Node ports: ${NODE_PORTS[*]}"
log "Test files: $TEST_FILES_COUNT"
log "Large file size: ${LARGE_FILE_SIZE_KB}KB"
log "Results will be saved to: $RESULTS_DIR"
echo ""

# Create diverse test files with verification
section "Creating Test Files"
log "Creating $TEST_FILES_COUNT test files with different characteristics..."

FILE_NAMES=()
FILE_CHECKSUMS=()

for i in $(seq 1 $TEST_FILES_COUNT); do
    case $i in
        1)
            # Small text file
            filename="small_text_$i.txt"
            echo "This is a small test file $i with content $(date) and random data: $RANDOM" > "$DATA_DIR/$filename"
            ;;
        2)
            # Medium binary file
            filename="medium_binary_$i.bin"
            head -c $((LARGE_FILE_SIZE_KB * 512)) /dev/urandom > "$DATA_DIR/$filename"
            ;;
        3)
            # Large text file
            filename="large_text_$i.txt"
            {
                echo "Large test file $i header: $(date)"
                for j in {1..100}; do
                    echo "Line $j: $(head -c 100 /dev/urandom | base64 | tr -d '\n')"
                done
                echo "Large test file $i footer: $(date)"
            } > "$DATA_DIR/$filename"
            ;;
        4)
            # Configuration-like file
            filename="config_$i.toml"
            {
                echo "[network]"
                echo "port = $((40000 + RANDOM % 1000))"
                echo "timeout = $((RANDOM % 60 + 10))"
                echo ""
                echo "[data]"
                echo "encryption = true"
                echo "compression = false"
                echo "timestamp = \"$(date -Iseconds)\""
            } > "$DATA_DIR/$filename"
            ;;
        *)
            # Mixed content file
            filename="mixed_content_$i.dat"
            {
                echo "Mixed content file $i"
                head -c $((LARGE_FILE_SIZE_KB * 256)) /dev/urandom | base64
                echo ""
                echo "Random number: $RANDOM"
                echo "Timestamp: $(date)"
            } > "$DATA_DIR/$filename"
            ;;
    esac
    
    FILE_NAMES+=("$filename")
    
    # Calculate checksum for verification
    checksum=$(sha256sum "$DATA_DIR/$filename" | cut -d' ' -f1)
    FILE_CHECKSUMS+=("$checksum")
    
    success "Created $filename ($(du -h "$DATA_DIR/$filename" | cut -f1))"
done

info "Total test data size: $(du -sh "$DATA_DIR" | cut -f1)"

# Start bootstrap node with enhanced monitoring
section "Starting Bootstrap Node"
log "Starting bootstrap node on port $BOOTSTRAP_PORT..."

# Set unique data directory for bootstrap node
export BOOTSTRAP_HOME="$TEST_DIR/bootstrap_home"
mkdir -p "$BOOTSTRAP_HOME"
HOME="$BOOTSTRAP_HOME" "$DFS_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$LOG_DIR/bootstrap.log" 2>&1 &
BOOTSTRAP_PID=$!
echo "$BOOTSTRAP_PID" > "$TEST_DIR/pids.txt"

record_test "bootstrap_startup" "STARTED" "Bootstrap node initiated" "0"

# Enhanced bootstrap info extraction with timeout
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""
bootstrap_start_time=$(date +%s)

for i in $(seq 1 $BOOTSTRAP_STARTUP_TIMEOUT); do
    if [ -f "$LOG_DIR/bootstrap.log" ]; then
        # Try multiple patterns for peer ID extraction
        BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$LOG_DIR/bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
        
        # Try multiple patterns for address extraction
        BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$LOG_DIR/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
        
        if [ -n "$BOOTSTRAP_PEER_ID" ] && [ -n "$BOOTSTRAP_ADDR" ]; then
            break
        fi
    fi
    sleep 1
done

bootstrap_end_time=$(date +%s)
bootstrap_duration=$((bootstrap_end_time - bootstrap_start_time))

if [ -z "$BOOTSTRAP_PEER_ID" ] || [ -z "$BOOTSTRAP_ADDR" ]; then
    error "Failed to get bootstrap node information after ${BOOTSTRAP_STARTUP_TIMEOUT}s"
    echo "Bootstrap log contents:"
    cat "$LOG_DIR/bootstrap.log"
    record_test "bootstrap_startup" "FAIL" "Could not extract bootstrap info" "$bootstrap_duration"
    exit 1
fi

success "Bootstrap node started (PID: $BOOTSTRAP_PID)"
info "Peer ID: $BOOTSTRAP_PEER_ID"
info "Address: $BOOTSTRAP_ADDR"
record_test "bootstrap_startup" "PASS" "Bootstrap node ready" "$bootstrap_duration"

# Start regular nodes with monitoring
section "Starting Cluster Nodes"
log "Starting ${#NODE_PORTS[@]} regular nodes..."
STARTED_NODES=0

for i in "${!NODE_PORTS[@]}"; do
    port=${NODE_PORTS[$i]}
    node_num=$((i + 1))
    
    log "Starting node $node_num on port $port..."
    
    node_start_time=$(date +%s)
    # Set unique data directory for each node
    export NODE_HOME="$TEST_DIR/node_${node_num}_home"
    mkdir -p "$NODE_HOME"
    HOME="$NODE_HOME" "$DFS_BINARY" --non-interactive service \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$port" \
        > "$LOG_DIR/node_$node_num.log" 2>&1 &
    
    NODE_PID=$!
    echo "$NODE_PID" >> "$TEST_DIR/pids.txt"
    
    # Wait a moment and check if the node started successfully
    sleep 2
    if kill -0 "$NODE_PID" 2>/dev/null; then
        success "Node $node_num started (PID: $NODE_PID, Port: $port)"
        STARTED_NODES=$((STARTED_NODES + 1))
        record_test "node_${node_num}_startup" "PASS" "Node started successfully" "2"
    else
        error "Node $node_num failed to start"
        record_test "node_${node_num}_startup" "FAIL" "Node failed to start" "2"
    fi
done

info "Successfully started $STARTED_NODES/${#NODE_PORTS[@]} nodes"

if [ "$STARTED_NODES" -lt "$MIN_NODES_RUNNING" ]; then
    error "Insufficient nodes running for testing (need at least $MIN_NODES_RUNNING)"
    exit 1
fi

echo ""
log "All nodes started! Waiting ${DHT_PROPAGATION_TIME}s for network to stabilize..."
sleep $DHT_PROPAGATION_TIME

# File storage testing with comprehensive verification
section "File Storage Testing"
log "Testing file storage across cluster..."

declare -a FILE_KEYS
STORAGE_START_TIME=$(date +%s)
SUCCESSFUL_STORES=0

for i in "${!FILE_NAMES[@]}"; do
    filename="${FILE_NAMES[$i]}"
    log "Storing $filename..."
    
    store_start_time=$(date +%s)
    
    # Use timeout to prevent hanging
    output=$(timeout 30 "$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$DATA_DIR/$filename" 2>&1) || true
    
    store_end_time=$(date +%s)
    store_duration=$((store_end_time - store_start_time))
    
    key=$(echo "$output" | grep "File stored with key:" | cut -d' ' -f5 2>/dev/null || true)
    
    if [ -n "$key" ]; then
        FILE_KEYS+=("$key")
        success "File $((i+1)) stored with key: ${key:0:16}... (${store_duration}s)"
        echo "$filename:$key" >> "$RESULTS_DIR/file_keys.txt"
        SUCCESSFUL_STORES=$((SUCCESSFUL_STORES + 1))
        record_test "storage_${filename}" "PASS" "File stored successfully" "$store_duration"
    else
        error "Failed to store $filename"
        echo "Output: $output"
        FILE_KEYS+=("")  # Keep array aligned
        record_test "storage_${filename}" "FAIL" "Storage operation failed" "$store_duration"
    fi
    
    sleep 2
done

STORAGE_END_TIME=$(date +%s)
STORAGE_TOTAL_TIME=$((STORAGE_END_TIME - STORAGE_START_TIME))
success "Storage phase completed in ${STORAGE_TOTAL_TIME}s"
info "Successfully stored $SUCCESSFUL_STORES/${#FILE_NAMES[@]} files"

if [ "$SUCCESSFUL_STORES" -eq 0 ]; then
    error "No files were stored successfully - cannot continue with retrieval tests"
    exit 1
fi

echo ""
log "Waiting ${DHT_PROPAGATION_TIME}s for DHT propagation..."
sleep $DHT_PROPAGATION_TIME

# File retrieval testing with verification
section "File Retrieval Testing"
log "Testing file retrieval and verification..."

RETRIEVAL_START_TIME=$(date +%s)
SUCCESSFUL_RETRIEVALS=0

for i in "${!FILE_KEYS[@]}"; do
    key="${FILE_KEYS[$i]}"
    filename="${FILE_NAMES[$i]}"
    
    if [ -z "$key" ]; then
        warn "Skipping retrieval test for $filename (no key)"
        continue
    fi
    
    retrieved_file="$RESULTS_DIR/retrieved_$filename"
    
    log "Retrieving $filename (key: ${key:0:16}...)..."
    
    start_time=$(date +%s)
    timeout "$RETRIEVAL_TIMEOUT" "$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        get "$key" "$retrieved_file" > /dev/null 2>&1
    exit_code=$?
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    if [ $exit_code -eq 0 ] && [ -f "$retrieved_file" ]; then
        # Verify file content using checksums
        retrieved_checksum=$(sha256sum "$retrieved_file" | cut -d' ' -f1)
        original_checksum="${FILE_CHECKSUMS[$i]}"
        
        if [ "$retrieved_checksum" = "$original_checksum" ]; then
            success "Retrieved and verified $filename (${duration}s)"
            SUCCESSFUL_RETRIEVALS=$((SUCCESSFUL_RETRIEVALS + 1))
            record_test "retrieval_${filename}" "PASS" "File retrieved and verified" "$duration"
        else
            error "Retrieved $filename but content differs!"
            record_test "retrieval_${filename}" "FAIL" "Content verification failed" "$duration"
        fi
    else
        error "Failed to retrieve $filename (timeout: ${RETRIEVAL_TIMEOUT}s)"
        record_test "retrieval_${filename}" "FAIL" "Retrieval failed or timed out" "$duration"
    fi
    
    sleep 1
done

RETRIEVAL_END_TIME=$(date +%s)
RETRIEVAL_TOTAL_TIME=$((RETRIEVAL_END_TIME - RETRIEVAL_START_TIME))
success "Retrieval phase completed in ${RETRIEVAL_TOTAL_TIME}s"
info "Successfully retrieved $SUCCESSFUL_RETRIEVALS/${#FILE_KEYS[@]} files"

# Cross-node availability testing
section "Cross-Node Availability Testing"
log "Testing file availability from multiple connection points..."

if [ "$SUCCESSFUL_STORES" -gt 0 ] && [ ${#FILE_KEYS[@]} -gt 0 ]; then
    test_key="${FILE_KEYS[0]}"  # Use first successfully stored file
    test_filename="${FILE_NAMES[0]}"
    
    if [ -n "$test_key" ]; then
        log "Testing availability of $test_filename from different access points..."
        
        availability_tests=0
        availability_passed=0
        
        for j in "${!NODE_PORTS[@]}"; do
            access_port=${NODE_PORTS[$j]}
            test_port=$((access_port + 200))  # Use different ports to avoid conflicts
            
            log "Testing access via port $test_port..."
            availability_tests=$((availability_tests + 1))
            
            start_time=$(date +%s)
            timeout 15 "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --port "$test_port" \
                --non-interactive \
                get "$test_key" "$RESULTS_DIR/availability_test_$test_port.txt" > /dev/null 2>&1
            exit_code=$?
            end_time=$(date +%s)
            duration=$((end_time - start_time))
            
            if [ $exit_code -eq 0 ] && [ -f "$RESULTS_DIR/availability_test_$test_port.txt" ]; then
                retrieved_checksum=$(sha256sum "$RESULTS_DIR/availability_test_$test_port.txt" | cut -d' ' -f1)
                if [ "$retrieved_checksum" = "${FILE_CHECKSUMS[0]}" ]; then
                    success "File accessible via port $test_port"
                    availability_passed=$((availability_passed + 1))
                    record_test "availability_port_$test_port" "PASS" "File accessible from access point" "$duration"
                else
                    warn "File retrieved via port $test_port but content differs"
                    record_test "availability_port_$test_port" "FAIL" "Content verification failed" "$duration"
                fi
            else
                warn "File not accessible via port $test_port"
                record_test "availability_port_$test_port" "FAIL" "File not accessible" "$duration"
            fi
        done
        
        log "Cross-node availability: $availability_passed/$availability_tests access points successful"
        if [ "$availability_passed" -eq "$availability_tests" ]; then
            success "100% cross-node accessibility achieved!"
        elif [ "$availability_passed" -gt 0 ]; then
            warn "Partial cross-node accessibility: $availability_passed/$availability_tests"
        else
            error "No cross-node accessibility - all access points failed"
        fi
    fi
fi

# Network resilience testing
section "Network Resilience Testing"
log "Testing file availability after stopping nodes..."

# Stop half of the nodes
nodes_to_stop=$((STARTED_NODES / 2))
log "Stopping $nodes_to_stop nodes to test fault tolerance..."

stopped_pids=($(tail -n +2 "$TEST_DIR/pids.txt" | head -n "$nodes_to_stop"))
for pid in "${stopped_pids[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        log "Stopped node (PID: $pid)"
    fi
done

sleep 5

# Test retrieval with reduced cluster
resilience_tests=0
resilience_passed=0

for i in $(seq 0 2); do  # Test first 3 files
    if [ $i -ge ${#FILE_KEYS[@]} ]; then
        break
    fi
    
    key="${FILE_KEYS[$i]}"
    filename="${FILE_NAMES[$i]}"
    
    if [ -z "$key" ]; then
        continue
    fi
    
    resilience_file="$RESULTS_DIR/resilience_$filename"
    resilience_tests=$((resilience_tests + 1))
    
    start_time=$(date +%s)
    timeout "$RETRIEVAL_TIMEOUT" "$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        get "$key" "$resilience_file" > /dev/null 2>&1
    exit_code=$?
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    if [ $exit_code -eq 0 ] && [ -f "$resilience_file" ]; then
        retrieved_checksum=$(sha256sum "$resilience_file" | cut -d' ' -f1)
        if [ "$retrieved_checksum" = "${FILE_CHECKSUMS[$i]}" ]; then
            success "Resilience test passed for $filename"
            resilience_passed=$((resilience_passed + 1))
            record_test "resilience_${filename}" "PASS" "File available after node failures" "$duration"
        else
            error "Resilience test failed for $filename (content differs)"
            record_test "resilience_${filename}" "FAIL" "Content verification failed" "$duration"
        fi
    else
        warn "Resilience test failed for $filename (could not retrieve)"
        record_test "resilience_${filename}" "FAIL" "Could not retrieve after node failures" "$duration"
    fi
done

info "Resilience test: $resilience_passed/$resilience_tests files available after node failures"

# Network health analysis
section "Network Health Analysis"
log "Analyzing cluster health..."

running_nodes=0
total_nodes=$((STARTED_NODES + 1))  # +1 for bootstrap

# Check bootstrap
if kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
    running_nodes=$((running_nodes + 1))
    success "Bootstrap node: Running"
else
    error "Bootstrap node: Stopped"
fi

# Check regular nodes
remaining_pids=($(tail -n +2 "$TEST_DIR/pids.txt"))
for i in "${!remaining_pids[@]}"; do
    pid="${remaining_pids[$i]}"
    if kill -0 "$pid" 2>/dev/null; then
        running_nodes=$((running_nodes + 1))
        status="Running"
    else
        status="Stopped"
    fi
    echo "Node $((i + 1)) (PID $pid): $status"
done

info "Cluster health: $running_nodes/$total_nodes nodes running"

# Generate comprehensive test report
section "Test Report Generation"
log "Generating comprehensive test report..."

# Create summary report
{
    echo "DataMesh Cluster Test Report"
    echo "======================="
    echo "Test Date: $(date)"
    echo "Test Directory: $TEST_DIR"
    echo ""
    echo "Configuration:"
    echo "- Bootstrap Port: $BOOTSTRAP_PORT"
    echo "- Node Ports: ${NODE_PORTS[*]}"
    echo "- Test Files: $TEST_FILES_COUNT"
    echo "- Nodes Started: $STARTED_NODES"
    echo ""
    echo "Results Summary:"
    echo "- Files Stored: $SUCCESSFUL_STORES/${#FILE_NAMES[@]}"
    echo "- Files Retrieved: $SUCCESSFUL_RETRIEVALS/${#FILE_KEYS[@]}"
    echo "- Availability Tests: $availability_passed/$availability_tests"
    echo "- Resilience Tests: $resilience_passed/$resilience_tests"
    echo ""
    echo "Performance:"
    echo "- Storage Time: ${STORAGE_TOTAL_TIME}s"
    echo "- Retrieval Time: ${RETRIEVAL_TOTAL_TIME}s"
    echo "- Bootstrap Startup: ${bootstrap_duration}s"
    echo ""
    echo "Node Status:"
    echo "- Total Nodes: $total_nodes"
    echo "- Running Nodes: $running_nodes"
    echo "- Cluster Health: $((running_nodes * 100 / total_nodes))%"
} > "$RESULTS_DIR/test_summary.txt"

# Generate file listing
if [ -f "$RESULTS_DIR/file_keys.txt" ]; then
    {
        echo ""
        echo "Stored Files:"
        cat "$RESULTS_DIR/file_keys.txt"
    } >> "$RESULTS_DIR/test_summary.txt"
fi

success "Test report saved to $RESULTS_DIR/test_summary.txt"

# Final assessment
section "Final Assessment"

if [ "$SUCCESSFUL_STORES" -gt 0 ] && [ "$SUCCESSFUL_RETRIEVALS" -gt 0 ]; then
    success "Core functionality verified: Storage and retrieval working"
else
    error "Core functionality failed: Issues with storage or retrieval"
fi

if [ "$availability_passed" -gt 0 ]; then
    success "Cross-node accessibility verified"
else
    warn "Cross-node accessibility not verified"
fi

if [ "$resilience_passed" -gt 0 ]; then
    success "Network resilience verified"
else
    warn "Network resilience not verified"
fi

# Calculate overall success rate
if [ "$USE_JSON" = true ]; then
    overall_stats=$(jq -r '.summary | "Total: \(.total), Passed: \(.passed), Failed: \(.failed), Success Rate: \((.passed * 100 / .total) | floor)%"' "$TEST_RESULTS")
    info "Overall test results: $overall_stats"
fi

log "Detailed logs available in: $LOG_DIR"
log "Test results available in: $RESULTS_DIR"
success "Enhanced cluster test completed!"

echo ""
echo "Key files for manual verification:"
echo "- Test summary: $RESULTS_DIR/test_summary.txt"
echo "- File keys: $RESULTS_DIR/file_keys.txt"
echo "- Test log: $RESULTS_DIR/test_log.txt"
if [ "$USE_JSON" = true ]; then
    echo "- JSON results: $TEST_RESULTS"
fi
echo "- Bootstrap log: $LOG_DIR/bootstrap.log"
echo "- Node logs: $LOG_DIR/node_*.log"

echo ""
echo "${YELLOW}Nodes are still running for manual interaction!${NC}"
echo ""
echo "You can now manually start a new node and connect it to the running cluster."
echo "For example, in a new terminal, run:"
echo ""
echo "  $DFS_BINARY --non-interactive service --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --port <NEW_PORT>"
echo ""
echo "Or interactively:"
echo "  $DFS_BINARY interactive --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --port <NEW_PORT>"
echo ""
echo "You can use the CLI to store, retrieve, or list files, and observe cluster behavior."
echo ""
echo "${CYAN}When you are finished, manually stop all nodes (kill the PIDs in $TEST_DIR/pids.txt and any you started manually).${NC}"
echo "Then press Enter here to clean up and exit."
read -r -p "Press Enter to clean up and exit..."

cleanup

log "All nodes stopped. Test session complete."
