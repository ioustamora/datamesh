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
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Caching configuration
    pub cache: CacheConfig,
    /// API server configuration
    pub api: ApiConfig,
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

/// Performance configuration for concurrent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Chunk operation settings
    pub chunks: ChunkPerformanceConfig,
    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,
    /// Timeout configurations
    pub timeouts: TimeoutConfig,
}

/// Configuration for concurrent chunk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPerformanceConfig {
    /// Maximum concurrent chunk retrievals
    pub max_concurrent_retrievals: usize,
    /// Maximum concurrent chunk uploads
    pub max_concurrent_uploads: usize,
    /// Timeout for individual chunk operations in seconds
    pub chunk_timeout_secs: u64,
    /// Number of retry attempts for failed chunks
    pub retry_failed_chunks: u32,
    /// Whether to prefer faster responding peers
    pub prefer_fast_peers: bool,
    /// Maximum time to wait for peer responses in seconds
    pub peer_response_timeout_secs: u64,
    /// Enable chunk operation metrics collection
    pub enable_metrics: bool,
}

/// Configuration for connection pooling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections per peer
    pub max_connections_per_peer: usize,
    /// Connection idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Connection keep-alive interval in seconds
    pub keep_alive_interval_secs: u64,
}

/// Configuration for various timeout values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// DHT query timeout in seconds
    pub dht_query_timeout_secs: u64,
    /// File operation timeout in seconds
    pub file_operation_timeout_secs: u64,
    /// Peer discovery timeout in seconds
    pub peer_discovery_timeout_secs: u64,
}

/// Configuration for intelligent caching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum size of file cache in GB
    pub file_cache_size_gb: f64,
    /// Maximum size of chunk cache in MB
    pub chunk_cache_size_mb: u64,
    /// Maximum size for individual files to be cached in MB
    pub max_file_size_mb: u64,
    /// Whether to enable predictive preloading
    pub preload_popular: bool,
    /// Time-to-live for cached items in hours
    pub ttl_hours: u64,
    /// Cleanup interval in minutes
    pub cleanup_interval_minutes: u64,
    /// Cache policies configuration
    pub policies: CachePoliciesConfig,
}

/// Configuration for cache policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePoliciesConfig {
    /// Weight for LRU (Least Recently Used) factor
    pub lru_weight: f64,
    /// Weight for frequency of access factor
    pub frequency_weight: f64,
    /// Weight for recency of access factor
    pub recency_weight: f64,
    /// Weight for file size factor (smaller files preferred)
    pub size_weight: f64,
}

