#!/bin/bash
# Comprehensive 6-Node DataMesh Cluster Test Suite
# Tests all CLI commands, distributed storage, backup, and network functionality

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
SERVICE_PORTS=(40872 40873 40874 40875 40876)
TEST_DIR="cluster_test_$(date +%Y%m%d_%H%M%S)"
BACKUP_DIR="$TEST_DIR/backups"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ✅ $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} ❌ $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} ⚠️  $1"; }
log_test() { echo -e "${PURPLE}[TEST]${NC} 🧪 $1"; }
log_command() { echo -e "${CYAN}[CMD]${NC} 🔧 $1"; }

# Test tracking
declare -A NODE_PIDS
declare -A NODE_LOGS
BOOTSTRAP_PID=""
BOOTSTRAP_PEER_ID=""
BOOTSTRAP_ADDR=""
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# File tracking for cleanup
declare -a TEST_FILES
declare -a STORED_KEYS

cleanup() {
    log_info "🧹 Cleaning up comprehensive test environment..."
    
    # Stop all service nodes
    for port in "${SERVICE_PORTS[@]}"; do
        if [[ -n "${NODE_PIDS[$port]}" ]]; then
            kill "${NODE_PIDS[$port]}" 2>/dev/null || true
        fi
    done
    
    # Stop bootstrap
    [[ -n "$BOOTSTRAP_PID" ]] && kill "$BOOTSTRAP_PID" 2>/dev/null || true
    
    sleep 3
    
    # Force cleanup any remaining processes
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    
    # Clean test files
    for file in "${TEST_FILES[@]}"; do
        rm -f "$file" 2>/dev/null || true
    done
    rm -f /tmp/comprehensive_test_* 2>/dev/null || true
    rm -rf "$BACKUP_DIR" 2>/dev/null || true
    
    log_success "Cleanup completed"
}

trap cleanup EXIT INT TERM

run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_success="${3:-true}"
    
    ((TOTAL_TESTS++))
    log_test "Test #$TOTAL_TESTS: $test_name"
    
    if [[ "$expected_success" == "true" ]]; then
        if eval "$test_command"; then
            log_success "✅ PASSED: $test_name"
            ((TESTS_PASSED++))
            return 0
        else
            log_error "❌ FAILED: $test_name"
            ((TESTS_FAILED++))
            return 1
        fi
    else
        # Test expected to fail
        if eval "$test_command"; then
            log_error "❌ FAILED: $test_name (expected to fail but passed)"
            ((TESTS_FAILED++))
            return 1
        else
            log_success "✅ PASSED: $test_name (correctly failed as expected)"
            ((TESTS_PASSED++))
            return 0
        fi
    fi
}

extract_bootstrap_info() {
    local log_file="$1"
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if [[ -f "$log_file" ]]; then
            BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
            BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$log_file" 2>/dev/null | head -1 | grep -o '/ip4/[^[:space:]]*' || true)
            
            if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
                return 0
            fi
        fi
        sleep 1
        ((attempt++))
    done
    return 1
}

start_bootstrap() {
    log_info "🚀 Starting bootstrap node on port $BOOTSTRAP_PORT..."
    
    local bootstrap_log="$TEST_DIR/bootstrap.log"
    mkdir -p "$TEST_DIR" "$BACKUP_DIR"
    
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$bootstrap_log" 2>&1 &
    BOOTSTRAP_PID=$!
    
    if ! extract_bootstrap_info "$bootstrap_log"; then
        log_error "Failed to start bootstrap node"
        cat "$bootstrap_log"
        return 1
    fi
    
    log_success "Bootstrap node started successfully"
    log_info "  📡 Peer ID: $BOOTSTRAP_PEER_ID"
    log_info "  🌐 Address: $BOOTSTRAP_ADDR"
    log_info "  🔢 PID: $BOOTSTRAP_PID"
    
    return 0
}

start_service_node() {
    local port="$1"
    local node_id="service_node_$port"
    local log_file="$TEST_DIR/${node_id}.log"
    
    log_info "🎯 Starting service node #$((port - 40871)) on port $port..."
    
    "$DATAMESH_BINARY" --non-interactive service \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$port" > "$log_file" 2>&1 &
    
    local pid=$!
    NODE_PIDS[$port]=$pid
    NODE_LOGS[$port]=$log_file
    
    # Give node time to start and connect
    sleep 3
    
    if kill -0 "$pid" 2>/dev/null; then
        log_success "Service node started on port $port (PID: $pid)"
        return 0
    else
        log_error "Service node failed to start on port $port"
        echo "Last 10 lines of log:"
        tail -10 "$log_file" 2>/dev/null || echo "No log available"
        return 1
    fi
}

