/// Test Utilities for DataMesh
///
/// This module provides common utilities and setup functions for testing,
/// improving test reliability and reducing code duplication.

use anyhow::Result;
use chrono::Local;
use std::p        let user = create_test_user("test_user");
        // Test that user is created with correct email
        assert_eq!(user.email, "test_user@test.com");
        assert!(user.reputation_score >= 0.0);:PathBuf;
use tempfile::TempDir;

use datamesh::config::Config;
use datamesh::database::{DatabaseManager, FileEntry};
use datamesh::key_manager::KeyManager;

/// Test setup helper that creates temporary directory and database
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
    pub db: DatabaseManager,
    pub key_manager: KeyManager,
}

impl TestEnvironment {
    /// Create a new test environment with temporary database and key manager
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseManager::new(&db_path)?;
        
        // Create key manager with deterministic key for testing
        let test_key = libsecp256k1::SecretKey::parse_slice(&[1u8; 32]).unwrap();
        let key_manager = KeyManager::new(test_key, "test_key".to_string());
        
        Ok(TestEnvironment {
            temp_dir,
            db_path,
            db,
            key_manager,
        })
    }

    /// Add test files to the database
    pub fn add_test_files(&self, count: usize) -> Result<Vec<FileEntry>> {
        let mut files = Vec::new();
        let upload_time = Local::now();
        
        for i in 0..count {
            let file_name = format!("test_file_{}", i);
            let file_key = format!("test_key_{}", i);
            let original_name = format!("original_{}.txt", i);
            let file_size = 1024 * (i + 1) as u64;
            let tags = vec![format!("tag_{}", i), "test".to_string()];
            
            let file_id = self.db.store_file(
                &file_name,
                &file_key,
                &original_name,
                file_size,
                upload_time,
                &tags,
                "test_public_key",
            )?;
            
            let file_entry = FileEntry {
                id: file_id,
                name: file_name,
                file_key,
                original_filename: original_name,
                file_size,
                upload_time,
                tags,
                public_key_hex: "test_public_key".to_string(),
                chunks_total: 6,
                chunks_healthy: 6,
            };
            
            files.push(file_entry);
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
    
    /// Assert that a result is successful and unwrap it
    pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("Expected Ok, got Err: {:?}", err),
        }
    }
    
    /// Assert that a result is an error
    pub fn assert_err<T: std::fmt::Debug, E>(result: Result<T, E>) {
        match result {
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(_) => {},
        }
    }
    
    /// Assert that a value is within a range
    pub fn assert_in_range(value: f64, min: f64, max: f64) {
        assert!(
            value >= min && value <= max,
            "Value {} is not in range [{}, {}]",
            value, min, max
        );
    }
}

/// Mock data generators for testing
pub mod mock_data {
    use super::*;
    use datamesh::governance::{AccountType, VerificationStatus, UserAccount};
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
            SubscriptionTier::Free,
            SubscriptionTier::Basic,
            SubscriptionTier::Pro,
            SubscriptionTier::Enterprise,
        ]
    }
    
    pub fn create_test_billing_cycles() -> Vec<BillingCycle> {
        vec![
            BillingCycle::Monthly,
            BillingCycle::Quarterly,
            BillingCycle::Yearly,
            BillingCycle::PayAsYouGo,
        ]
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Instant, Duration};
    
    pub struct PerformanceTest {
        start: Instant,
        operation_name: String,
    }
    
    impl PerformanceTest {
        pub fn new(operation_name: &str) -> Self {
            Self {
                start: Instant::now(),
                operation_name: operation_name.to_string(),
            }
        }
        
        pub fn finish(self, max_duration: Duration) {
            let elapsed = self.start.elapsed();
            assert!(
                elapsed <= max_duration,
                "Operation '{}' took {:?}, expected <= {:?}",
                self.operation_name, elapsed, max_duration
            );
        }
        
        pub fn finish_and_print(self) -> Duration {
            let elapsed = self.start.elapsed();
            println!("Operation '{}' took {:?}", self.operation_name, elapsed);
            elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_creation() {
        let env = TestEnvironment::new().unwrap();
        assert!(env.temp_dir.path().exists());
        assert!(env.db_path.exists());
    }
    
    #[test]
    fn test_file_creation() {
        let env = TestEnvironment::new().unwrap();
        let files = env.add_test_files(3).unwrap();
        assert_eq!(files.len(), 3);
        
        // Verify files are in database
        let all_files = env.db.list_files(None).unwrap();
        assert_eq!(all_files.len(), 3);
    }
    
    #[test]
    fn test_mock_data() {
        let user = mock_data::create_test_user("test_user");
        assert_eq!(user.user_id, "test_user");
        assert_eq!(user.email, "test_user@test.com");
        assert!(user.is_active);
        
        let tiers = mock_data::create_test_subscription_tiers();
        assert!(!tiers.is_empty());
        
        let cycles = mock_data::create_test_billing_cycles();
        assert!(!cycles.is_empty());
    }
    
    #[test]
    fn test_assertions() {
        // Test successful assertion
        let result: Result<i32, &str> = Ok(42);
        let value = assertions::assert_ok(result);
        assert_eq!(value, 42);
        
        // Test range assertion
        assertions::assert_in_range(5.0, 0.0, 10.0);
    }
    
    #[test]
    fn test_performance_utils() {
        use std::thread;
        use std::time::Duration;
        
        let perf = performance::PerformanceTest::new("test_operation");
        thread::sleep(Duration::from_millis(10));
        
        // This should pass
        perf.finish(Duration::from_millis(100));
    }
}
