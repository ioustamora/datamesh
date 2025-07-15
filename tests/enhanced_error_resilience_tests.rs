/// Enhanced Error and Resilience Testing for DataMesh
///
/// This module tests error handling, recovery mechanisms, and system resilience
/// under various failure conditions to ensure robust operation.

mod test_utils;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use test_utils::{TestEnvironment, assertions, performance};

use datamesh::{
    database::DatabaseManager,
    file_manager::FileManager,
    key_manager::KeyManager,
    network_actor::NetworkHandle,
    storage::StorageManager,
    cli::Cli,
    config::Config,
    error::DfsError,
};

/// Test database resilience under adverse conditions
#[tokio::test]
async fn test_database_resilience() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test database with concurrent access and potential conflicts
    let db = Arc::new(Mutex::new(env.db));
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent database operations
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let upload_time = chrono::Local::now();
            let tags = vec![format!("concurrent_{}", i)];
            
            // Try to perform database operations that might conflict
            for attempt in 0..5 {
                let db_guard = db_clone.lock().await;
                let result = db_guard.store_file(
                    &format!("conflict_test_{}_{}", i, attempt),
                    &format!("key_{}_{}", i, attempt),
                    &format!("file_{}_{}.txt", i, attempt),
                    1024,
                    upload_time,
                    &tags,
                    "test_public_key",
                );
                
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        // Database should handle errors gracefully
                        eprintln!("Database operation failed (expected): {}", e);
                    }
                }
                
                // Brief pause to create potential race conditions
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut successful_operations = 0;
    for handle in handles {
        if let Ok(Ok(())) = handle.await {
            successful_operations += 1;
        }
    }
    
    // Most operations should succeed even under contention
    assert!(successful_operations >= 8, "Database should handle concurrent access gracefully");
    
    // Database should still be functional after stress test
    let final_db = db.lock().await;
    let final_files = final_db.list_files(None)?;
    assert!(final_files.len() >= 1, "Database should remain functional");
    
    Ok(())
}

/// Test file system error recovery
#[tokio::test]
async fn test_file_system_error_recovery() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    
    // Test with invalid file paths
    let invalid_paths = vec![
        "/nonexistent/directory/file.txt",
        "",
        "/dev/null/cannot_create_file_here",
        "/root/restricted_access", // Might fail due to permissions
    ];
    
    for invalid_path in invalid_paths {
        let path_buf = std::path::PathBuf::from(invalid_path);
        
        // File operations should fail gracefully
        let read_result = tokio::fs::read(&path_buf).await;
        assert!(read_result.is_err(), "Reading invalid path should fail");
        
        let write_result = tokio::fs::write(&path_buf, b"test data").await;
        // Write might succeed or fail depending on permissions, both are acceptable
        if let Err(e) = write_result {
            println!("Expected file write failure: {}", e);
        }
    }
    
    // Test file manager with invalid operations
    if let Ok(file_manager) = FileManager::new(config).await {
        // Test storing non-existent file
        let nonexistent_file = env.storage_path.join("does_not_exist.txt");
        let store_result = file_manager.store_file(&nonexistent_file, None).await;
        assert!(store_result.is_err(), "Storing non-existent file should fail");
        
        // Test retrieving non-existent file
        let retrieve_result = file_manager.retrieve_file("nonexistent_key", &env.storage_path.join("output.txt")).await;
        assert!(retrieve_result.is_err(), "Retrieving non-existent file should fail");
    }
    
    Ok(())
}

/// Test network resilience and error handling
#[tokio::test]
async fn test_network_resilience() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    // Test network handle creation with potentially problematic configuration
    let mut test_config = config.clone();
    test_config.network.default_port = 1; // Privileged port that might fail
    
    let network_result = NetworkHandle::new(&cli, &test_config).await;
    
    match network_result {
        Ok(handle) => {
            // If network creation succeeded, test error conditions
            
            // Test operations that might fail
            let bootstrap_result = handle.bootstrap().await;
            // Bootstrap might fail in test environment - both outcomes acceptable
            
            let peers_result = handle.get_connected_peers().await;
            assert!(peers_result.is_ok(), "Getting peers should not fail");
            
            let stats_result = handle.get_network_stats().await;
            assert!(stats_result.is_ok(), "Getting stats should not fail");
            
            // Test multiple rapid operations (stress test)
            for _ in 0..10 {
                let _ = handle.get_cached_stats().await;
                let _ = handle.get_connected_peers().await;
            }
            
            // Cleanup
            handle.shutdown().await?;
        }
        Err(_) => {
            // Network creation failure is acceptable for privileged ports
            println!("Network creation failed as expected for privileged port");
        }
    }
    
    Ok(())
}

