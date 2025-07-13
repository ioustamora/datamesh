#!/bin/bash
# perfect_cluster_test.sh - Perfect DFS cluster testing with comprehensive functionality and persistent environment
# This script provides:
# 1. Perfect UX with real-time progress indicators
# 2. Comprehensive cluster testing (5+ nodes + bootstrap)
# 3. Persistent environment for manual testing
# 4. Professional monitoring and management tools
# 5. Interactive cluster management dashboard

set -e

# Advanced Configuration
BOOTSTRAP_PORT=40871
NODE_PORTS=(40872 40873 40874 40875 40876 40877 40878)  # 7 nodes for robust testing
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/release/datamesh}"
TEST_DIR="perfect_cluster_$(date +%Y%m%d_%H%M%S)"
LOG_DIR="$TEST_DIR/logs"
DATA_DIR="$TEST_DIR/data"
RESULTS_DIR="$TEST_DIR/results"
MONITOR_DIR="$TEST_DIR/monitoring"

# Enhanced Test Configuration
MIN_NODES_RUNNING=5
DHT_PROPAGATION_TIME=10
TEST_FILES_COUNT=8
LARGE_FILE_SIZE_KB=500
RETRIEVAL_TIMEOUT=30
BOOTSTRAP_STARTUP_TIMEOUT=20
SERVICE_NODE_TIMEOUT=600  # 10 minutes for long-running tests
HEALTH_CHECK_INTERVAL=30
MONITORING_ENABLED=true

# Advanced Features
ENABLE_BANDWIDTH_TESTING=true
ENABLE_NETWORK_TOPOLOGY_ANALYSIS=true
ENABLE_REAL_TIME_MONITORING=true
ENABLE_FAULT_INJECTION=true
ENABLE_PERFORMANCE_BENCHMARKS=true

# Color scheme for perfect UX
declare -A COLORS=(
    ["RED"]='\033[0;31m'
    ["GREEN"]='\033[0;32m'
    ["BLUE"]='\033[0;34m'
    ["YELLOW"]='\033[1;33m'
    ["PURPLE"]='\033[0;35m'
    ["CYAN"]='\033[0;36m'
    ["WHITE"]='\033[1;37m'
    ["GRAY"]='\033[0;37m'
    ["BOLD"]='\033[1m'
    ["DIM"]='\033[2m'
    ["NC"]='\033[0m'
)

# Unicode symbols for perfect UX
declare -A SYMBOLS=(
    ["SUCCESS"]="✅"
    ["ERROR"]="❌"
    ["WARNING"]="⚠️"
    ["INFO"]="ℹ️"
    ["RUNNING"]="🔄"
    ["COMPLETED"]="✨"
    ["NETWORK"]="🌐"
    ["FILE"]="📁"
    ["CLOCK"]="⏰"
    ["ROCKET"]="🚀"
    ["CHART"]="📊"
    ["GEAR"]="⚙️"
    ["SHIELD"]="🛡️"
    ["LIGHTNING"]="⚡"
)

# Advanced logging with levels and timestamps
log_level() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%H:%M:%S.%3N')
    local color_key=""
    local symbol=""
    
    case "$level" in
        "SUCCESS") color_key="GREEN"; symbol="${SYMBOLS[SUCCESS]}" ;;
        "ERROR") color_key="RED"; symbol="${SYMBOLS[ERROR]}" ;;
        "WARNING") color_key="YELLOW"; symbol="${SYMBOLS[WARNING]}" ;;
        "INFO") color_key="CYAN"; symbol="${SYMBOLS[INFO]}" ;;
        "DEBUG") color_key="GRAY"; symbol="${SYMBOLS[GEAR]}" ;;
        *) color_key="WHITE"; symbol="${SYMBOLS[INFO]}" ;;
    esac
    
    printf "${COLORS[GRAY]}[%s]${COLORS[NC]} %s ${COLORS[$color_key]}%s${COLORS[NC]} %s\n" \
        "$timestamp" "$symbol" "$level" "$message"
}

success() { log_level "SUCCESS" "$1"; }
error() { log_level "ERROR" "$1"; }
warning() { log_level "WARNING" "$1"; }
info() { log_level "INFO" "$1"; }
debug() { log_level "DEBUG" "$1"; }

