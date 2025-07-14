#!/bin/bash

# DataMesh Universal Multinode Testing Framework
# Main test execution script

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_RESULTS_DIR="${PROJECT_DIR}/test-results"
LOG_DIR="${PROJECT_DIR}/logs"

# Default configuration
NODE_COUNT=${DATAMESH_TEST_NODES:-5}
TEST_TIMEOUT=${DATAMESH_TEST_TIMEOUT:-1800}  # 30 minutes
ENABLE_UI_TESTS=${DATAMESH_ENABLE_UI_TESTS:-true}
ENABLE_NETWORK_SIM=${DATAMESH_TEST_NETWORK_SIM:-false}
ENABLE_MONITORING=${DATAMESH_ENABLE_MONITORING:-true}
PARALLEL_EXECUTION=${DATAMESH_PARALLEL_EXECUTION:-true}
WEBDRIVER_URL=${DATAMESH_WEBDRIVER_URL:-"http://localhost:4444"}

# Test categories
RUN_NETWORK_TESTS=${DATAMESH_RUN_NETWORK_TESTS:-true}
RUN_CLI_TESTS=${DATAMESH_RUN_CLI_TESTS:-true}
RUN_API_TESTS=${DATAMESH_RUN_API_TESTS:-true}
RUN_UI_TESTS=${DATAMESH_RUN_UI_TESTS:-true}
RUN_ECONOMY_TESTS=${DATAMESH_RUN_ECONOMY_TESTS:-true}
RUN_GOVERNANCE_TESTS=${DATAMESH_RUN_GOVERNANCE_TESTS:-true}
RUN_PERFORMANCE_TESTS=${DATAMESH_RUN_PERFORMANCE_TESTS:-true}
RUN_FAULT_TESTS=${DATAMESH_RUN_FAULT_TESTS:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

# Print help
print_help() {
    cat << EOF
DataMesh Universal Multinode Testing Framework

Usage: $0 [OPTIONS] [TEST_SUITE]

Options:
    -h, --help              Show this help message
    -n, --nodes COUNT       Number of nodes to deploy (default: $NODE_COUNT)
    -t, --timeout SECONDS   Test timeout in seconds (default: $TEST_TIMEOUT)
    --no-ui                 Disable UI tests
    --network-sim           Enable network simulation
    --no-monitoring         Disable monitoring
    --sequential            Run tests sequentially instead of parallel
    --webdriver-url URL     WebDriver URL for UI tests (default: $WEBDRIVER_URL)

Test Suites:
    basic                   Basic functionality tests (network + CLI)
    comprehensive           All test categories
    cli-only                CLI commands only
    ui-only                 UI interface only
    api-only                API endpoints only
    economy-only            Storage economy only
    governance-only         Governance system only
    performance-only        Performance benchmarks only
    fault-tolerance         Fault tolerance tests
    custom                  Custom test selection via environment variables

Environment Variables:
    DATAMESH_TEST_NODES              Number of nodes (default: 5)
    DATAMESH_TEST_TIMEOUT            Test timeout in seconds (default: 1800)
    DATAMESH_ENABLE_UI_TESTS         Enable UI tests (default: true)
    DATAMESH_TEST_NETWORK_SIM        Enable network simulation (default: false)
    DATAMESH_ENABLE_MONITORING       Enable monitoring (default: true)
    DATAMESH_PARALLEL_EXECUTION      Parallel execution (default: true)
    DATAMESH_WEBDRIVER_URL          WebDriver URL (default: http://localhost:4444)
    
    Test Category Controls:
    DATAMESH_RUN_NETWORK_TESTS       Run network tests (default: true)
    DATAMESH_RUN_CLI_TESTS           Run CLI tests (default: true)
    DATAMESH_RUN_API_TESTS           Run API tests (default: true)
    DATAMESH_RUN_UI_TESTS            Run UI tests (default: true)
    DATAMESH_RUN_ECONOMY_TESTS       Run economy tests (default: true)
    DATAMESH_RUN_GOVERNANCE_TESTS    Run governance tests (default: true)
    DATAMESH_RUN_PERFORMANCE_TESTS   Run performance tests (default: true)
    DATAMESH_RUN_FAULT_TESTS         Run fault tolerance tests (default: false)

Examples:
    $0 basic                        # Run basic test suite
    $0 comprehensive --nodes 7      # Run comprehensive tests with 7 nodes
    $0 ui-only --webdriver-url http://selenium:4444
    DATAMESH_TEST_NODES=10 $0 performance-only
    $0 fault-tolerance --network-sim

EOF
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_help
                exit 0
                ;;
            -n|--nodes)
                NODE_COUNT="$2"
                shift 2
                ;;
            -t|--timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --no-ui)
                ENABLE_UI_TESTS=false
                RUN_UI_TESTS=false
                shift
                ;;
            --network-sim)
                ENABLE_NETWORK_SIM=true
                shift
                ;;
            --no-monitoring)
                ENABLE_MONITORING=false
                shift
                ;;
            --sequential)
                PARALLEL_EXECUTION=false
                shift
                ;;
            --webdriver-url)
                WEBDRIVER_URL="$2"
                shift 2
                ;;
            basic|comprehensive|cli-only|ui-only|api-only|economy-only|governance-only|performance-only|fault-tolerance|custom)
                TEST_SUITE="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done

    # Set default test suite
    TEST_SUITE=${TEST_SUITE:-comprehensive}
}

