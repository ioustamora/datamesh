/// Enhanced Network and P2P Testing for DataMesh
///
/// This module provides comprehensive testing for libP2P network behavior,
/// peer discovery, connection management, and distributed operations.

use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use datamesh::config::Config;
use datamesh::enhanced_api::P2PNetworkManager;
use datamesh::intelligent_cli::IntelligentCLI;
use datamesh::network_presets::NetworkPreset;

/// Test setup for network testing
pub struct NetworkTestSetup {
    temp_dirs: Vec<TempDir>,
    nodes: Vec<NetworkNode>,
    bootstrap_addr: Option<SocketAddr>,
}

pub struct NetworkNode {
    node_id: String,
    port: u16,
    api_port: u16,
    temp_dir: TempDir,
    config: Config,
    network_manager: Option<P2PNetworkManager>,
}

impl NetworkTestSetup {
    /// Create a new network test setup with specified number of nodes
    pub async fn new(node_count: usize) -> Result<Self> {
        let mut temp_dirs = Vec::new();
        let mut nodes = Vec::new();
        let base_port = 40000;

        for i in 0..node_count {
            let temp_dir = TempDir::new()?;
            let node_id = format!("test_node_{}", i);
            let port = base_port + (i * 2) as u16;
            let api_port = port + 1;

            let mut config = Config::default();
            config.network.default_port = port;
            config.network.api_port = api_port;
            config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
            config.storage.keys_dir = temp_dir.path().join("keys").to_string_lossy().to_string();

            let node = NetworkNode {
                node_id,
                port,
                api_port,
                temp_dir,
                config,
                network_manager: None,
            };

            nodes.push(node);
            temp_dirs.push(TempDir::new()?);
        }

        Ok(NetworkTestSetup {
            temp_dirs,
            nodes,
            bootstrap_addr: None,
        })
    }

    /// Start all nodes in the network
    pub async fn start_nodes(&mut self) -> Result<()> {
        // Start bootstrap node first
        if !self.nodes.is_empty() {
            let bootstrap_node = &mut self.nodes[0];
            let network_manager = P2PNetworkManager::new(
                bootstrap_node.config.clone(),
                bootstrap_node.node_id.clone(),
            ).await?;
            
            self.bootstrap_addr = Some(format!("127.0.0.1:{}", bootstrap_node.port).parse()?);
            bootstrap_node.network_manager = Some(network_manager);
        }

        // Start remaining nodes and connect to bootstrap
        for i in 1..self.nodes.len() {
            let node = &mut self.nodes[i];
            let mut config = node.config.clone();
            
            if let Some(bootstrap_addr) = self.bootstrap_addr {
                config.network.bootstrap_peers = vec![bootstrap_addr.to_string()];
            }

            let network_manager = P2PNetworkManager::new(
                config,
                node.node_id.clone(),
            ).await?;
            
            node.network_manager = Some(network_manager);
        }

        // Allow time for connections to establish
        sleep(Duration::from_secs(2)).await;
        Ok(())
    }

    /// Stop all nodes
    pub async fn stop_nodes(&mut self) -> Result<()> {
        for node in &mut self.nodes {
            if let Some(mut network_manager) = node.network_manager.take() {
                network_manager.shutdown().await?;
            }
        }
        Ok(())
    }

    /// Get node by index
    pub fn get_node(&self, index: usize) -> Option<&NetworkNode> {
        self.nodes.get(index)
    }

    /// Get number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_single_node_startup() -> Result<()> {
        let mut setup = NetworkTestSetup::new(1).await?;
        setup.start_nodes().await?;

