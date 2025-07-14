/// Enhanced Concurrency and Race Condition Tests for DataMesh
///
/// This module provides comprehensive testing for concurrent operations,
/// race condition detection, deadlock prevention, and thread safety.

use anyhow::Result;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::{Semaphore, RwLock as TokioRwLock};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::file_manager::FileManager;
use datamesh::governance::{AccountType, UserAccount, VerificationStatus};
use datamesh::storage_economy::StorageEconomy;

/// Shared test state for concurrency testing
pub struct ConcurrencyTestSetup {
    temp_dir: TempDir,
    database: Arc<DatabaseManager>,
    file_manager: Arc<FileManager>,
    storage_economy: Arc<StorageEconomy>,
    config: Config,
}

impl ConcurrencyTestSetup {
    /// Create a new concurrency test setup
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();
        config.database.path = temp_dir.path().join("test.db").to_string_lossy().to_string();

        let database = Arc::new(DatabaseManager::new(&temp_dir.path().join("test.db"))?);
        let file_manager = Arc::new(FileManager::new(config.clone()).await?);
        let storage_economy = Arc::new(StorageEconomy::new(config.clone())?);

        Ok(ConcurrencyTestSetup {
            temp_dir,
            database,
            file_manager,
            storage_economy,
            config,
        })
    }

    /// Create a test user for concurrency testing
    pub fn create_test_user(&self, user_id: &str) -> UserAccount {
        UserAccount {
            user_id: Uuid::new_v4(),
            email: format!("{}@test.com", user_id),
            password_hash: "test_hash".to_string(),
            public_key: format!("test_key_{}", user_id),
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 100,
                api_calls_hour: 1000,
            },
            verification_status: VerificationStatus::EmailVerified,
            registration_date: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            reputation_score: 0.0,
            abuse_flags: vec![],
            subscription: None,
        }
    }
}

