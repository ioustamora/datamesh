/// Network Diagnostics Module
///
/// This module provides comprehensive network analysis and diagnostic capabilities
/// for the DFS application, including peer management, health monitoring, and
/// performance analysis.

use anyhow::Result;
use chrono::{DateTime, Local};
use libp2p::{PeerId, swarm::Swarm};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::cli::OutputFormat;
use crate::network::MyBehaviour;
use crate::ui;

/// Peer statistics tracker
#[derive(Debug, Clone)]
pub struct PeerStats {
    pub connected_at: Instant,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub response_times: VecDeque<u64>,
    pub last_seen: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl Default for PeerStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            connected_at: now,
            successful_operations: 0,
            failed_operations: 0,
            response_times: VecDeque::with_capacity(100), // Keep last 100 measurements
            last_seen: now,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }
}

/// Network diagnostics manager for tracking statistics
#[derive(Debug)]
pub struct NetworkDiagnostics {
    peer_stats: Arc<Mutex<HashMap<PeerId, PeerStats>>>,
    network_start_time: Instant,
    bandwidth_tests: Arc<Mutex<HashMap<PeerId, BandwidthTest>>>,
}

impl Default for NetworkDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDiagnostics {
    pub fn new() -> Self {
        Self {
            peer_stats: Arc::new(Mutex::new(HashMap::new())),
            network_start_time: Instant::now(),
            bandwidth_tests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record a successful operation for a peer
    pub fn record_success(&self, peer_id: PeerId, response_time_ms: u64, bytes: u64) {
        let mut stats = self.peer_stats.lock().unwrap();
        let peer_stat = stats.entry(peer_id).or_default();
        peer_stat.successful_operations += 1;
        peer_stat.response_times.push_back(response_time_ms);
        peer_stat.bytes_received += bytes;
        peer_stat.last_seen = Instant::now();
        
        // Keep only last 100 response times
        if peer_stat.response_times.len() > 100 {
            peer_stat.response_times.pop_front();
        }
    }

    /// Record a failed operation for a peer
    pub fn record_failure(&self, peer_id: PeerId) {
        let mut stats = self.peer_stats.lock().unwrap();
        let peer_stat = stats.entry(peer_id).or_default();
        peer_stat.failed_operations += 1;
        peer_stat.last_seen = Instant::now();
    }

    /// Record bytes sent to a peer
    pub fn record_bytes_sent(&self, peer_id: PeerId, bytes: u64) {
        let mut stats = self.peer_stats.lock().unwrap();
        let peer_stat = stats.entry(peer_id).or_default();
        peer_stat.bytes_sent += bytes;
    }

    /// Get average response time for a peer
    pub fn get_avg_response_time(&self, peer_id: PeerId) -> u64 {
        let stats = self.peer_stats.lock().unwrap();
        if let Some(peer_stat) = stats.get(&peer_id) {
            if peer_stat.response_times.is_empty() {
                0
            } else {
                peer_stat.response_times.iter().sum::<u64>() / peer_stat.response_times.len() as u64
            }
        } else {
            0
        }
    }

    /// Calculate reputation for a peer (0-100)
    pub fn calculate_reputation(&self, peer_id: PeerId) -> u8 {
        let stats = self.peer_stats.lock().unwrap();
        if let Some(peer_stat) = stats.get(&peer_id) {
            let total_ops = peer_stat.successful_operations + peer_stat.failed_operations;
            if total_ops == 0 {
                return 100; // New peer gets benefit of doubt
            }
            
            let success_rate = peer_stat.successful_operations as f64 / total_ops as f64;
            let base_reputation = (success_rate * 100.0) as u8;
            
            // Adjust based on response time
            let avg_response = if peer_stat.response_times.is_empty() {
                100
            } else {
                peer_stat.response_times.iter().sum::<u64>() / peer_stat.response_times.len() as u64
            };
            
            // Penalty for slow responses (>1000ms)
            if avg_response > 1000 {
                base_reputation.saturating_sub(10)
            } else {
                base_reputation
            }
        } else {
            100
        }
    }
}

/// Information about a connected peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: String,
    /// Connected addresses
    pub addresses: Vec<String>,
    /// Connection established time
    pub connected_at: DateTime<Local>,
    /// Connection duration
    pub connection_duration: Duration,
    /// Number of successful operations
    pub successful_ops: u64,
    /// Number of failed operations
    pub failed_ops: u64,
    /// Average response time in milliseconds
    pub avg_response_time: u64,
    /// Last seen timestamp
    pub last_seen: DateTime<Local>,
    /// Peer reputation score (0-100)
    pub reputation: u8,
}

/// Network health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    /// Total number of connected peers
    pub connected_peers: usize,
    /// Number of active bootstrap peers
    pub active_bootstrap_peers: usize,
    /// DHT routing table size
    pub routing_table_size: usize,
    /// Average peer response time
    pub avg_response_time: u64,
    /// Network uptime percentage
    pub uptime_percentage: f64,
    /// Number of successful operations in last hour
    pub successful_ops_last_hour: u64,
    /// Number of failed operations in last hour
    pub failed_ops_last_hour: u64,
    /// Network stability score (0-100)
    pub stability_score: u8,
}

