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
        interactive::run_interactive_mode(
            &context.cli,
            context.key_manager.clone(),
            self.bootstrap_peer,
            self.bootstrap_addr.clone(),
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
        interactive::run_service_mode(
            &context.cli,
            context.key_manager.clone(),
            self.bootstrap_peer,
            self.bootstrap_addr.clone(),
            self.port,
            self.timeout,
        ).await
    }
    
    fn command_name(&self) -> &'static str {
        "service_mode"
    }
}