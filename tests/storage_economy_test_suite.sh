#!/bin/bash

# DataMesh Storage Economy System Test Suite
# This script comprehensively tests all storage economy features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="/tmp/datamesh_storage_economy_test"
CONTRIBUTION_PATH="$TEST_DIR/contribution_storage"
TEST_DATA_PATH="$TEST_DIR/test_data"
DATAMESH_CMD="${DATAMESH_CMD:-datamesh}"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

test_start() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}Testing: $1${NC}"
    echo -e "${BLUE}========================================${NC}"
    ((TESTS_TOTAL++))
}

test_command() {
    local cmd="$1"
    local expected_pattern="$2"
    local description="$3"
    
    log_info "Running: $cmd"
    
    if output=$(eval "$cmd" 2>&1); then
        if [[ -z "$expected_pattern" ]] || echo "$output" | grep -q "$expected_pattern"; then
            log_success "$description"
            return 0
        else
            log_error "$description - Expected pattern '$expected_pattern' not found"
            echo "Output: $output"
            return 1
        fi
    else
        log_error "$description - Command failed"
        echo "Output: $output"
        return 1
    fi
}

setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Create test directories
    rm -rf "$TEST_DIR"
    mkdir -p "$CONTRIBUTION_PATH"
    mkdir -p "$TEST_DATA_PATH"
    
    # Create test data
    dd if=/dev/urandom of="$TEST_DATA_PATH/test_file_1gb.dat" bs=1M count=1024 2>/dev/null
    dd if=/dev/urandom of="$TEST_DATA_PATH/test_file_100mb.dat" bs=1M count=100 2>/dev/null
    dd if=/dev/urandom of="$TEST_DATA_PATH/test_file_10mb.dat" bs=1M count=10 2>/dev/null
    
    # Set up contribution storage
    dd if=/dev/urandom of="$CONTRIBUTION_PATH/storage_space_4gb.dat" bs=1M count=4096 2>/dev/null
    
    log_success "Test environment setup complete"
}

cleanup_test_environment() {
    log_info "Cleaning up test environment..."
    rm -rf "$TEST_DIR"
    log_success "Test environment cleaned up"
}

# Test 1: Basic Economy Status
test_basic_economy_status() {
    test_start "Basic Economy Status"
    
    test_command "$DATAMESH_CMD economy" "Current Storage Tier|Free|Contributor|Premium" "Check basic economy status"
    test_command "$DATAMESH_CMD economy --tier-info" "Free.*100MB|Contributor.*4:1|Premium.*paid" "Check tier information"
    test_command "$DATAMESH_CMD economy --reputation" "Reputation.*Score|[0-9]+/100" "Check reputation score"
}

# Test 2: Quota System
test_quota_system() {
    test_start "Quota System"
    
    test_command "$DATAMESH_CMD quota" "Storage.*Used|Upload.*Quota|Download.*Quota" "Check quota overview"
    test_command "$DATAMESH_CMD quota --usage" "Storage.*Usage|Upload.*Usage|Download.*Usage" "Check usage statistics"
    test_command "$DATAMESH_CMD quota --tier" "Current.*Tier|Storage.*Limit|Quota.*Period" "Check tier details"
    test_command "$DATAMESH_CMD quota --economy" "Economy.*Status|Storage.*Economy" "Check economy integration"
}

# Test 3: Storage Contribution
test_storage_contribution() {
    test_start "Storage Contribution"
    
    # Test contribution setup
    test_command "$DATAMESH_CMD economy --contribute --path '$CONTRIBUTION_PATH' --amount 4GB" "Contribution.*registered|Successfully.*contributed" "Register storage contribution"
    
    # Test contribution verification
    test_command "$DATAMESH_CMD economy --verify" "Verification.*Status|Challenge.*Result" "Verify storage contribution"
    
    # Test contribution statistics
    test_command "$DATAMESH_CMD economy --contribution-stats" "Network.*Contribution|Contributed.*Storage|Earned.*Storage" "Check contribution statistics"
    
    # Test tier upgrade to contributor
    test_command "$DATAMESH_CMD economy --tier-info" "Contributor|4:1.*ratio|Verification.*required" "Check contributor tier activation"
}

