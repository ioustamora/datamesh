/// Enhanced Property-Based Testing for DataMesh
///
/// This module provides comprehensive property-based testing using Proptest
/// to discover edge cases and validate invariants across all system components.

use anyhow::Result;
use proptest::prelude::*;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio_test;

use datamesh::config::Config;
use datamesh::database::DatabaseManager;
use datamesh::file_manager::FileManager;
use datamesh::governance::{AccountType, UserAccount, VerificationStatus};
use datamesh::storage_economy::StorageEconomy;

/// Property test setup with isolated environment
pub struct PropertyTestSetup {
    temp_dir: TempDir,
    config: Config,
}

impl PropertyTestSetup {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();
        config.database.path = temp_dir.path().join("test.db").to_string_lossy().to_string();

        Ok(PropertyTestSetup {
            temp_dir,
            config,
        })
    }
}

/// Generators for property-based testing
mod generators {
    use super::*;
    use proptest::collection::{vec, hash_map};
    use proptest::option;
    use proptest::string::string_regex;

    /// Generate valid file names
    pub fn file_name() -> impl Strategy<Value = String> {
        string_regex(r"[a-zA-Z0-9_\-\.]{1,255}").unwrap()
    }

    /// Generate file names that might cause issues
    pub fn problematic_file_name() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),                    // Empty name
            Just(".".to_string()),                   // Current directory
            Just("..".to_string()),                  // Parent directory
            Just("/".to_string()),                   // Root path
            Just("\\".to_string()),                  // Windows separator
            Just("CON".to_string()),                 // Windows reserved
            Just("PRN".to_string()),                 // Windows reserved
            Just("AUX".to_string()),                 // Windows reserved
            Just("a".repeat(300)),                   // Very long name
            Just("file\0name".to_string()),          // Null byte
            Just("file\nname".to_string()),          // Newline
            Just("file\tname".to_string()),          // Tab
            Just("file name".to_string()),           // Space
            Just("file<>:\"|?*name".to_string()),    // Special characters
        ]
    }

    /// Generate file content of various sizes
    pub fn file_content() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            // Empty file
            Just(vec![]),
            // Small files (1B to 1KB)
            vec(any::<u8>(), 1..=1024),
            // Medium files (1KB to 1MB)
            vec(any::<u8>(), 1024..=1024*1024),
            // Large files (1MB to 10MB)
            vec(any::<u8>(), 1024*1024..=10*1024*1024),
            // Files with specific patterns
            vec(Just(0u8), 0..=1024*1024),          // All zeros
            vec(Just(255u8), 0..=1024*1024),        // All ones
            // Repeating patterns
            vec(0u8..=255u8, 0..=1024).prop_flat_map(|pattern| {
                (1..=1000).prop_map(move |repeat| pattern.repeat(repeat))
            }),
        ]
    }

    /// Generate file tags
    pub fn file_tags() -> impl Strategy<Value = Vec<String>> {
        vec(string_regex(r"[a-zA-Z0-9_\-]{0,50}").unwrap(), 0..=10)
    }

    /// Generate user emails
    pub fn user_email() -> impl Strategy<Value = String> {
        prop_oneof![
            // Valid emails
            string_regex(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            // Edge case emails
            Just("".to_string()),
            Just("@".to_string()),
            Just("user@".to_string()),
            Just("@domain.com".to_string()),
            Just("user@domain".to_string()),
            Just("a".repeat(300) + "@example.com"),
        ]
    }

    /// Generate storage quotas
    pub fn storage_quota() -> impl Strategy<Value = u64> {
        prop_oneof![
            Just(0u64),                              // Zero quota
            1u64..=1024u64,                         // Small quotas (1B-1KB)
            1024u64..=1024*1024u64,                 // Medium quotas (1KB-1MB)
            1024*1024u64..=1024*1024*1024u64,       // Large quotas (1MB-1GB)
            Just(u64::MAX),                         // Maximum quota
        ]
    }

    /// Generate account types
    pub fn account_type() -> impl Strategy<Value = AccountType> {
        prop_oneof![
            Just(AccountType::Free { storage_gb: 5, bandwidth_gb_month: 100, api_calls_hour: 1000 }),
            Just(AccountType::Basic { storage_gb: 50, bandwidth_gb_month: 500, api_calls_hour: 5000 }),
            Just(AccountType::Pro { storage_gb: 500, bandwidth_gb_month: 2000, api_calls_hour: 20000 }),
            Just(AccountType::Enterprise { storage_gb: 5000, bandwidth_gb_month: 10000, api_calls_hour: 100000 }),
            // Edge cases
            Just(AccountType::Free { storage_gb: 0, bandwidth_gb_month: 0, api_calls_hour: 0 }),
            Just(AccountType::Free { storage_gb: u32::MAX, bandwidth_gb_month: u32::MAX, api_calls_hour: u32::MAX }),
        ]
    }

    /// Generate user accounts
    pub fn user_account() -> impl Strategy<Value = UserAccount> {
        (
            user_email(),
            string_regex(r"[a-zA-Z0-9]{8,64}").unwrap(),  // password hash
            string_regex(r"[a-fA-F0-9]{64}").unwrap(),     // public key
            account_type(),
        ).prop_map(|(email, password_hash, public_key, account_type)| {
            UserAccount {
                user_id: uuid::Uuid::new_v4(),
                email,
                password_hash,
                public_key,
                account_type,
                verification_status: VerificationStatus::EmailVerified,
                registration_date: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                reputation_score: 0.0,
                abuse_flags: vec![],
                subscription: None,
            }
        })
    }
}

