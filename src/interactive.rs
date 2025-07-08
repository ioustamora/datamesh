/// Interactive Mode Module
///
/// This module provides interactive and service mode functionality for the DFS application.
/// It implements:
/// - An interactive console for user commands
/// - A service mode for background operation
/// - Command parsing and execution
/// - Network event handling
///
/// The interactive mode provides a command-line interface for users to perform
/// file operations, check network status, and manage keys in real-time.
///
/// The service mode runs DFS as a background process, maintaining DHT connectivity
/// and providing persistent storage capabilities.

use anyhow::Result;
use futures::stream::StreamExt;
use libp2p::{Multiaddr, PeerId};
use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult};
use libp2p::swarm::SwarmEvent;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::cli::Cli;
use crate::config::Config;
use crate::file_storage::{FileRetrieval, StoredFile, PUB_DATA_SHARDS, PUB_PARITY_SHARDS};
use crate::key_manager::{get_default_keys_dir, KeyManager};
use crate::network::{create_swarm_and_connect_multi_bootstrap, MyBehaviourEvent};
use crate::database::{self, DatabaseManager};
use crate::ui;
use crate::error_handling;
use crate::network_diagnostics;

/// Number of data shards for Reed-Solomon erasure coding
const DATA_SHARDS: usize = 4;

