use anyhow::Result;
/// Actor-based File Command Handlers
///
/// This module contains actor-based handlers for all file-related operations:
/// put, get, list, info, stats
use std::error::Error;
use std::path::PathBuf;

use crate::commands::actor_commands::{ActorCommandContext, ActorCommandHandler};
use crate::database::DatabaseManager;
use crate::error::DfsError;
use crate::ui;

/// Actor-based Put command handler
#[derive(Debug, Clone)]
pub struct ActorPutCommand {
    pub path: PathBuf,
    pub public_key: Option<String>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorPutCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        let result = context
            .context
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
        "actor_file_put"
    }
}

/// Actor-based Get command handler
#[derive(Debug, Clone)]
pub struct ActorGetCommand {
    pub identifier: String,
    pub output_path: PathBuf,
    pub private_key: Option<String>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorGetCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        context
            .context
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
        "actor_file_get"
    }
}

/// Actor-based List command handler
#[derive(Debug, Clone)]
pub struct ActorListCommand {
    pub public_key: Option<String>,
    pub tags: Option<String>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorListCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        // Initialize database
        let db_path = crate::database::get_default_db_path()?;
        let db = DatabaseManager::new(&db_path)?;

        // Parse tag filter
        let tag_filter = self.tags.as_ref().map(|t| t.as_str());

        // Get files from database
        let files = db.list_files(tag_filter)?;

        if let Some(pk) = &self.public_key {
            // Filter by public key if specified
            let filtered_files: Vec<_> = files
                .into_iter()
                .filter(|f| f.public_key_hex == *pk)
                .collect();
            ui::print_file_list(&filtered_files);
        } else {
            // Show all files for this user's default key
            let target_public_key = &context.context.key_manager.key_info.public_key_hex;
            let filtered_files: Vec<_> = files
                .into_iter()
                .filter(|f| f.public_key_hex == *target_public_key)
                .collect();
            ui::print_file_list(&filtered_files);
        }

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "actor_file_list"
    }
}

/// Actor-based Info command handler
#[derive(Debug, Clone)]
pub struct ActorInfoCommand {
    pub identifier: String,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorInfoCommand {
    async fn execute(&self, _context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = DatabaseManager::new(&db_path)?;

        // Find file by name or key
        let stored_file = if self.identifier.len() == 64 {
            // Looks like a file key
            db.get_file_by_key(&self.identifier)?
        } else {
            // Treat as file name
            db.get_file_by_name(&self.identifier)?
        };

        let file = stored_file.ok_or_else(|| DfsError::FileNotFound(self.identifier.clone()))?;

        ui::print_file_info(&file);
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "actor_file_info"
    }
}

/// Actor-based Stats command handler
#[derive(Debug, Clone)]
pub struct ActorStatsCommand {}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorStatsCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        let db_path = crate::database::get_default_db_path()?;
        let db = DatabaseManager::new(&db_path)?;
        let stats = db.get_stats()?;

        ui::print_database_stats(&stats);

        // Also print network stats
        match context.context.get_network_stats().await {
            Ok(network_stats) => {
                println!("\nNetwork Statistics:");
                println!("  Local Peer ID: {}", network_stats.local_peer_id);
                println!("  Connected Peers: {}", network_stats.connected_peers);
                println!("  Pending Queries: {}", network_stats.pending_queries);
                println!("  Routing Table Size: {}", network_stats.routing_table_size);
            }
            Err(e) => {
                ui::print_warning(&format!("Failed to get network stats: {}", e));
            }
        }

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "actor_file_stats"
    }
}