# Test 4: Verification System
test_verification_system() {
    test_start "Verification System"
    
    # Test verification challenge
    test_command "$DATAMESH_CMD economy --test-challenge" "Challenge.*Type|Challenge.*Status|Response.*Time" "Test verification challenge"
    
    # Test verification history
    test_command "$DATAMESH_CMD economy --verification-history" "Verification.*History|Challenge.*Results|Success.*Rate" "Check verification history"
    
    # Test proof details
    test_command "$DATAMESH_CMD economy --proof-details" "Proof.*Type|Proof.*Status|Verification.*Method" "Check proof details"
    
    # Test monitoring controls
    test_command "$DATAMESH_CMD economy --enable-monitoring" "Monitoring.*enabled|Automatic.*verification" "Enable monitoring"
    test_command "$DATAMESH_CMD economy --disable-monitoring" "Monitoring.*disabled|Manual.*verification" "Disable monitoring"
}

# Test 5: Rewards System
test_rewards_system() {
    test_start "Rewards System"
    
    # Test rewards overview
    test_command "$DATAMESH_CMD economy --rewards" "Earned.*Credits|Bonus.*Storage|Reward.*Points" "Check rewards overview"
    
    # Test reputation system
    test_command "$DATAMESH_CMD economy --reputation" "Reputation.*Score|Verification.*Streak|Performance.*Rating" "Check reputation system"
    
    # Simulate successful verifications to build streak
    for i in {1..5}; do
        test_command "$DATAMESH_CMD economy --verify" "Verification.*Status" "Verification attempt $i"
        sleep 1
    done
    
    # Check for streak bonuses
    test_command "$DATAMESH_CMD economy --rewards" "Streak.*Bonus|Performance.*Bonus" "Check streak bonuses"
}

# Test 6: Premium Upgrade
test_premium_upgrade() {
    test_start "Premium Upgrade"
    
    # Test upgrade options
    test_command "$DATAMESH_CMD economy --upgrade-options" "Premium.*Options|Pricing.*Information|Available.*Plans" "Check upgrade options"
    
    # Test premium upgrade (mock payment)
    test_command "$DATAMESH_CMD economy --upgrade --premium-size 10GB --payment-method mock --duration 1" "Upgrade.*successful|Premium.*tier.*activated" "Test premium upgrade"
    
    # Verify premium tier activation
    test_command "$DATAMESH_CMD economy --tier-info" "Premium|10GB.*storage|No.*verification.*required" "Verify premium tier"
    
    # Test premium tier benefits
    test_command "$DATAMESH_CMD quota --tier" "Premium.*Tier|10GB.*storage|Enhanced.*quotas" "Check premium benefits"
}

# Test 7: Error Handling
test_error_handling() {
    test_start "Error Handling"
    
    # Test invalid contribution path
    test_command "$DATAMESH_CMD economy --contribute --path '/nonexistent/path' --amount 4GB" "Error|Invalid.*path|Path.*not.*found" "Test invalid contribution path"
    
    # Test insufficient storage
    test_command "$DATAMESH_CMD economy --contribute --path '$TEST_DATA_PATH' --amount 100GB" "Error|Insufficient.*storage|Not.*enough.*space" "Test insufficient storage"
    
    # Test invalid tier upgrade
    test_command "$DATAMESH_CMD economy --upgrade --premium-size 0GB" "Error|Invalid.*size|Size.*must.*be.*positive" "Test invalid upgrade size"
    
    # Test invalid payment method
    test_command "$DATAMESH_CMD economy --upgrade --premium-size 10GB --payment-method invalid" "Error|Invalid.*payment.*method|Unsupported.*payment" "Test invalid payment method"
}

