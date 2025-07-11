/// Network operation command handlers
/// 
/// This module contains handlers for all network-related operations:
/// peers, health, network, discover, distribution, bandwidth

use std::error::Error;
use anyhow::Result;

use crate::commands::{CommandHandler, CommandContext};

/// Peers command handler
#[derive(Debug, Clone)]
pub struct PeersCommand {
    pub detailed: bool,
    pub format: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for PeersCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use std::collections::HashMap;
        
        ui::print_header("Network Peers");
        
        // Try to get network stats if available
        if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
            match network_diagnostics.get_peer_info().await {
                Ok(peers) => {
                    if peers.is_empty() {
                        ui::print_info("No peers connected");
                    } else {
                        ui::print_info(&format!("Found {} connected peers:", peers.len()));
                        
                        for (i, peer) in peers.iter().enumerate() {
                            if self.detailed {
                                println!("\n{}. Peer ID: {}", i + 1, peer.peer_id);
                                if let Some(addr) = &peer.address {
                                    println!("   Address: {}", addr);
                                }
                                println!("   Connection Quality: {:.1}%", peer.connection_quality * 100.0);
                                println!("   Latency: {}ms", peer.latency_ms);
                                println!("   Connected: {}", if peer.is_connected { "Yes" } else { "No" });
                            } else {
                                println!("{}. {} ({}ms)", i + 1, peer.peer_id, peer.latency_ms);
                            }
                        }
                    }
                }
                Err(e) => {
                    ui::print_warning(&format!("Failed to get peer info: {}", e));
                    // Fallback to basic peer display
                    ui::print_info("Network diagnostics unavailable - use interactive mode for full peer info");
                }
            }
        } else {
            ui::print_info("Network diagnostics unavailable - use interactive mode for peer discovery");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_peers"
    }
}

/// Health command handler
#[derive(Debug, Clone)]
pub struct HealthCommand {
    pub continuous: bool,
    pub interval: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for HealthCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use tokio::time::{sleep, Duration};
        
        ui::print_header("Network Health Monitor");
        
        let run_continuous = self.continuous;
        let check_interval = Duration::from_secs(self.interval.unwrap_or(30));
        
        loop {
            // Check network health
            if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
                match network_diagnostics.check_network_health().await {
                    Ok(health) => {
                        ui::print_info(&format!("Network Health Score: {:.1}%", health.overall_score * 100.0));
                        println!("Connected Peers: {}", health.connected_peers);
                        println!("Average Latency: {}ms", health.average_latency);
                        println!("Failed Connections: {}", health.failed_connections);
                        println!("Network Uptime: {:.2}%", health.uptime_percentage * 100.0);
                        
                        if health.overall_score < 0.5 {
                            ui::print_warning("Network health is below optimal levels");
                        } else if health.overall_score > 0.8 {
                            ui::print_success("Network health is excellent");
                        } else {
                            ui::print_info("Network health is acceptable");
                        }
                    }
                    Err(e) => {
                        ui::print_error(&format!("Failed to check network health: {}", e));
                    }
                }
            } else {
                ui::print_warning("Network diagnostics unavailable");
                // Basic health check
                ui::print_info("Basic health check: System operational");
            }
            
            if !run_continuous {
                break;
            }
            
            println!("\nNext check in {} seconds (Ctrl+C to stop)...", check_interval.as_secs());
            sleep(check_interval).await;
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_health"
    }
}

/// Network topology command handler
#[derive(Debug, Clone)]
pub struct NetworkCommand {
    pub depth: Option<u32>,
    pub visualize: bool,
}