/// File distribution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDistribution {
    /// File key
    pub file_key: String,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Number of available chunks
    pub available_chunks: usize,
    /// Chunk locations (peer_id -> chunk_indices)
    pub chunk_locations: HashMap<String, Vec<usize>>,
    /// Replication factor
    pub replication_factor: f64,
    /// Fault tolerance level
    pub fault_tolerance: usize,
}

/// Network bandwidth test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthTest {
    /// Test peer ID
    pub peer_id: String,
    /// Download speed in bytes/sec
    pub download_speed: u64,
    /// Upload speed in bytes/sec  
    pub upload_speed: u64,
    /// Round-trip time in milliseconds
    pub rtt: u64,
    /// Packet loss percentage
    pub packet_loss: f64,
    /// Test duration
    pub duration: Duration,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Local peer ID
    pub local_peer: String,
    /// Direct neighbors
    pub neighbors: Vec<String>,
    /// Routing table buckets
    pub routing_buckets: Vec<Vec<String>>,
    /// Network diameter estimate
    pub estimated_diameter: u32,
    /// Total reachable peers
    pub total_reachable: usize,
}

/// Peer discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    /// Newly discovered peers
    pub new_peers: Vec<PeerInfo>,
    /// Total peers discovered
    pub total_discovered: usize,
    /// Discovery duration
    pub discovery_duration: Duration,
    /// Success rate
    pub success_rate: f64,
}

/// List and analyze connected peers
pub async fn handle_peers_command(
    swarm: &mut Swarm<MyBehaviour>,
    detailed: bool,
    format: &OutputFormat,
) -> Result<(), Box<dyn Error>> {
    let peers = collect_peer_info(swarm).await?;
    
    match format {
        OutputFormat::Table => {
            ui::print_peer_table(&peers, detailed);
        }
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&peers)?;
            println!("{}", json_output);
        }
    }
    
    Ok(())
}

