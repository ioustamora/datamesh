/// Key Rotation Module for Perfect Forward Secrecy
///
/// This module implements automatic key rotation to provide perfect forward secrecy.
/// Keys are rotated on a configurable schedule to ensure that compromise of current
/// keys does not affect past encrypted data.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use ecies::SecretKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{info, error};
use crate::error::DfsResult;

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// How often to rotate keys (in hours)
    pub rotation_interval_hours: u64,
    /// How many old keys to keep for decryption
    pub key_history_size: usize,
    /// Whether automatic rotation is enabled
    pub auto_rotation_enabled: bool,
    /// Minimum interval between manual rotations (in minutes)
    pub min_manual_rotation_interval_minutes: u64,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            rotation_interval_hours: 24, // Rotate daily by default
            key_history_size: 7, // Keep 7 days of keys
            auto_rotation_enabled: true,
            min_manual_rotation_interval_minutes: 60, // Minimum 1 hour between manual rotations
        }
    }
}

/// A versioned encryption key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedKey {
    /// The secret key
    pub key: SecretKey,
    /// Key version identifier
    pub version: u64,
    /// When this key was created
    pub created_at: DateTime<Utc>,
    /// When this key expires (optional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this key can be used for encryption (current key)
    pub is_current: bool,
}

impl VersionedKey {
    /// Create a new versioned key
    pub fn new(version: u64) -> Self {
        Self {
            key: SecretKey::random(&mut OsRng),
            version,
            created_at: Utc::now(),
            expires_at: None,
            is_current: true,
        }
    }

    /// Check if this key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Mark this key as expired
    pub fn expire(&mut self) {
        self.expires_at = Some(Utc::now());
        self.is_current = false;
    }
}

/// Manages key rotation for perfect forward secrecy
pub struct KeyRotationManager {
    config: KeyRotationConfig,
    current_version: u64,
    keys: Arc<RwLock<Vec<VersionedKey>>>,
    last_rotation: Arc<RwLock<Instant>>,
    last_manual_rotation: Arc<RwLock<Option<Instant>>>,
}