# Setup test environment
setup_environment() {
    log_info "Setting up test environment"
    
    # Create directories
    mkdir -p "$TEST_RESULTS_DIR"
    mkdir -p "$LOG_DIR"
    
    # Create timestamp for this test run
    TEST_RUN_ID="test-$(date '+%Y%m%d-%H%M%S')"
    TEST_RUN_DIR="${TEST_RESULTS_DIR}/${TEST_RUN_ID}"
    mkdir -p "$TEST_RUN_DIR"
    
    # Set log file
    LOG_FILE="${LOG_DIR}/${TEST_RUN_ID}.log"
    
    log_info "Test run ID: $TEST_RUN_ID"
    log_info "Results directory: $TEST_RUN_DIR"
    log_info "Log file: $LOG_FILE"
}

# Configure test suite
configure_test_suite() {
    log_info "Configuring test suite: $TEST_SUITE"
    
    case $TEST_SUITE in
        basic)
            RUN_NETWORK_TESTS=true
            RUN_CLI_TESTS=true
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        comprehensive)
            # All tests enabled (use defaults)
            ;;
        cli-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=true
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        ui-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=false
            RUN_API_TESTS=false
            RUN_UI_TESTS=true
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        api-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=false
            RUN_API_TESTS=true
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        economy-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=false
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=true
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        governance-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=false
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=true
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=false
            ;;
        performance-only)
            RUN_NETWORK_TESTS=false
            RUN_CLI_TESTS=false
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=true
            RUN_FAULT_TESTS=false
            ;;
        fault-tolerance)
            RUN_NETWORK_TESTS=true
            RUN_CLI_TESTS=false
            RUN_API_TESTS=false
            RUN_UI_TESTS=false
            RUN_ECONOMY_TESTS=false
            RUN_GOVERNANCE_TESTS=false
            RUN_PERFORMANCE_TESTS=false
            RUN_FAULT_TESTS=true
            ENABLE_NETWORK_SIM=true
            ;;
        custom)
            # Use environment variables (already set)
            ;;
    esac
    
    log_info "Test configuration:"
    log_info "  Nodes: $NODE_COUNT"
    log_info "  Network tests: $RUN_NETWORK_TESTS"
    log_info "  CLI tests: $RUN_CLI_TESTS"
    log_info "  API tests: $RUN_API_TESTS"
    log_info "  UI tests: $RUN_UI_TESTS"
    log_info "  Economy tests: $RUN_ECONOMY_TESTS"
    log_info "  Governance tests: $RUN_GOVERNANCE_TESTS"
    log_info "  Performance tests: $RUN_PERFORMANCE_TESTS"
    log_info "  Fault tolerance tests: $RUN_FAULT_TESTS"
    log_info "  Network simulation: $ENABLE_NETWORK_SIM"
    log_info "  Monitoring: $ENABLE_MONITORING"
    log_info "  Parallel execution: $PARALLEL_EXECUTION"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites"
    
    # Check if datamesh binary exists
    if ! command -v datamesh &> /dev/null; then
        log_error "datamesh binary not found. Please build the project first."
        log_info "Run: cargo build --release"
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check UI test prerequisites
    if [[ "$RUN_UI_TESTS" == "true" ]]; then
        log_info "Checking UI test prerequisites"
        
        # Test WebDriver connection
        if ! curl -s "$WEBDRIVER_URL/status" > /dev/null; then
            log_warning "WebDriver not accessible at $WEBDRIVER_URL"
            log_warning "UI tests will be skipped. To run UI tests:"
            log_warning "  docker run -d -p 4444:4444 selenium/standalone-chrome"
            RUN_UI_TESTS=false
        else
            log_success "WebDriver connection verified"
        fi
    fi
    
    # Check available ports
    check_port_availability
    
    log_success "Prerequisites check completed"
}

