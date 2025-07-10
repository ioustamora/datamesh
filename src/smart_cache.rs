/// Advanced Caching System Module
///
/// This module implements the Advanced Caching System as outlined in the
/// DataMesh Application & Network Improvements Roadmap. It provides:
/// - Intelligent file caching with access pattern analysis
/// - Multi-level caching (file and chunk level)
/// - Predictive cache prefetching
/// - Smart eviction policies based on access patterns
/// - Performance metrics and monitoring
/// - Integration with concurrent chunk operations

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::num::NonZeroUsize;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use lru::LruCache;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};
use crate::concurrent_chunks::ConcurrentChunkManager;

/// Configuration for the smart cache system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum size of file cache in bytes
    pub file_cache_size_bytes: u64,
    /// Maximum size of chunk cache in bytes
    pub chunk_cache_size_bytes: u64,
    /// Maximum size for individual files to be cached
    pub max_file_size_bytes: u64,
    /// Whether to enable predictive preloading
    pub preload_popular: bool,
    /// Time-to-live for cached items in hours
    pub ttl_hours: u64,
    /// Cleanup interval for expired items
    pub cleanup_interval: Duration,
    /// Cache policies configuration
    pub policies: CachePolicies,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            file_cache_size_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            chunk_cache_size_bytes: 500 * 1024 * 1024,     // 500MB
            max_file_size_bytes: 100 * 1024 * 1024,        // 100MB
            preload_popular: true,
            ttl_hours: 24,
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            policies: CachePolicies::default(),
        }
    }
}

/// Cache policies for smart eviction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePolicies {
    /// Weight for LRU (Least Recently Used) factor
    pub lru_weight: f64,
    /// Weight for frequency of access factor
    pub frequency_weight: f64,
    /// Weight for recency of access factor
    pub recency_weight: f64,
    /// Weight for file size factor (smaller files preferred)
    pub size_weight: f64,
}

impl Default for CachePolicies {
    fn default() -> Self {
        Self {
            lru_weight: 0.4,
            frequency_weight: 0.3,
            recency_weight: 0.2,
            size_weight: 0.1,
        }
    }
}

/// Priority levels for cached items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Metadata for cached files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_key: String,
    pub file_name: String,
    pub file_size: u64,
    pub stored_at: DateTime<Utc>,
    pub content_type: Option<String>,
    pub checksum: String,
}

/// A cached file with metadata and access tracking
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub data: Vec<u8>,
    pub metadata: FileMetadata,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
    pub cache_priority: CachePriority,
    pub ttl: Duration,
}

/// Types of file access for pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    CacheHit,
    NetworkFetch,
    Preload,
}

/// File access event for pattern tracking
#[derive(Debug, Clone)]
pub struct FileAccess {
    pub file_key: String,
    pub access_type: AccessType,
    pub timestamp: DateTime<Utc>,
    pub response_time: Duration,
    pub file_size: u64,
}

/// Prediction score for file access likelihood
#[derive(Debug, Clone)]
pub struct AccessPrediction {
    pub file_key: String,
    pub confidence: f64,
    pub predicted_access_time: DateTime<Utc>,
    pub reason: String,
}

/// Simple LRU-based predictor
#[derive(Debug)]
pub struct LRUPredictor {
    recent_accesses: VecDeque<String>,
    max_history: usize,
}

impl LRUPredictor {
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_accesses: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn record_access(&mut self, file_key: String) {
        // Remove if already exists
        if let Some(pos) = self.recent_accesses.iter().position(|x| x == &file_key) {
            self.recent_accesses.remove(pos);
        }
        
        // Add to front
        self.recent_accesses.push_front(file_key);
        
        // Trim if too large
        if self.recent_accesses.len() > self.max_history {
            self.recent_accesses.pop_back();
        }
    }

    pub fn predict_access(&self, file_key: &str) -> f64 {
        if let Some(pos) = self.recent_accesses.iter().position(|x| x == file_key) {
            // More recent accesses have higher prediction scores
            1.0 - (pos as f64 / self.max_history as f64)
        } else {
            0.0
        }
    }

    pub fn get_predicted_popular_files(&self, limit: usize) -> Vec<String> {
        self.recent_accesses
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }
}

