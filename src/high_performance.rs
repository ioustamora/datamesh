use anyhow::Result;
use libp2p::kad::{Quorum, Record, RecordKey};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;

use crate::network_actor::NetworkHandle;
use crate::quorum_manager::QuorumManager;

/// Configuration for high-performance operations
#[derive(Debug, Clone)]
pub struct HighPerformanceConfig {
    /// Maximum concurrent upload operations
    pub max_concurrent_uploads: usize,
    /// Maximum concurrent download operations  
    pub max_concurrent_downloads: usize,
    /// Timeout for individual chunk operations
    pub chunk_timeout: Duration,
    /// Maximum retries for failed operations
    pub max_retries: u32,
    /// Connection pool size per peer
    pub connection_pool_size: usize,
    /// Enable operation pipelining
    pub enable_pipelining: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
    /// Enable aggressive caching
    pub enable_aggressive_caching: bool,
}

impl Default for HighPerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_uploads: 16,
            max_concurrent_downloads: 32,
            chunk_timeout: Duration::from_secs(10),
            max_retries: 3,
            connection_pool_size: 8,
            enable_pipelining: true,
            batch_size: 10,
            enable_aggressive_caching: true,
        }
    }
}

/// High-performance storage operations manager
pub struct HighPerformanceManager {
    network: Arc<NetworkHandle>,
    quorum_manager: Arc<QuorumManager>,
    config: HighPerformanceConfig,
    upload_semaphore: Arc<Semaphore>,
    download_semaphore: Arc<Semaphore>,
    operation_stats: Arc<RwLock<OperationStats>>,
    chunk_cache: Arc<RwLock<HashMap<RecordKey, CachedChunk>>>,
}

#[derive(Debug, Clone)]
struct CachedChunk {
    data: Vec<u8>,
    timestamp: Instant,
    access_count: u64,
}

#[derive(Debug, Default, Clone)]
struct OperationStats {
    total_uploads: u64,
    total_downloads: u64,
    failed_uploads: u64,
    failed_downloads: u64,
    average_upload_time: Duration,
    average_download_time: Duration,
    cache_hits: u64,
    cache_misses: u64,
}

