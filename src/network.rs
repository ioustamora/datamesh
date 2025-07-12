/// Network Module
///
/// This module implements the peer-to-peer networking functionality for DFS using libp2p.
/// It handles:
/// - Setting up and managing the libp2p Swarm
/// - Kademlia DHT configuration and operations
/// - Peer discovery and bootstrap operations
/// - Network event handling
///
/// The network layer is built on libp2p's Kademlia DHT implementation, which provides
/// a distributed key-value store for file chunks. The module includes functionality for
/// bootstrapping new nodes into the network and maintaining peer connections.
use anyhow::Result;
use futures::stream::StreamExt;
use libp2p::kad::{Behaviour as Kademlia, Event as KademliaEvent};
use libp2p::{
    identity, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder, Multiaddr,
};
use std::error::Error;

use crate::cli::Cli;
use crate::config::Config;

/// Combined network behavior for the DFS node
///
/// This struct combines all network behaviors used by the DFS application,
/// currently just Kademlia DHT functionality.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct MyBehaviour {
    /// Kademlia DHT behavior - using standard MemoryStore for Send + Sync compatibility
    pub kad: Kademlia<libp2p::kad::store::MemoryStore>,
}

/// Events emitted by the network behavior
#[derive(Debug)]
pub enum MyBehaviourEvent {
    /// Kademlia DHT events
    Kad(KademliaEvent),
}

impl From<KademliaEvent> for MyBehaviourEvent {
    fn from(event: KademliaEvent) -> Self {
        MyBehaviourEvent::Kad(event)
    }
}

impl MyBehaviour {}

/// Creates a new libp2p swarm and connects to the network
///
/// # Arguments
///
/// * `cli` - Command line arguments
///
/// # Returns
///
/// A configured libp2p Swarm ready for network operations
pub async fn create_swarm_and_connect(
    cli: &Cli,
    _config: &Config,
) -> Result<Swarm<MyBehaviour>, Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Create the swarm using the new libp2p 0.56 API
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();

            // Use standard memory store for Send + Sync compatibility
            let storage = libp2p::kad::store::MemoryStore::new(peer_id);
            let mut kad = Kademlia::new(peer_id, storage);

            // Configure Kademlia for better connectivity
            // Use Server mode to allow serving DHT requests and better peer discovery
            kad.set_mode(Some(libp2p::kad::Mode::Server));

            // Note: automatic bootstrapping will be handled manually in the application loop

            MyBehaviour { kad }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(120)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if let (Some(peer), Some(addr)) = (cli.bootstrap_peer, cli.bootstrap_addr.clone()) {
        println!("Adding bootstrap peer: {} at {}", peer, addr);
        swarm.behaviour_mut().kad.add_address(&peer, addr.clone());

        // Also try to explicitly connect to the bootstrap peer
        if let Err(e) = swarm.dial(addr.clone()) {
            println!("Warning: Failed to dial bootstrap peer: {:?}", e);
        }

        // Add bootstrap peer to routing table immediately
        swarm.behaviour_mut().kad.add_address(&peer, addr);
    } else if cli.network.as_deref() != Some("local") {
        println!("No bootstrap peers configured");
    }

    Ok(swarm)
}

/// Creates a new libp2p swarm and connects to the network using multi-bootstrap support
///
/// # Arguments
///
/// * `cli` - Command line arguments
/// * `config` - Configuration including bootstrap peer settings
///
/// # Returns
///
/// A configured libp2p Swarm ready for network operations with bootstrap connections
pub async fn create_swarm_and_connect_multi_bootstrap(
    cli: &Cli,
    config: &Config,
) -> Result<Swarm<MyBehaviour>, Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Create the swarm using the new libp2p 0.56 API
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();

            // Use standard memory store for Send + Sync compatibility
            let storage = libp2p::kad::store::MemoryStore::new(peer_id);
            let mut kad = Kademlia::new(peer_id, storage);

            // Configure Kademlia for better connectivity
            // Use Server mode to allow serving DHT requests and better peer discovery
            kad.set_mode(Some(libp2p::kad::Mode::Server));

            MyBehaviour { kad }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(120)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Use multi-bootstrap manager for connections
    let mut bootstrap_manager = config.network.bootstrap.to_bootstrap_manager()?;

    // Add CLI bootstrap peers if provided (for backward compatibility)
    match cli.get_all_bootstrap_peers() {
        Ok(cli_peers) => {
            for peer in cli_peers {
                println!("Adding CLI bootstrap peer: {}", peer.peer_id);
                bootstrap_manager.add_bootstrap_peer(peer);
            }
        }
        Err(e) => {
            println!("Warning: Failed to parse CLI bootstrap peers: {}", e);
        }
    }

    // Connect to bootstrap network
    if bootstrap_manager.get_peer_count() > 0 {
        println!(
            "Connecting to bootstrap network with {} peers",
            bootstrap_manager.get_peer_count()
        );

        match bootstrap_manager.connect_to_network(&mut swarm).await {
            Ok(connected_peers) => {
                println!(
                    "Successfully connected to {} bootstrap peers",
                    connected_peers.len()
                );
                for peer_id in connected_peers {
                    println!("Connected to: {}", peer_id);
                }
            }
            Err(e) => {
                println!("Warning: Failed to connect to bootstrap network: {}", e);
                // Continue without bootstrap connections
            }
        }
    } else {
        println!("No bootstrap peers configured");
    }

    Ok(swarm)
}

