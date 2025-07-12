/// Enhanced Integration Tests for DataMesh
/// 
/// This comprehensive test suite validates end-to-end functionality and 
/// integration between different components of the DataMesh system.

use anyhow::Result;
use chrono::Local;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::file_manager::FileManager;
use datamesh::governance::{AccountType, UserAccount, VerificationStatus};
use datamesh::key_manager::KeyManager;
use datamesh::storage_manager::StorageManager;
use datamesh::billing_system::{BillingCycle, SubscriptionTier};

/// Complete integration test environment
struct IntegrationTestEnv {
    temp_dir: TempDir,
    config: Config,
    db: DatabaseManager,
    storage_manager: StorageManager,
    file_manager: FileManager,
    key_manager: KeyManager,
}

impl IntegrationTestEnv {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("integration_test.db");
        let storage_path = temp_dir.path().join("storage");
        
        // Create configuration
        let mut config = Config::default();
        config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();
        config.storage.data_dir = storage_path.to_string_lossy().to_string();
        
        // Initialize components
        let db = DatabaseManager::new(&db_path)?;
        let storage_manager = StorageManager::new(&storage_path)?;
        let file_manager = FileManager::new(&storage_path)?;
        
        // Create key manager with test key
        let test_key = ecies::SecretKey::from_slice(&[1u8; 32]).unwrap();
        let key_manager = KeyManager::new(test_key, "integration_test_key".to_string());
        
