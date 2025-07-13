// ===================================================================================================
// Command Handling System - Clean CLI Command Architecture
// ===================================================================================================
//
// This module implements a clean, maintainable command handling architecture that replaces
// the previous massive switch statement in main.rs. The design promotes separation of concerns,
// testability, and extensibility.
//
// ## ARCHITECTURAL BENEFITS
//
// ### 1. Separation of Concerns
// - Each command type has its own module and handler
// - Business logic is separated from CLI parsing
// - Network, file, and admin operations are cleanly separated
// - Each handler focuses on a single responsibility
//
// ### 2. Maintainability Improvements
// - New commands can be added without modifying existing code
// - Each command handler is independently testable
// - Error handling is standardized across all commands
// - Performance monitoring is automatically applied
//
// ### 3. Performance Monitoring Integration
// - All commands are automatically wrapped with performance tracking
// - Operation timing is collected for analysis
// - Success/failure rates are monitored
// - Resource usage patterns can be identified
//
// ### 4. Context Pattern
// - Shared context provides consistent access to system resources
// - Key manager, network diagnostics, and CLI configuration are available
// - Thread-safe sharing through Arc for concurrent operations
//
// ## COMMAND CATEGORIES
//
// ### File Operations (file_commands.rs)
// - Put: Store files in the distributed storage system
// - Get: Retrieve files by identifier
// - List: Browse stored files with filtering
// - Info: Get metadata about specific files
// - Stats: Display storage statistics and health
//
// ### Network Operations (network_commands.rs)
// - Peers: Display connected peers and network topology
// - Health: Monitor network health and connectivity
// - Network: Analyze network structure and routing
// - Discover: Find and connect to new peers
// - Distribution: Analyze file distribution across the network
// - Bandwidth: Test network performance between peers
//
// ### Service Operations (service_commands.rs)
// - Bootstrap: Start a bootstrap node for network formation
// - Interactive: Enter interactive mode for manual operations
// - Service: Run as a background service node
//
// ### Admin Operations (admin_commands.rs)
// - Config: Manage system configuration
// - Metrics: Export performance and usage metrics
// - Networks: List available network presets
//
// ### Advanced Operations (advanced_commands.rs)
// - Governance operations
// - Admin functions
// - Experimental features
//
// ## ERROR HANDLING STRATEGY
//
// All command handlers return Result<(), Box<dyn Error>> providing:
// - Consistent error propagation
// - Detailed error messages for troubleshooting
// - Proper resource cleanup on failures
// - User-friendly error reporting
//
// ===================================================================================================

use anyhow::Result;
use std::error::Error;
use std::sync::Arc;

use crate::cli::{Cli, Commands};
use crate::key_manager::KeyManager;
use crate::network_diagnostics::NetworkDiagnostics;
use crate::performance;

// Command handler modules organized by functional area
pub mod actor_commands;      // Actor-based command implementations
pub mod actor_file_commands; // Actor-based file operations
pub mod admin_commands;      // Administrative and configuration commands
pub mod advanced_commands;   // Advanced features and experimental functionality
pub mod economy_command;     // Storage economy management commands
pub mod file_commands;       // Core file storage and retrieval operations
pub mod missing_commands;    // Implementation of missing critical commands
pub mod network_commands;    // Network monitoring and management operations
pub mod service_commands;    // Node service operations (bootstrap, interactive, service)

/// Shared context for all command handlers providing access to system resources.
///
/// The CommandContext follows the dependency injection pattern, providing command
/// handlers with access to all necessary system resources without requiring them
/// to create or manage these resources directly.
///
/// ## Design Benefits
/// - **Consistent Access**: All handlers get the same interface to system resources
/// - **Testability**: Context can be mocked for unit testing
/// - **Resource Management**: Centralized management of expensive resources
/// - **Thread Safety**: Arc-wrapped resources can be safely shared across threads
///
/// ## Resource Lifecycle
/// The context maintains shared ownership of resources through Arc smart pointers,
/// ensuring that resources remain available for the duration of command execution
/// while allowing efficient sharing across concurrent operations.
#[derive(Debug)]
pub struct CommandContext {
    pub cli: Cli,                                           // Command line configuration and arguments
    pub key_manager: Arc<KeyManager>,                       // Cryptographic key management system
    pub network_diagnostics: Option<Arc<NetworkDiagnostics>>, // Network health monitoring tools
}

