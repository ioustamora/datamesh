/// Enhanced Integration Tests for DataMesh System
///
/// This module provides end-to-end integration tests that validate the complete
/// system functionality including network operations, data flow, and cross-module interactions.

mod test_utils;

use anyhow::Result;
use std::time::Duration;
use std::sync::Arc;
use test_utils::{TestEnvironment, assertions, mock_data, performance};
use tempfile::TempDir;
use tokio::time::timeout;

// Import DataMesh modules
use datamesh::{
    database::DatabaseManager,
    economics::{EconomicConfig, EconomicModel},
    governance::{NetworkGovernance, AccountType, UserAccount},
    key_manager::KeyManager,
    network::{NetworkConfig, NetworkManager},
    storage::{StorageManager, FileMetadata},
};

/// Test the complete file upload and retrieval workflow
#[tokio::test]
async fn test_complete_file_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Create test components
    let km = KeyManager::new()?;
    let storage = StorageManager::new(&env.storage_path)?;
    
    // Test data of various sizes
    let test_files = vec![
        ("small.txt", b"Small test file content".to_vec()),
        ("medium.txt", vec![b'M'; 1024]), // 1KB
        ("large.txt", vec![b'L'; 1024 * 100]), // 100KB
        ("binary.bin", (0..256).collect::<Vec<u8>>().repeat(4)), // Binary data
    ];
    
    let mut file_keys = Vec::new();
    
    // Upload phase
    let upload_perf = performance::PerformanceTest::new("complete_upload_workflow");
    for (filename, content) in &test_files {
        // Encrypt the content
        let encrypted_content = km.encrypt(content)?;
        
        // Store in database and storage
        let file_key = format!("key_{}", filename);
        let metadata = FileMetadata {
            name: filename.to_string(),
            size: content.len() as u64,
            content_type: mime_guess::from_path(filename).first_or_octet_stream().to_string(),
            checksum: sha256::digest(content),
            encrypted: true,
        };
        
        storage.store_file(&file_key, &encrypted_content, &metadata).await?;
        
        let upload_time = chrono::Local::now();
        let tags = vec!["integration".to_string(), "test".to_string()];
        
        env.db.store_file(
            filename,
            &file_key,
            filename,
            content.len() as u64,
            upload_time,
            &tags,
            "test_public_key",
        )?;
        
        file_keys.push((file_key, content.clone()));
    }
    upload_perf.finish(Duration::from_secs(10));
    
    // Verification phase
    let verify_perf = performance::PerformanceTest::new("complete_verification_workflow");
    for (file_key, original_content) in &file_keys {
        // Retrieve from storage
        let retrieved_encrypted = storage.retrieve_file(file_key).await?;
        
        // Decrypt content
        let decrypted_content = km.decrypt(&retrieved_encrypted)?;
        
        // Verify content integrity
        assert_eq!(&decrypted_content, original_content, "Content mismatch for key: {}", file_key);
        
        // Verify database entry
        let db_entry = env.db.get_file_by_key(file_key)?;
        assert!(db_entry.is_some(), "Database entry should exist for key: {}", file_key);
        
        let entry = db_entry.unwrap();
        assert_eq!(entry.file_size, original_content.len() as u64);
        assert_eq!(entry.file_key, *file_key);
    }
    verify_perf.finish(Duration::from_secs(5));
    
    // Cleanup verification
    for (file_key, _) in &file_keys {
        storage.delete_file(file_key).await?;
        
        // Verify deletion
        let result = storage.retrieve_file(file_key).await;
        assert!(result.is_err(), "File should be deleted from storage");
    }
    
    Ok(())
}

/// Test network configuration and peer management
#[tokio::test]
async fn test_network_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Test multiple network configurations
    let configs = vec![
        NetworkConfig {
            port: 40870,
            bootstrap_peers: vec!["/ip4/127.0.0.1/tcp/40871".to_string()],
            max_connections: 50,
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
        },
        NetworkConfig {
            port: 40871,
            bootstrap_peers: vec!["/ip4/127.0.0.1/tcp/40870".to_string()],
            max_connections: 100,
            connection_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(15),
        },
    ];
    
    for (i, config) in configs.iter().enumerate() {
        let node_dir = temp_dir.path().join(format!("node_{}", i));
        std::fs::create_dir_all(&node_dir)?;
        
        // Test network manager creation
        let network_manager = NetworkManager::new(config.clone(), &node_dir).await?;
        
        // Test configuration validation
        assert_eq!(network_manager.get_local_port(), config.port);
        assert_eq!(network_manager.get_max_connections(), config.max_connections);
        
        // Test peer list management
        let initial_peers = network_manager.get_connected_peers().await;
        assert!(initial_peers.is_empty(), "Should start with no connected peers");
        
        // Test network information
        let network_info = network_manager.get_network_info().await;
        assert!(!network_info.local_peer_id.is_empty(), "Should have local peer ID");
        assert!(network_info.listening_addresses.len() > 0, "Should have listening addresses");
        
        // Clean shutdown
        network_manager.shutdown().await?;
    }
    
    Ok(())
}

