/// Integration Tests for DFS Core Modules
///
/// This module provides integration tests for the newly implemented core modules,
/// ensuring they work correctly together in realistic scenarios.

use tempfile::TemporaryDirectory;
use tokio_test;
use std::path::PathBuf;
use chrono::Local;

// Import the DFS modules we want to test
use datamesh::database::{DatabaseManager, get_default_db_path};
use datamesh::file_manager::{SearchCriteria, SizeRange};
use datamesh::batch_operations::{BatchPutConfig, BatchTagConfig};
use datamesh::health_manager::{RepairConfig, CleanupConfig};

#[tokio::test]
async fn test_database_operations() {
    let temp_dir = TemporaryDirectory::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test storing a file
    let upload_time = Local::now();
    let tags = vec!["test".to_string(), "integration".to_string()];
    
    let file_id = db.store_file(
        "test-file",
        "test-key-123", 
        "test.txt",
        1024,
        upload_time,
        &tags,
        "test-public-key"
    ).unwrap();
    
    assert!(file_id > 0);
    
    // Test retrieving by name
    let file = db.get_file_by_name("test-file").unwrap().unwrap();
    assert_eq!(file.name, "test-file");
    assert_eq!(file.file_key, "test-key-123");
    assert_eq!(file.tags.len(), 2);
    assert!(file.tags.contains(&"test".to_string()));
    
    // Test retrieving by key
    let file_by_key = db.get_file_by_key("test-key-123").unwrap().unwrap();
    assert_eq!(file_by_key.name, "test-file");
    
    // Test listing files
    let files = db.list_files(None).unwrap();
    assert_eq!(files.len(), 1);
    
    // Test tag filtering
    let tagged_files = db.list_files(Some("test")).unwrap();
    assert_eq!(tagged_files.len(), 1);
    
    // Test stats
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_files, 1);
    assert_eq!(stats.total_size, 1024);
}

#[tokio::test]
async fn test_file_search() {
    let temp_dir = TemporaryDirectory::new().unwrap();
    let db_path = temp_dir.path().join("search_test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Add multiple test files
    let upload_time = Local::now();
    
    db.store_file(
        "document1", "key1", "document1.pdf", 1024 * 1024, // 1MB
        upload_time, &vec!["work".to_string(), "pdf".to_string()], "pubkey1"
    ).unwrap();
    
    db.store_file(
        "image1", "key2", "image1.jpg", 500 * 1024, // 500KB
        upload_time, &vec!["personal".to_string(), "photo".to_string()], "pubkey1"
    ).unwrap();
    
    db.store_file(
        "large-file", "key3", "video.mp4", 100 * 1024 * 1024, // 100MB
        upload_time, &vec!["video".to_string(), "personal".to_string()], "pubkey1"
    ).unwrap();
    
    // Test basic search
    let criteria = SearchCriteria {
        query: "document".to_string(),
        file_type: None,
        size_range: None,
        date_range: None,
        use_regex: false,
        limit: 10,
    };
    
    let results = datamesh::file_manager::search_files(criteria).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "document1");
    
    // Test size range search
    let size_criteria = SearchCriteria {
        query: "".to_string(),
        file_type: None,
        size_range: Some(SizeRange::LessThan(1024 * 1024)), // Less than 1MB
        date_range: None,
        use_regex: false,
        limit: 10,
    };
    
    let size_results = datamesh::file_manager::search_files(size_criteria).await.unwrap();
    assert_eq!(size_results.len(), 1);
    assert_eq!(size_results[0].name, "image1");
}

#[tokio::test] 
async fn test_batch_operations() {
    // Test batch tagging
    let config = BatchTagConfig {
        pattern: "test*".to_string(),
        add_tags: vec!["batch".to_string(), "automated".to_string()],
        remove_tags: vec!["old".to_string()],
        dry_run: true,
    };
    
    let result = datamesh::batch_operations::batch_tag(config).await.unwrap();
    assert_eq!(result.failed, 0); // Should succeed even with no matching files in dry run
}

#[tokio::test]
async fn test_health_manager() {
    // Test quota management
    let quota_result = datamesh::health_manager::manage_quota(
        true, // show usage
        Some("1GB".to_string()), // set limit
        Some(80), // warning threshold
    ).await;
    
    assert!(quota_result.is_ok());
    
    // Test cleanup in dry run mode
    let cleanup_config = CleanupConfig {
        orphaned: true,
        duplicates: true,
        low_health: false,
        dry_run: true,
        force: false,
    };
    
    let cleanup_result = datamesh::health_manager::cleanup_storage(cleanup_config).await;
    assert!(cleanup_result.is_ok());
    
    // Test health report generation
    let health_report = datamesh::health_manager::generate_health_report().await;
    assert!(health_report.is_ok());
}

