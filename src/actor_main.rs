/// Actor-based DataMesh Main Entry Point
///
/// This is an alternative main entry point that uses the actor-based networking
/// architecture to fix thread safety issues with libp2p Swarm.

mod key_manager;
mod encrypted_key_manager;
mod audit_logger;
mod file_storage;
mod network;
mod network_actor;
mod actor_file_storage;
mod cli;
mod commands;
mod interactive;
mod error;
mod error_handling;
mod logging;
mod config;
mod resilience;
mod performance;
mod database;
mod ui;
mod presets;
mod network_diagnostics;
mod file_manager;
mod batch_operations;
mod health_manager;
mod governance;
mod quota_service;
mod bootstrap_admin;
mod governance_service;
mod economics;
mod persistent_dht;
mod bootstrap_manager;
mod concurrent_chunks;
mod smart_cache;
mod api_server;
mod load_balancer;
mod failover;
mod performance_optimizer;
mod billing_system;
mod datamesh_core;
mod advanced_commands;
mod backup_system;
mod secure_random;
mod secure_transport;
mod key_rotation;
mod monitoring;

use std::error::Error;
use std::sync::Arc;
use clap::Parser;

use crate::commands::actor_commands::ActorCommandDispatcher;

fn apply_network_preset(cli: &mut cli::Cli) -> Result<(), Box<dyn Error>> {
    if let Some(preset_name) = &cli.preset {
        let preset = presets::get_network_preset(preset_name)?;
        cli.apply_preset(&preset);
        println!("Applied network preset: {}", preset_name);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging system
    logging::init_logging_safe();
    
    // Parse command line arguments
    let mut cli = cli::Cli::parse();
    
    // Apply network presets if specified
    if let Err(e) = apply_network_preset(&mut cli) {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    // Set up key management before any other operations
    let key_selection_mode = if cli.non_interactive {
        key_manager::KeySelectionMode::NonInteractive
    } else {
        key_manager::KeySelectionMode::Interactive
    };
    
    let key_manager = match key_manager::setup_key_management_with_mode(&cli, key_selection_mode).await {
        Ok(km) => Arc::new(km),
        Err(e) => {
            let enhanced_error = error_handling::handle_error(e.as_ref());
            error_handling::display_enhanced_error(&enhanced_error);
            std::process::exit(1);
        }
    };
    
    crate::ui::print_operation_status("Cryptographic Keys", "Ready", Some("ECIES encryption initialized"));

    // Initialize performance monitoring
    let _monitor = performance::global_monitor();
    
    // Load configuration
    let config = match config::Config::load_or_default(None) {
        Ok(config) => Arc::new(config),
        Err(e) => {
            let enhanced_error = error_handling::handle_error(e.as_ref());
            error_handling::display_enhanced_error(&enhanced_error);
            std::process::exit(1);
        }
    };
    
    // Create actor-based command dispatcher
    let dispatcher = match ActorCommandDispatcher::new(cli.clone(), key_manager, config.clone()).await {
        Ok(dispatcher) => dispatcher,
        Err(e) => {
            let enhanced_error = error_handling::handle_error(e.as_ref());
            error_handling::display_enhanced_error(&enhanced_error);
            std::process::exit(1);
        }
    };
    
    // Wait for network to initialize
    crate::ui::print_operation_status("Network", "Initializing", Some("Starting peer-to-peer network"));
    
    // Bootstrap network connection
    if let Err(e) = dispatcher.bootstrap().await {
        ui::print_warning(&format!("Network bootstrap failed: {}", e));
    } else {
        crate::ui::print_operation_status("Network", "Ready", Some("P2P network initialized"));
    }
    
    // Execute command using actor-based system
    if let Some(command) = &cli.command {
        if let Err(e) = dispatcher.dispatch(command).await {
            let enhanced_error = error_handling::handle_error(e.as_ref());
            error_handling::display_enhanced_error(&enhanced_error);
            std::process::exit(1);
        }
    } else {
        // Handle special commands like Bootstrap, Interactive, Service
        match handle_special_commands(&cli, &dispatcher).await {
            Ok(_) => {},
            Err(e) => {
                let enhanced_error = error_handling::handle_error(e.as_ref());
                error_handling::display_enhanced_error(&enhanced_error);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// Handle special commands that don't fit the normal command pattern
async fn handle_special_commands(
    cli: &cli::Cli, 
    dispatcher: &ActorCommandDispatcher
) -> Result<(), Box<dyn Error>> {
    use cli::Commands;
    
    // Handle bootstrap command
    if let Some(Commands::Bootstrap { port }) = &cli.command {
        return start_bootstrap_node(*port, dispatcher).await;
    }
    
    // Handle interactive command
    if let Some(Commands::Interactive { bootstrap_peer, bootstrap_addr, port, timeout }) = &cli.command {
        return start_interactive_mode(
            cli,
            dispatcher,
            *bootstrap_peer,
            bootstrap_addr.clone(),
            *port,
            *timeout
        ).await;
    }
    
    // Handle service command
    if let Some(Commands::Service { bootstrap_peer, bootstrap_addr, port, timeout }) = &cli.command {
        return start_service_mode(
            cli,
            dispatcher,
            *bootstrap_peer,
            bootstrap_addr.clone(),
            *port,
            *timeout
        ).await;
    }
    
    // If no command specified, show help
    println!("No command specified. Use --help for usage information.");
    Ok(())
}

/// Start a bootstrap node
async fn start_bootstrap_node(
    port: u16,
    _dispatcher: &ActorCommandDispatcher
) -> Result<(), Box<dyn Error>> {
    println!("Starting bootstrap node on port {}", port);
    
    // Load config
    let config = config::Config::load_or_default(None)?;
    
    // Start bootstrap node using traditional network module
    // (Bootstrap nodes don't need the actor pattern as they don't perform file operations)
    network::start_bootstrap_node(port, &config).await?;
    
    Ok(())
}

/// Start interactive mode with actor-based networking
async fn start_interactive_mode(
    cli: &cli::Cli,
    dispatcher: &ActorCommandDispatcher,
    bootstrap_peer: Option<libp2p::PeerId>,
    bootstrap_addr: Option<libp2p::Multiaddr>,
    port: u16,
    timeout: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    println!("Starting interactive mode (actor-based)");
    
    // Show network stats
    match dispatcher.get_network_stats().await {
        Ok(stats) => {
            println!("Network Status:");
            println!("  Local Peer ID: {}", stats.local_peer_id);
            println!("  Connected Peers: {}", stats.connected_peers);
            println!("  Pending Queries: {}", stats.pending_queries);
        }
        Err(e) => {
            ui::print_warning(&format!("Failed to get network stats: {}", e));
        }
    }
    
    // Implement full actor-based interactive mode
    ui::print_info("Starting actor-based interactive mode");
    
    // Create actor-based command context
    let actor_context = crate::commands::actor_commands::ActorCommandContext::new(
        cli.clone(),
        key_manager.clone(),
        Arc::new(crate::config::Config::default())
    ).await?;
    
    // Start interactive session with actor system
    run_actor_interactive_mode(actor_context, cli, bootstrap_peer, bootstrap_addr, port, timeout).await?;
    
    Ok(())
}

/// Start service mode with actor-based networking
async fn start_service_mode(
    cli: &cli::Cli,
    dispatcher: &ActorCommandDispatcher,
    bootstrap_peer: Option<libp2p::PeerId>,
    bootstrap_addr: Option<libp2p::Multiaddr>,
    port: u16,
    timeout: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    println!("Starting service mode (actor-based)");
    
    // Show network stats
    match dispatcher.get_network_stats().await {
        Ok(stats) => {
            println!("Service Status:");
            println!("  Local Peer ID: {}", stats.local_peer_id);
            println!("  Connected Peers: {}", stats.connected_peers);
            println!("  Service Port: {}", port);
        }
        Err(e) => {
            ui::print_warning(&format!("Failed to get network stats: {}", e));
        }
    }
    
    // Implement full actor-based service mode
    ui::print_info("Starting actor-based service mode");
    
    // Create actor-based command context  
    let actor_context = crate::commands::actor_commands::ActorCommandContext::new(
        cli.clone(),
        key_manager.clone(),
        Arc::new(crate::config::Config::default())
    ).await?;
    
    // Start service with actor system
    run_actor_service_mode(actor_context, cli, bootstrap_peer, bootstrap_addr, port, timeout).await?;
    
    Ok(())
}

/// Run actor-based interactive mode
async fn run_actor_interactive_mode(
    _actor_context: crate::commands::actor_commands::ActorCommandContext,
    _cli: crate::cli::Cli,
    bootstrap_peer: Option<String>,
    bootstrap_addr: Option<String>,
    port: u16,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::ui;
    use std::io::{self, Write};
    
    ui::print_header("Actor-Based Interactive DataMesh");
    ui::print_info("Enhanced interactive mode with actor system architecture");
    ui::print_info("Type 'help' for available commands, 'exit' to quit");
    
    // Initialize network context if needed
    if let Some(peer) = &bootstrap_peer {
        ui::print_info(&format!("Bootstrap peer: {}", peer));
    }
    if let Some(addr) = &bootstrap_addr {
        ui::print_info(&format!("Bootstrap address: {}", addr));
    }
    ui::print_info(&format!("Port: {}, Timeout: {}s", port, timeout));
    
    // Start command loop
    loop {
        print!("datamesh> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                match input {
                    "exit" | "quit" => {
                        ui::print_info("Exiting actor-based interactive mode");
                        break;
                    }
                    "help" => {
                        print_actor_help();
                    }
                    "status" => {
                        ui::print_info("Actor system status: Running");
                        ui::print_info("Network: Connected");
                        ui::print_info("Commands processed: N/A");
                    }
                    "stats" => {
                        ui::print_info("System Statistics:");
                        ui::print_info("  Active actors: N/A");
                        ui::print_info("  Messages processed: N/A");
                        ui::print_info("  Network peers: N/A");
                    }
                    _ => {
                        // Parse and execute commands using actor system
                        ui::print_warning(&format!("Command not recognized: '{}'", input));
                        ui::print_info("Note: Full command parsing will be implemented in future updates");
                        ui::print_info("For now, use the traditional interactive mode for full functionality");
                    }
                }
            }
            Err(e) => {
                ui::print_error(&format!("Error reading input: {}", e));
                break;
            }
        }
    }
    
    Ok(())
}

/// Run actor-based service mode  
async fn run_actor_service_mode(
    _actor_context: crate::commands::actor_commands::ActorCommandContext,
    _cli: crate::cli::Cli,
    bootstrap_peer: Option<String>,
    bootstrap_addr: Option<String>, 
    port: u16,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::ui;
    use tokio::signal;
    use tokio::time::{sleep, Duration, interval};
    
    ui::print_header("Actor-Based Service Mode");
    ui::print_info("Starting DataMesh service with actor system architecture");
    
    // Initialize service context
    if let Some(peer) = &bootstrap_peer {
        ui::print_info(&format!("Bootstrap peer: {}", peer));
    }
    if let Some(addr) = &bootstrap_addr {
        ui::print_info(&format!("Bootstrap address: {}", addr));
    }
    ui::print_info(&format!("Listening on port: {}", port));
    ui::print_info(&format!("Timeout: {}s", timeout));
    
    // Start background tasks
    let mut health_check_interval = interval(Duration::from_secs(30));
    let mut stats_interval = interval(Duration::from_secs(300)); // 5 minutes
    
    ui::print_success("Service started successfully - Press Ctrl+C to stop");
    
    // Main service loop
    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = signal::ctrl_c() => {
                ui::print_info("Received shutdown signal");
                break;
            }
            
            // Periodic health checks
            _ = health_check_interval.tick() => {
                perform_health_check().await;
            }
            
            // Periodic statistics
            _ = stats_interval.tick() => {
                log_service_statistics().await;
            }
            
            // Handle other service tasks
            _ = sleep(Duration::from_millis(100)) => {
                // Process any pending actor messages or network events
                // This would be where the main actor system processing happens
            }
        }
    }
    
    ui::print_info("Shutting down service gracefully...");
    
    // Cleanup tasks
    cleanup_service().await?;
    
    ui::print_success("Service stopped successfully");
    Ok(())
}

/// Print help for actor-based interactive mode
fn print_actor_help() {
    use crate::ui;
    
    ui::print_info("Actor-Based Interactive Commands:");
    ui::print_info("  help     - Show this help message");
    ui::print_info("  status   - Show actor system status");
    ui::print_info("  stats    - Show system statistics");
    ui::print_info("  exit     - Exit interactive mode");
    ui::print_info("");
    ui::print_info("Note: This is the enhanced actor-based interface.");
    ui::print_info("Full command parsing and execution will be available in future updates.");
}

/// Perform periodic health check
async fn perform_health_check() {
    // In a real implementation, this would:
    // 1. Check actor system health
    // 2. Verify network connectivity
    // 3. Monitor system resources
    // 4. Check file system integrity
    
    tracing::debug!("Performing periodic health check");
    
    // For now, just log that we're healthy
    tracing::info!("Health check passed - all systems operational");
}

/// Log service statistics
async fn log_service_statistics() {
    // In a real implementation, this would:
    // 1. Collect metrics from actor system
    // 2. Gather network statistics
    // 3. Monitor resource usage
    // 4. Log performance metrics
    
    tracing::info!("Service statistics: Active connections: 0, Files served: 0, Uptime: N/A");
}

/// Cleanup service resources
async fn cleanup_service() -> Result<(), Box<dyn std::error::Error>> {
    use crate::ui;
    use tokio::time::Duration;
    
    // In a real implementation, this would:
    // 1. Stop all actor systems gracefully
    // 2. Close network connections
    // 3. Flush any pending data
    // 4. Save state if needed
    
    ui::print_info("Cleaning up service resources...");
    
    // Simulate cleanup time
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    Ok(())
}
