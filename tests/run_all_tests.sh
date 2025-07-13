#!/bin/bash

# DataMesh Comprehensive Test Runner
# Runs all test suites: Rust unit tests, API tests, frontend tests, and E2E tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$TEST_DIR")"
FRONTEND_VUE_DIR="$PROJECT_ROOT/web-interface"
FRONTEND_REACT_DIR="$PROJECT_ROOT/web-ui"

# Test results tracking
RUST_TESTS_PASSED=0
API_TESTS_PASSED=0
VUE_TESTS_PASSED=0
REACT_TESTS_PASSED=0
E2E_TESTS_PASSED=0
SHELL_TESTS_PASSED=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

test_section() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# Run Rust unit tests
run_rust_tests() {
    test_section "Running Rust Unit Tests"
    
    cd "$PROJECT_ROOT"
    
    log_info "Running comprehensive unit tests..."
    if cargo test --test comprehensive_unit_tests; then
        log_success "Rust unit tests passed"
        RUST_TESTS_PASSED=1
    else
        log_error "Rust unit tests failed"
    fi
    
    log_info "Running API integration tests..."
    if cargo test --test api_integration_tests; then
        log_success "API integration tests passed"
        API_TESTS_PASSED=1
    else
        log_error "API integration tests failed"
    fi
    
    log_info "Running economy API tests..."
    if cargo test --test economy_api_tests; then
        log_success "Economy API tests passed"
    else
        log_error "Economy API tests failed"
    fi
}

# Run Vue.js frontend tests
run_vue_tests() {
    test_section "Running Vue.js Frontend Tests"
    
    if [[ ! -d "$FRONTEND_VUE_DIR" ]]; then
        log_warning "Vue.js frontend directory not found, skipping Vue tests"
        return
    fi
    
    cd "$FRONTEND_VUE_DIR"
    
    # Check if dependencies are installed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing Vue.js dependencies..."
        npm install
    fi
    
    log_info "Running Vue.js component tests..."
    if npm test; then
        log_success "Vue.js tests passed"
        VUE_TESTS_PASSED=1
    else
        log_error "Vue.js tests failed"
    fi
    
    log_info "Running Vue.js test coverage..."
    if npm run test:coverage; then
        log_success "Vue.js coverage generated"
    else
        log_warning "Vue.js coverage generation failed"
    fi
}

# Run React frontend tests
run_react_tests() {
    test_section "Running React Frontend Tests"
    
    if [[ ! -d "$FRONTEND_REACT_DIR" ]]; then
        log_warning "React frontend directory not found, skipping React tests"
        return
    fi
    
    cd "$FRONTEND_REACT_DIR"
    
    # Check if dependencies are installed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing React dependencies..."
        npm install
    fi
    
    log_info "Running React component tests..."
    if npm test; then
        log_success "React tests passed"
        REACT_TESTS_PASSED=1
    else
        log_error "React tests failed"
    fi
    
    log_info "Running React test coverage..."
    if npm run test:coverage; then
        log_success "React coverage generated"
    else
        log_warning "React coverage generation failed"
    fi
}

# Run end-to-end integration tests
run_e2e_tests() {
    test_section "Running End-to-End Integration Tests"
    
    cd "$PROJECT_ROOT"
    
    log_info "Starting DataMesh API server for E2E tests..."
    
    # Start API server in background
    cargo run --bin datamesh -- --port 8081 --config tests/test_config.toml &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 3
    
    log_info "Running E2E integration tests..."
    if cargo test --test integration_economy_e2e; then
        log_success "E2E integration tests passed"
        E2E_TESTS_PASSED=1
    else
        log_error "E2E integration tests failed"
    fi
    
    # Stop server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
}

# Run shell-based tests
run_shell_tests() {
    test_section "Running Shell-based Economy Tests"
    
    cd "$TEST_DIR"
    
    log_info "Running storage economy test suite..."
    if bash storage_economy_test_suite.sh --quick; then
        log_success "Shell economy tests passed"
        SHELL_TESTS_PASSED=1
    else
        log_error "Shell economy tests failed"
    fi
    
    if [[ "${FULL_MODE:-0}" == "1" ]]; then
        log_info "Running full shell test suite with load tests..."
        if bash storage_economy_test_suite.sh --full; then
            log_success "Full shell test suite passed"
        else
            log_error "Full shell test suite failed"
        fi
    fi
}