# Check port availability
check_port_availability() {
    local base_port=40000
    local ports_needed=$((NODE_COUNT * 2))  # Each node needs 2 ports
    
    log_info "Checking port availability (need $ports_needed ports starting from $base_port)"
    
    for ((i=0; i<ports_needed; i++)); do
        local port=$((base_port + i))
        if netstat -ln 2>/dev/null | grep -q ":$port "; then
            log_error "Port $port is already in use"
            log_error "Please free the port or choose a different base port"
            exit 1
        fi
    done
    
    log_success "All required ports are available"
}

# Build test framework
build_test_framework() {
    log_info "Building test framework"
    
    cd "$PROJECT_DIR"
    
    # Build the main project if needed
    if [[ ! -f "target/release/datamesh" ]]; then
        log_info "Building DataMesh binary"
        cargo build --release || {
            log_error "Failed to build DataMesh binary"
            exit 1
        }
    fi
    
    # Build test framework
    log_info "Building test framework"
    cargo build --release --bin test-orchestrator || {
        log_error "Failed to build test framework"
        exit 1
    }
    
    log_success "Build completed"
}

# Run the test suite
run_test_suite() {
    log_info "Starting test execution"
    
    # Create configuration file
    local config_file="${TEST_RUN_DIR}/test-config.toml"
    create_test_config "$config_file"
    
    # Start background monitoring if enabled
    local monitoring_pid=""
    if [[ "$ENABLE_MONITORING" == "true" ]]; then
        start_monitoring "$TEST_RUN_DIR" &
        monitoring_pid=$!
        log_info "Monitoring started (PID: $monitoring_pid)"
    fi
    
    # Execute tests
    local test_cmd="cargo run --release --bin test-orchestrator -- --config $config_file"
    
    log_info "Executing: $test_cmd"
    
    if timeout "$TEST_TIMEOUT" $test_cmd 2>&1 | tee "$LOG_FILE"; then
        log_success "Test execution completed successfully"
        local exit_code=0
    else
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_error "Test execution timed out after $TEST_TIMEOUT seconds"
        else
            log_error "Test execution failed with exit code $exit_code"
        fi
    fi
    
    # Stop monitoring
    if [[ -n "$monitoring_pid" ]]; then
        kill "$monitoring_pid" 2>/dev/null || true
        log_info "Monitoring stopped"
    fi
    
    return $exit_code
}

