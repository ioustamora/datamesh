// ===================================================================================================
// DataMesh - Distributed Data Storage System
// ===================================================================================================
//
// A secure, fault-tolerant distributed data storage system built with Rust and libp2p.
//
// ## ARCHITECTURE OVERVIEW
//
// DataMesh is structured around several core architectural patterns:
//
// ### 1. Actor-Based Networking (network_actor.rs)
// - Isolates libp2p Swarm operations in a dedicated thread
// - Uses message-passing for thread-safe network communication
// - Prevents direct Swarm sharing which can cause synchronization issues
//
// ### 2. Thread-Safe Storage Layer (actor_file_storage.rs)
// - Implements file storage operations using the network actor
// - Provides Reed-Solomon erasure coding for fault tolerance (4+2 shards)
// - Uses ECIES encryption for secure data protection
//
// ### 3. Command Handler Pattern (commands/mod.rs)
// - Clean separation of CLI parsing from business logic
// - Each command has its own handler module for maintainability
// - Standardized error handling and response patterns
//
// ### 4. Distributed Hash Table (DHT) Integration
// - Uses Kademlia DHT for decentralized peer discovery and routing
// - Implements intelligent quorum management for optimal storage success
// - Handles bootstrap peer connectivity and network formation
//
// ### 5. Security Layer
// - ECIES (Elliptic Curve Integrated Encryption Scheme) for file encryption
// - BLAKE3 for fast, secure hashing
// - Secure key management with multiple key support
//
// ### 6. Performance Monitoring
// - Real-time operation tracking and timing
// - Resource usage monitoring
// - Network health and connectivity metrics
//
// ===================================================================================================

// Core Storage and File Management
mod actor_file_storage;           // Actor-based file storage with Reed-Solomon erasure coding
mod file_storage;                 // Traditional file storage implementation
mod file_manager;                 // High-level file management operations
mod concurrent_chunks;            // Parallel chunk processing for large files
mod storage_economy;              // Storage economy system with contribution tiers and verification

// Network and P2P Communication
mod network;                      // Core libp2p networking setup and configuration
mod network_actor;                // Actor-based thread-safe network communication
mod network_diagnostics;          // Network health monitoring and diagnostics
mod bootstrap_manager;            // Bootstrap peer management and connectivity
mod persistent_dht;               // Persistent DHT state management

// Command Line Interface and User Interaction
mod cli;                          // Command line argument parsing and validation
mod commands;                     // Command handler implementations
mod interactive;                  // Interactive mode and user interface
mod ui;                           // User interface utilities and progress indicators

// Security and Cryptography
mod key_manager;                  // Cryptographic key management and ECIES operations
mod encrypted_key_manager;        // Enhanced key management with encryption
mod secure_random;                // Cryptographically secure random number generation
mod secure_transport;             // Secure network transport layer

// System Configuration and Management
mod config;                       // Configuration management and defaults
mod presets;                      // Network preset configurations
mod database;                     // SQLite database operations for metadata
mod thread_safe_database;         // Thread-safe database wrapper

// Performance and Monitoring
mod performance;                  // Performance monitoring and metrics collection
mod performance_optimizer;        // Automatic performance optimization
mod monitoring;                   // Comprehensive system monitoring
mod logging;                      // Structured logging and tracing

// Advanced Features and Administration
mod advanced_commands;            // Advanced administrative commands
mod bootstrap_admin;              // Bootstrap node administration
mod api_server;                   // HTTP API server for web interface
mod governance;                   // Decentralized governance system
mod governance_service;           // Governance service implementation

// Reliability and Fault Tolerance
mod error;                        // Core error types and definitions
mod error_handling;               // Enhanced error handling and recovery
mod failover;                     // Automatic failover mechanisms
mod resilience;                   // System resilience and recovery
mod health_manager;               // Node health monitoring and management

// Resource Management
mod quota_service;                // Storage quota management
mod economics;                    // Economic model and incentives
mod billing_system;               // Usage tracking and billing
mod load_balancer;                // Request load balancing

// System Utilities
mod thread_safe_command_context;  // Thread-safe command execution context
mod thread_safe_file_commands;    // Thread-safe file operation wrappers
mod smart_cache;                  // Intelligent caching system
mod audit_logger;                 // Security audit logging
mod backup_system;                // Data backup and recovery
mod batch_operations;             // Batch processing capabilities
mod datamesh_core;                // Core system functionality
mod websocket;                    // WebSocket functionality for real-time updates

use std::error::Error;
use std::sync::Arc;