#[test]
fn test_database_name_generation() {
    let temp_dir = TemporaryDirectory::new().unwrap();
    let db_path = temp_dir.path().join("name_test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test unique name generation
    let name1 = db.generate_unique_name("test.txt").unwrap();
    assert_eq!(name1, "test");
    
    // Store a file with this name
    let upload_time = Local::now();
    db.store_file(
        &name1, "key1", "test.txt", 1024, upload_time, 
        &vec![], "pubkey1"
    ).unwrap();
    
    // Generate another name - should be different
    let name2 = db.generate_unique_name("test.txt").unwrap();
    assert_ne!(name1, name2);
    assert!(name2.starts_with("test"));
}

#[test]
fn test_database_error_handling() {
    let temp_dir = TemporaryDirectory::new().unwrap();
    let db_path = temp_dir.path().join("error_test.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Test duplicate name error
    let upload_time = Local::now();
    db.store_file(
        "duplicate-test", "key1", "test.txt", 1024, upload_time,
        &vec![], "pubkey1"
    ).unwrap();
    
    // Try to store another file with the same name - should fail
    let duplicate_result = db.store_file(
        "duplicate-test", "key2", "test2.txt", 2048, upload_time,
        &vec![], "pubkey1"
    );
    
    assert!(duplicate_result.is_err());
    
    // Test non-existent file retrieval
    let missing_file = db.get_file_by_name("non-existent").unwrap();
    assert!(missing_file.is_none());
    
    let missing_by_key = db.get_file_by_key("non-existent-key").unwrap();
    assert!(missing_by_key.is_none());
}

#[test]
fn test_presets_functionality() {
    use datamesh::presets::{NetworkPresets, parse_network_spec};
    
    let presets = NetworkPresets::new();
    
    // Test built-in presets exist
    assert!(presets.get_preset("local").is_some());
    assert!(presets.get_preset("public").is_some()); 
    assert!(presets.get_preset("test").is_some());
    assert!(presets.get_preset("nonexistent").is_none());
    
    // Test preset application
    let local_config = presets.apply_preset("local").unwrap();
    assert!(local_config.discovery_enabled);
    assert!(!local_config.bootstrap_peers.is_empty());
    
    // Test custom network spec parsing
    let multiaddr_config = parse_network_spec("/ip4/127.0.0.1/tcp/40871").unwrap();
    assert_eq!(multiaddr_config.bootstrap_peers.len(), 1);
    assert!(multiaddr_config.bootstrap_peers[0].peer_id.is_none());
}

#[test]
fn test_error_handling_integration() {
    use datamesh::error_handling::{
        handle_error, file_not_found_error_with_suggestions,
        operation_error_with_context, ErrorBatch
    };
    use std::io::{Error as IoError, ErrorKind};
    
    // Test IO error handling
    let io_error = IoError::new(ErrorKind::NotFound, "File not found");
    let enhanced = handle_error(&io_error);
    assert!(!enhanced.suggestions.is_empty());
    
    // Test file not found error
    let file_error = file_not_found_error_with_suggestions("test.txt");
    assert!(!file_error.suggestions.is_empty());
    assert!(file_error.suggestions.iter().any(|s| s.contains("list")));
    
    // Test operation context
    let op_error = operation_error_with_context("put", &io_error);
    assert!(op_error.context.is_some());
    assert!(op_error.context.as_ref().unwrap().contains("upload"));
    
    // Test error batch
    let mut batch = ErrorBatch::new("Test batch".to_string());
    batch.add_error(file_error);
    batch.add_error(enhanced);
    
    assert_eq!(batch.count(), 2);
    assert!(!batch.is_empty());
}

#[tokio::test]
async fn test_module_integration() {
    // Test that modules work together properly
    let temp_dir = TemporaryDirectory::new().unwrap();
    let db_path = temp_dir.path().join("integration.db");
    
    let db = DatabaseManager::new(&db_path).unwrap();
    
    // Add some test data
    let upload_time = Local::now();
    db.store_file(
        "integration-test", "int-key-123", "integration.txt", 
        2048, upload_time, &vec!["integration".to_string()], "test-key"
    ).unwrap();
    
    // Test search integration
    let search_criteria = SearchCriteria {
        query: "integration".to_string(),
        file_type: None,
        size_range: Some(SizeRange::GreaterThan(1024)), // > 1KB
        date_range: None,
        use_regex: false,
        limit: 5,
    };
    
    let search_results = datamesh::file_manager::search_files(search_criteria).await.unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "integration-test");
    
    // Test stats
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_files, 1);
    assert_eq!(stats.total_size, 2048);
}