# Create test configuration file
create_test_config() {
    local config_file="$1"
    
    cat > "$config_file" << EOF
[orchestrator]
node_count = $NODE_COUNT
base_port = 40000
test_timeout = $TEST_TIMEOUT
work_dir = "${TEST_RUN_DIR}/cluster"
enable_network_simulation = $ENABLE_NETWORK_SIM
enable_monitoring = $ENABLE_MONITORING
parallel_execution = $PARALLEL_EXECUTION

[test_suite]
include_network_tests = $RUN_NETWORK_TESTS
include_cli_tests = $RUN_CLI_TESTS
include_api_tests = $RUN_API_TESTS
include_ui_tests = $RUN_UI_TESTS
include_economy_tests = $RUN_ECONOMY_TESTS
include_governance_tests = $RUN_GOVERNANCE_TESTS
include_performance_tests = $RUN_PERFORMANCE_TESTS
include_fault_tests = $RUN_FAULT_TESTS

[test_data]
file_size_range = [1024, 10485760]  # 1KB to 10MB
file_count = 100
user_count = 20
proposal_count = 10

[ui_testing]
webdriver_url = "$WEBDRIVER_URL"
browser_timeout = 30
page_load_timeout = 10

[network_simulation]
enable_latency = true
enable_packet_loss = false
enable_bandwidth_limit = false
enable_partitions = $RUN_FAULT_TESTS

[monitoring]
metrics_interval = 5
collect_system_metrics = true
collect_network_metrics = true
collect_application_metrics = true
EOF
    
    log_info "Configuration written to: $config_file"
}

# Start background monitoring
start_monitoring() {
    local output_dir="$1"
    local monitoring_log="${output_dir}/monitoring.log"
    
    while true; do
        {
            echo "=== $(date) ==="
            echo "System Load:"
            uptime
            echo
            echo "Memory Usage:"
            free -h
            echo
            echo "Disk Usage:"
            df -h
            echo
            echo "Network Connections:"
            netstat -an | grep -E ":(40[0-9]{3}|41[0-9]{3})" | wc -l
            echo
        } >> "$monitoring_log"
        
        sleep 5
    done
}

# Generate test report
generate_report() {
    local exit_code=$1
    
    log_info "Generating test report"
    
    # Create HTML report
    local html_report="${TEST_RUN_DIR}/test-report.html"
    create_html_report "$html_report" "$exit_code"
    
    # Create summary
    create_test_summary "$exit_code"
    
    log_success "Test report generated: $html_report"
}

