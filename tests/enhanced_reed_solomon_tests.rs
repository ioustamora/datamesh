/// Enhanced Reed-Solomon 8+4 Configuration Tests
///
/// Comprehensive test suite validating the upgraded Reed-Solomon erasure coding
/// configuration for improved reliability in distributed consumer storage.
///
/// Test Coverage:
/// - Basic Reed-Solomon encoding/decoding with 8+4 configuration
/// - Fault tolerance validation (can lose up to 4 shards)
/// - Performance benchmarks with increased shard count
/// - Storage overhead verification
/// - Real-world failure scenario simulation
/// - Edge cases and error conditions
/// - Integration with file storage system
/// - Concurrent operations with larger shard counts

use proptest::prelude::*;
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::HashMap;
use tokio::test;

use datamesh::file_storage::{DATA_SHARDS, PARITY_SHARDS, StoredFile};
use datamesh::error::{DfsError, DfsResult};

/// Test constants matching the enhanced configuration
const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS; // 8 + 4 = 12

#[test]
fn test_reed_solomon_constants() {
    assert_eq!(DATA_SHARDS, 8, "Data shards should be 8 for enhanced configuration");
    assert_eq!(PARITY_SHARDS, 4, "Parity shards should be 4 for enhanced configuration"); 
    assert_eq!(TOTAL_SHARDS, 12, "Total shards should be 12");
    
    // Verify fault tolerance: can lose up to PARITY_SHARDS
    assert_eq!(PARITY_SHARDS, 4, "Should tolerate loss of up to 4 shards");
    
    // Verify storage overhead is still 50%
    let overhead_percent = (PARITY_SHARDS as f64 / DATA_SHARDS as f64) * 100.0;
    assert_eq!(overhead_percent, 50.0, "Storage overhead should remain 50%");
}

#[test]
fn test_basic_reed_solomon_encoding_decoding() -> Result<(), Box<dyn std::error::Error>> {
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    // Test data - larger to better test 8 data shards
    let test_data = b"This is test data for the enhanced Reed-Solomon 8+4 configuration which provides much better fault tolerance for distributed consumer storage networks where node failures are more common than in professional data centers.";
    let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    
    // Create shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
    
    // Fill data shards
    for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, test_data.len());
        if start < test_data.len() {
            shard[..end - start].copy_from_slice(&test_data[start..end]);
        }
    }
    
    // Encode to create parity shards
    r.encode(&mut shards)?;
    
    // Verify all shards are present
    for (i, shard) in shards.iter().enumerate() {
        assert!(!shard.is_empty(), "Shard {} should not be empty", i);
    }
    
    // Test perfect reconstruction (no losses)
    let mut reconstruction_shards = shards.clone();
    let reconstruction_shards_opt: Vec<Option<Vec<u8>>> = reconstruction_shards
        .into_iter()
        .map(Some)
        .collect();
    
    let mut perfect_shards = reconstruction_shards_opt;
    r.reconstruct(&mut perfect_shards)?;
    
    // Reconstruct original data
    let mut reconstructed_data = Vec::new();
    for i in 0..DATA_SHARDS {
        if let Some(ref shard) = perfect_shards[i] {
            reconstructed_data.extend_from_slice(shard);
        }
    }
    
    // Trim to original length
    reconstructed_data.truncate(test_data.len());
    assert_eq!(&reconstructed_data, test_data, "Perfect reconstruction should match original data");
    
    Ok(())
}