/// Main entry point for the DataMesh distributed storage system.
/// 
/// This function orchestrates the entire application lifecycle:
/// 1. Initializes structured logging and tracing
/// 2. Parses and validates command line arguments
/// 3. Applies network presets for common configurations
/// 4. Sets up cryptographic key management (ECIES)
/// 5. Initializes performance monitoring
/// 6. Delegates to the command handler system
/// 
/// The main function uses early return patterns for error handling,
/// ensuring that any failure results in a clean shutdown with proper
/// error reporting to the user.
///
/// ## Error Handling Strategy
/// - All errors are wrapped in enhanced error types for better UX
/// - Non-zero exit codes indicate failure states
/// - User-friendly error messages are displayed before exit
///
/// ## Performance Considerations
/// - Uses async/await throughout for non-blocking I/O
/// - Initializes performance monitoring early for complete metrics
/// - Key manager is wrapped in Arc for efficient sharing
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // ===== SYSTEM INITIALIZATION =====
    
    // Initialize structured logging and tracing system
    // This must be done first to capture all subsequent operations
    logging::init_logging_safe();

    // Parse command line arguments using clap
    // This includes validation of argument combinations and required fields
    let mut cli = cli::Cli::parse();

    // ===== NETWORK CONFIGURATION =====
    
    // Apply network presets if specified (--network flag)
    // Presets provide pre-configured bootstrap nodes for common networks
    if let Err(e) = apply_network_preset(&mut cli) {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    // ===== CRYPTOGRAPHIC SETUP =====
    
    // Configure key selection mode based on interactive flag
    // Non-interactive mode is essential for scripting and automation
    let key_selection_mode = if cli.non_interactive {
        key_manager::KeySelectionMode::NonInteractive
    } else {
        key_manager::KeySelectionMode::Interactive
    };

    // Initialize ECIES key management system
    // This handles both key generation and key loading from storage
    // The key manager is wrapped in Arc for efficient sharing across async tasks
    let key_manager =
        match key_manager::setup_key_management_with_mode(&cli, key_selection_mode).await {
            Ok(km) => Arc::new(km),
            Err(e) => {
                let enhanced_error = error_handling::handle_error(e.as_ref());
                error_handling::display_enhanced_error(&enhanced_error);
                std::process::exit(1);
            }
        };

    // Notify user that cryptographic system is ready
    // This provides immediate feedback that security is properly initialized
    crate::ui::print_operation_status(
        "Cryptographic Keys",
        "Ready",
        Some("ECIES encryption initialized"),
    );

    // ===== PERFORMANCE MONITORING =====
    
    // Initialize global performance monitoring
    // The _monitor variable ensures the monitor stays alive for the program duration
    let _monitor = performance::global_monitor();

    // ===== COMMAND EXECUTION =====
    
    // Execute the specified command using the clean handler architecture
    // Each command type has its own specialized handler for maintainability
    tracing::error!("🔥 main.rs calling execute_command with command: {:?}", cli.command);
    if let Err(e) = commands::execute_command(cli, key_manager).await {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    Ok(())
}

/// Apply network presets to CLI configuration.
///
/// Network presets allow users to quickly connect to well-known DataMesh networks
/// without manually specifying bootstrap nodes and configuration details.
///
/// ## Preset Processing Logic
/// 1. Parse the network specification string (e.g., "mainnet", "testnet")
/// 2. Load corresponding bootstrap peer configurations
/// 3. Apply bootstrap addresses and peer IDs if not already specified
/// 4. Override default ports with network-specific ports when appropriate
///
/// ## Example Network Specifications
/// - "mainnet" - Production DataMesh network
/// - "testnet" - Testing network with relaxed validation
/// - "local" - Local development network
///
/// ## Configuration Priority
/// User-specified CLI arguments take precedence over preset configurations.
/// This allows for partial override while still benefiting from presets.
///
/// ## Error Handling
/// Invalid network specifications return descriptive errors to help users
/// choose from available preset options.
///
/// # Arguments
/// * `cli` - Mutable reference to CLI configuration to be modified
///
/// # Returns
/// * `Ok(())` - Configuration successfully applied
/// * `Err(Box<dyn Error>)` - Invalid network specification or configuration error
fn apply_network_preset(cli: &mut cli::Cli) -> Result<(), Box<dyn Error>> {
    // Only process if a network preset was specified via --network flag
    if let Some(network_spec) = &cli.network {
        // Parse the network specification and load corresponding configuration
        // This validates the network name and returns bootstrap peer details
        let connection_config = presets::parse_network_spec(network_spec)?;

        // ===== BOOTSTRAP PEER CONFIGURATION =====
        
        // Apply bootstrap configuration if peers are defined in the preset
        if !connection_config.bootstrap_peers.is_empty() {
            // Only set bootstrap configuration if not explicitly provided by user
            // This preserves user choice while providing convenient defaults
            if cli.bootstrap_addr.is_none() {
                // Use the first bootstrap peer as the primary connection point
                // Additional peers will be discovered through DHT routing
                cli.bootstrap_addr = Some(connection_config.bootstrap_peers[0].address.clone());
                cli.bootstrap_peer = connection_config.bootstrap_peers[0].peer_id.clone();
            }
        }

        // ===== PORT CONFIGURATION =====
        
        // Apply network-specific port configuration when using default ports
        if connection_config.port != 0 {
            // Check if any command is using the default port (0)
            if matches!(
                &cli.command,
                cli::Commands::Bootstrap { port: 0 }
                    | cli::Commands::Interactive { port: 0, .. }
                    | cli::Commands::Service { port: 0, .. }
            ) {
                // Update command with preset port if currently using default
                // This provides network-appropriate default ports
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
