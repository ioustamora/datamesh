/// Setup Wizard Module
///
/// This module provides an interactive setup wizard that guides users through
/// configuring and starting a DataMesh node. It automatically launches when
/// DataMesh is started without arguments and provides a smooth onboarding experience.
use anyhow::Result;
use colored::*;
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::io::BufReader;

use crate::cli::Cli;
use crate::config::Config;
use crate::interactive::run_interactive_mode;
use crate::key_manager::{get_default_keys_dir, KeyManager};
use crate::ui;

/// Node configuration options
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_type: NodeType,
    pub port: u16,
    pub bootstrap_peers: Vec<BootstrapPeer>,
    pub network_preset: Option<String>,
    pub keys_dir: Option<PathBuf>,
    pub key_name: Option<String>,
}

/// Types of nodes that can be configured
#[derive(Debug, Clone)]
pub enum NodeType {
    Bootstrap { port: u16 },
    Regular { connect_to_bootstrap: bool },
    Service { timeout: Option<u64> },
}

/// Bootstrap peer configuration
#[derive(Debug, Clone)]
pub struct BootstrapPeer {
    pub peer_id: PeerId,
    pub address: Multiaddr,
}

/// Interactive setup wizard
pub struct SetupWizard {
    config: NodeConfig,
    available_networks: HashMap<String, String>,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new() -> Self {
        let mut available_networks = HashMap::new();
        available_networks.insert("local".to_string(), "Local development network".to_string());
        available_networks.insert("public".to_string(), "Public DataMesh network".to_string());
        available_networks.insert("test".to_string(), "Test network for experiments".to_string());
        
        Self {
            config: NodeConfig {
                node_type: NodeType::Regular { connect_to_bootstrap: true },
                port: 0,
                bootstrap_peers: Vec::new(),
                network_preset: None,
                keys_dir: None,
                key_name: None,
            },
            available_networks,
        }
    }

    /// Run the interactive setup wizard
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        
        // Step 1: Node type selection
        self.configure_node_type().await?;
        
        // Step 2: Network configuration
        self.configure_network().await?;
        
        // Step 3: Key management
        self.configure_keys().await?;
        
        // Step 4: Show configuration summary
        self.show_configuration_summary();
        
        // Step 5: Start the node
        self.start_node().await?;
        
