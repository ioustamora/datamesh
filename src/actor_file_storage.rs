/// Actor-based File Storage Module
///
/// This module provides file storage operations using the network actor
/// for thread-safe network communication instead of sharing the Swarm directly.

use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use libp2p::kad::{Record, RecordKey, Quorum};
use reed_solomon_erasure::ReedSolomon;
use chrono::Local;

use crate::network_actor::NetworkHandle;
use crate::key_manager::{KeyManager, get_encryption_key, get_decryption_key};
use crate::cli::Cli;
use crate::config::Config;
use crate::thread_safe_database::ThreadSafeDatabaseManager;
use crate::file_storage::{StoredFile, DATA_SHARDS, PARITY_SHARDS};
use crate::logging::log_file_operation;
use crate::error::{DfsError, DfsResult};
use crate::ui;
use crate::performance;
use ecies::{encrypt, decrypt};

/// Actor-based file storage operations
pub struct ActorFileStorage {
    network: NetworkHandle,
    db: ThreadSafeDatabaseManager,
}

impl ActorFileStorage {
    /// Create a new actor-based file storage instance
    pub async fn new(cli: &Cli, config: &Config) -> Result<Self> {
        let network = NetworkHandle::new(cli, config).await?;
        let db_path = crate::database::get_default_db_path()?;
        let db = ThreadSafeDatabaseManager::new(&db_path.to_string_lossy())?;
        
        Ok(Self { network, db })
    }
    
    /// Store a file using the network actor
    pub async fn put_file(
        &self,
        path: &PathBuf,
        key_manager: &KeyManager,
        public_key: &Option<String>,
        name: &Option<String>,
        tags: &Option<String>,
    ) -> DfsResult<String> {
        let _timer = performance::start_operation("actor_file_put");
        
        ui::print_info("Reading file...");
        let file_data = fs::read(path)?;
        let file_size = file_data.len() as u64;
        
        // Create progress bar
        let progress = ui::ProgressManager::new_upload(file_size);
        
        let file_key = RecordKey::new(&blake3::hash(&file_data).as_bytes());
        
        // Get the encryption key (either specified public key or default)
        let (encryption_public_key, public_key_hex) = get_encryption_key(public_key, key_manager)?;
        let encrypted_data = encrypt(&encryption_public_key.serialize(), &file_data)
            .map_err(|e| DfsError::Crypto(format!("Encryption error: {:?}", e)))?;
        
        // Create Reed-Solomon encoder
        let r = ReedSolomon::<reed_solomon_erasure::galois_8::Field>::new(DATA_SHARDS, PARITY_SHARDS)?;
        let chunk_size = (encrypted_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
        
        // Create shards
        let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; DATA_SHARDS + PARITY_SHARDS];
        
        // Fill data shards
        for (i, shard) in shards.iter_mut().enumerate().take(DATA_SHARDS) {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, encrypted_data.len());
            if start < encrypted_data.len() {
                shard[..end - start].copy_from_slice(&encrypted_data[start..end]);
            }
        }
        
        // Encode to create parity shards
        r.encode(&mut shards)?;
        
        // Store each shard using the network actor
        let mut chunk_keys = Vec::new();
        let total_shards = shards.len();
        progress.set_message("Storing chunks...");
        
        for (i, shard) in shards.into_iter().enumerate() {
            let chunk_key = RecordKey::new(&blake3::hash(&shard).as_bytes());
            chunk_keys.push(chunk_key.as_ref().to_vec());
            
            let record = Record {
                key: chunk_key,
                value: shard,
                publisher: None,
                expires: None,
            };
            
            // Use network actor to store the record
            self.network.put_record(record, Quorum::One).await?;
            
            // Update progress
            let progress_value = ((i + 1) as f64 / total_shards as f64 * file_size as f64) as u64;
            progress.set_position(progress_value);
        }
        
        // Store file metadata using network actor
        let stored_file = StoredFile {
            chunk_keys,
            encryption_key: key_manager.key.serialize().to_vec(),
            file_size: file_data.len(),
            public_key_hex: public_key_hex.clone(),
            file_name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            stored_at: Local::now(),
        };
        
        let record = Record {
            key: file_key.clone(),
            value: serde_json::to_vec(&stored_file)?,
            publisher: None,
            expires: None,
        };
        
        self.network.put_record(record, Quorum::One).await?;
        