impl CommandContext {
    /// Create a new command context with the specified CLI and key manager.
    ///
    /// This constructor initializes the context with essential system resources
    /// and sets up optional diagnostics capabilities for network monitoring.
    ///
    /// # Arguments
    /// * `cli` - Parsed command line interface configuration
    /// * `key_manager` - Thread-safe key management system
    ///
    /// # Returns
    /// Fully initialized command context ready for handler execution
    pub fn new(cli: Cli, key_manager: Arc<KeyManager>) -> Self {
        Self {
            cli,
            key_manager,
            // Initialize network diagnostics for monitoring and troubleshooting
            network_diagnostics: Some(Arc::new(NetworkDiagnostics::new())),
        }
    }
}

/// Trait defining the interface for all command handlers.
///
/// This trait establishes a consistent contract for command execution across
/// all command types, ensuring uniform behavior for error handling, performance
/// monitoring, and resource management.
///
/// ## Design Principles
/// - **Async by Default**: All command operations are async for non-blocking execution
/// - **Error Propagation**: Standardized error handling with detailed error information
/// - **Performance Monitoring**: Automatic timing and success/failure tracking
/// - **Thread Safety**: Send + Sync bounds enable safe concurrent execution
///
/// ## Implementation Requirements
/// Implementers must provide:
/// 1. `execute()` - Core command logic
/// 2. `command_name()` - Identifier for monitoring and logging
///
/// The trait provides a default implementation of `execute_with_monitoring()`
/// that wraps the core execution with performance tracking.
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute the command with the provided context.
    ///
    /// This is the core method that contains the command's business logic.
    /// Implementers should focus on the command-specific functionality
    /// without worrying about performance monitoring or error standardization.
    ///
    /// # Arguments
    /// * `context` - Shared context providing access to system resources
    ///
    /// # Returns
    /// * `Ok(())` - Command executed successfully
    /// * `Err(Box<dyn Error>)` - Command failed with detailed error information
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>>;

    /// Get the command name for performance monitoring and logging.
    ///
    /// This identifier is used for:
    /// - Performance metrics collection
    /// - Log message tagging
    /// - Error reporting and debugging
    /// - Usage analytics
    ///
    /// # Returns
    /// Static string identifier for this command type
    fn command_name(&self) -> &'static str;

    /// Execute the command with automatic performance monitoring.
    ///
    /// This method wraps the core `execute()` method with performance tracking,
    /// measuring execution time and recording success/failure metrics. It provides
    /// a consistent monitoring interface across all command types.
    ///
    /// ## Monitoring Features
    /// - **Execution Timing**: Measures total command execution time
    /// - **Success Tracking**: Records successful command completions
    /// - **Failure Analysis**: Captures error information for troubleshooting
    /// - **Resource Usage**: Integrates with system-wide performance monitoring
    ///
    /// # Arguments
    /// * `context` - Shared context providing access to system resources
    ///
    /// # Returns
    /// Same as `execute()` but with automatic performance tracking
    async fn execute_with_monitoring(
        &self,
        context: &CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        // Start performance timer for this command execution
        let timer = performance::global_monitor().start_operation(self.command_name());
        
        // Execute the core command logic
        let result = self.execute(context).await;

        // Record performance metrics based on execution outcome
        match &result {
            Ok(_) => {
                // Command succeeded - record success metrics
                timer.complete_success(None)
            }
            Err(e) => {
                // Command failed - record failure metrics with error details
                timer.complete_failure(e.to_string())
            }
        }

        result
    }
}

