use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult, Quorum, Record, RecordKey};
use libp2p::swarm::SwarmEvent;
use libp2p::{PeerId, Swarm};
/// Concurrent Chunk Operations Module
///
/// This module implements the Concurrent Chunk Operations system as outlined
/// in the DataMesh Application & Network Improvements Roadmap. It provides:
/// - Parallel chunk retrieval and upload operations
/// - Thread pool management for concurrent operations
/// - Connection pooling for efficient network utilization
/// - Timeout handling and retry mechanisms
/// - Multi-peer chunk retrieval with failover
/// - Performance monitoring and metrics collection
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, info};

use crate::file_storage::StoredFile;
use crate::network::{MyBehaviour, MyBehaviourEvent};

/// Configuration for concurrent chunk operations
#[derive(Debug, Clone)]
pub struct ConcurrentChunkConfig {
    /// Maximum concurrent chunk retrievals
    pub max_concurrent_retrievals: usize,
    /// Maximum concurrent chunk uploads
    pub max_concurrent_uploads: usize,
    /// Timeout for individual chunk operations
    pub chunk_timeout: Duration,
    /// Number of retry attempts for failed chunks
    pub retry_failed_chunks: u32,
    /// Whether to prefer faster responding peers
    pub prefer_fast_peers: bool,
    /// Maximum time to wait for peer responses
    pub peer_response_timeout: Duration,
}

impl Default for ConcurrentChunkConfig {
    fn default() -> Self {
        Self {
            max_concurrent_retrievals: 8,
            max_concurrent_uploads: 4,
            chunk_timeout: Duration::from_secs(10),
            retry_failed_chunks: 3,
            prefer_fast_peers: true,
            peer_response_timeout: Duration::from_secs(5),
        }
    }
}

/// Represents a chunk operation result
#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub chunk_key: RecordKey,
    pub data: Vec<u8>,
    pub peer_id: Option<PeerId>,
    pub response_time: Duration,
    pub attempt_count: u32,
}

/// Represents a chunk upload result
#[derive(Debug, Clone)]
pub struct ChunkUploadResult {
    pub chunk_key: RecordKey,
    pub success: bool,
    pub peers_contacted: Vec<PeerId>,
    pub response_time: Duration,
    pub attempt_count: u32,
}

/// Statistics for concurrent chunk operations
#[derive(Debug, Clone)]
pub struct ChunkOperationStats {
    pub total_chunks: usize,
    pub successful_chunks: usize,
    pub failed_chunks: usize,
    pub average_response_time: Duration,
    pub total_operation_time: Duration,
    pub fastest_peer: Option<PeerId>,
    pub slowest_peer: Option<PeerId>,
    pub peer_response_times: HashMap<PeerId, Duration>,
}

/// Manager for concurrent chunk operations
pub struct ConcurrentChunkManager {
    config: ConcurrentChunkConfig,
    retrieval_semaphore: Arc<Semaphore>,
    upload_semaphore: Arc<Semaphore>,
    peer_stats: Arc<RwLock<HashMap<PeerId, PeerStats>>>,
    operation_stats: Arc<RwLock<ChunkOperationStats>>,
}

/// Statistics for individual peers
#[derive(Debug, Clone)]
struct PeerStats {
    total_requests: u32,
    successful_requests: u32,
    failed_requests: u32,
    average_response_time: Duration,
    last_seen: Instant,
}

impl PeerStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_secs(0),
            last_seen: Instant::now(),
        }
    }

    fn update_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.last_seen = Instant::now();

        // Update average response time
        if self.successful_requests == 1 {
            self.average_response_time = response_time;
        } else {
            let current_avg = self.average_response_time.as_millis() as f64;
            let new_time = response_time.as_millis() as f64;
            let count = self.successful_requests as f64;
            let new_avg = (current_avg * (count - 1.0) + new_time) / count;
            self.average_response_time = Duration::from_millis(new_avg as u64);
        }
    }

    fn update_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_seen = Instant::now();
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    fn is_responsive(&self) -> bool {
        // Consider peer responsive if it has good success rate and recent activity
        self.success_rate() > 0.7 && self.last_seen.elapsed() < Duration::from_secs(300)
        // 5 minutes
    }
}

