/// Network Presets Module
///
/// This module provides simplified network connection options for users,
/// abstracting away the complexity of peer IDs and multiaddresses.
///
/// It supports several preset configurations:
/// - Local: Auto-discover peers on the local network
/// - Public: Connect to well-known public bootstrap nodes
/// - Custom: User-defined preset configurations
use anyhow::Result;
use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::ui;

/// Network preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPreset {
    pub name: String,
    pub description: String,
    pub bootstrap_peers: Vec<BootstrapPeer>,
    pub default_port_range: Option<(u16, u16)>,
    pub discovery_enabled: bool,
}

/// Bootstrap peer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeer {
    pub peer_id: String,
    pub addresses: Vec<String>,
}

/// Built-in network presets
pub struct NetworkPresets {
    presets: HashMap<String, NetworkPreset>,
}

impl NetworkPresets {
    /// Create a new NetworkPresets instance with built-in presets
    pub fn new() -> Self {
        let mut presets = HashMap::new();

        // Local network preset - load from config file or use defaults
        let local_preset = Self::load_local_network_preset().unwrap_or_else(|_| NetworkPreset {
            name: "local".to_string(),
            description: "Auto-discover peers on local network".to_string(),
            bootstrap_peers: vec![],  // Empty for local auto-discovery
            default_port_range: Some((40870, 40890)),
            discovery_enabled: true,
        });
        presets.insert("local".to_string(), local_preset);

        // Public network preset (example - would need real public nodes)
        presets.insert(
            "public".to_string(),
            NetworkPreset {
                name: "public".to_string(),
                description: "Connect to public DFS network".to_string(),
                bootstrap_peers: vec![BootstrapPeer {
                    peer_id: "example".to_string(),
                    addresses: vec![
                        "/ip4/203.0.113.1/tcp/40871".to_string(),
                        "/ip4/198.51.100.1/tcp/40871".to_string(),
                    ],
                }],
                default_port_range: Some((0, 0)), // Random port
                discovery_enabled: true,
            },
        );

        // Testing preset
        presets.insert(
            "test".to_string(),
            NetworkPreset {
                name: "test".to_string(),
                description: "Local testing with single bootstrap node".to_string(),
                bootstrap_peers: vec![BootstrapPeer {
                    peer_id: "auto".to_string(),
                    addresses: vec!["/ip4/127.0.0.1/tcp/40871".to_string()],
                }],
                default_port_range: Some((40880, 40890)),
                discovery_enabled: true,
            },
        );

        Self { presets }
    }

    /// Load local network preset from config file
    fn load_local_network_preset() -> Result<NetworkPreset> {
        let config_path = std::path::Path::new("config/local_network.toml");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Local network config not found"));
        }
        
        let content = std::fs::read_to_string(config_path)?;
        let config: toml::Value = toml::from_str(&content)?;
        
        let mut bootstrap_peers = Vec::new();
        
