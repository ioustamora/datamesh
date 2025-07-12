use crate::commands::{CommandContext, CommandHandler};
/// Service operation command handlers
///
/// This module contains handlers for all service-related operations:
/// bootstrap, interactive, service
use std::error::Error;
use crate::{config, network};

/// Bootstrap command handler
#[derive(Debug, Clone)]
pub struct BootstrapCommand {
    pub port: u16,
}

#[async_trait::async_trait]
impl CommandHandler for BootstrapCommand {
    fn command_name(&self) -> &'static str {
        "bootstrap"
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};
        
        ui::print_header("Bootstrap Node");
        ui::print_info(&format!("Starting bootstrap node on port {}", self.port));
        
        // Use the actor-based networking system for consistency
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let mut cli_config = context.cli.clone();
        
        // Set the port for the bootstrap node
        if let crate::cli::Commands::Bootstrap { port } = &mut cli_config.command {
            *port = self.port;
        }
        
        let thread_safe_context = ThreadSafeCommandContext::new(
            cli_config,
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Bootstrap node initialized using actor-based system");
        ui::print_info(&format!("📡 Listening on port {}", self.port));
        ui::print_info("🔄 Bootstrap node is running...");
        ui::print_info("Press Ctrl+C to stop");
        
        // Bootstrap operation - connect to network and start serving
        match thread_safe_context.network.bootstrap().await {
            Ok(_) => {
                ui::print_success("✅ Bootstrap DHT initialized successfully");
            }
            Err(e) => {
                ui::print_warning(&format!("⚠️  Bootstrap warning: {}", e));
                ui::print_info("Continuing as isolated bootstrap node");
            }
        }
        
        // Keep the bootstrap node running
        loop {
            // Check network health periodically
            match thread_safe_context.network.get_connected_peers().await {
                Ok(peers) => {
                    ui::print_info(&format!("📊 Bootstrap node serving {} connected peers", peers.len()));
                }
                Err(e) => {
                    ui::print_warning(&format!("Network check error: {}", e));
                }
            }
            
            // Wait before next health check
            sleep(Duration::from_secs(30)).await;
        }
    }
}

/// Interactive command handler
#[derive(Debug, Clone)]
pub struct InteractiveCommand {
    pub bootstrap_peer: bool,
    pub bootstrap_addr: Option<String>,
    pub port: u16,
}

#[async_trait::async_trait]
impl CommandHandler for InteractiveCommand {
    fn command_name(&self) -> &'static str {
        "interactive"
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        ui::print_header("Interactive Mode");
        
        // Use the actor-based networking system for consistency
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network initialized using actor-based system");
        ui::print_info("Interactive mode is now available with full actor-based networking");
        ui::print_info("Type 'help' for available commands, 'exit' to quit");
        
        // Simple interactive loop
        loop {
            print!("datamesh> ");
            std::io::Write::flush(&mut std::io::stdout())?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            match input {
                "exit" | "quit" => {
                    ui::print_info("Goodbye!");
                    break;
                }
                "help" => {
                    ui::print_info("Available commands:");
                    ui::print_info("  stats     - Show network statistics");
                    ui::print_info("  peers     - Show connected peers");
                    ui::print_info("  health    - Check network health");
                    ui::print_info("  help      - Show this help");
                    ui::print_info("  exit/quit - Exit interactive mode");
                }
                "stats" => {
                    match thread_safe_context.network.get_network_stats().await {
                        Ok(stats) => {
                            ui::print_info(&format!("Connected peers: {}", stats.connected_peers));
                            ui::print_info(&format!("Network events: {}", stats.network_events));
                        }
                        Err(e) => ui::print_error(&format!("Failed to get stats: {}", e)),
                    }
                }
                "peers" => {
                    match thread_safe_context.network.get_connected_peers().await {
                        Ok(peers) => {
                            ui::print_info(&format!("Connected to {} peers", peers.len()));
                            for peer in peers.iter().take(5) {
                                ui::print_info(&format!("  📡 {}", peer));
                            }
                        }
                        Err(e) => ui::print_error(&format!("Failed to get peers: {}", e)),
                    }
                }
                "health" => {
                    ui::print_info("🏥 Checking network health...");
                    match thread_safe_context.network.get_connected_peers().await {
                        Ok(peers) => {
                            if peers.is_empty() {
                                ui::print_warning("⚠️  No peers connected");
                            } else {
                                ui::print_success(&format!("✅ Healthy - {} peers connected", peers.len()));
                            }
                        }
                        Err(e) => ui::print_error(&format!("❌ Network health check failed: {}", e)),
                    }
                }
                "" => continue,
                _ => {
                    ui::print_warning(&format!("Unknown command: '{}'. Type 'help' for available commands.", input));
                }
            }
        }
        
