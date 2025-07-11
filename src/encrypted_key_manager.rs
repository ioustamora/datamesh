use crate::secure_random;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
/// Encrypted Key Manager Module
///
/// This module provides password-protected key storage for enhanced security.
/// It implements the security recommendations from the hardening roadmap,
/// including Argon2 password hashing and AES-256-GCM encryption.
use anyhow::Result;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use ecies::SecretKey;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::key_manager::KeyManager;

/// Encrypted key file structure
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedKeyFile {
    /// AES-256-GCM encrypted key data
    pub ciphertext: Vec<u8>,
    /// Argon2 salt for password derivation
    pub salt: Vec<u8>,
    /// AES-GCM nonce for key encryption
    pub nonce: Vec<u8>,
    /// AES-GCM nonce for integrity hash encryption
    pub integrity_nonce: Vec<u8>,
    /// Encryption algorithm identifier
    pub algorithm: String,
    /// Key creation timestamp
    pub created: DateTime<Utc>,
    /// Argon2 password hash for verification
    pub password_hash: String,
    /// Key integrity hash (encrypted)
    pub integrity_hash: Vec<u8>,
}

/// Password-protected key manager
pub struct EncryptedKeyManager {
    pub name: String,
    pub created: DateTime<Utc>,
    key_data: Option<Vec<u8>>,
    password_hash: String,
    salt: Vec<u8>,
    nonce: Vec<u8>,
    integrity_nonce: Vec<u8>,
    integrity_hash: Vec<u8>,
}

impl EncryptedKeyManager {
    /// Create a new encrypted key manager from a secret key
    pub fn new(key: &SecretKey, name: String, password: &str) -> Result<Self> {
        let mut salt = vec![0u8; 32];
        let mut nonce = vec![0u8; 12];
        let mut integrity_nonce = vec![0u8; 12];

        // Generate random salt and nonces
        secure_random::fill_secure_bytes(&mut salt);
        secure_random::fill_secure_bytes(&mut nonce);
        secure_random::fill_secure_bytes(&mut integrity_nonce);

        // Hash password with Argon2
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(&salt)
            .map_err(|e| anyhow::anyhow!("Failed to encode salt: {}", e))?;
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        // Derive encryption key from password
        let derived_key = Self::derive_key_from_password(password, &salt)?;

        // Encrypt the secret key
        let key_data = key.serialize().to_vec();
        let cipher = Aes256Gcm::new(&derived_key);
        let nonce_array = Nonce::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_array, key_data.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Create integrity hash with different nonce
        let integrity_hash = blake3::hash(&key_data);
        let integrity_nonce_array = Nonce::from_slice(&integrity_nonce);
        let encrypted_integrity = cipher
            .encrypt(integrity_nonce_array, &integrity_hash.as_bytes()[..])
            .map_err(|e| anyhow::anyhow!("Integrity hash encryption failed: {}", e))?;

        Ok(EncryptedKeyManager {
            name,
            created: Utc::now(),
            key_data: Some(ciphertext),
            password_hash: password_hash.to_string(),
            salt,
            nonce,
            integrity_nonce,
            integrity_hash: encrypted_integrity,
        })
    }

