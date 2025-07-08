/// Key Management Module
///
/// This module handles cryptographic key management for the DFS application,
/// including key generation, storage, loading, and backup. It supports both
/// interactive and non-interactive key selection modes.
///
/// The module implements:
/// - ECIES key pair generation and management
/// - Secure key storage and retrieval
/// - Key backup and recovery
/// - User-friendly key selection interfaces
///
/// Keys are stored in a dedicated directory with separate files for the private key
/// and metadata to facilitate easier management and backup.
use anyhow::Result;
use chrono::{DateTime, Local};
use ecies::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::Cli;

/// Metadata about an ECIES key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EciesKeyInfo {
    /// User-friendly name for the key
    pub name: String,
    /// Creation timestamp
    pub created: DateTime<Local>,
    /// Hex-encoded public key
    pub public_key_hex: String,
}

/// Key manager that holds a secret key and its metadata
#[derive(Debug, Clone)]
pub struct KeyManager {
    /// The ECIES secret key
    pub key: SecretKey,
    /// Metadata about the key
    pub key_info: EciesKeyInfo,
}

/// Key selection mode for different application contexts
#[derive(Debug, Clone)]
pub enum KeySelectionMode {
    /// Interactive mode prompts the user for key selection
    Interactive,
    /// Non-interactive mode uses default key or fails
    NonInteractive,
    /// Force generation of a new key
    ForceGenerate,
}

impl KeyManager {
    pub fn new(key: SecretKey, name: String) -> Self {
        let public_key = PublicKey::from_secret_key(&key);
        let key_info = EciesKeyInfo {
            name,
            created: Local::now(),
            public_key_hex: hex::encode(public_key.serialize()),
        };
        
        Self { key, key_info }
    }
    
    pub fn save_to_file(&self, keys_dir: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(keys_dir)?;
        
        let key_file = keys_dir.join(format!("{}.key", self.key_info.name));
        let info_file = keys_dir.join(format!("{}.info", self.key_info.name));
        
        // Save the secret key (binary format)
        fs::write(&key_file, self.key.serialize())?;
        
        // Save the key info (JSON format)
        let info_json = serde_json::to_string_pretty(&self.key_info)?;
        fs::write(&info_file, info_json)?;
        
        println!("Key saved: {}", key_file.display());
        println!("Key info saved: {}", info_file.display());
        
        Ok(())
    }
    
    pub fn load_from_file(keys_dir: &Path, name: &str) -> Result<Self, Box<dyn Error>> {
        let key_file = keys_dir.join(format!("{}.key", name));
        let info_file = keys_dir.join(format!("{}.info", name));
        
        // Load the secret key
        let key_bytes = fs::read(&key_file)?;
        let key = SecretKey::parse_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse secret key: {:?}", e))?;
        
        // Load the key info
        let info_json = fs::read_to_string(&info_file)?;
        let key_info: EciesKeyInfo = serde_json::from_str(&info_json)?;
        
        Ok(Self { key, key_info })
    }
    
