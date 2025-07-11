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
    logging::init_logging();
    
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
    
    // For now, delegate to the old interactive system
    // TODO: Implement full actor-based interactive mode
    ui::print_info("Falling back to traditional interactive mode");
    interactive::run_interactive_mode(
        cli, 
        bootstrap_peer, 
        bootstrap_addr, 
        port, 
        timeout
    ).await?;
    
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
    
    // For now, delegate to the old service system
    // TODO: Implement full actor-based service mode
    ui::print_info("Falling back to traditional service mode");
    interactive::run_service_mode(
        cli, 
        bootstrap_peer, 
        bootstrap_addr, 
        port, 
        timeout
    ).await?;
    
    Ok(())
}
