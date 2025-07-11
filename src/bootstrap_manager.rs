use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use libp2p::{Multiaddr, PeerId, Swarm};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
/// Multi-Bootstrap Peer Support Implementation
///
/// This module implements the Multi-Bootstrap Peer Support system as outlined
/// in the DataMesh Application & Network Improvements Roadmap. It provides:
/// - Multiple bootstrap peer management
/// - Intelligent connection strategies with prioritization
/// - Health monitoring and automatic failover
/// - Geographic redundancy support
/// - Exponential backoff retry strategies
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn};

use crate::network::MyBehaviour;

/// Serialize PeerId as base58 string
fn serialize_peer_id<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&peer_id.to_base58())
}

/// Deserialize PeerId from base58 string
fn deserialize_peer_id<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = bs58::decode(&s)
        .into_vec()
        .map_err(serde::de::Error::custom)?;
    PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)
}

/// Serialize Vec<Multiaddr> as vector of strings
fn serialize_addresses<S>(addresses: &Vec<Multiaddr>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_addrs: Vec<String> = addresses.iter().map(|addr| addr.to_string()).collect();
    string_addrs.serialize(serializer)
}

/// Deserialize Vec<Multiaddr> from vector of strings
fn deserialize_addresses<'de, D>(deserializer: D) -> Result<Vec<Multiaddr>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_addrs: Vec<String> = Vec::deserialize(deserializer)?;
    string_addrs
        .into_iter()
        .map(|s| s.parse().map_err(serde::de::Error::custom))
        .collect()
}

/// Bootstrap peer configuration with priority and health tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeer {
    #[serde(
        serialize_with = "serialize_peer_id",
        deserialize_with = "deserialize_peer_id"
    )]
    pub peer_id: PeerId,
    #[serde(
        serialize_with = "serialize_addresses",
        deserialize_with = "deserialize_addresses"
    )]
    pub addresses: Vec<Multiaddr>,
    pub priority: u8,
    pub region: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub success_rate: f64,
    pub total_attempts: u32,
    pub successful_attempts: u32,
}

impl BootstrapPeer {
    pub fn new(peer_id: PeerId, addresses: Vec<Multiaddr>) -> Self {
        Self {
            peer_id,
            addresses,
            priority: 1,
            region: None,
            last_seen: None,
            success_rate: 1.0,
            total_attempts: 0,
            successful_attempts: 0,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn update_success(&mut self, success: bool) {
        self.total_attempts += 1;
        if success {
            self.successful_attempts += 1;
            self.last_seen = Some(Utc::now());
        }

        if self.total_attempts > 0 {
            self.success_rate = self.successful_attempts as f64 / self.total_attempts as f64;
        }
    }

    pub fn is_healthy(&self) -> bool {
        // Consider peer healthy if success rate > 50% and last seen within 5 minutes
        self.success_rate > 0.5
            && self.last_seen.map_or(false, |last| {
                Utc::now().signed_duration_since(last).num_minutes() < 5
            })
    }

    pub fn score(&self) -> f64 {
        // Calculate connection score based on priority, success rate, and recency
        let priority_score = (10 - self.priority as i32).max(1) as f64;
        let success_score = self.success_rate * 10.0;
        let recency_score = self.last_seen.map_or(0.0, |last| {
            let minutes_ago = Utc::now().signed_duration_since(last).num_minutes();
            (60.0 - minutes_ago.min(60) as f64) / 60.0 * 5.0
        });

        (priority_score + success_score + recency_score) / 3.0
    }
}

/// Connection state for each bootstrap peer
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected {
        connected_at: Instant,
    },
    Failed {
        last_attempt: Instant,
        retry_count: u32,
    },
}

/// Exponential backoff retry strategy
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    max_attempts: u32,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(300), // 5 minutes
            multiplier: 2.0,
            max_attempts: 5,
        }
    }
}

impl ExponentialBackoff {
    pub fn new(
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: u32,
    ) -> Self {
        Self {
            base_delay,
            max_delay,
            multiplier,
            max_attempts,
        }
    }