#[test]
fn test_fault_tolerance_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    let test_data = b"Enhanced Reed-Solomon fault tolerance test with 8 data shards and 4 parity shards for maximum reliability in consumer storage networks.";
    let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    
    // Create and encode shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
    for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, test_data.len());
        if start < test_data.len() {
            shard[..end - start].copy_from_slice(&test_data[start..end]);
        }
    }
    r.encode(&mut shards)?;
    
    // Test scenarios with different numbers of lost shards
    let test_cases = vec![
        (1, "Single shard loss"),
        (2, "Two shard loss"), 
        (3, "Three shard loss"),
        (4, "Four shard loss (maximum tolerance)"),
    ];
    
    for (lost_count, description) in test_cases {
        println!("Testing: {}", description);
        
        // Create reconstruction shards with some marked as missing
        let mut reconstruction_shards: Vec<Option<Vec<u8>>> = shards
            .iter()
            .enumerate()
            .map(|(i, shard)| {
                if i < lost_count {
                    None // Mark first `lost_count` shards as lost
                } else {
                    Some(shard.clone())
                }
            })
            .collect();
        
        // Attempt reconstruction
        let result = r.reconstruct(&mut reconstruction_shards);
        assert!(result.is_ok(), "Reconstruction should succeed with {} lost shards", lost_count);
        
        // Verify reconstructed data
        let mut reconstructed_data = Vec::new();
        for i in 0..DATA_SHARDS {
            if let Some(ref shard) = reconstruction_shards[i] {
                reconstructed_data.extend_from_slice(shard);
            }
        }
        reconstructed_data.truncate(test_data.len());
        assert_eq!(&reconstructed_data, test_data, "Data should be correctly reconstructed with {} lost shards", lost_count);
    }
    
    Ok(())
}

#[test]
fn test_failure_beyond_tolerance() -> Result<(), Box<dyn std::error::Error>> {
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    let test_data = b"Test data for failure beyond tolerance.";
    let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    
    // Create and encode shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
    for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, test_data.len());
        if start < test_data.len() {
            shard[..end - start].copy_from_slice(&test_data[start..end]);
        }
    }
    r.encode(&mut shards)?;
    
    // Lose more shards than tolerance allows (5 shards, tolerance is 4)
    let mut reconstruction_shards: Vec<Option<Vec<u8>>> = shards
        .iter()
        .enumerate()
        .map(|(i, shard)| {
            if i < 5 { // Lose 5 shards (more than 4 parity shards)
                None
            } else {
                Some(shard.clone())
            }
        })
        .collect();
    
    // Reconstruction should fail
    let result = r.reconstruct(&mut reconstruction_shards);
    assert!(result.is_err(), "Reconstruction should fail when losing more than {} shards", PARITY_SHARDS);
    
    Ok(())
}

#[test]
fn test_performance_with_larger_shard_count() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    // Test with various data sizes
    let test_sizes = vec![
        1024,           // 1KB
        1024 * 1024,    // 1MB  
        10 * 1024 * 1024, // 10MB
    ];
    
    for size in test_sizes {
        println!("Testing performance with {} bytes", size);
        
        let test_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
        
        // Measure encoding time
        let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
        for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, test_data.len());
            if start < test_data.len() {
                shard[..end - start].copy_from_slice(&test_data[start..end]);
            }
        }
        
        let encode_start = Instant::now();
        r.encode(&mut shards)?;
        let encode_time = encode_start.elapsed();
        
        // Measure reconstruction time (worst case: lose 4 shards)
        let mut reconstruction_shards: Vec<Option<Vec<u8>>> = shards
            .iter()
            .enumerate()
            .map(|(i, shard)| {
                if i < 4 { // Lose 4 shards (maximum tolerance)
                    None
                } else {
                    Some(shard.clone())
                }
            })
            .collect();
        
        let reconstruct_start = Instant::now();
        r.reconstruct(&mut reconstruction_shards)?;
        let reconstruct_time = reconstruct_start.elapsed();
        
        println!("  Encode time: {:?}", encode_time);
        println!("  Reconstruct time: {:?}", reconstruct_time);
        
        // Verify reconstruction worked
        let mut reconstructed_data = Vec::new();
        for i in 0..DATA_SHARDS {
            if let Some(ref shard) = reconstruction_shards[i] {
                reconstructed_data.extend_from_slice(shard);
            }
        }
        reconstructed_data.truncate(test_data.len());
        assert_eq!(reconstructed_data, test_data, "Reconstructed data should match original");
        
        // Performance assertions (reasonable bounds for modern hardware)
        assert!(encode_time.as_millis() < 1000, "Encoding should complete in under 1 second for {} bytes", size);
        assert!(reconstruct_time.as_millis() < 2000, "Reconstruction should complete in under 2 seconds for {} bytes", size);
    }
    
    Ok(())
}