/// Run the interactive console mode
///
/// # Arguments
///
/// * `cli` - Command line arguments
/// * `key_manager` - Key manager instance
/// * `bootstrap_peer` - Optional bootstrap peer ID
/// * `bootstrap_addr` - Optional bootstrap peer address
/// * `port` - Port to listen on
pub async fn run_interactive_mode(
    cli: &Cli,
    key_manager: KeyManager,
    bootstrap_peer: Option<PeerId>,
    bootstrap_addr: Option<Multiaddr>,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    // Create a modified CLI for network connection
    let mut network_cli = cli.clone();
    network_cli.bootstrap_peer = bootstrap_peer;
    network_cli.bootstrap_addr = bootstrap_addr.clone();
    if port > 0 {
        network_cli.port = port;
    }
    
    let config = Config::load_or_default(None)?;
    let mut swarm = create_swarm_and_connect_multi_bootstrap(&network_cli, &config).await?;
    
    if port > 0 {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
        swarm.listen_on(listen_addr.parse()?)?;
    }
    
    if let (Some(peer), Some(addr)) = (bootstrap_peer, bootstrap_addr) {
        swarm.behaviour_mut().kad.add_address(&peer, addr);
        println!("Connecting to bootstrap peer: {}", peer);
    }

    // Initialize database for interactive mode
    let db_path = database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    
    // Display enhanced welcome message
    ui::print_interactive_welcome(
        &swarm.local_peer_id().to_string(),
        &key_manager.key_info.public_key_hex
    );

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut pending_file_retrieval: Option<FileRetrieval> = None;
    
    loop {
        // Handle network events
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        ui::print_info(&format!("Listening on {}", address));
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let connected_peers: Vec<_> = swarm.connected_peers().collect();
                        ui::print_success(&format!("Connected to peer: {}", peer_id));
                        ui::print_connection_status(connected_peers.len());
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        let connected_peers: Vec<_> = swarm.connected_peers().collect();
                        ui::print_warning(&format!("Disconnected from peer: {}", peer_id));
                        ui::print_connection_status(connected_peers.len());
                    }
                    SwarmEvent::Behaviour(behaviour_event) => {
                        handle_behaviour_event(behaviour_event, &mut pending_file_retrieval, &key_manager, cli);
                    }
                    _ => {}
                }
            }
            line = lines.next_line() => {
                match line? {
                    Some(input) => {
                        let parts: Vec<&str> = input.trim().split_whitespace().collect();
                        if parts.is_empty() {
                            continue;
                        }
                        
                        match parts[0] {
                            "put" => {
                                if parts.len() < 2 {
                                    ui::print_error("Usage: put <file> [--name <alias>] [--tags <tag1,tag2>] [--public-key <hex>]");
                                    ui::print_info("Example: put document.pdf --name my-document --tags work,important");
                                    continue;
                                }
                                
                                let file_path = PathBuf::from(parts[1]);
                                if !file_path.exists() {
                                    ui::print_error(&format!("File not found: {}", file_path.display()));
                                    continue;
                                }
                                
                                if !file_path.is_file() {
                                    ui::print_error(&format!("Path is not a file: {}", file_path.display()));
                                    continue;
                                }
                                
                                let mut public_key = None;
                                let mut name = None;
                                let mut tags = None;
                                
                                // Parse optional arguments
                                let mut i = 2;
                                while i < parts.len() {
                                    match parts[i] {
                                        "--name" if i + 1 < parts.len() => {
                                            name = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        "--tags" if i + 1 < parts.len() => {
                                            tags = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        "--public-key" if i + 1 < parts.len() => {
                                            public_key = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        _ => {
                                            ui::print_warning(&format!("Unknown option: {}", parts[i]));
                                            i += 1;
                                        }
                                    }
                                }
                                
                                // Show operation confirmation
                                let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                                ui::print_confirmation(
                                    "Uploading file",
                                    &format!("{} ({})", file_path.display(), ui::format_file_size(file_size))
                                );
                                
                                match handle_put_interactive(&mut swarm, &file_path, &key_manager, &public_key, &name, &tags, &db).await {
                                    Ok(file_name) => ui::print_success(&format!("File stored successfully as '{}'", file_name)),
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("put", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "get" => {
                                if parts.len() < 3 {
                                    ui::print_error("Usage: get <name_or_key> <output_path> [--private-key <name>]");
                                    ui::print_info("Example: get my-document ./downloaded-doc.pdf");
                                    continue;
                                }
                                
                                let output_path = PathBuf::from(parts[2]);
                                if output_path.exists() {
                                    ui::print_warning(&format!("Output file already exists: {}", output_path.display()));
                                    ui::print_info("File will be overwritten if download succeeds");
                                }
                                
                                // Check if parent directory exists
                                if let Some(parent) = output_path.parent() {
                                    if !parent.exists() {
                                        ui::print_error(&format!("Output directory does not exist: {}", parent.display()));
                                        continue;
                                    }
                                }
                                
                                let identifier = parts[1].to_string();
                                let private_key = if parts.len() > 4 && parts[3] == "--private-key" {
                                    Some(parts[4].to_string())
                                } else {
                                    None
                                };
                                
                                // Try to resolve identifier to a file key using database
                                let file_key = if let Ok(Some(file_entry)) = db.get_file_by_name(&identifier) {
                                    ui::print_info(&format!("Found file '{}' in database", identifier));
                                    file_entry.file_key
                                } else if let Ok(Some(_)) = db.get_file_by_key(&identifier) {
                                    ui::print_info("Using provided file key");
                                    identifier.clone()
                                } else {
                                    ui::print_info("Treating as direct file key");
                                    identifier.clone()
                                };
                                
                                // Show operation confirmation
                                ui::print_confirmation(
                                    "Retrieving file",
                                    &format!("{} → {}", identifier, output_path.display())
                                );
                                
                                match handle_get_interactive(&mut swarm, file_key, output_path, &mut pending_file_retrieval, &private_key).await {
                                    Ok(_) => ui::print_info("File retrieval initiated..."),
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("get", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "list" => {
                                let mut public_key = None;
                                let mut tags = None;
                                
                                // Parse optional arguments
                                let mut i = 1;
                                while i < parts.len() {
                                    match parts[i] {
                                        "--public-key" if i + 1 < parts.len() => {
                                            public_key = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        "--tags" if i + 1 < parts.len() => {
                                            tags = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        _ => {
                                            ui::print_warning(&format!("Unknown option: {}", parts[i]));
                                            i += 1;
                                        }
                                    }
                                }
                                
                                match handle_list_interactive(&db, &key_manager, &public_key, &tags) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("list", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "keys" => {
                                println!("Available keys:");
                                let keys_dir = get_default_keys_dir().unwrap_or_else(|_| PathBuf::from("./keys"));
                                match KeyManager::list_keys(&keys_dir) {
                                    Ok(keys) => {
                                        for key_name in keys {
                                            if let Ok(info) = KeyManager::get_key_info(&keys_dir, &key_name) {
                                                println!("  {} (created: {}, public: {}...)", 
                                                    info.name, 
                                                    info.created.format("%Y-%m-%d %H:%M:%S"),
                                                    &info.public_key_hex[..16]
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => println!("Error listing keys: {}", e),
                                }
                            }
                            "quit" | "exit" => {
                                println!("Goodbye!");
                                return Ok(());
                            }
                            "info" => {
                                if parts.len() < 2 {
                                    ui::print_error("Usage: info <name_or_key>");
                                    continue;
                                }
                                
                                let identifier = parts[1];
                                match handle_info_interactive(&db, identifier) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::handle_error(e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "stats" => {
                                match handle_stats_interactive(&db) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::handle_error(e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "status" => {
                                let connected_peers: Vec<_> = swarm.connected_peers().collect();
                                let listening_addrs: Vec<String> = swarm.listeners()
                                    .map(|addr| addr.to_string())
                                    .collect();
                                
                                ui::print_detailed_network_status(
                                    &swarm.local_peer_id().to_string(),
                                    &listening_addrs,
                                    connected_peers.len()
                                );
                            }
                            "peers" => {
                                let detailed = parts.len() > 1 && parts[1] == "--detailed";
                                let format = crate::cli::OutputFormat::Table;
                                
                                match network_diagnostics::handle_peers_command(&mut swarm, detailed, &format).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("peers", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "health" => {
                                let continuous = parts.len() > 1 && parts[1] == "--continuous";
                                let interval = if parts.len() > 3 && parts[2] == "--interval" {
                                    parts[3].parse().unwrap_or(5)
                                } else {
                                    5
                                };
                                
                                match network_diagnostics::handle_health_command(&mut swarm, continuous, interval).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("health", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "network" => {
                                let depth = if parts.len() > 2 && parts[1] == "--depth" {
                                    parts[2].parse().unwrap_or(2)
                                } else {
                                    2
                                };
                                let visualize = parts.len() > 1 && parts[1] == "--visualize";
                                
                                match network_diagnostics::handle_network_command(&mut swarm, depth, visualize).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("network", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "discover" => {
                                let timeout = if parts.len() > 2 && parts[1] == "--timeout" {
                                    parts[2].parse().unwrap_or(30)
                                } else {
                                    30
                                };
                                let bootstrap_all = parts.len() > 1 && parts[1] == "--bootstrap-all";
                                
                                match network_diagnostics::handle_discover_command(&mut swarm, timeout, bootstrap_all).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("discover", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "distribution" => {
                                let mut file_key = None;
                                let mut public_key = None;
                                
                                let mut i = 1;
                                while i < parts.len() {
                                    match parts[i] {
                                        "--file-key" if i + 1 < parts.len() => {
                                            file_key = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        "--public-key" if i + 1 < parts.len() => {
                                            public_key = Some(parts[i + 1].to_string());
                                            i += 2;
                                        }
                                        _ => {
                                            ui::print_warning(&format!("Unknown option: {}", parts[i]));
                                            i += 1;
                                        }
                                    }
                                }
                                
                                match network_diagnostics::handle_distribution_command(&mut swarm, &file_key, &public_key).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("distribution", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "bandwidth" => {
                                let test_peer = if parts.len() > 2 && parts[1] == "--test-peer" {
                                    Some(parts[2].to_string())
                                } else {
                                    None
                                };
                                let duration = if parts.len() > 4 && parts[3] == "--duration" {
                                    parts[4].parse().unwrap_or(30)
                                } else {
                                    30
                                };
                                
                                match network_diagnostics::handle_bandwidth_command(&mut swarm, &test_peer, duration).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("bandwidth", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "help" => {
                                ui::print_interactive_welcome(
                                    &swarm.local_peer_id().to_string(),
                                    &key_manager.key_info.public_key_hex
                                );
                            }
                            _ => {
                                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
                            }
                        }
                    }
                    None => break, // EOF
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_put_interactive(
    swarm: &mut libp2p::swarm::Swarm<crate::network::MyBehaviour>,
    file_path: &PathBuf,
    key_manager: &KeyManager,
    public_key: &Option<String>,
    name: &Option<String>,
    tags: &Option<String>,
    db: &DatabaseManager,
) -> Result<String, Box<dyn Error>> {
    use libp2p::kad::{Quorum, Record, RecordKey};
    use reed_solomon_erasure::galois_8::ReedSolomon;
    use ecies::encrypt;
    use std::fs;
    use chrono::Local;
    use crate::key_manager::get_encryption_key;
    
    let file_data = fs::read(file_path)?;
    let file_key = RecordKey::new(&blake3::hash(&file_data).as_bytes());

    // Get the encryption key (either specified public key or default)
    let (encryption_public_key, public_key_hex) = get_encryption_key(public_key, key_manager)?;
    let encrypted_data = encrypt(&encryption_public_key.serialize(), &file_data)
        .map_err(|e| anyhow::anyhow!("Encryption error: {:?}", e))?;

    // Create Reed-Solomon encoder
    let r = ReedSolomon::new(PUB_DATA_SHARDS, PUB_PARITY_SHARDS)?;
    let chunk_size = (encrypted_data.len() + PUB_DATA_SHARDS - 1) / PUB_DATA_SHARDS;
    
    // Create shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; PUB_DATA_SHARDS + PUB_PARITY_SHARDS];
    
    // Fill data shards
    for (i, shard) in shards.iter_mut().enumerate().take(PUB_DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, encrypted_data.len());
        if start < encrypted_data.len() {
            shard[..end - start].copy_from_slice(&encrypted_data[start..end]);
        }
    }
    
    // Encode to create parity shards
    r.encode(&mut shards)?;

    // Store each shard
    let mut chunk_keys = Vec::new();
    for shard in shards {
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
    }

    // Generate or use provided name
    let original_filename = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let file_name = if let Some(provided_name) = name {
        if db.is_name_taken(provided_name)? {
            return Err(format!("Name '{}' is already taken. Please choose a different name.", provided_name).into());
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

    // Store file metadata in DHT
    let stored_file = StoredFile {
        chunk_keys,
        encryption_key: key_manager.key.serialize().to_vec(),
        file_size: file_data.len(),
        public_key_hex: public_key_hex.clone(),
        file_name: original_filename.clone(),
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

    // Store in database
    let file_size = file_data.len() as u64;
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

    Ok(file_name)
}

async fn handle_get_interactive(
    swarm: &mut libp2p::swarm::Swarm<crate::network::MyBehaviour>,
    key: String,
    output_path: PathBuf,
    pending_file_retrieval: &mut Option<FileRetrieval>,
    _private_key: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    use libp2p::kad::RecordKey;
    use chrono::Local;
    
    let key_bytes = hex::decode(key)?;
    let record_key = RecordKey::from(key_bytes);
    swarm.behaviour_mut().kad.get_record(record_key);
    
    // Initialize file retrieval state
    *pending_file_retrieval = Some(FileRetrieval {
        stored_file: StoredFile {
            chunk_keys: Vec::new(),
            encryption_key: Vec::new(),
            file_size: 0,
            public_key_hex: String::new(),
            file_name: String::new(),
            stored_at: Local::now(),
        },
        chunks: HashMap::new(),
        output_path,
    });
    
    Ok(())
}

fn handle_behaviour_event(
    event: MyBehaviourEvent,
    pending_file_retrieval: &mut Option<FileRetrieval>,
    _key_manager: &KeyManager,
    _cli: &Cli,
) {
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
                                
                                // Update file retrieval state - we can't call swarm methods here,
                                // so we'll just update the state
                                if let Some(ref mut retrieval) = pending_file_retrieval {
                                    retrieval.stored_file = stored_file.clone();
                                    
                                    // Note: In interactive mode, we would need to queue chunk requests
                                    // for the next swarm poll cycle
                                }
                            } else {
                                // This might be a chunk
                                if let Some(ref mut retrieval) = pending_file_retrieval {
                                    // Check if this record key matches any of our expected chunk keys
                                    let record_key_bytes = record.key.as_ref().to_vec();
                                    if retrieval.stored_file.chunk_keys.contains(&record_key_bytes) {
                                        retrieval.chunks.insert(record.key.clone(), record.value.clone());
                                        ui::print_progress(
                                            retrieval.chunks.len(), 
                                            retrieval.stored_file.chunk_keys.len(),
                                            "Downloading chunks"
                                        );
                                        
                                        // Check if we have all chunks needed for reconstruction
                                        if retrieval.chunks.len() >= DATA_SHARDS {
                                            if let Err(e) = reconstruct_file_interactive(retrieval, _key_manager, _cli) {
                                                println!("Failed to reconstruct file: {:?}", e);
                                            } else {
                                                println!("File reconstruction complete!");
                                                // Clear the retrieval state
                                                // *pending_file_retrieval = None; // Can't do this in interactive mode easily
                                            }
                                        }
                                    }
                                }
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

fn reconstruct_file_interactive(
    retrieval: &mut FileRetrieval,
    _key_manager: &KeyManager,
    _cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    use reed_solomon_erasure::galois_8::ReedSolomon;
    use ecies::{decrypt, SecretKey};
    use libp2p::kad::RecordKey;
    use std::fs;
    
    let r = ReedSolomon::new(PUB_DATA_SHARDS, PUB_PARITY_SHARDS)?;
    
    // Prepare shards for reconstruction
    let mut shards: Vec<Option<Vec<u8>>> = vec![None; PUB_DATA_SHARDS + PUB_PARITY_SHARDS];
    
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
    for shard_opt in shards.iter().take(PUB_DATA_SHARDS) {
        if let Some(shard) = shard_opt {
            encrypted_data.extend_from_slice(shard);
        }
    }
    
    // Decrypt the data using the stored key
    let decryption_key = SecretKey::parse_slice(&retrieval.stored_file.encryption_key)
        .map_err(|e| anyhow::anyhow!("Failed to parse encryption key: {:?}", e))?;
    
    let decrypted_data = decrypt(&decryption_key.serialize(), &encrypted_data)
        .map_err(|e| anyhow::anyhow!("Decryption error: {:?}", e))?;
    
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

// Service mode - runs the DFS node without interactive input
pub async fn run_service_mode(
    cli: &Cli,
    key_manager: KeyManager,
    bootstrap_peer: Option<PeerId>,
    bootstrap_addr: Option<Multiaddr>,
    port: u16,
    timeout: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    // Create a modified CLI for network connection
    let mut network_cli = cli.clone();
    network_cli.bootstrap_peer = bootstrap_peer;
    network_cli.bootstrap_addr = bootstrap_addr.clone();
    if port > 0 {
        network_cli.port = port;
    }
    
    let config = Config::load_or_default(None)?;
    let mut swarm = create_swarm_and_connect_multi_bootstrap(&network_cli, &config).await?;
    
    if port > 0 {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
        swarm.listen_on(listen_addr.parse()?)?;
    }
    
    if let (Some(peer), Some(addr)) = (bootstrap_peer, bootstrap_addr) {
        swarm.behaviour_mut().kad.add_address(&peer, addr);
        println!("Connecting to bootstrap peer: {}", peer);
        
        // Bootstrap the DHT to improve connectivity
        swarm.behaviour_mut().kad.bootstrap().ok();
    }

    println!("DFS Service Mode");
    println!("================");
    println!("Peer ID: {:?}", swarm.local_peer_id());
    println!("Default public key: {}", key_manager.key_info.public_key_hex);
    println!("Service running... (use Ctrl+C to stop)");
    
    let mut pending_file_retrieval: Option<FileRetrieval> = None;
    let start_time = std::time::Instant::now();
    let mut last_bootstrap = std::time::Instant::now();
    
    loop {
        // Check timeout if specified
        if let Some(timeout_secs) = timeout {
            if start_time.elapsed().as_secs() >= timeout_secs {
                println!("Service timeout reached ({}s), shutting down...", timeout_secs);
                break;
            }
        }
        
        // Periodically re-bootstrap the DHT to maintain connectivity
        if last_bootstrap.elapsed().as_secs() >= 30 {  // More frequent bootstrapping
            println!("Performing DHT bootstrap to maintain connectivity...");
            swarm.behaviour_mut().kad.bootstrap().ok();
            last_bootstrap = std::time::Instant::now();
            
            // Print current peer count for diagnostics
            let connected_peers: Vec<_> = swarm.connected_peers().collect();
            println!("Currently connected to {} peers", connected_peers.len());
        }
        
        // Handle network events
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Service node connected to peer: {}", peer_id);
                    }
                    SwarmEvent::Behaviour(behaviour_event) => {
                        handle_behaviour_event(behaviour_event, &mut pending_file_retrieval, &key_manager, cli);
                    }
                    _ => {}
                }
            }
            // Add a small delay to prevent busy waiting
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Continue the loop
            }
        }
    }
    
    println!("DFS service stopped");
    Ok(())
}

/// Handle list command in interactive mode
fn handle_list_interactive(
    db: &DatabaseManager,
    key_manager: &KeyManager,
    public_key: &Option<String>,
    tags: &Option<String>,
) -> Result<(), Box<dyn Error>> {
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

/// Handle info command in interactive mode
fn handle_info_interactive(
    db: &DatabaseManager,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    // Try to find the file by name first, then by key
    let file = if let Some(file) = db.get_file_by_name(identifier)? {
        file
    } else if let Some(file) = db.get_file_by_key(identifier)? {
        file
    } else {
        ui::print_error(&format!("File not found: {}", identifier));
        ui::print_info("Use 'list' to see available files");
        return Ok(());
    };
    
    ui::print_file_info(&file);
    Ok(())
}

/// Handle stats command in interactive mode
fn handle_stats_interactive(
    db: &DatabaseManager,
) -> Result<(), Box<dyn Error>> {
    let stats = db.get_stats()?;
    ui::print_database_stats(&stats);
    Ok(())
}