        // Generate or use provided name
        let original_filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_name = if let Some(provided_name) = name {
            if self.db.is_name_taken(provided_name)? {
                return Err(crate::error_handling::storage_error_with_suggestions(
                    &format!("Name '{}' is already taken", provided_name)
                ).error);
            }
            provided_name.clone()
        } else {
            self.db.generate_unique_name(&original_filename)?
        };
        
        // Parse tags
        let file_tags = if let Some(tag_str) = tags {
            tag_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };
        
        // Store in database
        let upload_time = Local::now();
        let file_entry = crate::database::FileEntry {
            id: 0, // Will be auto-assigned by database
            name: file_name.clone(),
            file_key: hex::encode(file_key.as_ref()),
            original_filename: original_filename.clone(),
            file_size: file_size as u64,
            upload_time,
            tags: file_tags.clone(),
            public_key_hex: public_key_hex.clone(),
            chunks_total: 6,
            chunks_healthy: 6,
        };
        self.db.store_file(file_entry)?;
        
        progress.finish_with_message("Upload complete!");
        
        crate::logging::log_file_operation("store", &original_filename, &format!("size: {} bytes", file_data.len()));
        
        ui::print_success(&format!("File stored successfully as '{}'", file_name));
        println!("  Original: {}", original_filename);
        println!("  Size: {}", ui::format_file_size(file_size));
        println!("  Key: {}", hex::encode(file_key.as_ref()));
        if !file_tags.is_empty() {
            println!("  Tags: {}", file_tags.join(", "));
        }
        
        Ok(file_name)
    }
    
    /// Retrieve a file using the network actor
    pub async fn get_file(
        &self,
        identifier: &str,
        output_path: &PathBuf,
        key_manager: &KeyManager,
        private_key: &Option<String>,
    ) -> DfsResult<()> {
        let _timer = performance::start_operation("actor_file_get");
        
        // Try to resolve identifier to a file key
        let file_key = if let Some(file_entry) = self.db.get_file_by_name(identifier)? {
            ui::print_info(&format!("Found file '{}' in database", identifier));
            file_entry.file_key
        } else if let Some(_) = self.db.get_file_by_key(identifier)? {
            ui::print_info("Using provided file key");
            identifier.to_string()
        } else {
            ui::print_info("Treating as direct file key");
            identifier.to_string()
        };
        
        // Decode the file key
        let key_bytes = hex::decode(&file_key)?;
        let record_key = RecordKey::from(key_bytes);
        
        ui::print_info("Retrieving file metadata...");
        
        // Get file metadata using network actor
        let metadata_record = self.network.get_record(record_key).await?;
        
        let stored_file = match metadata_record {
            Some(record) => {
                serde_json::from_slice::<StoredFile>(&record.value)
                    .map_err(|e| DfsError::Serialization(format!("Failed to parse metadata: {}", e)))?
            }
            None => {
                return Err(DfsError::FileNotFound(format!("File metadata not found for key: {}", file_key)));
            }
        };
        
        ui::print_info(&format!("Retrieving {} chunks...", stored_file.chunk_keys.len()));
        
        // Create progress bar
        let progress = ui::ProgressManager::new_download(stored_file.file_size as u64);
        
        // Retrieve all chunks using network actor
        let mut chunks = vec![None; stored_file.chunk_keys.len()];
        
        for (i, chunk_key_bytes) in stored_file.chunk_keys.iter().enumerate() {
            let chunk_key = RecordKey::from(chunk_key_bytes.clone());
            
            match self.network.get_record(chunk_key).await? {
                Some(record) => {
                    chunks[i] = Some(record.value);
                    let progress_value = ((i + 1) as f64 / stored_file.chunk_keys.len() as f64 * stored_file.file_size as f64) as u64;
                    progress.set_position(progress_value);
                }
                None => {
                    return Err(DfsError::Network(format!("Chunk {} not found", i)));
                }
            }
        }
        
        progress.set_message("Reconstructing file...");
        
        // Reconstruct the file
        let mut shards: Vec<Option<Vec<u8>>> = chunks.into_iter().map(|c| c).collect();
        
        // Use Reed-Solomon to recover any missing shards
        let r = ReedSolomon::<reed_solomon_erasure::galois_8::Field>::new(DATA_SHARDS, PARITY_SHARDS)?;
        r.reconstruct_data(&mut shards)?;
        
        // Combine data shards
        let mut decrypted_data = Vec::new();
        for shard_opt in shards.iter().take(DATA_SHARDS) {
            if let Some(shard) = shard_opt {
                decrypted_data.extend_from_slice(shard);
            }
        }
        
        // Trim to actual file size
        decrypted_data.truncate(stored_file.file_size);
        
        // Decrypt the data
        let decryption_key = if let Some(pk) = private_key {
            let keys_dir = PathBuf::from("./keys"); // Default keys directory
            get_decryption_key(&Some(pk.clone()), key_manager, &keys_dir)?
        } else {
            ecies::SecretKey::parse_slice(&stored_file.encryption_key)
                .map_err(|e| DfsError::Crypto(format!("Failed to parse encryption key: {:?}", e)))?
        };
        
        let file_data = decrypt(&decryption_key.serialize(), &decrypted_data)
            .map_err(|e| DfsError::Crypto(format!("Decryption error: {:?}", e)))?;
        
        // Write to output file
        fs::write(output_path, &file_data)?;
        
        progress.finish_with_message("Download complete!");
        
        crate::logging::log_file_operation("retrieve", &stored_file.file_name, &format!("size: {} bytes", file_data.len()));
        
        ui::print_success(&format!("File retrieved successfully: {}", output_path.display()));
        
        Ok(())
    }
    
    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<crate::network_actor::NetworkStats> {
        self.network.get_network_stats().await
            .map_err(|e| anyhow::anyhow!(e))
    }
    
    /// Bootstrap the network
    pub async fn bootstrap(&self) -> DfsResult<()> {
        self.network.bootstrap().await
    }
    
    /// Get connected peers
    pub async fn get_connected_peers(&self) -> DfsResult<Vec<libp2p::PeerId>> {
        self.network.get_connected_peers().await
    }
}

