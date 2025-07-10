/// Network operation command handlers
/// 
/// This module contains handlers for all network-related operations:
/// peers, health, network, discover, distribution, bandwidth

use std::error::Error;
use anyhow::Result;

use crate::commands::{CommandHandler, CommandContext};
use crate::network_diagnostics;

/// Peers command handler
#[derive(Debug, Clone)]
pub struct PeersCommand {
    pub detailed: bool,
    pub format: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for PeersCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_peers_command(
            &context.cli,
            &context.key_manager,
            self.detailed,
            &self.format,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_peers"
    }
}

/// Health command handler
#[derive(Debug, Clone)]
pub struct HealthCommand {
    pub continuous: bool,
    pub interval: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for HealthCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_health_command(
            &context.cli,
            &context.key_manager,
            self.continuous,
            self.interval,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_health"
    }
}

/// Network topology command handler
#[derive(Debug, Clone)]
pub struct NetworkCommand {
    pub depth: Option<u32>,
    pub visualize: bool,
}

#[async_trait::async_trait]
impl CommandHandler for NetworkCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_network_command(
            &context.cli,
            &context.key_manager,
            self.depth,
            self.visualize,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_topology"
    }
}

/// Discover command handler
#[derive(Debug, Clone)]
pub struct DiscoverCommand {
    pub timeout: Option<u64>,
    pub bootstrap_all: bool,
}

#[async_trait::async_trait]
impl CommandHandler for DiscoverCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_discover_command(
            &context.cli,
            &context.key_manager,
            self.timeout,
            self.bootstrap_all,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_discover"
    }
}

/// Distribution command handler
#[derive(Debug, Clone)]
pub struct DistributionCommand {
    pub file_key: Option<String>,
    pub public_key: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for DistributionCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_distribution_command(
            &context.cli,
            &context.key_manager,
            &self.file_key,
            &self.public_key,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_distribution"
    }
}

/// Bandwidth command handler
#[derive(Debug, Clone)]
pub struct BandwidthCommand {
    pub test_peer: Option<String>,
    pub duration: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for BandwidthCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        network_diagnostics::handle_bandwidth_command(
            &context.cli,
            &context.key_manager,
            &self.test_peer,
            self.duration,
        ).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn command_name(&self) -> &'static str {
        "network_bandwidth"
    }
}