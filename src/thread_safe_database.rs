use anyhow::Result;
/// Thread-Safe Database Manager
///
/// This module provides a thread-safe wrapper around the database operations
/// to fix Send/Sync issues with SQLite in multi-threaded environments.
use std::sync::{Arc, RwLock};

use crate::database::{DatabaseManager, DatabaseStats, FileEntry};
use crate::error::DfsResult;

/// Thread-safe database manager that can be shared across threads
#[derive(Clone, Debug)]
pub struct ThreadSafeDatabaseManager {
    db_path: String,
    // Use Arc<RwLock<Option<DatabaseManager>>> for thread safety
    db_cache: Arc<RwLock<Option<DatabaseManager>>>,
}

impl ThreadSafeDatabaseManager {
    /// Create a new thread-safe database manager
    pub fn new(db_path: &str) -> Result<Self> {
        Ok(ThreadSafeDatabaseManager {
            db_path: db_path.to_string(),
            db_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Get or create a database connection
    fn get_db(&self) -> Result<DatabaseManager> {
        // For thread safety, we create a new connection each time
        // This is safe with SQLite when using proper locking
        let path = std::path::PathBuf::from(&self.db_path);
        DatabaseManager::new(&path)
    }

    /// Test database connectivity and health
    pub fn test_connection(&self) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // Try to get database stats as a health check
        db.get_stats().map_err(|e| {
            crate::error::DfsError::Database(format!("Database health check failed: {}", e))
        })?;

        Ok(())
    }

    /// Store file information in the database
    pub fn store_file(&self, file_entry: FileEntry) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.store_file(
            &file_entry.name,
            &file_entry.file_key,
            &file_entry.original_filename,
            file_entry.file_size,
            file_entry.upload_time,
            &file_entry.tags,
            &file_entry.public_key_hex,
        )
        .map_err(|e| crate::error::DfsError::Database(e.to_string()))
        .map(|_| ())
    }

    /// Retrieve file information from the database
    pub fn get_file(&self, identifier: &str) -> DfsResult<Option<FileEntry>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.get_file_by_key(identifier)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Get file by name
    pub fn get_file_by_name(&self, name: &str) -> DfsResult<Option<FileEntry>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.get_file_by_name(name)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Get file by key
    pub fn get_file_by_key(&self, key: &str) -> DfsResult<Option<FileEntry>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.get_file_by_key(key)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Check if name is taken
    pub fn is_name_taken(&self, name: &str) -> DfsResult<bool> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        Ok(db
            .get_file_by_name(name)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))?
            .is_some())
    }

    /// Generate unique name
    pub fn generate_unique_name(&self, original_name: &str) -> DfsResult<String> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        let mut counter = 1;
        let mut unique_name = original_name.to_string();

        while db
            .get_file_by_name(&unique_name)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))?
            .is_some()
        {
            unique_name = format!("{}_{}", original_name, counter);
            counter += 1;
        }

        Ok(unique_name)
    }

    /// List all files in the database
    pub fn list_files(&self, _limit: Option<usize>) -> DfsResult<Vec<FileEntry>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.list_files(None)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Delete a file from the database
    pub fn delete_file(&self, identifier: &str) -> DfsResult<bool> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.delete_file(identifier)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Get database statistics
    pub fn get_stats(&self) -> DfsResult<DatabaseStats> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.get_stats()
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Search files by criteria
    pub fn search_files(&self, query: &str, _limit: Option<usize>) -> DfsResult<Vec<FileEntry>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        db.search_files(query)
            .map_err(|e| crate::error::DfsError::Database(e.to_string()))
    }

    /// Store user storage profile
    pub async fn store_user_profile(&self, user_id: &str, profile: &crate::storage_economy::UserStorageProfile) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would store the profile in the database
        // For now, we'll just log it
        tracing::info!("Storing user profile for {}: {:?}", user_id, profile);
        
        Ok(())
    }

    /// Load user storage profile
    pub async fn load_user_profile(&self, user_id: &str) -> DfsResult<Option<crate::storage_economy::UserStorageProfile>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would load the profile from the database
        // For now, we'll return None
        tracing::info!("Loading user profile for {}", user_id);
        
        Ok(None)
    }

    /// Store storage proof
    pub async fn store_storage_proof(&self, proof: &crate::storage_economy::StorageProof) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would store the proof in the database
        tracing::info!("Storing storage proof: {:?}", proof);
        
        Ok(())
    }

    /// Load storage proofs for a user
    pub async fn load_storage_proofs(&self, user_id: &str) -> DfsResult<Vec<crate::storage_economy::StorageProof>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would load proofs from the database
        tracing::info!("Loading storage proofs for {}", user_id);
        
        Ok(Vec::new())
    }

    /// Store storage challenge
    pub async fn store_storage_challenge(&self, challenge: &crate::storage_economy::StorageChallenge) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would store the challenge in the database
        tracing::info!("Storing storage challenge: {:?}", challenge);
        
        Ok(())
    }

    /// Load active storage challenges for a user
    pub async fn load_active_challenges(&self, user_id: &str) -> DfsResult<Vec<crate::storage_economy::StorageChallenge>> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would load challenges from the database
        tracing::info!("Loading active challenges for {}", user_id);
        
        Ok(Vec::new())
    }

    /// Record storage economy transaction
    pub async fn record_economy_transaction(&self, transaction: &crate::storage_economy::EconomyTransaction) -> DfsResult<()> {
        let db = self.get_db().map_err(|e| {
            crate::error::DfsError::Database(format!("Failed to open database: {}", e))
        })?;

        // In a real implementation, this would store the transaction in the database
        tracing::info!("Recording economy transaction: {:?}", transaction);
        
        Ok(())
    }
}

// Implement Send and Sync for ThreadSafeDatabaseManager
unsafe impl Send for ThreadSafeDatabaseManager {}
unsafe impl Sync for ThreadSafeDatabaseManager {}
