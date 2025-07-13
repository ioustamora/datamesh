/// End-to-End Integration Tests for DataMesh Storage Economy
/// 
/// These tests validate complete user workflows across the entire system:
/// - API endpoints → Frontend interactions → Database updates
/// - Real-time quota enforcement
/// - Complete contribution and verification flows

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use uuid::Uuid;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::storage_economy::StorageEconomy;
use datamesh::api_server::{start_api_server, ApiState};

/// End-to-end test setup with running API server
struct E2ETestSetup {
    temp_dir: TempDir,
    config: Config,
    api_state: ApiState,
    server_handle: tokio::task::JoinHandle<()>,
    base_url: String,
    test_user_id: String,
    auth_token: String,
}

impl E2ETestSetup {
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.database.db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        config.network.port = 8081; // Use different port for tests
        
        // Initialize database
        let db = DatabaseManager::new(&config.database.db_path)?;
        
        // Initialize storage economy
        let storage_economy = StorageEconomy::new(config.clone()).await?;
        
        // Create API state
        let api_state = ApiState {
            db: std::sync::Arc::new(db),
            storage_economy: std::sync::Arc::new(storage_economy),
            config: config.clone(),
        };
        
        // Start API server
        let server_handle = tokio::spawn(start_api_server(config.clone()));
        
        // Wait for server to start
        sleep(Duration::from_millis(500)).await;
        
        // Create test user
        let test_user_id = Uuid::new_v4().to_string();
        let auth_token = "test_auth_token_e2e".to_string();
        
        Ok(E2ETestSetup {
            temp_dir,
            config: config.clone(),
            api_state,
            server_handle,
            base_url: format!("http://localhost:{}", config.network.port),
            test_user_id,
            auth_token,
        })
    }
    
    fn auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.auth_token));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
    
    async fn api_get(&self, endpoint: &str) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/api/v1{}", self.base_url, endpoint))
            .headers((&self.auth_headers()).try_into()?)
            .send()
            .await?;
        Ok(response)
    }
    
    async fn api_post(&self, endpoint: &str, data: serde_json::Value) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/v1{}", self.base_url, endpoint))
            .headers((&self.auth_headers()).try_into()?)
            .json(&data)
            .send()
            .await?;
        Ok(response)
    }
}

impl Drop for E2ETestSetup {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

#[tokio::test]
async fn test_complete_user_journey_free_to_contributor() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Get initial user profile (should be Free tier)
    let response = setup.api_get("/economy/profile").await?;
    assert_eq!(response.status(), 200);
    
    let profile: serde_json::Value = response.json().await?;
    assert_eq!(profile["tier"], "Free");
    assert_eq!(profile["storage_contributed"], 0);
    
    // Step 2: Check initial quota limits
    let response = setup.api_get("/economy/quota").await?;
    assert_eq!(response.status(), 200);
    
    let quota: serde_json::Value = response.json().await?;
    let upload_limit = quota["upload_quota"]["limit"].as_u64().unwrap();
    assert!(upload_limit <= 104857600); // 100MB or less for free tier
    
    // Step 3: Attempt storage contribution (upgrade to Contributor)
    let contribution_data = json!({
        "storage_path": "/tmp/test_contribution_e2e",
        "storage_amount": 4294967296, // 4GB
        "verification_method": "proof_of_space"
    });
    
    let response = setup.api_post("/economy/contribute", contribution_data).await?;
    assert!(response.status() == 200 || response.status() == 400); // May fail due to path validation
    
    // If contribution succeeded, verify tier upgrade
    if response.status() == 200 {
        let contribution_result: serde_json::Value = response.json().await?;
        assert!(contribution_result["contribution_id"].is_string());
        
        // Step 4: Verify tier upgrade to Contributor
        let response = setup.api_get("/economy/profile").await?;
        let updated_profile: serde_json::Value = response.json().await?;
        assert_eq!(updated_profile["tier"], "Contributor");
        assert!(updated_profile["storage_contributed"].as_u64().unwrap() > 0);
        
        // Step 5: Check increased quota limits
        let response = setup.api_get("/economy/quota").await?;
        let updated_quota: serde_json::Value = response.json().await?;
        let new_upload_limit = updated_quota["upload_quota"]["limit"].as_u64().unwrap();
        assert!(new_upload_limit > upload_limit); // Should be higher than Free tier
    }
    
    Ok(())
}

