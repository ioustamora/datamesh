use anyhow::Result;
use libp2p::kad::Quorum;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumConfig {
    /// Minimum quorum size regardless of network size
    pub min_quorum: usize,
    /// Maximum quorum size to prevent excessive overhead
    pub max_quorum: usize,
    /// Percentage of connected peers to use for quorum (0.0 to 1.0)
    pub quorum_percentage: f64,
    /// Minimum number of peers required before using percentage-based quorum
    pub min_peers_for_percentage: usize,
    /// Enable adaptive quorum based on network reliability
    pub adaptive_quorum: bool,
    /// Base reliability threshold for adaptive quorum
    pub reliability_threshold: f64,
}

impl Default for QuorumConfig {
    fn default() -> Self {
        Self {
            min_quorum: 1,
            max_quorum: 10,
            quorum_percentage: 0.5, // 50% of peers
            min_peers_for_percentage: 5,
            adaptive_quorum: true,
            reliability_threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerReliability {
    pub success_count: u64,
    pub failure_count: u64,
    pub last_success: Option<u64>,
    pub last_failure: Option<u64>,
    pub response_times: Vec<Duration>,
}

impl PeerReliability {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            response_times: Vec::new(),
        }
    }

    pub fn get_reliability_score(&self) -> f64 {
        let total_ops = self.success_count + self.failure_count;
        if total_ops == 0 {
            return 0.5; // Neutral score for new peers
        }

        let base_score = self.success_count as f64 / total_ops as f64;
        
        // Apply time-based decay for recent failures
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut time_factor = 1.0;
        if let Some(last_failure) = self.last_failure {
            let time_since_failure = current_time.saturating_sub(last_failure);
            if time_since_failure < 300 { // 5 minutes
                time_factor = 0.5; // Heavily penalize recent failures
            } else if time_since_failure < 3600 { // 1 hour
                time_factor = 0.8; // Moderately penalize
            }
        }

        // Factor in response times
        let response_factor = if !self.response_times.is_empty() {
            let avg_response = self.response_times.iter().sum::<Duration>().as_millis() as f64 
                / self.response_times.len() as f64;
            if avg_response < 1000.0 { // < 1 second
                1.2
            } else if avg_response < 5000.0 { // < 5 seconds
                1.0
            } else {
                0.8
            }
        } else {
            1.0
        };

        (base_score * time_factor * response_factor).min(1.0)
    }

    pub fn record_success(&mut self, response_time: Duration) {
        self.success_count += 1;
        self.last_success = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
        
        // Keep only last 10 response times for efficiency
        self.response_times.push(response_time);
        if self.response_times.len() > 10 {
            self.response_times.remove(0);
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
    }
}

#[derive(Clone)]
pub struct QuorumManager {
    config: QuorumConfig,
    peer_reliability: Arc<RwLock<HashMap<PeerId, PeerReliability>>>,
}

impl QuorumManager {
    pub fn new(config: QuorumConfig) -> Self {
        Self {
            config,
            peer_reliability: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate optimal quorum based on current network conditions
    pub async fn calculate_quorum(&self, connected_peers: &[PeerId]) -> Result<Quorum> {
        let peer_count = connected_peers.len();
        
        // For very small networks, use minimum quorum
        if peer_count < self.config.min_peers_for_percentage {
            let quorum_size = self.config.min_quorum.min(peer_count);
            if quorum_size == 0 {
                return Ok(Quorum::All); // Fallback for empty network
            }
            return Ok(Quorum::N(NonZeroUsize::new(quorum_size).unwrap()));
        }

        let base_quorum = if self.config.adaptive_quorum {
            self.calculate_adaptive_quorum(connected_peers).await?
        } else {
            self.calculate_percentage_quorum(peer_count)
        };

        // Ensure quorum is within configured bounds
        let final_quorum = base_quorum
            .max(self.config.min_quorum)
            .min(self.config.max_quorum)
            .min(peer_count);

        if final_quorum == 0 {
            Ok(Quorum::All)
        } else {
            Ok(Quorum::N(NonZeroUsize::new(final_quorum).unwrap()))
        }
    }

    /// Calculate quorum based on percentage of network size
    fn calculate_percentage_quorum(&self, peer_count: usize) -> usize {
        ((peer_count as f64 * self.config.quorum_percentage).ceil() as usize).max(1)
    }

    /// Calculate adaptive quorum based on peer reliability
    async fn calculate_adaptive_quorum(&self, connected_peers: &[PeerId]) -> Result<usize> {
        let reliability_map = self.peer_reliability.read().await;
        
        let mut reliable_peers = 0;
        let mut total_reliability = 0.0;
        
        for peer_id in connected_peers {
            if let Some(reliability) = reliability_map.get(peer_id) {
                let score = reliability.get_reliability_score();
                total_reliability += score;
                if score >= self.config.reliability_threshold {
                    reliable_peers += 1;
                }
            } else {
                // New peer, assume neutral reliability
                total_reliability += 0.5;
            }
        }

        let avg_reliability = if connected_peers.is_empty() {
            0.5
        } else {
            total_reliability / connected_peers.len() as f64
        };

        // If network is highly reliable, use smaller quorum
        // If network is less reliable, use larger quorum for safety
        let reliability_factor = if avg_reliability >= self.config.reliability_threshold {
            0.3 // Use 30% of peers for reliable networks
        } else if avg_reliability >= 0.6 {
            0.5 // Use 50% of peers for moderately reliable networks
        } else {
            0.7 // Use 70% of peers for unreliable networks
        };

        let adaptive_quorum = (connected_peers.len() as f64 * reliability_factor).ceil() as usize;
        Ok(adaptive_quorum.max(1))
    }

    /// Record successful operation for a peer
    pub async fn record_peer_success(&self, peer_id: PeerId, response_time: Duration) {
        let mut reliability_map = self.peer_reliability.write().await;
        let reliability = reliability_map.entry(peer_id).or_insert_with(PeerReliability::new);
        reliability.record_success(response_time);
    }

    /// Record failed operation for a peer
    pub async fn record_peer_failure(&self, peer_id: PeerId) {
        let mut reliability_map = self.peer_reliability.write().await;
        let reliability = reliability_map.entry(peer_id).or_insert_with(PeerReliability::new);
        reliability.record_failure();
    }

    /// Get reliability statistics for all peers
    pub async fn get_reliability_stats(&self) -> HashMap<PeerId, f64> {
        let reliability_map = self.peer_reliability.read().await;
        reliability_map
            .iter()
            .map(|(peer_id, reliability)| (*peer_id, reliability.get_reliability_score()))
            .collect()
    }

    /// Get current quorum configuration
    pub fn get_config(&self) -> &QuorumConfig {
        &self.config
    }

    /// Update quorum configuration
    pub fn update_config(&mut self, config: QuorumConfig) {
        self.config = config;
    }

    /// Clean up old peer reliability data
    pub async fn cleanup_stale_peers(&self, max_age_hours: u64) {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (max_age_hours * 3600);

        let mut reliability_map = self.peer_reliability.write().await;
        reliability_map.retain(|_, reliability| {
            // Keep peer if they had recent activity
            reliability.last_success.unwrap_or(0) > cutoff_time 
                || reliability.last_failure.unwrap_or(0) > cutoff_time
        });
    }

    /// Get network health summary
    pub async fn get_network_health(&self, connected_peers: &[PeerId]) -> NetworkHealth {
        let reliability_map = self.peer_reliability.read().await;
        
        let mut total_reliability = 0.0;
        let mut reliable_peers = 0;
        let mut fast_peers = 0;
        
        for peer_id in connected_peers {
            if let Some(reliability) = reliability_map.get(peer_id) {
                let score = reliability.get_reliability_score();
                total_reliability += score;
                
                if score >= self.config.reliability_threshold {
                    reliable_peers += 1;
                }
                
                if !reliability.response_times.is_empty() {
                    let avg_response = reliability.response_times.iter().sum::<Duration>().as_millis() as f64 
                        / reliability.response_times.len() as f64;
                    if avg_response < 2000.0 { // < 2 seconds
                        fast_peers += 1;
                    }
                }
            } else {
                total_reliability += 0.5; // Neutral for new peers
            }
        }

        let avg_reliability = if connected_peers.is_empty() {
            0.0
        } else {
            total_reliability / connected_peers.len() as f64
        };

        NetworkHealth {
            total_peers: connected_peers.len(),
            reliable_peers,
            fast_peers,
            average_reliability: avg_reliability,
            recommended_quorum: self.calculate_percentage_quorum(connected_peers.len()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkHealth {
    pub total_peers: usize,
    pub reliable_peers: usize,
    pub fast_peers: usize,
    pub average_reliability: f64,
    pub recommended_quorum: usize,
}

impl NetworkHealth {
    pub fn health_status(&self) -> &'static str {
        if self.average_reliability >= 0.8 && self.reliable_peers >= 3 {
            "Excellent"
        } else if self.average_reliability >= 0.6 && self.reliable_peers >= 2 {
            "Good"
        } else if self.average_reliability >= 0.4 && self.total_peers >= 2 {
            "Fair"
        } else {
            "Poor"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[tokio::test]
    async fn test_quorum_calculation() {
        let config = QuorumConfig::default();
        let manager = QuorumManager::new(config);
        
        // Test small network
        let small_peers = vec![PeerId::random(), PeerId::random()];
        let quorum = manager.calculate_quorum(&small_peers).await.unwrap();
        assert!(matches!(quorum, Quorum::N(_)));

        // Test larger network
        let large_peers: Vec<PeerId> = (0..10).map(|_| PeerId::random()).collect();
        let quorum = manager.calculate_quorum(&large_peers).await.unwrap();
        assert!(matches!(quorum, Quorum::N(_)));
    }

    #[tokio::test]
    async fn test_peer_reliability() {
        let mut reliability = PeerReliability::new();
        
        // Initial score should be neutral
        assert_eq!(reliability.get_reliability_score(), 0.5);
        
        // Record successes
        reliability.record_success(Duration::from_millis(500));
        reliability.record_success(Duration::from_millis(600));
        assert!(reliability.get_reliability_score() > 0.5);
        
        // Record failure
        reliability.record_failure();
        assert!(reliability.get_reliability_score() < 1.0);
    }

    #[tokio::test]
    async fn test_adaptive_quorum() {
        let config = QuorumConfig {
            adaptive_quorum: true,
            min_quorum: 1,
            max_quorum: 5,
            quorum_percentage: 0.5,
            min_peers_for_percentage: 3,
            reliability_threshold: 0.8,
        };
        
        let manager = QuorumManager::new(config);
        let peers: Vec<PeerId> = (0..6).map(|_| PeerId::random()).collect();
        
        // Record good performance for some peers
        for peer in &peers[0..3] {
            manager.record_peer_success(*peer, Duration::from_millis(500)).await;
            manager.record_peer_success(*peer, Duration::from_millis(400)).await;
        }
        
        // Record poor performance for others
        for peer in &peers[3..6] {
            manager.record_peer_failure(*peer).await;
            manager.record_peer_failure(*peer).await;
        }
        
        let quorum = manager.calculate_quorum(&peers).await.unwrap();
        assert!(matches!(quorum, Quorum::N(_)));
        
        let health = manager.get_network_health(&peers).await;
        assert_eq!(health.total_peers, 6);
        assert!(health.reliable_peers <= 3);
    }
}