#[async_trait::async_trait]
impl CommandHandler for NetworkCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_header("Network Topology");
        
        if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
            match network_diagnostics.get_network_topology(self.depth.unwrap_or(3)).await {
                Ok(topology) => {
                    ui::print_info(&format!("Network topology (depth: {})", topology.depth));
                    println!("Total nodes: {}", topology.total_nodes);
                    println!("Direct connections: {}", topology.direct_connections);
                    println!("Indirect connections: {}", topology.indirect_connections);
                    
                    if self.visualize {
                        println!("\n=== Network Visualization ===");
                        for (level, nodes) in topology.levels.iter().enumerate() {
                            println!("Level {}: {} nodes", level, nodes.len());
                            for node in nodes {
                                println!("  └─ {}", node);
                            }
                        }
                    }
                }
                Err(e) => {
                    ui::print_warning(&format!("Failed to get network topology: {}", e));
                    // Fallback topology info
                    ui::print_info("Network topology analysis unavailable - use interactive mode");
                }
            }
        } else {
            ui::print_info("Network diagnostics unavailable - basic topology info:");
            println!("Use interactive mode for detailed network topology visualization");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_topology"
    }
}

/// Discover command handler
#[derive(Debug, Clone)]
pub struct DiscoverCommand {
    pub timeout: Option<u64>,
    pub bootstrap_all: bool,
}

#[async_trait::async_trait]
impl CommandHandler for DiscoverCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use tokio::time::{timeout, Duration};
        
        ui::print_header("Network Discovery");
        
        let discovery_timeout = Duration::from_secs(self.timeout.unwrap_or(30));
        
        if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
            ui::print_info(&format!("Starting peer discovery (timeout: {}s)...", discovery_timeout.as_secs()));
            
            match timeout(discovery_timeout, network_diagnostics.discover_peers(self.bootstrap_all)).await {
                Ok(Ok(discovered_peers)) => {
                    if discovered_peers.is_empty() {
                        ui::print_warning("No new peers discovered");
                    } else {
                        ui::print_success(&format!("Discovered {} new peers:", discovered_peers.len()));
                        for (i, peer) in discovered_peers.iter().enumerate() {
                            println!("{}. {} ({}ms latency)", i + 1, peer.peer_id, peer.latency_ms);
                            if let Some(addr) = &peer.address {
                                println!("   Address: {}", addr);
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    ui::print_error(&format!("Discovery failed: {}", e));
                }
                Err(_) => {
                    ui::print_warning("Discovery timed out");
                }
            }
        } else {
            ui::print_warning("Network diagnostics unavailable");
            ui::print_info("Use interactive mode for peer discovery");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_discover"
    }
}

/// Distribution command handler
#[derive(Debug, Clone)]
pub struct DistributionCommand {
    pub file_key: Option<String>,
    pub public_key: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for DistributionCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_header("File Distribution Analysis");
        
        if let (Some(file_key), Some(public_key)) = (&self.file_key, &self.public_key) {
            if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
                match network_diagnostics.analyze_file_distribution(file_key, public_key).await {
                    Ok(distribution) => {
                        ui::print_info(&format!("File distribution analysis for: {}", file_key));
                        println!("Total replicas: {}", distribution.total_replicas);
                        println!("Healthy replicas: {}", distribution.healthy_replicas);
                        println!("Failed replicas: {}", distribution.failed_replicas);
                        println!("Replication factor: {:.1}x", distribution.replication_factor);
                        println!("Geographic distribution: {} regions", distribution.geographic_regions);
                        
                        if distribution.replication_factor < 3.0 {
                            ui::print_warning("Low replication factor - consider adding more replicas");
                        } else {
                            ui::print_success("File has good distribution across the network");
                        }
                        
                        println!("\n=== Replica Locations ===");
                        for (i, location) in distribution.replica_locations.iter().enumerate() {
                            println!("{}. {} ({})", i + 1, location.peer_id, location.region);
                        }
                    }
                    Err(e) => {
                        ui::print_error(&format!("Failed to analyze distribution: {}", e));
                    }
                }
            } else {
                ui::print_warning("Network diagnostics unavailable");
            }
        } else {
            // Show general distribution statistics
            if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
                match network_diagnostics.get_general_distribution_stats().await {
                    Ok(stats) => {
                        ui::print_info("General distribution statistics:");
                        println!("Total files in network: {}", stats.total_files);
                        println!("Average replication factor: {:.1}x", stats.average_replication);
                        println!("Storage utilization: {:.1}%", stats.storage_utilization * 100.0);
                        println!("Network redundancy: {:.1}%", stats.network_redundancy * 100.0);
                    }
                    Err(e) => {
                        ui::print_warning(&format!("Failed to get distribution stats: {}", e));
                    }
                }
            } else {
                ui::print_info("Use --file-key and --public-key options to analyze specific file distribution");
            }
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_distribution"
    }
}

