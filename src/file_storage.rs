/// File Storage Module
///
/// This module implements the core file storage and retrieval functionality for DFS.
/// It handles:
/// - File encryption using ECIES
/// - File chunking and erasure coding using Reed-Solomon
/// - Storing file chunks in the Kademlia DHT
/// - Retrieving and reassembling files from the network
/// - Error handling and retry logic for resilient operations
/// 
/// The implementation provides both synchronous and asynchronous interfaces for maximum flexibility.

use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;

use chrono::{DateTime, Local};
use ecies::{decrypt, encrypt, SecretKey};
use futures::StreamExt;
use libp2p::kad::{Quorum, Record, RecordKey};
use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::warn;

use crate::cli::Cli;
use crate::config::Config;
use crate::database::DatabaseManager;
use crate::error::{DfsError, DfsResult};
use crate::key_manager::{KeyManager, get_encryption_key};
use crate::network::create_swarm_and_connect_multi_bootstrap;
use crate::concurrent_chunks::ConcurrentChunkManager;
use crate::database;
use crate::ui;
use crate::smart_cache::{SmartCacheManager, AccessType};
use crate::performance;
use crate::resilience::{retry_async, RetryConfig};
use crate::logging::log_file_operation;

/// Number of data shards for Reed-Solomon erasure coding
const DATA_SHARDS: usize = 4;
/// Number of parity shards for Reed-Solomon erasure coding
const PARITY_SHARDS: usize = 2;

/// Public constant for data shards available to other modules
pub const PUB_DATA_SHARDS: usize = DATA_SHARDS;
/// Public constant for parity shards available to other modules
pub const PUB_PARITY_SHARDS: usize = PARITY_SHARDS;

/// Represents a file stored in the distributed file system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFile {
    /// Keys for all chunks of the file in the DHT
    pub chunk_keys: Vec<Vec<u8>>, // Store as Vec<u8> instead of RecordKey for serialization
    /// Encryption key used for this file
    pub encryption_key: Vec<u8>,
    /// Original file size in bytes
    pub file_size: usize,
    /// Hex-encoded public key used for encryption
    pub public_key_hex: String,
    /// Original file name
    pub file_name: String,
    /// Timestamp when the file was stored
    pub stored_at: DateTime<Local>,
}

#[derive(Debug)]
pub struct FileRetrieval {
    pub stored_file: StoredFile,
    pub chunks: HashMap<RecordKey, Vec<u8>>,
    pub output_path: PathBuf,
}