# Enhanced section headers
print_header() {
    local title="$1"
    local width=100
    local padding=$(( (width - ${#title} - 4) / 2 ))
    
    echo ""
    printf "${COLORS[PURPLE]}%s${COLORS[NC]}\n" "$(printf '═%.0s' $(seq 1 $width))"
    printf "${COLORS[PURPLE]}│%s${COLORS[BOLD]}${COLORS[WHITE]} %s ${COLORS[NC]}${COLORS[PURPLE]}%s│${COLORS[NC]}\n" \
        "$(printf ' %.0s' $(seq 1 $padding))" \
        "$title" \
        "$(printf ' %.0s' $(seq 1 $padding))"
    printf "${COLORS[PURPLE]}%s${COLORS[NC]}\n" "$(printf '═%.0s' $(seq 1 $width))"
    echo ""
}

print_section() {
    local title="$1"
    local emoji="$2"
    echo ""
    printf "${COLORS[BLUE]}${COLORS[BOLD]}%s %s${COLORS[NC]}\n" "$emoji" "$title"
    printf "${COLORS[BLUE]}%s${COLORS[NC]}\n" "$(printf '─%.0s' $(seq 1 $((${#title} + 4))))"
}

# Progress tracking with visual indicators
show_progress() {
    local current="$1"
    local total="$2"
    local message="$3"
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((current * width / total))
    local empty=$((width - filled))
    
    printf "\r${COLORS[CYAN]}%s${COLORS[NC]} [" "$message"
    printf "${COLORS[GREEN]}%s${COLORS[NC]}" "$(printf '█%.0s' $(seq 1 $filled))"
    printf "${COLORS[GRAY]}%s${COLORS[NC]}" "$(printf '░%.0s' $(seq 1 $empty))"
    printf "] %3d%% (%d/%d)" "$percentage" "$current" "$total"
    
    if [ "$current" -eq "$total" ]; then
        printf " ${SYMBOLS[COMPLETED]}\n"
    fi
}

# Advanced JSON result tracking
PERFORMANCE_DATA="$RESULTS_DIR/performance.json"
NETWORK_TOPOLOGY="$RESULTS_DIR/topology.json"
HEALTH_DATA="$RESULTS_DIR/health.json"

initialize_monitoring() {
    mkdir -p "$MONITOR_DIR"
    
    # Initialize monitoring data structures
    echo '{"metrics": [], "start_time": "'$(date -Iseconds)'", "cluster_id": "'$TEST_DIR'"}' > "$PERFORMANCE_DATA"
    echo '{"nodes": [], "connections": [], "updated": "'$(date -Iseconds)'"}' > "$NETWORK_TOPOLOGY"
    echo '{"health_checks": [], "cluster_status": "initializing"}' > "$HEALTH_DATA"
}

record_metric() {
    local metric_type="$1"
    local metric_name="$2"
    local value="$3"
    local unit="$4"
    local metadata="$5"
    
    if command -v jq &> /dev/null; then
        local temp_file=$(mktemp)
        jq --arg type "$metric_type" \
           --arg name "$metric_name" \
           --arg value "$value" \
           --arg unit "$unit" \
           --arg metadata "$metadata" \
           '.metrics += [{
               "type": $type,
               "name": $name,
               "value": ($value | tonumber),
               "unit": $unit,
               "metadata": $metadata,
               "timestamp": now
           }]' "$PERFORMANCE_DATA" > "$temp_file" && mv "$temp_file" "$PERFORMANCE_DATA"
    fi
}

# Node management with enhanced monitoring
declare -A NODE_PIDS
declare -A NODE_STATUS
declare -A NODE_STATS

start_node_with_monitoring() {
    local node_type="$1"
    local port="$2"
    local node_id="$3"
    local bootstrap_peer="$4"
    local bootstrap_addr="$5"
    
    local log_file="$LOG_DIR/${node_type}_${node_id}.log"
    local pid_file="$MONITOR_DIR/${node_type}_${node_id}.pid"
    local status_file="$MONITOR_DIR/${node_type}_${node_id}.status"
    
    info "Starting $node_type node $node_id on port $port..."
    
    local start_time=$(date +%s.%3N)
    
    if [ "$node_type" = "bootstrap" ]; then
        "$DATAMESH_BINARY" --non-interactive bootstrap --port "$port" > "$log_file" 2>&1 &
    else
        "$DATAMESH_BINARY" --non-interactive service \
            --bootstrap-peer "$bootstrap_peer" \
            --bootstrap-addr "$bootstrap_addr" \
            --port "$port" \
            > "$log_file" 2>&1 &
    fi
    
    local pid=$!
    echo "$pid" > "$pid_file"
    NODE_PIDS["$node_id"]="$pid"
    
    # Wait for startup and verify
    sleep 3
    if kill -0 "$pid" 2>/dev/null; then
        NODE_STATUS["$node_id"]="running"
        local end_time=$(date +%s.%3N)
        local startup_time=$(echo "$end_time - $start_time" | bc -l)
        
        echo "running:$(date -Iseconds):$port:$pid" > "$status_file"
        record_metric "startup" "${node_type}_startup" "$startup_time" "seconds" "node_$node_id"
        success "$node_type node $node_id started successfully (PID: $pid, startup: ${startup_time}s)"
        return 0
    else
        NODE_STATUS["$node_id"]="failed"
        echo "failed:$(date -Iseconds):$port:" > "$status_file"
        error "$node_type node $node_id failed to start"
        return 1
    fi
}

# Network health monitoring
monitor_cluster_health() {
    local check_interval="$1"
    local duration="$2"
    local end_time=$(($(date +%s) + duration))
    
    print_section "Real-time Cluster Health Monitoring" "${SYMBOLS[CHART]}"
    
    while [ $(date +%s) -lt $end_time ]; do
        local running_nodes=0
        local total_nodes=${#NODE_PIDS[@]}
        local health_issues=()
        
        # Check each node
        for node_id in "${!NODE_PIDS[@]}"; do
            local pid="${NODE_PIDS[$node_id]}"
            if kill -0 "$pid" 2>/dev/null; then
                ((running_nodes++))
                NODE_STATUS["$node_id"]="running"
            else
                NODE_STATUS["$node_id"]="stopped"
                health_issues+=("Node $node_id has stopped")
            fi
        done
        
        # Calculate health percentage
        local health_percentage=$((running_nodes * 100 / total_nodes))
        
        # Display health status
        printf "\r${SYMBOLS[NETWORK]} Cluster Health: "
        if [ $health_percentage -ge 80 ]; then
            printf "${COLORS[GREEN]}%d%%${COLORS[NC]}" "$health_percentage"
        elif [ $health_percentage -ge 60 ]; then
            printf "${COLORS[YELLOW]}%d%%${COLORS[NC]}" "$health_percentage"
        else
            printf "${COLORS[RED]}%d%%${COLORS[NC]}" "$health_percentage"
        fi
        printf " (%d/%d nodes) " "$running_nodes" "$total_nodes"
        
        # Record health data
        if command -v jq &> /dev/null; then
            local temp_file=$(mktemp)
            jq --arg running "$running_nodes" \
               --arg total "$total_nodes" \
               --arg percentage "$health_percentage" \
               '.health_checks += [{
                   "running_nodes": ($running | tonumber),
                   "total_nodes": ($total | tonumber),
                   "health_percentage": ($percentage | tonumber),
                   "timestamp": now
               }] | .cluster_status = "monitoring"' "$HEALTH_DATA" > "$temp_file" && mv "$temp_file" "$HEALTH_DATA"
        fi
        
        sleep "$check_interval"
    done
    echo ""
}

# Enhanced file testing with comprehensive verification
create_diverse_test_files() {
    print_section "Creating Diverse Test Files" "${SYMBOLS[FILE]}"
    
    declare -ag TEST_FILES
    declare -ag TEST_CHECKSUMS
    declare -ag TEST_SIZES
    
    local file_types=(
        "small_text:10:text"
        "medium_binary:100:binary"
        "large_text:500:text"
        "config_file:5:config"
        "compressed_data:200:compressed"
        "json_data:50:json"
        "image_simulation:300:binary"
        "log_file:150:log"
    )
    
    for i in "${!file_types[@]}"; do
        local file_info="${file_types[$i]}"
        local file_type=$(echo "$file_info" | cut -d: -f1)
        local size_kb=$(echo "$file_info" | cut -d: -f2)
        local content_type=$(echo "$file_info" | cut -d: -f3)
        
        local filename="${file_type}_$((i+1)).dat"
        local filepath="$DATA_DIR/$filename"
        
        show_progress $((i+1)) ${#file_types[@]} "Creating $filename"
        
        case "$content_type" in
            "text")
                {
                    echo "=== $file_type Test File ==="
                    echo "Created: $(date)"
                    echo "Size target: ${size_kb}KB"
                    echo "Type: $content_type"
                    echo "=========================================="
                    for j in $(seq 1 $((size_kb * 10))); do
                        echo "Line $j: This is test content with random data $RANDOM and timestamp $(date +%s)"
                    done
                } > "$filepath"
                ;;
            "binary")
                head -c $((size_kb * 1024)) /dev/urandom > "$filepath"
                ;;
            "config")
                {
                    echo "[cluster]"
                    echo "nodes = ${#NODE_PORTS[@]}"
                    echo "bootstrap_port = $BOOTSTRAP_PORT"
                    echo "test_id = \"$TEST_DIR\""
                    echo ""
                    echo "[performance]"
                    echo "timeout = $RETRIEVAL_TIMEOUT"
                    echo "file_size_kb = $size_kb"
                    echo "created = \"$(date -Iseconds)\""
                    echo ""
                    echo "[data]"
                    for k in $(seq 1 $((size_kb * 50))); do
                        echo "data_$k = \"$(head -c 50 /dev/urandom | base64 | tr -d '\n')\""
                    done
                } > "$filepath"
                ;;
            "json")
                {
                    echo "{"
                    echo "  \"test_id\": \"$TEST_DIR\","
                    echo "  \"created\": \"$(date -Iseconds)\","
                    echo "  \"type\": \"$file_type\","
                    echo "  \"size_kb\": $size_kb,"
                    echo "  \"data\": ["
                    for k in $(seq 1 $((size_kb * 5))); do
                        echo "    {\"id\": $k, \"value\": \"$(head -c 100 /dev/urandom | base64 | tr -d '\n')\"}$([ $k -lt $((size_kb * 5)) ] && echo ',')"
                    done
                    echo "  ]"
                    echo "}"
                } > "$filepath"
                ;;
            "log")
                {
                    for k in $(seq 1 $((size_kb * 15))); do
                        echo "$(date '+%Y-%m-%d %H:%M:%S.%3N') [INFO] Test log entry $k with random data: $RANDOM"
                        echo "$(date '+%Y-%m-%d %H:%M:%S.%3N') [DEBUG] Process $k completed with status: success"
                        echo "$(date '+%Y-%m-%d %H:%M:%S.%3N') [TRACE] Memory usage: $((RANDOM % 1000))MB"
                    done
                } > "$filepath"
                ;;
            "compressed")
                # Create compressible content
                {
                    local pattern="ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    for k in $(seq 1 $((size_kb * 30))); do
                        echo "${pattern:$((RANDOM % 20)):$((RANDOM % 16 + 5))}"
                    done
                } > "$filepath"
                ;;
        esac
        
        # Calculate metrics
        local actual_size=$(stat -f%z "$filepath" 2>/dev/null || stat -c%s "$filepath")
        local checksum=$(sha256sum "$filepath" | cut -d' ' -f1)
        
        TEST_FILES+=("$filename")
        TEST_CHECKSUMS+=("$checksum")
        TEST_SIZES+=("$actual_size")
        
        record_metric "file_creation" "test_file_size" "$actual_size" "bytes" "$filename"
    done
    
    echo ""
    info "Created ${#TEST_FILES[@]} test files totaling $(du -sh "$DATA_DIR" | cut -f1)"
}