        Ok(())
    }

    /// Print welcome message
    fn print_welcome(&self) {
        ui::print_header("🚀 DataMesh Setup Wizard");
        
        println!("Welcome to DataMesh! This wizard will guide you through setting up your node.\n");
        
        ui::print_section("📋 What we'll configure:");
        println!("  • Node type (Bootstrap, Regular, or Service)");
        println!("  • Network connection and peers");
        println!("  • Encryption keys and security");
        println!("  • Port and connectivity options");
        
        ui::print_separator();
        println!("Press Enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    /// Configure node type
    async fn configure_node_type(&mut self) -> Result<()> {
        ui::print_section("🔧 Node Configuration");
        
        loop {
            println!("\nWhat type of node would you like to run?");
            println!("  {} - Bootstrap node (helps other nodes connect)", "1".green().bold());
            println!("  {} - Regular node (connect to existing network)", "2".green().bold());
            println!("  {} - Service node (runs in background)", "3".green().bold());
            
            print!("\nEnter your choice (1-3): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => {
                    self.config.node_type = NodeType::Bootstrap { port: 40871 };
                    println!("✅ Bootstrap node selected");
                    break;
                }
                "2" => {
                    self.config.node_type = NodeType::Regular { connect_to_bootstrap: true };
                    println!("✅ Regular node selected");
                    break;
                }
                "3" => {
                    self.config.node_type = NodeType::Service { timeout: None };
                    println!("✅ Service node selected");
                    break;
                }
                _ => {
                    println!("❌ Invalid choice. Please enter 1, 2, or 3.");
                }
            }
        }
        
        // Configure port
        self.configure_port().await?;
        
        Ok(())
    }

    /// Configure network port
    async fn configure_port(&mut self) -> Result<()> {
        println!("\n📡 Port Configuration");
        
        let default_port = match self.config.node_type {
            NodeType::Bootstrap { .. } => 40871,
            _ => 0,
        };
        
        loop {
            if default_port == 0 {
                println!("Enter port number (0 for automatic selection): ");
            } else {
                println!("Enter port number (default: {}): ", default_port);
            }
            
            print!("Port: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let port_str = input.trim();
            if port_str.is_empty() {
                self.config.port = default_port;
                break;
            }
            
            match port_str.parse::<u16>() {
                Ok(port) => {
                    if port > 0 && port <= 65535 {
                        self.config.port = port;
                        break;
                    } else {
                        println!("❌ Port must be between 1 and 65535");
                    }
                }
                Err(_) => {
                    println!("❌ Invalid port number");
                }
            }
        }
        
        if self.config.port == 0 {
            println!("✅ Automatic port selection enabled");
        } else {
            println!("✅ Port {} selected", self.config.port);
        }
        
        Ok(())
    }

    /// Configure network settings
    async fn configure_network(&mut self) -> Result<()> {
        match self.config.node_type {
            NodeType::Bootstrap { .. } => {
                println!("\n🌐 Bootstrap nodes don't need network configuration");
                println!("   Other nodes will connect to you!");
                return Ok(());
            }
            _ => {}
        }
        
        ui::print_section("🌐 Network Configuration");
        
        // Choose network preset
        self.choose_network_preset().await?;
        
        // Configure bootstrap peers if needed
        if matches!(self.config.node_type, NodeType::Regular { connect_to_bootstrap: true }) {
            self.configure_bootstrap_peers().await?;
        }
        
        Ok(())
    }

    /// Choose network preset
    async fn choose_network_preset(&mut self) -> Result<()> {
        println!("\nChoose a network preset:");
        
        let mut options: Vec<_> = self.available_networks.iter().collect();
        options.sort_by_key(|(k, _)| *k);
        
        for (i, (network, description)) in options.iter().enumerate() {
            println!("  {} - {} ({})", 
                (i + 1).to_string().green().bold(), 
                network, 
                description);
        }
        println!("  {} - Custom configuration", 
            (options.len() + 1).to_string().green().bold());
        
        loop {
            print!("\nEnter your choice (1-{}): ", options.len() + 1);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim().parse::<usize>() {
                Ok(choice) if choice > 0 && choice <= options.len() => {
                    let (network, _) = options[choice - 1];
                    self.config.network_preset = Some(network.clone());
                    println!("✅ {} network selected", network);
                    break;
                }
                Ok(choice) if choice == options.len() + 1 => {
                    println!("✅ Custom configuration selected");
                    break;
                }
                _ => {
                    println!("❌ Invalid choice. Please enter a number between 1 and {}", options.len() + 1);
                }
            }
        }
        
        Ok(())
    }

    /// Configure bootstrap peers
    async fn configure_bootstrap_peers(&mut self) -> Result<()> {
        println!("\n🔗 Bootstrap Peer Configuration");
        
        if self.config.network_preset.is_some() {
            println!("Using default bootstrap peers for selected network preset.");
            return Ok(());
        }
        
        println!("Enter bootstrap peers (format: peer_id@address)");
        println!("Press Enter without input to finish");
        
        loop {
            print!("Bootstrap peer: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let input = input.trim();
            if input.is_empty() {
                break;
            }
            
            match self.parse_bootstrap_peer(input) {
                Ok(peer) => {
                    self.config.bootstrap_peers.push(peer);
                    println!("✅ Bootstrap peer added");
                }
                Err(e) => {
                    println!("❌ Invalid bootstrap peer format: {}", e);
                    println!("   Expected format: peer_id@address");
                }
            }
        }
        
        if self.config.bootstrap_peers.is_empty() {
            println!("⚠️  No bootstrap peers configured. Node will start in isolated mode.");
        }
        
        Ok(())
    }

    /// Parse bootstrap peer string
    fn parse_bootstrap_peer(&self, input: &str) -> Result<BootstrapPeer> {
        let parts: Vec<&str> = input.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid format"));
        }
        
        let peer_id = PeerId::from_str(parts[0])?;
        let address = Multiaddr::from_str(parts[1])?;
        
        Ok(BootstrapPeer { peer_id, address })
    }

    /// Configure keys
    async fn configure_keys(&mut self) -> Result<()> {
        ui::print_section("🔐 Key Management");
        
        let keys_dir = get_default_keys_dir()?;
        self.config.keys_dir = Some(keys_dir.clone());
        
        println!("Keys directory: {}", keys_dir.display());
        
        // List available keys
        match KeyManager::list_keys(&keys_dir) {
            Ok(keys) if !keys.is_empty() => {
                println!("\nAvailable keys:");
                for (i, key_name) in keys.iter().enumerate() {
                    if let Ok(info) = KeyManager::get_key_info(&keys_dir, key_name) {
                        println!("  {} - {} (created: {})", 
                            (i + 1).to_string().green().bold(),
                            info.name,
                            info.created.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
                
                println!("  {} - Generate new key", 
                    (keys.len() + 1).to_string().green().bold());
                
                loop {
                    print!("\nSelect key (1-{}): ", keys.len() + 1);
                    io::stdout().flush()?;
                    
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    
                    match input.trim().parse::<usize>() {
                        Ok(choice) if choice > 0 && choice <= keys.len() => {
                            self.config.key_name = Some(keys[choice - 1].clone());
                            println!("✅ Key '{}' selected", keys[choice - 1]);
                            break;
                        }
                        Ok(choice) if choice == keys.len() + 1 => {
                            self.generate_new_key().await?;
                            break;
                        }
                        _ => {
                            println!("❌ Invalid choice. Please enter a number between 1 and {}", keys.len() + 1);
                        }
                    }
                }
            }
            _ => {
                println!("No existing keys found. Generating new key...");
                self.generate_new_key().await?;
            }
        }
        
        Ok(())
    }

    /// Generate new key
    async fn generate_new_key(&mut self) -> Result<()> {
        print!("Enter name for new key (default: datamesh-key): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let key_name = input.trim();
        let key_name = if key_name.is_empty() {
            "datamesh-key".to_string()
        } else {
            key_name.to_string()
        };
        
        let keys_dir = self.config.keys_dir.as_ref().unwrap();
        match KeyManager::generate_key(keys_dir, &key_name) {
            Ok(key_manager) => {
                self.config.key_name = Some(key_name.clone());
                println!("✅ New key '{}' generated successfully", key_name);
                println!("Public key: {}", key_manager.key_info.public_key_hex);
            }
            Err(e) => {
                println!("❌ Failed to generate key: {}", e);
                return Err(e.into());
            }
        }
        
        Ok(())
    }

    /// Show configuration summary
    fn show_configuration_summary(&self) {
        ui::print_section("📋 Configuration Summary");
        
        // Node type
        let node_type_str = match &self.config.node_type {
            NodeType::Bootstrap { port } => format!("Bootstrap (port: {})", port),
            NodeType::Regular { connect_to_bootstrap } => {
                if *connect_to_bootstrap {
                    "Regular (will connect to bootstrap)".to_string()
                } else {
                    "Regular (isolated mode)".to_string()
                }
            }
            NodeType::Service { timeout } => {
                if let Some(timeout) = timeout {
                    format!("Service (timeout: {}s)", timeout)
                } else {
                    "Service (no timeout)".to_string()
                }
            }
        };
        
        ui::print_key_value("Node Type", &node_type_str);
        ui::print_key_value("Port", &self.config.port.to_string());
        
        if let Some(ref network) = self.config.network_preset {
            ui::print_key_value("Network", network);
        }
        
        if !self.config.bootstrap_peers.is_empty() {
            println!("\n📡 Bootstrap Peers:");
            for peer in &self.config.bootstrap_peers {
                println!("  • {}@{}", peer.peer_id, peer.address);
            }
        }
        
        if let Some(ref key_name) = self.config.key_name {
            ui::print_key_value("Key", key_name);
        }
        
        ui::print_separator();
        println!("Press Enter to start the node...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    /// Start the configured node
    async fn start_node(&self) -> Result<()> {
        ui::print_section("🚀 Starting DataMesh Node");
        
        // Create CLI configuration from wizard settings
        let cli = self.create_cli_config();
        
        // Create key manager
        let keys_dir = self.config.keys_dir.as_ref().unwrap();
        let key_name = self.config.key_name.as_ref().unwrap();
        let key_manager = match KeyManager::load_from_file(keys_dir, key_name) {
            Ok(km) => km,
            Err(_) => {
                // Generate a new key if loading fails
                KeyManager::generate_key(keys_dir, key_name)?
            }
        };
        
        // Use the commands module to execute the configured command
        use std::sync::Arc;
        use crate::commands;
        
        println!("🔄 Starting DataMesh node...");
        
        // Execute the command using the standard command dispatch system
        let key_manager_arc = Arc::new(key_manager);
        match commands::execute_command(cli, key_manager_arc).await {
            Ok(_) => {},
            Err(e) => {
                return Err(anyhow::anyhow!("Command execution failed: {}", e));
            }
        }
        
        Ok(())
    }

    /// Create CLI configuration from wizard settings
    fn create_cli_config(&self) -> Cli {
        use crate::cli::{Cli, Commands};
        
        // Create base CLI configuration
        let mut cli = Cli {
            bootstrap_peer: None,
            bootstrap_addr: None,
            bootstrap_peers: None,
            port: self.config.port,
            network: self.config.network_preset.clone(),
            keys_dir: self.config.keys_dir.clone(),
            key_name: self.config.key_name.clone(),
            non_interactive: false,
            command: Commands::Interactive {
                bootstrap_peer: None,
                bootstrap_addr: None,
                port: self.config.port,
            },
        };
        
        // Set bootstrap peers if available
        if !self.config.bootstrap_peers.is_empty() {
            let bootstrap_peers: Vec<String> = self.config.bootstrap_peers
                .iter()
                .map(|peer| format!("{}@{}", peer.peer_id, peer.address))
                .collect();
            
            cli.bootstrap_peers = Some(bootstrap_peers);
            
            // Set the first bootstrap peer as the primary
            if let Some(first_peer) = self.config.bootstrap_peers.first() {
                cli.bootstrap_peer = Some(first_peer.peer_id);
                cli.bootstrap_addr = Some(first_peer.address.clone());
            }
        }
        
        // Set the appropriate command based on node type
        cli.command = match &self.config.node_type {
            NodeType::Bootstrap { port } => {
                Commands::Bootstrap { port: *port }
            }
            NodeType::Regular { .. } => {
                Commands::Interactive {
                    bootstrap_peer: cli.bootstrap_peer,
                    bootstrap_addr: cli.bootstrap_addr.clone(),
                    port: self.config.port,
                }
            }
            NodeType::Service { timeout } => {
                Commands::Service {
                    bootstrap_peer: cli.bootstrap_peer,
                    bootstrap_addr: cli.bootstrap_addr.clone(),
                    port: self.config.port,
                    timeout: *timeout,
                }
            }
        };
        
        cli
    }
}

/// Check if DataMesh should start the setup wizard
pub fn should_start_wizard() -> bool {
    // Check if we're being called with no arguments or just the binary name
    let args: Vec<String> = std::env::args().collect();
    args.len() <= 1 || (args.len() == 2 && args[1] == "datamesh")
}

/// Launch the setup wizard
pub async fn launch_setup_wizard() -> Result<()> {
    let mut wizard = SetupWizard::new();
    wizard.run().await
}