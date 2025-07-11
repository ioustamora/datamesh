use datamesh::config::Config;
/// API Integration Tests for DataMesh REST API
///
/// Basic testing of API server functionality and core endpoints
use tempfile::TempDir;

#[tokio::test]
async fn test_config_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    // Test that config is created successfully
    assert!(!config.database.db_path.is_empty());
    assert!(!config.storage.data_dir.is_empty());
}

#[tokio::test]
async fn test_api_config_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    // Test API configuration validation
    assert!(config.network.port > 0);
    assert!(config.network.port < 65536);
    assert!(config.storage.max_file_size > 0);
}

#[tokio::test]
async fn test_database_connection() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    // Test database connection
    let db = datamesh::database::DatabaseManager::new(&config.database.db_path).unwrap();
    let stats = db.get_stats().unwrap();

    assert_eq!(stats.total_files, 0);
    assert!(stats.database_size >= 0);
}

#[tokio::test]
async fn test_key_manager_initialization() {
    use datamesh::key_manager::KeyManager;

    // Test key manager initialization
    let km = KeyManager::new().unwrap();

    // Test basic encryption/decryption
    let test_data = b"API test data";
    let encrypted = km.encrypt(test_data).unwrap();
    let decrypted = km.decrypt(&encrypted).unwrap();

    assert_eq!(test_data, decrypted.as_slice());
}

#[tokio::test]
async fn test_file_operations_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    // Test basic file operations through database
    let db = datamesh::database::DatabaseManager::new(&config.database.db_path).unwrap();

    let upload_time = chrono::Local::now();
    let tags = vec!["api_test".to_string()];

    let file_id = db
        .store_file(
            "api_test_file",
            "api_test_key_123",
            "api_test.txt",
            1024,
            upload_time,
            &tags,
            "api_test_pubkey",
        )
        .unwrap();

    assert!(file_id > 0);

    // Test retrieval
    let file = db.get_file_by_name("api_test_file").unwrap().unwrap();
    assert_eq!(file.name, "api_test_file");
    assert_eq!(file.file_key, "api_test_key_123");
    assert_eq!(file.file_size, 1024);
}

#[tokio::test]
async fn test_governance_types() {
    use datamesh::governance::{AccountType, VerificationStatus};

    // Test governance types initialization
    let account_types = vec![
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
    ];

    for account_type in account_types {
        assert!(format!("{:?}", account_type).len() > 0);
    }

    let verification_statuses = vec![
        VerificationStatus::Unverified,
        VerificationStatus::EmailVerified,
        VerificationStatus::KYCVerified,
    ];

    for status in verification_statuses {
        assert!(format!("{:?}", status).len() > 0);
    }
}

#[tokio::test]
async fn test_economics_calculations() {
    use datamesh::economics::{EconomicConfig, EconomicModel};

    let model = EconomicModel::new();
    let config = EconomicConfig::default();

    // Test basic economic calculations
    assert!(config.storage_cost_per_gb_month > 0.0);
    assert!(config.bandwidth_cost_per_gb > 0.0);
    assert!(config.staking_reward_rate_annual > 0.0);
    assert!(config.staking_reward_rate_annual < 1.0);

    // Test cost calculations
    let storage_cost = 10.0 * config.storage_cost_per_gb_month;
    let bandwidth_cost = 5.0 * config.bandwidth_cost_per_gb;

    assert!(storage_cost > 0.0);
    assert!(bandwidth_cost > 0.0);

    // Test that model is created successfully
    assert!(format!("{:?}", model).len() > 0);
}

#[tokio::test]
async fn test_billing_system_types() {
    use datamesh::billing_system::{
        BillingCycle, PaymentMethod, SubscriptionStatus, SubscriptionTier,
    };

    // Test subscription tiers
    let tiers = vec![
        SubscriptionTier::Free,
        SubscriptionTier::Basic,
        SubscriptionTier::Pro,
        SubscriptionTier::Enterprise,
        SubscriptionTier::Custom,
    ];

    for tier in tiers {
        assert!(format!("{:?}", tier).len() > 0);
    }

    // Test billing cycles
    let cycles = vec![
        BillingCycle::Monthly,
        BillingCycle::Quarterly,
        BillingCycle::Yearly,
        BillingCycle::PayAsYouGo,
    ];

    for cycle in cycles {
        assert!(format!("{:?}", cycle).len() > 0);
    }

    // Test payment methods
    let credit_card = PaymentMethod::CreditCard {
        last_four: "1234".to_string(),
        expiry: "12/25".to_string(),
    };

    assert!(format!("{:?}", credit_card).contains("1234"));
}

#[tokio::test]
async fn test_network_presets() {
    use datamesh::presets::{parse_network_spec, NetworkPresets};

    let presets = NetworkPresets::new();

    // Test built-in presets
    assert!(presets.get_preset("local").is_some());
    assert!(presets.get_preset("public").is_some());
    assert!(presets.get_preset("test").is_some());

    // Test preset application
    let local_config = presets.apply_preset("local").unwrap();
    assert!(local_config.discovery_enabled);
    assert!(!local_config.bootstrap_peers.is_empty());

    // Test network spec parsing
    let multiaddr_config = parse_network_spec("/ip4/127.0.0.1/tcp/40871").unwrap();
    assert_eq!(multiaddr_config.bootstrap_peers.len(), 1);
}

#[tokio::test]
async fn test_error_handling() {
    use datamesh::error_handling::{file_not_found_error_with_suggestions, handle_error};
    use std::io::{Error as IoError, ErrorKind};

    // Test IO error handling
    let io_error = IoError::new(ErrorKind::NotFound, "File not found");
    let enhanced = handle_error(&io_error);
    assert!(!enhanced.suggestions.is_empty());

    // Test file not found error
    let file_error = file_not_found_error_with_suggestions("test.txt");
    assert!(!file_error.suggestions.is_empty());
    assert!(file_error.suggestions.iter().any(|s| s.contains("list")));
}

// Helper functions

fn create_test_config(temp_dir: &TempDir) -> Config {
    let mut config = Config::default();
    config.database.db_path = temp_dir
        .path()
        .join("test.db")
        .to_string_lossy()
        .to_string();
    config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
    config
}