/// Test economic model integration with governance
#[tokio::test]
async fn test_economics_governance_integration() -> Result<()> {
    let economic_model = EconomicModel::new();
    let governance = NetworkGovernance::new();
    let config = EconomicConfig::default();
    
    // Create test users with different account types
    let users = vec![
        UserAccount {
            user_id: "free_user".to_string(),
            email: "free@test.com".to_string(),
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 100,
                api_calls_hour: 1000,
            },
            is_active: true,
            verification_status: datamesh::governance::VerificationStatus::EmailVerified,
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        },
        UserAccount {
            user_id: "premium_user".to_string(),
            email: "premium@test.com".to_string(),
            account_type: AccountType::Premium {
                storage_gb: 100,
                bandwidth_gb_month: 1000,
                api_calls_hour: 10000,
            },
            is_active: true,
            verification_status: datamesh::governance::VerificationStatus::KYCVerified,
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        },
        UserAccount {
            user_id: "enterprise_user".to_string(),
            email: "enterprise@test.com".to_string(),
            account_type: AccountType::Enterprise {
                storage_unlimited: true,
                bandwidth_unlimited: true,
                api_calls_unlimited: true,
                sla_guarantee: 0.999,
            },
            is_active: true,
            verification_status: datamesh::governance::VerificationStatus::KYCVerified,
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        },
    ];
    
    // Test cost calculations for different account types
    let usage_scenarios = vec![
        (1.0, 1.0),    // Light usage
        (10.0, 10.0),  // Medium usage
        (100.0, 100.0), // Heavy usage
    ];
    
    for user in &users {
        for (storage_gb, bandwidth_gb) in &usage_scenarios {
            // Calculate costs
            let storage_cost = storage_gb * config.storage_cost_per_gb_month;
            let bandwidth_cost = bandwidth_gb * config.bandwidth_cost_per_gb;
            let total_cost = storage_cost + bandwidth_cost;
            
            // Test account limits
            match &user.account_type {
                AccountType::Free { storage_gb: limit_storage, bandwidth_gb_month: limit_bandwidth, .. } => {
                    if *storage_gb > *limit_storage as f64 {
                        // Should exceed free tier limits
                        assert!(total_cost > 0.0, "Should incur costs when exceeding free tier");
                    }
                }
                AccountType::Premium { storage_gb: limit_storage, bandwidth_gb_month: limit_bandwidth, .. } => {
                    if *storage_gb > *limit_storage as f64 || *bandwidth_gb > *limit_bandwidth as f64 {
                        // Should have premium pricing
                        assert!(total_cost > 0.0, "Should incur costs for premium usage");
                    }
                }
                AccountType::Enterprise { .. } => {
                    // Enterprise accounts may have different pricing models
                    // Cost could be 0 for unlimited accounts or based on negotiated rates
                }
            }
            
            // Test billing calculation
            let monthly_bill = economic_model.calculate_monthly_cost(
                *storage_gb,
                *bandwidth_gb,
                &user.account_type,
            );
            assert!(monthly_bill >= 0.0, "Monthly bill should be non-negative");
        }
    }
    
    Ok(())
}

