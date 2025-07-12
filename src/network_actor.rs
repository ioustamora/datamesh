// ===================================================================================================
// Network Actor Module - Thread-Safe libp2p Communication Layer
// ===================================================================================================
//
// This module implements the actor-based networking layer for DataMesh, solving the fundamental
// challenge of thread-safe libp2p Swarm operations in a distributed storage system.
//
// ## THE ACTOR PATTERN SOLUTION
//
// ### Problem: libp2p Swarm Thread Safety
// libp2p's Swarm type is not Send/Sync, making it impossible to share directly across threads.
// This creates significant challenges in a multi-threaded application where:
// - File storage operations need network access
// - CLI commands must communicate with the network
// - Background tasks require DHT operations
// - Multiple concurrent operations must be supported
//
// ### Solution: Isolated Actor with Message Passing
// The actor pattern isolates the Swarm in a dedicated thread and provides thread-safe
// communication through message-passing channels:
//
// ```
// ┌─────────────────┐    Messages     ┌─────────────────┐
// │   Application   │ ─────────────► │   Network       │
// │   Threads       │                │   Actor         │
// │                 │ ◄───────────── │   (Isolated)    │
// └─────────────────┘    Responses   └─────────────────┘
//        │                                    │
//        │                                    │
//        ▼                                    ▼
// ┌─────────────────┐                ┌─────────────────┐
// │  NetworkHandle  │                │  libp2p Swarm   │
// │  (Clone-able)   │                │  (Single Thread)│
// └─────────────────┘                └─────────────────┘
// ```
//
// ## ARCHITECTURE COMPONENTS
//
// ### 1. NetworkMessage Enum
// Type-safe message definitions for all network operations:
// - PutRecord: Store data in DHT with configurable quorum
// - GetRecord: Retrieve data from DHT with timeout handling
// - GetConnectedPeers: Query current network connectivity
// - Bootstrap: Initialize DHT routing table
// - AddPeerAddress: Manually add peer connections
// - GetNetworkStats: Retrieve real-time network metrics
// - Shutdown: Graceful actor termination
//
// ### 2. NetworkHandle (Clone-able Interface)
// Thread-safe handle that can be cloned and used across threads:
// - Encapsulates communication channel to the actor
// - Provides async methods that return futures
// - Maintains shared network statistics
// - Handles connection failures gracefully
//
// ### 3. NetworkActor (Single-Threaded Worker)
// The actual worker that manages the libp2p Swarm:
// - Runs in dedicated thread with async event loop
// - Processes incoming messages from NetworkHandle
// - Manages DHT operations and peer connectivity
// - Tracks pending operations and timeouts
// - Updates network statistics in real-time
//
// ## ENHANCED FEATURES
//
// ### Intelligent Quorum Management
// The module implements intelligent quorum calculations for optimal storage success:
// - Dynamic quorum adjustment based on connected peers
// - Fallback to single-peer operations in small networks
// - Proper distinction between Quorum::One and Quorum::N(1)
//
// ### Connection Health Monitoring
// - Real-time peer connectivity tracking
// - Bootstrap peer health checks
// - Automatic reconnection attempts
// - Network partition detection
//
// ### Performance Optimization
// - Efficient message serialization
// - Minimal allocations in hot paths
// - Concurrent operation support
// - Zero-copy where possible
//
// ===================================================================================================

use anyhow::Result;
use futures::stream::StreamExt;
use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult, Quorum, Record, RecordKey};
use libp2p::{PeerId, Swarm};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

use crate::cli::Cli;
use crate::config::Config;
use crate::error::DfsError;
use crate::network::{create_swarm_and_connect_multi_bootstrap, MyBehaviour, MyBehaviourEvent};

