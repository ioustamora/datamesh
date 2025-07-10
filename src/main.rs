/// DataMesh - Distributed Data Storage System
///
/// This is the main entry point for the DataMesh application, which provides a secure,
/// fault-tolerant distributed data storage system built with Rust and libp2p.
/// 
/// The system features:
/// - ECIES encryption for secure file storage
/// - Reed-Solomon erasure coding for fault tolerance
/// - Kademlia DHT for decentralized storage
/// - BLAKE3 hashing for optimal performance
///
/// The main module has been refactored to use a clean command handler architecture
/// instead of a massive switch statement, improving maintainability and testability.

mod key_manager;
mod encrypted_key_manager;
mod audit_logger;
mod file_storage;
mod network;
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

use std::error::Error;
use std::sync::Arc;


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
    
    // Execute command using new handler system
    if let Err(e) = commands::execute_command(cli, key_manager).await {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    Ok(())
}

/// Apply network presets to CLI configuration
fn apply_network_preset(cli: &mut cli::Cli) -> Result<(), Box<dyn Error>> {
    if let Some(network_spec) = &cli.network {
        let connection_config = presets::parse_network_spec(network_spec)?;
        
        // Apply bootstrap configuration if specified
        if let Some(bootstrap_nodes) = connection_config.bootstrap_nodes {
            if !bootstrap_nodes.is_empty() {
                // Set the first bootstrap node as the default if not already specified
                if cli.bootstrap_addr.is_none() {
                    cli.bootstrap_addr = Some(bootstrap_nodes[0].clone());
                    cli.bootstrap_peer = true;
                }
            }
        }
        
        // Apply other network-specific configurations
        if let Some(port) = connection_config.default_port {
            if matches!(&cli.command, cli::Commands::Bootstrap { port: 0 } | 
                                     cli::Commands::Interactive { port: 0, .. } | 
                                     cli::Commands::Service { port: 0, .. }) {
                // Update command with preset port if using default
                match &mut cli.command {
                    cli::Commands::Bootstrap { port: p } if *p == 0 => *p = port,
                    cli::Commands::Interactive { port: p, .. } if *p == 0 => *p = port,
                    cli::Commands::Service { port: p, .. } if *p == 0 => *p = port,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}