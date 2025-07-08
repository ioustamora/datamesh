use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use libp2p::{PeerId, Multiaddr};

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
    /// Multi-bootstrap peer configuration
    pub bootstrap: BootstrapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapNode {
    pub peer_id: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// List of bootstrap peers
    pub peers: Vec<BootstrapPeerConfig>,
    /// Maximum number of bootstrap connection attempts
    pub max_attempts: u32,
    /// Retry interval in seconds
    pub retry_interval_secs: u64,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum number of connections to maintain
    pub max_connections: usize,
    /// Preferred region for bootstrap connections
    pub preferred_region: Option<String>,
    /// Exponential backoff configuration
    pub backoff: BackoffConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeerConfig {
    /// Peer ID as a string
    pub peer_id: String,
    /// List of multiaddresses for this peer
    pub addresses: Vec<String>,
    /// Priority level (1 = highest, 10 = lowest)
    pub priority: u8,
    /// Geographic region
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// Base delay in seconds
    pub base_delay_secs: u64,
    /// Maximum delay in seconds
    pub max_delay_secs: u64,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Maximum number of retry attempts
    pub max_attempts: u32,
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
                bootstrap: BootstrapConfig {
                    peers: vec![],
                    max_attempts: 5,
                    retry_interval_secs: 2,
                    health_check_interval_secs: 30,
                    min_connections: 3,
                    max_connections: 8,
                    preferred_region: None,
                    backoff: BackoffConfig {
                        base_delay_secs: 2,
                        max_delay_secs: 300,
                        multiplier: 2.0,
                        max_attempts: 5,
                    },
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

impl BootstrapConfig {
    /// Convert configuration to bootstrap manager
    pub fn to_bootstrap_manager(&self) -> Result<crate::bootstrap_manager::BootstrapManager, Box<dyn std::error::Error>> {
        use crate::bootstrap_manager::{BootstrapManager, BootstrapPeer, ExponentialBackoff};
        use std::str::FromStr;
        
        let mut manager = BootstrapManager::new()
            .with_connection_limits(self.min_connections, self.max_connections);
        
        if let Some(ref region) = self.preferred_region {
            manager = manager.with_preferred_region(region.clone());
        }
        
        // Configure retry strategy
        let retry_strategy = ExponentialBackoff::new(
            Duration::from_secs(self.backoff.base_delay_secs),
            Duration::from_secs(self.backoff.max_delay_secs),
            self.backoff.multiplier,
            self.backoff.max_attempts,
        );
        manager = manager.with_retry_strategy(retry_strategy);
        
        // Add bootstrap peers
        for peer_config in &self.peers {
            let peer_id = PeerId::from_str(&peer_config.peer_id)?;
            let addresses: Result<Vec<Multiaddr>, _> = peer_config.addresses
                .iter()
                .map(|addr| Multiaddr::from_str(addr))
                .collect();
            
            let mut peer = BootstrapPeer::new(peer_id, addresses?)
                .with_priority(peer_config.priority);
            
            if let Some(ref region) = peer_config.region {
                peer = peer.with_region(region.clone());
            }
            
            manager.add_bootstrap_peer(peer);
        }
        
        Ok(manager)
    }
}
}