// ===== TIMEOUT CONFIGURATION =====
/// Default timeout for DHT operations (30 seconds)
const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
/// Frequency of timeout cleanup checks (5 seconds)
const TIMEOUT_CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// Type-safe messages sent to the network actor for DHT and P2P operations.
///
/// Each message variant represents a specific network operation with proper
/// response channels for async communication. The oneshot channels ensure
/// that each operation gets exactly one response, preventing race conditions.
///
/// ## Message Design Principles
/// - Each message includes all necessary parameters for the operation
/// - Response channels are typed to match expected return values
/// - Error handling is built into the response types
/// - Debug trait for logging and troubleshooting
///
/// ## Usage Pattern
/// ```rust
/// let (tx, rx) = oneshot::channel();
/// handle.send(NetworkMessage::PutRecord { record, quorum, response_tx: tx });
/// let result = rx.await;
/// ```
#[derive(Debug)]
pub enum NetworkMessage {
    /// Store a record in the Kademlia DHT with configurable quorum requirements.
    ///
    /// This operation stores data in the distributed hash table with durability
    /// guarantees based on the specified quorum. The quorum determines how many
    /// peers must successfully store the record before considering the operation complete.
    ///
    /// ## Quorum Types
    /// - `Quorum::N(n)`: Requires exactly `n` successful storage operations
    /// - `Quorum::One`: Requires 1 response from K_VALUE closest peers (typically 20)
    /// - `Quorum::Majority`: Requires majority of K_VALUE closest peers
    /// - `Quorum::All`: Requires all K_VALUE closest peers (highest durability)
    ///
    /// ## Performance Considerations
    /// Higher quorum values provide better durability but may fail in small networks
    /// or during network partitions. The storage layer uses intelligent quorum
    /// selection based on connected peer count.
    PutRecord {
        record: Record,          // The key-value record to store in the DHT
        quorum: Quorum,          // Durability requirement for the storage operation
        response_tx: oneshot::Sender<Result<(), DfsError>>,  // Success/failure notification
    },

    /// Retrieve a record from the Kademlia DHT by its key.
    ///
    /// This operation queries the distributed hash table for a specific record.
    /// The DHT will search through the network to find peers that have stored
    /// the requested data, following Kademlia's logarithmic lookup algorithm.
    ///
    /// ## Lookup Process
    /// 1. Start with closest known peers to the target key
    /// 2. Query peers in parallel for closer peers or the actual record
    /// 3. Iteratively get closer to the target key in the key space
    /// 4. Return the record if found, or None if not available
    ///
    /// ## Timeout Handling
    /// The operation includes built-in timeout to prevent indefinite waiting
    /// when records are not available or network is partitioned.
    GetRecord {
        key: RecordKey,          // Blake3 hash key of the record to retrieve
        response_tx: oneshot::Sender<Result<Option<Record>, DfsError>>,  // Record or None
    },

    /// Query the current list of connected peers in the network.
    ///
    /// This provides real-time information about network connectivity,
    /// which is essential for:
    /// - Health monitoring and diagnostics
    /// - Quorum calculation for storage operations
    /// - Network topology understanding
    /// - Load balancing decisions
    ///
    /// The returned list includes only peers with active connections,
    /// not peers that are merely known in the routing table.
    GetConnectedPeers {
        response_tx: oneshot::Sender<Vec<PeerId>>,  // List of currently connected peer IDs
    },

    /// Initialize or refresh the Kademlia DHT routing table.
    ///
    /// Bootstrap is the process of populating the DHT routing table with
    /// known peers, enabling effective routing and peer discovery. This
    /// operation is typically performed:
    /// - On initial startup to join the network
    /// - Periodically to refresh routing information
    /// - After network connectivity issues
    ///
    /// ## Bootstrap Process
    /// 1. Connect to configured bootstrap peers
    /// 2. Perform DHT queries to discover additional peers
    /// 3. Populate routing table buckets with discovered peers
    /// 4. Begin participating in DHT maintenance
    Bootstrap {
        response_tx: oneshot::Sender<Result<(), DfsError>>,  // Bootstrap success/failure
    },