/// Analyzes file access patterns for intelligent caching decisions
pub struct AccessPatternAnalyzer {
    access_history: VecDeque<FileAccess>,
    popularity_scores: HashMap<String, f64>,
    prediction_model: LRUPredictor,
    max_history_size: usize,
}

impl AccessPatternAnalyzer {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            access_history: VecDeque::with_capacity(max_history_size),
            popularity_scores: HashMap::new(),
            prediction_model: LRUPredictor::new(max_history_size / 4),
            max_history_size,
        }
    }

    pub fn record_access(&mut self, file_key: String, access_type: AccessType, response_time: Duration, file_size: u64) {
        let access = FileAccess {
            file_key: file_key.clone(),
            access_type,
            timestamp: Utc::now(),
            response_time,
            file_size,
        };

        // Add to history
        self.access_history.push_back(access);
        
        // Trim history if too large
        if self.access_history.len() > self.max_history_size {
            self.access_history.pop_front();
        }

        // Update prediction model
        self.prediction_model.record_access(file_key.clone());

        // Update popularity scores
        self.update_popularity_score(&file_key);
    }

    fn update_popularity_score(&mut self, file_key: &str) {
        let now = Utc::now();
        let recent_accesses = self.access_history
            .iter()
            .rev()
            .take(100) // Look at last 100 accesses
            .filter(|access| access.file_key == file_key)
            .count();

        // Calculate score based on recent access frequency
        let base_score = recent_accesses as f64;
        
        // Apply time decay - more recent accesses are weighted higher
        let time_weighted_score = self.access_history
            .iter()
            .rev()
            .take(100)
            .filter(|access| access.file_key == file_key)
            .map(|access| {
                let age_hours = (now - access.timestamp).num_hours() as f64;
                let decay_factor = (-age_hours / 24.0).exp(); // Exponential decay over 24 hours
                decay_factor
            })
            .sum::<f64>();

        let final_score = base_score + time_weighted_score;
        self.popularity_scores.insert(file_key.to_string(), final_score);
    }

    pub fn get_access_frequency(&self, file_key: &str) -> u64 {
        self.access_history
            .iter()
            .filter(|access| access.file_key == file_key)
            .count() as u64
    }

    pub fn predict_future_access(&self, file_key: &str) -> f64 {
        // Combine multiple factors for prediction
        let lru_score = self.prediction_model.predict_access(file_key);
        let popularity_score = self.popularity_scores.get(file_key).copied().unwrap_or(0.0);
        let frequency_score = self.get_access_frequency(file_key) as f64 / 10.0; // Normalize

        // Weighted combination
        (lru_score * 0.4 + popularity_score * 0.4 + frequency_score * 0.2).min(1.0)
    }

    pub fn get_predicted_popular_files(&self, limit: usize) -> Vec<String> {
        let mut files_with_scores: Vec<(String, f64)> = self.popularity_scores
            .iter()
            .map(|(file_key, score)| (file_key.clone(), *score))
            .collect();

        files_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        files_with_scores
            .into_iter()
            .take(limit)
            .map(|(file_key, _)| file_key)
            .collect()
    }

    pub fn should_cache_file(&self, file_key: &str, file_size: u64, max_file_size: u64) -> bool {
        // Don't cache very large files unless they're very popular
        if file_size > max_file_size {
            let frequency = self.get_access_frequency(file_key);
            return frequency >= 5; // Only cache large files if accessed 5+ times
        }

        // Cache small files that have been accessed multiple times
        if file_size < 1_000_000 && self.get_access_frequency(file_key) >= 2 {
            return true;
        }

        // Use prediction for borderline cases
        self.predict_future_access(file_key) > 0.7
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub file_cache_hits: u64,
    pub file_cache_misses: u64,
    pub chunk_cache_hits: u64,
    pub chunk_cache_misses: u64,
    pub total_cached_files: usize,
    pub total_cached_chunks: usize,
    pub cache_size_bytes: u64,
    pub hit_ratio: f64,
    pub average_response_time_ms: f64,
    pub evictions: u64,
    pub preloads: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            file_cache_hits: 0,
            file_cache_misses: 0,
            chunk_cache_hits: 0,
            chunk_cache_misses: 0,
            total_cached_files: 0,
            total_cached_chunks: 0,
            cache_size_bytes: 0,
            hit_ratio: 0.0,
            average_response_time_ms: 0.0,
            evictions: 0,
            preloads: 0,
        }
    }
}

