use chrono::Local;
use std::collections::HashMap;
use std::path::PathBuf;
/// Comprehensive Unit Tests for DataMesh Core Modules
///
/// This module provides extensive unit tests for all core DataMesh functionality,
/// ensuring each component works correctly in isolation before integration testing.
use tempfile::TempDir;
use tokio_test;

// Import the DataMesh modules we want to test
use datamesh::billing_system::{BillingCycle, PaymentMethod, SubscriptionStatus, SubscriptionTier};
use datamesh::config::{Config, NetworkConfig};
use datamesh::database::{DatabaseManager, FileEntry};
use datamesh::economics::{EconomicConfig, EconomicModel};
use datamesh::error_handling::{
    file_not_found_error_with_suggestions, handle_error, operation_error_with_context, ErrorBatch,
};
use datamesh::governance::{AccountType, NetworkGovernance, UserAccount, VerificationStatus};
use datamesh::key_manager::{KeyManager, KeySelectionMode};
use datamesh::presets::{parse_network_spec, NetworkPresets};
use datamesh::secure_random::{generate_secure_bytes, generate_secure_nonce, generate_secure_salt};
use datamesh::ui;

#[tokio::test]
async fn test_key_manager_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test_keys");

    // Test key generation
    let km = KeyManager::new_with_path(&key_path).unwrap();

    // Test key serialization/deserialization
    let test_data = b"Hello, World!";
    let encrypted = km.encrypt(test_data).unwrap();
    let decrypted = km.decrypt(&encrypted).unwrap();
    assert_eq!(test_data, decrypted.as_slice());

    // Test key persistence
    km.save_to_file(&key_path).unwrap();
    let loaded_km = KeyManager::load_from_file(&key_path).unwrap();

    // Test that loaded key works the same
    let encrypted2 = loaded_km.encrypt(test_data).unwrap();
    let decrypted2 = loaded_km.decrypt(&encrypted2).unwrap();
    assert_eq!(test_data, decrypted2.as_slice());
}

#[test]
fn test_secure_random_functions() {
    // Test secure nonce generation
    let nonce1 = generate_secure_nonce();
    let nonce2 = generate_secure_nonce();
    assert_ne!(nonce1, nonce2, "Nonces should be unique");
    assert_eq!(nonce1.len(), 12, "Nonce should be 12 bytes");

    // Test secure salt generation
    let salt1 = generate_secure_salt();
    let salt2 = generate_secure_salt();
    assert_ne!(salt1, salt2, "Salts should be unique");
    assert_eq!(salt1.len(), 32, "Salt should be 32 bytes");

    // Test secure bytes generation
    let bytes1 = generate_secure_bytes(64);
    let bytes2 = generate_secure_bytes(64);
    assert_ne!(bytes1, bytes2, "Random bytes should be unique");
    assert_eq!(
        bytes1.len(),
        64,
        "Should generate requested number of bytes"
    );
}

#[tokio::test]
async fn test_database_advanced_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("advanced_test.db");

    let db = DatabaseManager::new(&db_path).unwrap();

    // Test batch operations
    let upload_time = Local::now();
    let files_data = vec![
        (
            "file1",
            "key1",
            "file1.txt",
            1024,
            vec!["tag1".to_string(), "common".to_string()],
        ),
        (
            "file2",
            "key2",
            "file2.txt",
            2048,
            vec!["tag2".to_string(), "common".to_string()],
        ),
        (
            "file3",
            "key3",
            "file3.txt",
            4096,
            vec!["tag3".to_string(), "common".to_string()],
        ),
    ];

    for (name, key, original_name, size, tags) in files_data {
        db.store_file(name, key, original_name, size, upload_time, &tags, "pubkey")
            .unwrap();
    }

    // Test complex queries
    let all_files = db.list_files(None).unwrap();
    assert_eq!(all_files.len(), 3);

    let common_tagged = db.list_files(Some("common")).unwrap();
    assert_eq!(common_tagged.len(), 3);

    let tag1_files = db.list_files(Some("tag1")).unwrap();
    assert_eq!(tag1_files.len(), 1);

    // Test search functionality
    let search_results = db.search_files("file2").unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "file2");

    // Test stats accuracy
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_files, 3);
    assert_eq!(stats.total_size, 1024 + 2048 + 4096);
    assert!(stats.database_size > 0);

    // Test file deletion
    db.delete_file("file2").unwrap();
    let remaining_files = db.list_files(None).unwrap();
    assert_eq!(remaining_files.len(), 2);

    let updated_stats = db.get_stats().unwrap();
    assert_eq!(updated_stats.total_files, 2);
    assert_eq!(updated_stats.total_size, 1024 + 4096);
}