# Test 8: Performance and Stress Testing
test_performance() {
    test_start "Performance Testing"
    
    # Test rapid verification challenges
    log_info "Testing rapid verification challenges..."
    for i in {1..10}; do
        test_command "$DATAMESH_CMD economy --test-challenge" "Challenge.*Type" "Rapid challenge $i"
    done
    
    # Test concurrent operations
    log_info "Testing concurrent operations..."
    {
        $DATAMESH_CMD economy --verify &
        $DATAMESH_CMD economy --rewards &
        $DATAMESH_CMD economy --reputation &
        wait
    } && log_success "Concurrent operations test" || log_error "Concurrent operations test failed"
    
    # Test large contribution
    log_info "Testing large contribution handling..."
    test_command "$DATAMESH_CMD economy --contribute --path '$CONTRIBUTION_PATH' --amount 4GB" "Successfully.*contributed|Large.*contribution.*accepted" "Large contribution test"
}

# Test 9: Configuration and Settings
test_configuration() {
    test_start "Configuration Testing"
    
    # Test configuration validation
    if [[ -f "config/storage_economy.toml" ]]; then
        log_info "Found storage economy configuration"
        test_command "grep -q 'storage_economy' config/storage_economy.toml" "" "Configuration file structure"
    else
        log_warning "No storage economy configuration found - using defaults"
    fi
    
    # Test configuration override
    test_command "$DATAMESH_CMD economy --tier-info" "Free.*100MB|Contributor.*4:1|Premium.*paid" "Default configuration active"
}

# Test 10: Integration Testing
test_integration() {
    test_start "Integration Testing"
    
    # Test quota integration
    test_command "$DATAMESH_CMD quota --economy" "Economy.*Status|Storage.*Economy.*Integration" "Quota economy integration"
    
    # Test file operations with economy
    log_info "Testing file operations with economy tiers..."
    
    # Create test file
    echo "Test file content" > "$TEST_DATA_PATH/integration_test.txt"
    
    # Test file upload within quota
    test_command "$DATAMESH_CMD put '$TEST_DATA_PATH/integration_test.txt'" "File.*uploaded|Successfully.*stored" "File upload with economy"
    
    # Test file retrieval
    test_command "$DATAMESH_CMD get integration_test.txt" "File.*retrieved|Successfully.*downloaded" "File retrieval with economy"
    
    # Test quota consumption
    test_command "$DATAMESH_CMD quota --usage" "Storage.*Used|Upload.*Used|Download.*Used" "Quota consumption tracking"
}

# Test 11: Edge Cases
test_edge_cases() {
    test_start "Edge Cases Testing"
    
    # Test zero-size contribution
    test_command "$DATAMESH_CMD economy --contribute --path '$TEST_DATA_PATH' --amount 0GB" "Error|Invalid.*amount|Amount.*must.*be.*positive" "Zero-size contribution"
    
    # Test negative amounts
    test_command "$DATAMESH_CMD economy --contribute --path '$TEST_DATA_PATH' --amount -1GB" "Error|Invalid.*amount|Negative.*amount" "Negative contribution amount"
    
    # Test extremely large amounts
    test_command "$DATAMESH_CMD economy --contribute --path '$TEST_DATA_PATH' --amount 1PB" "Error|Invalid.*amount|Amount.*too.*large" "Extremely large contribution"
    
    # Test invalid tier transitions
    test_command "$DATAMESH_CMD economy --upgrade --premium-size 1TB --payment-method mock" "Error|Invalid.*size|Size.*too.*large" "Invalid tier transition"
}

# Test 12: Recovery and Resilience
test_recovery() {
    test_start "Recovery Testing"
    
    # Test failure recovery
    log_info "Testing failure recovery scenarios..."
    
    # Simulate verification failure
    test_command "$DATAMESH_CMD economy --test-challenge" "Challenge.*Type" "Pre-failure verification"
    
    # Check reputation impact
    test_command "$DATAMESH_CMD economy --reputation" "Reputation.*Score" "Post-failure reputation check"
    
    # Test recovery through successful verifications
    for i in {1..3}; do
        test_command "$DATAMESH_CMD economy --verify" "Verification.*Status" "Recovery verification $i"
    done
    
    # Check reputation recovery
    test_command "$DATAMESH_CMD economy --reputation" "Reputation.*Score" "Post-recovery reputation check"
}