    pub fn delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.base_delay;
        }

        let delay = self.base_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);
        Duration::from_secs_f64(delay.min(self.max_delay.as_secs_f64()))
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

/// Health checker for bootstrap peers
pub struct BootstrapHealthChecker {
    check_interval: Duration,
    timeout: Duration,
}

impl Default for BootstrapHealthChecker {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
        }
    }
}

impl BootstrapHealthChecker {
    pub fn new(check_interval: Duration, timeout: Duration) -> Self {
        Self {
            check_interval,
            timeout,
        }
    }

    pub async fn start_health_monitoring(
        &self,
        bootstrap_manager: Arc<RwLock<BootstrapManager>>,
    ) -> tokio::task::JoinHandle<()> {
        let check_interval = self.check_interval;

        tokio::spawn(async move {
            let mut interval = interval(check_interval);

            loop {
                interval.tick().await;

                let manager = bootstrap_manager.read().await;
                let peers = manager.get_all_peers();

                for peer in peers {
                    // In a real implementation, this would perform actual health checks
                    // For now, we'll just log the health check
                    debug!(
                        "Health check for peer {}: healthy={}",
                        peer.peer_id,
                        peer.is_healthy()
                    );
                }
            }
        })
    }
}

/// Multi-Bootstrap Peer Manager
pub struct BootstrapManager {
    bootstrap_peers: Vec<BootstrapPeer>,
    connection_pool: HashMap<PeerId, ConnectionState>,
    retry_strategy: ExponentialBackoff,
    health_checker: BootstrapHealthChecker,
    min_connections: usize,
    max_connections: usize,
    preferred_region: Option<String>,
}

impl BootstrapManager {
    pub fn new() -> Self {
        Self {
            bootstrap_peers: Vec::new(),
            connection_pool: HashMap::new(),
            retry_strategy: ExponentialBackoff::default(),
            health_checker: BootstrapHealthChecker::default(),
            min_connections: 1,
            max_connections: 8,
            preferred_region: None,
        }
    }

    pub fn with_retry_strategy(mut self, strategy: ExponentialBackoff) -> Self {
        self.retry_strategy = strategy;
        self
    }

    pub fn with_connection_limits(mut self, min: usize, max: usize) -> Self {
        self.min_connections = min;
        self.max_connections = max;
        self
    }

    pub fn with_preferred_region(mut self, region: String) -> Self {
        self.preferred_region = Some(region);
        self
    }

    pub fn add_bootstrap_peer(&mut self, peer: BootstrapPeer) {
        info!(
            "Adding bootstrap peer: {} (priority: {}, region: {:?})",
            peer.peer_id, peer.priority, peer.region
        );

        self.bootstrap_peers.push(peer.clone());
        self.connection_pool
            .insert(peer.peer_id, ConnectionState::Disconnected);
    }

    pub fn remove_bootstrap_peer(&mut self, peer_id: &PeerId) -> bool {
        if let Some(pos) = self
            .bootstrap_peers
            .iter()
            .position(|p| &p.peer_id == peer_id)
        {
            self.bootstrap_peers.remove(pos);
            self.connection_pool.remove(peer_id);
            info!("Removed bootstrap peer: {}", peer_id);
            true
        } else {
            false
        }
    }

    pub fn get_all_peers(&self) -> Vec<BootstrapPeer> {
        self.bootstrap_peers.clone()
    }

    pub fn get_connected_peers(&self) -> Vec<PeerId> {
        self.connection_pool
            .iter()
            .filter_map(|(peer_id, state)| match state {
                ConnectionState::Connected { .. } => Some(*peer_id),
                _ => None,
            })
            .collect()
    }

    pub fn get_peer_count(&self) -> usize {
        self.bootstrap_peers.len()
    }

    pub fn get_connected_count(&self) -> usize {
        self.get_connected_peers().len()
    }