        Ok(())
    }
}

/// Service command handler
#[derive(Debug, Clone)]
pub struct ServiceCommand {
    pub bootstrap_peer: Option<libp2p::PeerId>,
    pub bootstrap_addr: Option<libp2p::Multiaddr>,
    pub port: u16,
    pub timeout: u64,
}

#[async_trait::async_trait]
impl CommandHandler for ServiceCommand {
    fn command_name(&self) -> &'static str {
        "service"
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        use tokio::time::{sleep, Duration, Instant};
        
        ui::print_header("Service Node");
        ui::print_info(&format!("Starting service node on port {}", self.port));
        
        if let Some(addr) = &self.bootstrap_addr {
            ui::print_info(&format!("Bootstrap address: {}", addr));
        }
        if let Some(peer) = &self.bootstrap_peer {
            ui::print_info(&format!("Bootstrap peer: {}", peer));
        }
        ui::print_info(&format!("Timeout: {}s", self.timeout));
        
        // Use the actor-based networking system for consistency
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let mut cli_config = context.cli.clone();
        
        // Set the service configuration
        if let crate::cli::Commands::Service { port, bootstrap_peer, bootstrap_addr, timeout } = &mut cli_config.command {
            *port = self.port;
            *bootstrap_peer = self.bootstrap_peer;
            *bootstrap_addr = self.bootstrap_addr.clone();
            *timeout = Some(self.timeout);
        }
        
        let thread_safe_context = ThreadSafeCommandContext::new(
            cli_config,
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Service node initialized using actor-based system");
        ui::print_info(&format!("📡 Listening on port {}", self.port));
        
        // Connect to bootstrap peer if specified
        if let (Some(peer_id), Some(addr)) = (&self.bootstrap_peer, &self.bootstrap_addr) {
            ui::print_info("🔄 Connecting to bootstrap peer...");
            match thread_safe_context.network.add_peer_address(*peer_id, addr.clone()).await {
                Ok(_) => {
                    ui::print_success("✅ Connected to bootstrap peer");
                }
                Err(e) => {
                    ui::print_warning(&format!("⚠️  Bootstrap connection warning: {}", e));
                }
            }
        }
        
        // Initialize DHT
        match thread_safe_context.network.bootstrap().await {
            Ok(_) => {
                ui::print_success("✅ DHT bootstrap completed");
            }
            Err(e) => {
                ui::print_warning(&format!("⚠️  DHT bootstrap warning: {}", e));
            }
        }
        
        ui::print_info("🚀 Service node is running...");
        ui::print_info("The node will automatically store and serve files");
        
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(self.timeout);
        
        // Service node main loop
        loop {
            // Check if timeout has been reached
            if start_time.elapsed() >= timeout_duration {
                ui::print_info(&format!("⏰ Service timeout reached ({}s), shutting down", self.timeout));
                break;
            }
            
            // Check network health and display status
            match thread_safe_context.network.get_connected_peers().await {
                Ok(peers) => {
                    let remaining_time = timeout_duration.saturating_sub(start_time.elapsed()).as_secs();
                    ui::print_info(&format!(
                        "📊 Service active: {} peers connected, {}s remaining", 
                        peers.len(), 
                        remaining_time
                    ));
                }
                Err(e) => {
                    ui::print_warning(&format!("Network status check error: {}", e));
                }
            }
            
            // Display network statistics
            match thread_safe_context.network.get_network_stats().await {
                Ok(stats) => {
                    ui::print_info(&format!("📈 Network events processed: {}", stats.network_events));
                }
                Err(_) => {} // Ignore stats errors
            }
            
            // Wait before next status check
            sleep(Duration::from_secs(10)).await;
        }
        
        ui::print_success("✅ Service node shutdown complete");
        Ok(())
    }
}