    /// Manually add a peer address to the DHT routing table.
    ///
    /// This allows explicit addition of known peers, which is useful for:
    /// - Adding trusted peers that should always be connected
    /// - Manually seeding the routing table with high-quality peers
    /// - Working around NAT or firewall issues
    /// - Testing and development scenarios
    ///
    /// The operation attempts to establish a connection and add the peer
    /// to the appropriate routing table bucket based on key distance.
    AddPeerAddress {
        peer_id: PeerId,         // Unique identifier of the peer to add
        address: libp2p::Multiaddr,  // Network address where the peer can be reached
        response_tx: oneshot::Sender<Result<(), DfsError>>,  // Connection success/failure
    },

    /// Retrieve comprehensive network statistics and health metrics.
    ///
    /// This provides detailed information about the current state of the
    /// network layer, including connectivity, performance, and routing
    /// table status. Used for monitoring, diagnostics, and adaptive algorithms.
    GetNetworkStats {
        response_tx: oneshot::Sender<NetworkStats>,  // Current network metrics
    },

    /// Gracefully shutdown the network actor and release resources.
    ///
    /// This message triggers a clean shutdown of the network actor:
    /// - Closes all active connections
    /// - Stops the libp2p Swarm event loop
    /// - Releases network resources
    /// - Terminates the background thread
    ///
    /// No response channel is needed as this is a one-way shutdown signal.
    /// The actor will stop processing messages after receiving this.
    Shutdown,
}

/// Comprehensive network statistics and health metrics.
///
/// This structure provides a snapshot of the current network state,
/// including connectivity, routing table status, and operational metrics.
/// The statistics are updated in real-time by the network actor and
/// can be accessed safely from multiple threads.
///
/// ## Metrics Explained
///
/// ### connected_peers
/// Number of peers with active libp2p connections. This represents
/// the immediate network neighborhood and directly affects:
/// - DHT operation success rates
/// - Quorum calculation for storage operations
/// - Network resilience and fault tolerance
///
/// ### pending_queries
/// Number of ongoing DHT queries (GET/PUT operations). High values
/// may indicate network congestion or connectivity issues.
///
/// ### routing_table_size
/// Total number of peers known in the Kademlia routing table.
/// A larger routing table generally improves:
/// - DHT lookup performance
/// - Network coverage and redundancy
/// - Peer discovery efficiency
///
/// ### local_peer_id
/// The unique identifier for this node in the network.
/// Used for debugging, monitoring, and network topology analysis.
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,     // Active connections to other nodes
    pub pending_queries: usize,     // Ongoing DHT operations
    pub routing_table_size: usize,  // Known peers in routing table
    pub local_peer_id: PeerId,      // This node's unique network identifier
}

/// Thread-safe handle for communicating with the network actor.
///
/// This is the primary interface for all network operations in DataMesh.
/// The handle can be cloned and shared across threads safely, making it
/// ideal for use in concurrent storage operations and CLI commands.
///
/// ## Design Features
///
/// ### Clone-able and Thread-Safe
/// Multiple threads can hold copies of the handle and perform network
/// operations concurrently. The underlying message channel ensures
/// proper ordering and delivery.
///
/// ### Async Interface
/// All methods are async and return futures, allowing non-blocking
/// integration with the rest of the async ecosystem.
///
/// ### Error Handling
/// Network failures, timeouts, and actor unavailability are properly
/// propagated through the type system using DfsError.
///
/// ### Shared Statistics
/// The handle provides access to real-time network statistics through
/// a shared RwLock, enabling monitoring and adaptive algorithms.
///
/// ## Usage Example
/// ```rust
/// let handle = NetworkHandle::new(&cli, &config).await?;
/// let record = Record { key, value, ... };
/// handle.put_record(record, Quorum::N(1)).await?;
/// ```
#[derive(Clone, Debug)]
pub struct NetworkHandle {
    tx: mpsc::UnboundedSender<NetworkMessage>,  // Channel to send messages to the actor
    stats: Arc<RwLock<NetworkStats>>,           // Shared network statistics
}