/// Configuration for API server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum upload file size in bytes
    pub max_upload_size: u64,
    /// API rate limiting - requests per minute
    pub rate_limit_per_minute: u32,
    /// Enable HTTPS
    pub enable_https: bool,
    /// Path to TLS certificate file
    pub cert_path: Option<PathBuf>,
    /// Path to TLS private key file
    pub key_path: Option<PathBuf>,
    /// Enable Swagger UI
    pub enable_swagger: bool,
    /// API prefix (e.g., "/api/v1")
    pub api_prefix: String,
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
            performance: PerformanceConfig {
                chunks: ChunkPerformanceConfig {
                    max_concurrent_retrievals: 8,
                    max_concurrent_uploads: 4,
                    chunk_timeout_secs: 10,
                    retry_failed_chunks: 3,
                    prefer_fast_peers: true,
                    peer_response_timeout_secs: 5,
                    enable_metrics: true,
                },
                connection_pool: ConnectionPoolConfig {
                    max_connections_per_peer: 3,
                    idle_timeout_secs: 300, // 5 minutes
                    keep_alive_interval_secs: 60, // 1 minute
                },
                timeouts: TimeoutConfig {
                    dht_query_timeout_secs: 15,
                    file_operation_timeout_secs: 120, // 2 minutes
                    peer_discovery_timeout_secs: 30,
                },
            },
            cache: CacheConfig {
                file_cache_size_gb: 2.0,
                chunk_cache_size_mb: 500,
                max_file_size_mb: 100,
                preload_popular: true,
                ttl_hours: 24,
                cleanup_interval_minutes: 60,
                policies: CachePoliciesConfig {
                    lru_weight: 0.4,
                    frequency_weight: 0.3,
                    recency_weight: 0.2,
                    size_weight: 0.1,
                },
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_upload_size: 100 * 1024 * 1024, // 100MB
                rate_limit_per_minute: 1000,
                enable_https: false,
                cert_path: None,
                key_path: None,
                enable_swagger: true,
                api_prefix: "/api/v1".to_string(),
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

    /// Get chunk timeout as Duration
    pub fn chunk_timeout(&self) -> Duration {
        Duration::from_secs(self.performance.chunks.chunk_timeout_secs)
    }

    /// Get peer response timeout as Duration
    pub fn peer_response_timeout(&self) -> Duration {
        Duration::from_secs(self.performance.chunks.peer_response_timeout_secs)
    }

    /// Get DHT query timeout as Duration
    pub fn dht_query_timeout(&self) -> Duration {
        Duration::from_secs(self.performance.timeouts.dht_query_timeout_secs)
    }

    /// Get file operation timeout as Duration
    pub fn file_operation_timeout(&self) -> Duration {
        Duration::from_secs(self.performance.timeouts.file_operation_timeout_secs)
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

impl ChunkPerformanceConfig {
    /// Convert configuration to concurrent chunk config
    pub fn to_concurrent_chunk_config(&self) -> crate::concurrent_chunks::ConcurrentChunkConfig {
        crate::concurrent_chunks::ConcurrentChunkConfig {
            max_concurrent_retrievals: self.max_concurrent_retrievals,
            max_concurrent_uploads: self.max_concurrent_uploads,
            chunk_timeout: Duration::from_secs(self.chunk_timeout_secs),
            retry_failed_chunks: self.retry_failed_chunks,
            prefer_fast_peers: self.prefer_fast_peers,
            peer_response_timeout: Duration::from_secs(self.peer_response_timeout_secs),
        }
    }
}

impl CacheConfig {
    /// Convert configuration to smart cache config
    pub fn to_smart_cache_config(&self) -> crate::smart_cache::CacheConfig {
        crate::smart_cache::CacheConfig {
            file_cache_size_bytes: (self.file_cache_size_gb * 1024.0 * 1024.0 * 1024.0) as u64,
            chunk_cache_size_bytes: self.chunk_cache_size_mb * 1024 * 1024,
            max_file_size_bytes: self.max_file_size_mb * 1024 * 1024,
            preload_popular: self.preload_popular,
            ttl_hours: self.ttl_hours,
            cleanup_interval: Duration::from_secs(self.cleanup_interval_minutes * 60),
            policies: crate::smart_cache::CachePolicies {
                lru_weight: self.policies.lru_weight,
                frequency_weight: self.policies.frequency_weight,
                recency_weight: self.policies.recency_weight,
                size_weight: self.policies.size_weight,
            },
        }
    }
}

impl ApiConfig {
    /// Convert configuration to API server config
    pub fn to_api_server_config(&self) -> crate::api_server::ApiConfig {
        crate::api_server::ApiConfig {
            host: self.host.clone(),
            port: self.port,
            max_upload_size: self.max_upload_size,
            rate_limit_per_minute: self.rate_limit_per_minute,
            enable_https: self.enable_https,
            cert_path: self.cert_path.clone(),
            key_path: self.key_path.clone(),
            enable_swagger: self.enable_swagger,
            api_prefix: self.api_prefix.clone(),
        }
    }
}

/// Generate a default configuration file
pub fn generate_config(config_path: &PathBuf) -> crate::error::DfsResult<()> {
    let config = Config::default();
    config.save(config_path)?;
    Ok(())
}