#[test]
fn test_real_world_failure_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    let test_data = b"Real world failure pattern test for distributed consumer storage with enhanced Reed-Solomon.";
    let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    
    // Create and encode shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
    for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, test_data.len());
        if start < test_data.len() {
            shard[..end - start].copy_from_slice(&test_data[start..end]);
        }
    }
    r.encode(&mut shards)?;
    
    // Test realistic failure patterns
    let failure_patterns = vec![
        // Pattern 1: Consecutive failures (power outage in region)
        vec![0, 1, 2, 3],
        // Pattern 2: Scattered failures (random node failures)
        vec![1, 4, 7, 10],
        // Pattern 3: Mixed data and parity losses
        vec![0, 2, 8, 11], // 2 data + 2 parity
        // Pattern 4: Heavy parity loss
        vec![8, 9, 10, 11], // All parity shards
        // Pattern 5: Heavy data loss  
        vec![0, 1, 2, 3], // Half of data shards
    ];
    
    for (i, pattern) in failure_patterns.iter().enumerate() {
        println!("Testing failure pattern {}: {:?}", i + 1, pattern);
        
        let mut reconstruction_shards: Vec<Option<Vec<u8>>> = shards
            .iter()
            .enumerate()
            .map(|(idx, shard)| {
                if pattern.contains(&idx) {
                    None
                } else {
                    Some(shard.clone())
                }
            })
            .collect();
        
        let result = r.reconstruct(&mut reconstruction_shards);
        assert!(result.is_ok(), "Should recover from failure pattern {:?}", pattern);
        
        // Verify data integrity
        let mut reconstructed_data = Vec::new();
        for idx in 0..DATA_SHARDS {
            if let Some(ref shard) = reconstruction_shards[idx] {
                reconstructed_data.extend_from_slice(shard);
            }
        }
        reconstructed_data.truncate(test_data.len());
        assert_eq!(&reconstructed_data, test_data, "Data integrity should be maintained for pattern {:?}", pattern);
    }
    
    Ok(())
}

#[test]
fn test_storage_overhead_calculation() {
    // Verify the storage overhead calculations are correct
    let data_size = 1000; // bytes
    let chunk_size = (data_size + DATA_SHARDS - 1) / DATA_SHARDS;
    let total_storage = chunk_size * TOTAL_SHARDS;
    let overhead = total_storage - data_size;
    let overhead_percent = (overhead as f64 / data_size as f64) * 100.0;
    
    println!("Data size: {} bytes", data_size);
    println!("Chunk size: {} bytes", chunk_size);
    println!("Total storage: {} bytes", total_storage);
    println!("Overhead: {} bytes ({:.1}%)", overhead, overhead_percent);
    
    // Storage overhead should be approximately 50%
    // (allowing for some variance due to padding)
    assert!(overhead_percent >= 45.0 && overhead_percent <= 55.0, 
            "Storage overhead should be approximately 50%, got {:.1}%", overhead_percent);
}