/// Store a file using the actor-based network system
pub async fn store_file_with_network(
    _cli: &Cli,
    key_manager: &KeyManager,
    file_path: &PathBuf,
    public_key: &Option<String>,
    name: &Option<String>,
    tags: &Option<Vec<String>>,
    network: std::sync::Arc<NetworkHandle>,
    database: std::sync::Arc<ThreadSafeDatabaseManager>,
) -> DfsResult<String> {
    // Implementation using actor-based network operations
    // This avoids the Swarm Send/Sync issues by using message passing
    
    let file_data = std::fs::read(file_path)
        .map_err(|e| DfsError::Io(format!("Failed to read file: {}", e)))?;
    
    let original_filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Use the provided name or fall back to filename
    let display_name = name.as_ref().unwrap_or(&original_filename);
    
    // Generate encryption key
    let encryption_key = get_encryption_key(public_key, key_manager)?;
    
    // Encrypt the file data
    let encrypted_data = encrypt(&encryption_key.0.serialize(), &file_data)
        .map_err(|e| DfsError::Encryption(format!("Encryption failed: {:?}", e)))?;
    
    // Create Reed-Solomon encoder for erasure coding
    let reed_solomon = ReedSolomon::<reed_solomon_erasure::galois_8::Field>::new(DATA_SHARDS, PARITY_SHARDS)
        .map_err(|e| DfsError::Encoding(format!("Reed-Solomon setup failed: {}", e)))?;
    
    let chunk_size = (encrypted_data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; DATA_SHARDS + PARITY_SHARDS];
    
    // Fill data shards
    for (i, chunk) in encrypted_data.chunks(chunk_size).enumerate() {
        shards[i][..chunk.len()].copy_from_slice(chunk);
    }
    
    // Generate parity shards
    reed_solomon.encode(&mut shards)
        .map_err(|e| DfsError::Encoding(format!("Reed-Solomon encoding failed: {}", e)))?;
    
    // Store chunks in the DHT using the actor-based network
    let mut chunk_keys = Vec::new();
    for shard in shards {
        let chunk_key = RecordKey::new(&blake3::hash(&shard).as_bytes());
        let record = Record {
            key: chunk_key.clone(),
            value: shard,
            publisher: None,
            expires: None,
        };
        
        // Store using actor-based network (thread-safe)
        network.put_record(record, Quorum::One).await?;
        chunk_keys.push(chunk_key.as_ref().to_vec());
    }
    
    // Create file metadata
    let file_key = blake3::hash(&encrypted_data).to_hex().to_string();
    let stored_file = crate::file_storage::StoredFile {
        chunk_keys,
        encryption_key: encryption_key.0.serialize().to_vec(),
        file_size: file_data.len(),
        public_key_hex: encryption_key.1.clone(),
        file_name: display_name.clone(),
        stored_at: Local::now(),
    };
    
    // Store metadata in DHT
    let metadata_key = RecordKey::new(&file_key.as_bytes());
    let metadata_record = Record {
        key: metadata_key,
        value: serde_json::to_vec(&stored_file)
            .map_err(|e| DfsError::Serialization(format!("Failed to serialize metadata: {}", e)))?,
        publisher: None,
        expires: None,
    };
    
    network.put_record(metadata_record, Quorum::One).await?;
    
    // Store in database
    let chunk_size = 1024 * 1024; // 1MB chunks
    let total_chunks = (file_data.len() + chunk_size - 1) / chunk_size;
    let file_entry = crate::database::FileEntry {
        id: 0,
        name: file_key.clone(),
        file_key: file_key.clone(),
        original_filename: original_filename.clone(),
        file_size: file_data.len() as u64,
        upload_time: Local::now(),
        tags: tags.clone().unwrap_or_default(),
        public_key_hex: "".to_string(),
        chunks_total: total_chunks as u32,
        chunks_healthy: total_chunks as u32,
    };
    
    database.store_file(file_entry)?;
    
    // Log the operation
    log_file_operation("store", &original_filename, &format!("size: {} bytes", file_data.len()));
    
    Ok(file_key)
}

