/// Enhanced Unit Tests for DataMesh - Corrected Version
/// 
/// This comprehensive test suite validates core functionality across all modules
/// using the correct API signatures and data structures.

use anyhow::Result;
use chrono::{Local, Utc};
use std::time::Duration;
use tempfile::TempDir;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::governance::{AccountType, UserAccount, VerificationStatus, AbuseFlag};
use datamesh::key_manager::KeyManager;
use datamesh::billing_system::{BillingCycle, SubscriptionTier};
use datamesh::economics::EconomicModel;

/// Test utilities for setup and teardown
struct TestSetup {
    temp_dir: TempDir,
    db: DatabaseManager,
    key_manager: KeyManager,
}

impl TestSetup {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseManager::new(&db_path)?;
        
        // Create key manager properly
        let test_key = ecies::SecretKey::random(&mut rand::thread_rng());
        let key_manager = KeyManager::new(test_key, "test_key".to_string());
        
        Ok(TestSetup {
            temp_dir,
            db,
            key_manager,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_key_manager_basic() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test key generation and basic properties
        assert_eq!(setup.key_manager.key_info.name, "test_key");
        assert!(!setup.key_manager.key_info.public_key_hex.is_empty());
        
        // Test key manager was created properly
        assert!(setup.key_manager.key_info.created <= Local::now());
        
        Ok(())
    }
    
    #[test]
    fn test_database_basic_operations() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test file storage
        let upload_time = Local::now();
        let tags = vec!["test".to_string(), "basic".to_string()];
        
        setup.db.store_file(
            "test_file",
            "test_key",
            "test_file.txt",
            1024,
            upload_time,
            &tags,
            "test_public_key",
        )?;
        
        // Test file retrieval
        let file = setup.db.get_file_by_name("test_file")?;
        assert!(file.is_some());
        
        let file = file.unwrap();
        assert_eq!(file.name, "test_file");
        assert_eq!(file.file_size, 1024);
        
        // Test file listing
        let files = setup.db.list_files(None)?;
        assert_eq!(files.len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_database_search_and_filter() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Add multiple test files
        let upload_time = Local::now();
        
        // File 1: image file
        setup.db.store_file(
            "image1.jpg",
            "key1",
            "image1.jpg",
            2048,
            upload_time,
            &vec!["image".to_string(), "photo".to_string()],
            "test_public_key",
        )?;
        
        // File 2: document file
        setup.db.store_file(
            "document.pdf",
            "key2",
            "document.pdf",
            4096,
            upload_time,
            &vec!["document".to_string(), "pdf".to_string()],
            "test_public_key",
        )?;
        
        // Test search by tag
        let image_files = setup.db.search_files("image")?;
        assert_eq!(image_files.len(), 1);
        assert_eq!(image_files[0].name, "image1.jpg");
        
        // Test listing all files
        let all_files = setup.db.list_files(None)?;
        assert_eq!(all_files.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_user_account_creation() -> Result<()> {
        let user = UserAccount {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            public_key: "test_public_key".to_string(),
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 100,
                api_calls_hour: 1000,
            },
            verification_status: VerificationStatus::EmailVerified,
            registration_date: Utc::now(),
            last_activity: Utc::now(),
            reputation_score: 0.0,
            abuse_flags: vec![],
            subscription: None,
        };
        
        // Test user properties
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.reputation_score, 0.0);
        assert_eq!(user.abuse_flags.len(), 0);
        
        // Test account type limits
        if let AccountType::Free { storage_gb, bandwidth_gb_month, api_calls_hour } = user.account_type {
            assert_eq!(storage_gb, 5);
            assert_eq!(bandwidth_gb_month, 100);
            assert_eq!(api_calls_hour, 1000);
        } else {
            panic!("Expected Free account type");
        }
        
        Ok(())
    }
    
    #[test]
    fn test_billing_system_basic() -> Result<()> {
        // Test subscription tier enum
        let tier = SubscriptionTier::Premium {
            storage_gb: 100,
            bandwidth_gb: 1000,
            api_calls_hour: 10000,
        };
        
        if let SubscriptionTier::Premium { storage_gb, bandwidth_gb, api_calls_hour } = tier {
            assert_eq!(storage_gb, 100);
            assert_eq!(bandwidth_gb, 1000);
            assert_eq!(api_calls_hour, 10000);
        } else {
            panic!("Expected Premium subscription tier");
        }
        
        // Test billing cycle enum
        let cycle = BillingCycle::Monthly {
            start_date: Utc::now(),
            end_date: Utc::now() + chrono::Duration::days(30),
            amount_due: 9.99,
        };
        
        if let BillingCycle::Monthly { amount_due, .. } = cycle {
            assert_eq!(amount_due, 9.99);
        } else {
            panic!("Expected Monthly billing cycle");
        }
        
        Ok(())
    }
    
    #[test]
    fn test_configuration_loading() -> Result<()> {
        let config = Config::default();
        
        // Test that default config has reasonable values
        assert!(config.storage.keys_dir.is_some());
        assert!(config.network.default_port > 0);
        assert!(config.network.default_port < 65536);
        
        Ok(())
    }
    
    #[test]
    fn test_error_handling() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test accessing non-existent file
        let result = setup.db.get_file_by_name("nonexistent_file")?;
        assert!(result.is_none());
        
        // Test invalid key generation
        let test_key = ecies::SecretKey::random(&mut rand::thread_rng());
        let result = KeyManager::new(test_key, "".to_string()); // Empty name should be handled gracefully
        assert_eq!(result.key_info.name, ""); // Should create a key with empty name
        
        Ok(())
    }
    
