#!/bin/bash
# start_node.sh - Start a DFS node and connect to bootstrap

set -e

# Configuration
BOOTSTRAP_PEER="${1}"
BOOTSTRAP_ADDR="${2}"
PORT="${3:-0}"
DFS_BINARY="${DFS_BINARY:-./target/debug/dfs}"

if [ -z "$BOOTSTRAP_PEER" ] || [ -z "$BOOTSTRAP_ADDR" ]; then
    echo "Usage: $0 <bootstrap_peer_id> <bootstrap_addr> [port]"
    echo ""
    echo "Example:"
    echo "  $0 12D3KooW... /ip4/127.0.0.1/tcp/40871 40872"
    echo ""
    echo "To start without connecting to bootstrap:"
    echo "  $DFS_BINARY interactive"
    exit 1
fi

# Check if DFS binary exists
if [ ! -f "$DFS_BINARY" ]; then
    echo "Error: DFS binary not found at $DFS_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

echo "Starting DFS Node"
echo "================"
echo "Bootstrap Peer: $BOOTSTRAP_PEER"
echo "Bootstrap Addr: $BOOTSTRAP_ADDR"
echo "Local Port: $PORT"
echo "Binary: $DFS_BINARY"
echo ""

if [ "$PORT" != "0" ]; then
    exec "$DFS_BINARY" interactive \
        --bootstrap-peer "$BOOTSTRAP_PEER" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --port "$PORT"
else
    exec "$DFS_BINARY" interactive \
        --bootstrap-peer "$BOOTSTRAP_PEER" \
        --bootstrap-addr "$BOOTSTRAP_ADDR"
fi
