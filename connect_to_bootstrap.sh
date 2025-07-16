#!/bin/bash
# connect_to_bootstrap.sh - Simple script to connect to a running bootstrap node

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/debug/datamesh}"
BOOTSTRAP_LOG="${BOOTSTRAP_LOG:-bootstrap_diag.log}"
DEFAULT_PORT="40871"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo
    echo "Connect to a DataMesh bootstrap node"
    echo
    echo "OPTIONS:"
    echo "  -p, --peer-id <PEER_ID>     Bootstrap peer ID (auto-detected if not provided)"
    echo "  -a, --address <ADDRESS>     Bootstrap address (default: /ip4/127.0.0.1/tcp/${DEFAULT_PORT})"
    echo "  -n, --network <NETWORK>     Use network preset (local, public, test)"
    echo "  -P, --port <PORT>           Local port to listen on (default: 0 = random)"
    echo "  -m, --mode <MODE>           Connection mode: interactive, service, or bootstrap"
    echo "  -h, --help                  Show this help message"
    echo
    echo "COMMANDS:"
    echo "  connect       Connect to bootstrap node (default)"
    echo "  interactive   Start interactive mode"
    echo "  service       Start service mode"
    echo "  bootstrap     Start a new bootstrap node"
    echo "  status        Show network status"
    echo "  discover      Discover available bootstrap nodes"
    echo
    echo "EXAMPLES:"
    echo "  $0                                          # Auto-connect to local bootstrap"
    echo "  $0 interactive                              # Start interactive mode"
    echo "  $0 --network local interactive              # Use local preset"
    echo "  $0 --peer-id 12D3KooW... --address /ip4/127.0.0.1/tcp/40871"
    echo "  $0 bootstrap --port 40872                   # Start bootstrap on port 40872"
    echo
}

# Function to auto-detect bootstrap peer from log
auto_detect_bootstrap() {
    if [[ -f "$BOOTSTRAP_LOG" ]]; then
        local peer_id=$(grep -o "Peer ID: [0-9A-Za-z]*" "$BOOTSTRAP_LOG" | head -1 | cut -d' ' -f3)
        local address=$(grep -o "Listening on /ip4/[0-9.]*\/(tcp|udp)\/[0-9]*" "$BOOTSTRAP_LOG" | head -1)
        
        if [[ -n "$peer_id" && -n "$address" ]]; then
            print_success "Auto-detected bootstrap peer:"
            print_info "  Peer ID: $peer_id"
            print_info "  Address: $address"
            echo "$peer_id|$address"
            return 0
        fi
    fi
    return 1
}

# Function to check if bootstrap is running
check_bootstrap_running() {
    if pgrep -f "datamesh.*bootstrap" > /dev/null; then
        print_success "Bootstrap node is running"
        return 0
    else
        print_warning "No bootstrap node detected"
        return 1
    fi
}

# Function to start bootstrap node
start_bootstrap() {
    local port=${1:-$DEFAULT_PORT}
    print_info "Starting bootstrap node on port $port"
    
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        print_error "DataMesh binary not found at $DATAMESH_BINARY"
        print_info "Build with: cargo build"
        exit 1
    fi
    
    print_info "Starting bootstrap node (logs: $BOOTSTRAP_LOG)"
    "$DATAMESH_BINARY" bootstrap --port "$port" 2>&1 | tee "$BOOTSTRAP_LOG" &
    local bootstrap_pid=$!
    
    print_success "Bootstrap node started with PID $bootstrap_pid"
    print_info "Waiting for bootstrap to be ready..."
    
    # Wait for bootstrap to be ready
    for i in {1..10}; do
        if check_bootstrap_running; then
            break
        fi
        sleep 1
    done
    
    if auto_detect_bootstrap > /dev/null; then
        print_success "Bootstrap node is ready!"
    else
        print_warning "Bootstrap node started but auto-detection failed"
    fi
}

# Function to connect to bootstrap
connect_to_bootstrap() {
    local peer_id="$1"
    local address="$2"
    local mode="$3"
    local port="$4"
    local network="$5"
    
    if [[ ! -f "$DATAMESH_BINARY" ]]; then
        print_error "DataMesh binary not found at $DATAMESH_BINARY"
        print_info "Build with: cargo build"
        exit 1
    fi
    
    # Build command
    local cmd="$DATAMESH_BINARY"
    
    # Add network preset if specified
    if [[ -n "$network" ]]; then
        cmd="$cmd --network $network"
    fi
    
    # Add bootstrap peer info if provided
    if [[ -n "$peer_id" ]]; then
        cmd="$cmd --bootstrap-peer $peer_id"
    fi
    
    if [[ -n "$address" ]]; then
        cmd="$cmd --bootstrap-addr $address"
    fi
    
    # Add port if specified
    if [[ -n "$port" && "$port" != "0" ]]; then
        cmd="$cmd --port $port"
    fi
    
    # Add mode
    case "$mode" in
        "interactive")
            cmd="$cmd interactive"
            ;;
        "service")
            cmd="$cmd service"
            ;;
        "bootstrap")
            cmd="$cmd bootstrap"
            ;;
        *)
            cmd="$cmd interactive"
            ;;
    esac
    
    print_info "Connecting to DataMesh network..."
    print_info "Command: $cmd"
    
    # Execute command
    exec $cmd
}

# Parse command line arguments
PEER_ID=""
ADDRESS=""
NETWORK=""
PORT="0"
MODE="interactive"
COMMAND="connect"

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--peer-id)
            PEER_ID="$2"
            shift 2
            ;;
        -a|--address)
            ADDRESS="$2"
            shift 2
            ;;
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -P|--port)
            PORT="$2"
            shift 2
            ;;
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        connect|interactive|service|bootstrap|status|discover)
            COMMAND="$1"
            if [[ "$COMMAND" == "interactive" || "$COMMAND" == "service" || "$COMMAND" == "bootstrap" ]]; then
                MODE="$COMMAND"
            fi
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main logic
print_info "DataMesh Connection Helper"
print_info "========================="

case "$COMMAND" in
    "bootstrap")
        start_bootstrap "$PORT"
        ;;
    "status")
        check_bootstrap_running
        if auto_detect_bootstrap > /dev/null; then
            auto_detect_bootstrap
        fi
        ;;
    "discover")
        print_info "Discovering bootstrap nodes..."
        check_bootstrap_running
        auto_detect_bootstrap || print_warning "No bootstrap nodes found"
        ;;
    "connect"|"interactive"|"service")
        # Auto-detect if not provided
        if [[ -z "$PEER_ID" && -z "$ADDRESS" && -z "$NETWORK" ]]; then
            print_info "Auto-detecting bootstrap peer..."
            if bootstrap_info=$(auto_detect_bootstrap); then
                PEER_ID=$(echo "$bootstrap_info" | cut -d'|' -f1)
                ADDRESS=$(echo "$bootstrap_info" | cut -d'|' -f2)
            else
                print_warning "Could not auto-detect bootstrap peer"
                print_info "Using local network preset"
                NETWORK="local"
            fi
        fi
        
        connect_to_bootstrap "$PEER_ID" "$ADDRESS" "$MODE" "$PORT" "$NETWORK"
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac