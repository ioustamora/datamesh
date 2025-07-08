use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the DFS system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Network configuration
    pub network: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Default port for listening
    pub default_port: u16,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<BootstrapNode>,
    /// DHT replication factor
    pub replication_factor: usize,
    /// DHT storage configuration
    pub dht_storage: DHTStorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapNode {
    pub peer_id: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHTStorageConfig {
    /// Database storage path
    pub db_path: Option<PathBuf>,
    /// Memory cache size
    pub cache_size: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Default TTL for chunks in seconds
    pub default_ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Default keys directory
    pub keys_dir: Option<PathBuf>,
    /// Number of data shards for Reed-Solomon encoding
    pub data_shards: usize,
    /// Number of parity shards for Reed-Solomon encoding
    pub parity_shards: usize,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Chunk size for large files
    pub chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Key generation algorithm
    pub key_algorithm: String,
    /// Enable file integrity verification
    pub verify_integrity: bool,
    /// Encryption strength
    pub encryption_strength: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Enable structured logging
    pub structured: bool,
    /// Log to file
    pub log_file: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                default_port: 40871,
                connection_timeout_secs: 30,
                max_connections: 100,
                bootstrap_nodes: vec![],
                replication_factor: 3,
                dht_storage: DHTStorageConfig {
                    db_path: None,
                    cache_size: 1000,
                    cleanup_interval_secs: 24 * 60 * 60, // 24 hours
                    default_ttl_secs: 24 * 60 * 60, // 24 hours
                },
            },
            storage: StorageConfig {
                keys_dir: None,
                data_shards: 4,
                parity_shards: 2,
                max_file_size: 100 * 1024 * 1024, // 100MB
                chunk_size: 1024 * 1024, // 1MB
            },
            security: SecurityConfig {
                key_algorithm: "secp256k1".to_string(),
                verify_integrity: true,
                encryption_strength: "aes256".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                structured: true,
                log_file: None,
            },
        }
    }
}

impl Config {
    /// Load configuration from file, falling back to defaults
    pub fn load_or_default(config_path: Option<PathBuf>) -> crate::error::DfsResult<Self> {
        if let Some(path) = config_path {
            if path.exists() {
                let config_str = std::fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&config_str)
                    .map_err(|e| crate::error::DfsError::Serialization(format!("Config parse error: {}", e)))?;
                tracing::info!("Loaded configuration from {:?}", path);
                return Ok(config);
            }
        }
        
        tracing::info!("Using default configuration");
        Ok(Config::default())
    }

    /// Save configuration to file
    pub fn save(&self, config_path: &PathBuf) -> crate::error::DfsResult<()> {
        let config_str = toml::to_string_pretty(self)
            .map_err(|e| crate::error::DfsError::Serialization(format!("Config serialize error: {}", e)))?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(config_path, config_str)?;
        tracing::info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.network.connection_timeout_secs)
    }
}