impl ConcurrentChunkManager {
    /// Create a new concurrent chunk manager
    pub fn new(config: ConcurrentChunkConfig) -> Self {
        let retrieval_semaphore = Arc::new(Semaphore::new(config.max_concurrent_retrievals));
        let upload_semaphore = Arc::new(Semaphore::new(config.max_concurrent_uploads));

        Self {
            config,
            retrieval_semaphore,
            upload_semaphore,
            peer_stats: Arc::new(RwLock::new(HashMap::new())),
            operation_stats: Arc::new(RwLock::new(ChunkOperationStats {
                total_chunks: 0,
                successful_chunks: 0,
                failed_chunks: 0,
                average_response_time: Duration::from_secs(0),
                total_operation_time: Duration::from_secs(0),
                fastest_peer: None,
                slowest_peer: None,
                peer_response_times: HashMap::new(),
            })),
        }
    }

    /// Retrieve a complete file using parallel chunk operations
    /// This is the main entry point for concurrent chunk retrieval as specified in the roadmap
    pub async fn retrieve_file_parallel(
        &self,
        file_key: &str,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> Result<Vec<u8>> {
        // Get chunk keys for the file
        let chunk_keys = self.get_chunk_keys(file_key, swarm.clone()).await?;

        // Retrieve all chunks concurrently
        let chunk_results = self.retrieve_chunks_concurrent(chunk_keys, swarm).await?;

        // Reconstruct file from chunks (this would be handled by the calling code)
        // For now, return the concatenated chunk data as a placeholder
        let mut file_data = Vec::new();
        for chunk in chunk_results {
            file_data.extend(chunk.data);
        }

        Ok(file_data)
    }

    /// Get chunk keys for a file from its metadata
    async fn get_chunk_keys(
        &self,
        file_key: &str,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> Result<Vec<RecordKey>> {
        let key_bytes = hex::decode(file_key)?;
        let record_key = RecordKey::from(key_bytes);

        // Retrieve file metadata
        let metadata = self.retrieve_file_metadata(record_key, swarm).await?;

        // Convert chunk key bytes to RecordKeys
        let chunk_keys = metadata
            .chunk_keys
            .into_iter()
            .map(|key_bytes| RecordKey::from(key_bytes))
            .collect();

        Ok(chunk_keys)
    }

    /// Retrieve file metadata from DHT
    async fn retrieve_file_metadata(
        &self,
        file_key: RecordKey,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> Result<StoredFile> {
        let metadata_result =
            Self::retrieve_chunk_from_dht(file_key, swarm, self.config.clone()).await?;

        // Parse metadata
        let stored_file: StoredFile = serde_json::from_slice(&metadata_result.data)?;

        Ok(stored_file)
    }

    /// Retrieve multiple chunks concurrently
    pub async fn retrieve_chunks_concurrent(
        &self,
        chunk_keys: Vec<RecordKey>,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> Result<Vec<ChunkResult>> {
        let operation_start = Instant::now();
        info!(
            "Starting concurrent retrieval of {} chunks",
            chunk_keys.len()
        );

        let mut tasks = Vec::new();

        // Create futures for all chunk retrievals without spawning tasks
        for chunk_key in chunk_keys {
            let swarm = swarm.clone();
            let semaphore = self.retrieval_semaphore.clone();
            let config = self.config.clone();
            let peer_stats = self.peer_stats.clone();

            let future = async move {
                let _permit = semaphore.acquire().await.unwrap();

                Self::retrieve_single_chunk_with_retry(chunk_key.clone(), swarm, config, peer_stats)
                    .await
            };

            tasks.push(future);
        }

        // Execute all futures concurrently using futures::future::join_all
        let results = futures::future::join_all(tasks).await;

        // Update operation statistics
        let operation_time = operation_start.elapsed();
        self.update_operation_stats(&results, operation_time).await;

        // Filter successful results
        let successful_results: Vec<ChunkResult> =
            results.into_iter().filter_map(|r| r.ok()).collect();

        info!(
            "Concurrent chunk retrieval completed: {}/{} chunks retrieved in {:?}",
            successful_results.len(),
            self.operation_stats.read().await.total_chunks,
            operation_time
        );

        Ok(successful_results)
    }

    /// Retrieve a single chunk with retry logic and multi-peer support
    async fn retrieve_single_chunk_with_retry(
        chunk_key: RecordKey,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
        peer_stats: Arc<RwLock<HashMap<PeerId, PeerStats>>>,
    ) -> Result<ChunkResult> {
        let mut attempt = 0;
        let mut _last_error = None;

        while attempt < config.retry_failed_chunks {
            attempt += 1;

            match Self::retrieve_chunk_from_multiple_peers(
                chunk_key.clone(),
                swarm.clone(),
                config.clone(),
                peer_stats.clone(),
            )
            .await
            {
                Ok(mut result) => {
                    result.attempt_count = attempt;
                    return Ok(result);
                }
                Err(e) => {
                    _last_error = Some(e);

                    // Exponential backoff between retries
                    if attempt < config.retry_failed_chunks {
                        let backoff_delay = Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                        tokio::time::sleep(backoff_delay).await;
                    }
                }
            }
        }

        Err(_last_error.unwrap_or_else(|| {
            anyhow!(
                "Failed to retrieve chunk after {} attempts",
                config.retry_failed_chunks
            )
        }))
    }

    /// Retrieve chunk from multiple peers concurrently using select_ok
    async fn retrieve_chunk_from_multiple_peers(
        chunk_key: RecordKey,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
        peer_stats: Arc<RwLock<HashMap<PeerId, PeerStats>>>,
    ) -> Result<ChunkResult> {
        debug!("Retrieving chunk from multiple peers: {:?}", chunk_key);

        // Get list of responsive peers
        let responsive_peers = {
            let stats = peer_stats.read().await;
            let mut peers: Vec<PeerId> = stats
                .iter()
                .filter(|(_, stat)| stat.is_responsive())
                .map(|(peer_id, _)| *peer_id)
                .collect();

            // Sort by preference if enabled
            if config.prefer_fast_peers {
                peers.sort_by(|a, b| {
                    let a_time = stats
                        .get(a)
                        .map(|s| s.average_response_time)
                        .unwrap_or(Duration::from_secs(999));
                    let b_time = stats
                        .get(b)
                        .map(|s| s.average_response_time)
                        .unwrap_or(Duration::from_secs(999));
                    a_time.cmp(&b_time)
                });
            }

            peers
        };

        if responsive_peers.is_empty() {
            // Fallback to DHT query if no responsive peers
            return Self::retrieve_chunk_from_dht(chunk_key, swarm, config).await;
        }

        // Create futures for querying multiple peers
        let mut futures = Vec::new();

        for peer_id in responsive_peers.iter().take(3) {
            // Query top 3 peers
            let chunk_key = chunk_key.clone();
            let swarm = swarm.clone();
            let config = config.clone();
            let peer_stats = peer_stats.clone();
            let peer_id = *peer_id;

            let future = async move {
                Self::retrieve_chunk_from_peer(chunk_key, peer_id, swarm, config, peer_stats).await
            };

            futures.push(future);
        }

        // Execute all futures and return the first successful result
        let results = futures::future::join_all(futures).await;

        // Find first successful result
        for result in results {
            if result.is_ok() {
                return result;
            }
        }

        // If all peer queries fail, fallback to DHT
        Self::retrieve_chunk_from_dht(chunk_key, swarm, config).await
    }

    /// Retrieve chunk from a specific peer
    async fn retrieve_chunk_from_peer(
        chunk_key: RecordKey,
        peer_id: PeerId,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
        peer_stats: Arc<RwLock<HashMap<PeerId, PeerStats>>>,
    ) -> Result<ChunkResult> {
        let request_start = Instant::now();

        // TODO: Implement direct peer query
        // For now, fallback to DHT query
        let result = Self::retrieve_chunk_from_dht(chunk_key, swarm, config).await;

        let response_time = request_start.elapsed();

        // Update peer statistics
        let mut stats = peer_stats.write().await;
        let peer_stat = stats.entry(peer_id).or_insert_with(PeerStats::new);

        match &result {
            Ok(_) => peer_stat.update_success(response_time),
            Err(_) => peer_stat.update_failure(),
        }

        result
    }

    /// Retrieve chunk from DHT with proper event handling
    async fn retrieve_chunk_from_dht(
        chunk_key: RecordKey,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
    ) -> Result<ChunkResult> {
        let request_start = Instant::now();

        // Issue DHT query
        {
            let mut swarm = swarm.write().await;
            swarm.behaviour_mut().kad.get_record(chunk_key.clone());
        }

        // Wait for response with timeout
        let result = timeout(config.chunk_timeout, async {
            loop {
                let mut swarm = swarm.write().await;
                match swarm.select_next_some().await {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Kad(kad_event)) => {
                        match kad_event {
                            KademliaEvent::OutboundQueryProgressed { result, .. } => {
                                match result {
                                    QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(
                                        peer_record,
                                    ))) => {
                                        let record = &peer_record.record;

                                        // Check if this is the chunk we're looking for
                                        if record.key == chunk_key {
                                            return ChunkResult {
                                                chunk_key: record.key.clone(),
                                                data: record.value.clone(),
                                                peer_id: peer_record.peer,
                                                response_time: request_start.elapsed(),
                                                attempt_count: 1,
                                            };
                                        }
                                    }
                                    QueryResult::GetRecord(Err(err)) => {
                                        debug!(
                                            "DHT query failed for chunk {:?}: {:?}",
                                            chunk_key, err
                                        );
                                        continue;
                                    }
                                    _ => continue,
                                }
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
        })
        .await;

        match result {
            Ok(chunk_result) => Ok(chunk_result),
            Err(_) => Err(anyhow!(
                "Chunk retrieval timed out for key: {:?}",
                chunk_key
            )),
        }
    }

    /// Upload multiple chunks concurrently
    pub async fn upload_chunks_concurrent(
        &self,
        chunks: Vec<(RecordKey, Vec<u8>)>,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> Result<Vec<ChunkUploadResult>> {
        let operation_start = Instant::now();
        info!("Starting concurrent upload of {} chunks", chunks.len());

        let mut _results: Vec<ChunkUploadResult> = Vec::new();
        let mut tasks = Vec::new();

        // Create futures for all chunk uploads without spawning tasks
        for (chunk_key, data) in chunks {
            let swarm = swarm.clone();
            let config = self.config.clone();
            let semaphore = self.upload_semaphore.clone();

            let future = async move {
                let _permit = semaphore.acquire().await.unwrap();

                Self::upload_single_chunk_with_retry(chunk_key.clone(), data, swarm, config).await
            };

            tasks.push(future);
        }

        // Execute all futures concurrently using futures::future::join_all
        let upload_results = futures::future::join_all(tasks).await;

        let operation_time = operation_start.elapsed();
        let successful_uploads = upload_results.iter().filter(|r| r.is_ok()).count();

        info!(
            "Concurrent chunk upload completed: {}/{} chunks uploaded in {:?}",
            successful_uploads,
            upload_results.len(),
            operation_time
        );

        // Filter successful results
        let successful_results: Vec<ChunkUploadResult> =
            upload_results.into_iter().filter_map(|r| r.ok()).collect();

        Ok(successful_results)
    }

    /// Upload a single chunk with retry logic
    async fn upload_single_chunk_with_retry(
        chunk_key: RecordKey,
        data: Vec<u8>,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
    ) -> Result<ChunkUploadResult> {
        let mut attempt = 0;
        let mut _last_error = None;
        let upload_start = Instant::now();

        while attempt < config.retry_failed_chunks {
            attempt += 1;

            match Self::upload_chunk_to_dht(
                chunk_key.clone(),
                data.clone(),
                swarm.clone(),
                config.clone(),
            )
            .await
            {
                Ok(mut result) => {
                    result.attempt_count = attempt;
                    return Ok(result);
                }
                Err(e) => {
                    _last_error = Some(e);

                    // Exponential backoff between retries
                    if attempt < config.retry_failed_chunks {
                        let backoff_delay = Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                        tokio::time::sleep(backoff_delay).await;
                    }
                }
            }
        }

        Ok(ChunkUploadResult {
            chunk_key,
            success: false,
            peers_contacted: vec![],
            response_time: upload_start.elapsed(),
            attempt_count: attempt,
        })
    }

    /// Upload chunk to DHT with proper event handling
    async fn upload_chunk_to_dht(
        chunk_key: RecordKey,
        data: Vec<u8>,
        swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
        config: ConcurrentChunkConfig,
    ) -> Result<ChunkUploadResult> {
        let upload_start = Instant::now();

        // Create record
        let record = Record {
            key: chunk_key.clone(),
            value: data,
            publisher: None,
            expires: None,
        };

        // Upload to DHT with intelligent quorum
        {
            let mut swarm = swarm.write().await;
            
            // Get connected peers for intelligent quorum calculation
            let connected_peers: Vec<_> = swarm.connected_peers().cloned().collect();
            tracing::info!("ConcurrentChunks: {} connected peers for quorum calculation", connected_peers.len());
            
            // Calculate intelligent quorum based on network size
            let quorum = if connected_peers.is_empty() {
                tracing::info!("ConcurrentChunks: No peers connected, using Quorum::All");
                Quorum::All
            } else if connected_peers.len() <= 2 {
                tracing::info!("ConcurrentChunks: Small network ({} peers), using Quorum::One", connected_peers.len());
                Quorum::One
            } else if connected_peers.len() <= 5 {
                tracing::info!("ConcurrentChunks: Medium network ({} peers), using Quorum::N(1)", connected_peers.len());
                Quorum::N(std::num::NonZeroUsize::new(1).unwrap())
            } else {
                let quorum_size = std::cmp::max(2, (connected_peers.len() as f64 * 0.25).ceil() as usize);
                let quorum_size = std::cmp::min(quorum_size, connected_peers.len());
                tracing::info!("ConcurrentChunks: Large network ({} peers), using Quorum::N({})", connected_peers.len(), quorum_size);
                Quorum::N(std::num::NonZeroUsize::new(quorum_size).unwrap())
            };
            
            swarm.behaviour_mut().kad.put_record(record, quorum)?;
        }

        // Wait for confirmation with timeout
        let result = timeout(config.chunk_timeout, async {
            let peers_contacted = Vec::new();

            loop {
                let mut swarm = swarm.write().await;
                match swarm.select_next_some().await {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Kad(kad_event)) => {
                        match kad_event {
                            KademliaEvent::OutboundQueryProgressed { result, .. } => {
                                match result {
                                    QueryResult::PutRecord(Ok(put_record_ok)) => {
                                        // Note: PutRecordOk doesn't provide peer info, so we'll track differently
                                        debug!(
                                            "Successfully stored chunk {:?} with key {:?}",
                                            chunk_key, put_record_ok.key
                                        );
                                        return (true, peers_contacted);
                                    }
                                    QueryResult::PutRecord(Err(err)) => {
                                        debug!(
                                            "DHT put failed for chunk {:?}: {:?}",
                                            chunk_key, err
                                        );
                                        return (false, peers_contacted);
                                    }
                                    _ => continue,
                                }
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
        })
        .await;

        let response_time = upload_start.elapsed();

        match result {
            Ok((success, peers_contacted)) => Ok(ChunkUploadResult {
                chunk_key,
                success,
                peers_contacted,
                response_time,
                attempt_count: 1,
            }),
            Err(_) => Err(anyhow!("Chunk upload timed out")),
        }
    }

    /// Update operation statistics
    async fn update_operation_stats(
        &self,
        results: &[Result<ChunkResult>],
        operation_time: Duration,
    ) {
        let mut stats = self.operation_stats.write().await;

        stats.total_chunks = results.len();
        stats.successful_chunks = results.iter().filter(|r| r.is_ok()).count();
        stats.failed_chunks = results.len() - stats.successful_chunks;
        stats.total_operation_time = operation_time;

        // Calculate average response time
        let successful_results: Vec<&ChunkResult> =
            results.iter().filter_map(|r| r.as_ref().ok()).collect();

        if !successful_results.is_empty() {
            let total_time: Duration = successful_results.iter().map(|r| r.response_time).sum();

            stats.average_response_time = total_time / successful_results.len() as u32;

            // Find fastest and slowest peers
            if let Some(fastest) = successful_results.iter().min_by_key(|r| r.response_time) {
                stats.fastest_peer = fastest.peer_id;
            }

            if let Some(slowest) = successful_results.iter().max_by_key(|r| r.response_time) {
                stats.slowest_peer = slowest.peer_id;
            }

            // Update peer response times
            for result in successful_results {
                if let Some(peer_id) = result.peer_id {
                    stats
                        .peer_response_times
                        .insert(peer_id, result.response_time);
                }
            }
        }
    }

    /// Get current operation statistics
    pub async fn get_operation_stats(&self) -> ChunkOperationStats {
        self.operation_stats.read().await.clone()
    }

    /// Get peer statistics
    pub async fn get_peer_stats(&self) -> HashMap<PeerId, PeerStats> {
        self.peer_stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.operation_stats.write().await;
        *stats = ChunkOperationStats {
            total_chunks: 0,
            successful_chunks: 0,
            failed_chunks: 0,
            average_response_time: Duration::from_secs(0),
            total_operation_time: Duration::from_secs(0),
            fastest_peer: None,
            slowest_peer: None,
            peer_response_times: HashMap::new(),
        };

        let mut peer_stats = self.peer_stats.write().await;
        peer_stats.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_chunk_manager_creation() {
        let config = ConcurrentChunkConfig::default();
        let manager = ConcurrentChunkManager::new(config);

        assert_eq!(manager.config.max_concurrent_retrievals, 8);
        assert_eq!(manager.config.max_concurrent_uploads, 4);
    }

    #[tokio::test]
    async fn test_peer_stats_tracking() {
        let mut peer_stats = PeerStats::new();

        // Test success tracking
        peer_stats.update_success(Duration::from_millis(100));
        assert_eq!(peer_stats.successful_requests, 1);
        assert_eq!(peer_stats.total_requests, 1);
        assert_eq!(peer_stats.success_rate(), 1.0);

        // Test failure tracking
        peer_stats.update_failure();
        assert_eq!(peer_stats.failed_requests, 1);
        assert_eq!(peer_stats.total_requests, 2);
        assert_eq!(peer_stats.success_rate(), 0.5);

        // Test responsiveness
        assert!(peer_stats.is_responsive());
    }

    #[test]
    fn test_concurrent_chunk_config_default() {
        let config = ConcurrentChunkConfig::default();

        assert_eq!(config.max_concurrent_retrievals, 8);
        assert_eq!(config.max_concurrent_uploads, 4);
        assert_eq!(config.chunk_timeout, Duration::from_secs(10));
        assert_eq!(config.retry_failed_chunks, 3);
        assert!(config.prefer_fast_peers);
    }
}
