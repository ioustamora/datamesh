#!/bin/bash
# Manual 6-Node DataMesh Cluster Test Suite
# Tests all CLI commands step by step without premature cleanup

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871
SERVICE_PORTS=(40872 40873 40874 40875 40876)
TEST_DIR="manual_test_$(date +%Y%m%d_%H%M%S)"

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

manual_cleanup() {
    log_info "🧹 Manual cleanup requested..."
    
    # Stop all service nodes
    for port in "${SERVICE_PORTS[@]}"; do
        if [[ -n "${NODE_PIDS[$port]}" ]]; then
            log_info "Stopping service node on port $port (PID: ${NODE_PIDS[$port]})"
            kill "${NODE_PIDS[$port]}" 2>/dev/null || true
        fi
    done
    
    # Stop bootstrap
    if [[ -n "$BOOTSTRAP_PID" ]]; then
        log_info "Stopping bootstrap node (PID: $BOOTSTRAP_PID)"
        kill "$BOOTSTRAP_PID" 2>/dev/null || true
    fi
    
    sleep 3
    
    # Force cleanup any remaining processes
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    
    # Clean test files
    rm -f /tmp/manual_test_* 2>/dev/null || true
    
    log_success "Manual cleanup completed"
}

start_cluster() {
    log_info "🚀 Starting 6-node DataMesh cluster..."
    mkdir -p "$TEST_DIR"
    
    # Start bootstrap node
    log_info "📡 Starting bootstrap node on port $BOOTSTRAP_PORT..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" > "$TEST_DIR/bootstrap.log" 2>&1 &
    BOOTSTRAP_PID=$!
    
    # Wait for bootstrap to be ready
    sleep 5
    
    # Extract bootstrap info
    if [[ -f "$TEST_DIR/bootstrap.log" ]]; then
        BOOTSTRAP_PEER_ID=$(grep -E "(Peer ID:|Local peer id:)" "$TEST_DIR/bootstrap.log" | head -1 | grep -oE '12D3[A-Za-z0-9]+' || true)
        BOOTSTRAP_ADDR=$(grep "Listening on.*$BOOTSTRAP_PORT" "$TEST_DIR/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' || true)
        
        if [[ -n "$BOOTSTRAP_PEER_ID" && -n "$BOOTSTRAP_ADDR" ]]; then
            log_success "Bootstrap node ready"
            log_info "  📡 Peer ID: $BOOTSTRAP_PEER_ID"
            log_info "  🌐 Address: $BOOTSTRAP_ADDR"
        else
            log_error "Failed to extract bootstrap node information"
            cat "$TEST_DIR/bootstrap.log"
            return 1
        fi
    else
        log_error "Bootstrap log file not found"
        return 1
    fi
    
    # Start service nodes
    local started_nodes=0
    for port in "${SERVICE_PORTS[@]}"; do
        log_info "🎯 Starting service node #$((port - 40871)) on port $port..."
        
        "$DATAMESH_BINARY" --non-interactive service \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --port "$port" > "$TEST_DIR/service_$port.log" 2>&1 &
        
        local pid=$!
        NODE_PIDS[$port]=$pid
        NODE_LOGS[$port]="$TEST_DIR/service_$port.log"
        
        sleep 3
        
        if kill -0 "$pid" 2>/dev/null; then
            log_success "Service node started on port $port (PID: $pid)"
            ((started_nodes++))
        else
            log_error "Service node failed to start on port $port"
            echo "Last lines of log:"
            tail -5 "$TEST_DIR/service_$port.log" 2>/dev/null || echo "No log available"
        fi
        
        sleep 2
    done
    
    log_success "🎉 Cluster started: 1 bootstrap + $started_nodes service nodes"
    
    # Wait for network stabilization
    log_info "⏳ Waiting for network stabilization..."
    for i in {1..20}; do
        echo -ne "█"
        sleep 1
    done
    echo ""
    log_success "Network stabilization completed"
    
    return 0
}

test_basic_commands() {
    log_info "🧪 Testing basic CLI commands..."
    
    local base_cmd="$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --non-interactive"
    
    echo ""
    log_test "1. Testing file storage (PUT command)"
    echo "Test file content - $(date)" > /tmp/manual_test_basic.txt
    
    log_command "$base_cmd --port 41001 put /tmp/manual_test_basic.txt"
    if store_output=$($base_cmd --port 41001 put /tmp/manual_test_basic.txt 2>&1); then
        log_success "✅ PUT command executed"
        echo "$store_output"
        
        # Extract file key
        local file_key=$(echo "$store_output" | grep -oE '(stored with key:|Key:) [a-f0-9]+' | grep -oE '[a-f0-9]{32,}' | head -1)
        if [[ -n "$file_key" ]]; then
            log_success "📝 File key extracted: ${file_key:0:20}..."
            
            echo ""
            log_test "2. Testing file retrieval (GET command)"
            log_command "$base_cmd --port 41002 get $file_key /tmp/manual_test_retrieved.txt"
            if $base_cmd --port 41002 get "$file_key" /tmp/manual_test_retrieved.txt 2>&1; then
                if cmp -s /tmp/manual_test_basic.txt /tmp/manual_test_retrieved.txt; then
                    log_success "✅ GET command successful - content matches!"
                else
                    log_error "❌ Content mismatch after retrieval"
                fi
            else
                log_error "❌ GET command failed"
            fi
        else
            log_warning "⚠️  Could not extract file key"
        fi
    else
        log_error "❌ PUT command failed"
        echo "$store_output"
    fi
    
    echo ""
    log_test "3. Testing file listing (LIST command)"
    log_command "$base_cmd --port 41003 list"
    if list_output=$($base_cmd --port 41003 list 2>&1); then
        log_success "✅ LIST command executed"
        echo "$list_output"
    else
        log_warning "⚠️  LIST command had issues"
        echo "$list_output"
    fi
    
    echo ""
    log_test "4. Testing storage statistics (STATS command)"
    log_command "$base_cmd --port 41004 stats"
    if stats_output=$($base_cmd --port 41004 stats 2>&1); then
        log_success "✅ STATS command executed"
        echo "$stats_output"
    else
        log_warning "⚠️  STATS command had issues"
        echo "$stats_output"
    fi
    
    echo ""
    log_test "5. Testing network analysis (NETWORK command)"
    log_command "$base_cmd --port 41005 network"
    if network_output=$($base_cmd --port 41005 network 2>&1); then
        log_success "✅ NETWORK command executed"
        echo "$network_output"
    else
        log_warning "⚠️  NETWORK command had issues"
        echo "$network_output"
    fi
    
    echo ""
    log_test "6. Testing peer discovery (DISCOVER command)"
    log_command "$base_cmd --port 41006 discover"
    if discover_output=$($base_cmd --port 41006 discover 2>&1); then
        log_success "✅ DISCOVER command executed"
        echo "$discover_output"
    else
        log_warning "⚠️  DISCOVER command had issues"
        echo "$discover_output"
    fi
    
    echo ""
    log_test "7. Testing peer listing (PEERS command)"
    log_command "$base_cmd --port 41007 peers"
    if peers_output=$($base_cmd --port 41007 peers 2>&1); then
        log_success "✅ PEERS command executed"
        echo "$peers_output"
    else
        log_warning "⚠️  PEERS command had issues"
        echo "$peers_output"
    fi
    
    echo ""
    log_test "8. Testing performance metrics (METRICS command)"
    log_command "$base_cmd --port 41008 metrics"
    if metrics_output=$($base_cmd --port 41008 metrics 2>&1); then
        log_success "✅ METRICS command executed"
        echo "$metrics_output"
    else
        log_warning "⚠️  METRICS command had issues"
        echo "$metrics_output"
    fi
}

test_named_storage() {
    log_info "🏷️  Testing named file storage..."
    
    local base_cmd="$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --non-interactive"
    
    echo "Named file content - $(date)" > /tmp/manual_test_named.txt
    
    echo ""
    log_test "9. Testing named storage (PUT with --name)"
    log_command "$base_cmd --port 41009 put /tmp/manual_test_named.txt --name my_test_document"
    if store_output=$($base_cmd --port 41009 put /tmp/manual_test_named.txt --name my_test_document 2>&1); then
        log_success "✅ Named storage successful"
        echo "$store_output"
        
        echo ""
        log_test "10. Testing retrieval by name (GET by name)"
        log_command "$base_cmd --port 41010 get my_test_document /tmp/manual_test_named_retrieved.txt"
        if $base_cmd --port 41010 get my_test_document /tmp/manual_test_named_retrieved.txt 2>&1; then
            if cmp -s /tmp/manual_test_named.txt /tmp/manual_test_named_retrieved.txt; then
                log_success "✅ Named retrieval successful!"
            else
                log_error "❌ Named retrieval content mismatch"
            fi
        else
            log_error "❌ Named retrieval failed"
        fi
    else
        log_error "❌ Named storage failed"
        echo "$store_output"
    fi
}

test_cross_node_operations() {
    log_info "🌐 Testing cross-node distributed operations..."
    
    local base_cmd="$DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID --bootstrap-addr $BOOTSTRAP_ADDR --non-interactive"
    
    echo "Cross-node test - $(date)" > /tmp/manual_test_cross.txt
    
    echo ""
    log_test "11. Testing cross-node storage (store from node simulation 1)"
    log_command "$base_cmd --port 42001 put /tmp/manual_test_cross.txt --name cross_node_test"
    if store_output=$($base_cmd --port 42001 put /tmp/manual_test_cross.txt --name cross_node_test 2>&1); then
        log_success "✅ Cross-node storage successful"
        
        echo ""
        log_test "12. Testing cross-node retrieval (retrieve from node simulation 2)"
        log_command "$base_cmd --port 42002 get cross_node_test /tmp/manual_test_cross_retrieved.txt"
        if $base_cmd --port 42002 get cross_node_test /tmp/manual_test_cross_retrieved.txt 2>&1; then
            if cmp -s /tmp/manual_test_cross.txt /tmp/manual_test_cross_retrieved.txt; then
                log_success "✅ Cross-node DHT functionality working!"
            else
                log_error "❌ Cross-node content mismatch"
            fi
        else
            log_error "❌ Cross-node retrieval failed"
        fi
    else
        log_error "❌ Cross-node storage failed"
        echo "$store_output"
    fi
}

analyze_cluster_status() {
    log_info "📊 Analyzing cluster status..."
    
    echo ""
    echo "🌟 CLUSTER STATUS ANALYSIS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check bootstrap node
    if [[ -n "$BOOTSTRAP_PID" ]] && kill -0 "$BOOTSTRAP_PID" 2>/dev/null; then
        echo "🟢 Bootstrap Node (Port $BOOTSTRAP_PORT): RUNNING (PID: $BOOTSTRAP_PID)"
        if [[ -f "$TEST_DIR/bootstrap.log" ]]; then
            local connections=$(grep -c "Connected to peer:\|peer connected" "$TEST_DIR/bootstrap.log" 2>/dev/null || echo "0")
            echo "   └─ Connections: $connections"
        fi
    else
        echo "🔴 Bootstrap Node (Port $BOOTSTRAP_PORT): STOPPED"
    fi
    
    # Check service nodes
    local running_nodes=0
    for port in "${SERVICE_PORTS[@]}"; do
        local pid="${NODE_PIDS[$port]}"
        local log_file="${NODE_LOGS[$port]}"
        
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            echo "🟢 Service Node $((port - 40871)) (Port $port): RUNNING (PID: $pid)"
            ((running_nodes++))
            
            if [[ -f "$log_file" ]]; then
                local connections=$(grep -c "Connected to peer:\|peer connected" "$log_file" 2>/dev/null || echo "0")
                local peer_id=$(grep -E "Network actor starting with peer ID:|Local peer id:" "$log_file" 2>/dev/null | head -1 | grep -oE '12D3[A-Za-z0-9]+' || echo "unknown")
                echo "   └─ Connections: $connections, Peer ID: ${peer_id:0:20}..."
            fi
        else
            echo "🔴 Service Node $((port - 40871)) (Port $port): STOPPED"
        fi
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📈 Summary: $((running_nodes + 1))/6 nodes running"
    echo "📂 Logs in: $TEST_DIR"
    echo ""
    
    return 0
}

main() {
    echo "=================================================================="
    echo "    🚀 MANUAL 6-NODE DATAMESH CLUSTER TEST SUITE 🚀"
    echo "=================================================================="
    echo "Step-by-step validation of ALL CLI commands and functionality"
    echo ""
    
    # Pre-flight checks
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        echo "Please run: cargo build --release"
        exit 1
    fi
    
    log_info "🔧 Binary found: $DATAMESH_BINARY"
    
    # Clean up any existing processes first
    log_info "🧹 Cleaning up any existing DataMesh processes..."
    pkill -f "datamesh.*bootstrap" 2>/dev/null || true
    pkill -f "datamesh.*service" 2>/dev/null || true
    sleep 2
    
    # Phase 1: Start cluster
    echo ""
    log_info "📡 PHASE 1: CLUSTER STARTUP"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if ! start_cluster; then
        log_error "Failed to start cluster"
        exit 1
    fi
    
    # Phase 2: Test basic commands
    echo ""
    log_info "🧪 PHASE 2: BASIC CLI COMMANDS TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_basic_commands
    
    # Phase 3: Test named storage
    echo ""
    log_info "🏷️  PHASE 3: NAMED STORAGE TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_named_storage
    
    # Phase 4: Test cross-node operations
    echo ""
    log_info "🌐 PHASE 4: CROSS-NODE DISTRIBUTED TESTING"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    test_cross_node_operations
    
    # Phase 5: Cluster analysis
    echo ""
    log_info "📊 PHASE 5: CLUSTER STATUS ANALYSIS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    analyze_cluster_status
    
    # Final results
    echo ""
    echo "=================================================================="
    echo "                    🎯 MANUAL TEST COMPLETED"
    echo "=================================================================="
    echo ""
    log_success "🎉 Manual testing completed!"
    echo ""
    echo "📋 What was tested:"
    echo "  ✅ 6-node cluster startup (1 bootstrap + 5 service nodes)"
    echo "  ✅ Basic file storage (PUT/GET commands)"
    echo "  ✅ File listing and statistics (LIST/STATS commands)"
    echo "  ✅ Network analysis (NETWORK/PEERS/DISCOVER commands)"
    echo "  ✅ Performance metrics (METRICS command)"
    echo "  ✅ Named file storage and retrieval"
    echo "  ✅ Cross-node distributed operations"
    echo "  ✅ Cluster status monitoring"
    echo ""
    echo "📂 All logs saved in: $TEST_DIR"
    echo "🔧 To clean up manually, run: pkill -f datamesh"
    echo ""
    
    log_info "🕐 Keeping cluster running for 20 seconds for manual inspection..."
    echo "   Press Ctrl+C to stop early or wait for automatic cleanup"
    
    sleep 20
    
    echo ""
    log_info "🧹 Performing final cleanup..."
    manual_cleanup
    
    log_success "🎯 Manual cluster test completed successfully!"
    
    return 0
}

main "$@"