# Run linting and type checking
run_linting() {
    test_section "Running Code Quality Checks"
    
    cd "$PROJECT_ROOT"
    
    # Rust linting
    log_info "Running Rust clippy..."
    if cargo clippy -- -D warnings; then
        log_success "Rust clippy passed"
    else
        log_warning "Rust clippy found issues"
    fi
    
    # Rust formatting check
    log_info "Checking Rust formatting..."
    if cargo fmt -- --check; then
        log_success "Rust formatting is correct"
    else
        log_warning "Rust formatting issues found"
    fi
    
    # Vue.js linting
    if [[ -d "$FRONTEND_VUE_DIR" ]]; then
        cd "$FRONTEND_VUE_DIR"
        log_info "Running Vue.js linting..."
        if npm run lint; then
            log_success "Vue.js linting passed"
        else
            log_warning "Vue.js linting found issues"
        fi
    fi
    
    # React linting
    if [[ -d "$FRONTEND_REACT_DIR" ]]; then
        cd "$FRONTEND_REACT_DIR"
        log_info "Running React linting..."
        if npm run lint; then
            log_success "React linting passed"
        else
            log_warning "React linting found issues"
        fi
        
        log_info "Running React type checking..."
        if npm run type-check; then
            log_success "React type checking passed"
        else
            log_warning "React type checking found issues"
        fi
    fi
}

# Generate test coverage report
generate_coverage_report() {
    test_section "Generating Coverage Reports"
    
    cd "$PROJECT_ROOT"
    
    log_info "Generating Rust test coverage..."
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        cargo tarpaulin --out Html --output-dir target/coverage/
        log_success "Rust coverage report generated in target/coverage/"
    else
        log_warning "cargo-tarpaulin not installed, skipping Rust coverage"
    fi
    
    # Frontend coverage is generated during test runs
    if [[ -d "$FRONTEND_VUE_DIR/coverage" ]]; then
        log_success "Vue.js coverage available in web-interface/coverage/"
    fi
    
    if [[ -d "$FRONTEND_REACT_DIR/coverage" ]]; then
        log_success "React coverage available in web-ui/coverage/"
    fi
}

# Print final test results
print_results() {
    test_section "Test Results Summary"
    
    local total_passed=0
    local total_suites=6
    
    echo -e "Test Suite Results:"
    echo -e "==================="
    
    if [[ $RUST_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ Rust Unit Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ Rust Unit Tests${NC}"
    fi
    
    if [[ $API_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ API Integration Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ API Integration Tests${NC}"
    fi
    
    if [[ $VUE_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ Vue.js Frontend Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ Vue.js Frontend Tests${NC}"
    fi
    
    if [[ $REACT_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ React Frontend Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ React Frontend Tests${NC}"
    fi
    
    if [[ $E2E_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ End-to-End Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ End-to-End Tests${NC}"
    fi
    
    if [[ $SHELL_TESTS_PASSED == 1 ]]; then
        echo -e "${GREEN}✓ Shell Economy Tests${NC}"
        ((total_passed++))
    else
        echo -e "${RED}✗ Shell Economy Tests${NC}"
    fi
    
    echo -e "\nOverall: ${total_passed}/${total_suites} test suites passed"
    
    if [[ $total_passed == $total_suites ]]; then
        echo -e "${GREEN}🎉 All tests passed! DataMesh is ready for production.${NC}"
        exit 0
    else
        echo -e "${RED}⚠️  Some tests failed. Please review and fix issues before deployment.${NC}"
        exit 1
    fi
}

# Main execution
main() {
    echo -e "${GREEN}DataMesh Comprehensive Test Suite${NC}"
    echo -e "${GREEN}==================================${NC}"
    echo
    
    log_info "Starting comprehensive test run..."
    
    # Run all test suites
    run_rust_tests
    run_vue_tests
    run_react_tests
    run_e2e_tests
    run_shell_tests
    
    # Run code quality checks
    run_linting
    
    # Generate coverage reports
    generate_coverage_report
    
    # Print final results
    print_results
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --full)
            FULL_MODE=1
            shift
            ;;
        --quick)
            QUICK_MODE=1
            shift
            ;;
        --rust-only)
            RUST_ONLY=1
            shift
            ;;
        --frontend-only)
            FRONTEND_ONLY=1
            shift
            ;;
        --e2e-only)
            E2E_ONLY=1
            shift
            ;;
        --help)
            echo "DataMesh Test Runner"
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --full         Run all tests including load tests"
            echo "  --quick        Run quick test suite only"
            echo "  --rust-only    Run only Rust tests"
            echo "  --frontend-only Run only frontend tests"
            echo "  --e2e-only     Run only end-to-end tests"
            echo "  --help         Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run appropriate test subset based on flags
if [[ "${RUST_ONLY:-0}" == "1" ]]; then
    run_rust_tests
    run_linting
elif [[ "${FRONTEND_ONLY:-0}" == "1" ]]; then
    run_vue_tests
    run_react_tests
elif [[ "${E2E_ONLY:-0}" == "1" ]]; then
    run_e2e_tests
else
    main
fi