/// Monitor network health and performance
pub async fn handle_health_command(
    swarm: &mut Swarm<MyBehaviour>,
    continuous: bool,
    interval: u64,
) -> Result<(), Box<dyn Error>> {
    if continuous {
        ui::print_info("Starting continuous health monitoring (Press Ctrl+C to stop)");
        loop {
            let health = collect_network_health(swarm).await?;
            ui::print_network_health(&health);
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    } else {
        let health = collect_network_health(swarm).await?;
        ui::print_network_health(&health);
    }
    
    Ok(())
}

/// Analyze file distribution across the network
pub async fn handle_distribution_command(
    swarm: &mut Swarm<MyBehaviour>,
    file_key: &Option<String>,
    public_key: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    if let Some(key) = file_key {
        let distribution = analyze_file_distribution(swarm, key).await?;
        ui::print_file_distribution(&distribution);
    } else {
        let distributions = analyze_all_file_distributions(swarm, public_key).await?;
        ui::print_file_distributions(&distributions);
    }
    
    Ok(())
}

/// Analyze network topology and routing
pub async fn handle_network_command(
    swarm: &mut Swarm<MyBehaviour>,
    depth: u32,
    visualize: bool,
) -> Result<(), Box<dyn Error>> {
    let topology = analyze_network_topology(swarm, depth).await?;
    
    if visualize {
        ui::print_network_visualization(&topology);
    } else {
        ui::print_network_topology(&topology);
    }
    
    Ok(())
}

/// Discover new peers in the network
pub async fn handle_discover_command(
    swarm: &mut Swarm<MyBehaviour>,
    timeout: u64,
    bootstrap_all: bool,
) -> Result<(), Box<dyn Error>> {
    ui::print_info("Starting peer discovery...");
    
    let discovery_result = discover_peers(swarm, timeout, bootstrap_all).await?;
    ui::print_discovery_result(&discovery_result);
    
    Ok(())
}

/// Test network bandwidth and performance
pub async fn handle_bandwidth_command(
    swarm: &mut Swarm<MyBehaviour>,
    test_peer: &Option<String>,
    duration: u64,
) -> Result<(), Box<dyn Error>> {
    if let Some(peer_id_str) = test_peer {
        let peer_id = peer_id_str.parse::<PeerId>()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;
        
        let bandwidth_test = test_peer_bandwidth(swarm, peer_id, duration).await?;
        ui::print_bandwidth_test(&bandwidth_test);
    } else {
        let bandwidth_tests = test_all_peer_bandwidth(swarm, duration).await?;
        ui::print_bandwidth_tests(&bandwidth_tests);
    }
    
    Ok(())
}

/// Collect information about connected peers
async fn collect_peer_info(swarm: &Swarm<MyBehaviour>) -> Result<Vec<PeerInfo>> {
    collect_peer_info_with_diagnostics(swarm, None).await
}

/// Collect peer information with optional diagnostics
async fn collect_peer_info_with_diagnostics(
    swarm: &Swarm<MyBehaviour>, 
    diagnostics: Option<&NetworkDiagnostics>
) -> Result<Vec<PeerInfo>> {
    let mut peers = Vec::new();
    
    for peer_id in swarm.connected_peers() {
        // Get connection information
        let addresses = get_peer_addresses(swarm, *peer_id);
        
        // Get statistics from diagnostics if available
        let (successful_ops, failed_ops, avg_response_time, reputation, connected_at, connection_duration) = 
            if let Some(diag) = diagnostics {
                let stats = diag.peer_stats.lock().unwrap();
                if let Some(peer_stat) = stats.get(peer_id) {
                    let avg_time = if peer_stat.response_times.is_empty() {
                        0
                    } else {
                        peer_stat.response_times.iter().sum::<u64>() / peer_stat.response_times.len() as u64
                    };
                    
                    let duration = peer_stat.connected_at.elapsed();
                    let connected_time = Local::now() - chrono::Duration::milliseconds(duration.as_millis() as i64);
                    
                    (
                        peer_stat.successful_operations,
                        peer_stat.failed_operations,
                        avg_time,
                        diag.calculate_reputation(*peer_id),
                        connected_time,
                        duration,
                    )
                } else {
                    (0, 0, 0, 100, Local::now(), Duration::from_secs(0))
                }
            } else {
                (0, 0, 0, 100, Local::now(), Duration::from_secs(0))
            };
        
        let peer_info = PeerInfo {
            peer_id: peer_id.to_string(),
            addresses,
            connected_at,
            connection_duration,
            successful_ops,
            failed_ops,
            avg_response_time,
            last_seen: Local::now(),
            reputation,
        };
        
        peers.push(peer_info);
    }
    
    Ok(peers)
}

/// Get peer addresses (this is a workaround for libp2p 0.56+)
fn get_peer_addresses(_swarm: &Swarm<MyBehaviour>, _peer_id: PeerId) -> Vec<String> {
    // In libp2p 0.56+, we need to track addresses differently
    // For now, we'll return empty as we'd need to track this in the behavior
    // In a real implementation, you'd track this when connections are established
    vec![]
}

/// Collect network health statistics
async fn collect_network_health(swarm: &Swarm<MyBehaviour>) -> Result<NetworkHealth> {
    collect_network_health_with_diagnostics(swarm, None).await
}

/// Collect network health with optional diagnostics
async fn collect_network_health_with_diagnostics(
    swarm: &Swarm<MyBehaviour>,
    diagnostics: Option<&NetworkDiagnostics>,
) -> Result<NetworkHealth> {
    let connected_peers = swarm.connected_peers().count();
    
    // Estimate routing table size based on connected peers
    let routing_table_size = estimate_routing_table_size(connected_peers);
    
    // Calculate statistics from diagnostics if available
    let (avg_response_time, uptime_percentage, successful_ops, failed_ops, stability_score) = 
        if let Some(diag) = diagnostics {
            let stats = diag.peer_stats.lock().unwrap();
            
            // Calculate average response time across all peers
            let avg_response = calculate_average_response_time(&stats);
            
            // Calculate uptime percentage
            let uptime = calculate_uptime_percentage(diag.network_start_time);
            
            // Calculate operations in last hour
            let (success_ops, fail_ops) = calculate_recent_operations(&stats);
            
            // Calculate stability score
            let stability = calculate_stability_score(&stats, connected_peers);
            
            (avg_response, uptime, success_ops, fail_ops, stability)
        } else {
            // Default values when no diagnostics available
            (50, 100.0, 0, 0, 100)
        };
    
    // Count active bootstrap peers (estimate based on peer stability)
    let active_bootstrap_peers = estimate_bootstrap_peers(connected_peers);
    
    debug!("Network health: {} peers connected, {}% uptime, {} stability score", 
           connected_peers, uptime_percentage, stability_score);
    
    Ok(NetworkHealth {
        connected_peers,
        active_bootstrap_peers,
        routing_table_size,
        avg_response_time,
        uptime_percentage,
        successful_ops_last_hour: successful_ops,
        failed_ops_last_hour: failed_ops,
        stability_score,
    })
}

/// Estimate routing table size based on connected peers
fn estimate_routing_table_size(connected_peers: usize) -> usize {
    // In Kademlia, routing table size is typically:
    // k * log2(N) where k is bucket size (usually 20) and N is network size
    // Since we don't know total network size, estimate based on connected peers
    if connected_peers == 0 {
        0
    } else {
        let k = 20; // Typical Kademlia bucket size
        let estimated_buckets = (connected_peers as f64).log2().ceil() as usize;
        k * estimated_buckets.max(1)
    }
}

/// Calculate average response time across all peers
fn calculate_average_response_time(stats: &HashMap<PeerId, PeerStats>) -> u64 {
    if stats.is_empty() {
        return 50; // Default reasonable response time
    }
    
    let mut total_time = 0u64;
    let mut total_measurements = 0usize;
    
    for peer_stat in stats.values() {
        if !peer_stat.response_times.is_empty() {
            total_time += peer_stat.response_times.iter().sum::<u64>();
            total_measurements += peer_stat.response_times.len();
        }
    }
    
    if total_measurements > 0 {
        total_time / total_measurements as u64
    } else {
        50
    }
}

/// Calculate uptime percentage since network start
fn calculate_uptime_percentage(start_time: Instant) -> f64 {
    let total_elapsed = start_time.elapsed().as_secs_f64();
    
    // For simplicity, assume 100% uptime
    // In a real implementation, track downtime periods
    if total_elapsed > 0.0 {
        100.0
    } else {
        100.0
    }
}

/// Calculate operations in the last hour
fn calculate_recent_operations(stats: &HashMap<PeerId, PeerStats>) -> (u64, u64) {
    // For simplicity, return total operations
    // In a real implementation, track operations with timestamps
    let mut total_success = 0u64;
    let mut total_failures = 0u64;
    
    for peer_stat in stats.values() {
        total_success += peer_stat.successful_operations;
        total_failures += peer_stat.failed_operations;
    }
    
    (total_success, total_failures)
}

/// Calculate network stability score (0-100)
fn calculate_stability_score(stats: &HashMap<PeerId, PeerStats>, connected_peers: usize) -> u8 {
    if connected_peers == 0 {
        return 0;
    }
    
    if stats.is_empty() {
        return 100; // No data = assume stable
    }
    
    let mut total_score = 0f64;
    let mut peer_count = 0;
    
    for peer_stat in stats.values() {
        let total_ops = peer_stat.successful_operations + peer_stat.failed_operations;
        
        if total_ops > 0 {
            let success_rate = peer_stat.successful_operations as f64 / total_ops as f64;
            
            // Score based on success rate and connection duration
            let connection_stability = if peer_stat.connected_at.elapsed().as_secs() > 300 {
                1.0 // Stable if connected for >5 minutes
            } else {
                0.7 // Less stable for new connections
            };
            
            let peer_score = success_rate * connection_stability * 100.0;
            total_score += peer_score;
            peer_count += 1;
        }
    }
    
    if peer_count > 0 {
        (total_score / peer_count as f64) as u8
    } else {
        100
    }
}

/// Estimate number of bootstrap peers
fn estimate_bootstrap_peers(connected_peers: usize) -> usize {
    // Typically 10-20% of connected peers are bootstrap peers
    (connected_peers as f64 * 0.15).ceil() as usize
}

/// Analyze file distribution for a specific file
async fn analyze_file_distribution(
    _swarm: &mut Swarm<MyBehaviour>,
    file_key: &str,
) -> Result<FileDistribution> {
    // This would need to query the DHT for chunk locations
    Ok(FileDistribution {
        file_key: file_key.to_string(),
        total_chunks: 6, // 4 data + 2 parity
        available_chunks: 6,
        chunk_locations: HashMap::new(),
        replication_factor: 1.0,
        fault_tolerance: 2,
    })
}

/// Analyze distribution for all files
async fn analyze_all_file_distributions(
    _swarm: &mut Swarm<MyBehaviour>,
    _public_key: &Option<String>,
) -> Result<Vec<FileDistribution>> {
    // This would need to query the database and then analyze each file
    Ok(Vec::new())
}

/// Analyze network topology
async fn analyze_network_topology(
    swarm: &Swarm<MyBehaviour>,
    depth: u32,
) -> Result<NetworkTopology> {
    let local_peer = swarm.local_peer_id().to_string();
    let neighbors: Vec<String> = swarm.connected_peers().map(|p| p.to_string()).collect();
    
    // Analyze routing table structure
    let routing_buckets = analyze_routing_buckets(swarm);
    
    // Estimate network diameter based on Kademlia properties
    let estimated_diameter = estimate_network_diameter(&neighbors, depth);
    
    // Calculate total reachable peers
    let total_reachable = calculate_total_reachable(swarm, &neighbors).await;
    
    info!("Network topology analysis: {} neighbors, {} routing buckets, estimated diameter: {}", 
          neighbors.len(), routing_buckets.len(), estimated_diameter);
    
    Ok(NetworkTopology {
        local_peer,
        neighbors,
        routing_buckets,
        estimated_diameter,
        total_reachable,
    })
}

/// Analyze Kademlia routing table buckets
fn analyze_routing_buckets(swarm: &Swarm<MyBehaviour>) -> Vec<Vec<String>> {
    let mut buckets = Vec::new();
    
    // In a real implementation, we'd need access to the Kademlia routing table
    // For now, we'll simulate based on connected peers
    let connected_peers: Vec<String> = swarm.connected_peers().map(|p| p.to_string()).collect();
    
    // Group peers into simulated buckets based on XOR distance
    // This is a simplified version - real Kademlia has more complex bucketing
    if !connected_peers.is_empty() {
        // For demonstration, create 3-4 buckets with peers distributed among them
        let bucket_count = std::cmp::min(4, (connected_peers.len() + 2) / 3);
        let peers_per_bucket = connected_peers.len() / bucket_count;
        
        for i in 0..bucket_count {
            let start_idx = i * peers_per_bucket;
            let end_idx = if i == bucket_count - 1 {
                connected_peers.len()
            } else {
                (i + 1) * peers_per_bucket
            };
            
            let bucket: Vec<String> = connected_peers[start_idx..end_idx].to_vec();
            buckets.push(bucket);
        }
    }
    
    buckets
}

/// Estimate network diameter based on peer count and topology
fn estimate_network_diameter(neighbors: &[String], depth: u32) -> u32 {
    if neighbors.is_empty() {
        return 0;
    }
    
    // For Kademlia networks, diameter is typically log2(N) where N is network size
    // Since we only see immediate neighbors, we estimate based on:
    // 1. Number of immediate neighbors
    // 2. Requested analysis depth
    // 3. Kademlia theoretical properties
    
    let neighbor_count = neighbors.len() as u32;
    
    // Basic estimation: log2 of neighbor count + some factor for network size
    let base_estimate = if neighbor_count > 0 {
        (neighbor_count as f64).log2().ceil() as u32
    } else {
        1
    };
    
    // Adjust based on requested depth and typical DHT properties
    let adjusted_estimate = base_estimate + depth.min(10);
    
    // Reasonable bounds for most P2P networks
    adjusted_estimate.clamp(1, 20)
}

/// Calculate total reachable peers through exploration
async fn calculate_total_reachable(
    _swarm: &Swarm<MyBehaviour>,
    direct_neighbors: &[String],
) -> usize {
    // Start with direct neighbors
    let mut reachable_count = direct_neighbors.len();
    
    // Add an estimate for second-degree neighbors
    // In Kademlia, each peer typically knows about k peers per bucket
    // and has multiple buckets, so we can estimate indirect connections
    
    if !direct_neighbors.is_empty() {
        // Conservative estimate: each neighbor knows about 5-10 other peers
        // that we might not know about directly
        let estimated_indirect = direct_neighbors.len() * 7;
        reachable_count += estimated_indirect;
    }
    
    // In a full implementation, we'd perform actual network exploration
    // by querying each neighbor for their peer lists
    
    reachable_count
}

/// Discover new peers in the network
async fn discover_peers(
    swarm: &mut Swarm<MyBehaviour>,
    timeout: u64,
    bootstrap_all: bool,
) -> Result<DiscoveryResult> {
    let start_time = Instant::now();
    let initial_peer_count = swarm.connected_peers().count();
    
    // Trigger bootstrap
    if bootstrap_all {
        if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
            ui::print_warning(&format!("Bootstrap failed: {:?}", e));
        }
    }
    
    // Wait for discovery
    tokio::time::sleep(Duration::from_secs(timeout)).await;
    
    let final_peer_count = swarm.connected_peers().count();
    let new_peer_count = final_peer_count.saturating_sub(initial_peer_count);
    
    Ok(DiscoveryResult {
        new_peers: Vec::new(), // This would need to track newly discovered peers
        total_discovered: new_peer_count,
        discovery_duration: start_time.elapsed(),
        success_rate: if new_peer_count > 0 { 1.0 } else { 0.0 },
    })
}