/// Main smart cache manager implementing intelligent caching
pub struct SmartCacheManager {
    config: CacheConfig,
    file_cache: Arc<RwLock<LruCache<String, CachedFile>>>,
    chunk_cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    pub access_patterns: Arc<Mutex<AccessPatternAnalyzer>>,
    stats: Arc<RwLock<CacheStats>>,
    concurrent_chunks: Option<Arc<ConcurrentChunkManager>>,
}

impl SmartCacheManager {
    /// Create a new smart cache manager
    pub fn new(config: CacheConfig) -> Self {
        let file_cache_capacity = NonZeroUsize::new(
            (config.file_cache_size_bytes / 1024 / 1024) as usize // Rough estimate of files
        ).unwrap_or(NonZeroUsize::new(1000).unwrap());
        
        let chunk_cache_capacity = NonZeroUsize::new(
            (config.chunk_cache_size_bytes / 64 / 1024) as usize // Assuming 64KB average chunk size
        ).unwrap_or(NonZeroUsize::new(8000).unwrap());

        Self {
            config: config.clone(),
            file_cache: Arc::new(RwLock::new(LruCache::new(file_cache_capacity))),
            chunk_cache: Arc::new(RwLock::new(LruCache::new(chunk_cache_capacity))),
            access_patterns: Arc::new(Mutex::new(AccessPatternAnalyzer::new(10000))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            concurrent_chunks: None,
        }
    }

    /// Set the concurrent chunk manager for network operations
    pub fn set_concurrent_chunks(&mut self, concurrent_chunks: Arc<ConcurrentChunkManager>) {
        self.concurrent_chunks = Some(concurrent_chunks);
    }

    /// Start background tasks (cleanup, preloading, etc.)
    pub async fn start_background_tasks(&self) {
        self.start_cleanup_task().await;
        if self.config.preload_popular {
            self.start_preload_task().await;
        }
    }

    /// Start periodic cleanup of expired cache entries
    async fn start_cleanup_task(&self) {
        let file_cache = self.file_cache.clone();
        let cleanup_interval = self.config.cleanup_interval;
        let ttl_duration = Duration::from_secs(self.config.ttl_hours * 3600);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let mut cache = file_cache.write().await;
                let now = Utc::now();
                let mut expired_keys = Vec::new();
                
                // Find expired entries
                for (key, cached_file) in cache.iter() {
                    let age = now - cached_file.cached_at;
                    if age.to_std().unwrap_or(Duration::ZERO) > ttl_duration {
                        expired_keys.push(key.clone());
                    }
                }
                
                // Remove expired entries
                for key in expired_keys {
                    cache.pop(&key);
                    debug!("Evicted expired cache entry: {}", key);
                }
            }
        });
    }

    /// Start periodic preloading of popular files
    async fn start_preload_task(&self) {
        let access_patterns = self.access_patterns.clone();
        let cache_manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                if let Err(e) = cache_manager.preload_popular_files().await {
                    warn!("Failed to preload popular files: {}", e);
                }
            }
        });
    }

    /// Smart file retrieval with caching
    pub async fn get_file_smart(&self, file_key: &str) -> Result<Vec<u8>> {
        let start_time = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.get_from_cache(file_key).await? {
            let response_time = start_time.elapsed();
            
            // Update access patterns
            {
                let mut patterns = self.access_patterns.lock().await;
                patterns.record_access(
                    file_key.to_string(),
                    AccessType::CacheHit,
                    response_time,
                    cached.data.len() as u64,
                );
            }
            
            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.file_cache_hits += 1;
                self.update_hit_ratio(&mut stats);
            }
            
            debug!("Cache hit for file: {} ({}ms)", file_key, response_time.as_millis());
            return Ok(cached.data);
        }
        
        // Cache miss - retrieve from network
        let data = self.retrieve_from_network(file_key).await?;
        let response_time = start_time.elapsed();
        
        // Analyze if this file should be cached
        let should_cache = {
            let patterns = self.access_patterns.lock().await;
            patterns.should_cache_file(file_key, data.len() as u64, self.config.max_file_size_bytes)
        };
        
        if should_cache {
            self.cache_file_intelligent(file_key, data.clone()).await?;
        }
        
        // Update access patterns
        {
            let mut patterns = self.access_patterns.lock().await;
            patterns.record_access(
                file_key.to_string(),
                AccessType::NetworkFetch,
                response_time,
                data.len() as u64,
            );
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.file_cache_misses += 1;
            self.update_hit_ratio(&mut stats);
        }
        
        debug!("Cache miss for file: {} ({}ms)", file_key, response_time.as_millis());
        Ok(data)
    }

    /// Get file from cache if it exists
    async fn get_from_cache(&self, file_key: &str) -> Result<Option<CachedFile>> {
        let mut cache = self.file_cache.write().await;
        
        if let Some(cached_file) = cache.get_mut(file_key) {
            // Update access metadata
            cached_file.access_count += 1;
            cached_file.last_accessed = Utc::now();
            
            Ok(Some(cached_file.clone()))
        } else {
            Ok(None)
        }
    }

    /// Intelligently cache a file based on policies
    pub async fn cache_file_intelligent(&self, file_key: &str, data: Vec<u8>) -> Result<()> {
        let file_size = data.len() as u64;
        
        // Don't cache files that are too large
        if file_size > self.config.max_file_size_bytes {
            return Ok(());
        }
        
        let data_hash = blake3::hash(&data);
        let cached_file = CachedFile {
            data,
            metadata: FileMetadata {
                file_key: file_key.to_string(),
                file_name: file_key.to_string(), // TODO: Get actual filename
                file_size,
                stored_at: Utc::now(),
                content_type: None,
                checksum: data_hash.to_hex().to_string(),
            },
            access_count: 1,
            last_accessed: Utc::now(),
            cached_at: Utc::now(),
            cache_priority: self.calculate_cache_priority(file_key, file_size).await,
            ttl: Duration::from_secs(self.config.ttl_hours * 3600),
        };
        
        // Check if we need to evict files to make space
        let current_size = self.calculate_cache_size().await;
        if current_size + file_size > self.config.file_cache_size_bytes {
            self.evict_files_intelligently(file_size).await?;
        }
        
        // Add to cache
        {
            let mut cache = self.file_cache.write().await;
            cache.put(file_key.to_string(), cached_file);
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_cached_files = {
                let cache = self.file_cache.read().await;
                cache.len()
            };
            stats.cache_size_bytes = self.calculate_cache_size().await;
        }
        
        debug!("Cached file: {} ({} bytes)", file_key, file_size);
        Ok(())
    }

    /// Calculate cache priority for a file
    async fn calculate_cache_priority(&self, file_key: &str, _file_size: u64) -> CachePriority {
        let patterns = self.access_patterns.lock().await;
        let frequency = patterns.get_access_frequency(file_key);
        let prediction = patterns.predict_future_access(file_key);
        
        // Priority based on access frequency and prediction
        if frequency >= 10 || prediction > 0.9 {
            CachePriority::Critical
        } else if frequency >= 5 || prediction > 0.7 {
            CachePriority::High
        } else if frequency >= 2 || prediction > 0.5 {
            CachePriority::Medium
        } else {
            CachePriority::Low
        }
    }

    /// Intelligently evict files to make space
    async fn evict_files_intelligently(&self, needed_space: u64) -> Result<()> {
        let mut cache = self.file_cache.write().await;
        let mut freed_space = 0u64;
        let mut candidates = Vec::new();
        
        // Collect eviction candidates with scores
        for (key, cached_file) in cache.iter() {
            let score = self.calculate_eviction_score(cached_file).await;
            candidates.push((key.clone(), score, cached_file.metadata.file_size));
        }
        
        // Sort by eviction score (lowest first = most suitable for eviction)
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Evict files until we have enough space
        for (key, _score, file_size) in candidates {
            if freed_space >= needed_space {
                break;
            }
            
            cache.pop(&key);
            freed_space += file_size;
            
            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }
            
            debug!("Evicted file for space: {} ({} bytes)", key, file_size);
        }
        
        if freed_space < needed_space {
            return Err(anyhow!("Could not free enough cache space"));
        }
        
        Ok(())
    }

    /// Calculate eviction score (lower = more suitable for eviction)
    async fn calculate_eviction_score(&self, cached_file: &CachedFile) -> f64 {
        let now = Utc::now();
        let policies = &self.config.policies;
        
        // LRU factor (time since last access)
        let hours_since_access = (now - cached_file.last_accessed).num_hours() as f64;
        let lru_score = hours_since_access / 24.0; // Normalize to days
        
        // Frequency factor (inverse of access count)
        let frequency_score = 1.0 / (cached_file.access_count as f64 + 1.0);
        
        // Recency factor (time since cached)
        let hours_since_cached = (now - cached_file.cached_at).num_hours() as f64;
        let recency_score = hours_since_cached / 24.0;
        
        // Size factor (larger files get higher eviction scores)
        let size_score = (cached_file.metadata.file_size as f64) / (self.config.max_file_size_bytes as f64);
        
        // Priority factor
        let priority_score = match cached_file.cache_priority {
            CachePriority::Critical => 0.1,
            CachePriority::High => 0.3,
            CachePriority::Medium => 0.6,
            CachePriority::Low => 1.0,
        };
        
        // Weighted combination
        lru_score * policies.lru_weight +
        frequency_score * policies.frequency_weight +
        recency_score * policies.recency_weight +
        size_score * policies.size_weight +
        priority_score * 0.5 // Priority has its own weight
    }

    /// Calculate total cache size
    async fn calculate_cache_size(&self) -> u64 {
        let cache = self.file_cache.read().await;
        cache.iter()
            .map(|(_, cached_file)| cached_file.metadata.file_size)
            .sum()
    }

    /// Retrieve file from network using concurrent chunks if available
    async fn retrieve_from_network(&self, file_key: &str) -> Result<Vec<u8>> {
        if let Some(ref concurrent_chunks) = self.concurrent_chunks {
            // Use concurrent chunk manager for fast retrieval
            debug!("Using concurrent chunk retrieval for file: {}", file_key);
            // TODO: This would need a swarm parameter, for now it's a placeholder
            // concurrent_chunks.retrieve_file_parallel(file_key, swarm).await
        }
        
        // TODO: Integrate with actual file storage retrieval
        // This would call the file storage system's retrieve function
        // For now, return an error to indicate this needs integration
        Err(anyhow!("Network retrieval not yet integrated - file_key: {}", file_key))
    }

    /// Check if a file is cached
    pub async fn is_cached(&self, file_key: &str) -> bool {
        let cache = self.file_cache.read().await;
        cache.contains(file_key)
    }

    /// Preload popular files into cache
    pub async fn preload_popular_files(&self) -> Result<()> {
        let popular_files = {
            let patterns = self.access_patterns.lock().await;
            patterns.get_predicted_popular_files(50)
        };
        
        let mut preload_tasks = Vec::new();
        
        for file_key in popular_files {
            if !self.is_cached(&file_key).await {
                let cache_manager = self.clone();
                let key = file_key.clone();
                
                let task = tokio::spawn(async move {
                    match cache_manager.get_file_smart(&key).await {
                        Ok(_) => {
                            // Update preload statistics
                            let mut stats = cache_manager.stats.write().await;
                            stats.preloads += 1;
                            
                            // Record as preload access
                            let mut patterns = cache_manager.access_patterns.lock().await;
                            patterns.record_access(
                                key.clone(),
                                AccessType::Preload,
                                Duration::from_millis(0),
                                0,
                            );
                            
                            debug!("Preloaded popular file: {}", key);
                        }
                        Err(e) => {
                            warn!("Failed to preload popular file {}: {}", key, e);
                        }
                    }
                });
                
                preload_tasks.push(task);
            }
        }
        
        // Wait for a reasonable number of preloads to complete
        for task in preload_tasks.into_iter().take(10) {
            let _ = task.await;
        }
        
        Ok(())
    }

    /// Update cache hit ratio statistics
    fn update_hit_ratio(&self, stats: &mut CacheStats) {
        let total_requests = stats.file_cache_hits + stats.file_cache_misses;
        if total_requests > 0 {
            stats.hit_ratio = stats.file_cache_hits as f64 / total_requests as f64;
        }
    }

    /// Get current cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update current cache sizes
        stats.total_cached_files = {
            let cache = self.file_cache.read().await;
            cache.len()
        };
        
        stats.cache_size_bytes = self.calculate_cache_size().await;
        
        stats
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) -> Result<()> {
        {
            let mut file_cache = self.file_cache.write().await;
            file_cache.clear();
        }
        
        {
            let mut chunk_cache = self.chunk_cache.write().await;
            chunk_cache.clear();
        }
        
        // Reset statistics
        {
            let mut stats = self.stats.write().await;
            *stats = CacheStats::default();
        }
        
        info!("Cache cleared");
        Ok(())
    }
}