        // Parse bootstrap peers from TOML
        if let Some(bootstrap) = config.get("bootstrap") {
            if let Some(peers) = bootstrap.get("peers") {
                if let Some(peers_array) = peers.as_array() {
                    for peer_config in peers_array {
                        if let (Some(peer_id), Some(addresses)) = (
                            peer_config.get("peer_id").and_then(|v| v.as_str()),
                            peer_config.get("addresses").and_then(|v| v.as_array())
                        ) {
                            let addr_strings: Vec<String> = addresses
                                .iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect();
                            
                            if !addr_strings.is_empty() {
                                bootstrap_peers.push(BootstrapPeer {
                                    peer_id: peer_id.to_string(),
                                    addresses: addr_strings,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(NetworkPreset {
            name: "local".to_string(),
            description: "Local development network with configured bootstrap peers".to_string(),
            bootstrap_peers,
            default_port_range: Some((40870, 40890)),
            discovery_enabled: true,
        })
    }

    /// Load custom presets from configuration file
    pub fn load_custom_presets(&mut self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));

        let presets_file = config_dir.join("dfs").join("presets.toml");

        if presets_file.exists() {
            let content = std::fs::read_to_string(&presets_file)?;
            let custom_presets: HashMap<String, NetworkPreset> = toml::from_str(&content)?;

            for (name, preset) in custom_presets {
                self.presets.insert(name, preset);
            }

            ui::print_info(&format!("Loaded custom presets from {:?}", presets_file));
        }

        Ok(())
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&NetworkPreset> {
        self.presets.get(name)
    }

    /// List all available presets
    pub fn list_presets(&self) -> Vec<&NetworkPreset> {
        self.presets.values().collect()
    }

    /// Apply a preset to get connection parameters
    pub fn apply_preset(&self, preset_name: &str) -> Result<ConnectionConfig> {
        let preset = self
            .get_preset(preset_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown network preset: {}", preset_name))?;

        ui::print_info(&format!(
            "Using '{}' network preset: {}",
            preset.name, preset.description
        ));

        let mut config = ConnectionConfig {
            bootstrap_peers: Vec::new(),
            port: 0,
            discovery_enabled: preset.discovery_enabled,
        };

        // Handle port selection
        if let Some((min_port, max_port)) = preset.default_port_range {
            if min_port == 0 && max_port == 0 {
                config.port = 0; // Random port
            } else if min_port == max_port {
                config.port = min_port;
            } else {
                // For now, just use the minimum port. Could implement random selection.
                config.port = min_port;
            }
        }

        // Handle bootstrap peers
        for bootstrap_peer in &preset.bootstrap_peers {
            if bootstrap_peer.peer_id == "auto" {
                // For auto discovery, we'll try to connect to the addresses without a specific peer ID
                for addr_str in &bootstrap_peer.addresses {
                    if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                        config.bootstrap_peers.push(BootstrapConnection {
                            peer_id: None,
                            address: addr,
                        });
                    }
                }
            } else if bootstrap_peer.peer_id != "example" {
                // Skip example entries
                // Parse the peer ID and addresses
                if let Ok(peer_id) = PeerId::from_str(&bootstrap_peer.peer_id) {
                    for addr_str in &bootstrap_peer.addresses {
                        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                            config.bootstrap_peers.push(BootstrapConnection {
                                peer_id: Some(peer_id),
                                address: addr,
                            });
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    /// Save custom presets to configuration file
    pub fn save_custom_preset(&self, name: &str, preset: NetworkPreset) -> Result<()> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));

        let dfs_dir = config_dir.join("dfs");
        std::fs::create_dir_all(&dfs_dir)?;

        let presets_file = dfs_dir.join("presets.toml");

        // Load existing custom presets
        let mut custom_presets: HashMap<String, NetworkPreset> = if presets_file.exists() {
            let content = std::fs::read_to_string(&presets_file)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Add or update the preset
        custom_presets.insert(name.to_string(), preset);

        // Save back to file
        let content = toml::to_string_pretty(&custom_presets)?;
        std::fs::write(&presets_file, content)?;

        ui::print_success(&format!(
            "Saved custom preset '{}' to {:?}",
            name, presets_file
        ));
        Ok(())
    }
}

/// Resolved connection configuration from a preset
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub bootstrap_peers: Vec<BootstrapConnection>,
    pub port: u16,
    pub discovery_enabled: bool,
}

/// A resolved bootstrap connection
#[derive(Debug, Clone)]
pub struct BootstrapConnection {
    pub peer_id: Option<PeerId>,
    pub address: Multiaddr,
}

/// Parse a network specification that can be a preset name or custom connection string
pub fn parse_network_spec(spec: &str) -> Result<ConnectionConfig> {
    let mut presets = NetworkPresets::new();
    presets.load_custom_presets()?;

    // Check if it's a built-in preset
    if let Some(_) = presets.get_preset(spec) {
        return presets.apply_preset(spec);
    }

    // Check if it's a custom connection string (format: peer_id@address)
    if spec.contains('@') {
        let parts: Vec<&str> = spec.split('@').collect();
        if parts.len() == 2 {
            let peer_id = PeerId::from_str(parts[0])?;
            let address = parts[1].parse::<Multiaddr>()?;

            return Ok(ConnectionConfig {
                bootstrap_peers: vec![BootstrapConnection {
                    peer_id: Some(peer_id),
                    address,
                }],
                port: 0,
                discovery_enabled: true,
            });
        }
    }

    // Check if it's just an address
    if let Ok(address) = spec.parse::<Multiaddr>() {
        return Ok(ConnectionConfig {
            bootstrap_peers: vec![BootstrapConnection {
                peer_id: None,
                address,
            }],
            port: 0,
            discovery_enabled: true,
        });
    }

    Err(anyhow::anyhow!("Invalid network specification: {}. Use a preset name (local, public, test) or peer_id@address format.", spec))
}

/// Print available network presets
pub fn print_available_presets() {
    let mut presets = NetworkPresets::new();
    let _ = presets.load_custom_presets(); // Ignore errors for display

    ui::print_info("Available network presets:");

    for preset in presets.list_presets() {
        println!("  {} - {}", preset.name.bright_cyan(), preset.description);

        if !preset.bootstrap_peers.is_empty() {
            let peer_count = preset.bootstrap_peers.len();
            println!("    {} bootstrap peer(s)", peer_count);
        }

        if let Some((min, max)) = preset.default_port_range {
            if min == 0 && max == 0 {
                println!("    Port: random");
            } else if min == max {
                println!("    Port: {}", min);
            } else {
                println!("    Port range: {}-{}", min, max);
            }
        }

        println!();
    }
}

// Add this for the bright_cyan method
use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_creation() {
        let presets = NetworkPresets::new();
        assert!(presets.get_preset("local").is_some());
        assert!(presets.get_preset("public").is_some());
        assert!(presets.get_preset("test").is_some());
        assert!(presets.get_preset("nonexistent").is_none());
    }

    #[test]
    fn test_preset_application() {
        let presets = NetworkPresets::new();
        let config = presets.apply_preset("local").unwrap();
        assert!(config.discovery_enabled);
        assert!(!config.bootstrap_peers.is_empty());
    }

    #[test]
    fn test_network_spec_parsing() {
        // Test multiaddress parsing
        let config = parse_network_spec("/ip4/127.0.0.1/tcp/40871").unwrap();
        assert_eq!(config.bootstrap_peers.len(), 1);
        assert!(config.bootstrap_peers[0].peer_id.is_none());
    }
}

/// Get available networks for display
pub fn get_available_networks() -> HashMap<String, NetworkInfo> {
    let mut networks = HashMap::new();

    networks.insert(
        "local".to_string(),
        NetworkInfo {
            description: "Auto-discover peers on local network".to_string(),
            bootstrap_nodes: vec![
                "/ip4/127.0.0.1/tcp/40871".to_string(),
                "/ip6/::1/tcp/40871".to_string(),
            ],
            features: Some(vec!["auto-discovery".to_string(), "local-only".to_string()]),
        },
    );

    networks.insert(
        "public".to_string(),
        NetworkInfo {
            description: "Connect to public DataMesh network".to_string(),
            bootstrap_nodes: vec!["/ip4/147.75.77.187/tcp/40871".to_string()],
            features: Some(vec!["public".to_string(), "persistent".to_string()]),
        },
    );

    networks.insert(
        "testnet".to_string(),
        NetworkInfo {
            description: "Connect to DataMesh test network".to_string(),
            bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/40872".to_string()],
            features: Some(vec!["testing".to_string(), "development".to_string()]),
        },
    );

    networks
}

/// Network information for display
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub description: String,
    pub bootstrap_nodes: Vec<String>,
    pub features: Option<Vec<String>>,
}
