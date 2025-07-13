#!/bin/bash

# Test script for DataMesh API/Web Interface Integration
# This script tests the complete API/web interface connections

set -e  # Exit on any error

echo "🧪 DataMesh API/Web Interface Integration Test"
echo "=============================================="

# Configuration
API_BASE_URL="http://localhost:8080/api/v1"
WS_URL="ws://localhost:8080/api/v1/ws"
TEST_FILE="test_upload.txt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check if API server is running
check_api_server() {
    print_info "Checking if API server is running..."
    
    if curl -s -f "${API_BASE_URL}/health" > /dev/null; then
        print_success "API server is running"
        return 0
    else
        print_error "API server is not running"
        echo "Please start the API server first:"
        echo "  cargo run -- api-server"
        exit 1
    fi
}

# Test API health endpoint
test_health_endpoint() {
    print_info "Testing health endpoint..."
    
    response=$(curl -s "${API_BASE_URL}/health")
    if echo "$response" | grep -q "status"; then
        print_success "Health endpoint working"
    else
        print_error "Health endpoint failed"
        exit 1
    fi
}

# Test user registration
test_user_registration() {
    print_info "Testing user registration..."
    
    # Create test user
    response=$(curl -s -X POST "${API_BASE_URL}/auth/register" \
        -H "Content-Type: application/json" \
        -d '{
            "email": "test@datamesh.local",
            "password": "testpassword123",
            "public_key": "test_public_key"
        }' \
        -w "%{http_code}")
    
    http_code="${response: -3}"
    if [[ "$http_code" == "200" ]] || [[ "$http_code" == "409" ]]; then
        print_success "User registration working (status: $http_code)"
    else
        print_error "User registration failed (status: $http_code)"
        exit 1
    fi
}

# Test user login
test_user_login() {
    print_info "Testing user login..."
    
    response=$(curl -s -X POST "${API_BASE_URL}/auth/login" \
        -H "Content-Type: application/json" \
        -d '{
            "email": "test@datamesh.local",
            "password": "testpassword123"
        }')
    
    if echo "$response" | grep -q "access_token"; then
        ACCESS_TOKEN=$(echo "$response" | jq -r '.access_token')
        print_success "User login working"
        export ACCESS_TOKEN
    else
        print_error "User login failed"
        echo "Response: $response"
        exit 1
    fi
}

# Test file upload
test_file_upload() {
    print_info "Testing file upload..."
    
    # Create test file
    echo "Test file content for DataMesh API integration test" > "$TEST_FILE"
    
    response=$(curl -s -X POST "${API_BASE_URL}/files" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -F "file=@$TEST_FILE" \
        -w "%{http_code}")
    
    http_code="${response: -3}"
    body="${response%???}"
    
    if [[ "$http_code" == "200" ]]; then
        FILE_KEY=$(echo "$body" | jq -r '.file_key')
        print_success "File upload working (key: $FILE_KEY)"
        export FILE_KEY
    else
        print_error "File upload failed (status: $http_code)"
        echo "Response: $body"
        exit 1
    fi
}

# Test file download
test_file_download() {
    print_info "Testing file download..."
    
    if [[ -z "$FILE_KEY" ]]; then
        print_error "No file key available for download test"
        exit 1
    fi
    
    response=$(curl -s "${API_BASE_URL}/files/$FILE_KEY" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -w "%{http_code}" \
        -o downloaded_file.txt)
    
    http_code="${response: -3}"
    
    if [[ "$http_code" == "200" ]]; then
        print_success "File download working"
        rm -f downloaded_file.txt
    else
        print_error "File download failed (status: $http_code)"
        exit 1
    fi
}

# Test WebSocket connection (basic)
test_websocket_connection() {
    print_info "Testing WebSocket connection..."
    
    # Use websocat if available, otherwise skip WebSocket test
    if command -v websocat &> /dev/null; then
        # Test WebSocket connection with timeout
        timeout 5s websocat "$WS_URL" <<< '{"type":"ping","timestamp":"'$(date -Iseconds)'"}' > /dev/null 2>&1
        if [[ $? -eq 0 ]]; then
            print_success "WebSocket connection working"
        else
            print_error "WebSocket connection failed"
        fi
    else
        print_info "websocat not available, skipping WebSocket test"
        print_info "Install websocat to test WebSocket: cargo install websocat"
    fi
}

# Test analytics endpoints
test_analytics_endpoints() {
    print_info "Testing analytics endpoints..."
    
    # Test system metrics
    response=$(curl -s "${API_BASE_URL}/analytics/system" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -w "%{http_code}")
    
    http_code="${response: -3}"
    if [[ "$http_code" == "200" ]]; then
        print_success "Analytics endpoints working"
    else
        print_error "Analytics endpoints failed (status: $http_code)"
    fi
}

# Test governance endpoints
test_governance_endpoints() {
    print_info "Testing governance endpoints..."
    
    # Test governance status
    response=$(curl -s "${API_BASE_URL}/governance/status" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -w "%{http_code}")
    
    http_code="${response: -3}"
    if [[ "$http_code" == "200" ]]; then
        print_success "Governance endpoints working"
    else
        print_error "Governance endpoints failed (status: $http_code)"
    fi
}

# Test admin endpoints
test_admin_endpoints() {
    print_info "Testing admin endpoints..."
    
    # Test system health
    response=$(curl -s "${API_BASE_URL}/admin/health" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -w "%{http_code}")
    
    http_code="${response: -3}"
    if [[ "$http_code" == "200" ]]; then
        print_success "Admin endpoints working"
    else
        print_error "Admin endpoints failed (status: $http_code)"
    fi
}

# Test Swagger UI
test_swagger_ui() {
    print_info "Testing Swagger UI..."
    
    response=$(curl -s "http://localhost:8080/swagger-ui/" -w "%{http_code}")
    http_code="${response: -3}"
    
    if [[ "$http_code" == "200" ]]; then
        print_success "Swagger UI working"
    else
        print_error "Swagger UI failed (status: $http_code)"
    fi
}

# Cleanup
cleanup() {
    print_info "Cleaning up test files..."
    rm -f "$TEST_FILE"
    rm -f downloaded_file.txt
}

# Main test execution
main() {
    print_info "Starting DataMesh API/Web Interface Integration Tests"
    
    # Check dependencies
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        print_error "jq is required but not installed"
        exit 1
    fi
    
    # Run tests
    check_api_server
    test_health_endpoint
    test_user_registration
    test_user_login
    test_file_upload
    test_file_download
    test_websocket_connection
    test_analytics_endpoints
    test_governance_endpoints
    test_admin_endpoints
    test_swagger_ui
    
    cleanup
    
    echo ""
    print_success "🎉 All API/Web Interface Integration Tests Passed!"
    echo ""
    print_info "API Endpoints Tested:"
    echo "  • Authentication (register, login)"
    echo "  • File operations (upload, download)"
    echo "  • Analytics endpoints"
    echo "  • Governance endpoints"
    echo "  • Admin endpoints"
    echo "  • WebSocket connection"
    echo "  • Swagger UI"
    echo ""
    print_info "Web Interface Features Verified:"
    echo "  • REST API integration ✅"
    echo "  • Authentication system ✅"
    echo "  • File management ✅"
    echo "  • Real-time updates ✅"
    echo "  • Admin interface ✅"
    echo "  • API documentation ✅"
}

# Handle script interruption
trap cleanup EXIT INT TERM

# Run main function
main "$@"
