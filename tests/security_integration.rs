/// Security Integration Tests for DataMesh
///
/// This module tests cryptographic operations, security protocols, and 
/// integration between security components across the DataMesh system.

mod test_utils;

use anyhow::Result;
use ecies::{PublicKey, SecretKey};
use rand::rngs::OsRng;
use std::time::Duration;
use test_utils::{TestEnvironment, assertions, performance};

use datamesh::{
    key_manager::KeyManager,
    key_rotation::{KeyRotationManager, KeyRotationConfig},
    encrypted_key_manager::EncryptedKeyManager,
    database::DatabaseManager,
    governance::{UserAccount, AccountType, VerificationStatus},
};

/// Security test environment with multiple key managers and crypto components
struct SecurityTestEnv {
    test_env: TestEnvironment,
    key_manager_1: KeyManager,
    key_manager_2: KeyManager,
}

impl SecurityTestEnv {
    fn new() -> Result<Self> {
        let test_env = TestEnvironment::new()?;
        
        // Create multiple key managers for testing interactions
        let key1 = SecretKey::random(&mut OsRng);
        let key_manager_1 = KeyManager::new(key1, "test_key_1".to_string());
        
        let key2 = SecretKey::random(&mut OsRng);
        let key_manager_2 = KeyManager::new(key2, "test_key_2".to_string());
        
        Ok(SecurityTestEnv {
            test_env,
            key_manager_1,
            key_manager_2,
        })
    }
}

