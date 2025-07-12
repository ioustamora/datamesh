use anyhow::Result;
/// File operation command handlers
///
/// This module contains handlers for all file-related operations:
/// put, get, list, info, stats
use std::error::Error;
use std::path::PathBuf;

use crate::commands::{CommandContext, CommandHandler};
use crate::file_storage;
use crate::thread_safe_command_context::ThreadSafeCommandContext;

/// Put command handler
#[derive(Debug, Clone)]
pub struct PutCommand {
    pub path: PathBuf,
    pub public_key: Option<String>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl CommandHandler for PutCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        tracing::error!("🔥 PutCommand::execute called for: {}", self.path.display());
        // Create thread-safe context to avoid Swarm Send/Sync issues
        // Use the actual configuration instead of default to preserve bootstrap peers
        let config = match crate::config::Config::load_or_default(None) {
            Ok(config) => config,
            Err(_) => {
                // Fallback to default if no config file exists, but preserve CLI bootstrap settings
                crate::config::Config::default()
            }
        };
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            std::sync::Arc::new(config),
        )
        .await?;

        let result = thread_safe_context
            .store_file(&self.path, &self.public_key, &self.name, &self.tags)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        println!("File stored successfully with key: {}", result);
        if let Some(name) = &self.name {
            println!("Name: {}", name);
        }
        if let Some(tags) = &self.tags {
            println!("Tags: {}", tags.join(","));
        }

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "file_put"
    }
}

/// Get command handler
#[derive(Debug, Clone)]
pub struct GetCommand {
    pub identifier: String,
    pub output_path: PathBuf,
    pub private_key: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for GetCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        // Create thread-safe context to avoid Swarm Send/Sync issues
        let config = crate::config::Config::default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            std::sync::Arc::new(config),
        )
        .await?;

        thread_safe_context
            .retrieve_file(&self.identifier, &self.output_path, &self.private_key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;

        println!(
            "File retrieved successfully to: {}",
            self.output_path.display()
        );

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "file_get"
    }
}

/// List command handler
#[derive(Debug, Clone)]
pub struct ListCommand {
    pub public_key: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl CommandHandler for ListCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        let tags_str = self.tags.as_ref().map(|tags| tags.join(","));
        file_storage::handle_list_command(&context.key_manager, &self.public_key, &tags_str)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn command_name(&self) -> &'static str {
        "file_list"
    }
}

/// Info command handler
#[derive(Debug, Clone)]
pub struct InfoCommand {
    pub identifier: String,
}

#[async_trait::async_trait]
impl CommandHandler for InfoCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        file_storage::handle_info_command(&context.key_manager, &self.identifier)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn command_name(&self) -> &'static str {
        "file_info"
    }
}

/// Stats command handler
#[derive(Debug, Clone)]
pub struct StatsCommand;

#[async_trait::async_trait]
impl CommandHandler for StatsCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        file_storage::handle_stats_command(&context.key_manager)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn command_name(&self) -> &'static str {
        "file_stats"
    }
}