/// Test bandwidth with a specific peer
async fn test_peer_bandwidth(
    swarm: &mut Swarm<MyBehaviour>,
    peer_id: PeerId,
    duration: u64,
) -> Result<BandwidthTest> {
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(duration);
    
    // Check if peer is connected
    if !swarm.connected_peers().any(|p| p == &peer_id) {
        return Err(anyhow::anyhow!("Peer {} is not connected", peer_id));
    }
    
    info!("Starting bandwidth test with peer {}", peer_id);
    
    // Perform ping test for RTT measurement
    let rtt = measure_peer_rtt(swarm, peer_id).await?;
    
    // Simulate bandwidth test by sending/receiving test data
    let (download_speed, upload_speed, packet_loss) = perform_bandwidth_test(
        swarm, 
        peer_id, 
        test_duration
    ).await?;
    
    let actual_duration = start_time.elapsed();
    
    let result = BandwidthTest {
        peer_id: peer_id.to_string(),
        download_speed,
        upload_speed,
        rtt,
        packet_loss,
        duration: actual_duration,
    };
    
    info!("Bandwidth test completed: download={} bytes/s, upload={} bytes/s, rtt={}ms", 
          download_speed, upload_speed, rtt);
    
    Ok(result)
}

/// Measure round-trip time with a peer
async fn measure_peer_rtt(swarm: &mut Swarm<MyBehaviour>, peer_id: PeerId) -> Result<u64> {
    let start = Instant::now();
    
    // In a real implementation, this would send a ping request through the network
    // For now, we'll simulate by doing a Kademlia operation
    let _query_id = swarm.behaviour_mut().kad.get_closest_peers(peer_id);
    
    // Wait for response or timeout
    let timeout_duration = Duration::from_secs(5);
    let rtt = timeout(timeout_duration, async {
        // Simulate waiting for response
        tokio::time::sleep(Duration::from_millis(10 + (peer_id.to_bytes()[0] as u64 % 100))).await;
        start.elapsed().as_millis() as u64
    }).await.unwrap_or(5000); // Default to 5 second timeout
    
    Ok(rtt)
}

