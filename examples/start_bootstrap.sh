#!/bin/bash
# start_bootstrap.sh - Start a DFS bootstrap node

set -e

# Configuration
PORT="${1:-40871}"
DFS_BINARY="${DFS_BINARY:-./target/debug/dfs}"

# Check if DFS binary exists
if [ ! -f "$DFS_BINARY" ]; then
    echo "Error: DFS binary not found at $DFS_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

echo "Starting DFS Bootstrap Node"
echo "=========================="
echo "Port: $PORT"
echo "Binary: $DFS_BINARY"
echo ""
echo "Other nodes can connect with:"
echo "  --bootstrap-peer <PEER_ID_FROM_OUTPUT>"
echo "  --bootstrap-addr /ip4/<YOUR_IP>/tcp/$PORT"
echo ""

exec "$DFS_BINARY" bootstrap --port "$PORT"