# Create HTML test report
create_html_report() {
    local html_file="$1"
    local exit_code="$2"
    
    cat > "$html_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>DataMesh Test Report - $TEST_RUN_ID</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; }
        .success { color: green; }
        .failure { color: red; }
        .warning { color: orange; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>DataMesh Universal Test Report</h1>
        <p><strong>Test Run ID:</strong> $TEST_RUN_ID</p>
        <p><strong>Date:</strong> $(date)</p>
        <p><strong>Test Suite:</strong> $TEST_SUITE</p>
        <p><strong>Node Count:</strong> $NODE_COUNT</p>
        <p class="$([ $exit_code -eq 0 ] && echo 'success' || echo 'failure')">
            <strong>Overall Result:</strong> $([ $exit_code -eq 0 ] && echo 'PASSED' || echo 'FAILED')
        </p>
    </div>
    
    <div class="section">
        <h2>Test Configuration</h2>
        <table>
            <tr><th>Setting</th><th>Value</th></tr>
            <tr><td>Network Tests</td><td>$RUN_NETWORK_TESTS</td></tr>
            <tr><td>CLI Tests</td><td>$RUN_CLI_TESTS</td></tr>
            <tr><td>API Tests</td><td>$RUN_API_TESTS</td></tr>
            <tr><td>UI Tests</td><td>$RUN_UI_TESTS</td></tr>
            <tr><td>Economy Tests</td><td>$RUN_ECONOMY_TESTS</td></tr>
            <tr><td>Governance Tests</td><td>$RUN_GOVERNANCE_TESTS</td></tr>
            <tr><td>Performance Tests</td><td>$RUN_PERFORMANCE_TESTS</td></tr>
            <tr><td>Fault Tolerance Tests</td><td>$RUN_FAULT_TESTS</td></tr>
            <tr><td>Network Simulation</td><td>$ENABLE_NETWORK_SIM</td></tr>
            <tr><td>Monitoring</td><td>$ENABLE_MONITORING</td></tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Test Results</h2>
        <p><em>Detailed test results would be populated here by the test orchestrator.</em></p>
        <p>See the log file for detailed output: <code>$LOG_FILE</code></p>
    </div>
    
    <div class="section">
        <h2>Files Generated</h2>
        <ul>
            <li>Test Configuration: <code>${TEST_RUN_DIR}/test-config.toml</code></li>
            <li>Execution Log: <code>$LOG_FILE</code></li>
            <li>Monitoring Data: <code>${TEST_RUN_DIR}/monitoring.log</code></li>
            <li>Results Directory: <code>$TEST_RUN_DIR</code></li>
        </ul>
    </div>
</body>
</html>
EOF
}

# Create test summary
create_test_summary() {
    local exit_code="$1"
    local summary_file="${TEST_RUN_DIR}/SUMMARY.txt"
    
    cat > "$summary_file" << EOF
DataMesh Universal Multinode Test Summary
=========================================

Test Run ID: $TEST_RUN_ID
Date: $(date)
Test Suite: $TEST_SUITE
Node Count: $NODE_COUNT

Overall Result: $([ $exit_code -eq 0 ] && echo 'PASSED' || echo 'FAILED')

Configuration:
- Network Tests: $RUN_NETWORK_TESTS
- CLI Tests: $RUN_CLI_TESTS
- API Tests: $RUN_API_TESTS
- UI Tests: $RUN_UI_TESTS
- Economy Tests: $RUN_ECONOMY_TESTS
- Governance Tests: $RUN_GOVERNANCE_TESTS
- Performance Tests: $RUN_PERFORMANCE_TESTS
- Fault Tolerance Tests: $RUN_FAULT_TESTS
- Network Simulation: $ENABLE_NETWORK_SIM
- Monitoring: $ENABLE_MONITORING

Files:
- HTML Report: ${TEST_RUN_DIR}/test-report.html
- Execution Log: $LOG_FILE
- Test Configuration: ${TEST_RUN_DIR}/test-config.toml
- Results Directory: $TEST_RUN_DIR

To view the full report, open: ${TEST_RUN_DIR}/test-report.html
EOF

    log_info "Test summary created: $summary_file"
}

# Cleanup function
cleanup() {
    log_info "Performing cleanup"
    
    # Kill any remaining datamesh processes
    pkill -f "datamesh" 2>/dev/null || true
    
    # Clean up any temporary files
    # (Test orchestrator handles its own cleanup)
    
    log_info "Cleanup completed"
}

# Main execution
main() {
    # Set up signal handlers
    trap cleanup EXIT INT TERM
    
    # Parse arguments
    parse_arguments "$@"
    
    # Setup
    setup_environment
    configure_test_suite
    check_prerequisites
    build_test_framework
    
    # Execute tests
    local exit_code=0
    run_test_suite || exit_code=$?
    
    # Generate report
    generate_report "$exit_code"
    
    # Print final status
    if [[ $exit_code -eq 0 ]]; then
        log_success "All tests completed successfully!"
        log_success "Report: ${TEST_RUN_DIR}/test-report.html"
    else
        log_error "Tests failed with exit code: $exit_code"
        log_error "Check the logs for details: $LOG_FILE"
    fi
    
    exit $exit_code
}

# Run main function with all arguments
main "$@"