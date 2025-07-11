# DataMesh Examples

This directory contains practical examples and scripts for using the DataMesh system.

# DataMesh Examples

This directory contains practical examples and scripts for using the DataMesh system.

## Scripts

### 📦 backup.sh
Automatically backup files from a directory to the DataMesh network.

```bash
# Backup all files in Documents folder
./examples/backup.sh ~/Documents

# Backup specific directory
./examples/backup.sh /path/to/important/files
```

### 📁 restore.sh
Restore files from DataMesh using a backup key file.

```bash
# Restore all files from backup
./examples/restore.sh backup_keys_20250703_142030.txt

# Restore to specific directory
./examples/restore.sh backup_keys.txt ./my_restored_files
```

### 🚀 cluster_test.sh
**NEW!** Comprehensive cluster testing script that:
- Starts 1 bootstrap node + 5 regular nodes
- Tests file storage with encryption
- Tests cross-node file retrieval with decryption  
- Tests network resilience and fault tolerance
- Provides detailed test results and logs

```bash
# Run full cluster test
./examples/cluster_test.sh
```

### 🌐 start_bootstrap.sh
**NEW!** Start a DataMesh bootstrap node for other nodes to connect to.

```bash
# Start on default port (40871)
./examples/start_bootstrap.sh

# Start on custom port
./examples/start_bootstrap.sh 40880
```

### 🔗 start_node.sh
**NEW!** Start a DataMesh node and connect it to a bootstrap node.

```bash
# Connect to bootstrap node
./examples/start_node.sh 12D3KooW... /ip4/127.0.0.1/tcp/40871 40872
```

## Usage Examples

### Basic File Operations
```bash
# Store a document
./target/debug/dfs put important_document.pdf
# Output: File stored with key: a1b2c3d4...

# Retrieve it later
./target/debug/dfs get a1b2c3d4... ./recovered_document.pdf
```

### Network Setup
```bash
# Terminal 1: Start first node
./target/debug/dfs put file1.txt
# Note the peer ID and address

# Terminal 2: Connect to first node
./target/debug/dfs \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871 \
  put file2.txt
```

### Batch Operations
```bash
# Store multiple files and save keys
for file in *.txt; do
    key=$(./target/debug/dfs put "$file" | grep "key:" | cut -d' ' -f5)
    echo "$file:$key" >> my_files.keys
done

# Restore all files
while IFS=: read -r filename key; do
    ./target/debug/dfs get "$key" "./restored_$filename"
done < my_files.keys
```

### Testing Network Resilience
```bash
# Store a file with redundancy
./target/debug/dfs put test_file.txt
# Key: abc123...

# Kill some peers (network simulation)
# File should still be retrievable with remaining peers
./target/debug/dfs get abc123... recovered_test.txt
```

## Configuration Examples

### Environment Variables
```bash
# Use custom DataMesh binary location
export DataMesh_BINARY="/usr/local/bin/dfs"
./examples/backup.sh

# Enable debug logging
export RUST_LOG=debug
./target/debug/dfs put myfile.txt
```

### Bootstrap Configuration
```bash
# Connect to known bootstrap peer
export BOOTSTRAP_PEER="12D3KooWAtgLisEkS1Uc8mH7mZLFDRchZWCbzjvg38RTisBcpoG2"
export BOOTSTRAP_ADDR="/ip4/192.168.1.100/tcp/40871"

./target/debug/dfs \
  --bootstrap-peer "$BOOTSTRAP_PEER" \
  --bootstrap-addr "$BOOTSTRAP_ADDR" \
  put myfile.txt
```

## Best Practices

### Key Management
```bash
# Always backup your key files
cp backup_keys.txt backup_keys_backup.txt

# Use descriptive names for key files
key_file="backup_$(hostname)_$(date +%Y%m%d).txt"
```

### File Size Considerations
```bash
# Check file size before storing
if [ $(stat -f%z "$file" 2>/dev/null || stat -c%s "$file") -gt 104857600 ]; then
    echo "File too large (>100MB), consider splitting"
fi
```

### Network Health
```bash
# Test connectivity before large operations
echo "test" | ./target/debug/dfs put /dev/stdin > /dev/null
if [ $? -eq 0 ]; then
    echo "Network healthy, proceeding with backup"
else
    echo "Network issues detected"
fi
```

## Troubleshooting Examples

### Common Issues
```bash
# Check if binary is built
if [ ! -f "./target/debug/dfs" ]; then
    cargo build
fi

# Verify file exists before storing
if [ ! -f "$filename" ]; then
    echo "File not found: $filename"
    exit 1
fi

# Test with small file first
echo "test content" > test.txt
./target/debug/dfs put test.txt
```

### Debug Mode
```bash
# Enable verbose logging
RUST_LOG=debug ./target/debug/dfs put myfile.txt 2>&1 | tee debug.log

# Check network activity
RUST_LOG=libp2p=info ./target/debug/dfs put myfile.txt
```
