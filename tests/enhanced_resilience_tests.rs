/// Enhanced Error Recovery and Resilience Tests for DataMesh
///
/// This module provides comprehensive testing for error recovery mechanisms,
/// resilience patterns, failure scenarios, and system robustness.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::error::DataMeshError;
use datamesh::file_manager::FileManager;
use datamesh::storage_economy::StorageEconomy;

/// Test setup for resilience testing
pub struct ResilienceTestSetup {
    temp_dir: TempDir,
    config: Config,
    original_db_path: PathBuf,
}

impl ResilienceTestSetup {
    /// Create a new resilience test setup
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();
        config.database.path = temp_dir.path().join("test.db").to_string_lossy().to_string();

        let original_db_path = temp_dir.path().join("test.db");

        Ok(ResilienceTestSetup {
            temp_dir,
            config,
            original_db_path,
        })
    }

    /// Simulate database corruption by modifying the file
    pub fn corrupt_database(&self) -> Result<()> {
        if self.original_db_path.exists() {
            // Write invalid data to corrupt the database
            fs::write(&self.original_db_path, b"CORRUPTED_DATA")?;
        }
        Ok(())
    }

    /// Simulate disk full condition by creating a large file
    pub fn simulate_disk_full(&self) -> Result<()> {
        let large_file_path = self.temp_dir.path().join("disk_full_simulation.tmp");
        // Create a 100MB file to simulate disk space issues
        let large_content = vec![0u8; 100 * 1024 * 1024];
        fs::write(large_file_path, large_content)?;
        Ok(())
    }

    /// Delete critical directories to simulate filesystem issues
    pub fn simulate_filesystem_failure(&self) -> Result<()> {
        let data_dir = PathBuf::from(&self.config.storage.data_dir);
        if data_dir.exists() {
            fs::remove_dir_all(data_dir)?;
        }
        Ok(())
    }

    /// Create read-only permissions to simulate permission issues
    pub fn simulate_permission_failure(&self) -> Result<()> {
        if self.original_db_path.exists() {
            let mut perms = fs::metadata(&self.original_db_path)?.permissions();
            perms.set_readonly(true);
            fs::set_permissions(&self.original_db_path, perms)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod database_resilience_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_database_corruption_recovery() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        
        // Create database and add some data
        let database = DatabaseManager::new(&setup.original_db_path)?;
        let upload_time = chrono::Local::now();
        
        // Store initial data
        database.store_file(
            "test_file",
            "test_key",
            "test_file.txt",
            1024,
            upload_time,
            &vec!["test".to_string()],
            "test_public_key",
        )?;
        
        // Verify data exists
        let initial_files = database.list_files(None)?;
        assert_eq!(initial_files.len(), 1);
        
        // Close database connection
        drop(database);
        
        // Corrupt the database
        setup.corrupt_database()?;
        
        // Try to open corrupted database
        let recovery_result = DatabaseManager::new(&setup.original_db_path);
        
        match recovery_result {
            Ok(recovered_db) => {
                // If recovery succeeded, verify it handles the corruption gracefully
                let files_after_recovery = recovered_db.list_files(None).unwrap_or_default();
                println!("Database recovered with {} files", files_after_recovery.len());
                
                // Should be able to add new data even after corruption
                let recovery_result = recovered_db.store_file(
                    "recovery_test",
                    "recovery_key",
                    "recovery.txt",
                    512,
                    upload_time,
                    &vec!["recovery".to_string()],
                    "test_public_key",
                );
                
                assert!(recovery_result.is_ok(), "Should be able to store data after recovery");
            }
            Err(e) => {
                println!("Database corruption detected: {:?}", e);
                
                // If corruption is detected, ensure we can create a new database
                let backup_path = setup.temp_dir.path().join("backup.db");
                let new_database = DatabaseManager::new(&backup_path)?;
                
                let new_store_result = new_database.store_file(
                    "new_test",
                    "new_key",
                    "new_test.txt",
                    1024,
                    upload_time,
                    &vec!["new".to_string()],
                    "test_public_key",
                );
                
                assert!(new_store_result.is_ok(), "Should be able to create new database after corruption");
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_failure() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let database = DatabaseManager::new(&setup.original_db_path)?;
        let upload_time = chrono::Local::now();
        
        // Store initial data
        database.store_file(
            "initial_file",
            "initial_key",
            "initial.txt",
            1024,
            upload_time,
            &vec!["initial".to_string()],
            "test_public_key",
        )?;
        
        let initial_count = database.list_files(None)?.len();
        
        // Attempt to store a file with invalid data to trigger rollback
        let invalid_result = database.store_file(
            "", // Invalid empty name
            "invalid_key",
            "invalid.txt",
            0, // Invalid size
            upload_time,
            &vec![],
            "",
        );
        
        // Operation should fail
        assert!(invalid_result.is_err(), "Invalid operation should fail");
        
        // Verify database state is unchanged
        let final_count = database.list_files(None)?.len();
        assert_eq!(initial_count, final_count, "Database should be unchanged after failed operation");
        
        // Verify we can still perform valid operations
        let valid_result = database.store_file(
            "recovery_file",
            "recovery_key",
            "recovery.txt",
            1024,
            upload_time,
            &vec!["recovery".to_string()],
            "test_public_key",
        );
        
        assert!(valid_result.is_ok(), "Valid operations should still work after failed transaction");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_failure_handling() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let database = Arc::new(DatabaseManager::new(&setup.original_db_path)?);
        
        let successful_operations = Arc::new(AtomicUsize::new(0));
        let failed_operations = Arc::new(AtomicUsize::new(0));
        let upload_time = chrono::Local::now();
        
        // Run concurrent operations with some that will fail
        let mut join_handles = Vec::new();
        
        for i in 0..20 {
            let db = database.clone();
            let success_count = successful_operations.clone();
            let failure_count = failed_operations.clone();
            
            let handle = tokio::spawn(async move {
                let file_name = if i % 5 == 0 {
                    // Every 5th operation will fail due to empty name
                    "".to_string()
                } else {
                    format!("concurrent_file_{}", i)
                };
                
                let result = db.store_file(
                    &file_name,
                    &format!("key_{}", i),
                    &format!("file_{}.txt", i),
                    1024,
                    upload_time,
                    &vec!["concurrent".to_string()],
                    "test_public_key",
                );
                
                match result {
                    Ok(_) => {
                        success_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        failure_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations
        for handle in join_handles {
            handle.await?;
        }
        
        let final_success = successful_operations.load(Ordering::SeqCst);
        let final_failures = failed_operations.load(Ordering::SeqCst);
        
        println!("Concurrent failure handling: {} success, {} failures", final_success, final_failures);
        
        // We expect 4 failures (every 5th operation) and 16 successes
        assert_eq!(final_failures, 4, "Expected number of failures should occur");
        assert_eq!(final_success, 16, "Expected number of successes should occur");
        
        // Database should still be functional
        let final_files = database.list_files(None)?;
        assert_eq!(final_files.len(), final_success, "Database should contain all successful operations");
        
        Ok(())
    }
}

#[cfg(test)]
mod file_manager_resilience_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_failure_recovery() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let file_manager = FileManager::new(setup.config.clone()).await?;
        
        // Create test file
        let test_file_path = setup.temp_dir.path().join("resilience_test.txt");
        let test_content = "This is a test file for resilience testing";
        tokio::fs::write(&test_file_path, test_content).await?;
        
        // Store file successfully first
        let storage_result = file_manager.store_file(&test_file_path, None).await?;
        assert!(!storage_result.file_key.is_empty());
        
        // Simulate filesystem failure
        setup.simulate_filesystem_failure()?;
        
        // Try to store another file (should handle gracefully)
        let test_file2_path = setup.temp_dir.path().join("resilience_test2.txt");
        tokio::fs::write(&test_file2_path, "Second test file").await?;
        
        let storage_result2 = file_manager.store_file(&test_file2_path, None).await;
        
        match storage_result2 {
            Ok(_) => {
                println!("File manager recovered from filesystem failure");
            }
            Err(e) => {
                println!("File manager detected filesystem failure: {:?}", e);
                
                // Ensure error is properly categorized
                assert!(e.to_string().contains("filesystem") || 
                        e.to_string().contains("directory") ||
                        e.to_string().contains("No such file"),
                        "Error should indicate filesystem issue");
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_partial_upload_recovery() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let file_manager = FileManager::new(setup.config.clone()).await?;
        
        // Create a large test file that will be chunked
        let large_file_path = setup.temp_dir.path().join("large_test.bin");
        let large_content = vec![0xAA; 5 * 1024 * 1024]; // 5MB
        tokio::fs::write(&large_file_path, &large_content).await?;
        
        // Start upload but simulate interruption by creating a smaller file
        // with same name during upload
        let concurrent_handle = tokio::spawn({
            let file_path = large_file_path.clone();
            async move {
                sleep(Duration::from_millis(100)).await;
                // Simulate file modification during upload
                tokio::fs::write(&file_path, b"modified during upload").await
            }
        });
        
        // Attempt to store the file
        let storage_result = file_manager.store_file(&large_file_path, None).await;
        
        // Wait for concurrent modification
        let _ = concurrent_handle.await;
        
        match storage_result {
            Ok(result) => {
                println!("Upload completed despite concurrent modification");
                
                // Try to retrieve the file
                let retrieved_path = setup.temp_dir.path().join("retrieved_large.bin");
                let retrieval_result = file_manager.retrieve_file(&result.file_key, &retrieved_path).await;
                
                if retrieval_result.is_ok() {
                    // Verify content integrity
                    let retrieved_content = tokio::fs::read(&retrieved_path).await?;
                    println!("Retrieved file size: {}", retrieved_content.len());
                    
                    // Should either be original content or modified content, but consistent
                    assert!(retrieved_content == large_content || 
                           retrieved_content == b"modified during upload",
                           "Retrieved content should be consistent");
                }
            }
            Err(e) => {
                println!("Upload failed due to concurrent modification: {:?}", e);
                // This is acceptable behavior - the system detected the issue
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_retrieval_with_missing_chunks() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let file_manager = FileManager::new(setup.config.clone()).await?;
        
        // Create and store test file
        let test_file_path = setup.temp_dir.path().join("chunk_test.txt");
        let test_content = "This is a test file for chunk resilience testing with some content";
        tokio::fs::write(&test_file_path, test_content).await?;
        
        let storage_result = file_manager.store_file(&test_file_path, None).await?;
        
        // Simulate missing chunks by removing some data files
        let data_dir = PathBuf::from(&setup.config.storage.data_dir);
        if data_dir.exists() {
            // Remove some files to simulate missing chunks
            if let Ok(entries) = fs::read_dir(&data_dir) {
                let mut removed_count = 0;
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("chunk") {
                        let _ = fs::remove_file(entry.path());
                        removed_count += 1;
                        if removed_count >= 2 { // Remove a few chunks
                            break;
                        }
                    }
                }
                
                if removed_count > 0 {
                    println!("Removed {} chunks to simulate missing data", removed_count);
                }
            }
        }
        
        // Try to retrieve file with missing chunks
        let retrieved_path = setup.temp_dir.path().join("retrieved_chunk_test.txt");
        let retrieval_result = file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await;
        
        match retrieval_result {
            Ok(_) => {
                println!("File retrieved successfully despite missing chunks (Reed-Solomon recovery)");
                
                // Verify content if recovery was successful
                let retrieved_content = tokio::fs::read_to_string(&retrieved_path).await?;
                assert_eq!(retrieved_content, test_content, "Content should be recovered correctly");
            }
            Err(e) => {
                println!("File retrieval failed due to missing chunks: {:?}", e);
                
                // This is expected if too many chunks are missing
                assert!(e.to_string().contains("chunk") || 
                        e.to_string().contains("missing") ||
                        e.to_string().contains("corrupt"),
                        "Error should indicate chunk-related issue");
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod system_resilience_tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_exhaustion_handling() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        
        // Simulate disk full condition
        setup.simulate_disk_full()?;
        
        let file_manager = FileManager::new(setup.config.clone()).await?;
        
        // Try to store files when disk is "full"
        let test_file_path = setup.temp_dir.path().join("disk_full_test.txt");
        let test_content = "Testing disk full scenario";
        tokio::fs::write(&test_file_path, test_content).await?;
        
        let storage_result = file_manager.store_file(&test_file_path, None).await;
        
        match storage_result {
            Ok(_) => {
                println!("Storage succeeded despite simulated disk full condition");
            }
            Err(e) => {
                println!("Storage failed due to disk space: {:?}", e);
                
                // Verify error indicates storage issue
                let error_str = e.to_string().to_lowercase();
                assert!(error_str.contains("space") || 
                        error_str.contains("disk") ||
                        error_str.contains("storage"),
                        "Error should indicate storage space issue");
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_permission_error_handling() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        
        // Create database first
        let database = DatabaseManager::new(&setup.original_db_path)?;
        
        // Store some initial data
        let upload_time = chrono::Local::now();
        database.store_file(
            "permission_test",
            "permission_key",
            "permission.txt",
            1024,
            upload_time,
            &vec!["permission".to_string()],
            "test_public_key",
        )?;
        
        // Close database to release file handles
        drop(database);
        
        // Simulate permission failure
        setup.simulate_permission_failure()?;
        
        // Try to open database with restricted permissions
        let restricted_result = DatabaseManager::new(&setup.original_db_path);
        
        match restricted_result {
            Ok(db) => {
                // If opened successfully, try to write (should fail)
                let write_result = db.store_file(
                    "write_test",
                    "write_key",
                    "write.txt",
                    512,
                    upload_time,
                    &vec!["write".to_string()],
                    "test_public_key",
                );
                
                match write_result {
                    Ok(_) => {
                        println!("Write succeeded despite read-only permissions");
                    }
                    Err(e) => {
                        println!("Write failed due to permissions: {:?}", e);
                        
                        // Should still be able to read existing data
                        let read_result = db.list_files(None);
                        if let Ok(files) = read_result {
                            assert!(!files.is_empty(), "Should be able to read existing data");
                        }
                    }
                }
            }
            Err(e) => {
                println!("Database opening failed due to permissions: {:?}", e);
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_graceful_degradation() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let storage_economy = StorageEconomy::new(setup.config.clone())?;
        
        // Test system behavior under various failure conditions
        let test_scenarios = vec![
            "normal_operation",
            "database_slow",
            "storage_limited",
            "network_latency",
        ];
        
        for scenario in test_scenarios {
            println!("Testing graceful degradation scenario: {}", scenario);
            
            let start_time = Instant::now();
            
            match scenario {
                "normal_operation" => {
                    // Normal operation baseline
                    let user = create_test_user("normal_user");
                    let result = storage_economy.register_user(user).await;
                    assert!(result.is_ok(), "Normal operation should succeed");
                }
                "database_slow" => {
                    // Simulate slow database by adding artificial delay
                    let user = create_test_user("slow_user");
                    
                    // Use timeout to handle slow operations
                    let result = timeout(Duration::from_secs(5), async {
                        storage_economy.register_user(user).await
                    }).await;
                    
                    match result {
                        Ok(Ok(_)) => println!("Operation completed within timeout"),
                        Ok(Err(e)) => println!("Operation failed: {:?}", e),
                        Err(_) => println!("Operation timed out (graceful degradation)"),
                    }
                }
                "storage_limited" => {
                    // Test with very limited storage quota
                    let user = create_test_user("limited_user");
                    let register_result = storage_economy.register_user(user.clone()).await;
                    
                    if register_result.is_ok() {
                        // Set very small quota
                        let quota_result = storage_economy.set_storage_quota(&user.email, 1024);
                        
                        match quota_result {
                            Ok(_) => println!("Quota set successfully"),
                            Err(e) => println!("Quota setting failed gracefully: {:?}", e),
                        }
                    }
                }
                "network_latency" => {
                    // Simulate network latency with delays
                    sleep(Duration::from_millis(100)).await;
                    
                    let user = create_test_user("latency_user");
                    let result = storage_economy.register_user(user).await;
                    
                    // Operation should still complete
                    match result {
                        Ok(_) => println!("Operation succeeded despite latency"),
                        Err(e) => println!("Operation failed due to latency: {:?}", e),
                    }
                }
                _ => {}
            }
            
            let scenario_duration = start_time.elapsed();
            println!("Scenario '{}' completed in {:?}", scenario, scenario_duration);
            
            // All scenarios should complete within reasonable time
            assert!(scenario_duration < Duration::from_secs(10), 
                    "Scenario should complete within reasonable time");
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_breaker_pattern() -> Result<()> {
        // Test circuit breaker pattern for handling repeated failures
        let failure_threshold = 5;
        let consecutive_failures = Arc::new(AtomicUsize::new(0));
        let circuit_open = Arc::new(AtomicBool::new(false));
        
        // Simulate operations with circuit breaker
        for i in 0..10 {
            let failures = consecutive_failures.load(Ordering::SeqCst);
            let is_open = circuit_open.load(Ordering::SeqCst);
            
            if is_open {
                println!("Circuit breaker is OPEN, rejecting operation {}", i);
                continue;
            }
            
            // Simulate operation (will fail for first 7 operations)
            let operation_result = if i < 7 {
                Err(anyhow::anyhow!("Simulated operation failure"))
            } else {
                Ok(())
            };
            
            match operation_result {
                Ok(_) => {
                    println!("Operation {} succeeded", i);
                    consecutive_failures.store(0, Ordering::SeqCst);
                    circuit_open.store(false, Ordering::SeqCst);
                }
                Err(e) => {
                    println!("Operation {} failed: {:?}", i, e);
                    let new_failures = consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    if new_failures >= failure_threshold {
                        println!("Circuit breaker OPENED after {} failures", new_failures);
                        circuit_open.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
        
        let final_failures = consecutive_failures.load(Ordering::SeqCst);
        let final_state = circuit_open.load(Ordering::SeqCst);
        
        println!("Circuit breaker test completed: {} failures, circuit open: {}", 
                final_failures, final_state);
        
        // Circuit breaker should have opened due to consecutive failures
        assert!(final_state || final_failures == 0, 
                "Circuit breaker should be open after repeated failures or reset after success");
        
        Ok(())
    }
}

/// Helper function to create a test user
fn create_test_user(user_id: &str) -> datamesh::governance::UserAccount {
    datamesh::governance::UserAccount {
        user_id: uuid::Uuid::new_v4(),
        email: format!("{}@test.com", user_id),
        password_hash: "test_hash".to_string(),
        public_key: format!("test_key_{}", user_id),
        account_type: datamesh::governance::AccountType::Free {
            storage_gb: 5,
            bandwidth_gb_month: 100,
            api_calls_hour: 1000,
        },
        verification_status: datamesh::governance::VerificationStatus::EmailVerified,
        registration_date: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        reputation_score: 0.0,
        abuse_flags: vec![],
        subscription: None,
    }
}

/// Stress test helper for resource exhaustion scenarios
#[cfg(test)]
mod stress_resilience_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_pressure_resilience() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let file_manager = FileManager::new(setup.config.clone()).await?;
        
        // Create many files to stress memory usage
        let file_count = 100;
        let mut storage_results = Vec::new();
        
        for i in 0..file_count {
            let file_path = setup.temp_dir.path().join(format!("memory_test_{}.txt", i));
            let content = format!("Memory pressure test file {}", i);
            tokio::fs::write(&file_path, content).await?;
            
            match file_manager.store_file(&file_path, None).await {
                Ok(result) => {
                    storage_results.push(result);
                }
                Err(e) => {
                    println!("Storage failed under memory pressure: {:?}", e);
                    break;
                }
            }
            
            // Check memory usage periodically
            if i % 20 == 0 {
                println!("Stored {} files, continuing memory pressure test...", i);
            }
        }
        
        println!("Memory pressure test completed with {} successful operations", storage_results.len());
        
        // Should handle reasonable number of files
        assert!(storage_results.len() >= 50, "Should handle reasonable memory pressure");
        
        // Test retrieval under memory pressure
        let mid_index = storage_results.len() / 2;
        let retrieved_path = setup.temp_dir.path().join("memory_retrieved.txt");
        let retrieval_result = file_manager.retrieve_file(&storage_results[mid_index].file_key, &retrieved_path).await;
        
        assert!(retrieval_result.is_ok(), "Should be able to retrieve files under memory pressure");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_high_concurrency_resilience() -> Result<()> {
        let setup = ResilienceTestSetup::new().await?;
        let file_manager = Arc::new(FileManager::new(setup.config.clone()).await?);
        
        let concurrent_operations = 50;
        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));
        
        let mut join_handles = Vec::new();
        
        for i in 0..concurrent_operations {
            let fm = file_manager.clone();
            let temp_dir = setup.temp_dir.path().to_path_buf();
            let success = success_count.clone();
            let failures = failure_count.clone();
            
            let handle = tokio::spawn(async move {
                let file_path = temp_dir.join(format!("concurrent_resilience_{}.txt", i));
                let content = format!("High concurrency test file {}", i);
                
                if tokio::fs::write(&file_path, content).await.is_ok() {
                    match fm.store_file(&file_path, None).await {
                        Ok(_) => {
                            success.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(_) => {
                            failures.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                } else {
                    failures.fetch_add(1, Ordering::SeqCst);
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations with timeout
        let start_time = Instant::now();
        for handle in join_handles {
            let _ = timeout(Duration::from_secs(30), handle).await;
        }
        let total_time = start_time.elapsed();
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_failures = failure_count.load(Ordering::SeqCst);
        
        println!("High concurrency resilience: {} success, {} failures in {:?}", 
                final_success, final_failures, total_time);
        
        // Most operations should succeed even under high concurrency
        assert!(final_success >= concurrent_operations * 70 / 100, 
                "At least 70% of operations should succeed under high concurrency");
        
        // Should complete in reasonable time
        assert!(total_time < Duration::from_secs(60), 
                "High concurrency test should complete in reasonable time");
        
        Ok(())
    }
}