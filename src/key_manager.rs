use crate::secure_random;
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
use rand::rngs::OsRng;
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
    /// BLAKE3 integrity hash of the private key
    pub integrity_hash: String,
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
}

impl KeyManager {
    pub fn new(key: SecretKey, name: String) -> Self {
        // Validate key strength
        if let Err(e) = Self::validate_key_strength(&key) {
            eprintln!("⚠️  Warning: Key strength validation failed: {}", e);
        }

        let public_key = PublicKey::from_secret_key(&key);
        let integrity_hash = Self::calculate_key_integrity(&key);

        let key_info = EciesKeyInfo {
            name,
            created: Local::now(),
            public_key_hex: hex::encode(public_key.serialize()),
            integrity_hash,
        };

        Self { key, key_info }
    }

    fn calculate_key_integrity(key: &SecretKey) -> String {
        let key_bytes = key.serialize();
        let hash = blake3::hash(&key_bytes);
        hex::encode(hash.as_bytes())
    }

    fn validate_key_strength(key: &SecretKey) -> Result<(), String> {
        let key_bytes = key.serialize();

        // Check key length
        if key_bytes.len() != 32 {
            return Err("Invalid key length - must be 32 bytes".to_string());
        }

        // Check for weak keys (all zeros, all ones, etc.)
        if key_bytes.iter().all(|&x| x == 0) {
            return Err("Weak key detected - all zeros".to_string());
        }

        if key_bytes.iter().all(|&x| x == 0xFF) {
            return Err("Weak key detected - all ones".to_string());
        }

        // Check for patterns that indicate weak entropy
        let mut pattern_count = 0;
        for i in 0..key_bytes.len() - 1 {
            if key_bytes[i] == key_bytes[i + 1] {
                pattern_count += 1;
            }
        }

        // If more than 75% of adjacent bytes are the same, it's likely weak
        if pattern_count > (key_bytes.len() * 3) / 4 {
            return Err("Weak key detected - low entropy".to_string());
        }

        Ok(())
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

        // Set secure file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Set key file to 600 (read/write for owner only)
            let mut perms = fs::metadata(&key_file)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_file, perms)?;

