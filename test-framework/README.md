# DataMesh Universal Multinode Testing Framework

## Overview

This comprehensive testing framework provides automated testing capabilities for all DataMesh functionality in realistic multinode environments. It supports testing CLI commands, API endpoints, WebSocket communications, storage economy operations, governance features, and UI interactions across multiple interconnected nodes.

## Architecture

### 1. Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Test Orchestrator                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Node      │  │  Network    │  │   Test      │         │
│  │ Management  │  │ Simulation  │  │ Execution   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │    CLI      │  │     API     │  │     UI      │         │
│  │  Testing    │  │  Testing    │  │  Testing    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Monitoring  │  │ Validation  │  │ Reporting   │         │
│  │   System    │  │   Engine    │  │   System    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### 2. Key Features

- **Automatic Node Orchestration**: Deploy and manage 3-10 node clusters
- **Network Simulation**: Partition tolerance, latency injection, bandwidth limits
- **Comprehensive CLI Testing**: All 40+ commands with parameter combinations
- **API Integration Testing**: REST endpoints, WebSocket connections, authentication
- **UI Automation**: End-to-end browser testing with real user scenarios
- **Storage Economy Testing**: Multi-node verification, tier upgrades, challenges
- **Governance Testing**: Proposal creation, voting, consensus mechanisms
- **Performance Benchmarking**: Load testing, scalability analysis
- **Fault Injection**: Node failures, network partitions, data corruption
- **Real-time Monitoring**: Network health, storage verification, performance metrics

## Test Categories

### 1. Network Formation & Discovery Tests
- Bootstrap node connectivity across different configurations
- DHT routing table population and peer discovery
- Multi-bootstrap failover scenarios
- Network partition recovery
- Cross-region connectivity simulation

### 2. File Storage & Retrieval Tests
- Reed-Solomon shard distribution validation
- Quorum-based storage success verification
- Multi-node file retrieval consistency
- Large file chunking and reassembly
- Encryption/decryption across nodes
- Storage economy tier verification

### 3. CLI Command Integration Tests
- All file operations (put, get, list, info, stats)
- Network commands (peers, health, discover, bootstrap)
- Batch operations (batch-put, batch-get, batch-tag)
- Service operations (interactive, service, bootstrap)
- Economy commands (quota, economy, tiers)
- Governance commands (proposals, voting, operators)

### 4. API & WebSocket Tests
- Authentication flows across multiple nodes
- File upload/download through API
- Real-time WebSocket updates
- Storage economy API operations
- Governance API functionality
- Error handling and rate limiting

### 5. UI End-to-End Tests
- File manager operations
- Dashboard functionality
- Economy management interface
- Governance voting interface
- Administration panels
- Mobile interface testing
- PWA functionality

### 6. Storage Economy Tests
- Multi-node storage verification cycles
- Tier upgrade workflows
- Challenge-response mechanisms
- Reputation score calculations
- Economic transaction validation
- Quota enforcement

### 7. Governance System Tests
- Proposal creation and distribution
- Multi-node voting processes
- Consensus mechanism validation
- Operator management
- Permission enforcement
- Network health monitoring

### 8. Performance & Scalability Tests
- Concurrent file operations
- Network bandwidth utilization
- Storage optimization algorithms
- Load balancing across nodes
- Memory and CPU usage monitoring
- Database performance under load

### 9. Fault Tolerance Tests
- Node failure and recovery
- Network partition scenarios
- Bootstrap node failures
- Data corruption handling
- Storage verification failures
- Consensus failure recovery

### 10. Security & Cryptography Tests
- Key management across nodes
- ECIES encryption/decryption
- Authentication and authorization
- Secure transport validation
- WebSocket security
- API security headers

## Usage

### Quick Start
```bash
# Run basic 3-node cluster test
./scripts/run-tests.sh basic

# Run comprehensive test suite with 5 nodes
./scripts/run-tests.sh comprehensive --nodes 5

# Run UI-only tests
./scripts/run-tests.sh ui-only

# Run performance benchmarks with 10 nodes
./scripts/run-tests.sh performance-only --nodes 10
```

### Advanced Usage
```bash
# Test with fault tolerance and network simulation
./scripts/run-tests.sh fault-tolerance --network-sim --nodes 7

# Sequential testing for debugging
./scripts/run-tests.sh comprehensive --sequential --timeout 3600

# Custom WebDriver for UI tests
./scripts/run-tests.sh ui-only --webdriver-url http://selenium:4444

# Environment-based custom configuration
DATAMESH_TEST_NODES=8 DATAMESH_RUN_FAULT_TESTS=true ./scripts/run-tests.sh custom
```

### Framework Integration
```bash
# Build test framework
cargo build --release --bin test-orchestrator

# Run with library presets
cargo run --bin test-orchestrator -- --preset standard_cluster --scenario comprehensive

# Rust library usage
use datamesh_test_framework::presets::*;
use datamesh_test_framework::scenarios::*;

let config = standard_cluster();
let test_suite = comprehensive();
let orchestrator = TestOrchestrator::new(config).await?;
```

## Configuration

### Environment Variables

#### Core Configuration
- `DATAMESH_TEST_NODES` - Number of nodes (default: 5)
- `DATAMESH_TEST_TIMEOUT` - Test timeout in seconds (default: 1800)
- `DATAMESH_PARALLEL_EXECUTION` - Parallel execution (default: true)
- `DATAMESH_ENABLE_MONITORING` - Enable monitoring (default: true)

