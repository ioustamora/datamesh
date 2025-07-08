#!/bin/bash
# restore.sh - Simple restore script for DFS

set -e

# Configuration
KEY_FILE="${1:-backup_keys.txt}"
RESTORE_DIR="${2:-./restored_files}"
DFS_BINARY="${DFS_BINARY:-./target/debug/dfs}"

# Check if DFS binary exists
if [ ! -f "$DFS_BINARY" ]; then
    echo "Error: DFS binary not found at $DFS_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

# Check if key file exists
if [ ! -f "$KEY_FILE" ]; then
    echo "Error: Key file not found: $KEY_FILE"
    echo "Usage: $0 <key_file> [restore_directory]"
    exit 1
fi

echo "DataMesh Restore Script"
echo "=================="
echo "Restoring from key file: $KEY_FILE"
echo "Restore directory: $RESTORE_DIR"
echo ""

# Create restore directory
mkdir -p "$RESTORE_DIR"

# Counter for files processed
count=0
success=0

# Read key file and restore files
grep -v "^#" "$KEY_FILE" | grep ":" | while IFS=: read -r filename key; do
    if [ -n "$filename" ] && [ -n "$key" ]; then
        echo "Restoring: $filename"
        output_path="$RESTORE_DIR/$filename"
        
        # Restore file
        if "$DFS_BINARY" get "$key" "$output_path" 2>/dev/null; then
            echo "  ✓ Restored to: $output_path"
            success=$((success + 1))
        else
            echo "  ✗ Failed to restore (key: $key)"
        fi
        
        count=$((count + 1))
        echo ""
    fi
done

echo "Restore complete!"
echo "Files processed: $count"
echo "Check $RESTORE_DIR for restored files"
