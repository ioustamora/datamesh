#!/bin/bash

# Enhanced Test Runner for DataMesh
# 
# This script runs the consolidated and improved test suite with proper
# categorization, parallel execution, and comprehensive reporting.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
CARGO_TEST_THREADS=${CARGO_TEST_THREADS:-4}
TEST_TIMEOUT=${TEST_TIMEOUT:-300}

echo -e "${BLUE}🚀 DataMesh Enhanced Test Suite${NC}"
echo "=================================="
echo "Test threads: $CARGO_TEST_THREADS"
echo "Timeout: ${TEST_TIMEOUT}s"
echo ""

# Function to run test category with timing
run_test_category() {
    local category=$1
    local pattern=$2
    local description=$3
    
    echo -e "${YELLOW}📋 Running $description...${NC}"
    start_time=$(date +%s)
    
    if timeout ${TEST_TIMEOUT} cargo test --test "$pattern" -- --test-threads=$CARGO_TEST_THREADS --nocapture; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${GREEN}✅ $description completed in ${duration}s${NC}"
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${RED}❌ $description failed after ${duration}s${NC}"
        return 1
    fi
}

# Function to run individual test files
run_individual_tests() {
    echo -e "${YELLOW}📋 Running Individual Test Files...${NC}"
    
    local failed_tests=0
    local total_tests=0
    
    # List of test files to run
    test_files=(
        "enhanced_unit_tests:Enhanced Unit Tests"
        "comprehensive_unit_tests:Comprehensive Unit Tests" 
        "enhanced_integration_tests:Enhanced Integration Tests"
        "api_websocket_integration:API WebSocket Integration Tests"
        "security_integration:Security Integration Tests"
        "governance_workflows:Governance Workflow Tests"
        "property_based_tests:Property-Based Tests"
        "api_integration_tests:API Integration Tests"
        "economy_api_tests:Economy API Tests"
        "enhanced_storage_tests:Enhanced Storage Tests"
        "enhanced_network_tests:Enhanced Network Tests"
        "enhanced_resilience_tests:Enhanced Resilience Tests"
    )
    
    for test_entry in "${test_files[@]}"; do
        IFS=':' read -r test_file test_description <<< "$test_entry"
        total_tests=$((total_tests + 1))
        
        if ! run_test_category "$test_file" "$test_file" "$test_description"; then
            failed_tests=$((failed_tests + 1))
        fi
        echo ""
    done
    
    return $failed_tests
}

# Main test execution
main() {
    local start_time=$(date +%s)
    local failed_categories=0
    
    echo -e "${BLUE}🔧 Building project...${NC}"
    if ! cargo build --tests; then
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Build successful${NC}"
    echo ""
    
    # Run test categories
    echo -e "${BLUE}🧪 Running Test Categories${NC}"
    echo "=========================="
    
    # Unit Tests
    if ! run_test_category "unit" "enhanced_unit_tests" "Unit Tests"; then
        failed_categories=$((failed_categories + 1))
    fi
    echo ""
    
    # Integration Tests  
    if ! run_test_category "integration" "*integration*" "Integration Tests"; then
        failed_categories=$((failed_categories + 1))
    fi
    echo ""
    
    # Security Tests
    if ! run_test_category "security" "security_integration" "Security Tests"; then
        failed_categories=$((failed_categories + 1))
    fi
    echo ""
    
    # Governance Tests
    if ! run_test_category "governance" "governance_workflows" "Governance Tests"; then
        failed_categories=$((failed_categories + 1))
    fi
    echo ""
    
    # Property-Based Tests
    if ! run_test_category "property" "property_based_tests" "Property-Based Tests"; then
        failed_categories=$((failed_categories + 1))
    fi
    echo ""
    
    # Performance Tests (if enabled)
    if [ "${RUN_PERFORMANCE_TESTS}" = "true" ]; then
        echo -e "${YELLOW}⚡ Running Performance Tests...${NC}"
        if ! cargo test --release --test "*performance*" -- --test-threads=1; then
            failed_categories=$((failed_categories + 1))
        fi
        echo ""
    fi
    
    # Generate test report
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    echo -e "${BLUE}📊 Test Summary${NC}"
    echo "==============="
    echo "Total duration: ${total_duration}s"
    echo "Failed categories: $failed_categories"
    
    if [ $failed_categories -eq 0 ]; then
        echo -e "${GREEN}🎉 All test categories passed!${NC}"
        
        # Run code coverage if requested
        if [ "${GENERATE_COVERAGE}" = "true" ]; then
            echo -e "${YELLOW}📈 Generating code coverage...${NC}"
            cargo install cargo-tarpaulin || true
            cargo tarpaulin --out Html --output-dir target/coverage
            echo -e "${GREEN}📈 Coverage report generated in target/coverage/tarpaulin-report.html${NC}"
        fi
        
        exit 0
    else
        echo -e "${RED}💥 $failed_categories test categories failed${NC}"
        exit 1
    fi
}

# Help function
show_help() {
    echo "DataMesh Enhanced Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help              Show this help message"
    echo "  --coverage          Generate code coverage report"
    echo "  --performance       Include performance tests"
    echo "  --threads=N         Set number of test threads (default: 4)"
    echo "  --timeout=N         Set test timeout in seconds (default: 300)"
    echo ""
    echo "Environment Variables:"
    echo "  CARGO_TEST_THREADS  Number of parallel test threads"
    echo "  TEST_TIMEOUT        Test timeout in seconds"
    echo "  RUN_PERFORMANCE_TESTS  Set to 'true' to run performance tests"
    echo "  GENERATE_COVERAGE   Set to 'true' to generate coverage report"
    echo ""
    echo "Examples:"
    echo "  $0                          # Run all tests"
    echo "  $0 --coverage               # Run tests with coverage"
    echo "  $0 --performance            # Include performance tests"
    echo "  $0 --threads=8 --timeout=600  # Custom configuration"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help)
            show_help
            exit 0
            ;;
        --coverage)
            export GENERATE_COVERAGE=true
            shift
            ;;
        --performance)
            export RUN_PERFORMANCE_TESTS=true
            shift
            ;;
        --threads=*)
            export CARGO_TEST_THREADS="${1#*=}"
            shift
            ;;
        --timeout=*)
            export TEST_TIMEOUT="${1#*=}"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run main function
main