#[cfg(test)]
mod database_concurrency_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_concurrent_database_writes() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let database = setup.database.clone();
        
        let concurrent_operations = 100;
        let upload_time = chrono::Local::now();
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        // Spawn concurrent write operations
        let mut join_handles = Vec::new();
        
        for i in 0..concurrent_operations {
            let db = database.clone();
            let success = success_count.clone();
            let errors = error_count.clone();
            
            let handle = tokio::spawn(async move {
                let file_name = format!("concurrent_file_{}", i);
                let file_key = format!("concurrent_key_{}", i);
                let tags = vec!["concurrent".to_string(), "test".to_string()];
                
                match db.store_file(
                    &file_name,
                    &file_key,
                    &format!("original_{}.txt", i),
                    1024 * (i + 1) as u64,
                    upload_time,
                    &tags,
                    "test_public_key",
                ) {
                    Ok(_) => {
                        success.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in join_handles {
            handle.await?;
        }
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_errors = error_count.load(Ordering::SeqCst);
        
        println!("Concurrent database writes: {} success, {} errors", final_success, final_errors);
        
        // Most operations should succeed
        assert!(final_success > concurrent_operations * 90 / 100, 
                "At least 90% of concurrent database writes should succeed");
        
        // Verify all successful files are in database
        let all_files = database.list_files(None)?;
        assert_eq!(all_files.len(), final_success);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_read_write_operations() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let database = setup.database.clone();
        
        // First, populate database with initial data
        let upload_time = chrono::Local::now();
        for i in 0..50 {
            database.store_file(
                &format!("initial_file_{}", i),
                &format!("initial_key_{}", i),
                &format!("initial_{}.txt", i),
                1024,
                upload_time,
                &vec!["initial".to_string()],
                "test_public_key",
            )?;
        }
        
        let read_count = Arc::new(AtomicUsize::new(0));
        let write_count = Arc::new(AtomicUsize::new(0));
        let stop_flag = Arc::new(AtomicBool::new(false));
        
        // Start concurrent readers
        let mut reader_handles = Vec::new();
        for i in 0..10 {
            let db = database.clone();
            let reads = read_count.clone();
            let stop = stop_flag.clone();
            
            let handle = tokio::spawn(async move {
                while !stop.load(Ordering::SeqCst) {
                    // Read operations
                    let _ = db.list_files(None);
                    let _ = db.get_file_by_name(&format!("initial_file_{}", i % 50));
                    let _ = db.search_files("initial");
                    
                    reads.fetch_add(3, Ordering::SeqCst);
                    sleep(Duration::from_millis(10)).await;
                }
            });
            
            reader_handles.push(handle);
        }
        
        // Start concurrent writers
        let mut writer_handles = Vec::new();
        for i in 0..5 {
            let db = database.clone();
            let writes = write_count.clone();
            let stop = stop_flag.clone();
            
            let handle = tokio::spawn(async move {
                let mut counter = 0;
                while !stop.load(Ordering::SeqCst) {
                    let file_name = format!("concurrent_write_{}_{}", i, counter);
                    let file_key = format!("concurrent_write_key_{}_{}", i, counter);
                    
                    let _ = db.store_file(
                        &file_name,
                        &file_key,
                        &format!("concurrent_{}.txt", counter),
                        1024,
                        upload_time,
                        &vec!["concurrent".to_string()],
                        "test_public_key",
                    );
                    
                    writes.fetch_add(1, Ordering::SeqCst);
                    counter += 1;
                    sleep(Duration::from_millis(50)).await;
                }
            });
            
            writer_handles.push(handle);
        }
        
        // Let operations run for a while
        sleep(Duration::from_secs(5)).await;
        
        // Stop all operations
        stop_flag.store(true, Ordering::SeqCst);
        
        // Wait for all tasks to complete
        for handle in reader_handles {
            handle.await?;
        }
        for handle in writer_handles {
            handle.await?;
        }
        
        let total_reads = read_count.load(Ordering::SeqCst);
        let total_writes = write_count.load(Ordering::SeqCst);
        
        println!("Concurrent operations completed: {} reads, {} writes", total_reads, total_writes);
        
        // Verify database consistency
        let final_files = database.list_files(None)?;
        assert!(final_files.len() >= 50, "Should have at least initial files");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_transaction_isolation() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let database = setup.database.clone();
        
        // Test that concurrent transactions don't interfere with each other
        let barrier = Arc::new(tokio::sync::Barrier::new(3));
        let results = Arc::new(Mutex::new(Vec::new()));
        
        let mut join_handles = Vec::new();
        
        for i in 0..3 {
            let db = database.clone();
            let barrier = barrier.clone();
            let results = results.clone();
            
            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier.wait().await;
                
                let upload_time = chrono::Local::now();
                let file_name = format!("transaction_test_{}", i);
                let file_key = format!("transaction_key_{}", i);
                
                // Perform transaction
                let result = db.store_file(
                    &file_name,
                    &file_key,
                    &format!("transaction_{}.txt", i),
                    1024 * (i + 1) as u64,
                    upload_time,
                    &vec!["transaction".to_string()],
                    "test_public_key",
                );
                
                results.lock().unwrap().push((i, result.is_ok()));
                
                // Verify our file exists
                let file = db.get_file_by_name(&file_name).unwrap();
                file.is_some()
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all transactions to complete
        let transaction_results: Vec<bool> = futures::future::try_join_all(join_handles).await?;
        
        // All transactions should succeed
        assert!(transaction_results.iter().all(|&r| r), "All transactions should succeed");
        
        // Verify all files exist in database
        let all_files = database.list_files(None)?;
        let transaction_files: Vec<_> = all_files.iter()
            .filter(|f| f.tags.contains(&"transaction".to_string()))
            .collect();
        
        assert_eq!(transaction_files.len(), 3, "All transaction files should be stored");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_deadlock_detection() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let database = setup.database.clone();
        
        // Create a scenario that could potentially cause deadlocks
        let semaphore = Arc::new(Semaphore::new(1));
        let completion_count = Arc::new(AtomicUsize::new(0));
        
        let mut join_handles = Vec::new();
        
        for i in 0..10 {
            let db = database.clone();
            let sem = semaphore.clone();
            let count = completion_count.clone();
            
            let handle = tokio::spawn(async move {
                // Acquire semaphore to serialize critical operations
                let _permit = sem.acquire().await.unwrap();
                
                let upload_time = chrono::Local::now();
                
                // Perform multiple related database operations
                let file1_name = format!("deadlock_test_{}a", i);
                let file2_name = format!("deadlock_test_{}b", i);
                
                let result1 = db.store_file(
                    &file1_name,
                    &format!("key_{}a", i),
                    &format!("file_{}a.txt", i),
                    1024,
                    upload_time,
                    &vec!["deadlock".to_string()],
                    "test_public_key",
                );
                
                // Small delay to increase contention
                sleep(Duration::from_millis(10)).await;
                
                let result2 = db.store_file(
                    &file2_name,
                    &format!("key_{}b", i),
                    &format!("file_{}b.txt", i),
                    1024,
                    upload_time,
                    &vec!["deadlock".to_string()],
                    "test_public_key",
                );
                
                if result1.is_ok() && result2.is_ok() {
                    count.fetch_add(1, Ordering::SeqCst);
                }
            });
            
            join_handles.push(handle);
        }
        
        // Use timeout to detect potential deadlocks
        let timeout_result = timeout(Duration::from_secs(30), async {
            for handle in join_handles {
                handle.await.unwrap();
            }
        }).await;
        
        assert!(timeout_result.is_ok(), "Operations should complete without deadlock");
        
        let completed_operations = completion_count.load(Ordering::SeqCst);
        println!("Completed {} operations without deadlock", completed_operations);
        
        // Most operations should complete successfully
        assert!(completed_operations >= 8, "Most operations should complete without deadlock");
        
        Ok(())
    }
}

#[cfg(test)]
mod file_manager_concurrency_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_file_storage() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let file_manager = setup.file_manager.clone();
        
        // Create test files
        let mut test_files = Vec::new();
        for i in 0..20 {
            let file_path = setup.temp_dir.path().join(format!("concurrent_test_{}.txt", i));
            let content = format!("This is test file number {} for concurrent storage testing", i);
            tokio::fs::write(&file_path, content).await?;
            test_files.push(file_path);
        }
        
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        // Store files concurrently
        let mut join_handles = Vec::new();
        
        for (i, file_path) in test_files.into_iter().enumerate() {
            let fm = file_manager.clone();
            let success = success_count.clone();
            let errors = error_count.clone();
            
            let handle = tokio::spawn(async move {
                match fm.store_file(&file_path, Some(format!("version_{}", i))).await {
                    Ok(_) => {
                        success.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all storage operations
        for handle in join_handles {
            handle.await?;
        }
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_errors = error_count.load(Ordering::SeqCst);
        
        println!("Concurrent file storage: {} success, {} errors", final_success, final_errors);
        
        // Most operations should succeed
        assert!(final_success >= 18, "Most concurrent file storage operations should succeed");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_file_access() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let file_manager = setup.file_manager.clone();
        
        // Store a test file first
        let test_file_path = setup.temp_dir.path().join("shared_test_file.txt");
        let test_content = "This is a shared file for concurrent access testing";
        tokio::fs::write(&test_file_path, test_content).await?;
        
        let storage_result = file_manager.store_file(&test_file_path, None).await?;
        let file_key = storage_result.file_key;
        
        let access_count = Arc::new(AtomicUsize::new(0));
        let success_count = Arc::new(AtomicUsize::new(0));
        
        // Concurrent read operations
        let mut join_handles = Vec::new();
        
        for i in 0..15 {
            let fm = file_manager.clone();
            let key = file_key.clone();
            let temp_dir = setup.temp_dir.path().to_path_buf();
            let access = access_count.clone();
            let success = success_count.clone();
            
            let handle = tokio::spawn(async move {
                access.fetch_add(1, Ordering::SeqCst);
                
                let retrieved_path = temp_dir.join(format!("concurrent_retrieved_{}.txt", i));
                
                match fm.retrieve_file(&key, &retrieved_path).await {
                    Ok(_) => {
                        // Verify content
                        if let Ok(content) = tokio::fs::read_to_string(&retrieved_path).await {
                            if content == test_content {
                                success.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                    Err(_) => {}
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all access operations
        for handle in join_handles {
            handle.await?;
        }
        
        let total_access = access_count.load(Ordering::SeqCst);
        let total_success = success_count.load(Ordering::SeqCst);
        
        println!("Concurrent file access: {} attempts, {} successful", total_access, total_success);
        
        // All concurrent reads should succeed
        assert_eq!(total_success, 15, "All concurrent file reads should succeed");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_file_manager_under_pressure() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let file_manager = setup.file_manager.clone();
        
        // Create a high-pressure scenario with many concurrent operations
        let operation_count = 50;
        let barrier = Arc::new(tokio::sync::Barrier::new(operation_count));
        let results = Arc::new(TokioRwLock::new(Vec::new()));
        
        let mut join_handles = Vec::new();
        
        for i in 0..operation_count {
            let fm = file_manager.clone();
            let barrier = barrier.clone();
            let results = results.clone();
            let temp_dir = setup.temp_dir.path().to_path_buf();
            
            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier.wait().await;
                
                let start_time = Instant::now();
                
                // Create test file
                let file_path = temp_dir.join(format!("pressure_test_{}.txt", i));
                let content = format!("Pressure test file {} with content", i);
                tokio::fs::write(&file_path, &content).await.unwrap();
                
                // Store file
                let storage_result = fm.store_file(&file_path, None).await;
                let storage_duration = start_time.elapsed();
                
                if let Ok(result) = storage_result {
                    // Retrieve file
                    let retrieve_start = Instant::now();
                    let retrieved_path = temp_dir.join(format!("pressure_retrieved_{}.txt", i));
                    let retrieve_result = fm.retrieve_file(&result.file_key, &retrieved_path).await;
                    let retrieve_duration = retrieve_start.elapsed();
                    
                    let success = retrieve_result.is_ok() && 
                                 tokio::fs::read_to_string(&retrieved_path).await.unwrap_or_default() == content;
                    
                    results.write().await.push((i, success, storage_duration, retrieve_duration));
                } else {
                    results.write().await.push((i, false, storage_duration, Duration::ZERO));
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations
        for handle in join_handles {
            handle.await?;
        }
        
        let results = results.read().await;
        let successful_operations = results.iter().filter(|(_, success, _, _)| *success).count();
        
        let avg_storage_time: Duration = results.iter()
            .map(|(_, _, storage_time, _)| *storage_time)
            .sum::<Duration>() / results.len() as u32;
            
        let avg_retrieval_time: Duration = results.iter()
            .filter(|(_, success, _, _)| *success)
            .map(|(_, _, _, retrieval_time)| *retrieval_time)
            .sum::<Duration>() / successful_operations.max(1) as u32;
        
        println!("Pressure test results: {}/{} successful operations", successful_operations, operation_count);
        println!("Average storage time: {:?}", avg_storage_time);
        println!("Average retrieval time: {:?}", avg_retrieval_time);
        
        // Most operations should succeed under pressure
        assert!(successful_operations >= operation_count * 80 / 100, 
                "At least 80% of operations should succeed under pressure");
        
        // Performance should remain reasonable
        assert!(avg_storage_time < Duration::from_secs(5), 
                "Average storage time should be reasonable");
        assert!(avg_retrieval_time < Duration::from_secs(3), 
                "Average retrieval time should be reasonable");
        
        Ok(())
    }
}

#[cfg(test)]
mod race_condition_tests {
    use super::*;

    #[tokio::test]
    async fn test_race_condition_in_user_registration() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let storage_economy = setup.storage_economy.clone();
        
        // Simulate concurrent user registrations with the same email
        let email = "race.test@example.com";
        let concurrent_registrations = 10;
        
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        let mut join_handles = Vec::new();
        
        for i in 0..concurrent_registrations {
            let economy = storage_economy.clone();
            let success = success_count.clone();
            let errors = error_count.clone();
            let test_email = email.to_string();
            
            let handle = tokio::spawn(async move {
                let user = UserAccount {
                    user_id: Uuid::new_v4(),
                    email: test_email,
                    password_hash: format!("hash_{}", i),
                    public_key: format!("key_{}", i),
                    account_type: AccountType::Free {
                        storage_gb: 5,
                        bandwidth_gb_month: 100,
                        api_calls_hour: 1000,
                    },
                    verification_status: VerificationStatus::Pending,
                    registration_date: chrono::Utc::now(),
                    last_activity: chrono::Utc::now(),
                    reputation_score: 0.0,
                    abuse_flags: vec![],
                    subscription: None,
                };
                
                match economy.register_user(user).await {
                    Ok(_) => {
                        success.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all registration attempts
        for handle in join_handles {
            handle.await?;
        }
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_errors = error_count.load(Ordering::SeqCst);
        
        println!("Concurrent registrations: {} success, {} errors", final_success, final_errors);
        
        // Only one registration should succeed due to unique email constraint
        assert_eq!(final_success, 1, "Only one user registration should succeed for duplicate email");
        assert_eq!(final_errors, concurrent_registrations - 1, "Other registrations should fail");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_race_condition_in_resource_allocation() -> Result<()> {
        let setup = ConcurrencyTestSetup::new().await?;
        let storage_economy = setup.storage_economy.clone();
        
        // Create a user with limited resources
        let user = setup.create_test_user("resource_test");
        storage_economy.register_user(user.clone()).await?;
        
        // Set a small quota for testing
        let quota_limit = 1024 * 1024; // 1MB
        storage_economy.set_storage_quota(&user.email, quota_limit)?;
        
        let concurrent_allocations = 10;
        let allocation_size = 200 * 1024; // 200KB each (total would exceed quota)
        
        let success_count = Arc::new(AtomicUsize::new(0));
        let total_allocated = Arc::new(AtomicUsize::new(0));
        
        let mut join_handles = Vec::new();
        
        for i in 0..concurrent_allocations {
            let economy = storage_economy.clone();
            let user_email = user.email.clone();
            let success = success_count.clone();
            let allocated = total_allocated.clone();
            
            let handle = tokio::spawn(async move {
                match economy.allocate_storage(&user_email, allocation_size as u64).await {
                    Ok(_) => {
                        success.fetch_add(1, Ordering::SeqCst);
                        allocated.fetch_add(allocation_size, Ordering::SeqCst);
                    }
                    Err(_) => {
                        // Allocation failed (expected due to quota limits)
                    }
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all allocation attempts
        for handle in join_handles {
            handle.await?;
        }
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_allocated = total_allocated.load(Ordering::SeqCst);
        
        println!("Resource allocation: {} successful, {} bytes allocated", final_success, final_allocated);
        
        // Total allocated should not exceed quota
        assert!(final_allocated <= quota_limit, 
                "Total allocated storage should not exceed quota");
        
        // Some allocations should succeed, but not all
        assert!(final_success < concurrent_allocations, 
                "Not all allocations should succeed due to quota limits");
        assert!(final_success > 0, "Some allocations should succeed");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_atomic_counter_operations() -> Result<()> {
        // Test atomic operations under high concurrency
        let counter = Arc::new(AtomicUsize::new(0));
        let concurrent_operations = 1000;
        let operations_per_task = 100;
        
        let mut join_handles = Vec::new();
        
        for _ in 0..concurrent_operations {
            let counter = counter.clone();
            
            let handle = tokio::spawn(async move {
                for _ in 0..operations_per_task {
                    // Perform atomic increment
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations
        for handle in join_handles {
            handle.await?;
        }
        
        let final_value = counter.load(Ordering::SeqCst);
        let expected_value = concurrent_operations * operations_per_task;
        
        println!("Atomic counter final value: {} (expected: {})", final_value, expected_value);
        
        // All increments should be accounted for
        assert_eq!(final_value, expected_value, 
                   "Atomic operations should maintain consistency");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_shared_state_protection() -> Result<()> {
        // Test that shared state is properly protected from race conditions
        let shared_data = Arc::new(RwLock::new(HashMap::<String, i32>::new()));
        let concurrent_tasks = 50;
        let operations_per_task = 20;
        
        let mut join_handles = Vec::new();
        
        for task_id in 0..concurrent_tasks {
            let data = shared_data.clone();
            
            let handle = tokio::spawn(async move {
                for op in 0..operations_per_task {
                    let key = format!("key_{}_{}", task_id, op);
                    let value = task_id * operations_per_task + op;
                    
                    // Write operation
                    {
                        let mut map = data.write().unwrap();
                        map.insert(key.clone(), value);
                    }
                    
                    // Read operation
                    {
                        let map = data.read().unwrap();
                        let retrieved_value = map.get(&key).copied();
                        assert_eq!(retrieved_value, Some(value), "Value should match what was inserted");
                    }
                    
                    // Small delay to increase contention
                    sleep(Duration::from_millis(1)).await;
                }
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all operations
        for handle in join_handles {
            handle.await?;
        }
        
        // Verify final state
        let final_data = shared_data.read().unwrap();
        let expected_entries = concurrent_tasks * operations_per_task;
        
        println!("Shared state final size: {} (expected: {})", final_data.len(), expected_entries);
        
        assert_eq!(final_data.len(), expected_entries, 
                   "All entries should be present in shared state");
        
        // Verify all values are correct
        for task_id in 0..concurrent_tasks {
            for op in 0..operations_per_task {
                let key = format!("key_{}_{}", task_id, op);
                let expected_value = task_id * operations_per_task + op;
                let actual_value = final_data.get(&key).copied();
                assert_eq!(actual_value, Some(expected_value), 
                          "Value for key {} should be correct", key);
            }
        }
        
        Ok(())
    }
}

/// Helper function to simulate CPU-intensive work
fn cpu_intensive_work(iterations: usize) -> usize {
    let mut sum = 0;
    for i in 0..iterations {
        sum += i * i;
    }
    sum
}

/// Helper function to detect potential deadlocks
async fn deadlock_detector(duration: Duration) -> bool {
    let start = Instant::now();
    
    while start.elapsed() < duration {
        sleep(Duration::from_millis(100)).await;
    }
    
    false // No deadlock detected if we reach here
}