/// Test data consistency across system restarts
#[tokio::test]
async fn test_persistence_and_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let storage_path = temp_dir.path().join("storage");
    let key_path = temp_dir.path().join("keys");
    
    // Initial data setup
    let test_data = vec![
        ("persistent_file_1", b"First persistent file".to_vec()),
        ("persistent_file_2", b"Second persistent file".to_vec()),
        ("persistent_file_3", b"Third persistent file".to_vec()),
    ];
    
    let mut file_keys = Vec::new();
    
    // Phase 1: Create and populate system
    {
        let db = DatabaseManager::new(&db_path)?;
        let storage = StorageManager::new(&storage_path)?;
        let km = KeyManager::new_with_path(&key_path)?;
        
        for (filename, content) in &test_data {
            let encrypted_content = km.encrypt(content)?;
            let file_key = format!("persistent_key_{}", filename);
            
            // Store in both storage and database
            let metadata = FileMetadata {
                name: filename.to_string(),
                size: content.len() as u64,
                content_type: "text/plain".to_string(),
                checksum: sha256::digest(content),
                encrypted: true,
            };
            
            storage.store_file(&file_key, &encrypted_content, &metadata).await?;
            
            let upload_time = chrono::Local::now();
            let tags = vec!["persistent".to_string()];
            
            db.store_file(
                filename,
                &file_key,
                filename,
                content.len() as u64,
                upload_time,
                &tags,
                "test_public_key",
            )?;
            
            file_keys.push((file_key, content.clone()));
        }
        
        // Save key manager state
        km.save_to_file(&key_path)?;
    } // All components go out of scope here
    
    // Phase 2: Restart system and verify data persistence
    {
        let db = DatabaseManager::new(&db_path)?;
        let storage = StorageManager::new(&storage_path)?;
        let km = KeyManager::load_from_file(&key_path)?;
        
        // Verify all data is still accessible
        for (file_key, original_content) in &file_keys {
            // Check database entry
            let db_entry = db.get_file_by_key(file_key)?;
            assert!(db_entry.is_some(), "Database entry should persist for key: {}", file_key);
            
            // Check storage content
            let encrypted_content = storage.retrieve_file(file_key).await?;
            let decrypted_content = km.decrypt(&encrypted_content)?;
            
            assert_eq!(&decrypted_content, original_content, 
                      "Content should be identical after restart for key: {}", file_key);
        }
        
        // Verify database statistics
        let stats = db.get_stats()?;
        assert_eq!(stats.total_files, test_data.len() as u64, "File count should persist");
        
        // Test search functionality still works
        let search_results = db.search_files("persistent")?;
        assert_eq!(search_results.len(), test_data.len(), "Search should find all persistent files");
    }
    
    Ok(())
}

/// Test concurrent access from multiple clients
#[tokio::test]
async fn test_concurrent_multi_client_access() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Simulate multiple clients accessing the system concurrently
    let client_count = 5;
    let operations_per_client = 10;
    
    let mut client_handles = Vec::new();
    
    for client_id in 0..client_count {
        let db_path = env.db_path.clone();
        let storage_path = env.storage_path.clone();
        
        let handle = tokio::spawn(async move {
            let db = DatabaseManager::new(&db_path)?;
            let storage = StorageManager::new(&storage_path)?;
            let km = KeyManager::new()?;
            
            let mut client_results = Vec::new();
            
            for op_id in 0..operations_per_client {
                let filename = format!("client_{}_file_{}", client_id, op_id);
                let content = format!("Content from client {} operation {}", client_id, op_id).into_bytes();
                let file_key = format!("client_{}_key_{}", client_id, op_id);
                
                // Store operation
                let encrypted_content = km.encrypt(&content)?;
                let metadata = FileMetadata {
                    name: filename.clone(),
                    size: content.len() as u64,
                    content_type: "text/plain".to_string(),
                    checksum: sha256::digest(&content),
                    encrypted: true,
                };
                
                storage.store_file(&file_key, &encrypted_content, &metadata).await?;
                
                let upload_time = chrono::Local::now();
                let tags = vec![format!("client_{}", client_id)];
                
                db.store_file(
                    &filename,
                    &file_key,
                    &filename,
                    content.len() as u64,
                    upload_time,
                    &tags,
                    "test_public_key",
                )?;
                
                // Verify operation
                let retrieved_encrypted = storage.retrieve_file(&file_key).await?;
                let decrypted = km.decrypt(&retrieved_encrypted)?;
                
                assert_eq!(decrypted, content, "Content verification failed for client {} op {}", client_id, op_id);
                
                client_results.push((filename, file_key));
                
                // Small delay to simulate real usage patterns
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            Ok::<_, anyhow::Error>(client_results)
        });
        
        client_handles.push(handle);
    }
    
    // Wait for all clients to complete
    let mut all_results = Vec::new();
    for handle in client_handles {
        let client_results = handle.await??;
        all_results.extend(client_results);
    }
    
    // Verify total operation count
    assert_eq!(all_results.len(), client_count * operations_per_client, 
               "Should have results from all client operations");
    
    // Verify data consistency
    let all_files = env.db.list_files(None)?;
    assert!(all_files.len() >= all_results.len(), 
            "Database should contain at least all client files");
    
    // Test cross-client search
    for client_id in 0..client_count {
        let client_tag = format!("client_{}", client_id);
        let client_files = env.db.list_files(Some(&client_tag))?;
        assert_eq!(client_files.len(), operations_per_client, 
                   "Should find all files for client {}", client_id);
    }
    
    Ok(())
}