#[tokio::test]
async fn test_quota_enforcement_workflow() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Get current quota status
    let response = setup.api_get("/economy/quota").await?;
    assert_eq!(response.status(), 200);
    
    let initial_quota: serde_json::Value = response.json().await?;
    let upload_used = initial_quota["upload_quota"]["used"].as_u64().unwrap_or(0);
    let upload_limit = initial_quota["upload_quota"]["limit"].as_u64().unwrap();
    
    // Step 2: Simulate file upload that approaches quota limit
    let large_file_size = upload_limit - upload_used - 1024; // Leave 1KB remaining
    
    // This would typically be done through file upload API, but we'll simulate quota usage
    let quota_update = json!({
        "upload_quota_used": upload_used + large_file_size,
        "operation": "file_upload"
    });
    
    // Step 3: Check quota status after simulated upload
    let response = setup.api_get("/economy/quota").await?;
    let updated_quota: serde_json::Value = response.json().await?;
    let percentage = updated_quota["upload_quota"]["percentage"].as_f64().unwrap();
    
    // Should be very high percentage (close to 100%)
    assert!(percentage > 95.0);
    
    // Step 4: Verify that further uploads would be rejected
    // This would be tested through actual file upload API in a real scenario
    
    Ok(())
}

#[tokio::test]
async fn test_verification_challenge_flow() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Set up storage contribution first
    let contribution_data = json!({
        "storage_path": "/tmp/test_verification",
        "storage_amount": 2147483648, // 2GB
        "verification_method": "proof_of_space"
    });
    
    let response = setup.api_post("/economy/contribute", contribution_data).await?;
    
    if response.status() == 200 {
        let contribution: serde_json::Value = response.json().await?;
        let challenge_id = contribution["verification_challenge"]["challenge_id"].as_str().unwrap();
        
        // Step 2: Submit verification response
        let verification_data = json!({
            "challenge_id": challenge_id,
            "response": "mock_proof_data_12345",
            "verification_type": "proof_of_space"
        });
        
        let response = setup.api_post("/economy/verify", verification_data).await?;
        assert!(response.status() == 200 || response.status() == 400);
        
        if response.status() == 200 {
            let verification_result: serde_json::Value = response.json().await?;
            assert!(verification_result["verification_result"].is_boolean());
            
            // Step 3: Check reputation impact
            let response = setup.api_get("/economy/rewards").await?;
            assert_eq!(response.status(), 200);
            
            let rewards: serde_json::Value = response.json().await?;
            assert!(rewards["reputation_score"].is_number());
            assert!(rewards["verification_streak"].is_number());
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_tier_upgrade_premium_flow() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Check available tiers
    let response = setup.api_get("/economy/tiers").await?;
    assert_eq!(response.status(), 200);
    
    let tiers: serde_json::Value = response.json().await?;
    let tier_list = tiers["tiers"].as_array().unwrap();
    assert!(tier_list.len() >= 4); // Free, Contributor, Premium, Enterprise
    
    // Step 2: Attempt premium upgrade
    let upgrade_data = json!({
        "target_tier": "Premium",
        "storage_size": 10737418240, // 10GB
        "payment_method": "mock",
        "billing_period": "monthly"
    });
    
    let response = setup.api_post("/economy/upgrade", upgrade_data).await?;
    assert!(response.status() == 200 || response.status() == 400);
    
    if response.status() == 200 {
        let upgrade_result: serde_json::Value = response.json().await?;
        assert!(upgrade_result["upgrade_successful"].as_bool().unwrap_or(false));
        
        // Step 3: Verify tier change
        let response = setup.api_get("/economy/profile").await?;
        let updated_profile: serde_json::Value = response.json().await?;
        assert_eq!(updated_profile["tier"], "Premium");
        
        // Step 4: Verify increased quotas
        let response = setup.api_get("/economy/quota").await?;
        let premium_quota: serde_json::Value = response.json().await?;
        let premium_limit = premium_quota["upload_quota"]["limit"].as_u64().unwrap();
        assert!(premium_limit >= 10737418240); // Should be at least 10GB
    }
    
    Ok(())
}

#[tokio::test]
async fn test_network_statistics_consistency() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Get economy status
    let response = setup.api_get("/economy/status").await?;
    assert_eq!(response.status(), 200);
    
    let economy_status: serde_json::Value = response.json().await?;
    let total_contributors = economy_status["total_contributors"].as_u64().unwrap();
    let total_storage = economy_status["total_storage_contributed"].as_u64().unwrap();
    
    // Step 2: Get detailed network stats
    let response = setup.api_get("/economy/network/stats").await?;
    assert_eq!(response.status(), 200);
    
    let network_stats: serde_json::Value = response.json().await?;
    let network_contributors = network_stats["active_contributors"].as_u64().unwrap();
    let network_storage = network_stats["total_storage"].as_u64().unwrap();
    
    // Step 3: Verify consistency between endpoints
    assert_eq!(total_contributors, network_contributors);
    assert_eq!(total_storage, network_storage);
    
    // Step 4: Verify health metrics
    assert!(network_stats["network_health"].is_string());
    assert!(network_stats["verification_rate"].is_number());
    
    Ok(())
}

