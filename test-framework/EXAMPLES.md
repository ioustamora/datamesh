# DataMesh Testing Framework Examples

## Basic Usage Examples

### 1. Simple 3-Node Test

Run a basic functionality test with minimal resource usage:

```bash
./scripts/run-tests.sh basic
```

**What this does:**
- Deploys 3 DataMesh nodes
- Tests core file operations (put/get)
- Validates network connectivity
- Runs for approximately 10 minutes

**Expected output:**
```
[INFO] Setting up test environment
[INFO] Test run ID: test-20241214-143022
[INFO] Configuring test suite: basic
[SUCCESS] All required ports are available
[INFO] Building test framework
[SUCCESS] Build completed
[INFO] Starting test execution
[SUCCESS] Test execution completed successfully!
[SUCCESS] Report: test-results/test-20241214-143022/test-report.html
```

### 2. Comprehensive Testing

Full feature testing across all DataMesh components:

```bash
./scripts/run-tests.sh comprehensive --nodes 5
```

**Includes:**
- All CLI commands (40+ commands)
- Complete API endpoint testing
- UI automation testing
- Storage economy validation
- Governance system testing
- Performance benchmarks

## Advanced Usage Examples

### 3. Network Fault Tolerance Testing

Test how DataMesh handles network partitions and node failures:

```bash
./scripts/run-tests.sh fault-tolerance --network-sim --nodes 7
```

**Features tested:**
- Network partition recovery
- Node failure scenarios
- Data consistency during failures
- Bootstrap node failover
- Consensus mechanism validation

### 4. Performance Benchmarking

Stress test with maximum nodes and large datasets:

```bash
./scripts/run-tests.sh performance-only --nodes 10 --timeout 3600
```

**Performance metrics:**
- Concurrent file upload/download
- Network throughput measurement
- Storage efficiency analysis
- Memory and CPU usage tracking
- Database performance under load

### 5. UI-Only Testing

Focus on web interface functionality:

```bash
# Setup Selenium first
docker run -d -p 4444:4444 selenium/standalone-chrome

# Run UI tests
./scripts/run-tests.sh ui-only --webdriver-url http://localhost:4444
```

**UI features tested:**
- File manager operations
- Dashboard functionality
- Mobile responsiveness
- PWA capabilities
- Cross-browser compatibility

## Environment Configuration Examples

### 6. Custom Test Configuration

Use environment variables for specific test scenarios:

```bash
export DATAMESH_TEST_NODES=8
export DATAMESH_RUN_ECONOMY_TESTS=true
export DATAMESH_RUN_GOVERNANCE_TESTS=true
export DATAMESH_RUN_PERFORMANCE_TESTS=false
export DATAMESH_RUN_FAULT_TESTS=false

./scripts/run-tests.sh custom
```

### 7. Sequential Testing for Debugging

Run tests one at a time for easier debugging:

```bash
./scripts/run-tests.sh comprehensive --sequential --timeout 3600
```

**Benefits:**
- Easier to isolate failures
- Reduced resource contention
- Clearer logging output
- Better for development debugging

## Programmatic Usage Examples

### 8. Rust Library Integration

Use the framework as a Rust library:

```rust
use datamesh_test_framework::presets::*;
use datamesh_test_framework::scenarios::*;
use datamesh_test_framework::TestOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize framework
    datamesh_test_framework::helpers::init_test_framework()?;
    
    // Use preset configuration
    let config = standard_cluster();
    let test_suite = comprehensive();
    
    // Run tests
    let mut orchestrator = TestOrchestrator::new(config).await?;
    orchestrator.deploy_cluster().await?;
    
    let results = orchestrator.run_test_suite(test_suite).await?;
    let report = orchestrator.generate_report().await?;
    
    orchestrator.teardown().await?;
    
    println!("Tests completed: {} passed, {} failed", 
             report.passed_count, report.failed_count);
    
    Ok(())
}
```

### 9. Custom Test Scenario

Create a custom test scenario with specific requirements:

```rust
use datamesh_test_framework::*;

async fn custom_storage_economy_test() -> Result<TestReport> {
    let config = OrchestratorConfig {
        node_count: 6,
        base_port: 40000,
        test_timeout: Duration::from_secs(2400),
        work_dir: PathBuf::from("/tmp/custom-economy-test"),
        enable_network_simulation: false,
        enable_monitoring: true,
        parallel_execution: true,
        test_data_config: TestDataConfig {
            file_size_range: (1024 * 1024, 50 * 1024 * 1024), // 1MB to 50MB
            file_count: 200,
            user_count: 30,
            proposal_count: 15,
        },
        topology: ClusterTopology::Ring,
    };
    
    let test_suite = TestSuite {
        name: "Storage Economy Focus".to_string(),
        description: "Focused testing of storage economy features".to_string(),
        include_network_tests: true,
        include_cli_tests: true,
        include_api_tests: true,
        include_ui_tests: false,
        include_economy_tests: true,  // Main focus
        include_governance_tests: false,
        include_performance_tests: true,
        include_fault_tests: false,
    };
    
    helpers::quick_test(|| config, || test_suite).await
}
```