/// Test system behavior under stress conditions
#[tokio::test]
async fn test_system_stress_conditions() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test large file handling
    let large_file_size = 1024 * 1024; // 1MB
    let large_content = vec![b'X'; large_file_size];
    
    let stress_perf = performance::PerformanceTest::new("stress_large_file");
    
    let km = KeyManager::new()?;
    let storage = StorageManager::new(&env.storage_path)?;
    
    // Test encryption of large file
    let encrypted_large = km.encrypt(&large_content)?;
    assert!(encrypted_large.len() > large_file_size, "Encrypted content should be larger");
    
    // Test storage of large file
    let large_file_key = "stress_large_file_key";
    let metadata = FileMetadata {
        name: "large_stress_file.bin".to_string(),
        size: large_content.len() as u64,
        content_type: "application/octet-stream".to_string(),
        checksum: sha256::digest(&large_content),
        encrypted: true,
    };
    
    storage.store_file(large_file_key, &encrypted_large, &metadata).await?;
    
    // Test retrieval and decryption
    let retrieved_large = storage.retrieve_file(large_file_key).await?;
    let decrypted_large = km.decrypt(&retrieved_large)?;
    
    assert_eq!(decrypted_large.len(), large_content.len(), "Large file size should match");
    assert_eq!(decrypted_large, large_content, "Large file content should match");
    
    stress_perf.finish(Duration::from_secs(30)); // Allow reasonable time for large file operations
    
    // Test rapid small operations
    let rapid_perf = performance::PerformanceTest::new("stress_rapid_operations");
    
    for i in 0..100 {
        let small_content = format!("Rapid operation {}", i).into_bytes();
        let small_key = format!("rapid_key_{}", i);
        
        let encrypted = km.encrypt(&small_content)?;
        let decrypted = km.decrypt(&encrypted)?;
        
        assert_eq!(decrypted, small_content, "Rapid operation {} failed", i);
    }
    
    rapid_perf.finish(Duration::from_secs(5)); // Should be fast for small operations
    
    // Test memory usage with many files
    let mut file_keys = Vec::new();
    for i in 0..50 {
        let content = format!("Memory test file {}", i).into_bytes();
        let file_key = format!("memory_test_key_{}", i);
        
        let encrypted = km.encrypt(&content)?;
        let metadata = FileMetadata {
            name: format!("memory_test_{}.txt", i),
            size: content.len() as u64,
            content_type: "text/plain".to_string(),
            checksum: sha256::digest(&content),
            encrypted: true,
        };
        
        storage.store_file(&file_key, &encrypted, &metadata).await?;
        file_keys.push(file_key);
    }
    
    // Verify all files are still accessible
    for file_key in &file_keys {
        let retrieved = storage.retrieve_file(file_key).await?;
        assert!(!retrieved.is_empty(), "File should be retrievable: {}", file_key);
    }
    
    // Cleanup
    for file_key in &file_keys {
        storage.delete_file(file_key).await?;
    }
    storage.delete_file(large_file_key).await?;
    
    Ok(())
}

/// Test error recovery and resilience
#[tokio::test]
async fn test_error_recovery_resilience() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test database corruption recovery
    {
        // Store some data first
        let test_files = env.add_test_files(3)?;
        
        // Verify data exists
        let initial_files = env.db.list_files(None)?;
        assert_eq!(initial_files.len(), 3, "Should have 3 initial files");
        
        // Test that database can handle invalid queries gracefully
        let invalid_search = env.db.search_files("''; DROP TABLE files; --")?;
        assert!(invalid_search.is_empty(), "SQL injection attempt should return empty results");
        
        // Verify data is still intact after injection attempt
        let post_injection_files = env.db.list_files(None)?;
        assert_eq!(post_injection_files.len(), 3, "Files should still exist after injection attempt");
    }
    
    // Test network timeout handling
    {
        let config = NetworkConfig {
            port: 40879,
            bootstrap_peers: vec!["/ip4/192.0.2.1/tcp/40880".to_string()], // Non-routable IP
            max_connections: 10,
            connection_timeout: Duration::from_millis(100), // Very short timeout
            heartbeat_interval: Duration::from_millis(50),
        };
        
        let temp_dir = TempDir::new()?;
        let network_manager = NetworkManager::new(config, temp_dir.path()).await?;
        
        // Should handle connection failures gracefully
        let network_info = network_manager.get_network_info().await;
        assert!(!network_info.local_peer_id.is_empty(), "Should have local peer ID even with failed connections");
        
        network_manager.shutdown().await?;
    }
    
    // Test storage error recovery
    {
        let storage = StorageManager::new(&env.storage_path)?;
        
        // Test handling of non-existent file retrieval
        let result = storage.retrieve_file("non_existent_key").await;
        assert!(result.is_err(), "Should error when retrieving non-existent file");
        
        // Test that storage remains functional after error
        let km = KeyManager::new()?;
        let test_content = b"Recovery test content";
        let encrypted = km.encrypt(test_content)?;
        
        let metadata = FileMetadata {
            name: "recovery_test.txt".to_string(),
            size: test_content.len() as u64,
            content_type: "text/plain".to_string(),
            checksum: sha256::digest(test_content),
            encrypted: true,
        };
        
        storage.store_file("recovery_test_key", &encrypted, &metadata).await?;
        let retrieved = storage.retrieve_file("recovery_test_key").await?;
        let decrypted = km.decrypt(&retrieved)?;
        
        assert_eq!(decrypted, test_content, "Storage should work normally after error");
    }
    
    Ok(())
}

