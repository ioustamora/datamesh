/// Test Utilities for DataMesh
///
/// This module provides common utilities and setup functions for testing,
/// improving test reliability and reducing code duplication.

use anyhow::Result;
use chrono::Local;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

use datamesh::config::Config;
use datamesh::database::{DatabaseManager, FileEntry};
use datamesh::governance::{AccountType, UserAccount, VerificationStatus};
use datamesh::key_manager::KeyManager;

/// Test environment setup helper
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
    pub storage_path: PathBuf,
    pub db: DatabaseManager,
    pub key_manager: KeyManager,
}

impl TestEnvironment {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let storage_path = temp_dir.path().join("storage");
        
        let db = DatabaseManager::new(&db_path)?;
        // Create key manager with deterministic key for testing
        let test_key = ecies::SecretKey::from_slice(&[1u8; 32]).unwrap();
        let key_manager = KeyManager::new(test_key, "test_key".to_string());
        
        Ok(TestEnvironment {
            temp_dir,
            db_path,
            storage_path,
            db,
            key_manager,
        })
    }
    
    /// Add test files to the database
    pub fn add_test_files(&self, count: usize) -> Result<Vec<FileEntry>> {
        let mut files = Vec::new();
        
        for i in 0..count {
            let file_name = format!("test_file_{}", i);
            let file_key = format!("test_key_{}", i);
            let file_size = 1024 * (i + 1); // Different sizes
            let upload_time = Local::now();
            let tags = vec!["test".to_string(), format!("tag_{}", i)];
            
            self.db.store_file(
                &file_name,
                &file_key,
                &file_name,
                file_size as u64,
                upload_time,
                &tags,
                "test_public_key",
            )?;
            
            // Get the stored file to return
            if let Some(file) = self.db.get_file_by_name(&file_name)? {
                files.push(file);
            }
        }
        
        Ok(files)
    }
    
    /// Create test configuration
    pub fn create_test_config(&self) -> Config {
        let mut config = Config::default();
        config.storage.keys_dir = self.temp_dir.path().join("keys").to_string_lossy().to_string();
        config
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;
    
    /// Assert that a value is within a range
    pub fn assert_in_range(value: f64, min: f64, max: f64) {
        assert!(value >= min && value <= max, "Value {} not in range [{}, {}]", value, min, max);
    }
    
    /// Assert that a collection is not empty
    pub fn assert_not_empty<T>(collection: &[T], message: &str) {
        assert!(!collection.is_empty(), "{}", message);
    }
    
    /// Assert that a path exists
    pub fn assert_path_exists(path: &std::path::Path) {
        assert!(path.exists(), "Path {:?} does not exist", path);
    }
    
    /// Assert that a duration is within reasonable bounds
    pub fn assert_duration_reasonable(duration: Duration, max_expected: Duration) {
        assert!(duration <= max_expected, 
               "Duration {:?} exceeds maximum expected {:?}", duration, max_expected);
    }
}

/// Mock data generation helpers
pub mod mock_data {
    use super::*;
    use datamesh::billing_system::{BillingCycle, SubscriptionTier};
    
    pub fn create_test_user(id: &str) -> UserAccount {
        UserAccount {
            user_id: uuid::Uuid::new_v4(),
            email: format!("{}@test.com", id),
            password_hash: "test_hash".to_string(),
            public_key: "test_public_key".to_string(),
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 100,
                api_calls_hour: 1000,
            },
            verification_status: VerificationStatus::EmailVerified,
            registration_date: Local::now(),
            last_activity: Local::now(),
            reputation_score: 0.0,
            abuse_flags: 0,
            subscription: None,
        }
    }
    
    pub fn create_test_subscription_tiers() -> Vec<SubscriptionTier> {
        vec![
            SubscriptionTier {
                name: "Free".to_string(),
                price_monthly: 0.0,
                storage_gb: 5,
                bandwidth_gb: 100,
                api_calls_hour: 1000,
                support_level: "Basic".to_string(),
            },
            SubscriptionTier {
                name: "Premium".to_string(),
                price_monthly: 9.99,
                storage_gb: 100,
                bandwidth_gb: 1000,
                api_calls_hour: 10000,
                support_level: "Priority".to_string(),
            },
        ]
    }
    
    pub fn create_test_billing_cycles() -> Vec<BillingCycle> {
        vec![
            BillingCycle {
                cycle_id: uuid::Uuid::new_v4(),
                start_date: Local::now(),
                end_date: Local::now() + chrono::Duration::days(30),
                amount_due: 9.99,
                amount_paid: 0.0,
                status: "pending".to_string(),
            },
        ]
    }
}

/// Performance testing utilities
pub mod performance {
    use super::*;
    use std::time::Instant;
    
    pub struct PerformanceTest {
        name: String,
        start_time: Instant,
    }
    
    impl PerformanceTest {
        pub fn new(name: &str) -> Self {
            println!("Starting performance test: {}", name);
            Self {
                name: name.to_string(),
                start_time: Instant::now(),
            }
        }
        
        pub fn finish(self, expected_max_duration: Duration) {
            let elapsed = self.start_time.elapsed();
            println!("Performance test '{}' completed in {:?}", self.name, elapsed);
            
            if elapsed > expected_max_duration {
                println!("⚠️  Performance test '{}' took longer than expected: {:?} > {:?}", 
                        self.name, elapsed, expected_max_duration);
            } else {
                println!("✅ Performance test '{}' completed within expected time", self.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_setup() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Test that environment is set up correctly
        assert!(env.temp_dir.path().exists());
        assert!(env.db_path.exists());
        
        // Test database operations
        let upload_time = Local::now();
        let tags = vec!["test".to_string()];
        
        env.db.store_file(
            "test_file",
            "test_key",
            "test_file.txt",
            1024,
            upload_time,
            &tags,
            "test_public_key",
        )?;
        
        let file = env.db.get_file_by_name("test_file")?;
        assert!(file.is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_mock_data_generation() {
        let user = mock_data::create_test_user("test_user");
        // Test that user is created with correct email
        assert_eq!(user.email, "test_user@test.com");
        assert!(user.reputation_score >= 0.0);
    }
    
    #[test]
    fn test_performance_utilities() {
        let perf = performance::PerformanceTest::new("test_operation");
        
        // Simulate some work
        std::thread::sleep(Duration::from_millis(10));
        
        // This should complete quickly
        perf.finish(Duration::from_millis(100));
    }
}
