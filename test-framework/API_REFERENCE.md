# DataMesh Testing Framework API Reference

## Core Types and Structures

### TestOrchestrator

The main orchestrator for managing multinode test execution.

```rust
pub struct TestOrchestrator {
    config: OrchestratorConfig,
    node_manager: Arc<NodeManager>,
    network_simulator: Option<Arc<NetworkSimulator>>,
    test_executor: Arc<TestExecutor>,
    monitoring_system: Option<Arc<MonitoringSystem>>,
    validation_engine: Arc<ValidationEngine>,
    active_nodes: Arc<RwLock<HashMap<String, NodeInstance>>>,
    test_results: Arc<Mutex<Vec<TestResult>>>,
    start_time: Option<Instant>,
}
```

#### Methods

##### `new(config: OrchestratorConfig) -> Result<Self>`
Creates a new test orchestrator with the specified configuration.

##### `deploy_cluster() -> Result<()>`
Deploys the configured number of DataMesh nodes and establishes network connectivity.

##### `run_test_suite(test_suite: TestSuite) -> Result<Vec<TestResult>>`
Executes the specified test suite across all deployed nodes.

##### `generate_report() -> Result<TestReport>`
Generates a comprehensive test report with results, metrics, and analysis.

##### `teardown() -> Result<()>`
Gracefully shuts down all nodes and cleans up resources.

### OrchestratorConfig

Configuration for the test orchestrator.

```rust
pub struct OrchestratorConfig {
    pub node_count: usize,
    pub base_port: u16,
    pub test_timeout: Duration,
    pub work_dir: PathBuf,
    pub enable_network_simulation: bool,
    pub enable_monitoring: bool,
    pub parallel_execution: bool,
    pub test_data_config: TestDataConfig,
    pub topology: ClusterTopology,
}
```

#### Fields

- `node_count`: Number of nodes to deploy (3-10)
- `base_port`: Starting port for node allocation (default: 40000)
- `test_timeout`: Maximum time for test execution
- `work_dir`: Directory for test files and logs
- `enable_network_simulation`: Enable fault injection
- `enable_monitoring`: Enable system monitoring
- `parallel_execution`: Run tests in parallel vs sequential
- `test_data_config`: Configuration for test data generation
- `topology`: Network topology (Star, Ring, Mesh)

### TestSuite

Defines which test categories to include.

```rust
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub include_network_tests: bool,
    pub include_cli_tests: bool,
    pub include_api_tests: bool,
    pub include_ui_tests: bool,
    pub include_economy_tests: bool,
    pub include_governance_tests: bool,
    pub include_performance_tests: bool,
    pub include_fault_tests: bool,
}
```

### TestResult

Individual test case result.