    pub fn list_keys(keys_dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
        if !keys_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut key_names = Vec::new();
        for entry in fs::read_dir(keys_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "key") {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        // Check if corresponding .info file exists
                        let info_file = keys_dir.join(format!("{}.info", name));
                        if info_file.exists() {
                            key_names.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(key_names)
    }
    
    pub fn get_key_info(keys_dir: &Path, name: &str) -> Result<EciesKeyInfo, Box<dyn Error>> {
        let info_file = keys_dir.join(format!("{}.info", name));
        let info_json = fs::read_to_string(&info_file)?;
        let key_info: EciesKeyInfo = serde_json::from_str(&info_json)?;
        Ok(key_info)
    }
}

pub fn get_default_keys_dir() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(home_dir) = dirs::home_dir() {
        Ok(home_dir.join(".datamesh").join("keys"))
    } else {
        Ok(PathBuf::from("./keys"))
    }
}

fn prompt_user_input(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn generate_default_key_name() -> String {
    let now = Local::now();
    format!("dfs_key_{}", now.format("%Y%m%d_%H%M%S"))
}

pub fn parse_public_key(public_key_hex: &str) -> Result<PublicKey, Box<dyn Error>> {
    let key_bytes = hex::decode(public_key_hex)?;
    if key_bytes.len() != 65 {
        return Err(anyhow::anyhow!("Public key must be 65 bytes, got {}", key_bytes.len()).into());
    }
    let mut key_array = [0u8; 65];
    key_array.copy_from_slice(&key_bytes);
    PublicKey::parse(&key_array)
        .map_err(|e| anyhow::anyhow!("Failed to parse public key: {:?}", e).into())
}

pub fn get_encryption_key(
    public_key_opt: &Option<String>,
    key_manager: &KeyManager,
) -> Result<(PublicKey, String), Box<dyn Error>> {
    if let Some(public_key_hex) = public_key_opt {
        let public_key = parse_public_key(public_key_hex)?;
        Ok((public_key, public_key_hex.clone()))
    } else {
        let public_key = PublicKey::from_secret_key(&key_manager.key);
        Ok((public_key, key_manager.key_info.public_key_hex.clone()))
    }
}

pub fn get_decryption_key(
    private_key_opt: &Option<String>,
    key_manager: &KeyManager,
    keys_dir: &PathBuf,
) -> Result<SecretKey, Box<dyn Error>> {
    if let Some(private_key_name) = private_key_opt {
        let loaded_manager = KeyManager::load_from_file(keys_dir, private_key_name)?;
        Ok(loaded_manager.key)
    } else {
        Ok(key_manager.key.clone())
    }
}

pub async fn setup_key_management(cli: &Cli) -> Result<KeyManager, Box<dyn Error>> {
    setup_key_management_with_mode(cli, KeySelectionMode::Interactive).await
}

pub async fn setup_key_management_with_mode(cli: &Cli, mode: KeySelectionMode) -> Result<KeyManager, Box<dyn Error>> {
    let keys_dir = cli.keys_dir.clone()
        .unwrap_or_else(|| get_default_keys_dir().unwrap_or_else(|_| PathBuf::from("./keys")));
    
    if !matches!(mode, KeySelectionMode::NonInteractive) {
        println!("Using keys directory: {}", keys_dir.display());
    }
    
    // If a specific key name is provided, try to load it
    if let Some(key_name) = &cli.key_name {
        match KeyManager::load_from_file(&keys_dir, key_name) {
            Ok(key_manager) => {
                if !matches!(mode, KeySelectionMode::NonInteractive) {
                    println!("Loaded existing key: {}", key_name);
                    println!("Public key: {}", key_manager.key_info.public_key_hex);
                    println!("Created: {}", key_manager.key_info.created.format("%Y-%m-%d %H:%M:%S"));
                }
                return Ok(key_manager);
            }
            Err(e) => {
                if matches!(mode, KeySelectionMode::NonInteractive) {
                    // In non-interactive mode, generate a new key if the specified one doesn't exist
                    let secret_key = SecretKey::random(&mut rand::thread_rng());
                    let key_manager = KeyManager::new(secret_key, key_name.clone());
                    key_manager.save_to_file(&keys_dir)?;
                    return Ok(key_manager);
                } else {
                    println!("Failed to load key '{}': {}", key_name, e);
                    return Err(e);
                }
            }
        }
    }
    
    // List existing keys
    let available_keys = KeyManager::list_keys(&keys_dir)?;
    
    if available_keys.is_empty() || matches!(mode, KeySelectionMode::ForceGenerate) {
        if !matches!(mode, KeySelectionMode::NonInteractive) {
            println!("No ECIES keys found in the keys directory.");
            println!("You need to generate a new key to use the DFS system.");
        }
        
        let key_name = if let Some(name) = &cli.key_name {
            name.clone()
        } else {
            match mode {
                KeySelectionMode::NonInteractive | KeySelectionMode::ForceGenerate => {
                    generate_default_key_name()
                }
                KeySelectionMode::Interactive => {
                    let default_name = generate_default_key_name();
                    let input = prompt_user_input(&format!(
                        "Enter a name for the new key (default: {}): ", default_name
                    ))?;
                    if input.is_empty() { default_name } else { input }
                }
            }
        };
        
        if !matches!(mode, KeySelectionMode::NonInteractive) {
            println!("Generating new ECIES key pair...");
        }
        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let key_manager = KeyManager::new(secret_key, key_name);
        
        key_manager.save_to_file(&keys_dir)?;
        if !matches!(mode, KeySelectionMode::NonInteractive) {
            println!("New key generated and saved successfully!");
            println!("Public key: {}", key_manager.key_info.public_key_hex);
        }
        
        return Ok(key_manager);
    }
    
    // Multiple keys available - handle based on mode
    match mode {
        KeySelectionMode::NonInteractive => {
            // In non-interactive mode, automatically select the first available key
            let key_name = &available_keys[0];
            let key_manager = KeyManager::load_from_file(&keys_dir, key_name)?;
            return Ok(key_manager);
        }
        KeySelectionMode::Interactive => {
            // Interactive mode - let user choose
            println!("Found {} existing key(s):", available_keys.len());
            for (i, key_name) in available_keys.iter().enumerate() {
                if let Ok(info) = KeyManager::get_key_info(&keys_dir, key_name) {
                    println!("  {}. {} (created: {}, public: {}...)", 
                        i + 1, 
                        info.name, 
                        info.created.format("%Y-%m-%d %H:%M:%S"),
                        &info.public_key_hex[..16]
                    );
                }
            }
            println!("  {}. Generate a new key", available_keys.len() + 1);
            
            let choice = loop {
                let input = prompt_user_input(&format!(
                    "Choose a key (1-{}) or generate new ({}): ", 
                    available_keys.len(), 
                    available_keys.len() + 1
                ))?;
                
                if let Ok(choice) = input.parse::<usize>() {
                    if choice >= 1 && choice <= available_keys.len() + 1 {
                        break choice;
                    }
                }
                println!("Invalid choice. Please enter a number between 1 and {}.", available_keys.len() + 1);
            };
            
            if choice <= available_keys.len() {
                // Load existing key
                let key_name = &available_keys[choice - 1];
                let key_manager = KeyManager::load_from_file(&keys_dir, key_name)?;
                println!("Loaded key: {}", key_name);
                println!("Public key: {}", key_manager.key_info.public_key_hex);
                Ok(key_manager)
            } else {
                // Generate new key
                let default_name = generate_default_key_name();
                let key_name = prompt_user_input(&format!(
                    "Enter a name for the new key (default: {}): ", default_name
                ))?;
                let key_name = if key_name.is_empty() { default_name } else { key_name };
                
                println!("Generating new ECIES key pair...");
                let secret_key = SecretKey::random(&mut rand::thread_rng());
                let key_manager = KeyManager::new(secret_key, key_name);
                
                key_manager.save_to_file(&keys_dir)?;
                println!("New key generated and saved successfully!");
                println!("Public key: {}", key_manager.key_info.public_key_hex);
                
                Ok(key_manager)
            }
        }
        KeySelectionMode::ForceGenerate => {
            // Force generate new key
            let key_name = if let Some(name) = &cli.key_name {
                name.clone()
            } else {
                generate_default_key_name()
            };
            
            let secret_key = SecretKey::random(&mut rand::thread_rng());
            let key_manager = KeyManager::new(secret_key, key_name);
            
            key_manager.save_to_file(&keys_dir)?;
            println!("New key generated and saved successfully!");
            println!("Public key: {}", key_manager.key_info.public_key_hex);
            
            Ok(key_manager)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_key_generation_and_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();
        
        // Test key creation
        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let key_manager = KeyManager::new(secret_key, "test_key".to_string());
        
        // Test saving
        key_manager.save_to_file(keys_dir).unwrap();
        
        // Test loading
        let loaded_manager = KeyManager::load_from_file(keys_dir, "test_key").unwrap();
        
        // Verify keys match
        assert_eq!(key_manager.key.serialize(), loaded_manager.key.serialize());
        assert_eq!(key_manager.key_info.name, loaded_manager.key_info.name);
        assert_eq!(key_manager.key_info.public_key_hex, loaded_manager.key_info.public_key_hex);
    }

    #[test]
    fn test_key_listing() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();
        
        // Create multiple keys
        let key1 = KeyManager::new(SecretKey::random(&mut rand::thread_rng()), "key1".to_string());
        let key2 = KeyManager::new(SecretKey::random(&mut rand::thread_rng()), "key2".to_string());
        
        key1.save_to_file(keys_dir).unwrap();
        key2.save_to_file(keys_dir).unwrap();
        
        // Test listing
        let keys = KeyManager::list_keys(keys_dir).unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_public_key_parsing() {
        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let public_key = PublicKey::from_secret_key(&secret_key);
        let public_key_hex = hex::encode(public_key.serialize());
        
        // Test parsing
        let parsed_key = parse_public_key(&public_key_hex).unwrap();
        assert_eq!(public_key.serialize(), parsed_key.serialize());
    }

    #[test]
    fn test_invalid_public_key() {
        // Test with invalid hex
        assert!(parse_public_key("invalid_hex").is_err());
        
        // Test with wrong length
        assert!(parse_public_key("deadbeef").is_err());
    }

    #[test]
    fn test_get_encryption_key() {
        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let key_manager = KeyManager::new(secret_key, "test".to_string());
        
        // Test with no specific public key (should use default)
        let (pub_key, hex) = get_encryption_key(&None, &key_manager).unwrap();
        assert_eq!(hex, key_manager.key_info.public_key_hex);
        
        // Test with specific public key
        let custom_key = SecretKey::random(&mut rand::thread_rng());
        let custom_pub = PublicKey::from_secret_key(&custom_key);
        let custom_hex = hex::encode(custom_pub.serialize());
        
        let (parsed_pub, parsed_hex) = get_encryption_key(&Some(custom_hex.clone()), &key_manager).unwrap();
        assert_eq!(parsed_hex, custom_hex);
        assert_eq!(parsed_pub.serialize(), custom_pub.serialize());
    }
}