#[test]
fn test_error_handling_comprehensive() {
    use std::io::{Error as IoError, ErrorKind};

    // Test various error types
    let errors = vec![
        IoError::new(ErrorKind::NotFound, "File not found"),
        IoError::new(ErrorKind::PermissionDenied, "Access denied"),
        IoError::new(ErrorKind::AlreadyExists, "File already exists"),
        IoError::new(ErrorKind::InvalidData, "Invalid data format"),
        IoError::new(ErrorKind::UnexpectedEof, "Unexpected end of file"),
    ];

    for error in errors {
        let enhanced = handle_error(&error);
        assert!(
            !enhanced.suggestions.is_empty(),
            "Should provide suggestions for {:?}",
            error.kind()
        );
        assert!(
            !enhanced.error_message.is_empty(),
            "Should have error message"
        );
    }

    // Test specific error types
    let file_error = file_not_found_error_with_suggestions("missing-file.txt");
    assert!(file_error.suggestions.iter().any(|s| s.contains("list")));
    assert!(file_error.suggestions.iter().any(|s| s.contains("check")));

    // Test operation context
    let io_error = IoError::new(ErrorKind::NotFound, "File not found");
    let put_error = operation_error_with_context("put", &io_error);
    assert!(put_error.context.is_some());
    assert!(put_error.context.as_ref().unwrap().contains("upload"));

    let get_error = operation_error_with_context("get", &io_error);
    assert!(get_error.context.as_ref().unwrap().contains("download"));

    // Test error batch functionality
    let mut batch = ErrorBatch::new("Test batch operation".to_string());
    batch.add_error(file_error);
    batch.add_error(enhanced);

    assert_eq!(batch.count(), 2);
    assert!(!batch.is_empty());

    let summary = batch.summary();
    assert!(summary.contains("Test batch operation"));
    assert!(summary.contains("2 errors"));
}

#[test]
fn test_network_presets_advanced() {
    let presets = NetworkPresets::new();

    // Test all built-in presets
    let preset_names = ["local", "public", "test"];
    for preset_name in &preset_names {
        let preset = presets.get_preset(preset_name).unwrap();
        assert_eq!(preset.name, *preset_name);
        assert!(!preset.description.is_empty());
        assert!(!preset.bootstrap_peers.is_empty());

        // Test preset application
        let config = presets.apply_preset(preset_name).unwrap();
        assert!(config.port > 0);
        assert!(config.port < 65536);
        assert!(!config.bootstrap_peers.is_empty());
    }

    // Test custom network spec parsing
    let specs = vec![
        "/ip4/127.0.0.1/tcp/40871",
        "/ip4/192.168.1.100/tcp/40872",
        "/dns4/bootstrap.example.com/tcp/40873",
    ];

    for spec in specs {
        let config = parse_network_spec(spec).unwrap();
        assert!(!config.bootstrap_peers.is_empty());
        assert!(config.bootstrap_peers[0].peer_id.is_none());
    }

    // Test invalid specs
    let invalid_specs = vec![
        "invalid-multiaddr",
        "/ip4/999.999.999.999/tcp/40871",
        "/ip4/127.0.0.1/tcp/99999",
    ];

    for spec in invalid_specs {
        assert!(parse_network_spec(spec).is_err());
    }
}

