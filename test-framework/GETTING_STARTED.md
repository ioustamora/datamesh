# Getting Started with DataMesh Universal Testing Framework

## Quick Setup Guide

### 1. Prerequisites Check

Before running the tests, ensure your system meets the requirements:

```bash
# Check available memory (need at least 4GB)
free -h

# Check disk space (need at least 10GB in /tmp)
df -h /tmp

# Verify DataMesh binary exists
which datamesh || echo "DataMesh binary not found"
```

### 2. Build DataMesh

```bash
# From the main DataMesh directory
cargo build --release

# Verify build
./target/release/datamesh --version
```

### 3. Setup UI Testing (Optional)

If you want to run UI tests, set up Selenium WebDriver:

```bash
# Using Docker (recommended)
docker run -d -p 4444:4444 --name selenium selenium/standalone-chrome

# Verify WebDriver is running
curl http://localhost:4444/status
```

### 4. Build Test Framework

```bash
cd test-framework
cargo build --release
```

## Your First Test Run

### Basic 3-Node Test

Start with a simple test to verify everything works:

```bash
./scripts/run-tests.sh basic
```

This will:
- Deploy 3 DataMesh nodes
- Test basic file operations
- Test network connectivity
- Generate a test report

### Comprehensive 5-Node Test

Once basic tests pass, try a full test suite:

```bash
./scripts/run-tests.sh comprehensive --nodes 5
```

This includes:
- All CLI commands
- API endpoints
- Network operations
- Storage economy
- Governance features

## Understanding Test Results

### Test Output

Tests provide real-time output showing:
- Node deployment progress
- Individual test execution
- Pass/fail status
- Performance metrics

### Generated Reports

After completion, check:
- `test-results/test-*/test-report.html` - Detailed HTML report
- `test-results/test-*/SUMMARY.txt` - Quick text summary
- `logs/test-*.log` - Full execution logs

### Example Success Output

```
[INFO] 2024-12-14 14:30:22 Starting test execution
[INFO] 2024-12-14 14:30:25 Deploying 5-node cluster
[SUCCESS] 2024-12-14 14:30:45 All nodes deployed successfully
[INFO] 2024-12-14 14:30:46 Running comprehensive test suite
[SUCCESS] 2024-12-14 14:45:30 All tests completed successfully!
[SUCCESS] 2024-12-14 14:45:30 Report: test-results/test-20241214-143022/test-report.html
```

## Common First-Time Issues

### Port Conflicts

If you see port-related errors:

```bash
# Check what's using ports 40000-40010
netstat -ln | grep :4000

# Kill any existing DataMesh processes
pkill -f datamesh
```

### WebDriver Issues

If UI tests fail:

```bash
# Check if Selenium is running
docker ps | grep selenium

# Restart if needed
docker restart selenium

# Skip UI tests if not needed
./scripts/run-tests.sh comprehensive --no-ui
```

### Memory Issues

If nodes fail to start:

```bash
# Reduce node count
./scripts/run-tests.sh basic --nodes 3

# Or use basic test suite
./scripts/run-tests.sh basic
```

## Next Steps

### Explore Different Test Suites

```bash
# Test only CLI commands
./scripts/run-tests.sh cli-only

# Test UI interface
./scripts/run-tests.sh ui-only

# Test with network simulation
./scripts/run-tests.sh fault-tolerance --network-sim
```

### Customize Your Tests

```bash
# Test with more nodes
./scripts/run-tests.sh comprehensive --nodes 8

# Increase timeout for slow systems
./scripts/run-tests.sh comprehensive --timeout 3600

# Run sequentially for debugging
./scripts/run-tests.sh comprehensive --sequential
```

### Environment Variables

```bash
# Custom configuration
export DATAMESH_TEST_NODES=7
export DATAMESH_RUN_PERFORMANCE_TESTS=true
./scripts/run-tests.sh custom
```

## Help and Support

### Getting Help

```bash
# Show all available options
./scripts/run-tests.sh --help

# View test framework documentation
cat README.md
```

### Debugging Failed Tests

1. Check the detailed logs in `logs/test-*.log`
2. Look at individual node logs in `test-results/test-*/cluster/node-*/`
3. Review the HTML report for specific test failures
4. Verify system prerequisites are met

### Common Commands Reference

```bash
# Quick basic test
./scripts/run-tests.sh basic

# Full comprehensive test
./scripts/run-tests.sh comprehensive

# UI-only testing
./scripts/run-tests.sh ui-only

# Performance testing
./scripts/run-tests.sh performance-only --nodes 10

# Fault tolerance testing
./scripts/run-tests.sh fault-tolerance --network-sim

# Custom configuration
DATAMESH_TEST_NODES=6 ./scripts/run-tests.sh custom
```

## Integration Examples

### CI/CD Integration

Add to your `.github/workflows/test.yml`:

```yaml
- name: Run DataMesh Tests
  run: |
    cd test-framework
    ./scripts/run-tests.sh basic
    ./scripts/run-tests.sh comprehensive --no-ui --timeout 2400
```

### Development Workflow

```bash
# After making changes to DataMesh
cargo build --release

# Run targeted tests
cd test-framework
./scripts/run-tests.sh cli-only  # Test CLI changes
./scripts/run-tests.sh api-only  # Test API changes
./scripts/run-tests.sh ui-only   # Test UI changes
```

You're now ready to use the DataMesh Universal Testing Framework! Start with basic tests and gradually explore more advanced testing scenarios.