impl KeyRotationManager {
    /// Create a new key rotation manager
    pub fn new(config: KeyRotationConfig) -> Self {
        let initial_key = VersionedKey::new(1);
        
        Self {
            config,
            current_version: 1,
            keys: Arc::new(RwLock::new(vec![initial_key])),
            last_rotation: Arc::new(RwLock::new(Instant::now())),
            last_manual_rotation: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current encryption key
    pub fn get_current_key(&self) -> DfsResult<VersionedKey> {
        let keys = self.keys.read().unwrap();
        keys.iter()
            .find(|k| k.is_current && !k.is_expired())
            .cloned()
            .ok_or_else(|| crate::error::DfsError::Crypto("No current encryption key available".to_string()))
    }

    /// Get a decryption key by version
    pub fn get_key_by_version(&self, version: u64) -> Option<VersionedKey> {
        let keys = self.keys.read().unwrap();
        keys.iter().find(|k| k.version == version).cloned()
    }

    /// Get all available keys for decryption
    pub fn get_all_keys(&self) -> Vec<VersionedKey> {
        let keys = self.keys.read().unwrap();
        keys.clone()
    }

    /// Manually trigger key rotation
    pub fn rotate_key_manual(&mut self) -> DfsResult<u64> {
        // Check minimum interval for manual rotation
        if let Some(last_manual) = *self.last_manual_rotation.read().unwrap() {
            let min_interval = Duration::from_secs(self.config.min_manual_rotation_interval_minutes * 60);
            if last_manual.elapsed() < min_interval {
                return Err(crate::error::DfsError::Config(
                    format!("Manual rotation attempted too soon. Wait {} minutes.", 
                           self.config.min_manual_rotation_interval_minutes)
                ));
            }
        }

        let new_version = self.rotate_key_internal()?;
        *self.last_manual_rotation.write().unwrap() = Some(Instant::now());
        
        info!("Manual key rotation completed. New version: {}", new_version);
        Ok(new_version)
    }

    /// Internal key rotation logic
    fn rotate_key_internal(&mut self) -> DfsResult<u64> {
        let new_version = self.current_version + 1;
        let new_key = VersionedKey::new(new_version);

        {
            let mut keys = self.keys.write().unwrap();
            
            // Mark current key as no longer current
            for key in keys.iter_mut() {
                if key.is_current {
                    key.is_current = false;
                    key.expire();
                }
            }
            
            // Add new key
            keys.push(new_key);
            
            // Clean up old keys beyond history size
            if keys.len() > self.config.key_history_size {
                let to_remove = keys.len() - self.config.key_history_size;
                keys.drain(0..to_remove);
                info!("Cleaned up {} old keys", to_remove);
            }
        }

        self.current_version = new_version;
        *self.last_rotation.write().unwrap() = Instant::now();
        
        info!("Key rotation completed. New version: {}", new_version);
        Ok(new_version)
    }

    /// Check if automatic rotation is needed
    pub fn needs_rotation(&self) -> bool {
        if !self.config.auto_rotation_enabled {
            return false;
        }

        let last_rotation = *self.last_rotation.read().unwrap();
        let rotation_interval = Duration::from_secs(self.config.rotation_interval_hours * 3600);
        
        last_rotation.elapsed() >= rotation_interval
    }

    /// Perform automatic rotation if needed
    pub fn try_auto_rotate(&mut self) -> DfsResult<Option<u64>> {
        if self.needs_rotation() {
            let new_version = self.rotate_key_internal()?;
            info!("Automatic key rotation completed. New version: {}", new_version);
            Ok(Some(new_version))
        } else {
            Ok(None)
        }
    }

    /// Start automatic key rotation background task
    pub async fn start_auto_rotation_task(manager: Arc<RwLock<Self>>) {
        let check_interval = Duration::from_secs(3600); // Check every hour
        let mut interval_timer = interval(check_interval);

        info!("Starting automatic key rotation task");

        loop {
            interval_timer.tick().await;
            
            let needs_rotation = {
                let mgr = manager.read().unwrap();
                mgr.needs_rotation()
            };

            if needs_rotation {
                match manager.write() {
                    Ok(mut mgr) => {
                        if let Err(e) = mgr.try_auto_rotate() {
                            error!("Automatic key rotation failed: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to acquire write lock for key rotation: {}", e);
                    }
                }
            }
        }
    }

    /// Get rotation statistics
    pub fn get_rotation_stats(&self) -> RotationStats {
        let keys = self.keys.read().unwrap();
        let last_rotation = *self.last_rotation.read().unwrap();
        
        RotationStats {
            current_version: self.current_version,
            total_keys: keys.len(),
            active_keys: keys.iter().filter(|k| !k.is_expired()).count(),
            last_rotation_ago: last_rotation.elapsed(),
            next_rotation_in: if self.config.auto_rotation_enabled {
                let rotation_interval = Duration::from_secs(self.config.rotation_interval_hours * 3600);
                Some(rotation_interval.saturating_sub(last_rotation.elapsed()))
            } else {
                None
            },
            auto_rotation_enabled: self.config.auto_rotation_enabled,
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: KeyRotationConfig) {
        info!("Updating key rotation configuration");
        self.config = config;
    }
}

/// Statistics about key rotation
#[derive(Debug, Serialize)]
pub struct RotationStats {
    pub current_version: u64,
    pub total_keys: usize,
    pub active_keys: usize,
    pub last_rotation_ago: Duration,
    pub next_rotation_in: Option<Duration>,
    pub auto_rotation_enabled: bool,
}

/// Enhanced key manager with rotation support
pub struct RotatingKeyManager {
    base_key_manager: crate::key_manager::KeyManager,
    rotation_manager: Arc<RwLock<KeyRotationManager>>,
}

impl RotatingKeyManager {
    /// Create a new rotating key manager
    pub fn new(
        base_manager: crate::key_manager::KeyManager,
        rotation_config: KeyRotationConfig,
    ) -> Self {
        let rotation_manager = KeyRotationManager::new(rotation_config);
        
        Self {
            base_key_manager: base_manager,
            rotation_manager: Arc::new(RwLock::new(rotation_manager)),
        }
    }

    /// Get the current encryption key
    pub fn get_encryption_key(&self) -> DfsResult<(SecretKey, u64)> {
        let manager = self.rotation_manager.read().unwrap();
        let versioned_key = manager.get_current_key()?;
        Ok((versioned_key.key, versioned_key.version))
    }

    /// Get a decryption key by version
    pub fn get_decryption_key(&self, version: u64) -> Option<SecretKey> {
        let manager = self.rotation_manager.read().unwrap();
        manager.get_key_by_version(version).map(|k| k.key)
    }

    /// Manually rotate keys
    pub fn rotate_keys(&self) -> DfsResult<u64> {
        let mut manager = self.rotation_manager.write().unwrap();
        manager.rotate_key_manual()
    }

    /// Start automatic rotation task
    pub async fn start_auto_rotation(&self) {
        let manager = self.rotation_manager.clone();
        KeyRotationManager::start_auto_rotation_task(manager).await;
    }

    /// Get rotation statistics
    pub fn get_stats(&self) -> RotationStats {
        let manager = self.rotation_manager.read().unwrap();
        manager.get_rotation_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_versioned_key_creation() {
        let key = VersionedKey::new(1);
        assert_eq!(key.version, 1);
        assert!(key.is_current);
        assert!(!key.is_expired());
    }

    #[test]
    fn test_key_expiration() {
        let mut key = VersionedKey::new(1);
        assert!(!key.is_expired());
        
        key.expire();
        assert!(key.is_expired());
        assert!(!key.is_current);
    }

    #[test]
    fn test_key_rotation_manager() {
        let config = KeyRotationConfig::default();
        let mut manager = KeyRotationManager::new(config);
        
        // Test initial state
        let current_key = manager.get_current_key().unwrap();
        assert_eq!(current_key.version, 1);
        
        // Test manual rotation
        let new_version = manager.rotate_key_manual().unwrap();
        assert_eq!(new_version, 2);
        
        let current_key = manager.get_current_key().unwrap();
        assert_eq!(current_key.version, 2);
        
        // Test key retrieval by version
        let old_key = manager.get_key_by_version(1).unwrap();
        assert_eq!(old_key.version, 1);
        assert!(!old_key.is_current);
    }

    #[test]
    fn test_key_history_cleanup() {
        let config = KeyRotationConfig {
            key_history_size: 2,
            min_manual_rotation_interval_minutes: 0, // Allow immediate rotation for testing
            ..Default::default()
        };
        let mut manager = KeyRotationManager::new(config);
        
        // Rotate multiple times
        for _ in 0..5 {
            manager.rotate_key_manual().unwrap();
        }
        
        let all_keys = manager.get_all_keys();
        assert_eq!(all_keys.len(), 2); // Should only keep 2 keys
        
        // Should have versions 5 and 6 (latest 2)
        let versions: Vec<u64> = all_keys.iter().map(|k| k.version).collect();
        assert!(versions.contains(&5));
        assert!(versions.contains(&6));
    }

    #[tokio::test]
    async fn test_auto_rotation_needs_check() {
        let config = KeyRotationConfig {
            rotation_interval_hours: 1, // 1 hour
            auto_rotation_enabled: true,
            ..Default::default()
        };
        let manager = KeyRotationManager::new(config);
        
        // Should not need rotation immediately
        assert!(!manager.needs_rotation());
        
        // Test with short interval for quick testing
        let short_config = KeyRotationConfig {
            rotation_interval_hours: 0, // 0 hours (immediate)
            auto_rotation_enabled: true,
            ..Default::default()
        };
        let short_manager = KeyRotationManager::new(short_config);
        
        // Sleep briefly to ensure time has passed
        sleep(Duration::from_millis(10));
        assert!(short_manager.needs_rotation());
    }
}