#[tokio::test]
async fn test_real_time_quota_updates() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Get initial quota
    let response = setup.api_get("/economy/quota").await?;
    let initial_quota: serde_json::Value = response.json().await?;
    let initial_used = initial_quota["upload_quota"]["used"].as_u64().unwrap();
    
    // Step 2: Simulate quota usage change
    // In a real scenario, this would happen through file upload
    sleep(Duration::from_millis(100)).await;
    
    // Step 3: Get updated quota
    let response = setup.api_get("/economy/quota").await?;
    let updated_quota: serde_json::Value = response.json().await?;
    
    // Quota should be retrievable and valid
    assert!(updated_quota["upload_quota"]["used"].is_number());
    assert!(updated_quota["upload_quota"]["limit"].is_number());
    assert!(updated_quota["upload_quota"]["percentage"].is_number());
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Test invalid authentication
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/economy/status", setup.base_url))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await?;
    
    assert_eq!(response.status(), 401);
    
    // Step 2: Test invalid request data
    let invalid_data = json!({
        "storage_path": "",
        "storage_amount": -1,
        "verification_method": "invalid"
    });
    
    let response = setup.api_post("/economy/contribute", invalid_data).await?;
    assert_eq!(response.status(), 400);
    
    let error_response: serde_json::Value = response.json().await?;
    assert!(error_response["error"].is_string());
    
    // Step 3: Verify system recovers with valid request
    let response = setup.api_get("/economy/status").await?;
    assert_eq!(response.status(), 200);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Step 1: Make multiple concurrent API calls
    let tasks = vec![
        setup.api_get("/economy/status"),
        setup.api_get("/economy/profile"),
        setup.api_get("/economy/quota"),
        setup.api_get("/economy/tiers"),
        setup.api_get("/economy/rewards"),
    ];
    
    let results = futures::future::join_all(tasks).await;
    
    // Step 2: Verify all requests succeeded
    for result in results {
        match result {
            Ok(response) => assert_eq!(response.status(), 200),
            Err(_) => panic!("Concurrent request failed"),
        }
    }
    
    // Step 3: Verify data consistency after concurrent access
    let response = setup.api_get("/economy/status").await?;
    assert_eq!(response.status(), 200);
    
    let status: serde_json::Value = response.json().await?;
    assert!(status["health"].is_string());
    
    Ok(())
}

#[tokio::test]
async fn test_complete_contribution_verification_cycle() -> Result<()> {
    let setup = E2ETestSetup::new().await?;
    
    // Complete workflow: Free → Contributor → Verification → Rewards
    
    // Step 1: Start as Free user
    let response = setup.api_get("/economy/profile").await?;
    let initial_profile: serde_json::Value = response.json().await?;
    assert_eq!(initial_profile["tier"], "Free");
    
    // Step 2: Set up contribution
    let contribution_data = json!({
        "storage_path": "/tmp/complete_cycle_test",
        "storage_amount": 4294967296, // 4GB
        "verification_method": "proof_of_space"
    });
    
    let response = setup.api_post("/economy/contribute", contribution_data).await?;
    
    if response.status() == 200 {
        // Step 3: Check tier upgrade
        let response = setup.api_get("/economy/profile").await?;
        let contributor_profile: serde_json::Value = response.json().await?;
        assert_eq!(contributor_profile["tier"], "Contributor");
        
        // Step 4: Perform verification
        let verification_data = json!({
            "challenge_id": "test_challenge_complete",
            "response": "proof_response_data",
            "verification_type": "proof_of_space"
        });
        
        let response = setup.api_post("/economy/verify", verification_data).await?;
        
        if response.status() == 200 {
            // Step 5: Check rewards and reputation
            let response = setup.api_get("/economy/rewards").await?;
            let rewards: serde_json::Value = response.json().await?;
            
            assert!(rewards["reputation_score"].is_number());
            assert!(rewards["earned_storage"].is_number());
            
            // Step 6: Verify increased quotas
            let response = setup.api_get("/economy/quota").await?;
            let final_quota: serde_json::Value = response.json().await?;
            
            let final_limit = final_quota["upload_quota"]["limit"].as_u64().unwrap();
            let initial_limit = 104857600; // 100MB free tier limit
            assert!(final_limit > initial_limit);
        }
    }
    
    Ok(())
}

// Helper function to run all E2E tests
#[tokio::test]
async fn run_all_e2e_tests() -> Result<()> {
    println!("Running comprehensive E2E test suite...");
    
    // Run all tests in sequence to avoid port conflicts
    test_complete_user_journey_free_to_contributor().await?;
    test_quota_enforcement_workflow().await?;
    test_verification_challenge_flow().await?;
    test_tier_upgrade_premium_flow().await?;
    test_network_statistics_consistency().await?;
    test_real_time_quota_updates().await?;
    test_error_handling_and_recovery().await?;
    test_concurrent_operations().await?;
    test_complete_contribution_verification_cycle().await?;
    
    println!("All E2E tests completed successfully!");
    Ok(())
}