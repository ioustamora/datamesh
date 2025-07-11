use crate::commands::{CommandContext, CommandHandler};
/// Service operation command handlers
///
/// This module contains handlers for all service-related operations:
/// bootstrap, interactive, service
use std::error::Error;
use crate::{config, network};

/// Bootstrap command handler
#[derive(Debug, Clone)]
pub struct BootstrapCommand {
    pub port: u16,
}

#[async_trait::async_trait]
impl CommandHandler for BootstrapCommand {
    fn command_name(&self) -> &'static str {
        "bootstrap"
    }

    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        println!("Starting bootstrap node on port {}", self.port);
        
        // Load config
        let config = config::Config::load_or_default(None)?;
        
        // Start bootstrap node using traditional network module
        network::start_bootstrap_node(self.port, &config).await?;
        
        Ok(())
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
    fn command_name(&self) -> &'static str {
        "interactive"
    }

    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        println!("Interactive command temporarily disabled for refactoring");
        println!("Bootstrap peer: {}", self.bootstrap_peer);
        if let Some(addr) = &self.bootstrap_addr {
            println!("Bootstrap address: {}", addr);
        }
        println!("Port: {}", self.port);
        Ok(())
    }
}

/// Service command handler
#[derive(Debug, Clone)]
pub struct ServiceCommand {
    pub bootstrap_peer: Option<libp2p::PeerId>,
    pub bootstrap_addr: Option<libp2p::Multiaddr>,
    pub port: u16,
    pub timeout: u64,
}

#[async_trait::async_trait]
impl CommandHandler for ServiceCommand {
    fn command_name(&self) -> &'static str {
        "service"
    }

    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        println!("Starting service node on port {}", self.port);
        if let Some(addr) = &self.bootstrap_addr {
            println!("Bootstrap address: {}", addr);
        }
        println!("Timeout: {}s", self.timeout);
        
        // Load config
        let config = config::Config::load_or_default(None)?;
        
        // Start service node that connects to bootstrap
        network::start_service_node(self.port, self.bootstrap_peer, self.bootstrap_addr.clone(), &config).await?;
        
        Ok(())
    }
}
