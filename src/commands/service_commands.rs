/// Service operation command handlers
/// 
/// This module contains handlers for service-related operations:
/// bootstrap, interactive, service

use std::error::Error;
use anyhow::Result;

use crate::commands::{CommandHandler, CommandContext};
use crate::{network, interactive, config};

/// Bootstrap command handler
#[derive(Debug, Clone)]
pub struct BootstrapCommand {
    pub port: u16,
}

#[async_trait::async_trait]
impl CommandHandler for BootstrapCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        let config = config::Config::load_or_default(None)?;
        network::start_bootstrap_node(self.port, &config).await
    }
    
    fn command_name(&self) -> &'static str {
        "service_bootstrap"
    }
}

/// Interactive command handler
#[derive(Debug, Clone)]
pub struct InteractiveCommand {
    pub bootstrap_peer: bool,
    pub bootstrap_addr: Option<String>,
    pub port: u16,
}

#[async_trait::async_trait]
impl CommandHandler for InteractiveCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        // For now, convert the parameters to the expected types
        // TODO: Fix the command structure to match the CLI properly
        let bootstrap_peer = if self.bootstrap_peer {
            // We don't have the actual peer ID here, so we'll pass None
            // This needs to be fixed in the CLI integration
            None
        } else {
            None
        };
        
        let bootstrap_addr = if let Some(addr_str) = &self.bootstrap_addr {
            addr_str.parse().ok()
        } else {
            None
        };
        
        // Extract KeyManager from Arc - this is a temporary workaround
        let key_manager = (**context.key_manager).clone();
        
        interactive::run_interactive_mode(
            &context.cli,
            key_manager,
            bootstrap_peer,
            bootstrap_addr,
            self.port,
        ).await
    }
    
    fn command_name(&self) -> &'static str {
        "service_interactive"
    }
}

/// Service command handler
#[derive(Debug, Clone)]
pub struct ServiceCommand {
    pub bootstrap_peer: bool,
    pub bootstrap_addr: Option<String>,
    pub port: u16,
    pub timeout: Option<u64>,
}

#[async_trait::async_trait]
impl CommandHandler for ServiceCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        // For now, convert the parameters to the expected types
        // TODO: Fix the command structure to match the CLI properly
        let bootstrap_peer = if self.bootstrap_peer {
            // We don't have the actual peer ID here, so we'll pass None
            // This needs to be fixed in the CLI integration
            None
        } else {
            None
        };
        
        let bootstrap_addr = if let Some(addr_str) = &self.bootstrap_addr {
            addr_str.parse().ok()
        } else {
            None
        };
        
        // Extract KeyManager from Arc - this is a temporary workaround
        let key_manager = (**context.key_manager).clone();
        
        interactive::run_service_mode(
            &context.cli,
            key_manager,
            bootstrap_peer,
            bootstrap_addr,
            self.port,
            self.timeout,
        ).await
    }
    
    fn command_name(&self) -> &'static str {
        "service_mode"
    }
}