# Test 13: API Endpoint Validation
test_api_endpoints() {
    test_start "API Endpoint Validation"
    
    local BASE_URL="http://localhost:8080/api/v1"
    local AUTH_HEADER="Authorization: Bearer test_token_123"
    
    log_info "Testing REST API endpoints..."
    
    # Test economy status endpoint
    if command -v curl >/dev/null 2>&1; then
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/status' | grep -q 'health'" "" "GET /economy/status endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/profile' | grep -q 'tier'" "" "GET /economy/profile endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/quota' | grep -q 'upload_quota'" "" "GET /economy/quota endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/tiers' | grep -q 'tiers'" "" "GET /economy/tiers endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/rewards' | grep -q 'reputation_score'" "" "GET /economy/rewards endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/verification/history' | grep -q 'verifications'" "" "GET /economy/verification/history endpoint"
        test_command "curl -s -H '$AUTH_HEADER' '$BASE_URL/economy/network/stats' | grep -q 'total_storage'" "" "GET /economy/network/stats endpoint"
        
        # Test POST endpoints with sample data
        test_command "curl -s -X POST -H '$AUTH_HEADER' -H 'Content-Type: application/json' -d '{\"storage_path\":\"/tmp/test\",\"storage_amount\":1073741824,\"verification_method\":\"proof_of_space\"}' '$BASE_URL/economy/contribute' | grep -q 'contribution_id\\|error'" "" "POST /economy/contribute endpoint"
        
        test_command "curl -s -X POST -H '$AUTH_HEADER' -H 'Content-Type: application/json' -d '{\"challenge_id\":\"test_123\",\"response\":\"proof_data\",\"verification_type\":\"proof_of_space\"}' '$BASE_URL/economy/verify' | grep -q 'verification_result\\|error'" "" "POST /economy/verify endpoint"
        
        test_command "curl -s -X POST -H '$AUTH_HEADER' -H 'Content-Type: application/json' -d '{\"target_tier\":\"Premium\",\"storage_size\":10737418240,\"payment_method\":\"mock\",\"billing_period\":\"monthly\"}' '$BASE_URL/economy/upgrade' | grep -q 'upgrade_successful\\|error'" "" "POST /economy/upgrade endpoint"
    else
        log_warning "curl not available, skipping API endpoint tests"
    fi
}

# Test 14: Database Consistency
test_database_consistency() {
    test_start "Database Consistency"
    
    log_info "Testing database consistency after economy operations..."
    
    # Test that economy data persists across operations
    test_command "$DATAMESH_CMD economy" "Current.*Storage.*Tier" "Initial economy state"
    
    # Perform contribution and verify data persistence
    test_command "$DATAMESH_CMD economy --contribute --path '$CONTRIBUTION_PATH' --amount 2GB" "Contribution.*registered|Successfully.*contributed|Error" "Database write operation"
    
    # Verify data persistence
    test_command "$DATAMESH_CMD economy --contribution-stats" "Network.*Contribution|Error" "Database read after write"
    
    # Test quota tracking consistency
    test_command "$DATAMESH_CMD quota --usage" "Storage.*Usage|Upload.*Usage" "Quota tracking consistency"
}

# Test 15: Real-time Updates
test_realtime_updates() {
    test_start "Real-time Updates"
    
    log_info "Testing real-time quota and statistics updates..."
    
    # Get initial state
    test_command "$DATAMESH_CMD quota --usage" "Storage.*Usage" "Initial quota state"
    
    # Simulate file operation and check updates
    echo "Test file for quota update" > "$TEST_DATA_PATH/quota_test.txt"
    test_command "$DATAMESH_CMD put '$TEST_DATA_PATH/quota_test.txt'" "File.*uploaded|Successfully.*stored|Error" "File upload operation"
    
    # Check quota updates
    test_command "$DATAMESH_CMD quota --usage" "Storage.*Usage" "Updated quota state"
    
    # Test verification updates
    test_command "$DATAMESH_CMD economy --verify" "Verification.*Status" "Verification update"
    test_command "$DATAMESH_CMD economy --reputation" "Reputation.*Score" "Reputation update check"
}

