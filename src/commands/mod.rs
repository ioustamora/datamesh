use anyhow::Result;
/// Command handling system for DataMesh
///
/// This module provides a clean architecture for handling CLI commands
/// by extracting the logic from the massive main.rs file.
use std::error::Error;
use std::sync::Arc;

use crate::cli::{Cli, Commands};
use crate::key_manager::KeyManager;
use crate::network_diagnostics::NetworkDiagnostics;
use crate::performance;

pub mod actor_commands;
pub mod actor_file_commands;
pub mod admin_commands;
pub mod advanced_commands;
pub mod file_commands;
pub mod network_commands;
pub mod service_commands;

/// Shared context for all command handlers
#[derive(Debug)]
pub struct CommandContext {
    pub cli: Cli,
    pub key_manager: Arc<KeyManager>,
    pub network_diagnostics: Option<Arc<NetworkDiagnostics>>,
}

impl CommandContext {
    pub fn new(cli: Cli, key_manager: Arc<KeyManager>) -> Self {
        Self {
            cli,
            key_manager,
            network_diagnostics: Some(Arc::new(NetworkDiagnostics::new())),
        }
    }
}

/// Trait for command handlers
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>>;

    /// Get the command name for performance monitoring
    fn command_name(&self) -> &'static str;

    /// Execute with performance monitoring wrapper
    async fn execute_with_monitoring(
        &self,
        context: &CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let timer = performance::global_monitor().start_operation(self.command_name());
        let result = self.execute(context).await;

        match &result {
            Ok(_) => timer.complete_success(None),
            Err(e) => timer.complete_failure(e.to_string()),
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

        // Advanced operations - delegate to existing advanced_commands module
        _ => Box::new(advanced_commands::AdvancedCommandHandler {
            command: command.clone(),
        }),
    }
}

/// Main command dispatcher
pub async fn execute_command(cli: Cli, key_manager: Arc<KeyManager>) -> Result<(), Box<dyn Error>> {
    let context = CommandContext::new(cli.clone(), key_manager);
    let handler = create_command_handler(&cli.command);
    handler.execute_with_monitoring(&context).await
}