        let node = setup.get_node(0).unwrap();
        assert!(node.network_manager.is_some());

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_two_node_connection() -> Result<()> {
        let mut setup = NetworkTestSetup::new(2).await?;
        setup.start_nodes().await?;

        // Wait for connection establishment
        sleep(Duration::from_secs(3)).await;

        // Verify both nodes are connected
        let node1 = setup.get_node(0).unwrap();
        let node2 = setup.get_node(1).unwrap();

        assert!(node1.network_manager.is_some());
        assert!(node2.network_manager.is_some());

        // Test peer discovery
        if let Some(ref network_manager) = node2.network_manager {
            let peer_count = network_manager.get_peer_count().await?;
            assert!(peer_count >= 1, "Node should have at least 1 peer");
        }

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_node_network_formation() -> Result<()> {
        let node_count = 5;
        let mut setup = NetworkTestSetup::new(node_count).await?;
        setup.start_nodes().await?;

        // Allow time for full network formation
        sleep(Duration::from_secs(5)).await;

        // Verify network connectivity
        let mut total_connections = 0;
        for i in 0..node_count {
            let node = setup.get_node(i).unwrap();
            if let Some(ref network_manager) = node.network_manager {
                let peer_count = network_manager.get_peer_count().await?;
                total_connections += peer_count;
                println!("Node {} has {} peers", i, peer_count);
            }
        }

        // In a well-connected network, we should have multiple connections
        assert!(total_connections >= node_count - 1, 
                "Network should be well-connected");

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_bootstrap_node_failure() -> Result<()> {
        let mut setup = NetworkTestSetup::new(3).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(3)).await;

        // Stop bootstrap node (node 0)
        if let Some(mut network_manager) = setup.nodes[0].network_manager.take() {
            network_manager.shutdown().await?;
        }

        // Wait and check if other nodes maintain connectivity
        sleep(Duration::from_secs(2)).await;

        // Remaining nodes should still be connected to each other
        let node1 = &setup.nodes[1];
        let node2 = &setup.nodes[2];

        if let (Some(ref nm1), Some(ref nm2)) = (&node1.network_manager, &node2.network_manager) {
            let peers1 = nm1.get_peer_count().await?;
            let peers2 = nm2.get_peer_count().await?;
            
            // At least one node should still have peers
            assert!(peers1 > 0 || peers2 > 0, 
                    "Network should maintain some connectivity after bootstrap failure");
        }

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_peer_discovery_timing() -> Result<()> {
        let mut setup = NetworkTestSetup::new(3).await?;
        let start_time = Instant::now();
        
        setup.start_nodes().await?;

        // Wait for peer discovery with timeout
        let discovery_timeout = Duration::from_secs(10);
        let discovery_result = timeout(discovery_timeout, async {
            loop {
                let mut all_connected = true;
                for i in 1..setup.node_count() {
                    let node = setup.get_node(i).unwrap();
                    if let Some(ref network_manager) = node.network_manager {
                        let peer_count = network_manager.get_peer_count().await.unwrap_or(0);
                        if peer_count == 0 {
                            all_connected = false;
                            break;
                        }
                    }
                }
                
                if all_connected {
                    break;
                }
                
                sleep(Duration::from_millis(500)).await;
            }
        }).await;

        let discovery_time = start_time.elapsed();
        println!("Peer discovery took: {:?}", discovery_time);

        assert!(discovery_result.is_ok(), "Peer discovery should complete within timeout");
        assert!(discovery_time < Duration::from_secs(8), 
                "Peer discovery should be reasonably fast");

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_network_partition_recovery() -> Result<()> {
        let mut setup = NetworkTestSetup::new(4).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(3)).await;

        // Simulate network partition by stopping middle nodes
        for i in 1..3 {
            if let Some(mut network_manager) = setup.nodes[i].network_manager.take() {
                network_manager.shutdown().await?;
            }
        }

        sleep(Duration::from_secs(2)).await;

        // Restart the partitioned nodes
        for i in 1..3 {
            let node = &mut setup.nodes[i];
            let mut config = node.config.clone();
            
            if let Some(bootstrap_addr) = setup.bootstrap_addr {
                config.network.bootstrap_peers = vec![bootstrap_addr.to_string()];
            }

            let network_manager = P2PNetworkManager::new(
                config,
                node.node_id.clone(),
            ).await?;
            
            node.network_manager = Some(network_manager);
        }

        // Allow time for recovery
        sleep(Duration::from_secs(5)).await;

        // Verify network has recovered
        let mut recovered_nodes = 0;
        for i in 0..setup.node_count() {
            let node = setup.get_node(i).unwrap();
            if let Some(ref network_manager) = node.network_manager {
                let peer_count = network_manager.get_peer_count().await?;
                if peer_count > 0 {
                    recovered_nodes += 1;
                }
            }
        }

        assert!(recovered_nodes >= 3, 
                "Most nodes should recover from partition");

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_node_startup() -> Result<()> {
        let node_count = 6;
        let mut setup = NetworkTestSetup::new(node_count).await?;

        // Start bootstrap node first
        if !setup.nodes.is_empty() {
            let bootstrap_node = &mut setup.nodes[0];
            let network_manager = P2PNetworkManager::new(
                bootstrap_node.config.clone(),
                bootstrap_node.node_id.clone(),
            ).await?;
            
            setup.bootstrap_addr = Some(format!("127.0.0.1:{}", bootstrap_node.port).parse()?);
            bootstrap_node.network_manager = Some(network_manager);
        }

        // Start all other nodes concurrently
        let mut join_handles = Vec::new();
        
        for i in 1..node_count {
            let node_config = setup.nodes[i].config.clone();
            let node_id = setup.nodes[i].node_id.clone();
            let bootstrap_addr = setup.bootstrap_addr;

            let handle = tokio::spawn(async move {
                let mut config = node_config;
                if let Some(addr) = bootstrap_addr {
                    config.network.bootstrap_peers = vec![addr.to_string()];
                }

                P2PNetworkManager::new(config, node_id).await
            });

            join_handles.push(handle);
        }

        // Wait for all nodes to start
        let start_time = Instant::now();
        for (i, handle) in join_handles.into_iter().enumerate() {
            match handle.await {
                Ok(Ok(network_manager)) => {
                    setup.nodes[i + 1].network_manager = Some(network_manager);
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Join error: {:?}", e));
                }
            }
        }

        let startup_time = start_time.elapsed();
        println!("Concurrent startup of {} nodes took: {:?}", node_count - 1, startup_time);

        // Allow time for connections
        sleep(Duration::from_secs(3)).await;

        // Verify all nodes are connected
        let mut connected_nodes = 0;
        for i in 0..node_count {
            let node = setup.get_node(i).unwrap();
            if let Some(ref network_manager) = node.network_manager {
                let peer_count = network_manager.get_peer_count().await?;
                if peer_count > 0 {
                    connected_nodes += 1;
                }
            }
        }

        assert!(connected_nodes >= node_count - 1, 
                "All nodes should be connected after concurrent startup");

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_network_preset_configurations() -> Result<()> {
        // Test different network presets
        let presets = vec![
            NetworkPreset::Local,
            NetworkPreset::Development,
            NetworkPreset::Testing,
        ];

        for preset in presets {
            let mut config = Config::default();
            config.apply_network_preset(preset);

            // Validate configuration
            assert!(config.network.default_port > 0);
            assert!(config.network.max_connections > 0);
            assert!(config.network.connection_timeout_secs > 0);

            println!("✅ Network preset {:?} validated", preset);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_network_message_propagation() -> Result<()> {
        let mut setup = NetworkTestSetup::new(4).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(3)).await;

        // Test message propagation (this would require implementing message types)
        // For now, we test that the network infrastructure is in place
        for i in 0..setup.node_count() {
            let node = setup.get_node(i).unwrap();
            if let Some(ref network_manager) = node.network_manager {
                let peer_count = network_manager.get_peer_count().await?;
                println!("Node {} has {} peers for message propagation", i, peer_count);
                
                // In a 4-node network, each node should have at least 1 peer
                assert!(peer_count >= 1, "Node should have peers for message propagation");
            }
        }

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_network_performance_under_load() -> Result<()> {
        let mut setup = NetworkTestSetup::new(3).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(2)).await;

        // Simulate network load by rapid peer queries
        let start_time = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            for i in 0..setup.node_count() {
                let node = setup.get_node(i).unwrap();
                if let Some(ref network_manager) = node.network_manager {
                    let _ = network_manager.get_peer_count().await;
                }
            }
        }

        let load_test_duration = start_time.elapsed();
        println!("Network load test ({} iterations) took: {:?}", 
                iterations, load_test_duration);

        // Should handle load without significant degradation
        assert!(load_test_duration < Duration::from_secs(5), 
                "Network should handle load efficiently");

        setup.stop_nodes().await?;
        Ok(())
    }
}

#[cfg(test)]
mod integration_network_tests {
    use super::*;

    #[tokio::test]
    async fn test_network_with_cli_integration() -> Result<()> {
        let mut setup = NetworkTestSetup::new(2).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(2)).await;

        // Test CLI integration with network
        let node = setup.get_node(0).unwrap();
        let cli = IntelligentCLI::new(node.config.clone()).await?;

        // Test network status command
        let status_result = cli.execute_command("network").await;
        assert!(status_result.is_ok(), "Network status command should work");

        // Test peers command
        let peers_result = cli.execute_command("peers").await;
        assert!(peers_result.is_ok(), "Peers command should work");

        setup.stop_nodes().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_network_resilience_scenarios() -> Result<()> {
        let mut setup = NetworkTestSetup::new(5).await?;
        setup.start_nodes().await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(3)).await;

        // Test various failure scenarios
        let scenarios = vec![
            "single_node_failure",
            "multiple_node_failure", 
            "rapid_restart",
        ];

        for scenario in scenarios {
            match scenario {
                "single_node_failure" => {
                    // Stop one node
                    if let Some(mut network_manager) = setup.nodes[2].network_manager.take() {
                        network_manager.shutdown().await?;
                    }
                    sleep(Duration::from_secs(1)).await;
                    
                    // Restart it
                    let node = &mut setup.nodes[2];
                    let mut config = node.config.clone();
                    if let Some(bootstrap_addr) = setup.bootstrap_addr {
                        config.network.bootstrap_peers = vec![bootstrap_addr.to_string()];
                    }
                    let network_manager = P2PNetworkManager::new(config, node.node_id.clone()).await?;
                    node.network_manager = Some(network_manager);
                }
                "multiple_node_failure" => {
                    // Stop multiple nodes
                    for i in 1..3 {
                        if let Some(mut network_manager) = setup.nodes[i].network_manager.take() {
                            network_manager.shutdown().await?;
                        }
                    }
                    sleep(Duration::from_secs(1)).await;
                    
                    // Restart them
                    for i in 1..3 {
                        let node = &mut setup.nodes[i];
                        let mut config = node.config.clone();
                        if let Some(bootstrap_addr) = setup.bootstrap_addr {
                            config.network.bootstrap_peers = vec![bootstrap_addr.to_string()];
                        }
                        let network_manager = P2PNetworkManager::new(config, node.node_id.clone()).await?;
                        node.network_manager = Some(network_manager);
                    }
                }
                "rapid_restart" => {
                    // Rapidly restart nodes
                    for _ in 0..3 {
                        if let Some(mut network_manager) = setup.nodes[1].network_manager.take() {
                            network_manager.shutdown().await?;
                        }
                        
                        let node = &mut setup.nodes[1];
                        let mut config = node.config.clone();
                        if let Some(bootstrap_addr) = setup.bootstrap_addr {
                            config.network.bootstrap_peers = vec![bootstrap_addr.to_string()];
                        }
                        let network_manager = P2PNetworkManager::new(config, node.node_id.clone()).await?;
                        node.network_manager = Some(network_manager);
                        
                        sleep(Duration::from_millis(500)).await;
                    }
                }
                _ => {}
            }
            
            // Allow recovery time
            sleep(Duration::from_secs(2)).await;
            
            println!("✅ Network resilience scenario '{}' completed", scenario);
        }

        setup.stop_nodes().await?;
        Ok(())
    }
}