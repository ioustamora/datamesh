#!/bin/bash
# backup.sh - Simple backup script for DFS

set -e

# Configuration
BACKUP_DIR="${1:-$HOME/Documents}"
KEY_FILE="backup_keys_$(date +%Y%m%d_%H%M%S).txt"
DATAMESH_BINARY="${DATAMESH_BINARY:-./target/release/datamesh}"

# Check if DFS binary exists
if [ ! -f "$DATAMESH_BINARY" ]; then
    echo "Error: DFS binary not found at $DATAMESH_BINARY"
    echo "Build with: cargo build"
    exit 1
fi

echo "DataMesh Backup Script"
echo "================="
echo "Backing up files from: $BACKUP_DIR"
echo "Storing keys in: $KEY_FILE"
echo ""

# Create key file with header
cat > "$KEY_FILE" << EOF
# DataMesh Backup Keys - $(date)
# Format: filename:file_key
# Keep this file safe - you need these keys to retrieve your files!

EOF

# Counter for files processed
count=0

# Find and backup files
find "$BACKUP_DIR" -type f -size -100M | while read -r file; do
    echo "Backing up: $(basename "$file")"
    
    # Store file and capture key
    output=$("$DATAMESH_BINARY" put "$file" 2>&1)
    key=$(echo "$output" | grep "File stored with key:" | cut -d' ' -f5)
    
    if [ -n "$key" ]; then
        echo "$(basename "$file"):$key" >> "$KEY_FILE"
        count=$((count + 1))
        echo "  ✓ Stored with key: $key"
    else
        echo "  ✗ Failed to store file"
        echo "    Output: $output"
    fi
    
    echo ""
done

echo "Backup complete! Stored keys in: $KEY_FILE"
echo "To restore a file: $DATAMESH_BINARY get <key> <output_path>"