```rust
pub struct TestResult {
    pub test_case: TestCase,
    pub passed: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### TestCase

Test case definition.

```rust
pub struct TestCase {
    pub id: Uuid,
    pub name: String,
    pub category: TestCategory,
    pub description: String,
}
```

### TestCategory

Test category enumeration.

```rust
pub enum TestCategory {
    Network,
    CLI,
    API,
    UI,
    Economy,
    Governance,
    Performance,
    Fault,
}
```

## Node Management

### NodeManager

Manages DataMesh node lifecycle.

```rust
pub struct NodeManager {
    base_work_dir: PathBuf,
    base_port: u16,
    nodes: Arc<RwLock<HashMap<String, NodeInstance>>>,
}
```

#### Methods

##### `new(base_work_dir: PathBuf, base_port: u16) -> Self`
Creates a new node manager.

##### `deploy_nodes(count: usize, topology: ClusterTopology) -> Result<Vec<NodeInstance>>`
Deploys the specified number of nodes with the given topology.

##### `start_node(config: NodeConfig) -> Result<NodeInstance>`
Starts a single DataMesh node with the provided configuration.

##### `stop_node(node_id: &str) -> Result<()>`
Gracefully stops a specific node.

##### `stop_all_nodes() -> Result<()>`
Stops all managed nodes.

##### `get_node_status(node_id: &str) -> Result<NodeStatus>`
Retrieves the current status of a specific node.

### NodeInstance

Represents a running DataMesh node.

```rust
pub struct NodeInstance {
    pub node_id: String,
    pub port: u16,
    pub api_port: u16,
    pub work_dir: PathBuf,
    pub config_dir: PathBuf,
    pub process: Option<Child>,
    pub status: NodeStatus,
    pub start_time: Instant,
}
```

### NodeConfig

Configuration for a DataMesh node.

```rust
pub struct NodeConfig {
    pub node_id: String,
    pub port: u16,
    pub api_port: u16,
    pub work_dir: PathBuf,
    pub config_dir: PathBuf,
    pub bootstrap_peers: Vec<String>,
    pub network_config: NetworkConfig,
}
```

## Test Executors

### CliTestExecutor

Executes CLI command tests.

```rust
pub struct CliTestExecutor {
    nodes: Arc<RwLock<HashMap<String, NodeInstance>>>,
    test_data_generator: Arc<TestDataGenerator>,
    base_work_dir: PathBuf,
}
```

#### Methods

##### `new(nodes: Arc<RwLock<HashMap<String, NodeInstance>>>, base_work_dir: PathBuf) -> Self`
Creates a new CLI test executor.

##### `run_all_cli_tests() -> Result<Vec<TestResult>>`
Executes comprehensive CLI test suite.

##### `test_file_operations() -> Result<Vec<TestResult>>`
Tests file upload, download, and management commands.

##### `test_network_commands() -> Result<Vec<TestResult>>`
Tests network connectivity and peer management commands.

##### `test_batch_operations() -> Result<Vec<TestResult>>`
Tests batch file operations.

### ApiTestExecutor

Executes API endpoint tests.

```rust
pub struct ApiTestExecutor {
    nodes: Arc<RwLock<HashMap<String, NodeInstance>>>,
    http_client: reqwest::Client,
    websocket_client: tokio_tungstenite::WebSocketStream,
}
```

#### Methods

##### `new(nodes: Arc<RwLock<HashMap<String, NodeInstance>>>) -> Result<Self>`
Creates a new API test executor.

##### `run_all_api_tests() -> Result<Vec<TestResult>>`
Executes comprehensive API test suite.

##### `test_file_endpoints() -> Result<Vec<TestResult>>`
Tests file upload/download API endpoints.

##### `test_websocket_connections() -> Result<Vec<TestResult>>`
Tests WebSocket connectivity and real-time updates.

### UiTestExecutor

Executes UI automation tests.

```rust
pub struct UiTestExecutor {
    nodes: Arc<RwLock<HashMap<String, NodeInstance>>>,
    test_data_generator: Arc<TestDataGenerator>,
    browser_pool: BrowserPool,
    base_url_template: String,
}
```

#### Methods

##### `new(nodes: Arc<RwLock<HashMap<String, NodeInstance>>>, webdriver_url: String) -> Result<Self>`
Creates a new UI test executor.

##### `run_all_ui_tests() -> Result<Vec<TestResult>>`
Executes comprehensive UI test suite.

##### `test_file_manager_interface() -> Result<Vec<TestResult>>`
Tests file management UI functionality.

##### `test_dashboard_functionality() -> Result<Vec<TestResult>>`
Tests dashboard and metrics display.

## Network Simulation

### NetworkSimulator

Simulates network conditions and faults.

```rust
pub struct NetworkSimulator {
    latency_config: LatencyConfig,
    packet_loss_config: PacketLossConfig,
    bandwidth_config: BandwidthConfig,
    partition_config: PartitionConfig,
}
```

#### Methods

##### `new(config: NetworkSimulationConfig) -> Self`
Creates a new network simulator.

##### `apply_latency(nodes: &[NodeInstance], latency_ms: u64) -> Result<()>`
Applies network latency to communication between nodes.

##### `simulate_packet_loss(nodes: &[NodeInstance], loss_rate: f64) -> Result<()>`
Simulates packet loss between nodes.

##### `create_partition(nodes: &[NodeInstance], partition_groups: Vec<Vec<String>>) -> Result<()>`
Creates network partitions between node groups.

##### `restore_connectivity(nodes: &[NodeInstance]) -> Result<()>`
Restores normal network connectivity.

## Monitoring and Validation

### MonitoringSystem

Monitors system and application metrics.

```rust
pub struct MonitoringSystem {
    metrics_collector: Arc<MetricsCollector>,
    alert_manager: Arc<AlertManager>,
    reporting_interval: Duration,
}
```

#### Methods

##### `new(config: MonitoringConfig) -> Self`
Creates a new monitoring system.

##### `start_monitoring(nodes: &[NodeInstance]) -> Result<()>`
Starts monitoring for the specified nodes.

##### `collect_metrics() -> Result<SystemMetrics>`
Collects current system metrics.

##### `generate_performance_report() -> Result<PerformanceReport>`
Generates a performance analysis report.

### ValidationEngine

Validates test results and system state.

```rust
pub struct ValidationEngine {
    validation_rules: Vec<Box<dyn ValidationRule>>,
    consistency_checker: Arc<ConsistencyChecker>,
}
```

#### Methods

##### `new(config: ValidationConfig) -> Self`
Creates a new validation engine.

##### `validate_test_results(results: &[TestResult]) -> Result<ValidationReport>`
Validates test results against configured rules.

##### `check_data_consistency(nodes: &[NodeInstance]) -> Result<ConsistencyReport>`
Checks data consistency across nodes.

## Helper Functions and Utilities

### Preset Configurations

#### `presets::basic_cluster() -> OrchestratorConfig`
Returns configuration for basic 3-node testing.

#### `presets::standard_cluster() -> OrchestratorConfig`
Returns configuration for standard 5-node testing.

#### `presets::performance_cluster() -> OrchestratorConfig`
Returns configuration for performance testing with 10 nodes.

#### `presets::fault_tolerance_cluster() -> OrchestratorConfig`
Returns configuration for fault tolerance testing with network simulation.

#### `presets::ui_testing_cluster() -> OrchestratorConfig`
Returns configuration optimized for UI testing.

### Test Scenarios

#### `scenarios::basic_functionality() -> TestSuite`
Returns test suite for basic functionality validation.

#### `scenarios::comprehensive() -> TestSuite`
Returns comprehensive test suite covering all features.

#### `scenarios::ui_focused() -> TestSuite`
Returns UI-focused test suite.

#### `scenarios::performance_focused() -> TestSuite`
Returns performance-focused test suite.

#### `scenarios::fault_tolerance() -> TestSuite`
Returns fault tolerance test suite.

### Helper Functions

#### `helpers::init_test_framework() -> Result<()>`
Initializes logging and basic framework setup.

#### `helpers::load_config(path: &Path) -> Result<OrchestratorConfig>`
Loads configuration from TOML file.

#### `helpers::quick_test(preset: fn() -> OrchestratorConfig, scenario: fn() -> TestSuite) -> Result<TestReport>`
Runs a quick test with preset configuration and scenario.

#### `helpers::validate_environment() -> Result<Vec<String>>`
Validates system prerequisites for running tests.

## Error Types

### TestFrameworkError

Main error type for the testing framework.

```rust
pub enum TestFrameworkError {
    NodeStartupError(String),
    NetworkError(String),
    ValidationError(String),
    ConfigurationError(String),
    TestExecutionError(String),
    ResourceError(String),
}
```

## Configuration File Format

### TOML Configuration Example

```toml
[orchestrator]
node_count = 5
base_port = 40000
test_timeout = 1800  # seconds
work_dir = "/tmp/datamesh-test"
enable_network_simulation = false
enable_monitoring = true
parallel_execution = true
topology = "Star"  # Star, Ring, Mesh

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
base_latency_ms = 10
latency_variance_ms = 5
enable_packet_loss = false
packet_loss_rate = 0.01
enable_bandwidth_limit = false
bandwidth_limit_mbps = 100
enable_partitions = false

