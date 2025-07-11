#!/bin/bash
# cluster_manager.sh - Simple DFS cluster management tool

set -e

# Configuration
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/release/datamesh}"
BOOTSTRAP_PORT=${BOOTSTRAP_PORT:-40871}
NODE_PORTS=${NODE_PORTS:-"40872 40873 40874 40875 40876"}
CLUSTER_DIR="cluster_$(date +%Y%m%d_%H%M%S)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
info() { echo -e "${CYAN}ℹ${NC} $1"; }

# Usage function
usage() {
    echo "DataMesh Cluster Manager"
    echo "=================="
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  start       Start a new cluster"
    echo "  stop        Stop running cluster"
    echo "  status      Show cluster status"
    echo "  test        Run quick test"
    echo "  connect     Show connection info"
    echo "  logs        View cluster logs"
    echo "  clean       Clean up old cluster data"
    echo ""
    echo "Options:"
    echo "  --nodes N   Number of nodes to start (default: 5)"
    echo "  --port P    Bootstrap port (default: 40871)"
    echo "  --dir D     Cluster directory (default: auto-generated)"
    echo ""
    echo "Examples:"
    echo "  $0 start --nodes 7"
    echo "  $0 test"
    echo "  $0 stop"
    echo ""
}

# Parse command line arguments
COMMAND=""
NUM_NODES=5
CUSTOM_DIR=""

while [[ $# -gt 0 ]]; do
    case $1 in
        start|stop|status|test|connect|logs|clean)
            COMMAND="$1"
            shift
            ;;
        --nodes)
            NUM_NODES="$2"
            shift 2
            ;;
        --port)
            BOOTSTRAP_PORT="$2"
            shift 2
            ;;
        --dir)
            CUSTOM_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if [ -z "$COMMAND" ]; then
    usage
    exit 1
fi

# Set cluster directory
if [ -n "$CUSTOM_DIR" ]; then
    CLUSTER_DIR="$CUSTOM_DIR"
fi

LOG_DIR="$CLUSTER_DIR/logs"
PID_FILE="$CLUSTER_DIR/cluster.pids"
INFO_FILE="$CLUSTER_DIR/cluster.info"

# Check if DFS binary exists
check_binary() {
    if [ ! -f "$DATAMESH_BINARY" ]; then
        error "DataMesh binary not found at $DATAMESH_BINARY"
        echo "Build with: cargo build --release"
        exit 1
    fi
}

# Start cluster function
start_cluster() {
    check_binary
    
    if [ -f "$PID_FILE" ]; then
        warning "Cluster already appears to be running in $CLUSTER_DIR"
        echo "Use 'stop' command first or specify a different directory"
        exit 1
    fi
    
    log "Starting DFS cluster in $CLUSTER_DIR"
    mkdir -p "$LOG_DIR"
    
    # Generate node ports
    local node_ports=()
    for i in $(seq 1 $NUM_NODES); do
        node_ports+=($((BOOTSTRAP_PORT + i)))
    done
    
    # Start bootstrap node
    log "Starting bootstrap node on port $BOOTSTRAP_PORT..."
    "$DATAMESH_BINARY" --non-interactive bootstrap --port "$BOOTSTRAP_PORT" \
        > "$LOG_DIR/bootstrap.log" 2>&1 &
    local bootstrap_pid=$!
    echo "$bootstrap_pid" > "$PID_FILE"
    
    # Wait for bootstrap to start and extract info
    sleep 3
    local bootstrap_peer_id=""
    local bootstrap_addr=""
    
    for i in {1..10}; do
        if [ -f "$LOG_DIR/bootstrap.log" ]; then
            bootstrap_peer_id=$(grep -E "(Peer ID:|Local peer id:)" "$LOG_DIR/bootstrap.log" | head -1 | sed -E 's/.*(12D3[A-Za-z0-9]+).*/\1/' 2>/dev/null || true)
            bootstrap_addr=$(grep "Listening on.*$BOOTSTRAP_PORT" "$LOG_DIR/bootstrap.log" | head -1 | grep -o '/ip4/[^[:space:]]*' 2>/dev/null || true)
            
            if [ -n "$bootstrap_peer_id" ] && [ -n "$bootstrap_addr" ]; then
                break
            fi
        fi
        sleep 1
    done
    
    if [ -z "$bootstrap_peer_id" ] || [ -z "$bootstrap_addr" ]; then
        error "Failed to start bootstrap node"
        kill "$bootstrap_pid" 2>/dev/null || true
        rm -f "$PID_FILE"
        exit 1
    fi
    
    success "Bootstrap node started (PID: $bootstrap_pid)"
    
    # Save cluster info
    {
        echo "BOOTSTRAP_PID=$bootstrap_pid"
        echo "BOOTSTRAP_PORT=$BOOTSTRAP_PORT"
        echo "BOOTSTRAP_PEER_ID=$bootstrap_peer_id"
        echo "BOOTSTRAP_ADDR=$bootstrap_addr"
        echo "NODE_PORTS=(${node_ports[*]})"
        echo "CLUSTER_DIR=$CLUSTER_DIR"
        echo "STARTED=$(date)"
    } > "$INFO_FILE"
    
    # Start regular nodes
    log "Starting $NUM_NODES regular nodes..."
    local started_nodes=0
    
    for i in "${!node_ports[@]}"; do
        local port="${node_ports[$i]}"
        local node_num=$((i + 1))
        
        log "Starting node $node_num on port $port..."
        
        "$DATAMESH_BINARY" --non-interactive service \
            --bootstrap-peer "$bootstrap_peer_id" \
            --bootstrap-addr "$bootstrap_addr" \
            --port "$port" \
            > "$LOG_DIR/node_$node_num.log" 2>&1 &
        
        local node_pid=$!
        echo "$node_pid" >> "$PID_FILE"
        
        # Check if node started
        sleep 2
        if kill -0 "$node_pid" 2>/dev/null; then
            success "Node $node_num started (PID: $node_pid, Port: $port)"
            ((started_nodes++))
        else
            error "Node $node_num failed to start"
        fi
    done
    
    echo ""
    success "Cluster started successfully!"
    info "$started_nodes/$NUM_NODES nodes running"
    info "Bootstrap: $bootstrap_peer_id at $bootstrap_addr"
    echo ""
    echo "Connection information saved to: $INFO_FILE"
    echo "Logs available in: $LOG_DIR"
    echo ""
    echo "Use '$0 status' to check cluster health"
    echo "Use '$0 test' to run a quick functionality test"
}