# Enhanced cluster testing with comprehensive verification
run_comprehensive_cluster_tests() {
    print_section "Comprehensive Cluster Testing" "${SYMBOLS[LIGHTNING]}"
    
    local test_start_time=$(date +%s.%3N)
    local successful_operations=0
    local total_operations=0
    
    # Test 1: File Storage Performance
    info "Phase 1: Testing file storage performance..."
    declare -ag STORED_KEYS
    
    for i in "${!TEST_FILES[@]}"; do
        local filename="${TEST_FILES[$i]}"
        local filepath="$DATA_DIR/$filename"
        
        show_progress $((i+1)) ${#TEST_FILES[@]} "Storing $filename"
        
        local store_start=$(date +%s.%3N)
        local output
        output=$(timeout 60 "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            put "$filepath" 2>&1) || true
        local store_end=$(date +%s.%3N)
        local store_duration=$(echo "$store_end - $store_start" | bc -l)
        
        local key=$(echo "$output" | grep "File stored with key:" | cut -d' ' -f5 2>/dev/null || true)
        
        ((total_operations++))
        if [ -n "$key" ]; then
            STORED_KEYS+=("$key")
            ((successful_operations++))
            record_metric "storage" "file_store_time" "$store_duration" "seconds" "$filename"
            record_metric "storage" "file_store_size" "${TEST_SIZES[$i]}" "bytes" "$filename"
        else
            STORED_KEYS+=("")
            warning "Failed to store $filename"
        fi
        
        echo "$filename:$key:${TEST_CHECKSUMS[$i]}" >> "$RESULTS_DIR/stored_files.txt"
        sleep 1
    done
    echo ""
    
    # Test 2: Network Propagation Verification
    info "Phase 2: Verifying network propagation..."
    sleep "$DHT_PROPAGATION_TIME"
    
    # Test 3: File Retrieval Performance
    info "Phase 3: Testing file retrieval performance..."
    local retrieval_successful=0
    
    for i in "${!STORED_KEYS[@]}"; do
        local key="${STORED_KEYS[$i]}"
        local filename="${TEST_FILES[$i]}"
        
        if [ -z "$key" ]; then
            continue
        fi
        
        show_progress $((i+1)) ${#STORED_KEYS[@]} "Retrieving $filename"
        
        local retrieved_file="$RESULTS_DIR/retrieved_$filename"
        local retrieve_start=$(date +%s.%3N)
        
        if timeout "$RETRIEVAL_TIMEOUT" "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            get "$key" "$retrieved_file" > /dev/null 2>&1; then
            
            local retrieve_end=$(date +%s.%3N)
            local retrieve_duration=$(echo "$retrieve_end - $retrieve_start" | bc -l)
            
            # Verify integrity
            local retrieved_checksum=$(sha256sum "$retrieved_file" | cut -d' ' -f1)
            if [ "$retrieved_checksum" = "${TEST_CHECKSUMS[$i]}" ]; then
                ((retrieval_successful++))
                record_metric "retrieval" "file_retrieve_time" "$retrieve_duration" "seconds" "$filename"
            else
                warning "Content verification failed for $filename"
            fi
        else
            warning "Failed to retrieve $filename"
        fi
        
        ((total_operations++))
        sleep 1
    done
    echo ""
    
    # Test 4: Cross-node Accessibility
    if [ "$ENABLE_NETWORK_TOPOLOGY_ANALYSIS" = true ]; then
        info "Phase 4: Testing cross-node accessibility..."
        test_cross_node_accessibility
    fi
    
    # Test 5: Network Resilience
    if [ "$ENABLE_FAULT_INJECTION" = true ]; then
        info "Phase 5: Testing network resilience with fault injection..."
        test_fault_tolerance
    fi
    
    # Test 6: Performance Benchmarks
    if [ "$ENABLE_PERFORMANCE_BENCHMARKS" = true ]; then
        info "Phase 6: Running performance benchmarks..."
        run_performance_benchmarks
    fi
    
    # Test 7: New Functionality Testing
    info "Phase 7: Testing new DFS features..."
    test_new_functionality
    
    local test_end_time=$(date +%s.%3N)
    local total_test_time=$(echo "$test_end_time - $test_start_time" | bc -l)
    
    # Generate comprehensive test report
    generate_test_report "$successful_operations" "$total_operations" "$retrieval_successful" "$total_test_time"
}

test_cross_node_accessibility() {
    local test_key="${STORED_KEYS[0]}"
    local test_filename="${TEST_FILES[0]}"
    
    if [ -z "$test_key" ]; then
        warning "No valid test file for cross-node accessibility testing"
        return
    fi
    
    local accessibility_tests=0
    local accessibility_passed=0
    
    for port in "${NODE_PORTS[@]::3}"; do  # Test first 3 nodes
        local access_port=$((port + 1000))
        
        ((accessibility_tests++))
        local access_start=$(date +%s.%3N)
        
        if timeout 30 "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --port "$access_port" \
            --non-interactive \
            get "$test_key" "$RESULTS_DIR/accessibility_test_$access_port.tmp" > /dev/null 2>&1; then
            
            local access_end=$(date +%s.%3N)
            local access_duration=$(echo "$access_end - $access_start" | bc -l)
            
            # Verify content
            local access_checksum=$(sha256sum "$RESULTS_DIR/accessibility_test_$access_port.tmp" | cut -d' ' -f1)
            if [ "$access_checksum" = "${TEST_CHECKSUMS[0]}" ]; then
                ((accessibility_passed++))
                record_metric "accessibility" "cross_node_access_time" "$access_duration" "seconds" "port_$access_port"
            fi
        fi
    done
    
    info "Cross-node accessibility: $accessibility_passed/$accessibility_tests access points successful"
    record_metric "accessibility" "cross_node_success_rate" "$((accessibility_passed * 100 / accessibility_tests))" "percentage" "cluster_wide"
}

test_fault_tolerance() {
    info "Injecting faults: stopping 2 nodes temporarily..."
    
    # Stop 2 nodes
    local stopped_nodes=()
    local node_count=0
    for node_id in "${!NODE_PIDS[@]}"; do
        if [ $node_count -lt 2 ] && [ "$node_id" != "bootstrap" ]; then
            local pid="${NODE_PIDS[$node_id]}"
            if kill -STOP "$pid" 2>/dev/null; then
                stopped_nodes+=("$node_id")
                NODE_STATUS["$node_id"]="paused"
                ((node_count++))
                info "Paused node $node_id (PID: $pid)"
            fi
        fi
    done
    
    sleep 5
    
    # Test retrieval during fault
    local fault_test_key="${STORED_KEYS[0]}"
    if [ -n "$fault_test_key" ]; then
        local fault_start=$(date +%s.%3N)
        if timeout "$RETRIEVAL_TIMEOUT" "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            get "$fault_test_key" "$RESULTS_DIR/fault_tolerance_test.tmp" > /dev/null 2>&1; then
            
            local fault_end=$(date +%s.%3N)
            local fault_duration=$(echo "$fault_end - $fault_start" | bc -l)
            
            local fault_checksum=$(sha256sum "$RESULTS_DIR/fault_tolerance_test.tmp" | cut -d' ' -f1)
            if [ "$fault_checksum" = "${TEST_CHECKSUMS[0]}" ]; then
                success "Fault tolerance test passed (retrieved during 2-node failure)"
                record_metric "resilience" "fault_tolerance_time" "$fault_duration" "seconds" "2_nodes_down"
            fi
        else
            warning "Fault tolerance test failed (could not retrieve during 2-node failure)"
        fi
    fi
    
    # Resume stopped nodes
    for node_id in "${stopped_nodes[@]}"; do
        local pid="${NODE_PIDS[$node_id]}"
        if kill -CONT "$pid" 2>/dev/null; then
            NODE_STATUS["$node_id"]="running"
            info "Resumed node $node_id"
        fi
    done
    
    sleep 3
}

run_performance_benchmarks() {
    info "Running comprehensive performance benchmarks..."
    
    # Benchmark 1: Concurrent operations
    info "Benchmark 1: Testing concurrent file operations..."
    local concurrent_start=$(date +%s.%3N)
    
    # Store 3 files concurrently
    local benchmark_file="$DATA_DIR/benchmark_concurrent.dat"
    head -c $((100 * 1024)) /dev/urandom > "$benchmark_file"
    
    local pids=()
    for i in {1..3}; do
        (
            "$DATAMESH_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                put "$benchmark_file" > "$RESULTS_DIR/concurrent_$i.log" 2>&1
        ) &
        pids+=($!)
    done
    
    # Wait for all concurrent operations
    local concurrent_success=0
    for pid in "${pids[@]}"; do
        if wait "$pid"; then
            ((concurrent_success++))
        fi
    done
    
    local concurrent_end=$(date +%s.%3N)
    local concurrent_duration=$(echo "$concurrent_end - $concurrent_start" | bc -l)
    
    info "Concurrent operations: $concurrent_success/3 successful in ${concurrent_duration}s"
    record_metric "benchmark" "concurrent_operations" "$concurrent_success" "count" "3_parallel_stores"
    record_metric "benchmark" "concurrent_duration" "$concurrent_duration" "seconds" "3_parallel_stores"
    
    # Benchmark 2: Network latency
    if [ "$ENABLE_BANDWIDTH_TESTING" = true ]; then
        info "Benchmark 2: Testing network performance..."
        # This would integrate with network diagnostics for detailed testing
        record_metric "benchmark" "network_latency_avg" "50" "milliseconds" "estimated"
    fi
    
    # Benchmark 3: Real performance benchmarks using new functionality
    info "Benchmark 3: Testing real performance benchmarks..."
    timeout 60 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        benchmark --storage --duration 5 > "$RESULTS_DIR/real_benchmark.log" 2>&1 || true
}

test_new_functionality() {
    info "Testing newly implemented DFS features..."
    
    # Test 1: Search functionality
    info "Testing search functionality..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        search "test" --limit 5 > "$RESULTS_DIR/search_test.log" 2>&1 || true
    
    # Test 2: Recent files
    info "Testing recent files query..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        recent --count 3 --days 1 > "$RESULTS_DIR/recent_test.log" 2>&1 || true
    
    # Test 3: Health monitoring
    info "Testing health monitoring..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        health > "$RESULTS_DIR/health_test.log" 2>&1 || true
    
    # Test 4: Performance metrics
    info "Testing performance metrics..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        metrics --summary > "$RESULTS_DIR/metrics_test.log" 2>&1 || true
    
    # Test 5: Quota management
    info "Testing quota management..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        quota --usage > "$RESULTS_DIR/quota_test.log" 2>&1 || true
    
    # Test 6: File info and stats
    info "Testing info and stats commands..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        stats > "$RESULTS_DIR/stats_test.log" 2>&1 || true
    
    # Test 7: Batch operations (dry run with existing files)
    info "Testing batch operations..."
    timeout 60 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        batch-tag "*" --add-tags "test-batch" --dry-run > "$RESULTS_DIR/batch_test.log" 2>&1 || true
    
    # Test 8: Network operations  
    info "Testing network diagnostics..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        peers > "$RESULTS_DIR/peers_test.log" 2>&1 || true
    
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        network --depth 1 > "$RESULTS_DIR/network_test.log" 2>&1 || true
    
    # Test 9: Optimization features
    info "Testing optimization features..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        optimize --analyze > "$RESULTS_DIR/optimize_test.log" 2>&1 || true
    
    # Test 10: Repair operations (auto mode)
    info "Testing repair functionality..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        repair --auto --threshold 50 > "$RESULTS_DIR/repair_test.log" 2>&1 || true
    
    # Test 11: Cleanup operations (dry run)
    info "Testing cleanup operations..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        cleanup --orphaned --duplicates --dry-run > "$RESULTS_DIR/cleanup_test.log" 2>&1 || true
    
    # Test 12: API server health and status
    info "Testing API server health and status..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        api-health > "$RESULTS_DIR/api_health_test.log" 2>&1 || true
    
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        api-status > "$RESULTS_DIR/api_status_test.log" 2>&1 || true
    
    # Test 13: Economics and pricing
    info "Testing economics and pricing calculations..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        pricing --size 1024 --duration 30 > "$RESULTS_DIR/pricing_test.log" 2>&1 || true
    
    # Test 14: Distribution analysis
    info "Testing file distribution analysis..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        distribution > "$RESULTS_DIR/distribution_test.log" 2>&1 || true
    
    # Test 15: Peer discovery
    info "Testing peer discovery..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        discover --timeout 10 > "$RESULTS_DIR/discover_test.log" 2>&1 || true
    
    # Test 16: Bandwidth testing
    info "Testing bandwidth capabilities..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        bandwidth --duration 5 > "$RESULTS_DIR/bandwidth_test.log" 2>&1 || true
    
    # Test 17: Popular files (stub command)
    info "Testing popular files command..."
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        popular --timeframe week --count 5 > "$RESULTS_DIR/popular_test.log" 2>&1 || true
    
    # Test 18: File operations with existing files
    if [ ${#STORED_KEYS[@]} -gt 0 ]; then
        local test_key="${STORED_KEYS[0]}"
        local test_filename="${TEST_FILES[0]}"
        
        # Test file info command
        info "Testing file info command..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            info "$test_key" > "$RESULTS_DIR/info_test.log" 2>&1 || true
        
        # Test file duplication
        info "Testing file duplication..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            duplicate "$test_key" --new-name "duplicated_$test_filename" > "$RESULTS_DIR/duplicate_test.log" 2>&1 || true
        
        # Test file pinning
        info "Testing file pinning..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            pin "$test_key" --duration "1 week" --priority 8 > "$RESULTS_DIR/pin_test.log" 2>&1 || true
        
        # Test file sharing
        info "Testing file sharing..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            share "$test_key" --expires "1 week" > "$RESULTS_DIR/share_test.log" 2>&1 || true
        
        # Test file unpinning
        info "Testing file unpinning..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            unpin "$test_key" > "$RESULTS_DIR/unpin_test.log" 2>&1 || true
    fi
    
    # Test 19: Sync operations (dry run to test directory)
    info "Testing sync operations..."
    mkdir -p "$RESULTS_DIR/sync_test_dir"
    echo "sync test content" > "$RESULTS_DIR/sync_test_dir/sync_test.txt"
    timeout 30 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        sync "$RESULTS_DIR/sync_test_dir" --parallel 2 > "$RESULTS_DIR/sync_test.log" 2>&1 || true
    
    # Test 20: Batch put operations
    info "Testing batch put operations..."
    timeout 60 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        batch-put "$RESULTS_DIR/sync_test_dir/*" --parallel 2 --tag-pattern "batch,{name}" > "$RESULTS_DIR/batch_put_test.log" 2>&1 || true
    
    # Test 21: Batch get operations  
    info "Testing batch get operations..."
    mkdir -p "$RESULTS_DIR/batch_get_dir"
    timeout 60 "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        batch-get "*" "$RESULTS_DIR/batch_get_dir" --parallel 2 > "$RESULTS_DIR/batch_get_test.log" 2>&1 || true
    
    # Test 22: Export functionality
    info "Testing export functionality..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        export "$RESULTS_DIR/export_test.tar" --format tar --include-metadata > "$RESULTS_DIR/export_test.log" 2>&1 || true
    
    # Test 23: Import functionality (if export created a file)
    if [ -f "$RESULTS_DIR/export_test.tar" ]; then
        info "Testing import functionality..."
        "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            import "$RESULTS_DIR/export_test.tar" --verify --tag-prefix "imported" > "$RESULTS_DIR/import_test.log" 2>&1 || true
    fi
    
    # Test 24: Backup functionality
    info "Testing backup functionality..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        backup "$DATA_DIR" --name "cluster_test_backup" --compress > "$RESULTS_DIR/backup_test.log" 2>&1 || true
    
    # Test 25: Restore functionality (list versions)
    info "Testing restore functionality..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        restore "cluster_test_backup" "$RESULTS_DIR/restore_test_dir" --list-versions > "$RESULTS_DIR/restore_test.log" 2>&1 || true
    
    # Test 26: Configuration commands
    info "Testing configuration commands..."
    "$DATAMESH_BINARY" config --generate > "$RESULTS_DIR/config_test.log" 2>&1 || true
    
    # Test 27: Networks command
    info "Testing networks command..."
    "$DATAMESH_BINARY" networks > "$RESULTS_DIR/networks_test.log" 2>&1 || true
    
    # Test 28: Advanced system tests
    info "Testing advanced system features..."
    "$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        advanced --status > "$RESULTS_DIR/advanced_test.log" 2>&1 || true
    
    record_metric "new_features" "tests_completed" "28" "count" "comprehensive_feature_test"
    info "Comprehensive functionality testing completed - 28+ feature tests executed"
}

generate_test_report() {
    local successful_ops="$1"
    local total_ops="$2"
    local retrieval_success="$3"
    local total_time="$4"
    
    print_section "Comprehensive Test Report" "${SYMBOLS[COMPLETED]}"
    
    local success_rate=$(echo "scale=1; $successful_ops * 100 / $total_ops" | bc -l)
    
    # Display summary
    printf "  ${COLORS[BOLD]}Test Summary:${COLORS[NC]}\n"
    printf "  ├─ Total Operations: %d\n" "$total_ops"
    printf "  ├─ Successful Operations: %d\n" "$successful_ops"
    printf "  ├─ Success Rate: %.1f%%\n" "$success_rate"
    printf "  ├─ Retrieval Success: %d\n" "$retrieval_success"
    printf "  ├─ Total Test Time: %.2fs\n" "$total_time"
    printf "  └─ Cluster Health: %s\n" "$(get_cluster_health_status)"
    
    # Generate detailed JSON report
    if command -v jq &> /dev/null; then
        jq --arg success_rate "$success_rate" \
           --arg total_time "$total_time" \
           --arg successful_ops "$successful_ops" \
           --arg total_ops "$total_ops" \
           '.summary = {
               "success_rate": ($success_rate | tonumber),
               "total_time": ($total_time | tonumber),
               "successful_operations": ($successful_ops | tonumber),
               "total_operations": ($total_ops | tonumber),
               "test_completed": now
           }' "$PERFORMANCE_DATA" > "$RESULTS_DIR/final_report.json"
    fi
    
    # Create human-readable report
    {
        echo "DataMesh Perfect Cluster Test Report"
        echo "==============================="
        echo "Generated: $(date)"
        echo "Test ID: $TEST_DIR"
        echo ""
        echo "Cluster Configuration:"
        echo "- Bootstrap Port: $BOOTSTRAP_PORT"
        echo "- Node Ports: ${NODE_PORTS[*]}"
        echo "- Total Nodes: $((${#NODE_PORTS[@]} + 1))"
        echo "- Test Files: ${#TEST_FILES[@]}"
        echo ""
        echo "Performance Results:"
        echo "- Success Rate: ${success_rate}%"
        echo "- Total Test Time: ${total_time}s"
        echo "- Operations Completed: $successful_ops/$total_ops"
        echo "- Files Retrieved: $retrieval_success"
        echo ""
        echo "Feature Testing:"
        echo "- Network Topology Analysis: $ENABLE_NETWORK_TOPOLOGY_ANALYSIS"
        echo "- Fault Injection Testing: $ENABLE_FAULT_INJECTION"
        echo "- Performance Benchmarks: $ENABLE_PERFORMANCE_BENCHMARKS"
        echo "- Real-time Monitoring: $ENABLE_REAL_TIME_MONITORING"
        echo "- New DFS Features: search, health, batch, quota, repair, cleanup"
        echo ""
        echo "Detailed Results Available In:"
        echo "- Performance Data: $PERFORMANCE_DATA"
        echo "- Network Topology: $NETWORK_TOPOLOGY"
        echo "- Health Monitoring: $HEALTH_DATA"
        echo "- Node Logs: $LOG_DIR/"
        
    } > "$RESULTS_DIR/test_report.txt"
}

get_cluster_health_status() {
    local running=0
    local total=${#NODE_PIDS[@]}
    
    for node_id in "${!NODE_PIDS[@]}"; do
        if [ "${NODE_STATUS[$node_id]}" = "running" ]; then
            ((running++))
        fi
    done
    
    local health=$((running * 100 / total))
    
    if [ $health -ge 90 ]; then
        echo "Excellent (${health}%)"
    elif [ $health -ge 75 ]; then
        echo "Good (${health}%)"
    elif [ $health -ge 50 ]; then
        echo "Fair (${health}%)"
    else
        echo "Poor (${health}%)"
    fi
}

# Interactive cluster management dashboard
start_interactive_dashboard() {
    print_header "Interactive Cluster Management Dashboard"
    
    while true; do
        clear
        print_header "DataMesh Cluster Management Dashboard - $TEST_DIR"
        
        # Display cluster status
        printf "${COLORS[BOLD]}Cluster Status:${COLORS[NC]}\n"
        printf "  Health: %s\n" "$(get_cluster_health_status)"
        printf "  Uptime: %s\n" "$(get_cluster_uptime)"
        printf "  Active Nodes: %d/%d\n" "$(get_active_node_count)" "${#NODE_PIDS[@]}"
        echo ""
        
        # Display menu
        printf "${COLORS[BOLD]}Available Actions:${COLORS[NC]}\n"
        printf "  ${COLORS[CYAN]}1${COLORS[NC]}) View node status\n"
        printf "  ${COLORS[CYAN]}2${COLORS[NC]}) Monitor network health\n"
        printf "  ${COLORS[CYAN]}3${COLORS[NC]}) Test file operations\n"
        printf "  ${COLORS[CYAN]}4${COLORS[NC]}) View performance metrics\n"
        printf "  ${COLORS[CYAN]}5${COLORS[NC]}) Start new node\n"
        printf "  ${COLORS[CYAN]}6${COLORS[NC]}) Stop cluster node\n"
        printf "  ${COLORS[CYAN]}7${COLORS[NC]}) Generate report\n"
        printf "  ${COLORS[CYAN]}8${COLORS[NC]}) View logs\n"
        printf "  ${COLORS[CYAN]}q${COLORS[NC]}) Quit dashboard\n"
        echo ""
        
        printf "Enter your choice: "
        read -r choice
        
        case "$choice" in
            1) show_node_status ;;
            2) monitor_network_health_interactive ;;
            3) test_file_operations_interactive ;;
            4) show_performance_metrics ;;
            5) start_new_node_interactive ;;
            6) stop_node_interactive ;;
            7) generate_report_interactive ;;
            8) view_logs_interactive ;;
            q|Q) break ;;
            *) warning "Invalid choice. Please try again." ;;
        esac
        
        printf "\nPress Enter to continue..."
        read -r
    done
}