            // Set info file to 644 (read/write for owner, read for others)
            let mut perms = fs::metadata(&info_file)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&info_file, perms)?;
        }

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

        // Verify key integrity
        let calculated_hash = Self::calculate_key_integrity(&key);
        if calculated_hash != key_info.integrity_hash {
            return Err(anyhow::anyhow!(
                "Key integrity verification failed - key may be corrupted"
            )
            .into());
        }

        // Validate key strength
        if let Err(e) = Self::validate_key_strength(&key) {
            eprintln!("⚠️  Warning: Loaded key has weak strength: {}", e);
        }

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

    /// Securely delete a key file (legacy format)
    pub fn secure_delete_key(&self, keys_dir: &Path) -> Result<(), Box<dyn Error>> {
        let key_file = keys_dir.join(format!("{}.key", self.key_info.name));
        let info_file = keys_dir.join(format!("{}.info", self.key_info.name));

        // Secure overwrite key file
        if key_file.exists() {
            let file_size = fs::metadata(&key_file)?.len() as usize;

            // Multiple pass secure overwrite
            for pass in 0..3 {
                let overwrite_data = match pass {
                    0 => vec![0x00; file_size], // All zeros
                    1 => vec![0xFF; file_size], // All ones
                    2 => {
                        // Random data
                        let mut random_data = vec![0u8; file_size];
                        secure_random::fill_secure_bytes(&mut random_data);
                        random_data
                    }
                    _ => unreachable!(),
                };

                fs::write(&key_file, &overwrite_data)?;

                // Force sync to disk
                #[cfg(unix)]
                {
                    use std::os::unix::io::AsRawFd;
                    let file = std::fs::OpenOptions::new().write(true).open(&key_file)?;
                    unsafe {
                        libc::fsync(file.as_raw_fd());
                    }
                }
            }

            fs::remove_file(&key_file)?;
        }

        // Remove info file
        if info_file.exists() {
            fs::remove_file(&info_file)?;
        }

        Ok(())
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

pub async fn setup_key_management_with_mode(
    cli: &Cli,
    mode: KeySelectionMode,
) -> Result<KeyManager, Box<dyn Error>> {
    let keys_dir = cli
        .keys_dir
        .clone()
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
                    println!(
                        "Created: {}",
                        key_manager.key_info.created.format("%Y-%m-%d %H:%M:%S")
                    );
                }
                return Ok(key_manager);
            }
            Err(e) => {
                if matches!(mode, KeySelectionMode::NonInteractive) {
                    // In non-interactive mode, generate a new key if the specified one doesn't exist
                    let secret_key = SecretKey::random(&mut OsRng);
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

    if available_keys.is_empty() {
        if !matches!(mode, KeySelectionMode::NonInteractive) {
            println!("No ECIES keys found in the keys directory.");
            println!("You need to generate a new key to use the DFS system.");
        }

        let key_name = if let Some(name) = &cli.key_name {
            name.clone()
        } else {
            match mode {
                KeySelectionMode::NonInteractive => generate_default_key_name(),
                KeySelectionMode::Interactive => {
                    let default_name = generate_default_key_name();
                    let input = prompt_user_input(&format!(
                        "Enter a name for the new key (default: {}): ",
                        default_name
                    ))?;
                    if input.is_empty() {
                        default_name
                    } else {
                        input
                    }
                }
            }
        };

        if !matches!(mode, KeySelectionMode::NonInteractive) {
            println!("Generating new ECIES key pair...");
        }
        let secret_key = SecretKey::random(&mut OsRng);
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
                    println!(
                        "  {}. {} (created: {}, public: {}...)",
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
                println!(
                    "Invalid choice. Please enter a number between 1 and {}.",
                    available_keys.len() + 1
                );
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
                    "Enter a name for the new key (default: {}): ",
                    default_name
                ))?;
                let key_name = if key_name.is_empty() {
                    default_name
                } else {
                    key_name
                };

                println!("Generating new ECIES key pair...");
                let secret_key = SecretKey::random(&mut OsRng);
                let key_manager = KeyManager::new(secret_key, key_name);

                key_manager.save_to_file(&keys_dir)?;
                println!("New key generated and saved successfully!");
                println!("Public key: {}", key_manager.key_info.public_key_hex);

                Ok(key_manager)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_key_generation_and_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        // Test key creation
        let secret_key = SecretKey::random(&mut OsRng);
        let key_manager = KeyManager::new(secret_key, "test_key".to_string());

        // Test saving
        key_manager.save_to_file(keys_dir).unwrap();

        // Test loading
        let loaded_manager = KeyManager::load_from_file(keys_dir, "test_key").unwrap();

        // Verify keys match
        assert_eq!(key_manager.key.serialize(), loaded_manager.key.serialize());
        assert_eq!(key_manager.key_info.name, loaded_manager.key_info.name);
        assert_eq!(
            key_manager.key_info.public_key_hex,
            loaded_manager.key_info.public_key_hex
        );
    }

    #[test]
    fn test_key_listing() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        // Create multiple keys
        let key1 = KeyManager::new(
            SecretKey::random(&mut rand::thread_rng()),
            "key1".to_string(),
        );
        let key2 = KeyManager::new(
            SecretKey::random(&mut rand::thread_rng()),
            "key2".to_string(),
        );

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
        let secret_key = SecretKey::random(&mut OsRng);
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
        let secret_key = SecretKey::random(&mut OsRng);
        let key_manager = KeyManager::new(secret_key, "test".to_string());

        // Test with no specific public key (should use default)
        let (pub_key, hex) = get_encryption_key(&None, &key_manager).unwrap();
        assert_eq!(hex, key_manager.key_info.public_key_hex);

        // Test with specific public key
        let custom_key = SecretKey::random(&mut rand::thread_rng());
        let custom_pub = PublicKey::from_secret_key(&custom_key);
        let custom_hex = hex::encode(custom_pub.serialize());

        let (parsed_pub, parsed_hex) =
            get_encryption_key(&Some(custom_hex.clone()), &key_manager).unwrap();
        assert_eq!(parsed_hex, custom_hex);
        assert_eq!(parsed_pub.serialize(), custom_pub.serialize());
    }
}