/// Retrieve a file using the actor-based network system
pub async fn retrieve_file_with_network(
    _cli: &Cli,
    key_manager: &KeyManager,
    identifier: &str,
    output_path: &PathBuf,
    private_key: &Option<String>,
    network: std::sync::Arc<NetworkHandle>,
    _database: std::sync::Arc<ThreadSafeDatabaseManager>,
) -> DfsResult<()> {
    // Get file metadata from DHT
    let metadata_key = RecordKey::new(&identifier.as_bytes());
    let metadata_record = network.get_record(metadata_key).await?
        .ok_or_else(|| DfsError::NotFound("File metadata not found".to_string()))?;
    
    // Deserialize metadata
    let stored_file: crate::file_storage::StoredFile = serde_json::from_slice(&metadata_record.value)
        .map_err(|e| DfsError::Deserialization(format!("Failed to deserialize metadata: {}", e)))?;
    
    // Get chunks from DHT
    let mut chunks = Vec::new();
    for chunk_key_bytes in &stored_file.chunk_keys {
        let chunk_key = RecordKey::from(chunk_key_bytes.clone());
        let record = network.get_record(chunk_key).await?
            .ok_or_else(|| DfsError::NotFound("Chunk not found".to_string()))?;
        chunks.push(record.value);
    }
    
    // Reconstruct file using Reed-Solomon
    let reed_solomon = ReedSolomon::<reed_solomon_erasure::galois_8::Field>::new(DATA_SHARDS, PARITY_SHARDS)
        .map_err(|e| DfsError::Encoding(format!("Reed-Solomon setup failed: {}", e)))?;
    
    let mut shards: Vec<Option<Vec<u8>>> = chunks.into_iter().map(|chunk| Some(chunk)).collect();
    reed_solomon.reconstruct(&mut shards)
        .map_err(|e| DfsError::Encoding(format!("Reed-Solomon reconstruction failed: {}", e)))?;
    
    // Combine data shards
    let mut encrypted_data = Vec::new();
    for shard_opt in shards.iter().take(DATA_SHARDS) {
        if let Some(shard) = shard_opt {
            encrypted_data.extend_from_slice(shard);
        }
    }
    
    // Trim to actual file size
    encrypted_data.truncate(stored_file.file_size as usize);
    
    // Decrypt the file
    let decryption_key = get_decryption_key(private_key, key_manager, &PathBuf::from("./keys"))?;
    let file_data = decrypt(&decryption_key.serialize(), &encrypted_data)
        .map_err(|e| DfsError::Decryption(format!("Decryption failed: {:?}", e)))?;
    
    // Write to output file
    let file_size = file_data.len();
    std::fs::write(output_path, file_data)
        .map_err(|e| DfsError::Io(format!("Failed to write file: {}", e)))?;
    
    // Log the operation
    log_file_operation("retrieve", &stored_file.file_name, &format!("size: {} bytes", file_size));
    
    Ok(())
}