[monitoring]
metrics_interval = 5
collect_system_metrics = true
collect_network_metrics = true
collect_application_metrics = true
alert_on_failures = true
performance_threshold_cpu = 80.0
performance_threshold_memory = 80.0
```

## Environment Variables

All configuration options can be overridden using environment variables with the pattern:
`DATAMESH_<SECTION>_<KEY>`

For example:
- `DATAMESH_ORCHESTRATOR_NODE_COUNT=7`
- `DATAMESH_TEST_SUITE_INCLUDE_FAULT_TESTS=true`
- `DATAMESH_UI_TESTING_WEBDRIVER_URL=http://selenium:4444`

## Return Types and Status Codes

### TestReport

```rust
pub struct TestReport {
    pub test_run_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_duration: Duration,
    pub total_tests: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub test_results: Vec<TestResult>,
    pub performance_metrics: PerformanceMetrics,
    pub system_metrics: SystemMetrics,
    pub configuration: OrchestratorConfig,
}
```

### Exit Codes

- `0`: All tests passed successfully
- `1`: One or more tests failed
- `2`: Configuration error
- `3`: Environment setup error
- `4`: Resource allocation error
- `5`: Network connectivity error

This API reference provides comprehensive documentation for integrating with and extending the DataMesh Universal Testing Framework.