// Implement Clone for SmartCacheManager to enable sharing across tasks
impl Clone for SmartCacheManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            file_cache: self.file_cache.clone(),
            chunk_cache: self.chunk_cache.clone(),
            access_patterns: self.access_patterns.clone(),
            stats: self.stats.clone(),
            concurrent_chunks: self.concurrent_chunks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.file_cache_size_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(config.chunk_cache_size_bytes, 500 * 1024 * 1024);
        assert_eq!(config.max_file_size_bytes, 100 * 1024 * 1024);
        assert!(config.preload_popular);
        assert_eq!(config.ttl_hours, 24);
    }

    #[tokio::test]
    async fn test_smart_cache_manager_creation() {
        let config = CacheConfig::default();
        let cache_manager = SmartCacheManager::new(config);
        
        let stats = cache_manager.get_stats().await;
        assert_eq!(stats.total_cached_files, 0);
        assert_eq!(stats.cache_size_bytes, 0);
        assert_eq!(stats.hit_ratio, 0.0);
    }

    #[test]
    fn test_access_pattern_analyzer() {
        let mut analyzer = AccessPatternAnalyzer::new(1000);
        
        // Record some accesses
        analyzer.record_access("file1".to_string(), AccessType::NetworkFetch, Duration::from_millis(100), 1024);
        analyzer.record_access("file1".to_string(), AccessType::CacheHit, Duration::from_millis(10), 1024);
        analyzer.record_access("file2".to_string(), AccessType::NetworkFetch, Duration::from_millis(150), 2048);
        
        // Check frequency
        assert_eq!(analyzer.get_access_frequency("file1"), 2);
        assert_eq!(analyzer.get_access_frequency("file2"), 1);
        assert_eq!(analyzer.get_access_frequency("file3"), 0);
        
        // Check prediction
        let prediction1 = analyzer.predict_future_access("file1");
        let prediction2 = analyzer.predict_future_access("file2");
        let prediction3 = analyzer.predict_future_access("file3");
        
        assert!(prediction1 > prediction2);
        assert!(prediction2 > prediction3);
        assert_eq!(prediction3, 0.0);
    }

    #[test]
    fn test_lru_predictor() {
        let mut predictor = LRUPredictor::new(5);
        
        predictor.record_access("file1".to_string());
        predictor.record_access("file2".to_string());
        predictor.record_access("file3".to_string());
        predictor.record_access("file1".to_string()); // Access file1 again
        
        let score1 = predictor.predict_access("file1");
        let score2 = predictor.predict_access("file2");
        let score3 = predictor.predict_access("file3");
        let score4 = predictor.predict_access("file4");
        
        assert!(score1 > score2); // file1 was accessed more recently
        assert!(score2 > score3); // file2 was accessed more recently than file3
        assert_eq!(score4, 0.0);  // file4 was never accessed
    }

    #[test]
    fn test_cache_policies() {
        let policies = CachePolicies::default();
        assert_eq!(policies.lru_weight, 0.4);
        assert_eq!(policies.frequency_weight, 0.3);
        assert_eq!(policies.recency_weight, 0.2);
        assert_eq!(policies.size_weight, 0.1);
        
        // Weights should sum to 1.0
        let total = policies.lru_weight + policies.frequency_weight + 
                   policies.recency_weight + policies.size_weight;
        assert!((total - 1.0).abs() < 0.001);
    }
}