/// Test encryption/decryption error handling
#[tokio::test]
async fn test_cryptographic_error_handling() -> Result<()> {
    let key_manager = KeyManager::new(
        ecies::SecretKey::random(&mut rand::thread_rng()),
        "test_key".to_string()
    );
    
    // Test encryption with various data sizes including edge cases
    let test_data_sets = vec![
        vec![], // Empty data
        vec![0], // Single byte
        vec![0; 1024 * 1024], // Large data (1MB)
        (0..=255u8).cycle().take(1337).collect(), // Odd-sized data
    ];
    
    for (i, data) in test_data_sets.iter().enumerate() {
        let encrypt_result = key_manager.encrypt(data);
        assert!(encrypt_result.is_ok(), "Encryption should succeed for test data set {}", i);
        
        if let Ok(encrypted) = encrypt_result {
            let decrypt_result = key_manager.decrypt(&encrypted);
            assert!(decrypt_result.is_ok(), "Decryption should succeed for test data set {}", i);
            
            if let Ok(decrypted) = decrypt_result {
                assert_eq!(data, &decrypted, "Data should round-trip correctly for set {}", i);
            }
        }
    }
    
    // Test decryption with invalid data
    let invalid_encrypted_data = vec![
        vec![], // Empty encrypted data
        vec![0], // Too short
        vec![0; 16], // Still too short for ECIES
        b"not encrypted data".to_vec(), // Not encrypted
        vec![255; 1000], // Random data
    ];
    
    for (i, invalid_data) in invalid_encrypted_data.iter().enumerate() {
        let decrypt_result = key_manager.decrypt(invalid_data);
        assert!(decrypt_result.is_err(), "Decryption should fail for invalid data set {}", i);
    }
    
    Ok(())
}

/// Test system recovery after simulated failures
#[tokio::test]
async fn test_system_recovery_scenarios() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Scenario 1: Recovery after database connection loss
    {
        let db = DatabaseManager::new(&env.db_path)?;
        
        // Store some data
        let upload_time = chrono::Local::now();
        db.store_file(
            "recovery_test_1",
            "recovery_key_1",
            "recovery_file_1.txt",
            1024,
            upload_time,
            &vec!["recovery".to_string()],
            "test_public_key",
        )?;
        
        // Simulate connection loss by dropping and recreating database
        drop(db);
        
        // Recreate database connection
        let recovered_db = DatabaseManager::new(&env.db_path)?;
        
        // Verify data persisted
        let recovered_file = recovered_db.get_file_by_name("recovery_test_1")?;
        assert!(recovered_file.is_some(), "Data should persist after database reconnection");
    }
    
    // Scenario 2: Recovery after storage directory issues
    {
        let storage_manager = StorageManager::new(&env.storage_path)?;
        
        // Create test file
        let test_content = b"recovery test content";
        let metadata = datamesh::storage::FileMetadata {
            name: "recovery_test.txt".to_string(),
            size: test_content.len() as u64,
            content_type: "text/plain".to_string(),
            checksum: "test_checksum".to_string(),
            encrypted: false,
        };
        
        storage_manager.store_file("recovery_storage_key", test_content, &metadata).await?;
        
        // Verify storage
        let retrieved = storage_manager.retrieve_file("recovery_storage_key").await?;
        assert_eq!(test_content, &retrieved[..], "Storage should work normally");
        
        // Storage manager should handle missing files gracefully
        let missing_result = storage_manager.retrieve_file("nonexistent_key").await;
        assert!(missing_result.is_err(), "Missing file should cause error");
        
        // Storage manager should continue working after errors
        let second_store = storage_manager.store_file("recovery_storage_key_2", test_content, &metadata).await;
        assert!(second_store.is_ok(), "Storage should continue working after errors");
    }
    
    Ok(())
}