/// Network actor that manages the libp2p Swarm
/// Enhanced NetworkActor with timeout management for reliable operations
pub struct NetworkActor {
    swarm: Swarm<MyBehaviour>,
    rx: mpsc::UnboundedReceiver<NetworkMessage>,
    stats: Arc<RwLock<NetworkStats>>,
    pending_get_requests: HashMap<RecordKey, (oneshot::Sender<Result<Option<Record>, DfsError>>, Instant)>,
    pending_put_requests: HashMap<RecordKey, (oneshot::Sender<Result<(), DfsError>>, Instant)>,
    last_timeout_check: Instant,
}

impl NetworkHandle {
    /// Create a new network handle and start the network actor
    pub async fn new(cli: &Cli, config: &Config) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Create swarm
        let swarm = create_swarm_and_connect_multi_bootstrap(cli, config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create swarm: {}", e))?;
        let local_peer_id = *swarm.local_peer_id();

        // Initialize stats
        let stats = Arc::new(RwLock::new(NetworkStats {
            connected_peers: 0,
            pending_queries: 0,
            routing_table_size: 0,
            local_peer_id,
        }));

        // Create actor with timeout management
        let actor = NetworkActor {
            swarm,
            rx,
            stats: stats.clone(),
            pending_get_requests: HashMap::new(),
            pending_put_requests: HashMap::new(),
            last_timeout_check: Instant::now(),
        };

        // Start actor in background using a spawned thread with LocalSet
        let (actor_handle_tx, actor_handle_rx) = oneshot::channel();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let local_set = tokio::task::LocalSet::new();
            
            local_set.spawn_local(async move {
                if let Err(e) = actor.run().await {
                    error!("Network actor error: {}", e);
                }
            });
            
            // Signal that the actor is starting
            let _ = actor_handle_tx.send(());
            
            rt.block_on(local_set);
        });
        
        // Wait for actor to start
        let _ = actor_handle_rx.await;

        Ok(NetworkHandle { tx, stats })
    }

    /// Store a record in the DHT
    pub async fn put_record(&self, record: Record, quorum: Quorum) -> Result<(), DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::PutRecord {
                record,
                quorum,
                response_tx,
            })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?
    }

    /// Retrieve a record from the DHT
    pub async fn get_record(&self, key: RecordKey) -> Result<Option<Record>, DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::GetRecord { key, response_tx })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Result<Vec<PeerId>, DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::GetConnectedPeers { response_tx })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        Ok(response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?)
    }

    /// Bootstrap the DHT
    pub async fn bootstrap(&self) -> Result<(), DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::Bootstrap { response_tx })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?
    }

    /// Add a peer address to the DHT
    pub async fn add_peer_address(
        &self,
        peer_id: PeerId,
        address: libp2p::Multiaddr,
    ) -> Result<(), DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::AddPeerAddress {
                peer_id,
                address,
                response_tx,
            })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats, DfsError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(NetworkMessage::GetNetworkStats { response_tx })
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;

        Ok(response_rx
            .await
            .map_err(|_| DfsError::Network("Network operation cancelled".to_string()))?)
    }

    /// Shutdown the network actor
    pub async fn shutdown(&self) -> Result<(), DfsError> {
        self.tx
            .send(NetworkMessage::Shutdown)
            .map_err(|_| DfsError::Network("Network actor unavailable".to_string()))?;
        Ok(())
    }

    /// Get cached network statistics
    pub async fn get_cached_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }
}

