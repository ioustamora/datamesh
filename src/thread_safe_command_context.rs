/// Thread-Safe Command Context
/// 
/// This module provides thread-safe command context for actor-based commands
/// that can be shared across threads without Send/Sync issues.

use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;

use crate::cli::Cli;
use crate::key_manager::KeyManager;
use crate::config::Config;
use crate::network_actor::NetworkHandle;
use crate::thread_safe_database::ThreadSafeDatabaseManager;
use crate::error::DfsResult;

/// Thread-safe context for actor-based command handlers
#[derive(Clone, Debug)]
pub struct ThreadSafeCommandContext {
    pub cli: Cli,
    pub key_manager: Arc<KeyManager>,
    pub config: Arc<Config>,
    pub network: Arc<NetworkHandle>,
    pub database: Arc<ThreadSafeDatabaseManager>,
}

impl ThreadSafeCommandContext {
    /// Create a new thread-safe command context
    pub async fn new(cli: Cli, key_manager: Arc<KeyManager>, config: Arc<Config>) -> Result<Self> {
        // Create network handle
        let network = Arc::new(NetworkHandle::new(&cli, &config).await?);
        
        // Create thread-safe database manager
        let db_path = crate::database::get_default_db_path()?;
        let database = Arc::new(ThreadSafeDatabaseManager::new(&db_path.to_string_lossy())?);
        
        Ok(ThreadSafeCommandContext {
            cli,
            key_manager,
            config,
            network,
            database,
        })
    }
    
    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<crate::network_actor::NetworkStats> {
        self.network.get_network_stats().await
            .map_err(|e| anyhow::anyhow!(e))
    }
    
    /// Bootstrap the network
    pub async fn bootstrap(&self) -> Result<()> {
        self.network.bootstrap().await
            .map_err(|e| anyhow::anyhow!(e))
    }
    
    /// Store a file using the actor-based system
    pub async fn store_file(
        &self,
        file_path: &PathBuf,
        public_key: &Option<String>,
        name: &Option<String>,
        tags: &Option<Vec<String>>,
    ) -> DfsResult<String> {
        // Use the actor-based file storage system
        crate::actor_file_storage::store_file_with_network(
            &self.cli,
            &self.key_manager,
            file_path,
            public_key,
            name,
            tags,
            self.network.clone(),
            self.database.clone(),
        ).await
    }
    
    /// Retrieve a file using the actor-based system
    pub async fn retrieve_file(
        &self,
        identifier: &str,
        output_path: &PathBuf,
        private_key: &Option<String>,
    ) -> DfsResult<()> {
        // Use the actor-based file storage system
        crate::actor_file_storage::retrieve_file_with_network(
            &self.cli,
            &self.key_manager,
            identifier,
            output_path,
            private_key,
            self.network.clone(),
            self.database.clone(),
        ).await
    }
}

// Implement Send and Sync for ThreadSafeCommandContext
unsafe impl Send for ThreadSafeCommandContext {}
unsafe impl Sync for ThreadSafeCommandContext {}
