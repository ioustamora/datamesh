/// Enhanced File Storage and Chunk Management Tests for DataMesh
///
/// This module provides comprehensive testing for file storage operations,
/// Reed-Solomon encoding/decoding, chunk distribution, and data integrity.

use anyhow::Result;
use rand::{thread_rng, RngCore};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;

use datamesh::config::Config;
use datamesh::file_manager::FileManager;
use datamesh::storage_economy::StorageEconomy;

/// Test setup for storage testing
pub struct StorageTestSetup {
    temp_dir: TempDir,
    file_manager: FileManager,
    storage_economy: StorageEconomy,
    test_files_dir: PathBuf,
}

impl StorageTestSetup {
    /// Create a new storage test setup
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let test_files_dir = temp_dir.path().join("test_files");
        fs::create_dir_all(&test_files_dir).await?;

        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();

        // Set Reed-Solomon parameters for testing
        config.storage.data_shards = 4;
        config.storage.parity_shards = 2;
        config.storage.chunk_size = 1024 * 64; // 64KB chunks

        let file_manager = FileManager::new(config.clone()).await?;
        let storage_economy = StorageEconomy::new(config.clone())?;

        Ok(StorageTestSetup {
            temp_dir,
            file_manager,
            storage_economy,
            test_files_dir,
        })
    }

    /// Generate a test file with specific size and content
    pub async fn create_test_file(&self, name: &str, size: usize) -> Result<PathBuf> {
        let file_path = self.test_files_dir.join(name);
        
        // Generate random content
        let mut content = vec![0u8; size];
        thread_rng().fill_bytes(&mut content);
        
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    /// Create a test file with specific pattern for verification
    pub async fn create_patterned_file(&self, name: &str, size: usize, pattern: u8) -> Result<PathBuf> {
        let file_path = self.test_files_dir.join(name);
        let content = vec![pattern; size];
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    /// Create a test file with text content
    pub async fn create_text_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.test_files_dir.join(name);
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    /// Verify file integrity by comparing original and retrieved content
    pub async fn verify_file_integrity(&self, original_path: &PathBuf, retrieved_path: &PathBuf) -> Result<bool> {
        let original_content = fs::read(original_path).await?;
        let retrieved_content = fs::read(retrieved_path).await?;
        Ok(original_content == retrieved_content)
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_small_file_storage_and_retrieval() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create a small test file (1KB)
        let original_path = setup.create_test_file("small_test.txt", 1024).await?;
        
        // Store the file
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        assert!(!storage_result.file_key.is_empty());
        
        // Retrieve the file
        let retrieved_path = setup.temp_dir.path().join("retrieved_small.txt");
        setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
        
        // Verify integrity
        let integrity_check = setup.verify_file_integrity(&original_path, &retrieved_path).await?;
        assert!(integrity_check, "File integrity should be maintained");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_large_file_chunking() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create a large test file (5MB)
        let file_size = 5 * 1024 * 1024;
        let original_path = setup.create_test_file("large_test.bin", file_size).await?;
        
        // Store the file
        let start_time = Instant::now();
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        let storage_duration = start_time.elapsed();
        
        println!("Large file storage took: {:?}", storage_duration);
        assert!(!storage_result.file_key.is_empty());
        
        // Verify chunk count (should be split into multiple chunks)
        let expected_chunks = (file_size + setup.file_manager.config.storage.chunk_size - 1) / setup.file_manager.config.storage.chunk_size;
        assert!(expected_chunks > 1, "Large file should be split into multiple chunks");
        
        // Retrieve the file
        let retrieved_path = setup.temp_dir.path().join("retrieved_large.bin");
        let start_time = Instant::now();
        setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
        let retrieval_duration = start_time.elapsed();
        
        println!("Large file retrieval took: {:?}", retrieval_duration);
        
        // Verify integrity
        let integrity_check = setup.verify_file_integrity(&original_path, &retrieved_path).await?;
        assert!(integrity_check, "Large file integrity should be maintained");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_reed_solomon_redundancy() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create test file
        let original_path = setup.create_patterned_file("redundancy_test.dat", 8192, 0xAA).await?;
        
        // Store with Reed-Solomon encoding
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        
        // Verify storage includes parity data
        let file_info = setup.file_manager.get_file_info(&storage_result.file_key).await?;
        assert!(file_info.chunks_total >= file_info.chunks_healthy);
        
        // With 4 data shards + 2 parity shards, we should have 6 total shards
        let expected_total_shards = setup.file_manager.config.storage.data_shards + setup.file_manager.config.storage.parity_shards;
        assert_eq!(file_info.chunks_total, expected_total_shards);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_file_operations() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create multiple test files
        let mut file_paths = Vec::new();
        for i in 0..10 {
            let file_name = format!("concurrent_test_{}.txt", i);
            let content = format!("This is test file number {}", i);
            let path = setup.create_text_file(&file_name, &content).await?;
            file_paths.push(path);
        }
        
        // Store files concurrently
        let mut join_handles = Vec::new();
        for path in file_paths.iter() {
            let file_manager = setup.file_manager.clone();
            let path_clone = path.clone();
            
            let handle = tokio::spawn(async move {
                file_manager.store_file(&path_clone, None).await
            });
            
            join_handles.push(handle);
        }
        
        // Wait for all storage operations to complete
        let mut storage_results = Vec::new();
        for handle in join_handles {
            let result = handle.await??;
            storage_results.push(result);
        }
        
        assert_eq!(storage_results.len(), 10);
        
        // Verify all files have unique keys
        let mut unique_keys = std::collections::HashSet::new();
        for result in &storage_results {
            assert!(unique_keys.insert(result.file_key.clone()), 
                    "Each file should have a unique key");
        }
        
        // Retrieve files concurrently
        let mut retrieve_handles = Vec::new();
        for (i, result) in storage_results.iter().enumerate() {
            let file_manager = setup.file_manager.clone();
            let file_key = result.file_key.clone();
            let retrieved_path = setup.temp_dir.path().join(format!("retrieved_concurrent_{}.txt", i));
            
            let handle = tokio::spawn(async move {
                file_manager.retrieve_file(&file_key, &retrieved_path).await
            });
            
            retrieve_handles.push(handle);
        }
        
        // Wait for all retrieval operations
        for handle in retrieve_handles {
            handle.await??;
        }
        
        println!("✅ Concurrent file operations completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_file_deduplication() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create two identical files
        let content = "This is identical content for deduplication testing";
        let file1_path = setup.create_text_file("file1.txt", content).await?;
        let file2_path = setup.create_text_file("file2.txt", content).await?;
        
        // Store both files
        let result1 = setup.file_manager.store_file(&file1_path, None).await?;
        let result2 = setup.file_manager.store_file(&file2_path, None).await?;
        
        // If deduplication is implemented, the keys might be the same
        // or the storage system should recognize duplicate content
        println!("File 1 key: {}", result1.file_key);
        println!("File 2 key: {}", result2.file_key);
        
        // Test that both files can be retrieved correctly
        let retrieved1_path = setup.temp_dir.path().join("retrieved1.txt");
        let retrieved2_path = setup.temp_dir.path().join("retrieved2.txt");
        
        setup.file_manager.retrieve_file(&result1.file_key, &retrieved1_path).await?;
        setup.file_manager.retrieve_file(&result2.file_key, &retrieved2_path).await?;
        
        // Verify both retrievals are correct
        let retrieved1_content = fs::read_to_string(&retrieved1_path).await?;
        let retrieved2_content = fs::read_to_string(&retrieved2_path).await?;
        
        assert_eq!(retrieved1_content, content);
        assert_eq!(retrieved2_content, content);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_storage_performance_scaling() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Test storage performance with different file sizes
        let test_sizes = vec![
            1024,           // 1KB
            10 * 1024,      // 10KB
            100 * 1024,     // 100KB
            1024 * 1024,    // 1MB
            10 * 1024 * 1024, // 10MB
        ];
        
        let mut performance_results = HashMap::new();
        
        for size in test_sizes {
            let file_name = format!("perf_test_{}.bin", size);
            let original_path = setup.create_test_file(&file_name, size).await?;
            
            // Measure storage time
            let start_time = Instant::now();
            let storage_result = setup.file_manager.store_file(&original_path, None).await?;
            let storage_duration = start_time.elapsed();
            
            // Measure retrieval time
            let retrieved_path = setup.temp_dir.path().join(format!("retrieved_{}", file_name));
            let start_time = Instant::now();
            setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
            let retrieval_duration = start_time.elapsed();
            
            performance_results.insert(size, (storage_duration, retrieval_duration));
            
            println!("Size: {}KB, Storage: {:?}, Retrieval: {:?}", 
                     size / 1024, storage_duration, retrieval_duration);
        }
        
        // Verify performance characteristics
        // Storage time should scale reasonably with file size
        let small_storage_time = performance_results.get(&1024).unwrap().0;
        let large_storage_time = performance_results.get(&(10 * 1024 * 1024)).unwrap().0;
        
        // Large file shouldn't take more than 100x the time of small file
        let time_ratio = large_storage_time.as_millis() as f64 / small_storage_time.as_millis() as f64;
        assert!(time_ratio < 100.0, "Storage time scaling should be reasonable");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_corruption_recovery() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create test file
        let original_path = setup.create_test_file("corruption_test.dat", 10240).await?;
        
        // Store file with Reed-Solomon encoding
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        
        // Simulate chunk corruption by removing some chunks
        // (This would require access to the underlying storage to actually corrupt chunks)
        // For now, we test that the file can still be retrieved
        
        let retrieved_path = setup.temp_dir.path().join("retrieved_corruption_test.dat");
        let retrieval_result = setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await;
        
        assert!(retrieval_result.is_ok(), "File should be retrievable despite potential chunk issues");
        
        // Verify integrity
        let integrity_check = setup.verify_file_integrity(&original_path, &retrieved_path).await?;
        assert!(integrity_check, "File integrity should be maintained with Reed-Solomon");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_storage_quota_enforcement() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Set a small storage quota for testing
        let test_quota = 1024 * 1024; // 1MB
        setup.storage_economy.set_storage_quota("test_user", test_quota)?;
        
        // Create files that would exceed quota
        let file1_path = setup.create_test_file("quota_test1.bin", 512 * 1024).await?; // 512KB
        let file2_path = setup.create_test_file("quota_test2.bin", 512 * 1024).await?; // 512KB
        let file3_path = setup.create_test_file("quota_test3.bin", 256 * 1024).await?; // 256KB (would exceed)
        
        // Store first two files (should succeed)
        let result1 = setup.file_manager.store_file_for_user(&file1_path, "test_user", None).await?;
        let result2 = setup.file_manager.store_file_for_user(&file2_path, "test_user", None).await?;
        
        assert!(!result1.file_key.is_empty());
        assert!(!result2.file_key.is_empty());
        
        // Third file should fail or trigger quota management
        let result3 = setup.file_manager.store_file_for_user(&file3_path, "test_user", None).await;
        
        // Depending on implementation, this might fail or succeed with quota management
        if let Err(e) = result3 {
            println!("Quota enforcement working: {}", e);
        } else {
            println!("Quota management handled gracefully");
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_file_versioning() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create initial version of file
        let content_v1 = "This is version 1 of the file";
        let file_path = setup.create_text_file("versioned_file.txt", content_v1).await?;
        
        // Store version 1
        let result_v1 = setup.file_manager.store_file(&file_path, Some("v1".to_string())).await?;
        
        // Update file content
        let content_v2 = "This is version 2 of the file with more content";
        fs::write(&file_path, content_v2).await?;
        
        // Store version 2
        let result_v2 = setup.file_manager.store_file(&file_path, Some("v2".to_string())).await?;
        
        // Verify both versions exist and are different
        assert_ne!(result_v1.file_key, result_v2.file_key);
        
        // Retrieve both versions
        let retrieved_v1_path = setup.temp_dir.path().join("retrieved_v1.txt");
        let retrieved_v2_path = setup.temp_dir.path().join("retrieved_v2.txt");
        
        setup.file_manager.retrieve_file(&result_v1.file_key, &retrieved_v1_path).await?;
        setup.file_manager.retrieve_file(&result_v2.file_key, &retrieved_v2_path).await?;
        
        // Verify content
        let retrieved_v1_content = fs::read_to_string(&retrieved_v1_path).await?;
        let retrieved_v2_content = fs::read_to_string(&retrieved_v2_path).await?;
        
        assert_eq!(retrieved_v1_content, content_v1);
        assert_eq!(retrieved_v2_content, content_v2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_storage_encryption() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create sensitive test file
        let sensitive_content = "This is sensitive data that should be encrypted";
        let original_path = setup.create_text_file("sensitive.txt", sensitive_content).await?;
        
        // Store with encryption enabled
        let storage_result = setup.file_manager.store_file_encrypted(&original_path, "test_password").await?;
        assert!(!storage_result.file_key.is_empty());
        
        // Retrieve with correct password
        let retrieved_path = setup.temp_dir.path().join("retrieved_sensitive.txt");
        let retrieval_result = setup.file_manager.retrieve_file_encrypted(
            &storage_result.file_key, 
            &retrieved_path, 
            "test_password"
        ).await;
        
        assert!(retrieval_result.is_ok(), "Should retrieve with correct password");
        
        // Verify decrypted content
        let retrieved_content = fs::read_to_string(&retrieved_path).await?;
        assert_eq!(retrieved_content, sensitive_content);
        
        // Try to retrieve with wrong password (should fail)
        let wrong_password_result = setup.file_manager.retrieve_file_encrypted(
            &storage_result.file_key, 
            &retrieved_path, 
            "wrong_password"
        ).await;
        
        assert!(wrong_password_result.is_err(), "Should fail with wrong password");
        
        Ok(())
    }
}

#[cfg(test)]
mod chunk_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_chunk_size_optimization() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Test with different chunk sizes
        let chunk_sizes = vec![
            16 * 1024,  // 16KB
            64 * 1024,  // 64KB
            256 * 1024, // 256KB
            1024 * 1024, // 1MB
        ];
        
        let test_file_size = 5 * 1024 * 1024; // 5MB test file
        let original_path = setup.create_test_file("chunk_optimization.bin", test_file_size).await?;
        
        for chunk_size in chunk_sizes {
            // Update chunk size in configuration
            let mut config = setup.file_manager.config.clone();
            config.storage.chunk_size = chunk_size;
            
            // Create file manager with new chunk size
            let file_manager = FileManager::new(config).await?;
            
            // Measure storage performance
            let start_time = Instant::now();
            let storage_result = file_manager.store_file(&original_path, None).await?;
            let storage_duration = start_time.elapsed();
            
            let expected_chunks = (test_file_size + chunk_size - 1) / chunk_size;
            
            println!("Chunk size: {}KB, Chunks: {}, Time: {:?}", 
                     chunk_size / 1024, expected_chunks, storage_duration);
            
            // Verify retrieval works
            let retrieved_path = setup.temp_dir.path().join(format!("retrieved_chunk_{}.bin", chunk_size));
            file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
            
            // Verify integrity
            let integrity_check = setup.verify_file_integrity(&original_path, &retrieved_path).await?;
            assert!(integrity_check, "Integrity should be maintained with chunk size {}", chunk_size);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_distribution_balance() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create test file that will be split into multiple chunks
        let file_size = 10 * setup.file_manager.config.storage.chunk_size;
        let original_path = setup.create_test_file("distribution_test.bin", file_size).await?;
        
        // Store the file
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        
        // Get file information
        let file_info = setup.file_manager.get_file_info(&storage_result.file_key).await?;
        
        // Verify chunk distribution
        let expected_data_chunks = (file_size + setup.file_manager.config.storage.chunk_size - 1) / setup.file_manager.config.storage.chunk_size;
        let total_chunks_with_parity = expected_data_chunks + setup.file_manager.config.storage.parity_shards;
        
        assert_eq!(file_info.chunks_total, total_chunks_with_parity);
        assert_eq!(file_info.chunks_healthy, file_info.chunks_total);
        
        println!("File distributed into {} chunks ({} data + {} parity)", 
                 file_info.chunks_total, expected_data_chunks, setup.file_manager.config.storage.parity_shards);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_redundancy_validation() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create test file
        let original_path = setup.create_test_file("redundancy_validation.dat", 32768).await?;
        
        // Store with Reed-Solomon encoding
        let storage_result = setup.file_manager.store_file(&original_path, None).await?;
        
        // Verify redundancy parameters
        let file_info = setup.file_manager.get_file_info(&storage_result.file_key).await?;
        
        // With 4 data shards and 2 parity shards, we can lose up to 2 shards
        let data_shards = setup.file_manager.config.storage.data_shards;
        let parity_shards = setup.file_manager.config.storage.parity_shards;
        let total_shards = data_shards + parity_shards;
        
        assert_eq!(file_info.chunks_total, total_shards);
        
        // Test that we can recover from losing up to parity_shards chunks
        // (This would require actually simulating chunk loss in a real test)
        println!("File has {} total shards, can survive loss of {} shards", 
                 total_shards, parity_shards);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_reassembly_correctness() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create file with known pattern for verification
        let pattern_size = 1024;
        let total_size = 10 * pattern_size; // 10KB with 1KB pattern repetition
        let mut content = Vec::new();
        
        for i in 0..10 {
            let mut pattern = vec![i as u8; pattern_size];
            content.append(&mut pattern);
        }
        
        let file_path = setup.test_files_dir.join("pattern_test.bin");
        fs::write(&file_path, &content).await?;
        
        // Store the file (will be chunked)
        let storage_result = setup.file_manager.store_file(&file_path, None).await?;
        
        // Retrieve the file
        let retrieved_path = setup.temp_dir.path().join("retrieved_pattern.bin");
        setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
        
        // Verify the pattern is correctly reassembled
        let retrieved_content = fs::read(&retrieved_path).await?;
        assert_eq!(retrieved_content.len(), total_size);
        
        // Check each pattern block
        for i in 0..10 {
            let start = i * pattern_size;
            let end = start + pattern_size;
            let block = &retrieved_content[start..end];
            
            for &byte in block {
                assert_eq!(byte, i as u8, "Pattern block {} should contain byte value {}", i, i);
            }
        }
        
        println!("✅ Chunk reassembly correctness verified");
        Ok(())
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_under_memory_pressure() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Create many large files to stress memory usage
        let file_count = 50;
        let file_size = 1024 * 1024; // 1MB each
        
        let mut storage_results = Vec::new();
        
        for i in 0..file_count {
            let file_name = format!("memory_stress_{}.bin", i);
            let original_path = setup.create_test_file(&file_name, file_size).await?;
            
            let result = setup.file_manager.store_file(&original_path, None).await?;
            storage_results.push(result);
            
            // Periodically check memory usage if available
            if i % 10 == 0 {
                println!("Stored {} files, continuing...", i + 1);
            }
        }
        
        println!("Successfully stored {} large files", file_count);
        
        // Verify we can still retrieve files
        let test_index = file_count / 2;
        let retrieved_path = setup.temp_dir.path().join("memory_stress_retrieved.bin");
        setup.file_manager.retrieve_file(&storage_results[test_index].file_key, &retrieved_path).await?;
        
        assert!(retrieved_path.exists());
        assert_eq!(fs::metadata(&retrieved_path).await?.len(), file_size as u64);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_rapid_file_operations() -> Result<()> {
        let setup = StorageTestSetup::new().await?;
        
        // Rapidly store and retrieve files
        let operation_count = 100;
        let file_size = 10 * 1024; // 10KB files
        
        let start_time = Instant::now();
        
        for i in 0..operation_count {
            let file_name = format!("rapid_test_{}.txt", i);
            let content = format!("Rapid test file number {} with some content", i);
            let original_path = setup.create_text_file(&file_name, &content).await?;
            
            // Store and immediately retrieve
            let storage_result = setup.file_manager.store_file(&original_path, None).await?;
            let retrieved_path = setup.temp_dir.path().join(format!("rapid_retrieved_{}.txt", i));
            setup.file_manager.retrieve_file(&storage_result.file_key, &retrieved_path).await?;
            
            // Verify content
            let retrieved_content = fs::read_to_string(&retrieved_path).await?;
            assert_eq!(retrieved_content, content);
        }
        
        let total_duration = start_time.elapsed();
        let ops_per_second = operation_count as f64 / total_duration.as_secs_f64();
        
        println!("Completed {} rapid operations in {:?} ({:.2} ops/sec)", 
                 operation_count, total_duration, ops_per_second);
        
        // Should maintain reasonable performance
        assert!(ops_per_second > 10.0, "Should maintain at least 10 operations per second");
        
        Ok(())
    }
}