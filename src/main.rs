mod actor_file_storage;
mod advanced_commands;
mod api_server;
mod audit_logger;
mod batch_operations;
mod billing_system;
mod bootstrap_admin;
mod bootstrap_manager;
mod cli;
mod commands;
mod concurrent_chunks;
mod config;
mod database;
mod datamesh_core;
mod economics;
mod encrypted_key_manager;
mod error;
mod error_handling;
mod failover;
mod file_manager;
mod file_storage;
mod governance;
mod governance_service;
mod health_manager;
mod interactive;
mod key_manager;
mod load_balancer;
mod logging;
mod network;
mod network_actor;
mod network_diagnostics;
mod performance;
mod performance_optimizer;
mod persistent_dht;
mod presets;
mod quota_service;
mod resilience;
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
mod secure_random;
mod smart_cache;
mod thread_safe_command_context;
mod thread_safe_database;
mod thread_safe_file_commands;
mod ui;

use std::error::Error;
use std::sync::Arc;

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

    let key_manager =
        match key_manager::setup_key_management_with_mode(&cli, key_selection_mode).await {
            Ok(km) => Arc::new(km),
            Err(e) => {
                let enhanced_error = error_handling::handle_error(e.as_ref());
                error_handling::display_enhanced_error(&enhanced_error);
                std::process::exit(1);
            }
        };

    crate::ui::print_operation_status(
        "Cryptographic Keys",
        "Ready",
        Some("ECIES encryption initialized"),
    );

    // Initialize performance monitoring
    let _monitor = performance::global_monitor();

    // Execute command using new handler system
    tracing::error!("🔥 main.rs calling execute_command with command: {:?}", cli.command);
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
        if !connection_config.bootstrap_peers.is_empty() {
            // Set the first bootstrap node as the default if not already specified
            if cli.bootstrap_addr.is_none() {
                cli.bootstrap_addr = Some(connection_config.bootstrap_peers[0].address.clone());
                cli.bootstrap_peer = connection_config.bootstrap_peers[0].peer_id.clone();
            }
        }

        // Apply other network-specific configurations
        if connection_config.port != 0 {
            if matches!(
                &cli.command,
                cli::Commands::Bootstrap { port: 0 }
                    | cli::Commands::Interactive { port: 0, .. }
                    | cli::Commands::Service { port: 0, .. }
            ) {
                // Update command with preset port if using default
                match &mut cli.command {
                    cli::Commands::Bootstrap { port: p } if *p == 0 => *p = connection_config.port,
                    cli::Commands::Interactive { port: p, .. } if *p == 0 => {
                        *p = connection_config.port
                    }
                    cli::Commands::Service { port: p, .. } if *p == 0 => {
                        *p = connection_config.port
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
