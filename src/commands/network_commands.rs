use anyhow::Result;
/// Network operation command handlers
///
/// This module contains handlers for all network-related operations:
/// peers, health, network, discover, distribution, bandwidth
use std::error::Error;

use crate::commands::{CommandContext, CommandHandler};

/// Peers command handler - connects to actual network actor
#[derive(Debug, Clone)]
pub struct PeersCommand {
    pub detailed: bool,
    pub format: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for PeersCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use crate::thread_safe_command_context::ThreadSafeCommandContext;

        ui::print_header("Connected Network Peers");

        // Create thread-safe context to access network actor
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            std::sync::Arc::new(config),
        )
        .await?;

        // Get actual network statistics from the network actor
        match thread_safe_context.get_network_stats().await {
            Ok(stats) => {
                ui::print_success(&format!("Network Statistics (Local Peer: {})", stats.local_peer_id));
                ui::print_key_value("Connected Peers", &stats.connected_peers.to_string());
                ui::print_key_value("Routing Table Size", &stats.routing_table_size.to_string());
                ui::print_key_value("Pending Queries", &stats.pending_queries.to_string());

                if stats.connected_peers == 0 {
                    ui::print_warning("No peers currently connected");
                    ui::print_info("To connect to peers:");
                    ui::print_info("  - Start a bootstrap node: datamesh bootstrap --port 40871");
                    ui::print_info("  - Connect to existing network: datamesh service --bootstrap-peer <ID> --bootstrap-addr <ADDR>");
                    ui::print_info("  - Join interactive mode: datamesh interactive");
                } else {
                    // Attempt to get connected peer list
                    if let Ok(connected_peers) = thread_safe_context.network.get_connected_peers().await {
                        if self.detailed {
                            ui::print_header("Connected Peer Details");
                            for (i, peer_id) in connected_peers.iter().enumerate() {
                                println!("{}. Peer ID: {}", i + 1, peer_id);
                                ui::print_key_value("  Status", "Connected");
                                ui::print_key_value("  Protocol", "libp2p/Kademlia DHT");
                            }
                        } else {
                            ui::print_info(&format!("Peer IDs (showing first 5 of {}):", connected_peers.len()));
                            for (i, peer_id) in connected_peers.iter().take(5).enumerate() {
                                println!("  {}. {}", i + 1, peer_id);
                            }
                            if connected_peers.len() > 5 {
                                println!("  ... and {} more peers", connected_peers.len() - 5);
                                ui::print_info("Use --detailed flag to see all peers");
                            }
                        }
                    }
                }

                if self.format.as_ref().map(|f| f.to_lowercase()) == Some("json".to_string()) {
                    // Output JSON format for programmatic use
                    let json_output = serde_json::json!({
                        "local_peer_id": stats.local_peer_id.to_string(),
                        "connected_peers": stats.connected_peers,
                        "routing_table_size": stats.routing_table_size,
                        "pending_queries": stats.pending_queries,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    println!("{}", serde_json::to_string_pretty(&json_output)?);
                }
            }
            Err(e) => {
                ui::print_error(&format!("Failed to get network statistics: {}", e));
                ui::print_info("Network may not be initialized. Try:");
                ui::print_info("  - datamesh bootstrap --port 40871");
                ui::print_info("  - datamesh service --bootstrap-peer <ID> --bootstrap-addr <ADDR>");
            }
        }

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "network_peers"
    }
}

/// Health command handler - provides real network health monitoring
#[derive(Debug, Clone)]
pub struct HealthCommand {
    pub continuous: bool,
    pub interval: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for HealthCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use tokio::time::{sleep, Duration};

        ui::print_header("Network Health Monitor");

        let run_continuous = self.continuous;
        let check_interval = Duration::from_secs(self.interval.unwrap_or(30));