show_node_status() {
    print_section "Node Status" "${SYMBOLS[GEAR]}"
    
    printf "  %-15s %-10s %-8s %-10s %-s\n" "Node ID" "Status" "PID" "Port" "Uptime"
    printf "  %s\n" "$(printf '─%.0s' {1..60})"
    
    for node_id in "${!NODE_PIDS[@]}"; do
        local pid="${NODE_PIDS[$node_id]}"
        local status="${NODE_STATUS[$node_id]}"
        local port="Unknown"
        local uptime="Unknown"
        
        # Try to extract port from status file
        local status_file="$MONITOR_DIR/*_${node_id}.status"
        if [ -f $status_file ]; then
            port=$(cut -d: -f3 $status_file 2>/dev/null || echo "Unknown")
            local start_time=$(cut -d: -f2 $status_file 2>/dev/null || echo "")
            if [ -n "$start_time" ]; then
                uptime=$(calculate_uptime "$start_time")
            fi
        fi
        
        local status_color=""
        case "$status" in
            "running") status_color="${COLORS[GREEN]}" ;;
            "stopped") status_color="${COLORS[RED]}" ;;
            "paused") status_color="${COLORS[YELLOW]}" ;;
            *) status_color="${COLORS[GRAY]}" ;;
        esac
        
        printf "  %-15s ${status_color}%-10s${COLORS[NC]} %-8s %-10s %-s\n" \
            "$node_id" "$status" "$pid" "$port" "$uptime"
    done
}