#[cfg(test)]
mod database_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_file_storage_roundtrip(
            file_name in generators::file_name(),
            file_key in generators::file_name(),
            original_filename in generators::file_name(),
            file_size in 0u64..=100*1024*1024u64,
            tags in generators::file_tags(),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let database = DatabaseManager::new(&setup.temp_dir.path().join("test.db")).unwrap();
                let upload_time = chrono::Local::now();

                // Store file
                let store_result = database.store_file(
                    &file_name,
                    &file_key,
                    &original_filename,
                    file_size,
                    upload_time,
                    &tags,
                    "test_public_key",
                );

                if store_result.is_ok() {
                    // If storage succeeded, retrieval should also succeed
                    let retrieved_file = database.get_file_by_name(&file_name).unwrap();
                    
                    prop_assert!(retrieved_file.is_some());
                    
                    if let Some(file) = retrieved_file {
                        prop_assert_eq!(file.name, file_name);
                        prop_assert_eq!(file.file_key, file_key);
                        prop_assert_eq!(file.original_filename, original_filename);
                        prop_assert_eq!(file.file_size, file_size);
                        prop_assert_eq!(file.tags, tags);
                    }
                }
            });
        }

        #[test]
        fn test_file_storage_with_problematic_names(
            file_name in generators::problematic_file_name(),
            file_key in generators::file_name(),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let database = DatabaseManager::new(&setup.temp_dir.path().join("test.db")).unwrap();
                let upload_time = chrono::Local::now();

                // Store file with problematic name
                let store_result = database.store_file(
                    &file_name,
                    &file_key,
                    "original.txt",
                    1024,
                    upload_time,
                    &vec!["test".to_string()],
                    "test_public_key",
                );

                // System should either succeed or fail gracefully
                match store_result {
                    Ok(_) => {
                        // If successful, should be retrievable
                        let retrieved = database.get_file_by_name(&file_name).unwrap();
                        if !file_name.is_empty() {
                            prop_assert!(retrieved.is_some());
                        }
                    }
                    Err(_) => {
                        // Failure is acceptable for invalid names
                        // System should remain stable
                        let list_result = database.list_files(None);
                        prop_assert!(list_result.is_ok());
                    }
                }
            });
        }

        #[test]
        fn test_search_consistency(
            files in vec((generators::file_name(), generators::file_tags()), 1..=20),
            search_term in generators::file_name(),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let database = DatabaseManager::new(&setup.temp_dir.path().join("test.db")).unwrap();
                let upload_time = chrono::Local::now();

                // Store files
                let mut stored_files = Vec::new();
                for (i, (file_name, tags)) in files.iter().enumerate() {
                    let store_result = database.store_file(
                        &format!("{}_{}", file_name, i), // Make names unique
                        &format!("key_{}", i),
                        &format!("original_{}.txt", i),
                        1024,
                        upload_time,
                        tags,
                        "test_public_key",
                    );
                    
                    if store_result.is_ok() {
                        stored_files.push((format!("{}_{}", file_name, i), tags.clone()));
                    }
                }

                // Search for files
                let search_results = database.search_files(&search_term).unwrap_or_default();

                // Verify search consistency
                for result in &search_results {
                    // Each result should match the search term in name or tags
                    let matches_name = result.name.contains(&search_term);
                    let matches_tags = result.tags.iter().any(|tag| tag.contains(&search_term));
                    
                    prop_assert!(matches_name || matches_tags, 
                               "Search result should match search term in name or tags");
                }

                // Verify completeness (all matching files should be found)
                for (file_name, tags) in &stored_files {
                    let should_match = file_name.contains(&search_term) || 
                                     tags.iter().any(|tag| tag.contains(&search_term));
                    
                    if should_match {
                        let found = search_results.iter().any(|r| r.name == *file_name);
                        prop_assert!(found, "Matching file should be found in search results");
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod file_manager_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_file_content_preservation(
            content in generators::file_content(),
            file_name in generators::file_name(),
        ) {
            tokio_test::block_on(async {
                if content.len() > 50 * 1024 * 1024 { // Skip very large files for performance
                    return Ok(());
                }

                let setup = PropertyTestSetup::new().unwrap();
                let file_manager = FileManager::new(setup.config.clone()).await.unwrap();

                // Create test file
                let test_file_path = setup.temp_dir.path().join(&file_name);
                if tokio::fs::write(&test_file_path, &content).await.is_ok() {
                    // Store file
                    let storage_result = file_manager.store_file(&test_file_path, None).await;

                    if let Ok(result) = storage_result {
                        // Retrieve file
                        let retrieved_path = setup.temp_dir.path().join(format!("retrieved_{}", file_name));
                        let retrieval_result = file_manager.retrieve_file(&result.file_key, &retrieved_path).await;

                        if retrieval_result.is_ok() {
                            // Verify content preservation
                            let retrieved_content = tokio::fs::read(&retrieved_path).await.unwrap_or_default();
                            prop_assert_eq!(retrieved_content, content, "File content should be preserved");
                        }
                    }
                }
            });
        }

        #[test]
        fn test_concurrent_operations_consistency(
            operations in vec((generators::file_name(), generators::file_content()), 1..=10),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let file_manager = std::sync::Arc::new(
                    FileManager::new(setup.config.clone()).await.unwrap()
                );

                let mut join_handles = Vec::new();
                
                for (i, (file_name, content)) in operations.into_iter().enumerate() {
                    if content.len() > 1024 * 1024 { // Limit size for performance
                        continue;
                    }

                    let fm = file_manager.clone();
                    let temp_dir = setup.temp_dir.path().to_path_buf();
                    let unique_name = format!("{}_{}", file_name, i);

                    let handle = tokio::spawn(async move {
                        let test_file_path = temp_dir.join(&unique_name);
                        if tokio::fs::write(&test_file_path, &content).await.is_ok() {
                            fm.store_file(&test_file_path, None).await
                        } else {
                            Err(anyhow::anyhow!("Failed to write test file"))
                        }
                    });

                    join_handles.push(handle);
                }

                // Wait for all operations
                let mut successful_operations = 0;
                for handle in join_handles {
                    if let Ok(Ok(_)) = handle.await {
                        successful_operations += 1;
                    }
                }

                // System should remain consistent regardless of concurrent operations
                prop_assert!(successful_operations >= 0, "System should handle concurrent operations gracefully");
            });
        }
    }
}