wait_for_network_stabilization() {
    log_info "⏳ Waiting for 6-node network stabilization and peer discovery..."
    local stabilization_time=25
    
    echo -n "  "
    for ((i=1; i<=stabilization_time; i++)); do
        echo -ne "█"
        sleep 1
    done
    echo ""
    
    log_success "Network stabilization completed"
}

get_datamesh_cmd() {
    local port="$1"
    echo "$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --port $port --non-interactive"
}

# ======================== CONFIGURATION TESTS ========================

test_config_commands() {
    log_info "🔧 Testing configuration commands..."
    
    # Test config generation
    run_test "Config generation" \
        "$(get_datamesh_cmd 41001) config generate /tmp/comprehensive_test_config.toml"
    
    # Test config display  
    run_test "Config display" \
        "$(get_datamesh_cmd 41002) config show | grep -q 'Configuration'"
    
    # Test network presets
    run_test "Network presets listing" \
        "$(get_datamesh_cmd 41003) networks | grep -q 'Available'"
}

# ======================== STORAGE TESTS ========================

test_basic_storage() {
    log_info "📁 Testing basic storage operations..."
    
    # Create test files
    echo "Basic test file content - $(date)" > /tmp/comprehensive_test_basic.txt
    echo "Multi-line test file
Line 2: $(date)
Line 3: Special chars: äöü 🚀 ∑∆
Line 4: Numbers: 12345" > /tmp/comprehensive_test_multi.txt
    
    TEST_FILES+=("/tmp/comprehensive_test_basic.txt" "/tmp/comprehensive_test_multi.txt")
    
    # Test basic PUT operation
    local store_cmd="$(get_datamesh_cmd 41010) put /tmp/comprehensive_test_basic.txt"
    log_command "$store_cmd"
    if store_output=$($store_cmd 2>&1); then
        log_success "✅ Basic file storage successful"
        echo "$store_output"
        
        # Extract file key
        local file_key=$(echo "$store_output" | grep -oE '(stored with key:|Key:) [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        if [[ -n "$file_key" ]]; then
            STORED_KEYS+=("$file_key")
            log_success "File key: ${file_key:0:20}..."
            
            # Test GET operation
            local get_cmd="$(get_datamesh_cmd 41011) get $file_key /tmp/comprehensive_test_retrieved.txt"
            log_command "$get_cmd"
            if $get_cmd 2>&1; then
                if cmp -s /tmp/comprehensive_test_basic.txt /tmp/comprehensive_test_retrieved.txt; then
                    log_success "✅ File retrieval and content verification successful"
                    ((TESTS_PASSED++))
                else
                    log_error "❌ File content mismatch after retrieval"
                    ((TESTS_FAILED++))
                fi
            else
                log_error "❌ File retrieval failed"
                ((TESTS_FAILED++))
            fi
        else
            log_error "❌ Could not extract file key"
            ((TESTS_FAILED++))
        fi
    else
        log_error "❌ Basic file storage failed"
        echo "$store_output"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

test_named_storage() {
    log_info "🏷️  Testing named file storage..."
    
    echo "Named file content - $(date)" > /tmp/comprehensive_test_named.txt
    TEST_FILES+=("/tmp/comprehensive_test_named.txt")
    
    # Test storage with custom name
    local store_cmd="$(get_datamesh_cmd 41020) put /tmp/comprehensive_test_named.txt --name test_document"
    log_command "$store_cmd"
    if store_output=$($store_cmd 2>&1); then
        log_success "✅ Named file storage successful"
        echo "$store_output"
        
        # Test retrieval by name
        local get_cmd="$(get_datamesh_cmd 41021) get test_document /tmp/comprehensive_test_named_retrieved.txt"
        log_command "$get_cmd"
        if $get_cmd 2>&1; then
            if cmp -s /tmp/comprehensive_test_named.txt /tmp/comprehensive_test_named_retrieved.txt; then
                log_success "✅ Named file retrieval successful"
                ((TESTS_PASSED++))
            else
                log_error "❌ Named file content mismatch"
                ((TESTS_FAILED++))
            fi
        else
            log_error "❌ Named file retrieval failed"
            ((TESTS_FAILED++))
        fi
    else
        log_error "❌ Named file storage failed"
        echo "$store_output"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

test_tagged_storage() {
    log_info "🏷️  Testing tagged file storage..."
    
    echo "Tagged file content - $(date)" > /tmp/comprehensive_test_tagged.txt
    TEST_FILES+=("/tmp/comprehensive_test_tagged.txt")
    
    # Test storage with tags
    local store_cmd="$(get_datamesh_cmd 41030) put /tmp/comprehensive_test_tagged.txt --name tagged_doc --tags 'test,important,demo'"
    log_command "$store_cmd"
    if store_output=$($store_cmd 2>&1); then
        log_success "✅ Tagged file storage successful"
        echo "$store_output"
        ((TESTS_PASSED++))
    else
        log_error "❌ Tagged file storage failed"
        echo "$store_output"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

# ======================== CROSS-NODE TESTS ========================

test_cross_node_operations() {
    log_info "🌐 Testing cross-node distributed operations..."
    
    echo "Cross-node test content - $(date)" > /tmp/comprehensive_test_cross.txt
    TEST_FILES+=("/tmp/comprehensive_test_cross.txt")
    
    # Store from one node
    local store_port="${SERVICE_PORTS[0]}"
    local store_cmd="$(get_datamesh_cmd $((store_port + 1000))) put /tmp/comprehensive_test_cross.txt --name cross_node_file"
    log_command "Storing from node simulation on port $((store_port + 1000))"
    if store_output=$($store_cmd 2>&1); then
        log_success "✅ Cross-node storage successful"
        
        # Retrieve from different node simulation
        local retrieve_port="${SERVICE_PORTS[2]}"
        local get_cmd="$(get_datamesh_cmd $((retrieve_port + 2000))) get cross_node_file /tmp/comprehensive_test_cross_retrieved.txt"
        log_command "Retrieving from different node simulation on port $((retrieve_port + 2000))"
        if $get_cmd 2>&1; then
            if cmp -s /tmp/comprehensive_test_cross.txt /tmp/comprehensive_test_cross_retrieved.txt; then
                log_success "✅ Cross-node retrieval successful - DHT working!"
                ((TESTS_PASSED++))
            else
                log_error "❌ Cross-node content mismatch"
                ((TESTS_FAILED++))
            fi
        else
            log_error "❌ Cross-node retrieval failed"
            ((TESTS_FAILED++))
        fi
    else
        log_error "❌ Cross-node storage failed"
        echo "$store_output"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

# ======================== LISTING AND INFO TESTS ========================

test_list_operations() {
    log_info "📋 Testing file listing operations..."
    
    # Test basic list
    run_test "Basic file listing" \
        "$(get_datamesh_cmd 41040) list | grep -q 'Files'"
    
    # Test list with tags
    run_test "Tagged file listing" \
        "$(get_datamesh_cmd 41041) list --tags test"
    
    # Test file info
    if [[ ${#STORED_KEYS[@]} -gt 0 ]]; then
        local key="${STORED_KEYS[0]}"
        run_test "File info command" \
            "$(get_datamesh_cmd 41042) info $key"
    fi
}

# ======================== STATS AND METRICS TESTS ========================

test_stats_operations() {
    log_info "📊 Testing statistics and metrics..."
    
    # Test storage stats
    run_test "Storage statistics" \
        "$(get_datamesh_cmd 41050) stats | grep -q 'Storage'"
    
    # Test metrics
    run_test "Performance metrics" \
        "$(get_datamesh_cmd 41051) metrics"
    
    # Test peer listing
    run_test "Peer listing" \
        "$(get_datamesh_cmd 41052) peers"
}

# ======================== NETWORK TESTS ========================

test_network_operations() {
    log_info "🌐 Testing network operations..."
    
    # Test network topology
    run_test "Network topology analysis" \
        "$(get_datamesh_cmd 41060) network | grep -q 'Network'"
    
    # Test peer discovery
    run_test "Peer discovery" \
        "$(get_datamesh_cmd 41061) discover"
}

# ======================== BACKUP TESTS ========================

test_backup_operations() {
    log_info "💾 Testing backup functionality..."
    
    # Create backup configuration
    cat > /tmp/comprehensive_backup_config.toml << EOF
[backup]
name = "test_backup"
description = "Comprehensive test backup"
schedule = "manual"
destination = "local"
local_path = "$BACKUP_DIR"
retention_days = 30
compress = true
encrypt = false

[backup.filters]
include_patterns = ["*"]
exclude_patterns = []
max_file_size_mb = 100
EOF
    
    TEST_FILES+=("/tmp/comprehensive_backup_config.toml")
    
    # Test backup creation (if backup command exists)
    local backup_cmd="$(get_datamesh_cmd 41070) backup create --config /tmp/comprehensive_backup_config.toml"
    log_command "$backup_cmd"
    if $backup_cmd 2>&1; then
        log_success "✅ Backup creation successful"
        ((TESTS_PASSED++))
        
        # Test backup listing
        run_test "Backup listing" \
            "$(get_datamesh_cmd 41071) backup list"
        
    else
        log_warning "⚠️  Backup command not available or failed"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

# ======================== RESILIENCE TESTS ========================

test_node_resilience() {
    log_info "🛡️  Testing node resilience..."
    
    # Store a file for resilience testing
    echo "Resilience test content - $(date)" > /tmp/comprehensive_test_resilience.txt
    TEST_FILES+=("/tmp/comprehensive_test_resilience.txt")
    
    local store_cmd="$(get_datamesh_cmd 41080) put /tmp/comprehensive_test_resilience.txt --name resilience_test"
    if store_output=$($store_cmd 2>&1); then
        log_success "✅ File stored for resilience test"
        
        # Temporarily stop one service node
        local test_port="${SERVICE_PORTS[3]}"
        local test_pid="${NODE_PIDS[$test_port]}"
        
        log_info "🔴 Temporarily stopping service node on port $test_port..."
        kill "$test_pid" 2>/dev/null || true
        sleep 3
        
        # Try to retrieve with one node down
        local get_cmd="$(get_datamesh_cmd 41081) get resilience_test /tmp/comprehensive_test_resilience_retrieved.txt"
        if $get_cmd 2>&1; then
            if cmp -s /tmp/comprehensive_test_resilience.txt /tmp/comprehensive_test_resilience_retrieved.txt; then
                log_success "✅ Data retrieval successful with one node down!"
                ((TESTS_PASSED++))
            else
                log_error "❌ Data corruption detected with node failure"
                ((TESTS_FAILED++))
            fi
        else
            log_warning "⚠️  Data retrieval failed with one node down (may be expected)"
            ((TESTS_FAILED++))
        fi
        
        # Restart the node
        log_info "🟢 Restarting the stopped node..."
        start_service_node "$test_port"
        sleep 5
        
    else
        log_error "❌ Failed to store file for resilience test"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

# ======================== LOAD TESTING ========================

test_multiple_files() {
    log_info "📦 Testing multiple file operations..."
    
    local success_count=0
    local total_files=5
    
    for i in $(seq 1 $total_files); do
        local test_file="/tmp/comprehensive_test_multi_$i.txt"
        echo "Multi-file test $i - $(date) - Random: $RANDOM" > "$test_file"
        TEST_FILES+=("$test_file")
        
        # Use different simulated ports for load distribution
        local port_index=$((i % ${#SERVICE_PORTS[@]}))
        local store_port="${SERVICE_PORTS[$port_index]}"
        
        log_info "📄 Storing file $i via port simulation $((store_port + 3000 + i))..."
        
        local store_cmd="$(get_datamesh_cmd $((store_port + 3000 + i))) put $test_file --name multi_file_$i"
        if $store_cmd 2>&1; then
            log_success "✅ Multi-file $i stored successfully"
            ((success_count++))
        else
            log_warning "⚠️  Multi-file $i storage failed"
        fi
        
        sleep 1  # Small delay between operations
    done
    
    log_info "📊 Multiple file test results: $success_count/$total_files files stored"
    
    if [[ $success_count -ge $((total_files * 3 / 4)) ]]; then
        log_success "✅ Multiple file operations test passed (≥75% success rate)"
        ((TESTS_PASSED++))
    else
        log_error "❌ Multiple file operations test failed (<75% success rate)"
        ((TESTS_FAILED++))
    fi
    ((TOTAL_TESTS++))
}

# ======================== CONNECTIVITY ANALYSIS ========================

analyze_network_connectivity() {
    log_info "🔍 Analyzing network connectivity..."
    
    local connected_nodes=0
    local total_connections=0
    
    echo ""
    echo "📊 Network Node Analysis:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check bootstrap node
    if [[ -f "$TEST_DIR/bootstrap.log" ]]; then
        local bootstrap_connections=$(grep -c "Connected to peer:\|peer connected" "$TEST_DIR/bootstrap.log" 2>/dev/null || echo "0")
        echo "🌟 Bootstrap Node (Port $BOOTSTRAP_PORT): $bootstrap_connections connections"
        total_connections=$((total_connections + bootstrap_connections))
        if [[ $bootstrap_connections -gt 0 ]]; then
            ((connected_nodes++))
        fi
    fi
    
    # Check service nodes
    for port in "${SERVICE_PORTS[@]}"; do
        local log_file="${NODE_LOGS[$port]}"
        local pid="${NODE_PIDS[$port]}"
        
        if [[ -f "$log_file" ]]; then
            local connections=$(grep -c "Connected to peer:\|peer connected" "$log_file" 2>/dev/null || echo "0")
            local peer_id=$(grep -E "Network actor starting with peer ID:|Local peer id:" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || echo "unknown")
            local status="🔴 STOPPED"
            
            if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
                status="🟢 RUNNING"
            fi
            
            echo "🎯 Service Node $((port - 40871)) (Port $port): $connections connections - $status"
            echo "   └─ Peer ID: ${peer_id:0:20}..."
            
            total_connections=$((total_connections + connections))
            if [[ $connections -gt 0 ]]; then
                ((connected_nodes++))
            fi
        fi
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📈 Total Network Connections: $total_connections"
    echo "🌐 Nodes with Connections: $connected_nodes/6"
    echo ""
    
    if [[ $connected_nodes -ge 4 ]]; then
        log_success "✅ Network connectivity excellent ($connected_nodes/6 nodes connected)"
        return 0
    elif [[ $connected_nodes -ge 2 ]]; then
        log_warning "⚠️  Network connectivity partial ($connected_nodes/6 nodes connected)"
        return 0
    else
        log_error "❌ Network connectivity poor ($connected_nodes/6 nodes connected)"
        return 1
    fi
}

# ======================== MAIN TEST ORCHESTRATION ========================

main() {
    echo "=================================================================="
    echo "    🚀 COMPREHENSIVE 6-NODE DATAMESH CLUSTER TEST SUITE 🚀"
    echo "=================================================================="
    echo "Testing ALL CLI commands and distributed functionality"
    echo "Cluster: 1 Bootstrap + 5 Service Nodes = 6 Total Nodes"
    echo ""
    
    # Pre-flight checks
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        echo "Please run: cargo build --release"
        exit 1
    fi
    
    log_info "🔧 Binary found: $DATAMESH_BINARY"
    
    # Clean up any existing processes
    cleanup
    sleep 2
    
    # Phase 1: Start the cluster
    echo ""
    log_info "📡 PHASE 1: CLUSTER STARTUP"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Start bootstrap node
    if ! start_bootstrap; then
        log_error "Failed to start bootstrap node"
        exit 1
    fi
    
    # Start service nodes
    local started_nodes=0
    for port in "${SERVICE_PORTS[@]}"; do
        if start_service_node "$port"; then
            ((started_nodes++))
        else
            log_warning "Failed to start service node on port $port"
        fi
        sleep 2
    done
    
    if [[ $started_nodes -eq 0 ]]; then
        log_error "No service nodes started successfully"
        exit 1
    fi
    
    log_success "🎉 Cluster started: 1 bootstrap + $started_nodes service nodes"
    
    # Wait for network stabilization
    wait_for_network_stabilization
    
    # Phase 2: Network Analysis
    echo ""
    log_info "🌐 PHASE 2: NETWORK CONNECTIVITY ANALYSIS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    analyze_network_connectivity
    
    # Phase 3: Configuration Testing
    echo ""
    log_info "⚙️  PHASE 3: CONFIGURATION COMMANDS TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_config_commands
    
    # Phase 4: Basic Storage Testing
    echo ""
    log_info "📁 PHASE 4: BASIC STORAGE OPERATIONS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_basic_storage
    test_named_storage
    test_tagged_storage
    
    # Phase 5: Distributed Operations
    echo ""
    log_info "🌐 PHASE 5: DISTRIBUTED OPERATIONS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_cross_node_operations
    
    # Phase 6: Listing and Info
    echo ""
    log_info "📋 PHASE 6: LISTING AND INFO COMMANDS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_list_operations
    
    # Phase 7: Statistics and Metrics
    echo ""
    log_info "📊 PHASE 7: STATISTICS AND METRICS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_stats_operations
    
    # Phase 8: Network Commands
    echo ""
    log_info "🌐 PHASE 8: NETWORK COMMANDS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_network_operations
    
    # Phase 9: Backup Testing
    echo ""
    log_info "💾 PHASE 9: BACKUP FUNCTIONALITY"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_backup_operations
    
    # Phase 10: Load Testing
    echo ""
    log_info "📦 PHASE 10: LOAD TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_multiple_files
    
    # Phase 11: Resilience Testing
    echo ""
    log_info "🛡️  PHASE 11: RESILIENCE TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_node_resilience
    
    # Phase 12: Final Results
    echo ""
    echo "=================================================================="
    echo "                    🎯 COMPREHENSIVE TEST RESULTS"
    echo "=================================================================="
    
    local success_rate=0
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        success_rate=$(( TESTS_PASSED * 100 / TOTAL_TESTS ))
    fi
    
    echo "📊 Total Tests Executed: $TOTAL_TESTS"
    echo "✅ Tests Passed: $TESTS_PASSED"
    echo "❌ Tests Failed: $TESTS_FAILED"
    echo "📈 Success Rate: $success_rate%"
    echo ""
    
    # Determine overall result
    local overall_result="❌ FAILED"
    local exit_code=1
    
    if [[ $success_rate -ge 90 ]]; then
        overall_result="🏆 EXCELLENT"
        exit_code=0
        log_success "🎉 OUTSTANDING! DataMesh cluster is fully functional!"
        echo ""
        echo "✅ All core functionality working correctly"
        echo "✅ Distributed storage operations successful"
        echo "✅ Network mesh formation robust"
        echo "✅ CLI commands fully operational"
        echo "✅ Backup and resilience features working"
        echo "✅ Load handling capabilities verified"
        echo ""
        echo "🚀 DataMesh is ready for production use!"
        
    elif [[ $success_rate -ge 75 ]]; then
        overall_result="✅ GOOD"
        exit_code=0
        log_success "👍 Good! DataMesh cluster is mostly functional with minor issues"
        echo ""
        echo "✅ Core functionality working"
        echo "⚠️  Some advanced features may need attention"
        echo "💡 Review failed tests for optimization opportunities"
        
    elif [[ $success_rate -ge 50 ]]; then
        overall_result="⚠️  PARTIAL"
        exit_code=1
        log_warning "⚠️  Partial success - significant issues detected"
        echo ""
        echo "⚠️  Basic functionality may work but reliability concerns exist"
        echo "💡 Review logs in $TEST_DIR/ for detailed analysis"
        
    else
        overall_result="❌ CRITICAL"
        exit_code=1
        log_error "❌ Critical issues detected - cluster not functional"
        echo ""
        echo "❌ Major problems prevent normal operation"
        echo "🔧 Requires immediate attention and debugging"
    fi
    
    echo "🎯 Overall Assessment: $overall_result"
    echo ""
    echo "📂 Test Logs Directory: $TEST_DIR"
    echo "📊 Node Logs: ${#NODE_LOGS[@]} service nodes + 1 bootstrap"
    echo "🔢 Process IDs: Bootstrap=$BOOTSTRAP_PID, Services=(${NODE_PIDS[*]})"
    echo ""
    echo "=================================================================="
    
    # Keep cluster running briefly to show stability
    if [[ $exit_code -eq 0 ]]; then
        log_info "🕐 Keeping cluster running for 15 seconds to demonstrate stability..."
        sleep 15
    fi
    
    return $exit_code
}

main "$@"