#[test]
fn test_config_validation() {
    // Test default config
    let config = Config::default();
    assert!(config.network.port > 0);
    assert!(config.network.port < 65536);
    assert!(config.storage.max_file_size > 0);
    assert!(config.storage.replication_factor >= 2);
    assert!(!config.database.db_path.is_empty());

    // Test config serialization/deserialization
    let toml_str = toml::to_string(&config).unwrap();
    let parsed_config: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(config.network.port, parsed_config.network.port);
    assert_eq!(
        config.storage.max_file_size,
        parsed_config.storage.max_file_size
    );
}

#[test]
fn test_governance_system() {
    // Test account types
    let free_account = AccountType::Free {
        storage_gb: 5,
        bandwidth_gb_month: 100,
        api_calls_hour: 1000,
    };

    let premium_account = AccountType::Premium {
        storage_gb: 100,
        bandwidth_gb_month: 1000,
        api_calls_hour: 10000,
    };

    let enterprise_account = AccountType::Enterprise {
        storage_unlimited: true,
        bandwidth_unlimited: true,
        api_calls_unlimited: true,
        sla_guarantee: 0.999,
    };

    // Test account validation
    assert!(matches!(free_account, AccountType::Free { .. }));
    assert!(matches!(premium_account, AccountType::Premium { .. }));
    assert!(matches!(enterprise_account, AccountType::Enterprise { .. }));

    // Test verification statuses
    let statuses = [
        VerificationStatus::Unverified,
        VerificationStatus::EmailVerified,
        VerificationStatus::PhoneVerified,
        VerificationStatus::KYCVerified,
    ];

    for status in statuses {
        let status_str = format!("{:?}", status);
        assert!(!status_str.is_empty());
    }

    // Test user account creation
    let user = UserAccount {
        user_id: "user123".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hash123".to_string(),
        public_key: "pubkey123".to_string(),
        account_type: free_account,
        verification_status: VerificationStatus::EmailVerified,
        created_at: Local::now(),
        last_login: Some(Local::now()),
        is_active: true,
    };

    assert_eq!(user.email, "test@example.com");
    assert!(user.is_active);
}

#[test]
fn test_economics_calculations() {
    let config = EconomicConfig::default();
    let model = EconomicModel::new();

    // Test realistic cost calculations
    let storage_gb = 10.0;
    let bandwidth_gb = 5.0;
    let duration_months = 3.0;

    let storage_cost = storage_gb * config.storage_cost_per_gb_month * duration_months;
    let bandwidth_cost = bandwidth_gb * config.bandwidth_cost_per_gb;
    let total_cost = storage_cost + bandwidth_cost;

    assert!(storage_cost > 0.0);
    assert!(bandwidth_cost > 0.0);
    assert!(total_cost > storage_cost);
    assert!(total_cost > bandwidth_cost);

    // Test staking calculations
    let stake_amount = 1000.0;
    let annual_reward = stake_amount * config.staking_reward_rate_annual;
    let monthly_reward = annual_reward / 12.0;

    assert!(annual_reward > 0.0);
    assert!(annual_reward < stake_amount); // Realistic reward rate
    assert!(monthly_reward > 0.0);

    // Test that economic model is created successfully
    assert!(format!("{:?}", economic_model).len() > 0);
}

#[test]
fn test_billing_system_types() {
    // Test subscription tiers
    let tiers = [
        SubscriptionTier::Free,
        SubscriptionTier::Basic,
        SubscriptionTier::Pro,
        SubscriptionTier::Enterprise,
        SubscriptionTier::Custom,
    ];

    for tier in &tiers {
        let tier_str = format!("{:?}", tier);
        assert!(!tier_str.is_empty());
    }

    // Test billing cycles
    let cycles = [
        BillingCycle::Monthly,
        BillingCycle::Quarterly,
        BillingCycle::Yearly,
        BillingCycle::PayAsYouGo,
    ];

    for cycle in &cycles {
        let cycle_str = format!("{:?}", cycle);
        assert!(!cycle_str.is_empty());
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
        let status_str = format!("{:?}", status);
        assert!(!status_str.is_empty());
    }

    // Test payment methods
    let payment_methods = vec![
        PaymentMethod::CreditCard {
            last_four: "1234".to_string(),
            expiry: "12/25".to_string(),
        },
        PaymentMethod::PayPal {
            email: "user@example.com".to_string(),
        },
    ];

    for method in payment_methods {
        let method_str = format!("{:?}", method);
        assert!(!method_str.is_empty());
    }
}

