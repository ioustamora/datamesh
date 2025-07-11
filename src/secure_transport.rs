/// Secure Transport Configuration Module
///
/// This module provides enhanced security features for P2P transport,
/// including certificate pinning, peer authentication, and connection validation.

use std::collections::HashSet;
use std::sync::Arc;
use libp2p::{identity, PeerId};
use libp2p::noise;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

/// Secure transport configuration for P2P connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureTransportConfig {
    /// Whether to require peer authentication
    pub require_peer_authentication: bool,
    /// List of allowed peer public keys (certificate pinning)
    pub allowed_peer_keys: HashSet<String>,
    /// Whether to enable strict peer validation
    pub strict_peer_validation: bool,
    /// Maximum number of connections per peer
    pub max_connections_per_peer: u32,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
}

impl Default for SecureTransportConfig {
    fn default() -> Self {
        Self {
            require_peer_authentication: false, // Start permissive for development
            allowed_peer_keys: HashSet::new(),
            strict_peer_validation: false,
            max_connections_per_peer: 5,
            connection_timeout_secs: 30,
        }
    }
}

impl SecureTransportConfig {
    /// Create a new secure transport configuration
    pub fn new(require_auth: bool) -> Self {
        Self {
            require_peer_authentication: require_auth,
            ..Default::default()
        }
    }

    /// Add an allowed peer key for certificate pinning
    pub fn add_allowed_peer(&mut self, peer_key: String) {
        info!("Added allowed peer key: {}", peer_key);
        self.allowed_peer_keys.insert(peer_key);
    }

    /// Remove an allowed peer key
    pub fn remove_allowed_peer(&mut self, peer_key: &str) {
        if self.allowed_peer_keys.remove(peer_key) {
            info!("Removed allowed peer key: {}", peer_key);
        }
    }

    /// Check if a peer is allowed to connect
    pub fn is_peer_allowed(&self, peer_id: &PeerId) -> bool {
        if !self.require_peer_authentication {
            return true;
        }

        let peer_key = peer_id.to_string();
        let allowed = self.allowed_peer_keys.contains(&peer_key);
        
        if !allowed {
            warn!("Rejecting connection from unauthorized peer: {}", peer_key);
        } else {
            info!("Accepting connection from authorized peer: {}", peer_key);
        }
        
        allowed
    }

    /// Create a configured Noise transport with security enhancements
    pub fn create_noise_config(&self, keypair: &identity::Keypair) -> Result<noise::Config, Box<dyn std::error::Error>> {
        let config = noise::Config::new(keypair)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        // Additional security configurations would go here
        // For now, we return the basic config as libp2p handles most security
        Ok(config)
    }

    /// Validate connection parameters
    pub fn validate_connection(&self, peer_id: &PeerId, connection_count: u32) -> Result<(), SecurityError> {
        // Check peer allowlist
        if !self.is_peer_allowed(peer_id) {
            return Err(SecurityError::UnauthorizedPeer(peer_id.to_string()));
        }

        // Check connection limits
        if connection_count >= self.max_connections_per_peer {
            return Err(SecurityError::TooManyConnections {
                peer: peer_id.to_string(),
                current: connection_count,
                max: self.max_connections_per_peer,
            });
        }

        Ok(())
    }
}

/// Security-related errors for transport layer
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Unauthorized peer: {0}")]
    UnauthorizedPeer(String),
    
    #[error("Too many connections to peer {peer}: {current}/{max}")]
    TooManyConnections {
        peer: String,
        current: u32,
        max: u32,
    },
    
    #[error("Connection timeout exceeded")]
    ConnectionTimeout,
    
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),
}

/// Peer connection tracker for managing connection limits
#[derive(Debug, Default)]
pub struct PeerConnectionTracker {
    connections: std::collections::HashMap<PeerId, u32>,
}

