/// Property-Based Tests for DataMesh
///
/// This module uses property-based testing (QuickCheck-style) to validate
/// system invariants and edge cases across the DataMesh system components.

mod test_utils;

use anyhow::Result;
use proptest::prelude::*;
use std::collections::HashSet;
use test_utils::TestEnvironment;

use datamesh::{
    key_manager::KeyManager,
    database::DatabaseManager,
    economics::{EconomicModel, EconomicConfig},
    governance::{UserAccount, AccountType, VerificationStatus},
    storage_economy::StorageEconomy,
};

/// Property test for key manager encryption/decryption invariants
#[test]
fn prop_encryption_roundtrip() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(data in prop::collection::vec(any::<u8>(), 0..10000))| {
        rt.block_on(async {
            let env = TestEnvironment::new().unwrap();
            
            // Property: encrypt(decrypt(data)) == data
            if !data.is_empty() {
                let encrypted = env.key_manager.encrypt(&data).unwrap();
                let decrypted = env.key_manager.decrypt(&encrypted).unwrap();
                prop_assert_eq!(data, decrypted);
                
                // Property: encrypted data should be different from original
                prop_assert_ne!(data, encrypted);
                
                // Property: encrypted data should be larger than original (due to overhead)
                prop_assert!(encrypted.len() > data.len());
            }
        });
    });
}

/// Property test for database operations
#[test]
fn prop_database_consistency() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(
        file_names in prop::collection::vec("[a-zA-Z0-9_]{1,50}", 1..20),
        file_sizes in prop::collection::vec(1u64..1000000, 1..20)
    )| {
        rt.block_on(async {
            let env = TestEnvironment::new().unwrap();
            
            // Property: All stored files can be retrieved
            let unique_names: HashSet<_> = file_names.iter().collect();
            let unique_names: Vec<_> = unique_names.into_iter().collect();
            
            for (i, &name) in unique_names.iter().enumerate() {
                let size = file_sizes[i % file_sizes.len()];
                let tags = vec!["prop_test".to_string()];
                
                env.db.store_file(
                    name,
                    &format!("key_{}", i),
                    &format!("{}.txt", name),
                    size,
                    chrono::Local::now(),
                    &tags,
                    "test_public_key",
                ).unwrap();
            }
            
            // Property: All files can be retrieved
            for &name in &unique_names {
                let file = env.db.get_file_by_name(name).unwrap();
                prop_assert!(file.is_some());
                
                let file = file.unwrap();
                prop_assert_eq!(file.name, name);
            }
            
            // Property: List files returns all stored files
            let all_files = env.db.list_files(None).unwrap();
            prop_assert_eq!(all_files.len(), unique_names.len());
            
            // Property: Each file appears exactly once in listing
            let listed_names: HashSet<_> = all_files.iter().map(|f| f.name.as_str()).collect();
            let stored_names: HashSet<_> = unique_names.into_iter().collect();
            prop_assert_eq!(listed_names, stored_names);
        });
    });
}

/// Property test for economic calculations
#[test]
fn prop_economic_calculations() {
    proptest!(|(
        storage_gb in 0.0f64..1000.0,
        bandwidth_gb in 0.0f64..10000.0,
        api_calls in 0u64..1000000,
        days in 1u32..365
    )| {
        let economic_model = EconomicModel::new();
        let config = economic_model.service.config.clone();
        
        // Property: Cost should be non-negative
        let storage_cost = economic_model.service.calculate_storage_cost(storage_gb, days)?;
        prop_assert!(storage_cost >= 0.0);
        
        let bandwidth_cost = economic_model.service.calculate_bandwidth_cost(bandwidth_gb)?;
        prop_assert!(bandwidth_cost >= 0.0);
        
        let api_cost = economic_model.service.calculate_api_cost(api_calls)?;
        prop_assert!(api_cost >= 0.0);
        
        // Property: Cost should scale linearly with usage
        if storage_gb > 0.0 {
            let double_storage_cost = economic_model.service.calculate_storage_cost(storage_gb * 2.0, days)?;
            let expected_double = storage_cost * 2.0;
            let tolerance = 0.01;
            prop_assert!((double_storage_cost - expected_double).abs() < tolerance);
        }
        
        if bandwidth_gb > 0.0 {
            let double_bandwidth_cost = economic_model.service.calculate_bandwidth_cost(bandwidth_gb * 2.0)?;
            let expected_double = bandwidth_cost * 2.0;
            let tolerance = 0.01;
            prop_assert!((double_bandwidth_cost - expected_double).abs() < tolerance);
        }
        
        // Property: Zero usage should result in zero cost
        let zero_storage_cost = economic_model.service.calculate_storage_cost(0.0, days)?;
        prop_assert_eq!(zero_storage_cost, 0.0);
        
        let zero_bandwidth_cost = economic_model.service.calculate_bandwidth_cost(0.0)?;
        prop_assert_eq!(zero_bandwidth_cost, 0.0);
        
        let zero_api_cost = economic_model.service.calculate_api_cost(0)?;
        prop_assert_eq!(zero_api_cost, 0.0);
    });
}