# Stop cluster function
stop_cluster() {
    if [ ! -f "$PID_FILE" ]; then
        warning "No running cluster found"
        return
    fi
    
    log "Stopping cluster..."
    local stopped_count=0
    
    while read -r pid; do
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            log "Stopping process $pid..."
            kill "$pid" 2>/dev/null || true
            sleep 1
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
            ((stopped_count++))
        fi
    done < "$PID_FILE"
    
    rm -f "$PID_FILE"
    success "Stopped $stopped_count processes"
}

# Show cluster status
show_status() {
    if [ ! -f "$PID_FILE" ]; then
        warning "No running cluster found"
        return
    fi
    
    if [ ! -f "$INFO_FILE" ]; then
        warning "Cluster info not found"
        return
    fi
    
    # Load cluster info
    source "$INFO_FILE"
    
    echo "DataMesh Cluster Status"
    echo "=================="
    echo ""
    echo "Cluster Directory: $CLUSTER_DIR"
    echo "Started: $STARTED"
    echo "Bootstrap: $BOOTSTRAP_PEER_ID"
    echo "Bootstrap Address: $BOOTSTRAP_ADDR"
    echo ""
    
    local running_count=0
    local total_count=0
    
    echo "Process Status:"
    while read -r pid; do
        if [ -n "$pid" ]; then
            ((total_count++))
            if kill -0 "$pid" 2>/dev/null; then
                echo "  PID $pid: RUNNING"
                ((running_count++))
            else
                echo "  PID $pid: STOPPED"
            fi
        fi
    done < "$PID_FILE"
    
    echo ""
    local health_percentage=$((running_count * 100 / total_count))
    echo "Health: $running_count/$total_count processes running ($health_percentage%)"
    
    if [ $health_percentage -ge 80 ]; then
        success "Cluster is healthy"
    elif [ $health_percentage -ge 50 ]; then
        warning "Cluster has some issues"
    else
        error "Cluster is unhealthy"
    fi
}

# Run quick test
run_test() {
    if [ ! -f "$INFO_FILE" ]; then
        error "No cluster info found. Start cluster first."
        exit 1
    fi
    
    # Load cluster info
    source "$INFO_FILE"
    
    log "Running quick cluster test..."
    
    # Create test file
    local test_file="/tmp/dfs_test_$(date +%s).txt"
    echo "DataMesh test file created at $(date)" > "$test_file"
    echo "Random data: $RANDOM" >> "$test_file"
    
    # Store file
    log "Storing test file..."
    local output
    output=$("$DATAMESH_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$test_file" 2>&1) || true
    
    local key=$(echo "$output" | grep "File stored with key:" | cut -d' ' -f5 2>/dev/null || true)
    
    if [ -n "$key" ]; then
        success "File stored with key: ${key:0:32}..."
        
        # Retrieve file
        log "Retrieving file..."
        local retrieved_file="/tmp/dfs_retrieved_$(date +%s).txt"
        
        if "$DATAMESH_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            get "$key" "$retrieved_file" > /dev/null 2>&1; then
            
            # Verify content
            if cmp -s "$test_file" "$retrieved_file"; then
                success "File retrieved and verified successfully!"
                echo ""
                success "Cluster test PASSED"
            else
                error "Retrieved file content differs"
                echo ""
                error "Cluster test FAILED"
            fi
            
            # Cleanup
            rm -f "$retrieved_file"
        else
            error "Failed to retrieve file"
            echo ""
            error "Cluster test FAILED"
        fi
    else
        error "Failed to store file"
        echo "Output: $output"
        echo ""
        error "Cluster test FAILED"
    fi
    
    # Cleanup
    rm -f "$test_file"
}

