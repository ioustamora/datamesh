use anyhow::Result;
/// Actor-based Command System
///
/// This module provides command handling using the network actor pattern
/// for thread-safe network operations.
use std::error::Error;
use std::sync::Arc;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::key_manager::KeyManager;
use crate::network_actor::NetworkStats;
use crate::performance;
use crate::thread_safe_command_context::ThreadSafeCommandContext;

/// Actor-based context for command handlers
#[derive(Clone)]
pub struct ActorCommandContext {
    pub context: ThreadSafeCommandContext,
}

impl ActorCommandContext {
    /// Create a new actor-based command context
    pub async fn new(cli: Cli, key_manager: Arc<KeyManager>, config: Arc<Config>) -> Result<Self> {
        let context = ThreadSafeCommandContext::new(cli, key_manager, config).await?;
        Ok(ActorCommandContext { context })
    }
}

/// Trait for actor-based command handlers
#[async_trait::async_trait]
pub trait ActorCommandHandler: Send + Sync {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>>;

    /// Get the command name for performance monitoring
    fn command_name(&self) -> &'static str;

    /// Execute with performance monitoring wrapper
    async fn execute_with_monitoring(
        &self,
        context: &ActorCommandContext,
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

/// Actor-based command dispatcher
pub struct ActorCommandDispatcher {
    context: ActorCommandContext,
}

impl ActorCommandDispatcher {
    /// Create a new actor-based command dispatcher
    pub async fn new(cli: Cli, key_manager: Arc<KeyManager>, config: Arc<Config>) -> Result<Self> {
        let context = ActorCommandContext::new(cli, key_manager, config).await?;
        Ok(ActorCommandDispatcher { context })
    }

    /// Dispatch a command to the appropriate handler
    pub async fn dispatch(&self, command: &Commands) -> Result<(), Box<dyn Error>> {
        match command {
            Commands::Put {
                path,
                public_key,
                name,
                tags,
            } => {
                let handler = crate::commands::actor_file_commands::ActorPutCommand {
                    path: path.clone(),
                    public_key: public_key.clone(),
                    name: name.clone(),
                    tags: tags.clone().map(|t| vec![t]),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Get {
                identifier,
                output_path,
                private_key,
            } => {
                let handler = crate::commands::actor_file_commands::ActorGetCommand {
                    identifier: identifier.clone(),
                    output_path: output_path.clone(),
                    private_key: private_key.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::List { public_key, tags } => {
                let handler = crate::commands::actor_file_commands::ActorListCommand {
                    public_key: public_key.clone(),
                    tags: tags.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Stats => {
                let handler = crate::commands::actor_file_commands::ActorStatsCommand {};
                handler.execute_with_monitoring(&self.context).await
            }

            // Add other commands as needed
            _ => Err("Command not yet implemented in actor system".into()),
        }
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        self.context.context.get_network_stats().await
    }

    /// Bootstrap the network
    pub async fn bootstrap(&self) -> Result<()> {
        self.context.context.bootstrap().await
    }
}