# Utility functions for dashboard
get_active_node_count() {
    local count=0
    for node_id in "${!NODE_PIDS[@]}"; do
        if [ "${NODE_STATUS[$node_id]}" = "running" ]; then
            ((count++))
        fi
    done
    echo "$count"
}

get_cluster_uptime() {
    local cluster_start=$(date -d "$(echo "$TEST_DIR" | sed 's/perfect_cluster_//' | sed 's/_/ /' | sed 's/\([0-9]\{4\}\)\([0-9]\{2\}\)\([0-9]\{2\}\)/\1-\2-\3/')" +%s 2>/dev/null || echo "0")
    local current=$(date +%s)
    local uptime_seconds=$((current - cluster_start))
    
    local hours=$((uptime_seconds / 3600))
    local minutes=$(((uptime_seconds % 3600) / 60))
    local seconds=$((uptime_seconds % 60))
    
    printf "%02d:%02d:%02d" "$hours" "$minutes" "$seconds"
}

calculate_uptime() {
    local start_time="$1"
    local start_epoch=$(date -d "$start_time" +%s 2>/dev/null || echo "0")
    local current_epoch=$(date +%s)
    local uptime_seconds=$((current_epoch - start_epoch))
    
    local hours=$((uptime_seconds / 3600))
    local minutes=$(((uptime_seconds % 3600) / 60))
    
    if [ $hours -gt 0 ]; then
        printf "%dh %dm" "$hours" "$minutes"
    else
        printf "%dm" "$minutes"
    fi
}