impl PeerConnectionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new connection for a peer
    pub fn add_connection(&mut self, peer_id: PeerId) {
        let count = self.connections.entry(peer_id).or_insert(0);
        *count += 1;
        info!("Peer {} now has {} connections", peer_id, count);
    }

    /// Remove a connection for a peer
    pub fn remove_connection(&mut self, peer_id: &PeerId) {
        if let Some(count) = self.connections.get_mut(peer_id) {
            *count = count.saturating_sub(1);
            let connection_count = *count;
            if connection_count == 0 {
                self.connections.remove(peer_id);
            }
            info!("Peer {} now has {} connections", peer_id, connection_count);
        }
    }

    /// Get the current connection count for a peer
    pub fn get_connection_count(&self, peer_id: &PeerId) -> u32 {
        self.connections.get(peer_id).copied().unwrap_or(0)
    }

    /// Get all peers with active connections
    pub fn get_connected_peers(&self) -> Vec<PeerId> {
        self.connections.keys().cloned().collect()
    }
}

/// Enhanced security manager for P2P transport
pub struct SecurityManager {
    config: SecureTransportConfig,
    connection_tracker: Arc<std::sync::RwLock<PeerConnectionTracker>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecureTransportConfig) -> Self {
        Self {
            config,
            connection_tracker: Arc::new(std::sync::RwLock::new(PeerConnectionTracker::new())),
        }
    }

    /// Validate an incoming connection
    pub fn validate_incoming_connection(&self, peer_id: &PeerId) -> Result<(), SecurityError> {
        let tracker = self.connection_tracker.read().unwrap();
        let connection_count = tracker.get_connection_count(peer_id);
        self.config.validate_connection(peer_id, connection_count)
    }

    /// Register a new connection
    pub fn register_connection(&self, peer_id: PeerId) -> Result<(), SecurityError> {
        self.validate_incoming_connection(&peer_id)?;
        
        let mut tracker = self.connection_tracker.write().unwrap();
        tracker.add_connection(peer_id);
        Ok(())
    }

    /// Unregister a connection
    pub fn unregister_connection(&self, peer_id: &PeerId) {
        let mut tracker = self.connection_tracker.write().unwrap();
        tracker.remove_connection(peer_id);
    }

    /// Get security configuration
    pub fn config(&self) -> &SecureTransportConfig {
        &self.config
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecureTransportConfig) {
        info!("Updating security configuration");
        self.config = config;
    }

    /// Get connection statistics
    pub fn get_connection_stats(&self) -> Vec<(PeerId, u32)> {
        let tracker = self.connection_tracker.read().unwrap();
        tracker.connections.iter().map(|(k, v)| (*k, *v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_transport_config() {
        let mut config = SecureTransportConfig::new(true);
        
        // Test peer management
        let peer_key = "test_peer_key".to_string();
        config.add_allowed_peer(peer_key.clone());
        assert!(config.allowed_peer_keys.contains(&peer_key));
        
        config.remove_allowed_peer(&peer_key);
        assert!(!config.allowed_peer_keys.contains(&peer_key));
    }

    #[test]
    fn test_connection_tracker() {
        let mut tracker = PeerConnectionTracker::new();
        let peer_id = PeerId::random();
        
        // Test adding connections
        tracker.add_connection(peer_id);
        assert_eq!(tracker.get_connection_count(&peer_id), 1);
        
        tracker.add_connection(peer_id);
        assert_eq!(tracker.get_connection_count(&peer_id), 2);
        
        // Test removing connections
        tracker.remove_connection(&peer_id);
        assert_eq!(tracker.get_connection_count(&peer_id), 1);
        
        tracker.remove_connection(&peer_id);
        assert_eq!(tracker.get_connection_count(&peer_id), 0);
    }

    #[test]
    fn test_security_manager() {
        let config = SecureTransportConfig::new(false); // Permissive for testing
        let manager = SecurityManager::new(config);
        
        let peer_id = PeerId::random();
        
        // Test connection registration
        assert!(manager.register_connection(peer_id).is_ok());
        assert!(manager.register_connection(peer_id).is_ok());
        
        // Test connection stats
        let stats = manager.get_connection_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].1, 2); // 2 connections for the peer
        
        // Test unregistration
        manager.unregister_connection(&peer_id);
        manager.unregister_connection(&peer_id);
        
        let stats = manager.get_connection_stats();
        assert_eq!(stats.len(), 0); // No more connections
    }
}