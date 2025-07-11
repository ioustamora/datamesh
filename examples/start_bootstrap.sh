#!/bin/bash
# start_bootstrap.sh - Start a DataMesh bootstrap node

set -e

# Configuration
PORT="${1:-40871}"
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/debug/datamesh}"

# Check if DataMesh binary exists
if [ ! -f "$DATAMESH_BINARY" ]; then
    echo "Error: DataMesh binary not found at $DATAMESH_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

echo "Starting DataMesh Bootstrap Node"
echo "=============================="
echo "Port: $PORT"
echo "Binary: $DATAMESH_BINARY"
echo ""
echo "Other nodes can connect with:"
echo "  --bootstrap-peer <PEER_ID_FROM_OUTPUT>"
echo "  --bootstrap-addr /ip4/<YOUR_IP>/tcp/$PORT"
echo ""

exec "$DATAMESH_BINARY" bootstrap --port "$PORT"