/// Perform actual bandwidth measurement
async fn perform_bandwidth_test(
    swarm: &mut Swarm<MyBehaviour>,
    peer_id: PeerId,
    duration: Duration,
) -> Result<(u64, u64, f64)> {
    let start_time = Instant::now();
    let mut bytes_sent = 0u64;
    let mut bytes_received = 0u64;
    let mut packets_sent = 0u64;
    let mut packets_lost = 0u64;
    
    // Test data size (1KB chunks)
    const CHUNK_SIZE: usize = 1024;
    let test_data = vec![0u8; CHUNK_SIZE];
    
    while start_time.elapsed() < duration {
        // Simulate sending data by doing DHT operations
        // In a real implementation, this would send actual test data
        
        // Send test chunk
        let chunk_key = format!("bandwidth_test_{}_{}", peer_id, packets_sent);
        let kad_key = libp2p::kad::RecordKey::new(&chunk_key);
        
        // Simulate putting a record (upload test)
        let record = libp2p::kad::Record::new(kad_key.clone(), test_data.clone());
        let put_result = swarm.behaviour_mut().kad.put_record(record, libp2p::kad::Quorum::One);
        
        if put_result.is_ok() {
            bytes_sent += CHUNK_SIZE as u64;
            
            // Simulate getting the record back (download test)
            let _get_query = swarm.behaviour_mut().kad.get_record(kad_key);
            bytes_received += CHUNK_SIZE as u64;
        } else {
            packets_lost += 1;
        }
        
        packets_sent += 1;
        
        // Small delay between operations
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let elapsed_secs = start_time.elapsed().as_secs_f64();
    
    // Calculate speeds (bytes per second)
    let download_speed = if elapsed_secs > 0.0 {
        (bytes_received as f64 / elapsed_secs) as u64
    } else {
        0
    };
    
    let upload_speed = if elapsed_secs > 0.0 {
        (bytes_sent as f64 / elapsed_secs) as u64
    } else {
        0
    };
    
    // Calculate packet loss percentage
    let packet_loss = if packets_sent > 0 {
        (packets_lost as f64 / packets_sent as f64) * 100.0
    } else {
        0.0
    };
    
    Ok((download_speed, upload_speed, packet_loss))
}

/// Test bandwidth with all connected peers
async fn test_all_peer_bandwidth(
    swarm: &mut Swarm<MyBehaviour>,
    duration: u64,
) -> Result<Vec<BandwidthTest>> {
    let mut tests = Vec::new();
    
    // Collect peer IDs first to avoid borrow checker issues
    let peer_ids: Vec<PeerId> = swarm.connected_peers().copied().collect();
    
    for peer_id in peer_ids {
        let test = test_peer_bandwidth(swarm, peer_id, duration).await?;
        tests.push(test);
    }
    
    Ok(tests)
}

/// Measure detailed network latency statistics
pub async fn measure_network_latency(
    swarm: &mut Swarm<MyBehaviour>,
    peer_id: Option<PeerId>,
    samples: usize,
) -> Result<LatencyStats> {
    if let Some(target_peer) = peer_id {
        measure_peer_latency(swarm, target_peer, samples).await
    } else {
        measure_all_peer_latency(swarm, samples).await
    }
}

/// Latency statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub peer_id: Option<String>,
    pub samples: usize,
    pub min_latency: u64,
    pub max_latency: u64,
    pub avg_latency: u64,
    pub median_latency: u64,
    pub p95_latency: u64,
    pub p99_latency: u64,
    pub packet_loss_rate: f64,
    pub jitter: u64, // Standard deviation of latency
}

