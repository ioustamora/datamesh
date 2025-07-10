/// Integration Tests for DataMesh Core Modules
///
/// This module provides integration tests for the core modules,
/// ensuring they work correctly together in realistic scenarios.

use tempfile::TempDir;
use tokio_test;
use std::path::PathBuf;
use chrono::Local;

// Import the DataMesh modules we want to test
use datamesh::database::{DatabaseManager, get_default_db_path};
use datamesh::presets::{NetworkPresets, parse_network_spec};
use datamesh::error_handling::{handle_error, file_not_found_error_with_suggestions, operation_error_with_context, ErrorBatch};

#[tokio::test]
async fn test_database_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test storing a file
    let upload_time = Local::now();
    let tags = vec!["test".to_string(), "integration".to_string()];
    
    let file_id = db.store_file(
        "test-file",
        "test-key-123", 
        "test.txt",
        1024,
        upload_time,
        &tags,
        "test-public-key"
    ).unwrap();
    
    assert!(file_id > 0);
    
    // Test retrieving by name
    let file = db.get_file_by_name("test-file").unwrap().unwrap();
    assert_eq!(file.name, "test-file");
    assert_eq!(file.file_key, "test-key-123");
    assert_eq!(file.tags.len(), 2);
    assert!(file.tags.contains(&"test".to_string()));
    
    // Test retrieving by key
    let file_by_key = db.get_file_by_key("test-key-123").unwrap().unwrap();
    assert_eq!(file_by_key.name, "test-file");
    
    // Test listing files
    let files = db.list_files(None).unwrap();
    assert_eq!(files.len(), 1);
    
    // Test tag filtering
    let tagged_files = db.list_files(Some("test")).unwrap();
    assert_eq!(tagged_files.len(), 1);
    
    // Test stats
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_files, 1);
    assert_eq!(stats.total_size, 1024);
}

#[test]
fn test_database_name_generation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("name_test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test unique name generation
    let name1 = db.generate_unique_name("test.txt").unwrap();
    assert_eq!(name1, "test");
    
    // Store a file with this name
    let upload_time = Local::now();
    db.store_file(
        &name1, "key1", "test.txt", 1024, upload_time, 
        &vec![], "pubkey1"
    ).unwrap();
    
    // Generate another name - should be different
    let name2 = db.generate_unique_name("test.txt").unwrap();
    assert_ne!(name1, name2);
    assert!(name2.starts_with("test"));
}

#[test]
fn test_database_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("error_test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test duplicate name error
    let upload_time = Local::now();
    db.store_file(
        "duplicate-test", "key1", "test.txt", 1024, upload_time,
        &vec![], "pubkey1"
    ).unwrap();
    
    // Try to store another file with the same name - should fail
    let duplicate_result = db.store_file(
        "duplicate-test", "key2", "test2.txt", 2048, upload_time,
        &vec![], "pubkey1"
    );
    
    assert!(duplicate_result.is_err());
    
    // Test non-existent file retrieval
    let missing_file = db.get_file_by_name("non-existent").unwrap();
    assert!(missing_file.is_none());
    
    let missing_by_key = db.get_file_by_key("non-existent-key").unwrap();
    assert!(missing_by_key.is_none());
}

#[test]
fn test_presets_functionality() {
    let presets = NetworkPresets::new();
    
    // Test built-in presets exist
    assert!(presets.get_preset("local").is_some());
    assert!(presets.get_preset("public").is_some()); 
    assert!(presets.get_preset("test").is_some());
    assert!(presets.get_preset("nonexistent").is_none());
    
    // Test preset application
    let local_config = presets.apply_preset("local").unwrap();
    assert!(local_config.discovery_enabled);
    assert!(!local_config.bootstrap_peers.is_empty());
    
    // Test custom network spec parsing
    let multiaddr_config = parse_network_spec("/ip4/127.0.0.1/tcp/40871").unwrap();
    assert_eq!(multiaddr_config.bootstrap_peers.len(), 1);
    assert!(multiaddr_config.bootstrap_peers[0].peer_id.is_none());
}

#[test]
fn test_error_handling_integration() {
    use std::io::{Error as IoError, ErrorKind};
    
    // Test IO error handling
    let io_error = IoError::new(ErrorKind::NotFound, "File not found");
    let enhanced = handle_error(&io_error);
    assert!(!enhanced.suggestions.is_empty());
    
    // Test file not found error
    let file_error = file_not_found_error_with_suggestions("test.txt");
    assert!(!file_error.suggestions.is_empty());
    
    // Test operation context
    let op_error = operation_error_with_context("put", &io_error);
    assert!(op_error.context.is_some());
    assert!(op_error.context.as_ref().unwrap().contains("upload"));
    
    // Test error batch
    let mut batch = ErrorBatch::new("Test batch".to_string());
    batch.add_error(file_error);
    batch.add_error(enhanced);
    
    assert_eq!(batch.count(), 2);
    assert!(!batch.is_empty());
}