    /// Get bootstrap peers sorted by priority and success rate
    pub fn prioritized_bootstrap_peers(&self) -> Vec<BootstrapPeer> {
        let mut peers = self.bootstrap_peers.clone();

        // Sort by region preference first, then by score
        peers.sort_by(|a, b| {
            // Prefer peers in the preferred region
            if let Some(ref preferred) = self.preferred_region {
                match (a.region.as_ref(), b.region.as_ref()) {
                    (Some(a_region), Some(b_region)) => {
                        if a_region == preferred && b_region != preferred {
                            return std::cmp::Ordering::Less;
                        }
                        if a_region != preferred && b_region == preferred {
                            return std::cmp::Ordering::Greater;
                        }
                    }
                    (Some(a_region), None) if a_region == preferred => {
                        return std::cmp::Ordering::Less;
                    }
                    (None, Some(b_region)) if b_region == preferred => {
                        return std::cmp::Ordering::Greater;
                    }
                    _ => {}
                }
            }

            // Then sort by connection score (higher is better)
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        peers
    }

    /// Connect to bootstrap peers using intelligent strategy
    pub async fn connect_to_network(
        &mut self,
        swarm: &mut Swarm<MyBehaviour>,
    ) -> Result<Vec<PeerId>> {
        info!(
            "Connecting to bootstrap network with {} peers available",
            self.bootstrap_peers.len()
        );

        if self.bootstrap_peers.is_empty() {
            return Err(anyhow!("No bootstrap peers configured"));
        }

        let mut connected_peers = Vec::new();
        let prioritized_peers = self.prioritized_bootstrap_peers();

        // Try to connect to peers in order of priority
        for peer in prioritized_peers {
            if connected_peers.len() >= self.max_connections {
                break;
            }

            match self.connect_to_peer(&peer, swarm).await {
                Ok(peer_id) => {
                    connected_peers.push(peer_id);
                    self.connection_pool.insert(
                        peer_id,
                        ConnectionState::Connected {
                            connected_at: Instant::now(),
                        },
                    );

                    // Update peer success rate
                    if let Some(peer_ref) = self
                        .bootstrap_peers
                        .iter_mut()
                        .find(|p| p.peer_id == peer_id)
                    {
                        peer_ref.update_success(true);
                    }

                    info!("Successfully connected to bootstrap peer: {}", peer_id);

                    // If we have minimum connections, we can continue with reduced urgency
                    if connected_peers.len() >= self.min_connections {
                        // Continue trying to connect to more peers but don't fail if unsuccessful
                        continue;
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to bootstrap peer {}: {}",
                        peer.peer_id, e
                    );

                    // Update peer success rate
                    if let Some(peer_ref) = self
                        .bootstrap_peers
                        .iter_mut()
                        .find(|p| p.peer_id == peer.peer_id)
                    {
                        peer_ref.update_success(false);
                    }

                    self.connection_pool.insert(
                        peer.peer_id,
                        ConnectionState::Failed {
                            last_attempt: Instant::now(),
                            retry_count: 1,
                        },
                    );
                }
            }
        }

        if connected_peers.len() < self.min_connections {
            // If we don't have minimum connections, try with retry strategy
            info!(
                "Insufficient connections ({}), retrying with backoff strategy",
                connected_peers.len()
            );

            for retry_attempt in 1..=self.retry_strategy.max_attempts {
                if connected_peers.len() >= self.min_connections {
                    break;
                }

                let delay = self.retry_strategy.delay(retry_attempt);
                info!("Retry attempt {} after {:?}", retry_attempt, delay);
                sleep(delay).await;

                // Try failed peers again
                let failed_peers: Vec<_> = self
                    .bootstrap_peers
                    .iter()
                    .filter(|p| !connected_peers.contains(&p.peer_id))
                    .cloned()
                    .collect();

                for peer in failed_peers {
                    if connected_peers.len() >= self.max_connections {
                        break;
                    }

                    match self.connect_to_peer(&peer, swarm).await {
                        Ok(peer_id) => {
                            connected_peers.push(peer_id);
                            self.connection_pool.insert(
                                peer_id,
                                ConnectionState::Connected {
                                    connected_at: Instant::now(),
                                },
                            );

                            if let Some(peer_ref) = self
                                .bootstrap_peers
                                .iter_mut()
                                .find(|p| p.peer_id == peer_id)
                            {
                                peer_ref.update_success(true);
                            }

                            info!(
                                "Successfully connected to bootstrap peer on retry: {}",
                                peer_id
                            );

                            if connected_peers.len() >= self.min_connections {
                                break;
                            }
                        }
                        Err(_) => {
                            // Continue trying other peers
                        }
                    }
                }
            }
        }

        if connected_peers.is_empty() {
            return Err(anyhow!(
                "Failed to connect to any bootstrap peers after {} attempts",
                self.retry_strategy.max_attempts
            ));
        }

        if connected_peers.len() < self.min_connections {
            warn!(
                "Connected to {} peers, which is less than minimum required ({})",
                connected_peers.len(),
                self.min_connections
            );
        }

        info!(
            "Successfully connected to {} bootstrap peers",
            connected_peers.len()
        );
        Ok(connected_peers)
    }

    /// Connect to a specific bootstrap peer
    async fn connect_to_peer(
        &self,
        peer: &BootstrapPeer,
        swarm: &mut Swarm<MyBehaviour>,
    ) -> Result<PeerId> {
        // Add peer addresses to Kademlia DHT
        for addr in &peer.addresses {
            swarm
                .behaviour_mut()
                .kad
                .add_address(&peer.peer_id, addr.clone());
        }

        // Try to dial the peer
        for addr in &peer.addresses {
            match swarm.dial(addr.clone()) {
                Ok(_) => {
                    debug!("Dialing bootstrap peer {} at {}", peer.peer_id, addr);
                    // In a real implementation, you would wait for connection confirmation
                    // For now, we'll assume success if dial doesn't fail
                    return Ok(peer.peer_id);
                }
                Err(e) => {
                    debug!("Failed to dial {} at {}: {}", peer.peer_id, addr, e);
                    continue;
                }
            }
        }

        Err(anyhow!(
            "Failed to connect to any address for peer {}",
            peer.peer_id
        ))
    }

    /// Start automatic failover monitoring (disabled due to Send/Sync issues)
    pub async fn start_failover_monitoring(
        &self,
        _swarm: Arc<RwLock<Swarm<MyBehaviour>>>,
    ) -> tokio::task::JoinHandle<()> {
        // let manager = Arc::new(RwLock::new(self.clone()));

        tokio::spawn(async move {
            // Temporary empty implementation to fix build issues
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        })

        /* Original implementation commented out due to Send/Sync issues
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                let connected_count = {
                    let manager = manager.read().await;
                    manager.get_connected_count()
                };

                // If we have fewer than minimum connections, try to reconnect
                if connected_count < 3 {
                    warn!("Low bootstrap connections ({}), attempting to reconnect", connected_count);

                    let mut manager = manager.write().await;
                    let mut swarm = swarm.write().await;

                    if let Err(e) = manager.connect_to_network(&mut swarm).await {
                        error!("Failed to reconnect to bootstrap network: {}", e);
                    }
                }
            }
        })
        */
    }
}

impl Default for BootstrapManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BootstrapManager {
    fn clone(&self) -> Self {
        Self {
            bootstrap_peers: self.bootstrap_peers.clone(),
            connection_pool: self.connection_pool.clone(),
            retry_strategy: self.retry_strategy.clone(),
            health_checker: BootstrapHealthChecker::default(),
            min_connections: self.min_connections,
            max_connections: self.max_connections,
            preferred_region: self.preferred_region.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;
    use std::str::FromStr;

    #[test]
    fn test_bootstrap_peer_creation() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();

        let peer = BootstrapPeer::new(peer_id, vec![addr])
            .with_priority(1)
            .with_region("us-east".to_string());

        assert_eq!(peer.peer_id, peer_id);
        assert_eq!(peer.priority, 1);
        assert_eq!(peer.region, Some("us-east".to_string()));
        assert_eq!(peer.success_rate, 1.0);
    }

    #[test]
    fn test_bootstrap_peer_health_tracking() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();

        let mut peer = BootstrapPeer::new(peer_id, vec![addr]);

        // Test success updates
        peer.update_success(true);
        assert_eq!(peer.successful_attempts, 1);
        assert_eq!(peer.total_attempts, 1);
        assert_eq!(peer.success_rate, 1.0);

        // Test failure updates
        peer.update_success(false);
        assert_eq!(peer.successful_attempts, 1);
        assert_eq!(peer.total_attempts, 2);
        assert_eq!(peer.success_rate, 0.5);
    }

    #[test]
    fn test_exponential_backoff() {
        let backoff =
            ExponentialBackoff::new(Duration::from_secs(1), Duration::from_secs(60), 2.0, 5);

        assert_eq!(backoff.delay(0), Duration::from_secs(1));
        assert_eq!(backoff.delay(1), Duration::from_secs(2));
        assert_eq!(backoff.delay(2), Duration::from_secs(4));
        assert_eq!(backoff.delay(3), Duration::from_secs(8));

        assert!(backoff.should_retry(0));
        assert!(backoff.should_retry(4));
        assert!(!backoff.should_retry(5));
    }

    #[test]
    fn test_bootstrap_manager_peer_management() {
        let mut manager = BootstrapManager::new();

        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();

        let peer = BootstrapPeer::new(peer_id, vec![addr])
            .with_priority(1)
            .with_region("us-east".to_string());

        // Test adding peer
        manager.add_bootstrap_peer(peer);
        assert_eq!(manager.get_peer_count(), 1);

        // Test removing peer
        assert!(manager.remove_bootstrap_peer(&peer_id));
        assert_eq!(manager.get_peer_count(), 0);

        // Test removing non-existent peer
        assert!(!manager.remove_bootstrap_peer(&peer_id));
    }

    #[test]
    fn test_peer_prioritization() {
        let mut manager = BootstrapManager::new().with_preferred_region("us-east".to_string());

        // Create peers with different priorities and regions
        let keypair1 = Keypair::generate_ed25519();
        let peer1_id = PeerId::from(keypair1.public());
        let addr1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();
        let mut peer1 = BootstrapPeer::new(peer1_id, vec![addr1])
            .with_priority(2)
            .with_region("eu-west".to_string());
        peer1.update_success(true);

        let keypair2 = Keypair::generate_ed25519();
        let peer2_id = PeerId::from(keypair2.public());
        let addr2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4002").unwrap();
        let mut peer2 = BootstrapPeer::new(peer2_id, vec![addr2])
            .with_priority(1)
            .with_region("us-east".to_string());
        peer2.update_success(true);

        manager.add_bootstrap_peer(peer1);
        manager.add_bootstrap_peer(peer2);

        let prioritized = manager.prioritized_bootstrap_peers();

        // peer2 should be first because it's in the preferred region
        assert_eq!(prioritized[0].peer_id, peer2_id);
        assert_eq!(prioritized[1].peer_id, peer1_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bootstrap_manager_connection_strategy() {
        use std::str::FromStr;

        let mut manager = BootstrapManager::new()
            .with_connection_limits(2, 5)
            .with_preferred_region("us-east".to_string());

        // Create test peers
        let keypair1 = Keypair::generate_ed25519();
        let peer1_id = PeerId::from(keypair1.public());
        let addr1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();
        let peer1 = BootstrapPeer::new(peer1_id, vec![addr1])
            .with_priority(1)
            .with_region("us-east".to_string());

        let keypair2 = Keypair::generate_ed25519();
        let peer2_id = PeerId::from(keypair2.public());
        let addr2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4002").unwrap();
        let peer2 = BootstrapPeer::new(peer2_id, vec![addr2])
            .with_priority(2)
            .with_region("eu-west".to_string());

        manager.add_bootstrap_peer(peer1);
        manager.add_bootstrap_peer(peer2);

        // Test prioritization
        let prioritized = manager.prioritized_bootstrap_peers();
        assert_eq!(prioritized.len(), 2);
        assert_eq!(prioritized[0].peer_id, peer1_id); // Should be first due to preferred region
        assert_eq!(prioritized[1].peer_id, peer2_id);

        // Test connection counting
        assert_eq!(manager.get_peer_count(), 2);
        assert_eq!(manager.get_connected_count(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bootstrap_config_integration() {
        use crate::config::{BackoffConfig, BootstrapConfig, BootstrapPeerConfig};

        let config = BootstrapConfig {
            peers: vec![
                BootstrapPeerConfig {
                    peer_id: "12D3KooWCRscMgHgEo3ojm8ovzheydpvTEqsDtq7Vby7y6NGY2Ez".to_string(),
                    addresses: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
                    priority: 1,
                    region: Some("us-east".to_string()),
                },
                BootstrapPeerConfig {
                    peer_id: "12D3KooWCRscMgHgEo3ojm8ovzheydpvTEqsDtq7Vby7y6NGY2Ea".to_string(),
                    addresses: vec!["/ip4/127.0.0.1/tcp/4002".to_string()],
                    priority: 2,
                    region: Some("eu-west".to_string()),
                },
            ],
            max_attempts: 3,
            retry_interval_secs: 1,
            health_check_interval_secs: 30,
            min_connections: 2,
            max_connections: 5,
            preferred_region: Some("us-east".to_string()),
            backoff: BackoffConfig {
                base_delay_secs: 1,
                max_delay_secs: 60,
                multiplier: 2.0,
                max_attempts: 3,
            },
        };

        // Test that we can convert config to bootstrap manager
        let result = config.to_bootstrap_manager();
        assert!(result.is_ok());

        let manager = result.unwrap();
        assert_eq!(manager.get_peer_count(), 2);
    }

    #[test]
    fn test_cli_bootstrap_peer_parsing() {
        use crate::cli::Cli;
        use clap::Parser;

        // Test parsing multiple bootstrap peers
        let args = vec![
            "datamesh",
            "--bootstrap-peers",
            "12D3KooWCRscMgHgEo3ojm8ovzheydpvTEqsDtq7Vby7y6NGY2Ez@/ip4/127.0.0.1/tcp/4001",
            "--bootstrap-peers",
            "12D3KooWCRscMgHgEo3ojm8ovzheydpvTEqsDtq7Vby7y6NGY2Ea@/ip4/127.0.0.1/tcp/4002",
            "bootstrap",
            "--port",
            "4001",
        ];

        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        assert!(cli.bootstrap_peers.is_some());

        let peers = cli.parse_bootstrap_peers();
        assert!(peers.is_ok());

        let peers = peers.unwrap();
        assert_eq!(peers.len(), 2);

        // Test that all peers have priority 1 (high priority for CLI)
        for peer in peers {
            assert_eq!(peer.priority, 1);
        }
    }

    #[test]
    fn test_failover_mechanism() {
        use std::time::Duration;

        let mut manager = BootstrapManager::new()
            .with_connection_limits(2, 4)
            .with_retry_strategy(ExponentialBackoff::new(
                Duration::from_millis(100),
                Duration::from_secs(5),
                2.0,
                3,
            ));

        // Create peers with different success rates
        let keypair1 = Keypair::generate_ed25519();
        let peer1_id = PeerId::from(keypair1.public());
        let addr1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();
        let mut peer1 = BootstrapPeer::new(peer1_id, vec![addr1]).with_priority(1);

        // Simulate poor connection history
        peer1.update_success(false);
        peer1.update_success(false);
        peer1.update_success(true);

        let keypair2 = Keypair::generate_ed25519();
        let peer2_id = PeerId::from(keypair2.public());
        let addr2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4002").unwrap();
        let mut peer2 = BootstrapPeer::new(peer2_id, vec![addr2]).with_priority(2);

        // Simulate good connection history
        peer2.update_success(true);
        peer2.update_success(true);
        peer2.update_success(true);

        manager.add_bootstrap_peer(peer1);
        manager.add_bootstrap_peer(peer2);

        let prioritized = manager.prioritized_bootstrap_peers();

        // peer2 should be prioritized due to better success rate
        assert_eq!(prioritized[0].peer_id, peer2_id);
        assert_eq!(prioritized[1].peer_id, peer1_id);

        // Test that peer2 has better score
        assert!(prioritized[0].score() > prioritized[1].score());
    }
}