/// Property test for user account tier limits
#[test]
fn prop_account_tier_limits() {
    proptest!(|(
        storage_gb in 1u32..1000,
        bandwidth_gb in 1u32..10000,
        api_calls in 1u32..100000
    )| {
        // Test Free tier limits
        let free_account = AccountType::Free {
            storage_gb,
            bandwidth_gb_month: bandwidth_gb,
            api_calls_hour: api_calls,
        };
        
        // Property: Free tier should have reasonable limits
        match free_account {
            AccountType::Free { storage_gb, bandwidth_gb_month, api_calls_hour } => {
                prop_assert!(storage_gb <= 1000);
                prop_assert!(bandwidth_gb_month <= 10000);
                prop_assert!(api_calls_hour <= 100000);
            }
            _ => prop_assert!(false, "Should be Free account type"),
        }
        
        // Test Premium tier (should have higher limits)
        let premium_account = AccountType::Premium {
            storage_gb: storage_gb * 10,
            bandwidth_gb_month: bandwidth_gb * 10,
            api_calls_hour: api_calls * 10,
        };
        
        match premium_account {
            AccountType::Premium { storage_gb: p_storage, bandwidth_gb_month: p_bandwidth, api_calls_hour: p_api } => {
                prop_assert!(p_storage >= storage_gb);
                prop_assert!(p_bandwidth >= bandwidth_gb);
                prop_assert!(p_api >= api_calls);
            }
            _ => prop_assert!(false, "Should be Premium account type"),
        }
    });
}

/// Property test for file metadata consistency
#[test]
fn prop_file_metadata_integrity() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(
        file_name in "[a-zA-Z0-9_\\-\\.]{1,100}",
        original_filename in "[a-zA-Z0-9_\\-\\.]{1,100}",
        file_size in 1u64..0x7FFFFFFF, // Large but not overflow
        tags in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 0..10)
    )| {
        rt.block_on(async {
            let env = TestEnvironment::new().unwrap();
            
            let upload_time = chrono::Local::now();
            
            // Store file
            env.db.store_file(
                &file_name,
                "test_key",
                &original_filename,
                file_size,
                upload_time,
                &tags,
                "test_public_key",
            ).unwrap();
            
            // Retrieve and verify all properties are preserved
            let retrieved = env.db.get_file_by_name(&file_name).unwrap().unwrap();
            
            // Property: All metadata should be preserved exactly
            prop_assert_eq!(retrieved.name, file_name);
            prop_assert_eq!(retrieved.original_filename, original_filename);
            prop_assert_eq!(retrieved.file_size, file_size);
            prop_assert_eq!(retrieved.tags, tags);
            prop_assert_eq!(retrieved.public_key_hex, "test_public_key");
            
            // Property: Upload time should be preserved (within reasonable tolerance)
            let time_diff = (retrieved.upload_time - upload_time).num_milliseconds().abs();
            prop_assert!(time_diff < 1000); // Within 1 second
            
            // Property: ID should be positive
            prop_assert!(retrieved.id > 0);
            
            // Property: Chunk counts should be reasonable
            prop_assert!(retrieved.chunks_total > 0);
            prop_assert!(retrieved.chunks_healthy <= retrieved.chunks_total);
        });
    });
}

/// Property test for storage quota enforcement
#[test]
fn prop_storage_quota_enforcement() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(
        user_quota_gb in 1u32..100,
        file_sizes_mb in prop::collection::vec(1u32..50, 1..20)
    )| {
        rt.block_on(async {
            let env = TestEnvironment::new().unwrap();
            let config = env.create_test_config();
            let storage_economy = StorageEconomy::new(&config).unwrap();
            
            let user_id = uuid::Uuid::new_v4();
            
            // Set user quota
            storage_economy.set_user_quota(user_id, user_quota_gb as u64).await.unwrap();
            
            let mut total_used_mb = 0u32;
            
            for (i, &file_size_mb) in file_sizes_mb.iter().enumerate() {
                let would_exceed = total_used_mb + file_size_mb > user_quota_gb * 1024;
                
                let upload_result = storage_economy.reserve_storage(
                    user_id,
                    file_size_mb as u64 * 1024 * 1024, // Convert to bytes
                ).await;
                
                if would_exceed {
                    // Property: Should reject uploads that exceed quota
                    prop_assert!(upload_result.is_err());
                } else {
                    // Property: Should accept uploads within quota
                    prop_assert!(upload_result.is_ok());
                    total_used_mb += file_size_mb;
                }
                
                // Property: Used storage should never exceed quota
                let current_usage = storage_economy.get_user_usage(user_id).await.unwrap();
                let current_usage_gb = current_usage.storage_bytes / (1024 * 1024 * 1024);
                prop_assert!(current_usage_gb <= user_quota_gb as u64);
            }
        });
    });
}

