/// Thread-Safe File Commands
///
/// This module provides thread-safe implementations of file commands that use
/// the actor-based network layer instead of sharing Swarm instances directly.
use std::error::Error;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::commands::{CommandContext, CommandHandler};
use crate::config::Config;
use crate::key_manager::KeyManager;
use crate::thread_safe_command_context::ThreadSafeCommandContext;
use std::sync::Arc;

/// Thread-safe wrapper for file commands
#[derive(Debug, Clone)]
pub struct ThreadSafeFileCommands {
    context: ThreadSafeCommandContext,
}

impl ThreadSafeFileCommands {
    /// Create a new thread-safe file commands instance
    pub async fn new(
        cli: Cli,
        key_manager: Arc<KeyManager>,
        config: Arc<Config>,
    ) -> Result<Self, Box<dyn Error>> {
        let context = ThreadSafeCommandContext::new(cli, key_manager, config).await?;
        Ok(ThreadSafeFileCommands { context })
    }

    /// Store a file using thread-safe operations
    pub async fn put_file(
        &self,
        path: &PathBuf,
        public_key: &Option<String>,
        name: &Option<String>,
        tags: &Option<Vec<String>>,
    ) -> Result<String, Box<dyn Error>> {
        self.context
            .store_file(path, public_key, name, tags)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    /// Retrieve a file using thread-safe operations
    pub async fn get_file(
        &self,
        identifier: &str,
        output_path: &PathBuf,
        private_key: &Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.context
            .retrieve_file(identifier, output_path, private_key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

/// Thread-safe Put command
#[derive(Debug, Clone)]
pub struct ThreadSafePutCommand {
    pub path: PathBuf,
    pub public_key: Option<String>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub commands: ThreadSafeFileCommands,
}

impl ThreadSafePutCommand {
    pub async fn new(
        path: PathBuf,
        public_key: Option<String>,
        name: Option<String>,
        tags: Option<Vec<String>>,
        cli: Cli,
        key_manager: Arc<KeyManager>,
        config: Arc<Config>,
    ) -> Result<Self, Box<dyn Error>> {
        let commands = ThreadSafeFileCommands::new(cli, key_manager, config).await?;
        Ok(ThreadSafePutCommand {
            path,
            public_key,
            name,
            tags,
            commands,
        })
    }
}

#[async_trait::async_trait]
impl CommandHandler for ThreadSafePutCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        let result = self
            .commands
            .put_file(&self.path, &self.public_key, &self.name, &self.tags)
            .await?;
        println!("File stored successfully with key: {}", result);
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "thread_safe_put"
    }
}

/// Thread-safe Get command
#[derive(Debug, Clone)]
pub struct ThreadSafeGetCommand {
    pub identifier: String,
    pub output_path: PathBuf,
    pub private_key: Option<String>,
    pub commands: ThreadSafeFileCommands,
}

impl ThreadSafeGetCommand {
    pub async fn new(
        identifier: String,
        output_path: PathBuf,
        private_key: Option<String>,
        cli: Cli,
        key_manager: Arc<KeyManager>,
        config: Arc<Config>,
    ) -> Result<Self, Box<dyn Error>> {
        let commands = ThreadSafeFileCommands::new(cli, key_manager, config).await?;
        Ok(ThreadSafeGetCommand {
            identifier,
            output_path,
            private_key,
            commands,
        })
    }
}

#[async_trait::async_trait]
impl CommandHandler for ThreadSafeGetCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        self.commands
            .get_file(&self.identifier, &self.output_path, &self.private_key)
            .await?;
        println!(
            "File retrieved successfully to: {}",
            self.output_path.display()
        );
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "thread_safe_get"
    }
}