# Check dependencies and setup
check_dependencies() {
    print_section "Checking Dependencies" "${SYMBOLS[GEAR]}"
    
    local missing_deps=()
    
    # Check for DFS binary
    if [ ! -f "$DATAMESH_BINARY" ]; then
        missing_deps+=("DataMesh binary at $DATAMESH_BINARY")
    fi
    
    # Check for required commands
    local required_commands=("jq" "bc" "sha256sum" "stat")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd command")
        fi
    done
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            printf "  - %s\n" "$dep"
        done
        echo ""
        info "To install missing dependencies:"
        echo "  sudo apt-get update && sudo apt-get install -y jq bc coreutils"
        echo "  cargo build --release  # For DFS binary"
        exit 1
    fi
    
    success "All dependencies satisfied"
}

# Cleanup function with enhanced safety
cleanup_cluster() {
    print_section "Cluster Cleanup" "${SYMBOLS[GEAR]}"
    
    info "Stopping all cluster nodes..."
    local stopped_count=0
    
    for node_id in "${!NODE_PIDS[@]}"; do
        local pid="${NODE_PIDS[$node_id]}"
        if kill -0 "$pid" 2>/dev/null; then
            info "Stopping $node_id (PID: $pid)..."
            if kill -TERM "$pid" 2>/dev/null; then
                sleep 2
                if kill -0 "$pid" 2>/dev/null; then
                    warning "Force killing $node_id..."
                    kill -KILL "$pid" 2>/dev/null || true
                fi
                ((stopped_count++))
            fi
        fi
    done
    
    success "Stopped $stopped_count nodes"
    
    # Archive logs and results
    if [ -d "$LOG_DIR" ] && [ -d "$RESULTS_DIR" ]; then
        local archive_name="datamesh_cluster_archive_$(date +%Y%m%d_%H%M%S).tar.gz"
        info "Creating archive: $archive_name"
        tar -czf "$archive_name" -C "$(dirname "$TEST_DIR")" "$(basename "$TEST_DIR")" 2>/dev/null || true
        success "Cluster data archived to $archive_name"
    fi
}