#[tokio::test]
async fn test_end_to_end_encryption_workflow() -> Result<()> {
    let env = SecurityTestEnv::new()?;
    
    let perf_test = performance::PerformanceTest::new("e2e_encryption");
    
    // Test data of various sizes
    let test_data_sets = vec![
        ("small", vec![0u8; 32]),
        ("medium", vec![1u8; 1024]),
        ("large", vec![2u8; 1024 * 100]),
        ("binary", (0..=255u8).cycle().take(4096).collect()),
    ];
    
    for (name, data) in test_data_sets {
        // Encrypt with key manager 1
        let encrypted = env.key_manager_1.encrypt(&data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        assert!(encrypted.len() > data.len(), "Encrypted data should be larger");
        
        // Decrypt with same key manager
        let decrypted = env.key_manager_1.decrypt(&encrypted)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        assert_eq!(data, decrypted, "Decrypted data should match original for {}", name);
        
        // Verify other key manager cannot decrypt
        let wrong_decrypt_result = env.key_manager_2.decrypt(&encrypted);
        assert!(wrong_decrypt_result.is_err(), "Wrong key should not decrypt data");
    }
    
    perf_test.finish(Duration::from_secs(5));
    Ok(())
}

#[tokio::test]
async fn test_key_rotation_basic() -> Result<()> {
    let _env = SecurityTestEnv::new()?;
    
    // Test key rotation manager creation
    let config = KeyRotationConfig::default();
    let manager = KeyRotationManager::new(config)?;
    
    // Test basic key rotation functionality
    let stats = manager.get_stats().await?;
    assert!(stats.current_version >= 0);
    
    Ok(())
}

#[tokio::test]
async fn test_encrypted_key_storage() -> Result<()> {
    let _env = SecurityTestEnv::new()?;
    
    // Generate test key
    let test_key = SecretKey::random(&mut OsRng);
    
    // Create encrypted key manager
    let encrypted_manager = EncryptedKeyManager::new(
        &test_key, 
        "test_encrypted_key".to_string(), 
        "test_password"
    )?;
    
    // Test that encrypted manager was created successfully
    assert_eq!(encrypted_manager.name, "test_encrypted_key");
    
    Ok(())
}

#[tokio::test]
async fn test_multi_user_key_isolation() -> Result<()> {
    let env = SecurityTestEnv::new()?;
    
    // User 1 encrypts data
    let user1_data = b"user 1 private data";
    let user1_encrypted = env.key_manager_1.encrypt(user1_data)
        .map_err(|e| anyhow::anyhow!("User 1 encryption failed: {}", e))?;
    
    // User 2 encrypts data
    let user2_data = b"user 2 private data";
    let user2_encrypted = env.key_manager_2.encrypt(user2_data)
        .map_err(|e| anyhow::anyhow!("User 2 encryption failed: {}", e))?;
    
    // Verify isolation: User 1 cannot decrypt User 2's data
    let cross_decrypt_result = env.key_manager_1.decrypt(&user2_encrypted);
    assert!(cross_decrypt_result.is_err(), "User 1 should not decrypt User 2's data");
    
    // Verify isolation: User 2 cannot decrypt User 1's data
    let cross_decrypt_result = env.key_manager_2.decrypt(&user1_encrypted);
    assert!(cross_decrypt_result.is_err(), "User 2 should not decrypt User 1's data");
    
    // Verify each user can decrypt their own data
    let user1_decrypted = env.key_manager_1.decrypt(&user1_encrypted)
        .map_err(|e| anyhow::anyhow!("User 1 decryption failed: {}", e))?;
    assert_eq!(user1_data, &user1_decrypted[..]);
    
    let user2_decrypted = env.key_manager_2.decrypt(&user2_encrypted)
        .map_err(|e| anyhow::anyhow!("User 2 decryption failed: {}", e))?;
    assert_eq!(user2_data, &user2_decrypted[..]);
    
    Ok(())
}

#[tokio::test]
async fn test_cryptographic_performance_benchmarks() -> Result<()> {
    let env = SecurityTestEnv::new()?;
    
    // Benchmark encryption performance
    let data_sizes = vec![1024, 10 * 1024, 100 * 1024]; // 1KB to 100KB
    
    for size in data_sizes {
        let test_data = vec![0u8; size];
        
        // Benchmark encryption
        let encrypt_perf = performance::PerformanceTest::new(&format!("encrypt_{}kb", size / 1024));
        let encrypted = env.key_manager_1.encrypt(&test_data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        encrypt_perf.finish(Duration::from_secs(5));
        
        // Benchmark decryption
        let decrypt_perf = performance::PerformanceTest::new(&format!("decrypt_{}kb", size / 1024));
        let decrypted = env.key_manager_1.decrypt(&encrypted)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        decrypt_perf.finish(Duration::from_secs(5));
        
        assert_eq!(test_data, decrypted);
    }
    
    // Benchmark key generation
    let keygen_perf = performance::PerformanceTest::new("key_generation");
    for _ in 0..10 {
        let _key = SecretKey::random(&mut OsRng);
    }
    keygen_perf.finish(Duration::from_secs(10));
    
    Ok(())
}

#[tokio::test]
async fn test_key_manager_security() -> Result<()> {
    // Create two different key managers
    let key1 = SecretKey::random(&mut OsRng);
    let key_manager_1 = KeyManager::new(key1, "key1".to_string());
    
    let key2 = SecretKey::random(&mut OsRng);
    let key_manager_2 = KeyManager::new(key2, "key2".to_string());
    
    // Verify different keys produce different public keys
    assert_ne!(
        key_manager_1.key_info.public_key_hex,
        key_manager_2.key_info.public_key_hex
    );
    
    // Test that same data encrypted with different keys produces different results
    let test_data = b"test data for encryption";
    let encrypted_1 = key_manager_1.encrypt(test_data)
        .map_err(|e| anyhow::anyhow!("Encryption 1 failed: {}", e))?;
    let encrypted_2 = key_manager_2.encrypt(test_data)
        .map_err(|e| anyhow::anyhow!("Encryption 2 failed: {}", e))?;
    assert_ne!(encrypted_1, encrypted_2);
    
    // Test cross-decryption fails (key isolation)
    let cross_decrypt_result = key_manager_2.decrypt(&encrypted_1);
    assert!(cross_decrypt_result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_security_under_concurrent_load() -> Result<()> {
    let env = SecurityTestEnv::new()?;
    
    let perf_test = performance::PerformanceTest::new("concurrent_security_operations");
    
    // Spawn multiple concurrent security operations
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let key_manager = env.key_manager_1.clone();
        let handle = tokio::spawn(async move {
            let data = format!("concurrent test data {}", i);
            let encrypted = key_manager.encrypt(data.as_bytes())
                .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
            let decrypted = key_manager.decrypt(&encrypted)
                .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
            Ok::<_, anyhow::Error>(decrypted == data.as_bytes())
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(true)) = handle.await {
            success_count += 1;
        }
    }
    
    perf_test.finish(Duration::from_secs(10));
    
    assert_eq!(success_count, 10, "All concurrent operations should succeed");
    
    Ok(())
}