    /// Derive an encryption key from password and salt using Argon2
    fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>> {
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| anyhow::anyhow!("Failed to encode salt: {}", e))?;
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        // Use the hash bytes as the encryption key
        let hash_output = hash.hash.unwrap();
        let hash_bytes = hash_output.as_bytes();
        if hash_bytes.len() < 32 {
            return Err(anyhow::anyhow!("Derived key too short"));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash_bytes[..32]);
        Ok(Key::<Aes256Gcm>::from(key_bytes))
    }

    /// Save encrypted key to file
    pub fn save_to_file(&self, keys_dir: &Path) -> Result<()> {
        fs::create_dir_all(keys_dir)?;

        let key_file = keys_dir.join(format!("{}.key", self.name));

        let encrypted_key = EncryptedKeyFile {
            ciphertext: self.key_data.as_ref().unwrap().clone(),
            salt: self.salt.clone(),
            nonce: self.nonce.clone(),
            integrity_nonce: self.integrity_nonce.clone(),
            algorithm: "argon2id+aes256gcm".to_string(),
            created: self.created,
            password_hash: self.password_hash.clone(),
            integrity_hash: self.integrity_hash.clone(),
        };

        // Serialize to binary format
        let encrypted_data = bincode::serialize(&encrypted_key)?;
        fs::write(&key_file, encrypted_data)?;

        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_file)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_file, perms)?;
        }

        println!("Encrypted key saved: {}", key_file.display());
        Ok(())
    }

    /// Load encrypted key from file and decrypt with password
    pub fn load_from_file(keys_dir: &Path, name: &str, password: &str) -> Result<KeyManager> {
        let key_file = keys_dir.join(format!("{}.key", name));

        if !key_file.exists() {
            return Err(anyhow::anyhow!(
                "Key file not found: {}",
                key_file.display()
            ));
        }

        // Read encrypted data
        let encrypted_data = fs::read(&key_file)?;
        let encrypted_key: EncryptedKeyFile = bincode::deserialize(&encrypted_data)?;

        // Verify password
        let parsed_hash = PasswordHash::new(&encrypted_key.password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("Invalid password"))?;

        // Derive decryption key
        let derived_key = Self::derive_key_from_password(password, &encrypted_key.salt)?;

        // Decrypt the secret key
        let cipher = Aes256Gcm::new(&derived_key);
        let nonce = Nonce::from_slice(&encrypted_key.nonce);
        let plaintext = cipher
            .decrypt(nonce, encrypted_key.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        // Decrypt and verify integrity hash
        let integrity_nonce = Nonce::from_slice(&encrypted_key.integrity_nonce);
        let decrypted_integrity = cipher
            .decrypt(integrity_nonce, encrypted_key.integrity_hash.as_ref())
            .map_err(|e| anyhow::anyhow!("Integrity hash decryption failed: {}", e))?;

        let expected_integrity = blake3::hash(&plaintext);
        if decrypted_integrity != expected_integrity.as_bytes() {
            return Err(anyhow::anyhow!("Key integrity verification failed"));
        }

        // Parse the secret key
        let secret_key = SecretKey::parse_slice(&plaintext)
            .map_err(|e| anyhow::anyhow!("Failed to parse secret key: {:?}", e))?;

        Ok(KeyManager::new(secret_key, name.to_string()))
    }

    /// Check if a key file is encrypted (new format)
    pub fn is_encrypted_key_file(keys_dir: &Path, name: &str) -> bool {
        let key_file = keys_dir.join(format!("{}.key", name));

        if !key_file.exists() {
            return false;
        }

        // Try to deserialize as encrypted key file
        if let Ok(data) = fs::read(&key_file) {
            if let Ok(_) = bincode::deserialize::<EncryptedKeyFile>(&data) {
                return true;
            }
        }

        false
    }

    /// Migrate a legacy plain-text key to encrypted format
    pub fn migrate_legacy_key(keys_dir: &Path, name: &str, password: &str) -> Result<()> {
        // Check if already encrypted
        if Self::is_encrypted_key_file(keys_dir, name) {
            return Err(anyhow::anyhow!("Key is already encrypted"));
        }

        // Load the legacy key
        let legacy_key = KeyManager::load_from_file(keys_dir, name)
            .map_err(|e| anyhow::anyhow!("Failed to load legacy key: {}", e))?;

        // Create encrypted version
        let encrypted_manager = Self::new(&legacy_key.key, name.to_string(), password)?;

        // Backup original files
        let key_file = keys_dir.join(format!("{}.key", name));
        let info_file = keys_dir.join(format!("{}.info", name));
        let backup_key = keys_dir.join(format!("{}.key.backup", name));
        let backup_info = keys_dir.join(format!("{}.info.backup", name));

        if key_file.exists() {
            fs::copy(&key_file, &backup_key)?;
        }
        if info_file.exists() {
            fs::copy(&info_file, &backup_info)?;
        }

        // Save encrypted version
        encrypted_manager.save_to_file(keys_dir)?;

        // Clean up legacy info file if it exists
        if info_file.exists() {
            fs::remove_file(&info_file)?;
        }

        println!("Key '{}' migrated to encrypted format", name);
        println!("Legacy key backed up as: {}", backup_key.display());

        Ok(())
    }

    /// Change password for an encrypted key
    pub fn change_password(
        keys_dir: &Path,
        name: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Load with old password
        let key_manager = Self::load_from_file(keys_dir, name, old_password)?;

        // Create new encrypted version with new password
        let encrypted_manager = Self::new(&key_manager.key, name.to_string(), new_password)?;

        // Save with new password
        encrypted_manager.save_to_file(keys_dir)?;

        println!("Password changed for key '{}'", name);
        Ok(())
    }

    /// List all encrypted keys in the directory
    pub fn list_encrypted_keys(keys_dir: &Path) -> Result<Vec<String>> {
        if !keys_dir.exists() {
            return Ok(Vec::new());
        }

        let mut encrypted_keys = Vec::new();

        for entry in fs::read_dir(keys_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "key") {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        if Self::is_encrypted_key_file(keys_dir, name) {
                            encrypted_keys.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(encrypted_keys)
    }

    /// Get metadata for an encrypted key file
    pub fn get_encrypted_key_info(keys_dir: &Path, name: &str) -> Result<(DateTime<Utc>, String)> {
        let key_file = keys_dir.join(format!("{}.key", name));
        let encrypted_data = fs::read(&key_file)?;
        let encrypted_key: EncryptedKeyFile = bincode::deserialize(&encrypted_data)?;

        Ok((encrypted_key.created, encrypted_key.algorithm))
    }

    /// Securely delete an encrypted key file
    pub fn secure_delete_key(keys_dir: &Path, name: &str) -> Result<()> {
        let key_file = keys_dir.join(format!("{}.key", name));

        if !key_file.exists() {
            return Err(anyhow::anyhow!(
                "Key file not found: {}",
                key_file.display()
            ));
        }

        // Get file size for secure overwrite
        let file_size = fs::metadata(&key_file)?.len() as usize;

        // Perform multiple passes of secure overwrite
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

            // Overwrite file with pattern
            fs::write(&key_file, &overwrite_data)?;

            // Force sync to disk to ensure data is written
            #[cfg(unix)]
            {
                use std::os::unix::io::AsRawFd;
                let file = std::fs::OpenOptions::new().write(true).open(&key_file)?;
                unsafe {
                    libc::fsync(file.as_raw_fd());
                }
            }
        }

        // Finally remove the file
        fs::remove_file(&key_file)?;

        // Also remove any backup files
        let backup_files = [
            keys_dir.join(format!("{}.key.backup", name)),
            keys_dir.join(format!("{}.info.backup", name)),
            keys_dir.join(format!("{}.info", name)), // Legacy info file
        ];

        for backup_file in &backup_files {
            if backup_file.exists() {
                // Secure overwrite backup files too
                if let Ok(metadata) = fs::metadata(backup_file) {
                    let size = metadata.len() as usize;
                    let random_data = secure_random::generate_secure_bytes(size);
                    let _ = fs::write(backup_file, &random_data);
                }
                let _ = fs::remove_file(backup_file);
            }
        }

        println!("Key '{}' securely deleted", name);
        Ok(())
    }
}

/// Helper function to prompt for password securely
pub fn prompt_password(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{}", prompt);
    io::stdout().flush()?;

    // Try to use rpassword for hidden input, fallback to regular input
    match rpassword::read_password() {
        Ok(password) => Ok(password),
        Err(_) => {
            // Fallback to regular input if rpassword fails
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_manager::KeyManager;
    use rand::rngs::OsRng;
    use tempfile::TempDir;

    #[test]
    fn test_encrypted_key_creation_and_loading() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        // Create a test key
        let secret_key = SecretKey::random(&mut OsRng);
        let password = "test_password_123";

        // Create encrypted manager
        let encrypted_manager =
            EncryptedKeyManager::new(&secret_key, "test_key".to_string(), password).unwrap();

        // Save to file
        encrypted_manager.save_to_file(keys_dir).unwrap();

        // Load from file
        let loaded_manager =
            EncryptedKeyManager::load_from_file(keys_dir, "test_key", password).unwrap();

        // Verify keys match
        assert_eq!(secret_key.serialize(), loaded_manager.key.serialize());
        assert_eq!("test_key", loaded_manager.key_info.name);
    }

    #[test]
    fn test_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        let secret_key = SecretKey::random(&mut OsRng);
        let password = "correct_password";

        let encrypted_manager =
            EncryptedKeyManager::new(&secret_key, "test_key".to_string(), password).unwrap();
        encrypted_manager.save_to_file(keys_dir).unwrap();

        // Try to load with wrong password
        let result = EncryptedKeyManager::load_from_file(keys_dir, "test_key", "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_change() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        let secret_key = SecretKey::random(&mut OsRng);
        let old_password = "old_password";
        let new_password = "new_password";

        // Create with old password
        let encrypted_manager =
            EncryptedKeyManager::new(&secret_key, "test_key".to_string(), old_password).unwrap();
        encrypted_manager.save_to_file(keys_dir).unwrap();

        // Change password
        EncryptedKeyManager::change_password(keys_dir, "test_key", old_password, new_password)
            .unwrap();

        // Verify old password no longer works
        assert!(EncryptedKeyManager::load_from_file(keys_dir, "test_key", old_password).is_err());

        // Verify new password works
        let loaded_manager =
            EncryptedKeyManager::load_from_file(keys_dir, "test_key", new_password).unwrap();
        assert_eq!(secret_key.serialize(), loaded_manager.key.serialize());
    }

    #[test]
    fn test_legacy_key_migration() {
        let temp_dir = TempDir::new().unwrap();
        let keys_dir = temp_dir.path();

        // Create a legacy key
        let secret_key = SecretKey::random(&mut OsRng);
        let legacy_manager = KeyManager::new(secret_key, "legacy_key".to_string());
        legacy_manager.save_to_file(keys_dir).unwrap();

        // Migrate to encrypted format
        let password = "migration_password";
        EncryptedKeyManager::migrate_legacy_key(keys_dir, "legacy_key", password).unwrap();

        // Verify encrypted key works
        let loaded_manager =
            EncryptedKeyManager::load_from_file(keys_dir, "legacy_key", password).unwrap();
        assert_eq!(secret_key.serialize(), loaded_manager.key.serialize());

        // Verify backup files exist
        assert!(keys_dir.join("legacy_key.key.backup").exists());
    }
}