/// Test system limits and boundaries
#[tokio::test] 
async fn test_system_limits() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test filename length limits
    let long_filename = "a".repeat(1000);
    let upload_time = chrono::Local::now();
    let tags = vec!["limit_test".to_string()];
    
    let result = env.db.store_file(
        &long_filename,
        "long_filename_key",
        &long_filename,
        1024,
        upload_time,
        &tags,
        "test_public_key",
    );
    
    // Should either succeed or fail gracefully
    match result {
        Ok(_) => {
            // If it succeeds, verify retrieval works
            let retrieved = env.db.get_file_by_name(&long_filename)?;
            assert!(retrieved.is_some(), "Long filename should be retrievable if stored");
        }
        Err(_) => {
            // Graceful failure is also acceptable
        }
    }
    
    // Test many tags
    let many_tags: Vec<String> = (0..100).map(|i| format!("tag_{}", i)).collect();
    let many_tags_result = env.db.store_file(
        "many_tags_file",
        "many_tags_key",
        "many_tags.txt",
        1024,
        upload_time,
        &many_tags,
        "test_public_key",
    );
    
    // Should handle many tags gracefully
    assert!(many_tags_result.is_ok(), "Should handle many tags gracefully");
    
    // Test empty inputs
    let empty_result = env.db.store_file(
        "",
        "empty_name_key",
        "",
        0,
        upload_time,
        &vec![],
        "test_public_key",
    );
    
    // Should handle empty inputs appropriately
    match empty_result {
        Ok(_) => {
            // If allowed, verify it's retrievable
            let retrieved = env.db.get_file_by_key("empty_name_key")?;
            assert!(retrieved.is_some(), "Empty name file should be retrievable if stored");
        }
        Err(_) => {
            // Rejecting empty names is also valid
        }
    }
    
    Ok(())
}