## Integration Examples

### 10. CI/CD Pipeline Integration

#### GitHub Actions

```yaml
name: DataMesh Multinode Tests
on: 
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  basic-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Build DataMesh
      run: cargo build --release
      
    - name: Setup Selenium
      run: |
        docker run -d -p 4444:4444 selenium/standalone-chrome
        
    - name: Run Basic Tests
      run: |
        cd test-framework
        ./scripts/run-tests.sh basic
        
    - name: Upload Test Results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: test-framework/test-results/

  comprehensive-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      
    - name: Build DataMesh
      run: cargo build --release
      
    - name: Run Comprehensive Tests
      run: |
        cd test-framework
        ./scripts/run-tests.sh comprehensive --no-ui --timeout 2400
```

#### GitLab CI

```yaml
stages:
  - build
  - test-basic
  - test-comprehensive

variables:
  DATAMESH_TEST_TIMEOUT: "2400"

build-datamesh:
  stage: build
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/datamesh

basic-tests:
  stage: test-basic
  script:
    - cd test-framework
    - ./scripts/run-tests.sh basic
  artifacts:
    when: always
    paths:
      - test-framework/test-results/
    reports:
      junit: test-framework/test-results/*/junit-report.xml

comprehensive-tests:
  stage: test-comprehensive
  only:
    - main
  script:
    - cd test-framework
    - ./scripts/run-tests.sh comprehensive --no-ui
  artifacts:
    when: always
    paths:
      - test-framework/test-results/
```

### 11. Docker Integration

Run tests in a containerized environment:

```dockerfile
# Dockerfile.test
FROM rust:1.70 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    netcat \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/datamesh /usr/local/bin/
COPY test-framework /app/test-framework

WORKDIR /app/test-framework
CMD ["./scripts/run-tests.sh", "basic"]
```

```bash
# Build and run
docker build -f Dockerfile.test -t datamesh-test .
docker run --rm -v $(pwd)/test-results:/app/test-framework/test-results datamesh-test
```

### 12. Development Workflow Integration

Integrate testing into your development workflow:

```bash
#!/bin/bash
# scripts/dev-test.sh

echo "Running DataMesh development tests..."

# Build latest changes
cargo build --release

# Run targeted tests based on changes
if git diff --name-only HEAD~1 | grep -q "src/.*\.rs"; then
    echo "Rust code changes detected, running CLI tests..."
    cd test-framework && ./scripts/run-tests.sh cli-only
fi

if git diff --name-only HEAD~1 | grep -q "web/"; then
    echo "Web code changes detected, running UI tests..."
    cd test-framework && ./scripts/run-tests.sh ui-only
fi

if git diff --name-only HEAD~1 | grep -q "src/api"; then
    echo "API changes detected, running API tests..."
    cd test-framework && ./scripts/run-tests.sh api-only
fi

# Always run basic connectivity test
echo "Running basic connectivity test..."
cd test-framework && ./scripts/run-tests.sh basic
```

## Monitoring and Debugging Examples

### 13. Debug Mode with Detailed Logging

```bash
# Enable verbose logging
export RUST_LOG=debug
export DATAMESH_LOG_LEVEL=debug

# Run with monitoring enabled
./scripts/run-tests.sh comprehensive --sequential
```

### 14. Performance Monitoring

Monitor system resources during test execution:

```bash
# Terminal 1: Start tests
./scripts/run-tests.sh performance-only --nodes 8

# Terminal 2: Monitor resources
watch -n 1 'ps aux | grep datamesh; free -h; df -h /tmp'

# Terminal 3: Monitor network
watch -n 1 'netstat -an | grep :4000 | wc -l'
```

### 15. Test Result Analysis

Analyze test results programmatically:

```bash
#!/bin/bash
# scripts/analyze-results.sh

LATEST_RESULT=$(ls -t test-results/ | head -1)
RESULT_DIR="test-results/$LATEST_RESULT"

echo "Analyzing test results from $LATEST_RESULT"

# Extract key metrics
echo "=== Test Summary ==="
cat "$RESULT_DIR/SUMMARY.txt"

echo "=== Performance Metrics ==="
grep -E "(duration|throughput|latency)" "$RESULT_DIR"/*.log

echo "=== Error Analysis ==="
grep -E "(ERROR|FAILED)" logs/test-*.log | head -10

echo "=== Resource Usage ==="
tail -20 "$RESULT_DIR/monitoring.log"
```

These examples demonstrate the flexibility and power of the DataMesh Universal Testing Framework across different use cases, from simple development testing to comprehensive CI/CD integration.