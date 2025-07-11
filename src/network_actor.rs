use anyhow::Result;
use futures::stream::StreamExt;
use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult, Quorum, Record, RecordKey};
use libp2p::{PeerId, Swarm};
/// Network Actor Module
///
/// This module implements the actor-based networking layer for DFS.
/// It provides thread-safe network operations using message-passing
/// instead of sharing the Swarm directly across threads.
///
/// Architecture:
/// - NetworkActor: Manages the libp2p Swarm in a dedicated thread
/// - NetworkMessage: Commands sent to the network actor
/// - NetworkResponse: Responses from network operations
/// - NetworkHandle: Thread-safe handle for communicating with the network actor
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

use crate::cli::Cli;
use crate::config::Config;
use crate::error::DfsError;
use crate::network::{create_swarm_and_connect_multi_bootstrap, MyBehaviour, MyBehaviourEvent};

/// Messages sent to the network actor
#[derive(Debug)]
pub enum NetworkMessage {
    /// Store a record in the DHT
    PutRecord {
        record: Record,
        quorum: Quorum,
        response_tx: oneshot::Sender<Result<(), DfsError>>,
    },
    /// Retrieve a record from the DHT
    GetRecord {
        key: RecordKey,
        response_tx: oneshot::Sender<Result<Option<Record>, DfsError>>,
    },
    /// Get connected peers
    GetConnectedPeers {
        response_tx: oneshot::Sender<Vec<PeerId>>,
    },
    /// Bootstrap the DHT
    Bootstrap {
        response_tx: oneshot::Sender<Result<(), DfsError>>,
    },
    /// Add a peer address to the DHT
    AddPeerAddress {
        peer_id: PeerId,
        address: libp2p::Multiaddr,
        response_tx: oneshot::Sender<Result<(), DfsError>>,
    },
    /// Get network statistics
    GetNetworkStats {
        response_tx: oneshot::Sender<NetworkStats>,
    },
    /// Shutdown the network actor
    Shutdown,
}

/// Network operation statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub pending_queries: usize,
    pub routing_table_size: usize,
    pub local_peer_id: PeerId,
}

/// Handle for communicating with the network actor
#[derive(Clone, Debug)]
pub struct NetworkHandle {
    tx: mpsc::UnboundedSender<NetworkMessage>,
    stats: Arc<RwLock<NetworkStats>>,
}

/// Network actor that manages the libp2p Swarm
pub struct NetworkActor {
    swarm: Swarm<MyBehaviour>,
    rx: mpsc::UnboundedReceiver<NetworkMessage>,
    stats: Arc<RwLock<NetworkStats>>,
    pending_get_requests: HashMap<RecordKey, oneshot::Sender<Result<Option<Record>, DfsError>>>,
    pending_put_requests: HashMap<RecordKey, oneshot::Sender<Result<(), DfsError>>>,
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

        // Create actor
        let actor = NetworkActor {
            swarm,
            rx,
            stats: stats.clone(),
            pending_get_requests: HashMap::new(),
            pending_put_requests: HashMap::new(),
        };

        // Start actor in background
        // Start the network actor in a local task set to avoid Send issues
        let local_set = tokio::task::LocalSet::new();
        local_set.spawn_local(async move {
            if let Err(e) = actor.run().await {
                error!("Network actor error: {}", e);
            }
        });

        // Run the local set on the current thread
        tokio::task::spawn_local(async move {
            local_set.await;
        });

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

        // Start listening
        if let Err(e) = self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?) {
            warn!("Failed to start listening: {}", e);
        }

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

                // Periodic stats update
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    self.update_stats().await;
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

                match self.swarm.behaviour_mut().kad.put_record(record, quorum) {
                    Ok(_) => {
                        // Store pending request
                        self.pending_put_requests.insert(key, response_tx);
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
                self.pending_get_requests.insert(key, response_tx);
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

                                if let Some(response_tx) = self.pending_get_requests.remove(&key) {
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
                                if let Some(response_tx) = self.pending_put_requests.remove(&key) {
                                    let _ = response_tx.send(Ok(()));
                                }
                            }

                            QueryResult::PutRecord(Err(err)) => {
                                debug!("Put record failed: {:?}", err);
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