/// Test end-to-end user scenarios
#[tokio::test]
async fn test_end_to_end_user_scenarios() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Scenario 1: New user onboarding
    let new_user = mock_data::create_test_user("new_user_123");
    assert!(new_user.is_active, "New user should be active");
    
    // Scenario 2: File management workflow
    let km = KeyManager::new()?;
    let storage = StorageManager::new(&env.storage_path)?;
    
    // User uploads multiple files
    let user_files = vec![
        ("document.pdf", b"PDF document content".to_vec()),
        ("image.jpg", b"JPEG image data".to_vec()),
        ("data.csv", b"CSV,data,file\n1,2,3".to_vec()),
    ];
    
    let mut user_file_keys = Vec::new();
    
    for (filename, content) in &user_files {
        let encrypted_content = km.encrypt(content)?;
        let file_key = format!("user_123_{}", filename);
        
        let metadata = FileMetadata {
            name: filename.to_string(),
            size: content.len() as u64,
            content_type: mime_guess::from_path(filename).first_or_octet_stream().to_string(),
            checksum: sha256::digest(content),
            encrypted: true,
        };
        
        storage.store_file(&file_key, &encrypted_content, &metadata).await?;
        
        let upload_time = chrono::Local::now();
        let tags = vec!["user_123".to_string(), "personal".to_string()];
        
        env.db.store_file(
            filename,
            &file_key,
            filename,
            content.len() as u64,
            upload_time,
            &tags,
            &new_user.user_id,
        )?;
        
        user_file_keys.push(file_key);
    }
    
    // User searches their files
    let user_files_list = env.db.list_files(Some("user_123"))?;
    assert_eq!(user_files_list.len(), 3, "User should see all their files");
    
    // User downloads a specific file
    let pdf_files = env.db.search_files("document.pdf")?;
    assert_eq!(pdf_files.len(), 1, "Should find the PDF document");
    
    let pdf_file = &pdf_files[0];
    let pdf_encrypted = storage.retrieve_file(&pdf_file.file_key).await?;
    let pdf_decrypted = km.decrypt(&pdf_encrypted)?;
    
    assert_eq!(pdf_decrypted, b"PDF document content", "PDF content should match");
    
    // User deletes a file
    let csv_files = env.db.search_files("data.csv")?;
    assert_eq!(csv_files.len(), 1, "Should find the CSV file");
    
    let csv_file = &csv_files[0];
    env.db.delete_file(&csv_file.name)?;
    storage.delete_file(&csv_file.file_key).await?;
    
    // Verify deletion
    let remaining_files = env.db.list_files(Some("user_123"))?;
    assert_eq!(remaining_files.len(), 2, "Should have 2 files after deletion");
    
    // Scenario 3: User account upgrade
    let economic_model = EconomicModel::new();
    
    // Calculate costs for current usage
    let current_usage_cost = economic_model.calculate_monthly_cost(
        2.0, // 2GB storage
        5.0, // 5GB bandwidth
        &new_user.account_type,
    );
    
    // Simulate upgrade to premium
    let premium_account = AccountType::Premium {
        storage_gb: 100,
        bandwidth_gb_month: 1000,
        api_calls_hour: 10000,
    };
    
    let premium_cost = economic_model.calculate_monthly_cost(
        2.0, // Same usage
        5.0,
        &premium_account,
    );
    
    // Premium might have different pricing structure
    assert!(premium_cost >= 0.0, "Premium cost should be valid");
    
    Ok(())
}

/// Test realistic multi-user collaboration workflow
#[tokio::test]
async fn test_collaborative_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let km = KeyManager::new()?;
    let storage = StorageManager::new(&env.storage_path)?;
    
    // Create team members
    let team_lead = mock_data::create_test_user("team_lead");
    let developer1 = mock_data::create_test_user("developer1");
    let developer2 = mock_data::create_test_user("developer2");
    
    // Team lead uploads project files
    let project_files = vec![
        ("README.md", b"# Project Documentation\nThis is a collaborative project.".to_vec()),
        ("config.json", br#"{"version": "1.0", "features": ["collaboration"]}"#.to_vec()),
        ("data.txt", b"Shared project data for team collaboration".to_vec()),
    ];
    
    let mut shared_file_keys = Vec::new();
    
    for (filename, content) in &project_files {
        let encrypted_content = km.encrypt(content)?;
        let file_key = format!("project_{}", filename);
        
        let metadata = FileMetadata {
            name: filename.to_string(),
            size: content.len() as u64,
            content_type: mime_guess::from_path(filename).first_or_octet_stream().to_string(),
            checksum: sha256::digest(content),
            encrypted: true,
        };
        
        storage.store_file(&file_key, &encrypted_content, &metadata).await?;
        
        let upload_time = chrono::Local::now();
        let tags = vec!["project".to_string(), "shared".to_string(), "team".to_string()];
        
        env.db.store_file(
            filename,
            &file_key,
            filename,
            content.len() as u64,
            upload_time,
            &tags,
            &team_lead.user_id,
        )?;
        
        shared_file_keys.push(file_key);
    }
    
    // Team members access shared files
    let shared_files = env.db.search_files("project")?;
    assert_eq!(shared_files.len(), 3, "All team members should see shared project files");
    
    // Developer1 modifies and uploads new version
    let readme_files = env.db.search_files("README.md")?;
    assert_eq!(readme_files.len(), 1, "Should find README file");
    
    let original_readme = &readme_files[0];
    let readme_encrypted = storage.retrieve_file(&original_readme.file_key).await?;
    let readme_content = km.decrypt(&readme_encrypted)?;
    
    // Developer1 adds to README
    let updated_readme = [
        readme_content.as_slice(),
        b"\n\n## Developer1 Contributions\n- Added feature X\n- Fixed bug Y"
    ].concat();
    
    let updated_encrypted = km.encrypt(&updated_readme)?;
    let updated_key = "project_README_v2.md";
    
    let updated_metadata = FileMetadata {
        name: "README_v2.md".to_string(),
        size: updated_readme.len() as u64,
        content_type: "text/markdown".to_string(),
        checksum: sha256::digest(&updated_readme),
        encrypted: true,
    };
    
    storage.store_file(updated_key, &updated_encrypted, &updated_metadata).await?;
    
    let upload_time = chrono::Local::now();
    let tags = vec!["project".to_string(), "shared".to_string(), "updated".to_string()];
    
    env.db.store_file(
        "README_v2.md",
        updated_key,
        "README_v2.md",
        updated_readme.len() as u64,
        upload_time,
        &tags,
        &developer1.user_id,
    )?;
    
    // Verify collaboration workflow
    let all_project_files = env.db.search_files("project")?;
    assert_eq!(all_project_files.len(), 4, "Should have original files plus updated README");
    
    // Developer2 can access the updated file
    let updated_files = env.db.search_files("README_v2")?;
    assert_eq!(updated_files.len(), 1, "Should find updated README");
    
    let updated_file = &updated_files[0];
    let updated_retrieved = storage.retrieve_file(&updated_file.file_key).await?;
    let final_content = km.decrypt(&updated_retrieved)?;
    
    assert!(final_content.len() > readme_content.len(), "Updated README should be longer");
    assert!(String::from_utf8_lossy(&final_content).contains("Developer1 Contributions"), 
            "Should contain developer1's additions");
    
    println!("✅ Collaborative workflow test completed successfully");
    Ok(())
}

