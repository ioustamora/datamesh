# DataMesh Startup Guide

## Quick Start

The startup wizard issue has been fixed. You can now connect to bootstrap nodes directly without the wizard interfering.

### 1. Start a Bootstrap Node

```bash
# Start bootstrap on default port (40871)
./target/debug/datamesh bootstrap

# Start bootstrap on custom port
./target/debug/datamesh bootstrap --port 33000
```

### 2. Connect to Bootstrap Node

#### Using the Connection Helper (Recommended)

```bash
# Auto-detect and connect to local bootstrap
./connect_to_bootstrap.sh

# Start interactive mode
./connect_to_bootstrap.sh interactive

# Start service mode
./connect_to_bootstrap.sh service

# Use network preset
./connect_to_bootstrap.sh --network local interactive
```

#### Using Direct Commands

```bash
# Connect to bootstrap with peer ID and address
./target/debug/datamesh interactive \
    --bootstrap-peer 12D3KooWQ6QGTdm2tyuTfMEzKevcy9d4zY7zyHLtfJGHDTs2n3YF \
    --bootstrap-addr /ip4/127.0.0.1/tcp/40871

# Use network preset
./target/debug/datamesh interactive --network local

# Start service mode
./target/debug/datamesh service \
    --bootstrap-peer 12D3KooWQ6QGTdm2tyuTfMEzKevcy9d4zY7zyHLtfJGHDTs2n3YF \
    --bootstrap-addr /ip4/127.0.0.1/tcp/40871
```

## What Was Fixed

### 1. Startup Wizard Interference
- **Problem**: Wizard launched automatically preventing direct connections
- **Solution**: Modified `src/main.rs` to only launch wizard when no network arguments are provided
- **Function**: `has_network_connection_args()` checks for network parameters

### 2. Bootstrap Manager Integration
- **Problem**: Service commands didn't use advanced bootstrap features
- **Solution**: Integrated `BootstrapManager` with retry logic and health monitoring
- **Files**: `src/commands/service_commands.rs`

### 3. Interactive Mode Improvements
- **Problem**: Interactive mode had warnings about incomplete integration
- **Solution**: Simplified interactive commands to work directly with swarm
- **Features**: Real-time network event display, improved command handling

### 4. Network Presets
- **Problem**: No easy way to connect to common network configurations
- **Solution**: Added local network preset support
- **Config**: `config/local_network.toml` with default bootstrap configurations

### 5. Connection Helper Script
- **Problem**: Complex command-line arguments required for connections
- **Solution**: Created `connect_to_bootstrap.sh` with auto-detection
- **Features**: Auto-detect bootstrap peers, simplified connection commands

## Network Presets

### Local Network
The local network preset (`--network local`) now uses configuration from `config/local_network.toml`:

```toml
# Common bootstrap peer for local development
[[bootstrap.peers]]
peer_id = "12D3KooWQ6QGTdm2tyuTfMEzKevcy9d4zY7zyHLtfJGHDTs2n3YF"
addresses = [
    "/ip4/127.0.0.1/tcp/40871",
    "/ip4/127.0.0.1/tcp/33000"
]
```

### Available Presets
- `local`: Local development network with configured bootstrap peers
- `public`: Public DataMesh network (placeholder)
- `test`: Testing network with single bootstrap

## Interactive Mode Commands

Once connected, use these commands in interactive mode:

```
datamesh> help          # Show available commands
datamesh> status        # Show network status
datamesh> peers         # List connected peers
datamesh> health        # Check network health
datamesh> stats         # Show network statistics
datamesh> network       # Show network topology
datamesh> put <file>    # Store file (shows usage info)
datamesh> get <key>     # Retrieve file (shows usage info)
datamesh> list          # List files (shows usage info)
datamesh> exit          # Exit interactive mode
```

## Troubleshooting

### Bootstrap Node Not Starting
```bash
# Check if port is in use
lsof -i :40871

# Use different port
./target/debug/datamesh bootstrap --port 40872
```

### Connection Fails
```bash
# Check bootstrap is running
./connect_to_bootstrap.sh status

# Try auto-detection
./connect_to_bootstrap.sh discover

# Use network preset
./connect_to_bootstrap.sh --network local
```

### Build Issues
```bash
# Clean build
cargo clean
cargo build

# Check for compilation errors
cargo check
```

## Example Workflow

1. **Start Bootstrap Node**:
   ```bash
   ./target/debug/datamesh bootstrap --port 40871
   ```

2. **Connect in Another Terminal**:
   ```bash
   ./connect_to_bootstrap.sh interactive
   ```

3. **Use Interactive Commands**:
   ```
   datamesh> status
   datamesh> peers  
   datamesh> health
   datamesh> help
   ```

The startup wizard will no longer interfere with direct connections, and the bootstrap manager provides robust connection handling with automatic retries and health monitoring.