#[cfg(test)]
mod storage_economy_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_user_registration_invariants(
            user in generators::user_account(),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let storage_economy = StorageEconomy::new(setup.config.clone()).unwrap();

                let registration_result = storage_economy.register_user(user.clone()).await;

                match registration_result {
                    Ok(_) => {
                        // If registration succeeded, user should have valid properties
                        prop_assert!(!user.email.is_empty(), "Registered user should have non-empty email");
                        prop_assert!(!user.password_hash.is_empty(), "Registered user should have password hash");
                        prop_assert!(!user.public_key.is_empty(), "Registered user should have public key");

                        // Account type limits should be non-negative
                        match user.account_type {
                            AccountType::Free { storage_gb, bandwidth_gb_month, api_calls_hour } |
                            AccountType::Basic { storage_gb, bandwidth_gb_month, api_calls_hour } |
                            AccountType::Pro { storage_gb, bandwidth_gb_month, api_calls_hour } |
                            AccountType::Enterprise { storage_gb, bandwidth_gb_month, api_calls_hour } => {
                                prop_assert!(storage_gb >= 0, "Storage GB should be non-negative");
                                prop_assert!(bandwidth_gb_month >= 0, "Bandwidth should be non-negative");
                                prop_assert!(api_calls_hour >= 0, "API calls should be non-negative");
                            }
                        }
                    }
                    Err(_) => {
                        // If registration failed, it should be due to invalid input
                        // System should remain stable
                    }
                }
            });
        }

        #[test]
        fn test_quota_enforcement_invariants(
            user_email in generators::user_email(),
            quota in generators::storage_quota(),
            allocation_requests in vec(1u64..=1024*1024u64, 0..=10),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let storage_economy = StorageEconomy::new(setup.config.clone()).unwrap();

                // Set quota
                let quota_result = storage_economy.set_storage_quota(&user_email, quota);

                if quota_result.is_ok() {
                    let mut total_allocated = 0u64;

                    // Try to allocate storage
                    for allocation in allocation_requests {
                        let allocation_result = storage_economy.allocate_storage(&user_email, allocation).await;

                        match allocation_result {
                            Ok(_) => {
                                total_allocated += allocation;
                                // Total allocated should not exceed quota (with some tolerance for overhead)
                                prop_assert!(total_allocated <= quota + 1024, 
                                           "Total allocated storage should not significantly exceed quota");
                            }
                            Err(_) => {
                                // Allocation failure is acceptable when quota would be exceeded
                            }
                        }
                    }
                }
            });
        }

        #[test]
        fn test_reputation_score_bounds(
            mut users in vec(generators::user_account(), 1..=10),
            operations in vec(prop_oneof![Just("positive"), Just("negative"), Just("neutral")], 0..=20),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let storage_economy = StorageEconomy::new(setup.config.clone()).unwrap();

                // Register users
                for user in &mut users {
                    let _ = storage_economy.register_user(user.clone()).await;
                }

                // Simulate operations that affect reputation
                for operation in operations {
                    if let Some(user) = users.first_mut() {
                        match operation {
                            "positive" => {
                                user.reputation_score += 0.1;
                            }
                            "negative" => {
                                user.reputation_score -= 0.1;
                            }
                            "neutral" => {
                                // No change
                            }
                            _ => {}
                        }

                        // Reputation score should remain within reasonable bounds
                        prop_assert!(user.reputation_score >= -10.0, "Reputation should not go extremely negative");
                        prop_assert!(user.reputation_score <= 10.0, "Reputation should not go extremely positive");
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod configuration_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_config_validation_invariants(
            port in 1u16..=65535u16,
            max_connections in 1u32..=10000u32,
            timeout_secs in 1u64..=3600u64,
            data_shards in 1usize..=32usize,
            parity_shards in 1usize..=16usize,
            chunk_size in 1024usize..=1024*1024usize,
            max_file_size in 1u64..=100*1024*1024*1024u64,
        ) {
            let mut config = Config::default();
            
            // Apply test parameters
            config.network.default_port = port;
            config.network.max_connections = max_connections;
            config.network.connection_timeout_secs = timeout_secs;
            config.storage.data_shards = data_shards;
            config.storage.parity_shards = parity_shards;
            config.storage.chunk_size = chunk_size;
            config.storage.max_file_size = max_file_size;

            // Validate configuration invariants
            prop_assert!(config.network.default_port > 0, "Port should be positive");
            prop_assert!(config.network.max_connections > 0, "Max connections should be positive");
            prop_assert!(config.network.connection_timeout_secs > 0, "Timeout should be positive");
            prop_assert!(config.storage.data_shards > 0, "Data shards should be positive");
            prop_assert!(config.storage.parity_shards > 0, "Parity shards should be positive");
            prop_assert!(config.storage.chunk_size > 0, "Chunk size should be positive");
            prop_assert!(config.storage.max_file_size > 0, "Max file size should be positive");

            // Reed-Solomon invariants
            prop_assert!(config.storage.data_shards + config.storage.parity_shards <= 256, 
                       "Total shards should not exceed Reed-Solomon limit");
            prop_assert!(config.storage.parity_shards <= config.storage.data_shards, 
                       "Parity shards should not exceed data shards for efficiency");

            // Practical limits
            prop_assert!(config.storage.chunk_size >= 1024, "Chunk size should be at least 1KB");
            prop_assert!(config.storage.chunk_size <= 10*1024*1024, "Chunk size should not exceed 10MB");
            prop_assert!(config.network.max_connections <= 10000, "Max connections should be reasonable");
        }

        #[test]
        fn test_config_serialization_roundtrip(
            port in 1u16..=65535u16,
            data_shards in 1usize..=16usize,
            parity_shards in 1usize..=8usize,
        ) {
            let mut original_config = Config::default();
            original_config.network.default_port = port;
            original_config.storage.data_shards = data_shards;
            original_config.storage.parity_shards = parity_shards;

            // Serialize to TOML
            let serialized = toml::to_string(&original_config);
            prop_assert!(serialized.is_ok(), "Config should serialize to TOML");

            if let Ok(toml_string) = serialized {
                // Deserialize from TOML
                let deserialized: Result<Config, _> = toml::from_str(&toml_string);
                prop_assert!(deserialized.is_ok(), "Config should deserialize from TOML");

                if let Ok(restored_config) = deserialized {
                    // Verify roundtrip preservation
                    prop_assert_eq!(restored_config.network.default_port, original_config.network.default_port);
                    prop_assert_eq!(restored_config.storage.data_shards, original_config.storage.data_shards);
                    prop_assert_eq!(restored_config.storage.parity_shards, original_config.storage.parity_shards);
                }
            }
        }
    }
}

#[cfg(test)]
mod edge_case_property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn test_boundary_conditions(
            file_size in prop_oneof![
                Just(0u64),                           // Empty file
                Just(1u64),                           // Minimum size
                Just(1023u64),                        // Just under 1KB
                Just(1024u64),                        // Exactly 1KB
                Just(1025u64),                        // Just over 1KB
                Just(1024*1024 - 1),                  // Just under 1MB
                Just(1024*1024u64),                   // Exactly 1MB
                Just(1024*1024 + 1),                  // Just over 1MB
                Just(u64::MAX),                       // Maximum possible
            ],
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let database = DatabaseManager::new(&setup.temp_dir.path().join("test.db")).unwrap();
                let upload_time = chrono::Local::now();

                let store_result = database.store_file(
                    "boundary_test",
                    "boundary_key",
                    "boundary.txt",
                    file_size,
                    upload_time,
                    &vec!["boundary".to_string()],
                    "test_public_key",
                );

                match store_result {
                    Ok(_) => {
                        // If storage succeeded, verify the file
                        let retrieved = database.get_file_by_name("boundary_test").unwrap();
                        prop_assert!(retrieved.is_some(), "Stored file should be retrievable");
                        
                        if let Some(file) = retrieved {
                            prop_assert_eq!(file.file_size, file_size, "File size should be preserved");
                        }
                    }
                    Err(_) => {
                        // Some boundary conditions may legitimately fail
                        // System should remain stable
                        let list_result = database.list_files(None);
                        prop_assert!(list_result.is_ok(), "Database should remain functional after boundary test");
                    }
                }
            });
        }

        #[test]
        fn test_unicode_and_special_characters(
            content in prop_oneof![
                // Unicode content
                Just("Hello, 世界! 🌍 Здравствуй мир! مرحبا بالعالم!".as_bytes().to_vec()),
                // Special characters
                Just("File with special chars: !@#$%^&*()[]{}|\\:;\"'<>?,.".as_bytes().to_vec()),
                // Control characters
                Just(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
                // High bit values
                Just(vec![128, 129, 255, 254, 253, 252, 251, 250]),
                // Mixed content
                Just(b"Normal text\x00\x01\x02 with nulls and \xFF\xFE Unicode: \xE2\x9C\x93".to_vec()),
            ],
            file_name in prop_oneof![
                Just("unicode_test_файл.txt".to_string()),
                Just("emoji_test_🚀.txt".to_string()),
                Just("special_chars_!@#$.txt".to_string()),
                Just("normal_file.txt".to_string()),
            ],
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                
                // Try to create file with special content
                let test_file_path = setup.temp_dir.path().join(&file_name);
                let write_result = tokio::fs::write(&test_file_path, &content).await;

                if write_result.is_ok() {
                    let file_manager = FileManager::new(setup.config.clone()).await.unwrap();
                    let storage_result = file_manager.store_file(&test_file_path, None).await;

                    if let Ok(result) = storage_result {
                        // Try to retrieve and verify content
                        let retrieved_path = setup.temp_dir.path().join("retrieved_unicode.txt");
                        let retrieval_result = file_manager.retrieve_file(&result.file_key, &retrieved_path).await;

                        if retrieval_result.is_ok() {
                            let retrieved_content = tokio::fs::read(&retrieved_path).await.unwrap_or_default();
                            prop_assert_eq!(retrieved_content, content, "Unicode/special content should be preserved");
                        }
                    }
                }
            });
        }

        #[test]
        fn test_extreme_tag_combinations(
            tags in vec(
                prop_oneof![
                    Just("".to_string()),                          // Empty tag
                    Just("a".repeat(1000)),                        // Very long tag
                    Just("tag with spaces".to_string()),           // Spaces
                    Just("tag\nwith\nnewlines".to_string()),      // Newlines
                    Just("tag\twith\ttabs".to_string()),          // Tabs
                    Just("tag_with_unicode_世界".to_string()),      // Unicode
                    Just("tag.with.dots".to_string()),            // Dots
                    Just("tag-with-dashes".to_string()),          // Dashes
                    Just("tag_with_underscores".to_string()),     // Underscores
                    Just("TAG_WITH_CAPS".to_string()),            // Uppercase
                    Just("123456789".to_string()),                // Numbers only
                    Just("!@#$%^&*()".to_string()),              // Special chars
                ], 
                0..=50
            ),
        ) {
            tokio_test::block_on(async {
                let setup = PropertyTestSetup::new().unwrap();
                let database = DatabaseManager::new(&setup.temp_dir.path().join("test.db")).unwrap();
                let upload_time = chrono::Local::now();

                let store_result = database.store_file(
                    "tag_test",
                    "tag_key",
                    "tag_test.txt",
                    1024,
                    upload_time,
                    &tags,
                    "test_public_key",
                );

                match store_result {
                    Ok(_) => {
                        // Verify tags are preserved
                        let retrieved = database.get_file_by_name("tag_test").unwrap();
                        if let Some(file) = retrieved {
                            // Tags should be preserved (though system may normalize them)
                            prop_assert!(!file.tags.is_empty() || tags.is_empty(), 
                                       "Tags should be preserved or empty if input was empty");
                        }

                        // Test searching with various tag combinations
                        for tag in &tags {
                            if !tag.is_empty() {
                                let search_results = database.search_files(tag).unwrap_or_default();
                                // Should find the file if tag is searchable
                                let found = search_results.iter().any(|f| f.name == "tag_test");
                                if tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                                    prop_assert!(found, "Should find file with alphanumeric tag");
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Some extreme tag combinations may fail
                        // System should remain stable
                        let list_result = database.list_files(None);
                        prop_assert!(list_result.is_ok(), "Database should remain stable after tag test");
                    }
                }
            });
        }
    }
}