/// Test real-world performance scenarios
#[tokio::test]
async fn test_realistic_performance_scenarios() -> Result<()> {
    let env = TestEnvironment::new()?;
    let km = KeyManager::new()?;
    let storage = StorageManager::new(&env.storage_path)?;
    
    // Scenario 1: Photo backup workflow (realistic user scenario)
    let photo_perf = performance::PerformanceTest::new("photo_backup_workflow");
    
    let photo_sizes = vec![
        ("thumbnail.jpg", 50 * 1024),      // 50KB
        ("medium.jpg", 500 * 1024),        // 500KB  
        ("high_res.jpg", 2 * 1024 * 1024), // 2MB
        ("raw_photo.cr2", 25 * 1024 * 1024), // 25MB
    ];
    
    for (photo_name, size) in &photo_sizes {
        let photo_content = vec![0xFFu8; *size]; // Simulate binary photo data
        let encrypted_photo = km.encrypt(&photo_content)?;
        let photo_key = format!("photo_{}", photo_name);
        
        let metadata = FileMetadata {
            name: photo_name.to_string(),
            size: *size as u64,
            content_type: "image/jpeg".to_string(),
            checksum: sha256::digest(&photo_content),
            encrypted: true,
        };
        
        storage.store_file(&photo_key, &encrypted_photo, &metadata).await?;
        
        let upload_time = chrono::Local::now();
        let tags = vec!["photos".to_string(), "backup".to_string()];
        
        env.db.store_file(
            photo_name,
            &photo_key,
            photo_name,
            *size as u64,
            upload_time,
            &tags,
            "photo_user",
        )?;
    }
    
    photo_perf.finish(Duration::from_secs(30)); // Allow reasonable time for photo processing
    
    // Scenario 2: Document management workflow
    let doc_perf = performance::PerformanceTest::new("document_management_workflow");
    
    // Simulate uploading a batch of business documents
    for i in 0..20 {
        let doc_content = format!("Business document {} content with important data and information that would typically be found in a real document.", i).repeat(100);
        let doc_bytes = doc_content.into_bytes();
        let encrypted_doc = km.encrypt(&doc_bytes)?;
        let doc_key = format!("business_doc_{}", i);
        
        let metadata = FileMetadata {
            name: format!("document_{}.txt", i),
            size: doc_bytes.len() as u64,
            content_type: "text/plain".to_string(),
            checksum: sha256::digest(&doc_bytes),
            encrypted: true,
        };
        
        storage.store_file(&doc_key, &encrypted_doc, &metadata).await?;
        
        let upload_time = chrono::Local::now();
        let tags = vec!["documents".to_string(), "business".to_string()];
        
        env.db.store_file(
            &format!("document_{}.txt", i),
            &doc_key,
            &format!("document_{}.txt", i),
            doc_bytes.len() as u64,
            upload_time,
            &tags,
            "business_user",
        )?;
    }
    
    doc_perf.finish(Duration::from_secs(15)); // Should be efficient for text documents
    
    // Scenario 3: Search and retrieval performance
    let search_perf = performance::PerformanceTest::new("search_retrieval_performance");
    
    // Test various search patterns
    let search_terms = vec!["photos", "business", "document", "backup"];
    
    for search_term in &search_terms {
        let search_results = env.db.search_files(search_term)?;
        assert!(!search_results.is_empty(), "Should find files for search term: {}", search_term);
        
        // Test retrieval of first result
        if let Some(first_result) = search_results.first() {
            let retrieved_encrypted = storage.retrieve_file(&first_result.file_key).await?;
            let decrypted = km.decrypt(&retrieved_encrypted)?;
            assert!(!decrypted.is_empty(), "Retrieved content should not be empty");
        }
    }
    
    search_perf.finish(Duration::from_secs(5)); // Search should be fast
    
    // Verify total system state
    let all_files = env.db.list_files(None)?;
    assert!(all_files.len() >= 24, "Should have photos + documents stored"); // 4 photos + 20 documents
    
    println!("✅ Realistic performance scenarios completed successfully");
    Ok(())
}