#### Test Categories
- `DATAMESH_RUN_NETWORK_TESTS` - Run network tests (default: true)
- `DATAMESH_RUN_CLI_TESTS` - Run CLI tests (default: true)
- `DATAMESH_RUN_API_TESTS` - Run API tests (default: true)
- `DATAMESH_RUN_UI_TESTS` - Run UI tests (default: true)
- `DATAMESH_RUN_ECONOMY_TESTS` - Run economy tests (default: true)
- `DATAMESH_RUN_GOVERNANCE_TESTS` - Run governance tests (default: true)
- `DATAMESH_RUN_PERFORMANCE_TESTS` - Run performance tests (default: true)
- `DATAMESH_RUN_FAULT_TESTS` - Run fault tolerance tests (default: false)

#### Advanced Features
- `DATAMESH_TEST_NETWORK_SIM` - Enable network simulation (default: false)
- `DATAMESH_WEBDRIVER_URL` - WebDriver URL for UI tests (default: http://localhost:4444)
- `DATAMESH_ENABLE_UI_TESTS` - Enable UI testing (default: true)

### TOML Configuration

```toml
[orchestrator]
node_count = 5
base_port = 40000
test_timeout = 1800
work_dir = "/tmp/datamesh-test"
enable_network_simulation = false
enable_monitoring = true
parallel_execution = true

[test_suite]
include_network_tests = true
include_cli_tests = true
include_api_tests = true
include_ui_tests = true
include_economy_tests = true
include_governance_tests = true
include_performance_tests = true
include_fault_tests = false

[test_data]
file_size_range = [1024, 10485760]  # 1KB to 10MB
file_count = 100
user_count = 20
proposal_count = 10

[ui_testing]
webdriver_url = "http://localhost:4444"
browser_timeout = 30
page_load_timeout = 10

[network_simulation]
enable_latency = true
enable_packet_loss = false
enable_bandwidth_limit = false
enable_partitions = false

[monitoring]
metrics_interval = 5
collect_system_metrics = true
collect_network_metrics = true
collect_application_metrics = true
```

## Test Scenarios

### Available Presets

- **`basic_cluster()`** - 3 nodes, basic functionality testing (10 minutes)
- **`standard_cluster()`** - 5 nodes, comprehensive testing (30 minutes)
- **`performance_cluster()`** - 10 nodes, performance benchmarks (1 hour)
- **`fault_tolerance_cluster()`** - 7 nodes, network simulation (40 minutes)
- **`ui_testing_cluster()`** - 3 nodes, UI-focused testing (20 minutes)

### Available Test Scenarios

- **`basic_functionality()`** - Core network, CLI, and API tests
- **`comprehensive()`** - All test categories enabled
- **`ui_focused()`** - Web interface and user experience testing
- **`performance_focused()`** - Performance and scalability testing
- **`fault_tolerance()`** - Network partition and failure recovery
- **`economy_focused()`** - Storage economy and verification testing
- **`governance_focused()`** - Governance and consensus testing

## Prerequisites

### System Requirements
- Minimum 4GB RAM
- 10GB available disk space in `/tmp`
- DataMesh binary built and accessible
- Rust toolchain for framework compilation

### UI Testing Requirements
```bash
# Install Selenium WebDriver (for UI tests)
docker run -d -p 4444:4444 selenium/standalone-chrome

# Or use local ChromeDriver
wget https://chromedriver.storage.googleapis.com/latest/chromedriver_linux64.zip
unzip chromedriver_linux64.zip
sudo mv chromedriver /usr/local/bin/
```

### Building DataMesh
```bash
# Build DataMesh before running tests
cd /path/to/datamesh
cargo build --release

# Ensure binary is in PATH or set explicitly
export PATH="$PWD/target/release:$PATH"
```

## Integration

### CI/CD Integration
```yaml
# GitHub Actions example
name: DataMesh Multinode Tests
on: [push, pull_request]
jobs:
  multinode-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
    - name: Setup Selenium
      run: docker run -d -p 4444:4444 selenium/standalone-chrome
    - name: Build DataMesh
      run: cargo build --release
    - name: Run Basic Tests
      run: cd test-framework && ./scripts/run-tests.sh basic
    - name: Run Comprehensive Tests
      run: cd test-framework && ./scripts/run-tests.sh comprehensive --no-ui
```

### Performance Monitoring
```bash
# Enable detailed monitoring
DATAMESH_ENABLE_MONITORING=true ./scripts/run-tests.sh performance-only

# View monitoring data
cat test-results/test-*/monitoring.log
```

## Output and Reporting

### Test Results Structure
```
test-results/
└── test-20241214-143022/
    ├── test-report.html          # Main HTML report
    ├── SUMMARY.txt              # Text summary
    ├── test-config.toml         # Test configuration used
    ├── monitoring.log           # System monitoring data
    └── cluster/                 # Node data and logs
        ├── node-0/
        ├── node-1/
        └── ...
```

### HTML Report Features
- Overall test results and status
- Test configuration summary
- Individual test case results
- Performance metrics and charts
- Error details and stack traces
- System resource usage graphs

## Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Check for port usage
netstat -ln | grep :4000

# Kill conflicting processes
pkill -f datamesh
```

#### WebDriver Connection Issues
```bash
# Test WebDriver connectivity
curl http://localhost:4444/status

# Restart Selenium
docker restart $(docker ps -q --filter ancestor=selenium/standalone-chrome)
```

#### Insufficient Resources
```bash
# Check available memory
free -h

# Check disk space
df -h /tmp

# Reduce node count for resource-constrained systems
DATAMESH_TEST_NODES=3 ./scripts/run-tests.sh basic
```

## Contributing

To extend the testing framework:

1. Add new test cases to appropriate modules (`src/cli_tests.rs`, `src/ui_tests.rs`, etc.)
2. Create new test scenarios in `src/lib.rs`
3. Update configuration options in `src/config.rs`
4. Add new presets for specific testing needs
5. Update documentation and examples

### Development Workflow
```bash
# Run framework tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy

# Build all binaries
cargo build --release --bins
```