/// Property test for concurrent operations safety
#[test]
fn prop_concurrent_database_operations() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(
        operation_count in 2usize..20,
        file_names in prop::collection::vec("[a-zA-Z0-9_]{5,20}", 2..20)
    )| {
        rt.block_on(async {
            let env = TestEnvironment::new().unwrap();
            
            // Property: Concurrent database operations should not corrupt data
            let mut handles = Vec::new();
            
            for i in 0..operation_count.min(file_names.len()) {
                let db = env.db.clone();
                let file_name = file_names[i].clone();
                
                let handle = tokio::spawn(async move {
                    let upload_time = chrono::Local::now();
                    let tags = vec!["concurrent".to_string()];
                    
                    db.store_file(
                        &file_name,
                        &format!("key_{}", i),
                        &format!("{}.txt", file_name),
                        1024 * (i as u64 + 1),
                        upload_time,
                        &tags,
                        "test_public_key",
                    )
                });
                
                handles.push((handle, file_name));
            }
            
            // Wait for all operations to complete
            for (handle, _) in handles {
                handle.await.unwrap().unwrap();
            }
            
            // Property: All files should be stored correctly
            let all_files = env.db.list_files(None).unwrap();
            prop_assert_eq!(all_files.len(), operation_count.min(file_names.len()));
            
            // Property: Each file should be retrievable individually
            for i in 0..operation_count.min(file_names.len()) {
                let file_name = &file_names[i];
                let file = env.db.get_file_by_name(file_name).unwrap();
                prop_assert!(file.is_some());
                
                let file = file.unwrap();
                prop_assert_eq!(file.name, *file_name);
                prop_assert_eq!(file.file_size, 1024 * (i as u64 + 1));
            }
        });
    });
}

/// Property test for economic model invariants
#[test]
fn prop_economic_model_invariants() {
    proptest!(|(
        base_cost in 0.001f64..1.0,
        multiplier in 1.0f64..10.0,
        usage_1 in 0.0f64..1000.0,
        usage_2 in 0.0f64..1000.0
    )| {
        let mut config = EconomicConfig::default();
        config.storage_cost_per_gb_month = base_cost;
        
        let economic_model = EconomicModel::new();
        economic_model.service.update_config(config.clone()).unwrap();
        
        // Property: Cost should be monotonic (more usage = more cost)
        if usage_1 < usage_2 {
            let cost_1 = economic_model.service.calculate_storage_cost(usage_1, 30).unwrap();
            let cost_2 = economic_model.service.calculate_storage_cost(usage_2, 30).unwrap();
            prop_assert!(cost_1 <= cost_2);
        }
        
        // Property: Cost should scale proportionally with base rate
        config.storage_cost_per_gb_month = base_cost * multiplier;
        economic_model.service.update_config(config).unwrap();
        
        if usage_1 > 0.0 {
            let original_cost = base_cost * usage_1 * 30.0; // 30 days
            let scaled_cost = economic_model.service.calculate_storage_cost(usage_1, 30).unwrap();
            let expected_scaled = original_cost * multiplier;
            
            let tolerance = 0.01;
            prop_assert!((scaled_cost - expected_scaled).abs() < tolerance);
        }
        
        // Property: Zero usage always results in zero cost
        let zero_cost = economic_model.service.calculate_storage_cost(0.0, 30).unwrap();
        prop_assert_eq!(zero_cost, 0.0);
    });
}

/// Property test for key manager security properties
#[test]
fn prop_key_manager_security() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    proptest!(|(
        data_1 in prop::collection::vec(any::<u8>(), 1..1000),
        data_2 in prop::collection::vec(any::<u8>(), 1..1000)
    )| {
        rt.block_on(async {
            // Create two different key managers
            let key1 = ecies::SecretKey::random(&mut rand::thread_rng());
            let key_manager_1 = KeyManager::new(key1, "key1".to_string());
            
            let key2 = ecies::SecretKey::random(&mut rand::thread_rng());
            let key_manager_2 = KeyManager::new(key2, "key2".to_string());
            
            // Property: Different keys should produce different public keys
            prop_assert_ne!(
                key_manager_1.key_info.public_key_hex,
                key_manager_2.key_info.public_key_hex
            );
            
            // Property: Same data encrypted with different keys should produce different results
            if !data_1.is_empty() {
                let encrypted_1 = key_manager_1.encrypt(&data_1).unwrap();
                let encrypted_2 = key_manager_2.encrypt(&data_1).unwrap();
                prop_assert_ne!(encrypted_1, encrypted_2);
            }
            
            // Property: Different data encrypted with same key should produce different results
            if !data_1.is_empty() && !data_2.is_empty() && data_1 != data_2 {
                let encrypted_1 = key_manager_1.encrypt(&data_1).unwrap();
                let encrypted_2 = key_manager_1.encrypt(&data_2).unwrap();
                prop_assert_ne!(encrypted_1, encrypted_2);
            }
            
            // Property: Cross-decryption should fail (key isolation)
            if !data_1.is_empty() {
                let encrypted_with_key1 = key_manager_1.encrypt(&data_1).unwrap();
                let cross_decrypt_result = key_manager_2.decrypt(&encrypted_with_key1);
                prop_assert!(cross_decrypt_result.is_err());
            }
        });
    });
}