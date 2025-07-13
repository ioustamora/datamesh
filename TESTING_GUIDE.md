# DataMesh Cluster Testing Guide

## Overview
This guide explains the streamlined testing approach for DataMesh after cleaning up redundant test scripts.

## Current Test Structure

### 🟢 Primary Test Suite
- **`examples/perfect_cluster_test.sh`** - The comprehensive cluster test
  - **1200+ lines** of comprehensive testing code
  - Tests **30+ CLI commands** with full coverage
  - **7 nodes + bootstrap** for robust network testing
  - **Interactive dashboard** for real-time monitoring
  - **Professional UX** with progress indicators and colored output

### 🟡 Supporting Tests
- **`examples/simple_test.sh`** - Quick validation test
  - Basic put/get operations
  - Good for rapid functionality checks

## What Was Removed

### Redundant Test Scripts (Backed up to `backup_tests/`)
- `examples/archived/` - 13 duplicate/outdated cluster test scripts
- `test_dht_diagnostic.sh` - Superseded by perfect_cluster_test.sh
- `test_simple_2nodes.sh` - Limited to 2 nodes
- `test_storage_standalone.sh` - Single-node only

### Temporary Test Directories
- All `*_test_*` directories from previous test runs

## CLI Commands Tested

The `perfect_cluster_test.sh` covers all implemented CLI commands:

### Core Operations
- `put` - File storage with various options
- `get` - File retrieval with integrity verification
- `list` - File listing with filtering
- `info` - Detailed file information

### Network & Monitoring
- `peers` - Peer connection status
- `stats` - Storage statistics
- `metrics` - Performance metrics
- `health` - Network health monitoring
- `network` - Network topology analysis
- `discover` - Peer discovery
- `bandwidth` - Bandwidth testing
- `distribution` - File distribution analysis

### Advanced Features
- `search` - Content search functionality
- `recent` - Recently accessed files
- `popular` - Popular files (stub command)
- `batch-put` - Batch file upload
- `batch-get` - Batch file download
- `batch-tag` - Bulk tagging operations
- `sync` - Directory synchronization
- `duplicate` - File duplication
- `pin/unpin` - File pinning for availability
- `share` - File sharing links

### Management & Maintenance
- `repair` - File repair operations
- `cleanup` - Storage cleanup
- `optimize` - Performance optimization
- `quota` - Storage quota management
- `backup/restore` - Backup and restore operations
- `export/import` - Data export/import
- `benchmark` - Performance benchmarking

### System & Config
- `config` - Configuration management
- `networks` - Network presets
- `advanced` - Advanced system features
- `api-health/api-status` - API server monitoring
- `pricing` - Economics calculations

## Running Tests

### Quick Test
```bash
cd /home/user/dev/datamesh
./examples/simple_test.sh
```

### Comprehensive Test
```bash
cd /home/user/dev/datamesh
./examples/perfect_cluster_test.sh
```

### Interactive Mode
The perfect cluster test includes an interactive dashboard:
```bash
./examples/perfect_cluster_test.sh
# Follow the prompts for interactive monitoring
```

## Test Features

### Perfect Cluster Test Capabilities
- **Multi-node cluster**: 7 service nodes + 1 bootstrap node
- **Fault tolerance testing**: Node failure injection
- **Performance benchmarking**: Real-world performance metrics
- **Network analysis**: Topology mapping and health monitoring
- **Comprehensive logging**: JSON metrics and detailed logs
- **Professional UX**: Progress bars, colored output, symbols
- **Interactive dashboard**: Real-time cluster management

### Test Output
All test results are saved to timestamped directories:
- `perfect_cluster_YYYYMMDD_HHMMSS/`
  - `logs/` - Node logs
  - `results/` - Test results and metrics
  - `monitoring/` - Real-time monitoring data
  - `data/` - Test files and artifacts

## Development Workflow

1. **During Development**: Use `simple_test.sh` for quick validation
2. **Before Commits**: Run `perfect_cluster_test.sh` for comprehensive testing
3. **Network Debugging**: Use the interactive dashboard features
4. **Performance Analysis**: Check the JSON metrics output

## Maintenance

### Adding New Commands
When adding new CLI commands:
1. Add test cases to `perfect_cluster_test.sh` in the `test_new_functionality()` function
2. Follow the existing pattern for timeout, error handling, and logging
3. Update this documentation

### Backup Location
All removed test files are backed up in `backup_tests/` directory for reference.

## Best Practices

- Always run tests after making changes to CLI commands
- Use the interactive dashboard for debugging network issues
- Review the JSON metrics for performance insights
- Keep test files small and focused on specific functionality
- Use descriptive test names and comprehensive logging

This streamlined approach ensures thorough testing while maintaining a clean, maintainable codebase.