/// Starts a service node that connects to a bootstrap node
///
/// This function initializes a libp2p swarm configured as a regular service node,
/// connecting to the specified bootstrap node for network participation.
///
/// # Arguments
///
/// * `port` - The port to listen on for incoming connections
/// * `bootstrap_peer` - The peer ID of the bootstrap node to connect to
/// * `bootstrap_addr` - The multiaddress of the bootstrap node
///
/// # Returns
///
/// Result indicating success or failure
pub async fn start_service_node(port: u16, bootstrap_peer: Option<PeerId>, bootstrap_addr: Option<Multiaddr>, _config: &Config) -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();

            // Use standard memory store for Send + Sync compatibility
            let storage = libp2p::kad::store::MemoryStore::new(peer_id);
            let mut kad = Kademlia::new(peer_id, storage);

            // Configure as a client mode for service nodes
            kad.set_mode(Some(libp2p::kad::Mode::Client));

            // Add bootstrap peer if provided
            if let (Some(peer_id), Some(addr)) = (bootstrap_peer, bootstrap_addr.clone()) {
                kad.add_address(&peer_id, addr);
            }

            MyBehaviour { kad }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    println!("Starting as service node on port {}", port);
    if let Some(addr) = &bootstrap_addr {
        println!("Bootstrap address: {}", addr);
    }
    
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
    swarm.listen_on(listen_addr.parse()?)?;

    println!("Service node started!");
    println!("Peer ID: {}", local_peer_id);
    
    // Try to connect to bootstrap node if provided
    if let (Some(peer_id), Some(addr)) = (bootstrap_peer, bootstrap_addr) {
        println!("Connecting to bootstrap node: {} at {}", peer_id, addr);
        swarm.dial(addr)?;
    }

    println!("\nWaiting for connections...");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to peer: {}", peer_id);
                // Trigger bootstrap when connected
                if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
                    println!("Bootstrap trigger failed: {:?}", e);
                } else {
                    println!("Bootstrap triggered successfully");
                }
            }
            SwarmEvent::Behaviour(event) => {
                match event {
                    MyBehaviourEvent::Kad(kad_event) => {
                        println!("Kad event: {:?}", kad_event);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Starts a bootstrap node for the network
///
/// This function initializes a libp2p swarm configured as a bootstrap node,
/// listening for incoming connections from other peers.
///
/// # Arguments
///
/// * `port` - The port to listen on for incoming connections
///
/// # Returns
///
/// Result indicating success or failure
pub async fn start_bootstrap_node(port: u16, _config: &Config) -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();

            // Use standard memory store for Send + Sync compatibility
            let storage = libp2p::kad::store::MemoryStore::new(peer_id);
            let mut kad = Kademlia::new(peer_id, storage);

            // Configure as a server mode for better connectivity
            kad.set_mode(Some(libp2p::kad::Mode::Server));

            MyBehaviour { kad }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    println!("Starting as bootstrap node on port {}", port);
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
    swarm.listen_on(listen_addr.parse()?)?;

    println!("Bootstrap node started!");
    println!("Peer ID: {}", local_peer_id);
    println!("Other nodes can connect with:");
    println!("  --bootstrap-peer {}", local_peer_id);
    println!("  --bootstrap-addr /ip4/<YOUR_IP>/tcp/{}", port);
    println!("\nWaiting for connections...");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(event) => {
                match event {
                    MyBehaviourEvent::Kad(_kad_event) => {
                        // Handle Kademlia events for bootstrap node
                    }
                }
            }
            _ => {}
        }
    }
}