        // Create thread-safe context for network access
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            std::sync::Arc::new(config),
        )
        .await?;

        loop {
            let start_time = std::time::Instant::now();
            
            // Perform comprehensive health check
            ui::print_info(&format!("Health Check - {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
            
            let mut health_score = 100u8;
            let mut issues = Vec::<String>::new();

            // 1. Network Connectivity Check
            match thread_safe_context.get_network_stats().await {
                Ok(stats) => {
                    ui::print_success(&format!("✅ Network Layer: Operational (Peer: {})", stats.local_peer_id));
                    
                    // Check peer connectivity
                    if stats.connected_peers == 0 {
                        health_score -= 30;
                        issues.push("No peers connected - network isolation".to_string());
                        ui::print_warning("⚠️  Peer Connectivity: No peers connected");
                    } else {
                        ui::print_success(&format!("✅ Peer Connectivity: {} peers connected", stats.connected_peers));
                    }

                    // Check routing table health
                    if stats.routing_table_size < 5 {
                        health_score -= 10;
                        issues.push("Small routing table - limited network reach".to_string());
                        ui::print_warning(&format!("⚠️  Routing Table: Only {} peers known", stats.routing_table_size));
                    } else {
                        ui::print_success(&format!("✅ Routing Table: {} peers known", stats.routing_table_size));
                    }

                    // Check for excessive pending queries
                    if stats.pending_queries > 10 {
                        health_score -= 15;
                        issues.push("High query load - potential network congestion".to_string());
                        ui::print_warning(&format!("⚠️  Query Load: {} pending queries", stats.pending_queries));
                    } else {
                        ui::print_success(&format!("✅ Query Load: {} pending queries", stats.pending_queries));
                    }
                }
                Err(e) => {
                    health_score -= 50;
                    issues.push(format!("Network actor unavailable: {}", e));
                    ui::print_error(&format!("❌ Network Layer: Failed - {}", e));
                }
            }

            // 2. Database Health Check
            match thread_safe_context.database.test_connection() {
                Ok(_) => {
                    ui::print_success("✅ Database: Operational");
                }
                Err(e) => {
                    health_score -= 20;
                    issues.push(format!("Database issues: {}", e));
                    ui::print_error(&format!("❌ Database: Failed - {}", e));
                }
            }

            // 3. Bootstrap Test (if applicable)
            if let Ok(_bootstrap_result) = thread_safe_context.bootstrap().await {
                ui::print_success("✅ Bootstrap: Successful");
            } else {
                // Bootstrap failure is not critical if we already have peers
                if let Ok(stats) = thread_safe_context.get_network_stats().await {
                    if stats.connected_peers == 0 {
                        health_score -= 10;
                        issues.push("Bootstrap failed and no peers connected".to_string());
                        ui::print_warning("⚠️  Bootstrap: Failed (no peers available)");
                    } else {
                        ui::print_info("ℹ️  Bootstrap: Not needed (peers already connected)");
                    }
                }
            }

            // Overall Health Assessment
            let health_status = match health_score {
                90..=100 => ("🟢 EXCELLENT", "green"),
                70..=89 => ("🟡 GOOD", "yellow"),
                50..=69 => ("🟠 WARNING", "orange"), 
                30..=49 => ("🔴 CRITICAL", "red"),
                _ => ("💀 SYSTEM FAILURE", "red"),
            };

            ui::print_header(&format!("Overall Health: {} (Score: {}%)", health_status.0, health_score));
            
            if !issues.is_empty() {
                ui::print_warning("Issues Detected:");
                for issue in &issues {
                    println!("  • {}", issue);
                }
                
                ui::print_info("Recommendations:");
                if issues.iter().any(|i| i.contains("No peers connected")) {
                    ui::print_info("  - Connect to a bootstrap node: datamesh service --bootstrap-peer <ID> --bootstrap-addr <ADDR>");
                    ui::print_info("  - Start your own bootstrap: datamesh bootstrap --port 40871");
                }
                if issues.iter().any(|i| i.contains("Database")) {
                    ui::print_info("  - Check disk space and file permissions");
                    ui::print_info("  - Try cleanup: datamesh cleanup --orphaned");
                }
            }

            let check_duration = start_time.elapsed();
            ui::print_info(&format!("Health check completed in {}ms", check_duration.as_millis()));

            if !run_continuous {
                break;
            }

            println!(
                "\nNext check in {} seconds (Ctrl+C to stop)...",
                check_interval.as_secs()
            );
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

        if let Some(_network_diagnostics) = context.network_diagnostics.as_ref() {
            ui::print_info(&format!(
                "Network topology visualization (depth: {})",
                self.depth.unwrap_or(3)
            ));
            ui::print_info("Full topology analysis available in interactive mode");
            ui::print_info("Use: datamesh interactive > network");
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
        use tokio::time::Duration;

        ui::print_header("Network Discovery");

        let discovery_timeout = Duration::from_secs(self.timeout.unwrap_or(30));

        if let Some(_network_diagnostics) = context.network_diagnostics.as_ref() {
            ui::print_info(&format!(
                "Peer discovery (timeout: {}s) available in interactive mode",
                discovery_timeout.as_secs()
            ));
            ui::print_info("Use: datamesh interactive > discover");
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

        if let (Some(file_key), Some(_public_key)) = (&self.file_key, &self.public_key) {
            if let Some(_network_diagnostics) = context.network_diagnostics.as_ref() {
                ui::print_info(&format!("File distribution analysis for: {}", file_key));
                ui::print_info("Detailed distribution analysis available in interactive mode");
                ui::print_info("Use: datamesh interactive > distribution");
            } else {
                ui::print_warning("Network diagnostics unavailable");
            }
        } else {
            // Show general distribution statistics
            if let Some(_network_diagnostics) = context.network_diagnostics.as_ref() {
                ui::print_info("General distribution statistics available in interactive mode");
                ui::print_info("Use: datamesh interactive > distribution");
            } else {
                ui::print_info(
                    "Use --file-key and --public-key options to analyze specific file distribution",
                );
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

        if let Some(_network_diagnostics) = context.network_diagnostics.as_ref() {
            if let Some(test_peer) = &self.test_peer {
                ui::print_info(&format!(
                    "Testing bandwidth to peer: {} ({}s)",
                    test_peer,
                    test_duration.as_secs()
                ));

                ui::print_info(&format!(
                    "Bandwidth test to peer: {} ({}s)",
                    test_peer,
                    test_duration.as_secs()
                ));
                ui::print_info("Detailed bandwidth testing available in interactive mode");
                ui::print_info("Use: datamesh interactive > bandwidth");
            } else {
                // Test bandwidth to all connected peers
                ui::print_info(&format!(
                    "Testing bandwidth to all peers ({}s)...",
                    test_duration.as_secs()
                ));

                ui::print_info(&format!(
                    "Network bandwidth test ({}s) available in interactive mode",
                    test_duration.as_secs()
                ));
                ui::print_info("Use: datamesh interactive > bandwidth");
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

/// Helper function to format durations into human-readable strings
fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    
    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m {}s", total_seconds / 60, total_seconds % 60)
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Helper function to format bytes into human-readable units
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let base = 1024u64;
    let unit_index = ((bytes as f64).ln() / (base as f64).ln()).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    
    let value = bytes as f64 / (base.pow(unit_index as u32) as f64);
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", value, UNITS[unit_index])
    }
}