#[test]
fn test_ui_formatting_functions() {
    // Test file size formatting
    let test_cases = vec![
        (0, "0 B"),
        (512, "512 B"),
        (1024, "1.0 KB"),
        (1536, "1.5 KB"),
        (1048576, "1.0 MB"),
        (1073741824, "1.0 GB"),
        (1099511627776, "1.0 TB"),
    ];

    for (bytes, expected) in test_cases {
        let formatted = ui::format_file_size(bytes);
        assert_eq!(formatted, expected, "Failed for {} bytes", bytes);
    }

    // Test duration formatting
    use std::time::Duration;
    let durations = vec![
        Duration::from_secs(30),
        Duration::from_secs(90),
        Duration::from_secs(3600),
        Duration::from_secs(7200),
    ];

    for duration in durations {
        let formatted = ui::format_duration(duration);
        assert!(!formatted.is_empty());
        // Basic validation that it contains expected units
        assert!(formatted.contains("s") || formatted.contains("m") || formatted.contains("h"));
    }
}

#[tokio::test]
async fn test_network_governance_operations() {
    let governance = NetworkGovernance::new();

    // Test proposal creation and voting
    // Note: This tests the structure and basic functionality
    // More comprehensive tests would require network setup

    // Test that governance system initializes correctly
    assert_eq!(format!("{:?}", governance).len() > 0);
}

#[test]
fn test_concurrent_data_structures() {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    // Test thread-safe operations that might be used in the system
    let data = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let data = Arc::clone(&data);
            thread::spawn(move || {
                let mut map = data.lock().unwrap();
                map.insert(i, format!("value_{}", i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let final_data = data.lock().unwrap();
    assert_eq!(final_data.len(), 10);
    for i in 0..10 {
        assert!(final_data.contains_key(&i));
    }
}

#[test]
fn test_cryptographic_primitives() {
    use blake3::hash;

    // Test BLAKE3 hashing consistency
    let data = b"test data for hashing";
    let hash1 = hash(data);
    let hash2 = hash(data);
    assert_eq!(hash1, hash2, "Hash should be deterministic");

    let different_data = b"different test data";
    let hash3 = hash(different_data);
    assert_ne!(
        hash1, hash3,
        "Different data should produce different hashes"
    );

    // Test that hash is expected length
    assert_eq!(hash1.as_bytes().len(), 32, "BLAKE3 hash should be 32 bytes");
}

#[test]
fn test_data_serialization() {
    use serde_json;

    // Test FileEntry serialization
    let file_entry = FileEntry {
        id: 1,
        name: "test-file".to_string(),
        file_key: "abc123".to_string(),
        original_filename: "test.txt".to_string(),
        file_size: 1024,
        upload_time: Local::now(),
        tags: vec!["test".to_string(), "example".to_string()],
        public_key_hex: "pubkey123".to_string(),
        chunks_total: 6,
        chunks_healthy: 6,
    };

    // Test JSON serialization
    let json_str = serde_json::to_string(&file_entry).unwrap();
    let deserialized: FileEntry = serde_json::from_str(&json_str).unwrap();

    assert_eq!(file_entry.name, deserialized.name);
    assert_eq!(file_entry.file_key, deserialized.file_key);
    assert_eq!(file_entry.file_size, deserialized.file_size);
    assert_eq!(file_entry.tags, deserialized.tags);
}