#[tokio::test]
async fn test_module_integration() {
    // Test that modules work together properly
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("integration.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Add some test data
    let upload_time = Local::now();
    db.store_file(
        "integration-test", "int-key-123", "integration.txt", 
        2048, upload_time, &vec!["integration".to_string()], "test-key"
    ).unwrap();
    
    // Test stats
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_files, 1);
    assert_eq!(stats.total_size, 2048);
}

#[test]
fn test_governance_types() {
    use datamesh::governance::{AccountType, VerificationStatus};
    
    // Test account types
    let account_types = [
        AccountType::Free {
            storage_gb: 5,
            bandwidth_gb_month: 100,
            api_calls_hour: 1000,
        },
        AccountType::Premium {
            storage_gb: 100,
            bandwidth_gb_month: 1000,
            api_calls_hour: 10000,
        },
        AccountType::Enterprise {
            storage_unlimited: true,
            bandwidth_unlimited: true,
            api_calls_unlimited: true,
            sla_guarantee: 0.999,
        },
    ];
    
    // Ensure all account types are valid
    for account_type in &account_types {
        // This just tests that the enums are properly defined
        assert!(format!("{:?}", account_type).len() > 0);
    }
    
    // Test verification statuses
    let verification_statuses = [
        VerificationStatus::Unverified,
        VerificationStatus::EmailVerified,
        VerificationStatus::PhoneVerified,
        VerificationStatus::KYCVerified,
    ];
    
    for status in &verification_statuses {
        assert!(format!("{:?}", status).len() > 0);
    }
}

#[test]
fn test_economics_model() {
    use datamesh::economics::{EconomicModel, EconomicConfig};
    
    let economic_model = EconomicModel::new();
    let config = EconomicConfig::default();
    
    // Test that default configuration is reasonable
    assert!(config.storage_cost_per_gb_month > 0.0);
    assert!(config.bandwidth_cost_per_gb > 0.0);
    assert!(config.staking_reward_rate_annual > 0.0);
    assert!(config.staking_reward_rate_annual < 1.0); // Should be less than 100%
    
    // Test cost calculations
    let storage_gb = 10.0;
    let bandwidth_gb = 5.0;
    
    let storage_cost = storage_gb * config.storage_cost_per_gb_month;
    let bandwidth_cost = bandwidth_gb * config.bandwidth_cost_per_gb;
    let total_cost = storage_cost + bandwidth_cost;
    
    assert!(storage_cost > 0.0);
    assert!(bandwidth_cost > 0.0);
    assert!(total_cost > 0.0);
}

#[test] 
fn test_billing_system_types() {
    use datamesh::billing_system::{SubscriptionTier, BillingCycle, SubscriptionStatus, PaymentMethod};
    
    // Test subscription tiers
    let tiers = [
        SubscriptionTier::Free,
        SubscriptionTier::Basic,
        SubscriptionTier::Pro,
        SubscriptionTier::Enterprise,
        SubscriptionTier::Custom,
    ];
    
    for tier in &tiers {
        assert!(format!("{:?}", tier).len() > 0);
    }
    
    // Test billing cycles
    let cycles = [
        BillingCycle::Monthly,
        BillingCycle::Quarterly,
        BillingCycle::Yearly,
        BillingCycle::PayAsYouGo,
    ];
    
    for cycle in &cycles {
        assert!(format!("{:?}", cycle).len() > 0);
    }
    
    // Test subscription statuses
    let statuses = [
        SubscriptionStatus::Active,
        SubscriptionStatus::Suspended,
        SubscriptionStatus::Cancelled,
        SubscriptionStatus::Expired,
        SubscriptionStatus::PendingPayment,
    ];
    
    for status in &statuses {
        assert!(format!("{:?}", status).len() > 0);
    }
    
    // Test payment methods
    let credit_card = PaymentMethod::CreditCard { 
        last_four: "1234".to_string(), 
        expiry: "12/25".to_string() 
    };
    let paypal = PaymentMethod::PayPal { 
        email: "user@example.com".to_string() 
    };
    
    assert!(format!("{:?}", credit_card).contains("1234"));
    assert!(format!("{:?}", paypal).contains("user@example.com"));
}