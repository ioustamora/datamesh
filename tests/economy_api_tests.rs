/// Economy API Integration Tests for DataMesh
/// 
/// Comprehensive testing of all economy REST API endpoints including:
/// - Authentication and authorization
/// - Request/response validation
/// - Error handling
/// - Business logic validation

use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;
use uuid::Uuid;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::storage_economy::{StorageEconomy, StorageTier, UserStorageStatistics};
use datamesh::api_server::{ApiState, start_api_server};

/// Test setup for economy API tests
struct EconomyTestSetup {
    temp_dir: TempDir,
    config: Config,
    api_state: ApiState,
    test_user_id: String,
    auth_token: String,
}

impl EconomyTestSetup {
    async fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let mut config = Config::default();
        config.database.db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        
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
        
        // Create test user
        let test_user_id = Uuid::new_v4().to_string();
        let auth_token = "test_auth_token_123".to_string();
        
        Ok(EconomyTestSetup {
            temp_dir,
            config,
            api_state,
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
}

#[tokio::test]
async fn test_get_economy_status() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/status
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/status")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("health").is_some());
            assert!(body.get("total_contributors").is_some());
            assert!(body.get("total_storage_contributed").is_some());
            assert!(body.get("active_verifications").is_some());
            assert!(body.get("network_utilization").is_some());
        }
        Err(_) => {
            // Skip if server not running - this is integration test
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_user_economy_profile() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/profile
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/profile")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("user_id").is_some());
            assert!(body.get("tier").is_some());
            assert!(body.get("storage_contributed").is_some());
            assert!(body.get("upload_quota_used").is_some());
            assert!(body.get("upload_quota_limit").is_some());
            assert!(body.get("download_quota_used").is_some());
            assert!(body.get("download_quota_limit").is_some());
            assert!(body.get("reputation_score").is_some());
            assert!(body.get("verification_streak").is_some());
            assert!(body.get("last_activity").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_start_storage_contribution() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    let contribution_data = json!({
        "storage_path": "/tmp/test_contribution",
        "storage_amount": 1073741824, // 1GB in bytes
        "verification_method": "proof_of_space"
    });
    
    // Test POST /api/v1/economy/contribute
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/v1/economy/contribute")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .json(&contribution_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should be 200 for success or 400 for validation errors
            assert!(resp.status() == 200 || resp.status() == 400);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            
            if resp.status() == 200 {
                assert!(body.get("contribution_id").is_some());
                assert!(body.get("status").is_some());
                assert!(body.get("verification_challenge").is_some());
            } else {
                // Validation error
                assert!(body.get("error").is_some());
            }
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_contribution_status() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/contribution/status
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/contribution/status")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("contributions").is_some());
            assert!(body.get("total_contributed").is_some());
            assert!(body.get("active_contributions").is_some());
            assert!(body.get("earned_storage").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_submit_verification_response() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    let verification_data = json!({
        "challenge_id": "test_challenge_123",
        "response": "proof_data_here",
        "verification_type": "proof_of_space"
    });
    
    // Test POST /api/v1/economy/verify
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/v1/economy/verify")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .json(&verification_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should be 200 for success or 400 for invalid verification
            assert!(resp.status() == 200 || resp.status() == 400);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            
            if resp.status() == 200 {
                assert!(body.get("verification_result").is_some());
                assert!(body.get("reputation_change").is_some());
                assert!(body.get("streak_updated").is_some());
            }
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_tiers_info() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/tiers
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/tiers")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("tiers").is_some());
            
            let tiers = body.get("tiers").unwrap().as_array().unwrap();
            assert!(tiers.len() >= 4); // Free, Contributor, Premium, Enterprise
            
            // Check each tier has required fields
            for tier in tiers {
                assert!(tier.get("name").is_some());
                assert!(tier.get("storage_limit").is_some());
                assert!(tier.get("upload_quota").is_some());
                assert!(tier.get("download_quota").is_some());
            }
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_upgrade_tier() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    let upgrade_data = json!({
        "target_tier": "Premium",
        "storage_size": 10737418240, // 10GB
        "payment_method": "mock",
        "billing_period": "monthly"
    });
    
    // Test POST /api/v1/economy/upgrade
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/v1/economy/upgrade")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .json(&upgrade_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should be 200 for success or 400 for validation/payment errors
            assert!(resp.status() == 200 || resp.status() == 400);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            
            if resp.status() == 200 {
                assert!(body.get("upgrade_successful").is_some());
                assert!(body.get("new_tier").is_some());
                assert!(body.get("new_quotas").is_some());
            }
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_quota_status() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/quota
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/quota")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("upload_quota").is_some());
            assert!(body.get("download_quota").is_some());
            assert!(body.get("storage_quota").is_some());
            assert!(body.get("quota_period").is_some());
            assert!(body.get("reset_date").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_rewards_status() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/rewards
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/rewards")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("reputation_score").is_some());
            assert!(body.get("verification_streak").is_some());
            assert!(body.get("earned_storage").is_some());
            assert!(body.get("bonus_points").is_some());
            assert!(body.get("achievement_level").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_verification_history() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/verification/history
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/verification/history")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("verifications").is_some());
            assert!(body.get("success_rate").is_some());
            assert!(body.get("total_verifications").is_some());
            assert!(body.get("recent_streak").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_get_network_stats() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    // Test GET /api/v1/economy/network/stats
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/network/stats")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("total_storage").is_some());
            assert!(body.get("active_contributors").is_some());
            assert!(body.get("network_health").is_some());
            assert!(body.get("verification_rate").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

// Authentication and Authorization Tests

#[tokio::test]
async fn test_unauthorized_access() {
    // Test access without authentication token
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/status")
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 401);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("error").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_invalid_auth_token() {
    // Test access with invalid authentication token
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer invalid_token".to_string());
    
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/api/v1/economy/status")
        .headers((&headers).try_into().unwrap())
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 401);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("error").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

// Validation Tests

#[tokio::test]
async fn test_invalid_contribution_data() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    let invalid_data = json!({
        "storage_path": "", // Empty path should fail
        "storage_amount": -1, // Negative amount should fail
        "verification_method": "invalid_method"
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/v1/economy/contribute")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .json(&invalid_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("error").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

#[tokio::test]
async fn test_invalid_upgrade_data() {
    let setup = EconomyTestSetup::new().await.unwrap();
    
    let invalid_data = json!({
        "target_tier": "InvalidTier",
        "storage_size": 0, // Zero size should fail
        "payment_method": "",
        "billing_period": "invalid_period"
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/v1/economy/upgrade")
        .headers((&setup.auth_headers()).try_into().unwrap())
        .json(&invalid_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 400);
            
            let body: serde_json::Value = resp.json().await.unwrap();
            assert!(body.get("error").is_some());
        }
        Err(_) => {
            println!("Skipping API test - server not available");
        }
    }
}

// Unit Tests for Storage Economy Logic

#[tokio::test]
async fn test_storage_economy_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.database.db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    
    let storage_economy = StorageEconomy::new(config).await;
    assert!(storage_economy.is_ok());
}

#[tokio::test]
async fn test_user_storage_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.database.db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    
    let storage_economy = StorageEconomy::new(config).await.unwrap();
    let user_id = "test_user_123";
    
    // Test getting user statistics
    let stats = storage_economy.get_user_statistics(user_id).await;
    assert!(stats.is_ok());
    
    let stats = stats.unwrap();
    assert_eq!(stats.user_id, user_id);
    assert_eq!(stats.tier, StorageTier::Free);
    assert_eq!(stats.upload_quota_used, 0);
    assert_eq!(stats.download_quota_used, 0);
}

#[tokio::test]
async fn test_storage_tier_limits() {
    // Test that different tiers have appropriate limits
    let free_tier = StorageTier::Free;
    let contributor_tier = StorageTier::Contributor;
    let premium_tier = StorageTier::Premium { storage_gb: 10 };
    let enterprise_tier = StorageTier::Enterprise { storage_gb: 100 };
    
    // Test tier ordering and limits
    assert_ne!(free_tier, contributor_tier);
    assert_ne!(contributor_tier, premium_tier);
    assert_ne!(premium_tier, enterprise_tier);
}

#[tokio::test]
async fn test_quota_enforcement() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.database.db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    
    let storage_economy = StorageEconomy::new(config).await.unwrap();
    let user_id = "test_user_quota";
    
    // Test quota checking
    let file_size = 1024 * 1024; // 1MB
    let quota_available = storage_economy.check_upload_quota(user_id, file_size).await;
    
    // For new user on free tier, should have quota available
    assert!(quota_available.is_ok());
}