impl HighPerformanceManager {
    pub fn new(
        network: Arc<NetworkHandle>,
        quorum_manager: Arc<QuorumManager>,
        config: HighPerformanceConfig,
    ) -> Self {
        let upload_semaphore = Arc::new(Semaphore::new(config.max_concurrent_uploads));
        let download_semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));

        Self {
            network,
            quorum_manager,
            config,
            upload_semaphore,
            download_semaphore,
            operation_stats: Arc::new(RwLock::new(OperationStats::default())),
            chunk_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store multiple chunks concurrently with optimized performance
    pub async fn store_chunks_concurrent(
        &self,
        chunks: Vec<(RecordKey, Vec<u8>)>,
    ) -> Result<Vec<Result<(), String>>> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Process chunks in batches to prevent overwhelming the network
        for batch in chunks.chunks(self.config.batch_size) {
            let batch_futures: Vec<_> = batch
                .iter()
                .map(|(key, data)| self.store_single_chunk_optimized(key.clone(), data.clone()))
                .collect();

            let batch_results = futures::future::join_all(batch_futures).await;
            results.extend(batch_results);

            // Small delay between batches to prevent network congestion
            if !self.config.enable_pipelining {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        // Update statistics
        {
            let mut stats = self.operation_stats.write().await;
            stats.total_uploads += chunks.len() as u64;
            stats.average_upload_time = 
                (stats.average_upload_time + start_time.elapsed()) / 2;
            stats.failed_uploads += results.iter().filter(|r| r.is_err()).count() as u64;
        }

        Ok(results)
    }

    /// Retrieve multiple chunks concurrently with optimized performance
    pub async fn retrieve_chunks_concurrent(
        &self,
        keys: Vec<RecordKey>,
    ) -> Result<Vec<Option<Vec<u8>>>> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Check cache first if aggressive caching is enabled
        if self.config.enable_aggressive_caching {
            let mut cache_results = Vec::new();
            let mut uncached_keys = Vec::new();

            {
                let cache = self.chunk_cache.read().await;
                for key in &keys {
                    if let Some(cached) = cache.get(key) {
                        // Check if cache entry is still fresh (1 hour)
                        if cached.timestamp.elapsed() < Duration::from_secs(3600) {
                            cache_results.push((key.clone(), Some(cached.data.clone())));
                            continue;
                        }
                    }
                    uncached_keys.push(key.clone());
                }
            }

            // Update cache hit statistics
            {
                let mut stats = self.operation_stats.write().await;
                stats.cache_hits += cache_results.len() as u64;
                stats.cache_misses += uncached_keys.len() as u64;
            }

            // Retrieve uncached chunks
            if !uncached_keys.is_empty() {
                let uncached_results = self.retrieve_uncached_chunks(uncached_keys).await?;
                
                // Merge results maintaining original order
                let mut cache_iter = cache_results.into_iter();
                let mut uncached_iter = uncached_results.into_iter();
                
                for key in &keys {
                    if let Some((_, data)) = cache_iter.find(|(k, _)| k == key) {
                        results.push(data);
                    } else {
                        results.push(uncached_iter.next().unwrap_or(None));
                    }
                }
            } else {
                results = cache_results.into_iter().map(|(_, data)| data).collect();
            }
        } else {
            results = self.retrieve_uncached_chunks(keys).await?;
        }

        // Update statistics
        {
            let mut stats = self.operation_stats.write().await;
            stats.total_downloads += results.len() as u64;
            stats.average_download_time = 
                (stats.average_download_time + start_time.elapsed()) / 2;
            stats.failed_downloads += results.iter().filter(|r| r.is_none()).count() as u64;
        }

        Ok(results)
    }

    /// Store a single chunk with optimizations
    async fn store_single_chunk_optimized(
        &self,
        key: RecordKey,
        data: Vec<u8>,
    ) -> Result<(), String> {
        let _permit = self.upload_semaphore.acquire().await.map_err(|e| e.to_string())?;

        let record = Record {
            key: key.clone(),
            value: data.clone(),
            publisher: None,
            expires: None,
        };

        // Get connected peers for quorum calculation
        let connected_peers = self.network.get_connected_peers().await
            .map_err(|e| e.to_string())?;

        // Calculate optimal quorum
        let quorum = self.quorum_manager.calculate_quorum(&connected_peers).await
            .unwrap_or(Quorum::One);

        // Retry logic with exponential backoff
        let mut attempts = 0;
        let mut delay = Duration::from_millis(100);

        while attempts < self.config.max_retries {
            let store_future = self.network.put_record(record.clone(), quorum.clone());
            
            match timeout(self.config.chunk_timeout, store_future).await {
                Ok(Ok(_)) => {
                    // Cache the chunk if aggressive caching is enabled
                    if self.config.enable_aggressive_caching {
                        let cached_chunk = CachedChunk {
                            data,
                            timestamp: Instant::now(),
                            access_count: 0,
                        };
                        let mut cache = self.chunk_cache.write().await;
                        cache.insert(key, cached_chunk);
                    }
                    return Ok(());
                }
                Ok(Err(e)) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        return Err(format!("Store failed after {} retries: {}", attempts, e));
                    }
                }
                Err(_) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        return Err(format!("Store timeout after {} retries", attempts));
                    }
                }
            }

            // Exponential backoff
            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay * 2, Duration::from_secs(5));
        }

        Err("Max retries exceeded".to_string())
    }

    /// Retrieve uncached chunks with optimized concurrent access
    async fn retrieve_uncached_chunks(
        &self,
        keys: Vec<RecordKey>,
    ) -> Result<Vec<Option<Vec<u8>>>> {
        let chunk_futures: Vec<_> = keys
            .into_iter()
            .map(|key| self.retrieve_single_chunk_optimized(key))
            .collect();

        // Execute all futures concurrently
        let all_results = futures::future::join_all(chunk_futures).await;
        let results: Result<Vec<_>, _> = all_results.into_iter().collect();
        let results = results?;

        Ok(results)
    }

    /// Retrieve a single chunk with optimizations
    async fn retrieve_single_chunk_optimized(
        &self,
        key: RecordKey,
    ) -> Result<Option<Vec<u8>>> {
        let _permit = self.download_semaphore.acquire().await?;

        // Check cache first
        if self.config.enable_aggressive_caching {
            let mut cache = self.chunk_cache.write().await;
            if let Some(cached) = cache.get_mut(&key) {
                if cached.timestamp.elapsed() < Duration::from_secs(3600) {
                    cached.access_count += 1;
                    return Ok(Some(cached.data.clone()));
                } else {
                    // Remove stale cache entry
                    cache.remove(&key);
                }
            }
        }

        // Retrieve from network with retry logic
        let mut attempts = 0;
        let mut delay = Duration::from_millis(50);

        while attempts < self.config.max_retries {
            let get_future = self.network.get_record(key.clone());
            
            match timeout(self.config.chunk_timeout, get_future).await {
                Ok(Ok(Some(record))) => {
                    let data = record.value;
                    
                    // Cache the retrieved chunk
                    if self.config.enable_aggressive_caching {
                        let cached_chunk = CachedChunk {
                            data: data.clone(),
                            timestamp: Instant::now(),
                            access_count: 1,
                        };
                        let mut cache = self.chunk_cache.write().await;
                        cache.insert(key, cached_chunk);
                    }
                    
                    return Ok(Some(data));
                }
                Ok(Ok(None)) => return Ok(None),
                Ok(Err(_)) | Err(_) => {
                    attempts += 1;
                    if attempts < self.config.max_retries {
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, Duration::from_secs(2));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> OperationStats {
        self.operation_stats.read().await.clone()
    }

    /// Clean up old cache entries
    pub async fn cleanup_cache(&self, max_age: Duration) {
        let mut cache = self.chunk_cache.write().await;
        cache.retain(|_, chunk| chunk.timestamp.elapsed() < max_age);
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, u64, u64) {
        let cache = self.chunk_cache.read().await;
        let size = cache.len();
        let total_access = cache.values().map(|c| c.access_count).sum();
        let total_size: u64 = cache.values().map(|c| c.data.len() as u64).sum();
        (size, total_access, total_size)
    }

    /// Warm up cache with frequently accessed chunks
    pub async fn warmup_cache(&self, keys: Vec<RecordKey>) -> Result<()> {
        let warmup_chunks = self.retrieve_chunks_concurrent(keys).await?;
        tracing::info!("Cache warmed up with {} chunks", warmup_chunks.len());
        Ok(())
    }

    /// Update configuration for runtime tuning
    pub fn update_config(&mut self, config: HighPerformanceConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &HighPerformanceConfig {
        &self.config
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operations_per_second: f64,
    pub average_latency: Duration,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub concurrent_operations: usize,
}

impl HighPerformanceManager {
    /// Calculate current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let stats = self.operation_stats.read().await;
        
        let total_ops = stats.total_uploads + stats.total_downloads;
        let total_failures = stats.failed_uploads + stats.failed_downloads;
        let total_cache_ops = stats.cache_hits + stats.cache_misses;
        
        PerformanceMetrics {
            operations_per_second: total_ops as f64 / 60.0, // Approximate ops/sec
            average_latency: (stats.average_upload_time + stats.average_download_time) / 2,
            success_rate: if total_ops > 0 {
                (total_ops - total_failures) as f64 / total_ops as f64
            } else {
                1.0
            },
            cache_hit_rate: if total_cache_ops > 0 {
                stats.cache_hits as f64 / total_cache_ops as f64
            } else {
                0.0
            },
            concurrent_operations: self.upload_semaphore.available_permits() + 
                                 self.download_semaphore.available_permits(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quorum_manager::{QuorumManager, QuorumConfig};
    use libp2p::kad::RecordKey;

    #[tokio::test]
    async fn test_performance_config() {
        let config = HighPerformanceConfig::default();
        assert!(config.max_concurrent_uploads > 0);
        assert!(config.max_concurrent_downloads > 0);
        assert!(config.chunk_timeout > Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        // This would require a mock network handle for full testing
        // For now, we test the cache data structures
        let cached_chunk = CachedChunk {
            data: vec![1, 2, 3, 4],
            timestamp: Instant::now(),
            access_count: 1,
        };
        
        assert_eq!(cached_chunk.data.len(), 4);
        assert!(cached_chunk.timestamp.elapsed() < Duration::from_secs(1));
    }
}