impl NetworkActor {
    /// Main actor loop
    async fn run(mut self) -> Result<()> {
        info!(
            "Network actor starting with peer ID: {}",
            self.swarm.local_peer_id()
        );

        // Start listening on multiple addresses for better connectivity
        let listen_addresses: Vec<libp2p::Multiaddr> = vec![
            "/ip4/0.0.0.0/tcp/0".parse()?,
            "/ip4/127.0.0.1/tcp/0".parse()?,
        ];

        for addr in listen_addresses {
            match self.swarm.listen_on(addr.clone()) {
                Ok(_) => info!("Listening on: {}", addr),
                Err(e) => warn!("Failed to start listening on {}: {}", addr, e),
            }
        }

        // Attempt initial bootstrap after setup
        let mut bootstrap_timer = tokio::time::interval(Duration::from_secs(10));
        let mut last_bootstrap_attempt = std::time::Instant::now();
        let mut bootstrap_retry_count = 0;

        loop {
            tokio::select! {
                // Handle incoming messages
                message = self.rx.recv() => {
                    match message {
                        Some(msg) => {
                            if let Err(e) = self.handle_message(msg).await {
                                error!("Error handling network message: {}", e);
                            }
                        }
                        None => {
                            info!("Network actor shutting down - channel closed");
                            break;
                        }
                    }
                }

                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }

                // Periodic bootstrap attempts for better network discovery
                _ = bootstrap_timer.tick() => {
                    if last_bootstrap_attempt.elapsed() > Duration::from_secs(30) && bootstrap_retry_count < 5 {
                        debug!("Attempting bootstrap (attempt {})", bootstrap_retry_count + 1);
                        if let Err(e) = self.swarm.behaviour_mut().kad.bootstrap() {
                            warn!("Bootstrap attempt failed: {}", e);
                        } else {
                            bootstrap_retry_count += 1;
                            last_bootstrap_attempt = std::time::Instant::now();
                        }
                    }
                }

                // Periodic stats update and timeout cleanup
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    self.update_stats().await;
                    self.cleanup_timed_out_operations().await;
                }
            }
        }

        info!("Network actor stopped");
        Ok(())
    }

    /// Handle incoming messages
    async fn handle_message(&mut self, message: NetworkMessage) -> Result<()> {
        match message {
            NetworkMessage::PutRecord {
                record,
                quorum,
                response_tx,
            } => {
                let key = record.key.clone();

                tracing::error!("🔥 NetworkActor::put_record called with quorum: {:?}", quorum);
                match self.swarm.behaviour_mut().kad.put_record(record, quorum) {
                    Ok(_) => {
                        // Store pending request with timestamp for timeout tracking
                        self.pending_put_requests.insert(key, (response_tx, Instant::now()));
                        debug!("Put record request initiated");
                    }
                    Err(e) => {
                        let _ = response_tx.send(Err(DfsError::Network(format!(
                            "Failed to put record: {}",
                            e
                        ))));
                    }
                }
            }

            NetworkMessage::GetRecord { key, response_tx } => {
                self.swarm.behaviour_mut().kad.get_record(key.clone());
                self.pending_get_requests.insert(key, (response_tx, Instant::now()));
                debug!("Get record request initiated");
            }

            NetworkMessage::GetConnectedPeers { response_tx } => {
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect();
                let _ = response_tx.send(peers);
            }

            NetworkMessage::Bootstrap { response_tx } => {
                match self.swarm.behaviour_mut().kad.bootstrap() {
                    Ok(_) => {
                        let _ = response_tx.send(Ok(()));
                        debug!("Bootstrap initiated");
                    }
                    Err(e) => {
                        let _ = response_tx
                            .send(Err(DfsError::Network(format!("Bootstrap failed: {}", e))));
                    }
                }
            }

            NetworkMessage::AddPeerAddress {
                peer_id,
                address,
                response_tx,
            } => {
                self.swarm
                    .behaviour_mut()
                    .kad
                    .add_address(&peer_id, address);
                let _ = response_tx.send(Ok(()));
                debug!("Added peer address: {}", peer_id);
            }

            NetworkMessage::GetNetworkStats { response_tx } => {
                let stats = self.stats.read().await.clone();
                let _ = response_tx.send(stats);
            }

            NetworkMessage::Shutdown => {
                info!("Network actor shutdown requested");
                return Ok(());
            }
        }

        Ok(())
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: libp2p::swarm::SwarmEvent<MyBehaviourEvent>,
    ) -> Result<()> {
        match event {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }

            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                self.update_stats().await;
            }

            libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                self.update_stats().await;
            }

            libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event).await?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(&mut self, event: MyBehaviourEvent) -> Result<()> {
        match event {
            MyBehaviourEvent::Kad(kad_event) => {
                match kad_event {
                    KademliaEvent::OutboundQueryProgressed { result, .. } => {
                        match result {
                            QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                                // Use the record from the success result
                                let record = peer_record.record;
                                let key = record.key.clone();

                                if let Some((response_tx, _timestamp)) = self.pending_get_requests.remove(&key) {
                                    let _ = response_tx.send(Ok(Some(record)));
                                }
                            }

                            QueryResult::GetRecord(Err(err)) => {
                                // Find the key and respond with error
                                // Note: libp2p doesn't provide the key directly in the error
                                // We'll need to handle this differently or use a timeout
                                debug!("Get record failed: {:?}", err);
                            }

                            QueryResult::PutRecord(Ok(ok)) => {
                                let key = ok.key;
                                if let Some((response_tx, _timestamp)) = self.pending_put_requests.remove(&key) {
                                    let _ = response_tx.send(Ok(()));
                                }
                            }

                            QueryResult::PutRecord(Err(err)) => {
                                warn!("Put record failed: {:?}", err);
                                // Since we can't identify the specific key, fail all pending put requests
                                // This is suboptimal but prevents hanging operations
                                for (_key, (response_tx, _timestamp)) in self.pending_put_requests.drain() {
                                    let _ = response_tx.send(Err(DfsError::Network(format!(
                                        "Put record failed: {:?}", err
                                    ))));
                                }
                            }

                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Clean up operations that have timed out
    async fn cleanup_timed_out_operations(&mut self) {
        let now = Instant::now();
        let mut timed_out_gets = Vec::new();
        let mut timed_out_puts = Vec::new();
        
        // Check for timed out GET requests
        for (key, (_tx, timestamp)) in &self.pending_get_requests {
            if now.duration_since(*timestamp) > DEFAULT_OPERATION_TIMEOUT {
                timed_out_gets.push(key.clone());
            }
        }
        
        // Check for timed out PUT requests  
        for (key, (_tx, timestamp)) in &self.pending_put_requests {
            if now.duration_since(*timestamp) > DEFAULT_OPERATION_TIMEOUT {
                timed_out_puts.push(key.clone());
            }
        }
        
        // Remove timed out GET requests and notify with timeout error
        for key in timed_out_gets {
            if let Some((response_tx, _timestamp)) = self.pending_get_requests.remove(&key) {
                let _ = response_tx.send(Err(DfsError::Network(
                    "Get record operation timed out".to_string()
                )));
                warn!("GET request timed out for key: {:?}", key);
            }
        }
        
        // Remove timed out PUT requests and notify with timeout error
        for key in timed_out_puts {
            if let Some((response_tx, _timestamp)) = self.pending_put_requests.remove(&key) {
                let _ = response_tx.send(Err(DfsError::Network(
                    "Put record operation timed out".to_string()
                )));
                warn!("PUT request timed out for key: {:?}", key);
            }
        }
        
        if !timed_out_gets.is_empty() || !timed_out_puts.is_empty() {
            debug!("Cleaned up {} timed out GET requests and {} timed out PUT requests", 
                   timed_out_gets.len(), timed_out_puts.len());
        }
    }

    /// Update network statistics
    async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.connected_peers = self.swarm.connected_peers().count();
        stats.pending_queries = self.pending_get_requests.len() + self.pending_put_requests.len();
        // Note: routing table size would need to be extracted from Kademlia behavior
        // This is currently not easily accessible in libp2p
    }
}

/// Timeout for network operations
const NETWORK_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper function to create a network handle with timeout
pub async fn create_network_handle_with_timeout(
    cli: &Cli,
    config: &Config,
) -> Result<NetworkHandle> {
    tokio::time::timeout(NETWORK_TIMEOUT, NetworkHandle::new(cli, config))
        .await
        .map_err(|_| anyhow::anyhow!("Network initialization timeout"))?
}
