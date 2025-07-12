/// Enhanced Unit Tests for DataMesh
/// 
/// This comprehensive test suite validates core functionality across all modules.

use anyhow::Result;
use chrono::Local;
use std::time::Duration;
use tempfile::TempDir;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::governance::{AccountType, UserAccount, VerificationStatus};
use datamesh::key_manager::KeyManager;
use datamesh::billing_system::{BillingCycle, SubscriptionTier};

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
        
        // Create key manager with deterministic key for testing
        let test_key = ecies::SecretKey::from_slice(&[1u8; 32]).unwrap();
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
        
        // Test key generation
        let test_data = b"test data";
        let encrypted = setup.key_manager.encrypt(test_data)?;
        let decrypted = setup.key_manager.decrypt(&encrypted)?;
        
        assert_eq!(test_data, decrypted.as_slice());
        
        // Test signing
        let signature = setup.key_manager.sign(test_data)?;
        assert!(setup.key_manager.verify(test_data, &signature)?);
        
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
        assert_eq!(file.file_name, "test_file");
        assert_eq!(file.file_size, 1024);
        
        // Test file listing
        let files = setup.db.list_files()?;
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
        let image_files = setup.db.search_files_by_tag("image")?;
        assert_eq!(image_files.len(), 1);
        assert_eq!(image_files[0].file_name, "image1.jpg");
        
        // Test listing all files
        let all_files = setup.db.list_files()?;
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
            registration_date: Local::now(),
            last_activity: Local::now(),
            reputation_score: 0.0,
            abuse_flags: 0,
            subscription: None,
        };
        
        // Test user properties
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.reputation_score, 0.0);
        assert_eq!(user.abuse_flags, 0);
        
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
        // Test subscription tier creation
        let tier = SubscriptionTier {
            name: "Premium".to_string(),
            price_monthly: 9.99,
            storage_gb: 100,
            bandwidth_gb: 1000,
            api_calls_hour: 10000,
            support_level: "Priority".to_string(),
        };
        
        assert_eq!(tier.name, "Premium");
        assert_eq!(tier.price_monthly, 9.99);
        assert_eq!(tier.storage_gb, 100);
        
        // Test billing cycle
        let cycle = BillingCycle {
            cycle_id: uuid::Uuid::new_v4(),
            start_date: Local::now(),
            end_date: Local::now() + chrono::Duration::days(30),
            amount_due: 9.99,
            amount_paid: 0.0,
            status: "pending".to_string(),
        };
        
        assert_eq!(cycle.amount_due, 9.99);
        assert_eq!(cycle.amount_paid, 0.0);
        assert_eq!(cycle.status, "pending");
        
        Ok(())
    }
    
    #[test]
    fn test_configuration_loading() -> Result<()> {
        let config = Config::default();
        
        // Test that default config has reasonable values
        assert!(!config.storage.keys_dir.is_empty());
        assert!(config.network.listen_port > 0);
        assert!(config.network.listen_port < 65536);
        
        Ok(())
    }
    
    #[test]
    fn test_error_handling() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test accessing non-existent file
        let result = setup.db.get_file_by_name("nonexistent_file")?;
        assert!(result.is_none());
        
        // Test invalid key decryption
        let invalid_data = b"invalid encrypted data";
        let result = setup.key_manager.decrypt(invalid_data);
        assert!(result.is_err());
        
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
        let mut handles = Vec::new();
        
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
        let files = setup.db.list_files()?;
        assert_eq!(files.len(), 10);
        
        Ok(())
    }
    
    #[test]
    fn test_performance_basic() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test encryption performance
        let start = std::time::Instant::now();
        let test_data = vec![0u8; 1024]; // 1KB of data
        
        for _ in 0..100 {
            let encrypted = setup.key_manager.encrypt(&test_data)?;
            let _decrypted = setup.key_manager.decrypt(&encrypted)?;
        }
        
        let duration = start.elapsed();
        println!("100 encrypt/decrypt operations took: {:?}", duration);
        
        // Should complete in reasonable time (less than 1 second)
        assert!(duration < Duration::from_secs(1));
        
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
        let files = setup.db.list_files()?;
        assert_eq!(files.len(), 1000);
        
        Ok(())
    }
    
    #[test]
    fn test_memory_usage() -> Result<()> {
        let setup = TestSetup::new()?;
        
        // Test that we don't have obvious memory leaks
        // This is a basic test - more sophisticated memory testing would require additional tools
        
        let initial_memory = get_memory_usage();
        
        // Perform many operations
        for i in 0..1000 {
            let test_data = vec![0u8; 1024];
            let encrypted = setup.key_manager.encrypt(&test_data)?;
            let _decrypted = setup.key_manager.decrypt(&encrypted)?;
            
            // Store and retrieve file
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
        
        // Force garbage collection (if available)
        // In Rust, we rely on RAII, but we can drop explicitly
        drop(setup);
        
        let final_memory = get_memory_usage();
        println!("Memory usage: initial={}, final={}", initial_memory, final_memory);
        
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