/// Bandwidth command handler
#[derive(Debug, Clone)]
pub struct BandwidthCommand {
    pub test_peer: Option<String>,
    pub duration: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for BandwidthCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use tokio::time::Duration;
        
        ui::print_header("Network Bandwidth Test");
        
        let test_duration = Duration::from_secs(self.duration.unwrap_or(10));
        
        if let Some(network_diagnostics) = context.network_diagnostics.as_ref() {
            if let Some(test_peer) = &self.test_peer {
                ui::print_info(&format!("Testing bandwidth to peer: {} ({}s)", test_peer, test_duration.as_secs()));
                
                match network_diagnostics.test_peer_bandwidth(test_peer, test_duration).await {
                    Ok(results) => {
                        println!("\n=== Bandwidth Test Results ===");
                        println!("Upload speed: {:.2} Mbps", results.upload_mbps);
                        println!("Download speed: {:.2} Mbps", results.download_mbps);
                        println!("Average latency: {}ms", results.average_latency_ms);
                        println!("Packet loss: {:.2}%", results.packet_loss_percentage);
                        println!("Jitter: {}ms", results.jitter_ms);
                        
                        // Quality assessment
                        let quality_score = (results.upload_mbps + results.download_mbps) / 2.0 - (results.packet_loss_percentage * 10.0);
                        if quality_score > 50.0 {
                            ui::print_success("Excellent connection quality");
                        } else if quality_score > 20.0 {
                            ui::print_info("Good connection quality");
                        } else {
                            ui::print_warning("Poor connection quality");
                        }
                    }
                    Err(e) => {
                        ui::print_error(&format!("Bandwidth test failed: {}", e));
                    }
                }
            } else {
                // Test bandwidth to all connected peers
                ui::print_info(&format!("Testing bandwidth to all peers ({}s)...", test_duration.as_secs()));
                
                match network_diagnostics.test_network_bandwidth(test_duration).await {
                    Ok(results) => {
                        if results.is_empty() {
                            ui::print_warning("No peers available for bandwidth testing");
                        } else {
                            println!("\n=== Network Bandwidth Summary ===");
                            let mut total_upload = 0.0;
                            let mut total_download = 0.0;
                            
                            for (i, result) in results.iter().enumerate() {
                                println!("{}. {} - Up: {:.1} Mbps, Down: {:.1} Mbps", 
                                         i + 1, result.peer_id, result.upload_mbps, result.download_mbps);
                                total_upload += result.upload_mbps;
                                total_download += result.download_mbps;
                            }
                            
                            println!("\nTotal Upload: {:.1} Mbps", total_upload);
                            println!("Total Download: {:.1} Mbps", total_download);
                            println!("Average Upload: {:.1} Mbps", total_upload / results.len() as f64);
                            println!("Average Download: {:.1} Mbps", total_download / results.len() as f64);
                        }
                    }
                    Err(e) => {
                        ui::print_error(&format!("Network bandwidth test failed: {}", e));
                    }
                }
            }
        } else {
            ui::print_warning("Network diagnostics unavailable");
            ui::print_info("Use interactive mode for bandwidth testing");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "network_bandwidth"
    }
}