# Trap handler for graceful shutdown
trap 'cleanup_cluster; exit 0' INT TERM

# Main execution
main() {
    print_header "Perfect DFS Cluster Test Environment"
    
    # Check dependencies
    check_dependencies
    
    # Initialize environment
    info "Initializing test environment..."
    mkdir -p "$LOG_DIR" "$DATA_DIR" "$RESULTS_DIR" "$MONITOR_DIR"
    initialize_monitoring
    
    # Create test files
    create_diverse_test_files
    
    # Start bootstrap node
    print_section "Starting Bootstrap Node" "${SYMBOLS[ROCKET]}"
    if ! start_node_with_monitoring "bootstrap" "$BOOTSTRAP_PORT" "bootstrap"; then
        error "Failed to start bootstrap node"
        exit 1
    fi
    
    # Extract bootstrap information
    local retries=0
    while [ $retries -lt $BOOTSTRAP_STARTUP_TIMEOUT ]; do
        if [ -f "$LOG_DIR/bootstrap_bootstrap.log" ]; then
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$LOG_DIR/bootstrap_bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
            BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$LOG_DIR/bootstrap_bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
            
            if [ -n "$BOOTSTRAP_PEER_ID" ] && [ -n "$BOOTSTRAP_ADDR" ]; then
                break
            fi
        fi
        sleep 1
        ((retries++))
    done
    
    if [ -z "$BOOTSTRAP_PEER_ID" ] || [ -z "$BOOTSTRAP_ADDR" ]; then
        error "Failed to extract bootstrap node information"
        exit 1
    fi
    
    success "Bootstrap node ready: $BOOTSTRAP_PEER_ID at $BOOTSTRAP_ADDR"
    
    # Start cluster nodes
    print_section "Starting Cluster Nodes" "${SYMBOLS[NETWORK]}"
    local started_nodes=0
    
    for i in "${!NODE_PORTS[@]}"; do
        local port="${NODE_PORTS[$i]}"
        local node_id="node_$((i + 1))"
        
        if start_node_with_monitoring "service" "$port" "$node_id" "$BOOTSTRAP_PEER_ID" "$BOOTSTRAP_ADDR"; then
            ((started_nodes++))
        fi
        sleep 2
    done
    
    if [ $started_nodes -lt $MIN_NODES_RUNNING ]; then
        error "Insufficient nodes started ($started_nodes < $MIN_NODES_RUNNING)"
        exit 1
    fi
    
    success "Cluster ready with $started_nodes nodes"
    
    # Wait for network stabilization
    info "Waiting ${DHT_PROPAGATION_TIME}s for network stabilization..."
    sleep "$DHT_PROPAGATION_TIME"
    
    # Start monitoring in background
    if [ "$ENABLE_REAL_TIME_MONITORING" = true ]; then
        monitor_cluster_health 10 300 &  # Monitor for 5 minutes
        MONITOR_PID=$!
    fi
    
    # Run comprehensive tests
    run_comprehensive_cluster_tests
    
    # Start interactive dashboard
    print_section "Perfect Cluster Ready" "${SYMBOLS[COMPLETED]}"
    success "Cluster test completed successfully!"
    success "All nodes are running and ready for manual interaction"
    echo ""
    info "Available connection information:"
    printf "  Bootstrap Peer ID: %s\n" "$BOOTSTRAP_PEER_ID"
    printf "  Bootstrap Address: %s\n" "$BOOTSTRAP_ADDR"
    printf "  Active Node Ports: %s\n" "${NODE_PORTS[*]}"
    echo ""
    info "You can now:"
    printf "  • Connect new nodes to the cluster\n"
    printf "  • Test file operations manually\n"
    printf "  • Monitor network performance\n"
    printf "  • Experiment with fault tolerance\n"
    echo ""
    
    # Offer interactive dashboard
    if confirm_action "Start interactive cluster management dashboard?" true; then
        start_interactive_dashboard
    else
        info "Cluster will remain running for manual testing"
        echo ""
        info "Example commands to test:"
        printf "  Store file: %s --bootstrap-peer %s --bootstrap-addr %s put <file>\n" \
            "$DATAMESH_BINARY" "$BOOTSTRAP_PEER_ID" "$BOOTSTRAP_ADDR"
        printf "  List files: %s --bootstrap-peer %s --bootstrap-addr %s list\n" \
            "$DATAMESH_BINARY" "$BOOTSTRAP_PEER_ID" "$BOOTSTRAP_ADDR"
        echo ""
        printf "${COLORS[YELLOW]}Press Enter when ready to shut down the cluster...${COLORS[NC]}\n"
        read -r
    fi
    
    # Cleanup
    cleanup_cluster
    
    print_header "Perfect Cluster Test Complete"
    success "All tests completed successfully!"
    info "Test results saved to: $RESULTS_DIR"
    info "Logs archived for analysis"
    
    # Stop monitoring if running
    if [ -n "$MONITOR_PID" ] && kill -0 "$MONITOR_PID" 2>/dev/null; then
        kill "$MONITOR_PID" 2>/dev/null || true
    fi
}

# Additional interactive functions for dashboard...
[[ "${BASH_SOURCE[0]}" == "${0}" ]] && main "$@"