# Show connection info
show_connection() {
    if [ ! -f "$INFO_FILE" ]; then
        error "No cluster info found. Start cluster first."
        exit 1
    fi
    
    # Load cluster info
    source "$INFO_FILE"
    
    echo "DataMesh Cluster Connection Information"
    echo "=================================="
    echo ""
    echo "To connect to this cluster, use:"
    echo ""
    echo "Bootstrap Peer ID: $BOOTSTRAP_PEER_ID"
    echo "Bootstrap Address: $BOOTSTRAP_ADDR"
    echo ""
    echo "Example commands:"
    echo ""
    echo "Store a file:"
    echo "  $DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID \\"
    echo "    --bootstrap-addr $BOOTSTRAP_ADDR put <file>"
    echo ""
    echo "List files:"
    echo "  $DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID \\"
    echo "    --bootstrap-addr $BOOTSTRAP_ADDR list"
    echo ""
    echo "Interactive mode:"
    echo "  $DATAMESH_BINARY --bootstrap-peer $BOOTSTRAP_PEER_ID \\"
    echo "    --bootstrap-addr $BOOTSTRAP_ADDR interactive"
    echo ""
    echo "Available node ports: ${NODE_PORTS[*]}"
}

# View logs
view_logs() {
    if [ ! -d "$LOG_DIR" ]; then
        error "No logs directory found"
        exit 1
    fi
    
    local log_files=($(ls "$LOG_DIR"/*.log 2>/dev/null || true))
    
    if [ ${#log_files[@]} -eq 0 ]; then
        warning "No log files found"
        return
    fi
    
    echo "Available log files:"
    for i in "${!log_files[@]}"; do
        printf "  %d) %s\n" $((i+1)) "$(basename "${log_files[$i]}")"
    done
    echo ""
    
    printf "Enter log number to view (1-${#log_files[@]}), or 'a' for all: "
    read -r choice
    
    if [ "$choice" = "a" ] || [ "$choice" = "A" ]; then
        for log_file in "${log_files[@]}"; do
            echo ""
            echo "=== $(basename "$log_file") ==="
            tail -20 "$log_file"
        done
    elif [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#log_files[@]}" ]; then
        local selected_log="${log_files[$((choice-1))]}"
        echo ""
        echo "=== $(basename "$selected_log") ==="
        tail -50 "$selected_log"
        echo ""
        printf "Press 'f' for full log or Enter to continue: "
        read -r view_choice
        if [ "$view_choice" = "f" ] || [ "$view_choice" = "F" ]; then
            cat "$selected_log"
        fi
    else
        warning "Invalid choice"
    fi
}

# Clean up old clusters
clean_clusters() {
    echo "Cleaning up old cluster directories..."
    
    local cluster_dirs=($(ls -d cluster_* 2>/dev/null || true))
    
    if [ ${#cluster_dirs[@]} -eq 0 ]; then
        info "No cluster directories found"
        return
    fi
    
    echo "Found cluster directories:"
    for dir in "${cluster_dirs[@]}"; do
        local size=$(du -sh "$dir" 2>/dev/null | cut -f1)
        echo "  $dir ($size)"
    done
    echo ""
    
    printf "Delete all old cluster directories? [y/N]: "
    read -r confirm
    
    if [[ "$confirm" =~ ^[Yy]([Ee][Ss])?$ ]]; then
        for dir in "${cluster_dirs[@]}"; do
            if [ "$dir" != "$CLUSTER_DIR" ]; then
                rm -rf "$dir"
                success "Removed $dir"
            fi
        done
    else
        info "Cleanup cancelled"
    fi
}

# Execute command
case "$COMMAND" in
    start)
        start_cluster
        ;;
    stop)
        stop_cluster
        ;;
    status)
        show_status
        ;;
    test)
        run_test
        ;;
    connect)
        show_connection
        ;;
    logs)
        view_logs
        ;;
    clean)
        clean_clusters
        ;;
    *)
        error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac