# DataMesh Usage Guide

## Overview
DataMesh is a comprehensive distributed data storage system with **51 commands** covering all aspects of distributed storage. Built with libp2p, Kademlia DHT, ECIES encryption, and Reed-Solomon erasure coding for secure, fault-tolerant, high-performance data operations.

## 🚀 Quick Command Reference

### Core Operations (5 commands)
- `put` - Store files with encryption and Reed-Solomon coding
- `get` - Retrieve files with automatic chunk reconstruction  
- `list` - List files with filtering by tags and metadata
- `info` - Show detailed file information and health status
- `stats` - Display storage statistics and usage metrics

### Network Management (9 commands)
- `bootstrap` - Run dedicated bootstrap node for network entry
- `interactive` - Interactive console mode with real-time operations
- `service` - Background service mode for continuous operation
- `peers` - List and manage connected peers with detailed info
- `network` - Analyze network topology and routing tables
- `discover` - Discover new peers with configurable timeouts
- `distribution` - Analyze file distribution across network
- `health` - Monitor network health and performance metrics
- `bandwidth` - Test network bandwidth and latency

### File Management (5 commands)
- `sync` - Synchronize directories with file watching capabilities
- `backup` - Create versioned backups with incremental support
- `restore` - Restore from backups with integrity verification
- `duplicate` - Create copies of existing files with new metadata
- `rename` - Rename files without re-uploading content

### Search & Discovery (3 commands)
- `search` - Advanced multi-criteria file search with regex support
- `recent` - Show recently uploaded/accessed files with filters
- `popular` - Display most frequently accessed files by timeframe

### Batch Operations (3 commands)
- `batch-put` - Upload multiple files with pattern matching and parallel processing
- `batch-get` - Download multiple files to local directory with structure preservation
- `batch-tag` - Bulk tag operations with dry-run support

### Health & Maintenance (3 commands)
- `repair` - Repair corrupted or low-redundancy files automatically
- `cleanup` - Clean up orphaned chunks, duplicates, and optimize storage
- `quota` - Manage storage quotas and usage monitoring

### Security & Audit (4 commands)
- `keys` - Manage cryptographic keys with rotation and backup capabilities
- `audit` - View security audit logs and compliance reports
- `security` - Run security diagnostics and vulnerability checks
- `transport` - Configure and monitor transport layer security settings

### Import/Export (2 commands)  
- `export` - Export files to standard archive formats (tar, zip) with encryption
- `import` - Import from archives with structure preservation and verification

### Quick Actions (3 commands)
- `pin` - Pin important files for guaranteed availability with priority levels
- `unpin` - Remove pins from files to allow normal garbage collection
- `share` - Generate secure sharing links with expiration and password protection

### Performance & Configuration (5 commands)
- `optimize` - Optimize storage performance with defragmentation and rebalancing
- `benchmark` - Run comprehensive performance benchmarks for network and storage
- `config` - Generate and manage configuration files with network presets
- `metrics` - Display real-time performance metrics with export capabilities
- `networks` - List available network presets and connection templates

## Installation & Building

```bash
# Clone the repository
git clone <repository-url>
cd dfs

# Build the project
cargo build --release

# The binary will be located at target/release/dfs
```

## Basic Usage

### Command Structure
```bash
datamesh [OPTIONS] <COMMAND>
```

### Available Commands

#### 1. **Store a File (`put`)**
```bash
datamesh put <FILE_PATH> [--public-key <KEY>]
```

**Example:**
```bash
# Store a single file
./target/release/datamesh put /path/to/myfile.txt

# Store an image
./target/release/datamesh put /home/user/photos/vacation.jpg

# Store with specific public key
./target/release/datamesh put myfile.txt --public-key 04a1b2c3...
```

**What happens:**
1. File is encrypted using ECIES
2. Encrypted data is split into 4 data chunks + 2 parity chunks (Reed-Solomon)
3. Each chunk is stored in the Kademlia DHT
4. Returns a unique file key for retrieval

**Output:**
```
Local peer id: PeerId("12D3KooW...")
File stored with key: b8a4b47c80e43b0f765881a42d2eef9c275fce3ff20187597251d1d53d6ccbc5
Listening on /ip4/127.0.0.1/tcp/40871
```

#### 2. **Retrieve a File (`get`)**
```bash
datamesh get <FILE_KEY> <OUTPUT_PATH> [--private-key <KEY_FILE>]
```

**Example:**
```bash
# Retrieve a file using its key
./target/release/datamesh get b8a4b47c80e43b0f765881a42d2eef9c275fce3ff20187597251d1d53d6ccbc5 ./recovered_file.txt

# Retrieve with specific private key
./target/release/datamesh get <FILE_KEY> ./recovered_file.txt --private-key my_key
```

**What happens:**
1. Retrieves file metadata from DHT using the key
2. Downloads required chunks (needs at least 4 out of 6 for reconstruction)
3. Reconstructs the original encrypted data using Reed-Solomon
4. Decrypts the data and saves to specified path

#### 3. **List Files (`list`)**
```bash
datamesh list [--public-key <KEY>]
```

**Example:**
```bash
# List files accessible with default key
./target/release/datamesh list

# List files for specific public key
./target/release/datamesh list --public-key 04a1b2c3...
```

#### 4. **Bootstrap Node (`bootstrap`)**
```bash
datamesh bootstrap [--port <PORT>]
```

**Example:**
```bash
# Start bootstrap node on default port (40871)
./target/release/datamesh bootstrap

# Start bootstrap node on custom port
./target/release/datamesh bootstrap --port 41000
```

#### 5. **Interactive Mode (`interactive`)**
```bash
datamesh interactive [--bootstrap-peer <PEER_ID>] [--bootstrap-addr <ADDR>] [--port <PORT>]
```

**Example:**
```bash
# Start interactive mode
./target/release/datamesh interactive

# Connect to existing network
./target/release/datamesh interactive --bootstrap-peer 12D3KooW... --bootstrap-addr /ip4/127.0.0.1/tcp/40871
```

#### 6. **Service Mode (`service`)**
```bash
datamesh service [--bootstrap-peer <PEER_ID>] [--bootstrap-addr <ADDR>] [--port <PORT>] [--timeout <SECONDS>]
```

**Example:**
```bash
# Start service mode
./target/release/datamesh service

# Connect to existing network with timeout
./target/release/datamesh service --bootstrap-peer 12D3KooW... --bootstrap-addr /ip4/127.0.0.1/tcp/40871 --timeout 300
```

#### 7. **Configuration (`config`)**
```bash
datamesh config [--generate] [--config-path <PATH>]
```

**Example:**
```bash
# Generate default config
./target/release/datamesh config --generate

# View current config
./target/release/datamesh config

# Generate config at specific path
./target/release/datamesh config --generate --config-path /etc/datamesh/config.toml
```

#### 8. **Performance Metrics (`metrics`)**
```bash
datamesh metrics [--summary] [--export]
```

**Example:**
```bash
# Show performance summary
./target/release/datamesh metrics --summary

# Export metrics as JSON
./target/release/datamesh metrics --export
```

### Global Options

#### Key Management
```bash
# Specify keys directory
datamesh --keys-dir /path/to/keys <COMMAND>

# Use specific key file
datamesh --key-name my_key <COMMAND>

# Non-interactive mode
datamesh --non-interactive <COMMAND>
```

#### Network Options
```bash
# Connect to bootstrap peer
datamesh --bootstrap-peer <PEER_ID> --bootstrap-addr <MULTIADDR> <COMMAND>

# Use specific port
datamesh --port <PORT> <COMMAND>
```

**Example:**
```bash
./target/release/datamesh \
  --bootstrap-peer 12D3KooWAtgLisEkS1Uc8mH7mZLFDRchZWCbzjvg38RTisBcpoG2 \
  --bootstrap-addr /ip4/192.168.1.100/tcp/40871 \
  --keys-dir ./my_keys \
  --key-name production_key \
  put myfile.txt
```

## Complete Workflow Example

### 1. Start Bootstrap Node
```bash
# Terminal 1 - Start bootstrap node
./target/release/datamesh bootstrap
# Note the peer ID and listening address from output
```

### 2. Start Service Nodes
```bash
# Terminal 2 - Start service node
./target/release/datamesh service \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871

# Terminal 3 - Start another service node
./target/release/datamesh service \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871
```

### 3. Store and Retrieve Files
```bash
# Terminal 4 - Store a file
./target/release/datamesh \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871 \
  put myfile.txt

# Retrieve the file
./target/release/datamesh \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871 \
  get <FILE_KEY> ./retrieved_file.txt
```

### 4. Interactive Mode
```bash
# Start interactive session
./target/release/datamesh interactive \
  --bootstrap-peer 12D3KooW... \
  --bootstrap-addr /ip4/127.0.0.1/tcp/40871

# Then use commands interactively:
# > put myfile.txt
# > get <FILE_KEY> ./retrieved_file.txt
# > list
# > quit
```

## Key Management System

### Understanding File Keys
- **256-bit BLAKE3 hash** of the original file content
- **Hexadecimal format** for easy copying/sharing
- **Unique identifier** for each file in the network

### Cryptographic Keys
DataMesh uses ECIES (Elliptic Curve Integrated Encryption Scheme) for encryption:

#### Key Generation
```bash
# Keys are automatically generated on first use
./target/release/datamesh put myfile.txt
# This creates keys in ~/.datamesh/keys/ directory

# Force non-interactive key generation
./target/release/datamesh --non-interactive put myfile.txt
```

#### Key Storage
```bash
# Default keys directory: ~/.datamesh/keys/
# Each key pair consists of:
# - <name>.key (private key)
# - <name>.info (key metadata)

# Use custom keys directory
./target/release/datamesh --keys-dir /secure/path/keys put myfile.txt

# Use specific key name
./target/release/datamesh --key-name production_key put myfile.txt
```

#### Key Backup
```bash
# Backup your keys directory
cp -r ~/.datamesh/keys /secure/backup/location

# List available keys
ls ~/.datamesh/keys/
```

### Saving File Keys
```bash
# Store keys for later retrieval
echo "my_document.pdf: a1b2c3d4..." >> file_keys.txt

# Or use a simple script
./target/release/datamesh put mydoc.pdf | grep "File stored with key:" >> keys.log
```

## Security Features

### Encryption
- **ECIES (Elliptic Curve Integrated Encryption Scheme)** for file content
- **Unique keys** generated for each file
- **End-to-end encryption** - only you can decrypt your files

### Fault Tolerance
- **Reed-Solomon erasure coding** (4+2 configuration)
- **Can lose up to 2 chunks** and still recover the file
- **Distributed storage** across multiple peers

### Privacy
- **Content-based addressing** - keys derived from file content
- **No metadata leakage** - filenames not stored in network
- **Peer anonymity** - no tracking of who stores what

## Troubleshooting

### Common Issues

#### 1. **Network Connection Problems**
```bash
# Check if port is available
netstat -an | grep :40871

# Try different port range (automatic selection)
./target/debug/datamesh put myfile.txt
```

#### 2. **File Not Found During Retrieval**
```bash
# Verify the file key is correct
# Check if enough peers are online (need at least 4 chunks)
# Wait a few seconds for DHT propagation
```

#### 3. **Large File Handling**
```bash
# For files > 100MB, consider splitting manually
split -b 50M largefile.dat chunk_
./target/debug/datamesh put chunk_aa
./target/debug/datamesh put chunk_ab
# Store the chunk keys together
```

## Performance Tips

### Optimal File Sizes
- **Best performance**: 1KB - 100MB files
- **Large files**: Consider chunking manually
- **Small files**: Overhead might be significant

### Network Performance
- **Local network**: Near-instant operations
- **Internet**: Depends on peer connectivity and bandwidth
- **Bootstrap peers**: Use reliable, well-connected nodes

## Advanced Usage

### Cluster Testing
DFS includes comprehensive cluster testing scripts in the `examples/` directory:

#### Run Full Cluster Test
```bash
# Run comprehensive cluster test
cd examples
./cluster_test.sh

# The test will:
# 1. Start a bootstrap node
# 2. Start 5 service nodes
# 3. Store multiple test files
# 4. Verify retrieval and integrity
# 5. Test cross-node availability
# 6. Test network resilience
```

#### Manual Cluster Setup
```bash
# Terminal 1: Start bootstrap node
./examples/start_bootstrap.sh

# Terminal 2-6: Start service nodes
./examples/start_node.sh

# Terminal 7: Run tests
./examples/simple_test.sh
```

### Configuration Management
```bash
# Generate configuration file
./target/release/datamesh config --generate

# View current configuration
./target/release/datamesh config

# Use custom configuration
./target/release/datamesh config --config-path /etc/datamesh/config.toml
```

### Performance Monitoring
```bash
# View performance metrics
./target/release/datamesh metrics --summary

# Export metrics to JSON
./target/release/datamesh metrics --export > metrics.json

# Monitor during operations
RUST_LOG=info ./target/release/datamesh put large_file.dat
```

### Scripting Integration
```bash
#!/bin/bash
# Backup script example
for file in /home/user/documents/*; do
    key=$(./target/release/datamesh put "$file" | grep "File stored with key:" | cut -d' ' -f5)
    echo "$(basename "$file"): $key" >> backup_keys.txt
done
```

### Monitoring
```bash
# Check peer connections
./target/release/datamesh put test.txt 2>&1 | grep -E "(Listening|Local peer)"

# Verify file storage
echo "test content" > test.txt
./target/release/datamesh put test.txt
# Use the returned key to verify retrieval
```

## Network Architecture

### Peer Discovery
1. **Bootstrap peers** provide initial network entry
2. **Kademlia DHT** maintains peer routing tables
3. **Automatic discovery** of additional peers

### Data Distribution
1. **Content addressing** using BLAKE3 hashes
2. **DHT storage** distributes chunks across network
3. **Replication** handled by Kademlia protocol

### Fault Tolerance
1. **Reed-Solomon coding** provides redundancy
2. **Multiple peers** store different chunks
3. **Automatic recovery** from partial failures

## Security Considerations

### What's Protected
- ✅ **File content** (encrypted)
- ✅ **Chunk integrity** (cryptographic hashes)
- ✅ **Network transport** (libp2p security)

### What's Not Protected
- ❌ **File existence** (keys are public)
- ❌ **File size** (visible to network)
- ❌ **Access patterns** (who retrieves what)

### Best Practices
1. **Keep file keys secure** - they're your only way to retrieve files
2. **Use trusted bootstrap peers** - they can see your network activity
3. **Consider file size implications** - very large files create more network traffic
4. **Backup file keys** - losing them means losing access to your files

## Getting Help

### Debug Mode
```bash
# Enable verbose logging
RUST_LOG=debug ./target/debug/datamesh put myfile.txt

# Check network status
RUST_LOG=libp2p=debug ./target/debug/datamesh put myfile.txt
```

### Common Commands Reference
```bash
# Quick help
./target/release/datamesh --help
./target/release/datamesh put --help
./target/release/datamesh get --help

# Subcommand help
./target/release/datamesh interactive --help
./target/release/datamesh service --help
./target/release/datamesh bootstrap --help
./target/release/datamesh config --help
./target/release/datamesh metrics --help
```