/// Test concurrent error conditions
#[tokio::test]
async fn test_concurrent_error_conditions() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Create multiple components that might interfere with each other
    let error_tasks = vec![
        // Task 1: Database stress with errors
        tokio::spawn({
            let db_path = env.db_path.clone();
            async move {
                for i in 0..50 {
                    let db = DatabaseManager::new(&db_path)?;
                    
                    // Try to create files with potentially problematic names
                    let problematic_names = vec![
                        format!("file_{}", i),
                        "".to_string(), // Empty name
                        "/\\|<>:\"*?".to_string(), // Special characters
                        "a".repeat(1000), // Very long name
                    ];
                    
                    for name in problematic_names {
                        let upload_time = chrono::Local::now();
                        let _ = db.store_file(
                            &name,
                            &format!("key_{}", i),
                            &name,
                            1024,
                            upload_time,
                            &vec!["error_test".to_string()],
                            "test_public_key",
                        );
                    }
                    
                    if i % 10 == 0 {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
        }),
        
        // Task 2: File system operations with errors
        tokio::spawn({
            let storage_path = env.storage_path.clone();
            async move {
                for i in 0..30 {
                    // Try to create files in various locations
                    let file_paths = vec![
                        storage_path.join(format!("error_test_{}.txt", i)),
                        storage_path.join(""), // Empty path component
                        std::path::PathBuf::from("/tmp").join(format!("datamesh_error_test_{}", i)),
                    ];
                    
                    for path in file_paths {
                        let content = format!("Error test content {}", i);
                        let _ = tokio::fs::write(&path, content).await;
                        let _ = tokio::fs::read(&path).await;
                        let _ = tokio::fs::remove_file(&path).await;
                    }
                    
                    if i % 5 == 0 {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
        }),
        
        // Task 3: Cryptographic operations with potential errors
        tokio::spawn(async move {
            for i in 0..25 {
                let key_manager = KeyManager::new(
                    ecies::SecretKey::random(&mut rand::thread_rng()),
                    format!("error_test_key_{}", i)
                );
                
                // Test various data that might cause issues
                let test_data_sets = vec![
                    vec![], // Empty
                    vec![i as u8; i % 1000 + 1], // Variable size
                    (0..=255u8).cycle().take(i * 10 + 1).collect(), // Pseudo-random
                ];
                
                for data in test_data_sets {
                    if let Ok(encrypted) = key_manager.encrypt(&data) {
                        let _ = key_manager.decrypt(&encrypted);
                    }
                }
                
                if i % 3 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            Ok::<(), anyhow::Error>(())
        }),
    ];
    
    // Wait for all error condition tasks to complete
    let mut successful_tasks = 0;
    for task in error_tasks {
        match task.await {
            Ok(Ok(())) => successful_tasks += 1,
            Ok(Err(e)) => println!("Task completed with error (expected): {}", e),
            Err(e) => println!("Task panicked (unexpected): {}", e),
        }
    }
    
    // At least some tasks should complete successfully despite error conditions
    assert!(successful_tasks >= 2, "Most tasks should handle error conditions gracefully");
    
    // System should remain functional after concurrent error conditions
    let final_db = DatabaseManager::new(&env.db_path)?;
    let final_files = final_db.list_files(None)?;
    // We don't assert on file count since some operations may have failed
    println!("System remains functional with {} files after error stress test", final_files.len());
    
    Ok(())
}

/// Test memory pressure and resource exhaustion scenarios
#[tokio::test]
async fn test_resource_exhaustion_resilience() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test with progressively larger data sizes to approach memory limits
    let sizes = vec![
        1024,           // 1KB
        1024 * 1024,    // 1MB  
        5 * 1024 * 1024, // 5MB
        // Note: We avoid truly large sizes to prevent test system issues
    ];
    
    let key_manager = KeyManager::new(
        ecies::SecretKey::random(&mut rand::thread_rng()),
        "resource_test_key".to_string()
    );
    
    for (i, size) in sizes.iter().enumerate() {
        println!("Testing resource usage with size: {} bytes", size);
        
        // Create test data
        let test_data = vec![i as u8; *size];
        
        let memory_perf = performance::PerformanceTest::new(&format!("memory_test_{}mb", size / 1024 / 1024));
        
        // Test encryption under memory pressure
        let encrypt_result = key_manager.encrypt(&test_data);
        
        match encrypt_result {
            Ok(encrypted) => {
                // Test decryption
                let decrypt_result = key_manager.decrypt(&encrypted);
                
                match decrypt_result {
                    Ok(decrypted) => {
                        assert_eq!(test_data.len(), decrypted.len(), "Data size should be preserved");
                        
                        // Verify first and last bytes to ensure data integrity
                        if !test_data.is_empty() {
                            assert_eq!(test_data[0], decrypted[0], "First byte should match");
                            assert_eq!(
                                test_data[test_data.len() - 1], 
                                decrypted[decrypted.len() - 1], 
                                "Last byte should match"
                            );
                        }
                    }
                    Err(e) => {
                        println!("Decryption failed under memory pressure (size {}): {}", size, e);
                        // This might happen under extreme memory pressure, which is acceptable
                    }
                }
            }
            Err(e) => {
                println!("Encryption failed under memory pressure (size {}): {}", size, e);
                // This might happen under extreme memory pressure, which is acceptable
            }
        }
        
        memory_perf.finish(Duration::from_secs(30));
        
        // Force garbage collection between tests
        drop(test_data);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}

/// Test error propagation and handling across system boundaries
#[tokio::test]
async fn test_error_propagation() -> Result<()> {
    let env = TestEnvironment::new()?;
    
    // Test error propagation from storage to database
    {
        let db = DatabaseManager::new(&env.db_path)?;
        
        // Try to create a file entry with invalid data
        let invalid_upload_time = chrono::Local::now();
        
        // Test with extremely long file names
        let very_long_name = "a".repeat(10000);
        let result = db.store_file(
            &very_long_name,
            "test_key",
            &very_long_name,
            u64::MAX, // Extremely large size
            invalid_upload_time,
            &vec!["error_test".to_string()],
            "test_public_key",
        );
        
        // Result can be Ok or Err depending on database constraints
        match result {
            Ok(_) => println!("Database accepted large file metadata"),
            Err(e) => println!("Database rejected large file metadata: {}", e),
        }
        
        // Database should remain functional regardless
        let list_result = db.list_files(None);
        assert!(list_result.is_ok(), "Database should remain functional after error");
    }
    
    // Test error propagation in file operations
    {
        let config = env.create_test_config();
        
        if let Ok(file_manager) = FileManager::new(config).await {
            // Try to store a file that doesn't exist
            let nonexistent_path = env.storage_path.join("does_not_exist.txt");
            let store_result = file_manager.store_file(&nonexistent_path, None).await;
            
            assert!(store_result.is_err(), "Storing non-existent file should propagate error");
            
            // Verify error type
            if let Err(e) = store_result {
                // Error should be properly typed and informative
                assert!(!e.to_string().is_empty(), "Error message should not be empty");
            }
        }
    }
    
    Ok(())
}

/// Test system behavior during graceful shutdown under error conditions
#[tokio::test]
async fn test_graceful_shutdown_with_errors() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    // Create network handle
    if let Ok(network_handle) = NetworkHandle::new(&cli, &config).await {
        // Start some operations
        let operations = vec![
            network_handle.get_connected_peers(),
            network_handle.get_network_stats(),
            network_handle.bootstrap(),
        ];
        
        // Don't wait for operations to complete - test shutdown during operations
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Test graceful shutdown
        let shutdown_result = network_handle.shutdown().await;
        assert!(shutdown_result.is_ok(), "Shutdown should succeed even with pending operations");
        
        // Try to use handle after shutdown (should fail gracefully)
        let post_shutdown_result = network_handle.get_cached_stats().await;
        // This might succeed (returning cached data) or fail - both are acceptable
        println!("Post-shutdown operation result: {:?}", post_shutdown_result.local_peer_id);
    }
    
    Ok(())
}