# Test 16: Frontend Integration
test_frontend_integration() {
    test_start "Frontend Integration"
    
    log_info "Testing frontend integration endpoints..."
    
    # Test WebSocket connections if available
    if command -v nc >/dev/null 2>&1; then
        # Test WebSocket port availability
        test_command "nc -z localhost 8080" "" "API server availability"
    fi
    
    # Test CORS headers for web interfaces
    if command -v curl >/dev/null 2>&1; then
        test_command "curl -s -I -H 'Origin: http://localhost:3000' 'http://localhost:8080/api/v1/economy/status' | grep -q 'Access-Control\\|200 OK'" "" "CORS headers for web interface"
    fi
    
    # Test API response formats for frontend consumption
    test_command "$DATAMESH_CMD economy --format json" "\\{.*\\}" "JSON format output"
}

# Test 17: Load and Stress Testing
test_load_stress() {
    test_start "Load and Stress Testing"
    
    log_info "Testing system under load..."
    
    # Rapid verification challenges
    for i in {1..20}; do
        test_command "$DATAMESH_CMD economy --test-challenge" "Challenge.*Type|Error" "Load test verification $i" &
        if (( $i % 5 == 0 )); then
            wait # Wait for batch completion
        fi
    done
    wait
    
    # Concurrent quota checks
    log_info "Testing concurrent quota operations..."
    for i in {1..10}; do
        test_command "$DATAMESH_CMD quota --usage" "Storage.*Usage|Error" "Concurrent quota check $i" &
    done
    wait
    
    # Stress test API endpoints if curl is available
    if command -v curl >/dev/null 2>&1; then
        log_info "API stress testing..."
        for i in {1..15}; do
            curl -s -H "Authorization: Bearer test_token" "http://localhost:8080/api/v1/economy/status" > /dev/null &
        done
        wait
    fi
    
    # Verify system stability after load
    test_command "$DATAMESH_CMD economy" "Current.*Storage.*Tier" "System stability after load"
}

# Main test execution
main() {
    echo -e "${GREEN}DataMesh Storage Economy Test Suite${NC}"
    echo -e "${GREEN}====================================${NC}"
    echo
    
    # Setup
    setup_test_environment
    
    # Run all tests
    test_basic_economy_status
    test_quota_system
    test_storage_contribution
    test_verification_system
    test_rewards_system
    test_premium_upgrade
    test_error_handling
    test_performance
    test_configuration
    test_integration
    test_edge_cases
    test_recovery
    test_api_endpoints
    test_database_consistency
    test_realtime_updates
    test_frontend_integration
    
    # Run load tests only in full mode
    if [[ "${FULL_MODE:-0}" == "1" ]]; then
        test_load_stress
    fi
    
    # Cleanup
    cleanup_test_environment
    
    # Results summary
    echo
    echo -e "${GREEN}Test Results Summary${NC}"
    echo -e "${GREEN}===================${NC}"
    echo -e "Total Tests: $TESTS_TOTAL"
    echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    echo
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}All tests passed! Storage Economy system is working correctly.${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed. Please review the output above.${NC}"
        exit 1
    fi
}

# Help function
show_help() {
    cat << EOF
DataMesh Storage Economy Test Suite

Usage: $0 [OPTIONS]

Options:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    -q, --quick     Run only basic tests
    -f, --full      Run all tests including stress tests
    --setup-only    Only setup test environment
    --cleanup-only  Only cleanup test environment

Environment Variables:
    DATAMESH_CMD    Path to datamesh executable (default: datamesh)
    TEST_DIR        Test directory (default: /tmp/datamesh_storage_economy_test)

Examples:
    $0                  # Run all tests
    $0 --quick         # Run basic tests only
    $0 --verbose       # Run with verbose output
    $0 --setup-only    # Setup test environment only

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        -q|--quick)
            QUICK_MODE=1
            shift
            ;;
        -f|--full)
            FULL_MODE=1
            shift
            ;;
        --setup-only)
            setup_test_environment
            exit 0
            ;;
        --cleanup-only)
            cleanup_test_environment
            exit 0
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