/// Create a command handler from CLI commands
pub fn create_command_handler(command: &Commands) -> Box<dyn CommandHandler> {
    match command {
        // File operations
        Commands::Put {
            path,
            public_key,
            name,
            tags,
        } => Box::new(file_commands::PutCommand {
            path: path.clone(),
            public_key: public_key.clone(),
            name: name.clone(),
            tags: tags.as_ref().map(|t| vec![t.clone()]),
        }),
        Commands::Get {
            identifier,
            output_path,
            private_key,
        } => Box::new(file_commands::GetCommand {
            identifier: identifier.clone(),
            output_path: output_path.clone(),
            private_key: private_key.clone(),
        }),
        Commands::List { public_key, tags } => Box::new(file_commands::ListCommand {
            public_key: public_key.clone(),
            tags: tags.as_ref().map(|t| vec![t.clone()]),
        }),
        Commands::Info { identifier } => Box::new(file_commands::InfoCommand {
            identifier: identifier.clone(),
        }),
        Commands::Stats => Box::new(file_commands::StatsCommand),

        // Network operations
        Commands::Peers { detailed, format } => Box::new(network_commands::PeersCommand {
            detailed: *detailed,
            format: Some(format!("{:?}", format)),
        }),
        Commands::Health {
            continuous,
            interval,
        } => Box::new(network_commands::HealthCommand {
            continuous: *continuous,
            interval: Some(*interval),
        }),
        Commands::Network { depth, visualize } => Box::new(network_commands::NetworkCommand {
            depth: Some(*depth),
            visualize: *visualize,
        }),
        Commands::Discover {
            timeout,
            bootstrap_all,
        } => Box::new(network_commands::DiscoverCommand {
            timeout: Some(*timeout),
            bootstrap_all: *bootstrap_all,
        }),
        Commands::Distribution {
            file_key,
            public_key,
        } => Box::new(network_commands::DistributionCommand {
            file_key: file_key.clone(),
            public_key: public_key.clone(),
        }),
        Commands::Bandwidth {
            test_peer,
            duration,
        } => Box::new(network_commands::BandwidthCommand {
            test_peer: test_peer.clone(),
            duration: Some(*duration),
        }),

        // Service operations
        Commands::Bootstrap { port } => {
            Box::new(service_commands::BootstrapCommand { port: *port })
        }
        Commands::Interactive {
            bootstrap_peer,
            bootstrap_addr,
            port,
        } => Box::new(service_commands::InteractiveCommand {
            bootstrap_peer: bootstrap_peer.is_some(),
            bootstrap_addr: bootstrap_addr.as_ref().map(|addr| addr.to_string()),
            port: *port,
        }),
        Commands::Service {
            bootstrap_peer,
            bootstrap_addr,
            port,
            timeout,
        } => Box::new(service_commands::ServiceCommand {
            bootstrap_peer: *bootstrap_peer,
            bootstrap_addr: bootstrap_addr.clone(),
            port: *port,
            timeout: timeout.unwrap_or(60),
        }),

        // Admin operations
        Commands::Config {
            generate,
            config_path,
        } => Box::new(admin_commands::ConfigCommand {
            generate: *generate,
            config_path: config_path.clone(),
        }),
        Commands::Metrics { summary, export } => Box::new(admin_commands::MetricsCommand {
            summary: *summary,
            export: *export,
        }),
        Commands::Networks => Box::new(admin_commands::NetworksCommand),

        // Missing critical commands - now implemented
        Commands::Sync {
            local_dir,
            watch,
            bidirectional,
            exclude,
            parallel,
        } => Box::new(missing_commands::SyncCommand {
            local_dir: local_dir.clone(),
            watch: *watch,
            bidirectional: *bidirectional,
            exclude: exclude.clone(),
            parallel: *parallel,
        }),

        Commands::Backup {
            source,
            name,
            incremental,
            compress,
            schedule,
            exclude,
        } => Box::new(missing_commands::BackupCommand {
            source: source.clone(),
            name: name.clone(),
            incremental: *incremental,
            compress: *compress,
            schedule: schedule.clone(),
            exclude: exclude.clone(),
        }),

        Commands::Restore {
            backup_name,
            destination,
            version,
            verify,
            list_versions,
        } => Box::new(missing_commands::RestoreCommand {
            backup_name: backup_name.clone(),
            destination: destination.clone(),
            version: *version,
            verify: *verify,
            list_versions: *list_versions,
        }),

        Commands::Search {
            query,
            file_type,
            size,
            date,
            regex,
            limit,
        } => Box::new(missing_commands::SearchCommand {
            query: query.clone(),
            file_type: file_type.clone(),
            size: size.clone(),
            date: date.clone(),
            regex: *regex,
            limit: *limit,
        }),

        Commands::Cleanup {
            orphaned,
            duplicates,
            low_health,
            dry_run,
            force,
        } => Box::new(missing_commands::CleanupCommand {
            orphaned: *orphaned,
            duplicates: *duplicates,
            low_health: *low_health,
            dry_run: *dry_run,
            force: *force,
        }),

        Commands::Repair {
            target,
            auto,
            verify_all,
            threshold,
        } => Box::new(missing_commands::RepairCommand {
            target: target.clone(),
            auto: *auto,
            verify_all: *verify_all,
            threshold: *threshold,
        }),

        Commands::ApiServer {
            host,
            port,
            https,
            cert_path,
            key_path,
            no_swagger,
        } => Box::new(missing_commands::ApiServerCommand {
            host: host.clone(),
            port: *port,
            https: *https,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            no_swagger: *no_swagger,
        }),

        Commands::Duplicate {
            source,
            new_name,
            new_tags,
        } => Box::new(missing_commands::DuplicateCommand {
            source: source.clone(),
            new_name: new_name.clone(),
            new_tags: new_tags.clone(),
        }),

        Commands::Rename { old_name, new_name } => Box::new(missing_commands::RenameCommand {
            old_name: old_name.clone(),
            new_name: new_name.clone(),
        }),

        Commands::Recent {
            count,
            days,
            file_type,
        } => Box::new(missing_commands::RecentCommand {
            count: *count,
            days: *days,
            file_type: file_type.clone(),
        }),

        Commands::Popular { timeframe, count } => Box::new(missing_commands::PopularCommand {
            timeframe: timeframe.clone(),
            count: *count,
        }),

        Commands::BatchPut {
            pattern,
            recursive,
            parallel,
            base_dir,
            tag_pattern,
        } => Box::new(missing_commands::BatchPutCommand {
            pattern: pattern.clone(),
            recursive: *recursive,
            parallel: *parallel,
            base_dir: base_dir.clone(),
            tag_pattern: tag_pattern.clone(),
        }),

        Commands::BatchGet {
            pattern,
            destination,
            parallel,
            preserve_structure,
        } => Box::new(missing_commands::BatchGetCommand {
            pattern: pattern.clone(),
            destination: destination.clone(),
            parallel: *parallel,
            preserve_structure: *preserve_structure,
        }),

        Commands::BatchTag {
            pattern,
            add_tags,
            remove_tags,
            dry_run,
        } => Box::new(missing_commands::BatchTagCommand {
            pattern: pattern.clone(),
            add_tags: add_tags.clone(),
            remove_tags: remove_tags.clone(),
            dry_run: *dry_run,
        }),

        Commands::Quota { usage, limit, warn, economy, tier } => Box::new(missing_commands::QuotaCommand {
            usage: *usage,
            limit: limit.clone(),
            warn: *warn,
            economy: *economy,
            tier: *tier,
        }),

        Commands::Economy { 
            contribute, 
            path, 
            amount, 
            upgrade, 
            premium_size, 
            payment_method, 
            duration, 
            verify, 
            challenge_response, 
            challenge_id, 
            reputation,
            tier_info,
            contribution_stats,
            rewards,
            upgrade_options,
            verification_history,
            enable_monitoring,
            disable_monitoring,
            test_challenge,
            proof_details,
        } => Box::new(economy_command::EconomyCommand {
            contribute: *contribute,
            path: path.clone(),
            amount: amount.clone(),
            upgrade: *upgrade,
            premium_size: premium_size.clone(),
            payment_method: payment_method.clone(),
            duration: *duration,
            verify: *verify,
            challenge_response: challenge_response.clone(),
            challenge_id: challenge_id.clone(),
            reputation: *reputation,
            tier_info: *tier_info,
            contribution_stats: *contribution_stats,
            rewards: *rewards,
            upgrade_options: *upgrade_options,
            verification_history: *verification_history,
            enable_monitoring: *enable_monitoring,
            disable_monitoring: *disable_monitoring,
            test_challenge: *test_challenge,
            proof_details: *proof_details,
        }),

        Commands::Export {
            destination,
            format,
            encrypt,
            include_metadata,
            pattern,
        } => Box::new(missing_commands::ExportCommand {
            destination: destination.clone(),
            format: format.clone(),
            encrypt: *encrypt,
            include_metadata: *include_metadata,
            pattern: pattern.clone(),
        }),

        Commands::Import {
            archive,
            verify,
            preserve_structure,
            tag_prefix,
        } => Box::new(missing_commands::ImportCommand {
            archive: archive.clone(),
            verify: *verify,
            preserve_structure: *preserve_structure,
            tag_prefix: tag_prefix.clone(),
        }),

        Commands::Pin {
            target,
            priority,
            duration,
        } => Box::new(missing_commands::PinCommand {
            target: target.clone(),
            priority: Some(*priority),
            duration: duration.clone(),
        }),

        Commands::Unpin {
            target,
        } => Box::new(missing_commands::UnpinCommand {
            target: target.clone(),
            pin_id: None,
            all: false,
        }),

        Commands::Share {
            target,
            expires,
            password,
            public,
            qr_code,
        } => Box::new(missing_commands::ShareCommand {
            target: target.clone(),
            expires: expires.clone(),
            max_downloads: None,
            password: password.clone(),
            public: *public,
        }),

        Commands::Optimize {
            defrag,
            rebalance,
            compress,
            analyze,
        } => Box::new(missing_commands::OptimizeCommand {
            storage: false,
            network: false,
            defrag: *defrag,
            rebalance: *rebalance,
            dry_run: false,
        }),

        Commands::Benchmark {
            full,
            network,
            storage,
            duration,
        } => Box::new(missing_commands::BenchmarkCommand {
            test_type: if *full { "full".to_string() } else if *network { "network".to_string() } else if *storage { "storage".to_string() } else { "upload".to_string() },
            duration: Some(*duration),
            file_size: None,
            concurrent: None,
            iterations: None,
        }),

        // Shell completion generation
        Commands::GenerateCompletion { shell } => Box::new(admin_commands::CompletionCommand {
            shell: *shell,
        }),

        // Help/shortcuts command
        Commands::Help => Box::new(admin_commands::HelpCommand),

        // Advanced operations - delegate to existing advanced_commands module
        _ => Box::new(advanced_commands::AdvancedCommandHandler {
            command: command.clone(),
        }),
    }
}

/// Main command dispatcher
pub async fn execute_command(cli: Cli, key_manager: Arc<KeyManager>) -> Result<(), Box<dyn Error>> {
    tracing::error!("🔥 execute_command called with: {:?}", cli.command);
    let context = CommandContext::new(cli.clone(), key_manager);
    tracing::error!("🔥 context created, calling create_command_handler");
    let handler = create_command_handler(&cli.command);
    tracing::error!("🔥 handler created, calling execute_with_monitoring");
    handler.execute_with_monitoring(&context).await
}