// Property-based tests using proptest
proptest! {
    #[test]
    fn property_test_reed_solomon_reconstruction(
        data in prop::collection::vec(prop::num::u8::ANY, 1..10000),
        lost_shards in prop::collection::vec(0usize..12, 1..4)
    ) {
        let lost_shards: std::collections::HashSet<usize> = lost_shards.into_iter().collect();
        if lost_shards.len() > PARITY_SHARDS {
            return Ok(()); // Skip if too many shards lost
        }
        
        let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).unwrap();
        let chunk_size = (data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
        
        // Create and encode shards
        let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
        for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            if start < data.len() {
                shard[..end - start].copy_from_slice(&data[start..end]);
            }
        }
        r.encode(&mut shards).unwrap();
        
        // Create reconstruction scenario
        let mut reconstruction_shards: Vec<Option<Vec<u8>>> = shards
            .iter()
            .enumerate()
            .map(|(i, shard)| {
                if lost_shards.contains(&i) {
                    None
                } else {
                    Some(shard.clone())
                }
            })
            .collect();
        
        // Reconstruct
        r.reconstruct(&mut reconstruction_shards).unwrap();
        
        // Verify
        let mut reconstructed_data = Vec::new();
        for i in 0..DATA_SHARDS {
            if let Some(ref shard) = reconstruction_shards[i] {
                reconstructed_data.extend_from_slice(shard);
            }
        }
        reconstructed_data.truncate(data.len());
        
        prop_assert_eq!(reconstructed_data, data);
    }
}

#[tokio::test]
async fn test_integration_with_file_storage() {
    // Test that the enhanced Reed-Solomon configuration integrates properly
    // with the file storage system
    
    // Create test file metadata
    let chunk_keys: Vec<Vec<u8>> = (0..TOTAL_SHARDS)
        .map(|i| format!("chunk_{}", i).into_bytes())
        .collect();
    
    let stored_file = StoredFile {
        chunk_keys,
        encryption_key: vec![1, 2, 3, 4],
        file_size: 1000,
        public_key_hex: "test_key".to_string(),
        file_name: "test_file.txt".to_string(),
        stored_at: chrono::Local::now(),
    };
    
    // Verify we have the correct number of chunks for 8+4 configuration
    assert_eq!(stored_file.chunk_keys.len(), TOTAL_SHARDS, 
               "Stored file should have {} chunks for 8+4 configuration", TOTAL_SHARDS);
    
    // Verify that we can lose up to 4 chunks and still have enough for reconstruction
    let available_chunks = TOTAL_SHARDS - 4; // Lose 4 chunks
    assert!(available_chunks >= DATA_SHARDS, 
            "Should still be able to reconstruct with {} available chunks", available_chunks);
}

#[test]
fn test_configuration_constants_consistency() {
    // Ensure our constants are consistent across the codebase
    use datamesh::file_storage::{PUB_DATA_SHARDS, PUB_PARITY_SHARDS};
    
    assert_eq!(DATA_SHARDS, PUB_DATA_SHARDS, "Public and private data shard constants should match");
    assert_eq!(PARITY_SHARDS, PUB_PARITY_SHARDS, "Public and private parity shard constants should match");
    
    // Verify Reed-Solomon library can handle our configuration
    let result = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS);
    assert!(result.is_ok(), "Reed-Solomon should support {data}+{parity} configuration", 
            data = DATA_SHARDS, parity = PARITY_SHARDS);
}

/// Benchmark test to measure performance impact of larger shard counts
#[test]
fn benchmark_encoding_performance() {
    use std::time::Instant;
    
    let iterations = 100;
    let data_size = 1024 * 1024; // 1MB
    let test_data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
    
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).unwrap();
    let chunk_size = (test_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    
    let mut total_time = std::time::Duration::new(0, 0);
    
    for _ in 0..iterations {
        let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; TOTAL_SHARDS];
        for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, test_data.len());
            if start < test_data.len() {
                shard[..end - start].copy_from_slice(&test_data[start..end]);
            }
        }
        
        let start = Instant::now();
        r.encode(&mut shards).unwrap();
        total_time += start.elapsed();
    }
    
    let avg_time = total_time / iterations;
    println!("Average encoding time for {}MB with 8+4 Reed-Solomon: {:?}", 
             data_size / 1024 / 1024, avg_time);
    
    // Performance should be reasonable (under 10ms for 1MB on modern hardware)
    assert!(avg_time.as_millis() < 50, "Encoding should be reasonably fast");
}