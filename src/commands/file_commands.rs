/// File operation command handlers
/// 
/// This module contains handlers for all file-related operations:
/// put, get, list, info, stats

use std::error::Error;
use std::path::PathBuf;
use anyhow::Result;

use crate::commands::{CommandHandler, CommandContext};
use crate::file_storage;

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
        let tags_str = self.tags.as_ref().map(|tags| tags.join(","));
        
        // Clone the necessary data to avoid borrowing issues
        let cli = context.cli.clone();
        let key_manager = (*context.key_manager).clone();
        let path = self.path.clone();
        let public_key = self.public_key.clone();
        let name = self.name.clone();
        
        // Execute directly without spawn_blocking to avoid Send issues
        match file_storage::handle_put_command(
            &cli,
            &key_manager,
            &path,
            &public_key,
            &name,
            &tags_str,
        ).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn Error>),
        }
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
        // Clone the necessary data to avoid borrowing issues
        let cli = context.cli.clone();
        let key_manager = (*context.key_manager).clone();
        let identifier = self.identifier.clone();
        let output_path = self.output_path.clone();
        let private_key = self.private_key.clone();
        
        // Execute directly without spawn_blocking to avoid Send issues
        match file_storage::handle_get_command(
            &cli,
            &key_manager,
            &identifier,
            &output_path,
            &private_key,
        ).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn Error>),
        }
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
        file_storage::handle_list_command(
            &context.key_manager,
            &self.public_key,
            &tags_str,
        ).await
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
        file_storage::handle_info_command(
            &context.key_manager,
            &self.identifier,
        ).await
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
        file_storage::handle_stats_command(&context.key_manager).await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "file_stats"
    }
}