        Ok(IntegrationTestEnv {
            temp_dir,
            config,
            db,
            storage_manager,
            file_manager,
            key_manager,
        })
    }
    
    /// Create a test user with full setup
    fn create_test_user(&self, email: &str) -> Result<UserAccount> {
        let user = UserAccount {
            user_id: uuid::Uuid::new_v4(),
            email: email.to_string(),
            password_hash: "test_hash".to_string(),
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
        
        Ok(user)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_complete_file_workflow() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // 1. Create test user
        let user = env.create_test_user("test@example.com")?;
        
        // 2. Create test file content
        let file_content = b"This is test file content for integration testing";
        let file_name = "integration_test_file.txt";
        
        // 3. Encrypt file content
        let encrypted_content = env.key_manager.encrypt(file_content)?;
        
        // 4. Store file using storage manager
        let file_key = env.storage_manager.store_file(file_name, &encrypted_content)?;
        
        // 5. Store file metadata in database
        let upload_time = Local::now();
        let tags = vec!["integration".to_string(), "test".to_string()];
        
        env.db.store_file(
            file_name,
            &file_key,
            file_name,
            file_content.len() as u64,
            upload_time,
            &tags,
            &user.public_key,
        )?;
        
        // 6. Retrieve file metadata
        let file_entry = env.db.get_file_by_name(file_name)?;
        assert!(file_entry.is_some());
        
        let file_entry = file_entry.unwrap();
        assert_eq!(file_entry.file_name, file_name);
        assert_eq!(file_entry.file_size, file_content.len() as u64);
        
        // 7. Retrieve file content
        let retrieved_encrypted = env.storage_manager.get_file(&file_key)?;
        let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
        
        // 8. Verify content matches
        assert_eq!(file_content, retrieved_content.as_slice());
        
        println!("✅ Complete file workflow test passed");
        Ok(())
    }
    
    #[test]
    fn test_multi_user_file_sharing() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Create multiple users
        let user1 = env.create_test_user("user1@example.com")?;
        let user2 = env.create_test_user("user2@example.com")?;
        
        // User1 uploads a file
        let file_content = b"Shared file content";
        let file_name = "shared_file.txt";
        
        let encrypted_content = env.key_manager.encrypt(file_content)?;
        let file_key = env.storage_manager.store_file(file_name, &encrypted_content)?;
        
        // Store with user1's key
        env.db.store_file(
            file_name,
            &file_key,
            file_name,
            file_content.len() as u64,
            Local::now(),
            &vec!["shared".to_string()],
            &user1.public_key,
        )?;
        
        // User2 should be able to find and access the file (in a real system, this would involve permission checks)
        let file_entry = env.db.get_file_by_name(file_name)?;
        assert!(file_entry.is_some());
        
        // Retrieve and decrypt
        let retrieved_encrypted = env.storage_manager.get_file(&file_key)?;
        let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
        
        assert_eq!(file_content, retrieved_content.as_slice());
        
        println!("✅ Multi-user file sharing test passed");
        Ok(())
    }
    
    #[test]
    fn test_large_file_handling() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Create a larger test file (1MB)
        let file_size = 1024 * 1024;
        let file_content: Vec<u8> = (0..file_size).map(|i| (i % 256) as u8).collect();
        let file_name = "large_test_file.bin";
        
        // Process large file
        let encrypted_content = env.key_manager.encrypt(&file_content)?;
        let file_key = env.storage_manager.store_file(file_name, &encrypted_content)?;
        
        // Store metadata
        env.db.store_file(
            file_name,
            &file_key,
            file_name,
            file_content.len() as u64,
            Local::now(),
            &vec!["large".to_string()],
            "test_public_key",
        )?;
        
        // Retrieve and verify
        let retrieved_encrypted = env.storage_manager.get_file(&file_key)?;
        let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
        
        assert_eq!(file_content, retrieved_content);
        
        println!("✅ Large file handling test passed");
        Ok(())
    }
    
    #[test]
    fn test_concurrent_file_operations() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Simulate concurrent file uploads
        let num_files = 50;
        let mut file_keys = Vec::new();
        
        for i in 0..num_files {
            let file_content = format!("Concurrent file content {}", i);
            let file_name = format!("concurrent_file_{}.txt", i);
            
            // Encrypt and store
            let encrypted_content = env.key_manager.encrypt(file_content.as_bytes())?;
            let file_key = env.storage_manager.store_file(&file_name, &encrypted_content)?;
            
            // Store metadata
            env.db.store_file(
                &file_name,
                &file_key,
                &file_name,
                file_content.len() as u64,
                Local::now(),
                &vec!["concurrent".to_string()],
                "test_public_key",
            )?;
            
            file_keys.push((file_key, file_content));
        }
        
        // Verify all files can be retrieved
        for (i, (file_key, original_content)) in file_keys.iter().enumerate() {
            let retrieved_encrypted = env.storage_manager.get_file(file_key)?;
            let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
            
            assert_eq!(original_content.as_bytes(), retrieved_content.as_slice());
        }
        
        // Verify database contains all files
        let all_files = env.db.list_files()?;
        assert_eq!(all_files.len(), num_files);
        
        println!("✅ Concurrent file operations test passed");
        Ok(())
    }
    
    #[test]
    fn test_file_search_and_filtering() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Create files with different categories
        let file_categories = vec![
            ("document1.pdf", vec!["document", "pdf"]),
            ("image1.jpg", vec!["image", "photo"]),
            ("video1.mp4", vec!["video", "media"]),
            ("document2.pdf", vec!["document", "pdf"]),
            ("image2.png", vec!["image", "photo"]),
        ];
        
        // Upload all files
        for (file_name, tags) in &file_categories {
            let file_content = format!("Content of {}", file_name);
            let encrypted_content = env.key_manager.encrypt(file_content.as_bytes())?;
            let file_key = env.storage_manager.store_file(file_name, &encrypted_content)?;
            
            let tag_strings: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
            env.db.store_file(
                file_name,
                &file_key,
                file_name,
                file_content.len() as u64,
                Local::now(),
                &tag_strings,
                "test_public_key",
            )?;
        }
        
        // Test searching by different tags
        let document_files = env.db.search_files_by_tag("document")?;
        assert_eq!(document_files.len(), 2);
        
        let image_files = env.db.search_files_by_tag("image")?;
        assert_eq!(image_files.len(), 2);
        
        let video_files = env.db.search_files_by_tag("video")?;
        assert_eq!(video_files.len(), 1);
        
        // Test that all files are present
        let all_files = env.db.list_files()?;
        assert_eq!(all_files.len(), 5);
        
        println!("✅ File search and filtering test passed");
        Ok(())
    }
    
    #[test]
    fn test_billing_integration() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Create user with premium account
        let mut user = env.create_test_user("premium@example.com")?;
        
        // Create subscription tier
        let premium_tier = SubscriptionTier {
            name: "Premium".to_string(),
            price_monthly: 9.99,
            storage_gb: 100,
            bandwidth_gb: 1000,
            api_calls_hour: 10000,
            support_level: "Priority".to_string(),
        };
        
        // Create billing cycle
        let billing_cycle = BillingCycle {
            cycle_id: uuid::Uuid::new_v4(),
            start_date: Local::now(),
            end_date: Local::now() + chrono::Duration::days(30),
            amount_due: premium_tier.price_monthly,
            amount_paid: 0.0,
            status: "pending".to_string(),
        };
        
        // Update user account type
        user.account_type = AccountType::Premium {
            storage_gb: premium_tier.storage_gb,
            bandwidth_gb_month: premium_tier.bandwidth_gb,
            api_calls_hour: premium_tier.api_calls_hour,
        };
        
        // Test that premium user can store more files
        let large_file_content = vec![0u8; 1024 * 1024]; // 1MB
        let encrypted_content = env.key_manager.encrypt(&large_file_content)?;
        let file_key = env.storage_manager.store_file("premium_large_file.bin", &encrypted_content)?;
        
        env.db.store_file(
            "premium_large_file.bin",
            &file_key,
            "premium_large_file.bin",
            large_file_content.len() as u64,
            Local::now(),
            &vec!["premium".to_string()],
            &user.public_key,
        )?;
        
        // Verify file was stored successfully
        let file_entry = env.db.get_file_by_name("premium_large_file.bin")?;
        assert!(file_entry.is_some());
        
        println!("✅ Billing integration test passed");
        Ok(())
    }
    
    #[test]
    fn test_error_recovery() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Test recovery from storage errors
        let file_content = b"Test content for error recovery";
        let encrypted_content = env.key_manager.encrypt(file_content)?;
        
        // Store file successfully
        let file_key = env.storage_manager.store_file("error_test.txt", &encrypted_content)?;
        
        // Store metadata
        env.db.store_file(
            "error_test.txt",
            &file_key,
            "error_test.txt",
            file_content.len() as u64,
            Local::now(),
            &vec!["error_test".to_string()],
            "test_public_key",
        )?;
        
        // Verify file can be retrieved
        let retrieved_encrypted = env.storage_manager.get_file(&file_key)?;
        let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
        assert_eq!(file_content, retrieved_content.as_slice());
        
        // Test handling of non-existent file
        let result = env.storage_manager.get_file("nonexistent_key");
        assert!(result.is_err());
        
        // Test database recovery
        let nonexistent_file = env.db.get_file_by_name("nonexistent_file.txt")?;
        assert!(nonexistent_file.is_none());
        
        println!("✅ Error recovery test passed");
        Ok(())
    }
    
    #[test]
    fn test_performance_integration() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Test end-to-end performance
        let start = std::time::Instant::now();
        
        let num_operations = 100;
        for i in 0..num_operations {
            let file_content = format!("Performance test content {}", i);
            let file_name = format!("perf_test_{}.txt", i);
            
            // Full workflow: encrypt, store, save metadata
            let encrypted_content = env.key_manager.encrypt(file_content.as_bytes())?;
            let file_key = env.storage_manager.store_file(&file_name, &encrypted_content)?;
            
            env.db.store_file(
                &file_name,
                &file_key,
                &file_name,
                file_content.len() as u64,
                Local::now(),
                &vec!["performance".to_string()],
                "test_public_key",
            )?;
        }
        
        let storage_duration = start.elapsed();
        
        // Test retrieval performance
        let retrieval_start = std::time::Instant::now();
        
        let all_files = env.db.list_files()?;
        assert_eq!(all_files.len(), num_operations);
        
        // Retrieve first 10 files
        for i in 0..10 {
            let file_name = format!("perf_test_{}.txt", i);
            if let Some(file_entry) = env.db.get_file_by_name(&file_name)? {
                let retrieved_encrypted = env.storage_manager.get_file(&file_entry.file_key)?;
                let _retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
            }
        }
        
        let retrieval_duration = retrieval_start.elapsed();
        
        println!("Performance test results:");
        println!("  Storage ({} files): {:?}", num_operations, storage_duration);
        println!("  Retrieval (10 files): {:?}", retrieval_duration);
        
        // Performance assertions
        assert!(storage_duration < Duration::from_secs(30), "Storage took too long");
        assert!(retrieval_duration < Duration::from_secs(5), "Retrieval took too long");
        
        println!("✅ Performance integration test passed");
        Ok(())
    }
    
    #[test]
    fn test_system_limits() -> Result<()> {
        let env = IntegrationTestEnv::new()?;
        
        // Test system behavior at limits
        let user = env.create_test_user("limits@example.com")?;
        
        // Test file size limits (simulate)
        let reasonable_file_size = 1024 * 1024; // 1MB
        let large_file_content = vec![0u8; reasonable_file_size];
        
        let encrypted_content = env.key_manager.encrypt(&large_file_content)?;
        let file_key = env.storage_manager.store_file("limit_test.bin", &encrypted_content)?;
        
        env.db.store_file(
            "limit_test.bin",
            &file_key,
            "limit_test.bin",
            large_file_content.len() as u64,
            Local::now(),
            &vec!["limit_test".to_string()],
            &user.public_key,
        )?;
        
        // Test that file was stored and can be retrieved
        let file_entry = env.db.get_file_by_name("limit_test.bin")?;
        assert!(file_entry.is_some());
        
        let retrieved_encrypted = env.storage_manager.get_file(&file_key)?;
        let retrieved_content = env.key_manager.decrypt(&retrieved_encrypted)?;
        assert_eq!(large_file_content, retrieved_content);
        
        println!("✅ System limits test passed");
        Ok(())
    }
}