/// Test system behavior under realistic load patterns
#[tokio::test]  
async fn test_realistic_load_patterns() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Simulate realistic user behavior patterns
    let user_scenarios = vec![
        ("casual_user", 5, 100),      // 5 files, small sizes
        ("power_user", 25, 1000),     // 25 files, medium sizes  
        ("business_user", 50, 500),   // 50 files, mixed sizes
    ];
    
    let mut scenario_handles = Vec::new();
    
    for (user_type, file_count, size_multiplier) in user_scenarios {
        let db_path = env.db_path.clone();
        let storage_path = env.storage_path.clone();
        
        let handle = tokio::spawn(async move {
            let db = DatabaseManager::new(&db_path)?;
            let storage = StorageManager::new(&storage_path)?;
            let km = KeyManager::new()?;
            
            let mut user_results = Vec::new();
            
            for i in 0..file_count {
                // Simulate realistic file sizes and types
                let (content, file_type) = match i % 4 {
                    0 => (format!("Text document {} for {}", i, user_type).repeat(size_multiplier / 10), "text/plain"),
                    1 => (vec![0x89, 0x50, 0x4E, 0x47].into_iter().chain(vec![0u8; size_multiplier]).collect(), "image/png"), // PNG header + data
                    2 => (format!(r#"{{"id": {}, "user": "{}", "data": "{}"}}"#, i, user_type, "x".repeat(size_multiplier)).into_bytes(), "application/json"),
                    _ => (vec![0xFFu8; size_multiplier], "application/octet-stream"),
                };
                
                let content_bytes = if let Ok(text) = std::str::from_utf8(&content) {
                    text.as_bytes().to_vec()
                } else {
                    content
                };
                
                let encrypted_content = km.encrypt(&content_bytes)?;
                let file_key = format!("{}_{}", user_type, i);
                
                let metadata = FileMetadata {
                    name: format!("{}_file_{}", user_type, i),
                    size: content_bytes.len() as u64,
                    content_type: file_type.to_string(),
                    checksum: sha256::digest(&content_bytes),
                    encrypted: true,
                };
                
                storage.store_file(&file_key, &encrypted_content, &metadata).await?;
                
                let upload_time = chrono::Local::now();
                let tags = vec![user_type.to_string(), "load_test".to_string()];
                
                db.store_file(
                    &format!("{}_file_{}", user_type, i),
                    &file_key,
                    &format!("{}_file_{}", user_type, i),
                    content_bytes.len() as u64,
                    upload_time,
                    &tags,
                    user_type,
                )?;
                
                // Simulate realistic delay between uploads
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                user_results.push(file_key);
            }
            
            Ok::<_, anyhow::Error>((user_type, user_results))
        });
        
        scenario_handles.push(handle);
    }
    
    // Wait for all user scenarios to complete
    let mut total_files = 0;
    for handle in scenario_handles {
        let (user_type, files) = handle.await??;
        total_files += files.len();
        println!("User scenario '{}' completed with {} files", user_type, files.len());
    }
    
    // Verify system state after load test
    let final_files = env.db.list_files(None)?;
    assert!(final_files.len() >= total_files, "Database should contain all uploaded files");
    
    // Test search performance under load
    let search_start = std::time::Instant::now();
    let casual_files = env.db.search_files("casual_user")?;
    let power_files = env.db.search_files("power_user")?; 
    let business_files = env.db.search_files("business_user")?;
    let search_duration = search_start.elapsed();
    
    assert_eq!(casual_files.len(), 5, "Should find casual user files");
    assert_eq!(power_files.len(), 25, "Should find power user files");
    assert_eq!(business_files.len(), 50, "Should find business user files");
    assert!(search_duration < Duration::from_secs(2), "Search should remain fast under load");
    
    println!("✅ Realistic load pattern test completed with {} total files in {:?}", 
             total_files, search_duration);
    Ok(())
}