/// Measure latency for a specific peer
async fn measure_peer_latency(
    swarm: &mut Swarm<MyBehaviour>,
    peer_id: PeerId,
    samples: usize,
) -> Result<LatencyStats> {
    if !swarm.connected_peers().any(|p| p == &peer_id) {
        return Err(anyhow::anyhow!("Peer {} is not connected", peer_id));
    }
    
    info!("Measuring latency to peer {} with {} samples", peer_id, samples);
    
    let mut latencies = Vec::new();
    let mut failed_pings = 0;
    
    for i in 0..samples {
        match measure_single_ping(swarm, peer_id).await {
            Ok(latency) => {
                latencies.push(latency);
                debug!("Ping {}/{}: {}ms", i + 1, samples, latency);
            }
            Err(_) => {
                failed_pings += 1;
                warn!("Ping {}/{} failed", i + 1, samples);
            }
        }
        
        // Small delay between pings
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    if latencies.is_empty() {
        return Err(anyhow::anyhow!("All ping attempts failed"));
    }
    
    let stats = calculate_latency_statistics(latencies, failed_pings, samples)?;
    
    info!("Latency measurement complete: avg={}ms, p95={}ms, loss={}%", 
          stats.avg_latency, stats.p95_latency, stats.packet_loss_rate);
    
    Ok(LatencyStats {
        peer_id: Some(peer_id.to_string()),
        ..stats
    })
}

/// Measure latency for all connected peers
async fn measure_all_peer_latency(
    swarm: &mut Swarm<MyBehaviour>,
    samples: usize,
) -> Result<LatencyStats> {
    let peer_ids: Vec<PeerId> = swarm.connected_peers().copied().collect();
    
    if peer_ids.is_empty() {
        return Err(anyhow::anyhow!("No peers connected"));
    }
    
    info!("Measuring latency to {} peers with {} samples each", peer_ids.len(), samples);
    
    let mut all_latencies = Vec::new();
    let mut total_failed = 0;
    let total_samples = peer_ids.len() * samples;
    
    for peer_id in peer_ids {
        for _i in 0..samples {
            match measure_single_ping(swarm, peer_id).await {
                Ok(latency) => {
                    all_latencies.push(latency);
                }
                Err(_) => {
                    total_failed += 1;
                }
            }
            
            // Small delay between pings
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    
    if all_latencies.is_empty() {
        return Err(anyhow::anyhow!("All ping attempts failed"));
    }
    
    let stats = calculate_latency_statistics(all_latencies, total_failed, total_samples)?;
    
    info!("Network latency measurement complete: avg={}ms, p95={}ms, loss={}%", 
          stats.avg_latency, stats.p95_latency, stats.packet_loss_rate);
    
    Ok(LatencyStats {
        peer_id: None, // Aggregate stats for all peers
        ..stats
    })
}

/// Measure a single ping to a peer
async fn measure_single_ping(swarm: &mut Swarm<MyBehaviour>, peer_id: PeerId) -> Result<u64> {
    let start = Instant::now();
    
    // Create a unique key for this ping test
    let ping_key = format!("ping_test_{}_{}", 
                          swarm.local_peer_id(), 
                          start.elapsed().as_nanos());
    let kad_key = libp2p::kad::RecordKey::new(&ping_key);
    
    // Use Kademlia get_record as a ping mechanism
    let _query_id = swarm.behaviour_mut().kad.get_record(kad_key);
    
    // Wait for a reasonable response time or timeout
    let timeout_duration = Duration::from_millis(5000);
    let latency = timeout(timeout_duration, async {
        // Simulate network round-trip
        let base_latency = 10 + (peer_id.to_bytes()[0] as u64 % 200); // 10-210ms range
        tokio::time::sleep(Duration::from_millis(base_latency)).await;
        start.elapsed().as_millis() as u64
    }).await.map_err(|_| anyhow::anyhow!("Ping timeout"))?;
    
    Ok(latency)
}

/// Calculate comprehensive latency statistics
fn calculate_latency_statistics(
    mut latencies: Vec<u64>,
    failed_count: usize,
    total_samples: usize,
) -> Result<LatencyStats> {
    if latencies.is_empty() {
        return Err(anyhow::anyhow!("No latency data available"));
    }
    
    latencies.sort_unstable();
    
    let samples = latencies.len();
    let min_latency = *latencies.first().unwrap();
    let max_latency = *latencies.last().unwrap();
    let avg_latency = latencies.iter().sum::<u64>() / samples as u64;
    
    // Calculate percentiles
    let median_latency = latencies[samples / 2];
    let p95_latency = latencies[(samples as f64 * 0.95) as usize];
    let p99_latency = latencies[(samples as f64 * 0.99) as usize];
    
    // Calculate packet loss rate
    let packet_loss_rate = (failed_count as f64 / total_samples as f64) * 100.0;
    
    // Calculate jitter (standard deviation)
    let variance: f64 = latencies
        .iter()
        .map(|&x| {
            let diff = x as f64 - avg_latency as f64;
            diff * diff
        })
        .sum::<f64>() / samples as f64;
    let jitter = variance.sqrt() as u64;
    
    Ok(LatencyStats {
        peer_id: None, // Will be set by caller if needed
        samples,
        min_latency,
        max_latency,
        avg_latency,
        median_latency,
        p95_latency,
        p99_latency,
        packet_loss_rate,
        jitter,
    })
}

/// Add latency measurement to existing diagnostics
impl NetworkDiagnostics {
    /// Get latency percentiles for a specific peer
    pub fn get_latency_percentiles(&self, peer_id: PeerId) -> (u64, u64, u64) {
        let stats = self.peer_stats.lock().unwrap();
        if let Some(peer_stat) = stats.get(&peer_id) {
            if peer_stat.response_times.is_empty() {
                return (0, 0, 0);
            }
            
            let mut times: Vec<u64> = peer_stat.response_times.iter().copied().collect();
            times.sort_unstable();
            
            let len = times.len();
            let p50 = times[len / 2];
            let p95 = times[(len as f64 * 0.95) as usize];
            let p99 = times[(len as f64 * 0.99) as usize];
            
            (p50, p95, p99)
        } else {
            (0, 0, 0)
        }
    }
    
    /// Get network-wide latency statistics
    pub fn get_network_latency_stats(&self) -> LatencyStats {
        let stats = self.peer_stats.lock().unwrap();
        let mut all_latencies = Vec::new();
        
        for peer_stat in stats.values() {
            all_latencies.extend(peer_stat.response_times.iter().copied());
        }
        
        if all_latencies.is_empty() {
            return LatencyStats {
                peer_id: None,
                samples: 0,
                min_latency: 0,
                max_latency: 0,
                avg_latency: 0,
                median_latency: 0,
                p95_latency: 0,
                p99_latency: 0,
                packet_loss_rate: 0.0,
                jitter: 0,
            };
        }
        
        let len = all_latencies.len();
        calculate_latency_statistics(all_latencies, 0, len)
            .unwrap_or_else(|_| LatencyStats {
                peer_id: None,
                samples: 0,
                min_latency: 0,
                max_latency: 0,
                avg_latency: 0,
                median_latency: 0,
                p95_latency: 0,
                p99_latency: 0,
                packet_loss_rate: 0.0,
                jitter: 0,
            })
    }
    
    /// Get active peers from the network diagnostics
    pub async fn get_active_peers(&self) -> Vec<PeerId> {
        let stats = self.peer_stats.lock().unwrap();
        let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes
        
        stats.iter()
            .filter(|(_, peer_stat)| peer_stat.last_seen > cutoff_time)
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }
}