pub async fn handle_put_command(
    cli: &Cli,
    key_manager: &KeyManager,
    path: &PathBuf,
    public_key: &Option<String>,
    name: &Option<String>,
    tags: &Option<String>,
) -> DfsResult<()> {
    let _timer = performance::start_operation("file_put");
    
    // Initialize database
    let db_path = database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    
    ui::print_info("Reading file...");
    let file_data = fs::read(path)?;
    let file_size = file_data.len() as u64;
    
    // Create progress bar
    let progress = ui::ProgressManager::new_upload(file_size);
    
    let config = Config::load_or_default(None)?;
    let mut swarm = create_swarm_and_connect_multi_bootstrap(cli, &config).await?;
    let file_key = RecordKey::new(&blake3::hash(&file_data).as_bytes());

    // Get the encryption key (either specified public key or default)
    let (encryption_public_key, public_key_hex) = get_encryption_key(public_key, key_manager)?;
    let encrypted_data = encrypt(&encryption_public_key.serialize(), &file_data)
        .map_err(|e| DfsError::Crypto(format!("Encryption error: {:?}", e)))?;

    // Create Reed-Solomon encoder
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
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

    // Store each shard with progress updates
    let mut chunk_keys = Vec::new();
    let total_shards = shards.len();
    progress.set_message("Storing chunks...");
    
    // Check if concurrent chunk uploads are enabled
    let use_concurrent_chunks = config.performance.chunks.max_concurrent_uploads > 1;
    
    if use_concurrent_chunks {
        // Use concurrent chunk upload
        let swarm = Arc::new(RwLock::new(swarm));
        let chunk_config = config.performance.chunks.to_concurrent_chunk_config();
        let chunk_manager = ConcurrentChunkManager::new(chunk_config);
        
        // Prepare chunks for concurrent upload
        let mut chunks_to_upload = Vec::new();
        for shard in shards.into_iter() {
            let chunk_key = RecordKey::new(&blake3::hash(&shard).as_bytes());
            chunk_keys.push(chunk_key.as_ref().to_vec()); // Store as Vec<u8>
            chunks_to_upload.push((chunk_key, shard));
        }
        
        // Upload chunks concurrently
        println!("Uploading {} chunks concurrently...", chunks_to_upload.len());
        let upload_results = chunk_manager.upload_chunks_concurrent(chunks_to_upload, swarm.clone()).await;
        
        match upload_results {
            Ok(results) => {
                let successful_uploads = results.len();
                println!("Successfully uploaded {}/{} chunks", successful_uploads, total_shards);
                progress.set_position(file_size); // Set to 100%
            }
            Err(e) => {
                return Err(DfsError::Network(format!("Concurrent chunk upload failed: {}", e)));
            }
        }
        
        // Convert back to regular swarm for metadata upload
        let mut swarm = Arc::try_unwrap(swarm).map_err(|_| DfsError::Network("Failed to unwrap swarm".to_string()))?.into_inner();
        
        // Store file metadata
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
        swarm
            .behaviour_mut()
            .kad
            .put_record(record, Quorum::One)?;
    } else {
        // Use sequential upload (original implementation)
        for (i, shard) in shards.into_iter().enumerate() {
            let chunk_key = RecordKey::new(&blake3::hash(&shard).as_bytes());
            chunk_keys.push(chunk_key.as_ref().to_vec()); // Store as Vec<u8>
            let record = Record {
                key: chunk_key,
                value: shard,
                publisher: None,
                expires: None,
            };
            swarm
                .behaviour_mut()
                .kad
                .put_record(record, Quorum::One)?;
            
            // Update progress
            let progress_value = ((i + 1) as f64 / total_shards as f64 * file_size as f64) as u64;
            progress.set_position(progress_value);
        }
        
        // Store file metadata
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
        swarm
            .behaviour_mut()
            .kad
            .put_record(record, Quorum::One)?;
    }

    // Generate or use provided name
    let original_filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let file_name = if let Some(provided_name) = name {
        if db.is_name_taken(provided_name)? {
            return Err(crate::error_handling::storage_error_with_suggestions(
                &format!("Name '{}' is already taken", provided_name)
            ).error);
        }
        provided_name.clone()
    } else {
        db.generate_unique_name(&original_filename)?
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
    db.store_file(
        &file_name,
        &hex::encode(file_key.as_ref()),
        &original_filename,
        file_size,
        upload_time,
        &file_tags,
        &public_key_hex,
    )?;
    
    progress.finish_with_message("Upload complete!");
    
    log_file_operation("store", &original_filename, &format!("size: {} bytes", file_data.len()));
    
    ui::print_success(&format!("File stored successfully as '{}'", file_name));
    println!("  Original: {}", original_filename);
    println!("  Size: {}", ui::format_file_size(file_size));
    println!("  Key: {}", hex::encode(file_key.as_ref()));
    if !file_tags.is_empty() {
        println!("  Tags: {}", file_tags.join(", "));
    }
    
    Ok(())
}

pub async fn handle_get_command(
    cli: &Cli,
    key_manager: &KeyManager,
    identifier: &str,
    output_path: &PathBuf,
    private_key: &Option<String>,
) -> DfsResult<()> {
    // Initialize database
    let db_path = database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    
    // Try to resolve identifier to a file key
    let file_key = if let Some(file_entry) = db.get_file_by_name(identifier)? {
        ui::print_info(&format!("Found file '{}' in database", identifier));
        file_entry.file_key
    } else if let Some(_) = db.get_file_by_key(identifier)? {
        ui::print_info("Using provided file key");
        identifier.to_string()
    } else {
        // Assume it's a direct key
        ui::print_info("Treating as direct file key");
        identifier.to_string()
    };
    let retry_config = RetryConfig::default();
    
    retry_async(
        || async {
            attempt_file_retrieval(cli, key_manager, &file_key, output_path, private_key).await
        },
        retry_config,
        "file_retrieval",
    ).await
}

/// Concurrent file retrieval using ConcurrentChunkManager with Smart Caching
async fn attempt_concurrent_file_retrieval(
    cli: &Cli,
    _key_manager: &KeyManager,
    key: &str,
    output_path: &PathBuf,
    _private_key: &Option<String>,
    config: &Config,
) -> DfsResult<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    println!("Using concurrent chunk retrieval with smart caching for enhanced performance...");
    
    // Create smart cache manager
    let cache_config = config.cache.to_smart_cache_config();
    let mut smart_cache = SmartCacheManager::new(cache_config);
    
    // Create swarm and connect to bootstrap nodes
    let swarm = create_swarm_and_connect_multi_bootstrap(cli, config).await?;
    let swarm = Arc::new(RwLock::new(swarm));
    
    // Create concurrent chunk manager
    let chunk_config = config.performance.chunks.to_concurrent_chunk_config();
    let chunk_manager = Arc::new(ConcurrentChunkManager::new(chunk_config));
    
    // Connect smart cache with concurrent chunk manager
    smart_cache.set_concurrent_chunks(chunk_manager.clone());
    
    // Start background tasks for cache management
    smart_cache.start_background_tasks().await;
    
    // Check cache first
    match smart_cache.get_file_smart(key).await {
        Ok(cached_data) => {
            println!("Cache hit! Retrieved {} bytes from cache", cached_data.len());
            
            // TODO: Decrypt and verify the cached data
            // For now, just write the raw data
            std::fs::write(output_path, cached_data)?;
            println!("File saved to: {}", output_path.display());
            
            // Print cache statistics
            let stats = smart_cache.get_stats().await;
            println!("Cache stats - Hits: {}, Misses: {}, Hit ratio: {:.2}%", 
                     stats.file_cache_hits, stats.file_cache_misses, stats.hit_ratio * 100.0);
            
            return Ok(());
        }
        Err(e) => {
            // Cache miss or error - continue with network retrieval
            println!("Cache miss: {}", e);
        }
    }
    
    // Enhanced DHT bootstrapping and peer discovery
    println!("Bootstrapping into DHT network...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Connect to service nodes
    let service_ports = [40872, 40873, 40874, 40875, 40876];
    for port in service_ports {
        let addr = format!("/ip4/127.0.0.1/tcp/{}", port);
        if let Ok(multiaddr) = addr.parse::<libp2p::Multiaddr>() {
            println!("Attempting to connect to service node at {}...", addr);
            if let Err(e) = swarm.write().await.dial(multiaddr) {
                println!("Failed to connect to {}: {:?}", addr, e);
            } else {
                println!("Successfully initiated connection to {}", addr);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Trigger multiple rounds of peer discovery
    println!("Discovering peers in network...");
    for i in 1..=3 {
        println!("Bootstrap attempt {}/3...", i);
        swarm.write().await.behaviour_mut().kad.bootstrap().ok();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let swarm_guard = swarm.read().await;
        let connected_count = swarm_guard.connected_peers().count();
        drop(swarm_guard);
        println!("Current connected peers: {}", connected_count);
    }
    
    // Wait for DHT stabilization
    println!("Waiting for DHT routing table to stabilize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Print network status
    println!("DHT Network Status:");
    let (local_peer_id, connected_count) = {
        let swarm_guard = swarm.read().await;
        let local_peer_id = swarm_guard.local_peer_id().clone();
        let connected_count = swarm_guard.connected_peers().count();
        (local_peer_id, connected_count)
    };
    println!("  Local Peer ID: {}", local_peer_id);
    println!("  Connected Peers: {}", connected_count);
    
    // Start concurrent file retrieval
    println!("Starting concurrent file retrieval for key: {}", key);
    
    match chunk_manager.retrieve_file_parallel(key, swarm).await {
        Ok(file_data) => {
            // For now, just write the raw data
            // In a full implementation, this would be properly reconstructed with Reed-Solomon
            println!("Successfully retrieved file data ({} bytes) using concurrent chunks", file_data.len());
            
            // Cache the retrieved data for future use
            if let Err(e) = smart_cache.cache_file_intelligent(key, file_data.clone()).await {
                warn!("Failed to cache retrieved file: {}", e);
            } else {
                println!("File cached for future access");
            }
            
            // Update access patterns
            {
                let mut patterns = smart_cache.access_patterns.lock().await;
                patterns.record_access(
                    key.to_string(),
                    AccessType::NetworkFetch,
                    std::time::Duration::from_millis(0), // TODO: Track actual response time
                    file_data.len() as u64,
                );
            }
            
            // Write to output file
            std::fs::write(output_path, file_data)?;
            println!("File saved to: {}", output_path.display());
            
            // Print cache statistics
            let stats = smart_cache.get_stats().await;
            println!("Cache stats - Hits: {}, Misses: {}, Hit ratio: {:.2}%", 
                     stats.file_cache_hits, stats.file_cache_misses, stats.hit_ratio * 100.0);
            
            Ok(())
        }
        Err(e) => {
            println!("Concurrent chunk retrieval failed: {}", e);
            Err(DfsError::Network(format!("Concurrent chunk retrieval failed: {}", e)))
        }
    }
}

async fn attempt_file_retrieval(
    cli: &Cli,
    key_manager: &KeyManager,
    key: &str,
    output_path: &PathBuf,
    private_key: &Option<String>,
) -> DfsResult<()> {
    let config = Config::load_or_default(None)?;
    
    // Check if concurrent chunk operations are enabled
    let use_concurrent_chunks = config.performance.chunks.max_concurrent_retrievals > 1;
    
    if use_concurrent_chunks {
        return attempt_concurrent_file_retrieval(cli, key_manager, key, output_path, private_key, &config).await;
    }
    
    // Fall back to sequential retrieval
    let mut swarm = create_swarm_and_connect_multi_bootstrap(cli, &config).await?;
    
    // Enhanced DHT bootstrapping and peer discovery
    println!("Bootstrapping into DHT network...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Try to connect to common service node ports
    println!("Attempting direct connections to common service ports...");
    let service_ports = [40872, 40873, 40874, 40875, 40876];
    for port in service_ports {
        let addr = format!("/ip4/127.0.0.1/tcp/{}", port);
        if let Ok(multiaddr) = addr.parse::<libp2p::Multiaddr>() {
            println!("Attempting to connect to service node at {}...", addr);
            if let Err(e) = swarm.dial(multiaddr) {
                println!("Failed to connect to {}: {:?}", addr, e);
            } else {
                println!("Successfully initiated connection to {}", addr);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Trigger multiple rounds of peer discovery
    println!("Discovering peers in network...");
    for i in 1..=3 {
        println!("Bootstrap attempt {}/3...", i);
        swarm.behaviour_mut().kad.bootstrap().ok();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Check connected peers instead
        let connected_count = swarm.connected_peers().count();
        println!("Current connected peers: {}", connected_count);
    }
    
    // Additional time for DHT stabilization and peer discovery
    println!("Waiting for DHT routing table to stabilize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    let key_bytes = hex::decode(key)?;
    let record_key = RecordKey::from(key_bytes);
    
    // Print DHT network status before retrieval
    println!("DHT Network Status:");
    println!("  Local Peer ID: {}", swarm.local_peer_id());
    let connected_peers: Vec<_> = swarm.connected_peers().collect();
    println!("  Connected Peers: {}", connected_peers.len());
    for peer in &connected_peers {
        println!("    - {}", peer);
    }
    
    println!("Starting file retrieval for key: {}", key);
    swarm.behaviour_mut().kad.get_record(record_key);
    
    // Initialize file retrieval state  
    let mut pending_file_retrieval = Some(FileRetrieval {
        stored_file: StoredFile {
            chunk_keys: Vec::new(),
            encryption_key: Vec::new(),
            file_size: 0,
            public_key_hex: String::new(),
            file_name: String::new(),
            stored_at: Local::now(),
        },
        chunks: HashMap::new(),
        output_path: output_path.clone(),
    });

    // Wait for network events and file retrieval with timeout
    use futures::stream::StreamExt;
    use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult};
    use libp2p::swarm::SwarmEvent;
    use crate::network::MyBehaviourEvent;
    use tokio::time::{timeout, Duration};

    let start_time = std::time::Instant::now();
    let retrieval_timeout = Duration::from_secs(15); // Shorter timeout to allow for longer bootstrapping

    loop {
        // Check if we've exceeded our timeout
        if start_time.elapsed() > retrieval_timeout {
            let elapsed = start_time.elapsed().as_secs();
            println!("File retrieval timed out after {}s", elapsed);
            return Err(DfsError::Network(format!("File retrieval timed out after {}s - file may not be available in the network", elapsed)));
        }

        let next_event = timeout(Duration::from_secs(5), swarm.select_next_some()).await;
        
        match next_event {
            Ok(SwarmEvent::NewListenAddr { address, .. }) => {
                println!("Listening on {:?}", address);
            },
            Ok(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                println!("Connected to peer: {}", peer_id);
                // Add peer to DHT routing table
                swarm.behaviour_mut().kad.add_address(&peer_id, "/ip4/127.0.0.1/tcp/0".parse().unwrap());
            },
            Ok(SwarmEvent::Behaviour(event)) => {
                match event {
                    MyBehaviourEvent::Kad(kad_event) => {
                        match kad_event {
                            KademliaEvent::OutboundQueryProgressed { result, .. } => {
                                match result {
                                    QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                                        let record = &peer_record.record;
                                        
                                        // Try to parse as StoredFile metadata first
                                        if let Ok(stored_file) = serde_json::from_slice::<StoredFile>(&record.value) {
                                            println!("Found file metadata, retrieving {} chunks...", stored_file.chunk_keys.len());
                                            println!("File name: {}", stored_file.file_name);
                                            println!("Stored at: {}", stored_file.stored_at.format("%Y-%m-%d %H:%M:%S"));
                                            println!("Encrypted with public key: {}", stored_file.public_key_hex);
                                            
                                            // Update file retrieval state
                                            if let Some(ref mut retrieval) = pending_file_retrieval {
                                                retrieval.stored_file = stored_file.clone();
                                                
                                                // Start retrieving all chunks
                                                for chunk_key_bytes in &stored_file.chunk_keys {
                                                    let chunk_key = RecordKey::from(chunk_key_bytes.clone());
                                                    swarm.behaviour_mut().kad.get_record(chunk_key);
                                                }
                                            }
                                        } else {
                                            // This might be a chunk
                                            if let Some(ref mut retrieval) = pending_file_retrieval {
                                                // Check if this record key matches any of our expected chunk keys
                                                let record_key_bytes = record.key.as_ref().to_vec();
                                                if retrieval.stored_file.chunk_keys.contains(&record_key_bytes) {
                                                    retrieval.chunks.insert(record.key.clone(), record.value.clone());
                                                    println!("Retrieved chunk {}/{}", 
                                                        retrieval.chunks.len(), 
                                                        retrieval.stored_file.chunk_keys.len()
                                                    );
                                                    
                                                    // Check if we have all chunks needed for reconstruction
                                                    if retrieval.chunks.len() >= DATA_SHARDS {
                                                        if let Err(e) = reconstruct_file(retrieval, private_key, key_manager, cli) {
                                                            println!("Failed to reconstruct file: {:?}", e);
                                                        } else {
                                                            println!("File reconstruction complete!");
                                                            return Ok(());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    QueryResult::GetRecord(Err(err)) => {
                                        println!("Failed to get record: {:?}", err);
                                        
                                        // If record not found, try to discover more peers
                                        if matches!(err, libp2p::kad::GetRecordError::NotFound { .. }) {
                                            println!("Record not found, attempting peer discovery...");
                                            swarm.behaviour_mut().kad.bootstrap().ok();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                // Timeout on individual event - continue trying
                let elapsed = start_time.elapsed().as_secs();
                println!("No network activity after {}s, continuing...", elapsed);
            }
            _ => {}
        }
    }
}

pub async fn handle_list_command(
    key_manager: &KeyManager,
    public_key: &Option<String>,
    tags: &Option<String>,
) -> DfsResult<()> {
    // Initialize database
    let db_path = database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    
    // Parse tag filter
    let tag_filter = tags.as_ref().map(|t| t.as_str());
    
    // Get files from database
    let files = db.list_files(tag_filter)?;
    
    if let Some(pk) = public_key {
        // Filter by public key if specified
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|f| f.public_key_hex == *pk)
            .collect();
        ui::print_file_list(&filtered_files);
    } else {
        // Show all files for this user's default key
        let target_public_key = &key_manager.key_info.public_key_hex;
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|f| f.public_key_hex == *target_public_key)
            .collect();
        ui::print_file_list(&filtered_files);
    }
    
    Ok(())
}

fn reconstruct_file(
    retrieval: &mut FileRetrieval,
    private_key: &Option<String>,
    key_manager: &KeyManager,
    cli: &Cli,
) -> DfsResult<()> {
    let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)?;
    
    // Prepare shards for reconstruction
    let mut shards: Vec<Option<Vec<u8>>> = vec![None; DATA_SHARDS + PARITY_SHARDS];
    
    // Fill shards with available chunks
    for (i, chunk_key_bytes) in retrieval.stored_file.chunk_keys.iter().enumerate() {
        let chunk_key = RecordKey::from(chunk_key_bytes.clone());
        if let Some(chunk_data) = retrieval.chunks.get(&chunk_key) {
            shards[i] = Some(chunk_data.clone());
        }
    }
    
    // Reconstruct missing shards if needed
    r.reconstruct(&mut shards)?;
    
    // Combine data shards to reconstruct the encrypted file
    let mut encrypted_data = Vec::new();
    for shard_opt in shards.iter().take(DATA_SHARDS) {
        if let Some(shard) = shard_opt {
            encrypted_data.extend_from_slice(shard);
        }
    }
    
    // Get the decryption key
    let decryption_key = if let Some(private_key_name) = private_key {
        let keys_dir = cli.keys_dir.clone()
            .unwrap_or_else(|| crate::key_manager::get_default_keys_dir().unwrap_or_else(|_| PathBuf::from("./keys")));
        crate::key_manager::get_decryption_key(&Some(private_key_name.clone()), key_manager, &keys_dir)?
    } else {
        SecretKey::parse_slice(&retrieval.stored_file.encryption_key)
            .map_err(|e| DfsError::Crypto(format!("Failed to parse encryption key: {:?}", e)))?
    };
    
    let decrypted_data = decrypt(&decryption_key.serialize(), &encrypted_data)
        .map_err(|e| DfsError::Crypto(format!("Decryption error: {:?}", e)))?;
    
    // Trim to original file size
    let final_data = if decrypted_data.len() > retrieval.stored_file.file_size {
        &decrypted_data[..retrieval.stored_file.file_size]
    } else {
        &decrypted_data
    };
    
    // Write to output file
    fs::write(&retrieval.output_path, final_data)?;
    println!("File successfully retrieved and saved to: {:?}", retrieval.output_path);
    println!("Original file name: {}", retrieval.stored_file.file_name);
    println!("Stored at: {}", retrieval.stored_file.stored_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Encrypted with public key: {}", retrieval.stored_file.public_key_hex);
    
    Ok(())
}

/// Handle info command for a specific file
pub async fn handle_info_command(
    _key_manager: &KeyManager,
    identifier: &str,
) -> DfsResult<()> {
    // Get database connection
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Find file by name or key
    let stored_file = if identifier.len() == 64 {
        // Looks like a file key
        db.get_file_by_key(identifier)?
    } else {
        // Treat as file name
        db.get_file_by_name(identifier)?
    };
    
    let file = stored_file.ok_or_else(|| DfsError::FileNotFound(identifier.to_string()))?;
    
    ui::print_file_info(&file);
    Ok(())
}

/// Handle stats command for storage statistics
pub async fn handle_stats_command(_key_manager: &KeyManager) -> DfsResult<()> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    let stats = db.get_stats()?;
    
    ui::print_database_stats(&stats);
    Ok(())
}