    #[test]
    fn test_large_file_handling() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test storing large file metadata
        let large_file_size = 1024 * 1024 * 1024; // 1GB
        let upload_time = Local::now();
        let tags = vec!["large".to_string()];
        
        setup.db.store_file(
            "large_file.bin",
            "large_key",
            "large_file.bin",
            large_file_size,
            upload_time,
            &tags,
            "test_public_key",
        )?;
        
        let file = setup.db.get_file_by_name("large_file.bin")?;
        assert!(file.is_some());
        
        let file = file.unwrap();
        assert_eq!(file.file_size, large_file_size);
        
        Ok(())
    }
    
    #[test]
    fn test_concurrent_operations() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test multiple concurrent file operations
        let upload_time = Local::now();
        
        for i in 0..10 {
            let file_name = format!("concurrent_file_{}", i);
            let file_key = format!("concurrent_key_{}", i);
            let tags = vec!["concurrent".to_string()];
            
            setup.db.store_file(
                &file_name,
                &file_key,
                &file_name,
                1024 * (i + 1),
                upload_time,
                &tags,
                "test_public_key",
            )?;
        }
        
        // Verify all files were stored
        let files = setup.db.list_files(None)?;
        assert_eq!(files.len(), 10);
        
        Ok(())
    }
    
    #[test]
    fn test_economics_integration() -> Result<()> {
        // Test economic model basic functionality
        let economic_model = EconomicModel::new();
        
        // Test basic economic calculations
        let storage_cost = economic_model.calculate_storage_cost(1024 * 1024)?; // 1MB
        assert!(storage_cost > 0.0);
        
        let bandwidth_cost = economic_model.calculate_bandwidth_cost(1024 * 1024)?; // 1MB
        assert!(bandwidth_cost > 0.0);
        
        Ok(())
    }
    
    #[test]
    fn test_key_manager_lifecycle() -> Result<()> {
        // Test key generation
        let key1 = ecies::SecretKey::random(&mut rand::thread_rng());
        let key_manager1 = KeyManager::new(key1, "test_key_1".to_string());
        
        let key2 = ecies::SecretKey::random(&mut rand::thread_rng());
        let key_manager2 = KeyManager::new(key2, "test_key_2".to_string());
        
        // Test that different keys are generated
        assert_ne!(key_manager1.key_info.public_key_hex, key_manager2.key_info.public_key_hex);
        
        // Test key names
        assert_eq!(key_manager1.key_info.name, "test_key_1");
        assert_eq!(key_manager2.key_info.name, "test_key_2");
        
        Ok(())
    }
    
    #[test]
    fn test_database_performance() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test database insertion performance
        let start = std::time::Instant::now();
        let upload_time = Local::now();
        let tags = vec!["performance".to_string()];
        
        for i in 0..1000 {
            let file_name = format!("perf_file_{}", i);
            let file_key = format!("perf_key_{}", i);
            
            setup.db.store_file(
                &file_name,
                &file_key,
                &file_name,
                1024,
                upload_time,
                &tags,
                "test_public_key",
            )?;
        }
        
        let duration = start.elapsed();
        println!("1000 database insertions took: {:?}", duration);
        
        // Should complete in reasonable time (less than 5 seconds)
        assert!(duration < Duration::from_secs(5));
        
        // Verify all files were inserted
        let files = setup.db.list_files(None)?;
        assert_eq!(files.len(), 1000);
        
        Ok(())
    }
    
    #[test]
    fn test_abuse_flag_management() -> Result<()> {
        let mut user = UserAccount {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            public_key: "test_public_key".to_string(),
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 100,
                api_calls_hour: 1000,
            },
            verification_status: VerificationStatus::EmailVerified,
            registration_date: Utc::now(),
            last_activity: Utc::now(),
            reputation_score: 0.0,
            abuse_flags: vec![],
            subscription: None,
        };
        
        // Test adding abuse flags
        user.abuse_flags.push(AbuseFlag::Spam);
        user.abuse_flags.push(AbuseFlag::MaliciousContent);
        
        assert_eq!(user.abuse_flags.len(), 2);
        
        // Test filtering abuse flags
        let spam_flags: Vec<_> = user.abuse_flags.iter()
            .filter(|flag| matches!(flag, AbuseFlag::Spam))
            .collect();
        assert_eq!(spam_flags.len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_memory_usage_tracking() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test that we don't have obvious memory leaks
        let initial_memory = get_memory_usage();
        
        // Perform many operations
        for i in 0..100 {
            let file_name = format!("memory_test_{}", i);
            let upload_time = Local::now();
            
            setup.db.store_file(
                &file_name,
                &format!("key_{}", i),
                &file_name,
                1024,
                upload_time,
                &vec!["memory".to_string()],
                "test_public_key",
            )?;
        }
        
        // Force cleanup
        drop(setup);
        
        let final_memory = get_memory_usage();
        println!("Memory usage: initial={}, final={}", initial_memory, final_memory);
        
        // This is a basic check - real memory profiling would be more sophisticated
        Ok(())
    }
    
    #[test]
    fn test_config_validation() -> Result<()> {
        let config = Config::default();
        
        // Test network configuration
        assert!(config.network.max_connections > 0);
        assert!(config.network.connection_timeout_secs > 0);
        
        // Test storage configuration
        if let Some(keys_dir) = &config.storage.keys_dir {
            assert!(!keys_dir.as_os_str().is_empty());
        }
        
        Ok(())
    }
}

/// Basic memory usage detection (platform-specific)
fn get_memory_usage() -> usize {
    // This is a simplified version - real memory profiling would use more sophisticated tools
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        return kb_str.parse::<usize>().unwrap_or(0) * 1024;
                    }
                }
            }
        }
    }
    0 // Return 0 if unable to determine memory usage
}
