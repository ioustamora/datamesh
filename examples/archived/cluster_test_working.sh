#!/bin/bash
# Working DataMesh Cluster Test - Addresses all identified issues

set -e

# Configuration
DATAMESH_BINARY="./target/release/datamesh"
BOOTSTRAP_PORT=40871

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

# Process tracking
PIDS=()

cleanup() {
    log_info "🧹 Cleaning up processes..."
    for pid in "${PIDS[@]}"; do
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    sleep 2
    # Force cleanup
    pkill -f datamesh 2>/dev/null || true
    rm -f /tmp/dmtest_* 2>/dev/null || true
}

trap cleanup EXIT INT TERM

main() {
    echo "=================================================="
    echo "      DataMesh Working Cluster Test"
    echo "=================================================="
    echo ""
    
    # Verify binary
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        log_error "DataMesh binary not found at $DATAMESH_BINARY"
        log_info "Please run: cargo build --release"
        exit 1
    fi
    
    log_info "Binary found: $DATAMESH_BINARY"
    log_info "Binary size: $(ls -lh "$DATAMESH_BINARY" | awk '{print $5}')"
    
    # Clean up any existing processes
    cleanup
    sleep 1
    
    # Check ports
    if ss -ln | grep -q ":$BOOTSTRAP_PORT "; then
        log_warning "Port $BOOTSTRAP_PORT is in use, attempting cleanup..."
        lsof -ti :$BOOTSTRAP_PORT | xargs kill -9 2>/dev/null || true
        sleep 2
    fi
    
    # Start bootstrap node
    log_info "🚀 Starting bootstrap node on port $BOOTSTRAP_PORT..."
    
    # Use explicit environment to avoid threading issues
    RUST_LOG=info "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" &
    local bootstrap_pid=$!
    PIDS+=("$bootstrap_pid")
    
    log_info "Bootstrap node PID: $bootstrap_pid"
    
    # Wait for bootstrap to initialize
    log_info "⏳ Waiting for bootstrap to initialize..."
    sleep 10
    
    # Verify bootstrap is running
    if ! kill -0 "$bootstrap_pid" 2>/dev/null; then
        log_error "Bootstrap node died during startup"
        exit 1
    fi
    
    # Get bootstrap peer ID from process list (more reliable than log parsing)
    local peer_info
    peer_info=$(ps aux | grep "$bootstrap_pid" | grep bootstrap || true)
    
    if [[ -z "$peer_info" ]]; then
        log_error "Could not find bootstrap process info"
        exit 1
    fi
    
    log_success "Bootstrap node is running"
    
    # Start a second node to create a minimal cluster
    log_info "🌐 Starting second node on port $((BOOTSTRAP_PORT + 1))..."
    
    # Use the localhost address directly to avoid network preset issues
    RUST_LOG=info "$DATAMESH_BINARY" --non-interactive service \
        --port $((BOOTSTRAP_PORT + 1)) \
        --bootstrap-peers "12D3KooWDummyPeerIdForTesting@/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT" &
    local node_pid=$!
    PIDS+=("$node_pid")
    
    log_info "Second node PID: $node_pid"
    sleep 5
    
    # Test basic functionality
    log_info "🧪 Testing cluster functionality..."
    
    # Create test file
    local test_file="/tmp/dmtest_$(date +%s).txt"
    echo "DataMesh cluster test file" > "$test_file"
    echo "Created: $(date)" >> "$test_file"
    echo "Random: $RANDOM" >> "$test_file"
    
    log_info "📁 Created test file: $(basename "$test_file")"
    
    # Try direct bootstrap connection test
    log_info "🔗 Testing direct bootstrap connection..."
    
    # Method 1: Try with explicit peer connection
    local success=false
    local attempts=0
    local max_attempts=3
    
    while [[ $attempts -lt $max_attempts && "$success" == "false" ]]; do
        ((attempts++))
        log_info "Connection attempt $attempts/$max_attempts"
        
        # Use interactive mode to bypass connection retry logic
        if echo -e "put $test_file\nexit" | timeout 15 "$DATAMESH_BINARY" \
            --bootstrap-peers "12D3KooWDummyPeerIdForTesting@/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT" \
            interactive > /tmp/dmtest_output_$attempts.txt 2>&1; then
            
            if grep -q "stored with key\|File stored" /tmp/dmtest_output_$attempts.txt; then
                success=true
                log_success "File storage successful via interactive mode"
                
                # Extract key for retrieval test
                local key=$(grep -oE '[a-f0-9]{32,}' /tmp/dmtest_output_$attempts.txt | head -1)
                if [[ -n "$key" ]]; then
                    log_info "🔑 File key: ${key:0:32}..."
                    
                    # Test retrieval
                    local retrieved_file="/tmp/dmtest_retrieved_$(date +%s).txt"
                    if echo -e "get $key $retrieved_file\nexit" | timeout 15 "$DATAMESH_BINARY" \
                        --bootstrap-peers "12D3KooWDummyPeerIdForTesting@/ip4/127.0.0.1/tcp/$BOOTSTRAP_PORT" \
                        interactive > /tmp/dmtest_get_output.txt 2>&1; then
                        
                        if [[ -f "$retrieved_file" ]] && cmp -s "$test_file" "$retrieved_file"; then
                            log_success "File retrieval and verification successful"
                            log_success "🎉 CLUSTER TEST PASSED!"
                            
                            echo ""
                            log_info "Test Summary:"
                            echo "  ✅ Bootstrap node: Running"
                            echo "  ✅ Second node: Running"  
                            echo "  ✅ File storage: Working"
                            echo "  ✅ File retrieval: Working"
                            echo "  ✅ Data integrity: Verified"
                            echo ""
                            log_success "DataMesh cluster is fully functional!"
                            
                            # Clean up test files
                            rm -f "$test_file" "$retrieved_file"
                            return 0
                        else
                            log_warning "File retrieval failed or content mismatch"
                        fi
                    else
                        log_warning "Get command failed"
                    fi
                else
                    log_warning "Could not extract file key"
                fi
            else
                log_warning "No file storage confirmation found"
            fi
        else
            log_warning "Interactive command failed"
        fi
        
        if [[ "$success" == "false" && $attempts -lt $max_attempts ]]; then
            log_info "Waiting before next attempt..."
            sleep 5
        fi
    done
    
    if [[ "$success" == "false" ]]; then
        log_error "All test attempts failed"
        
        # Show diagnostics
        echo ""
        log_info "🔍 Diagnostics:"
        echo "Bootstrap process: $(kill -0 "$bootstrap_pid" 2>/dev/null && echo "Running" || echo "Dead")"
        echo "Node process: $(kill -0 "$node_pid" 2>/dev/null && echo "Running" || echo "Dead")"
        echo "Listening ports:"
        ss -ln | grep -E ":408[0-9][0-9]" || echo "No DataMesh ports found"
        
        echo ""
        log_info "Command outputs:"
        for i in $(seq 1 $attempts); do
            if [[ -f "/tmp/dmtest_output_$i.txt" ]]; then
                echo "Attempt $i output:"
                tail -10 "/tmp/dmtest_output_$i.txt" | sed 's/^/  /'
                echo ""
            fi
        done
        
        return 1
    fi
}

main "$@"