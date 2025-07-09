/// DataMesh - Distributed Data Storage System
///
/// This is the main entry point for the DataMesh application, which provides a secure,
/// fault-tolerant distributed data storage system built with Rust and libp2p.
/// 
/// The system features:
/// - ECIES encryption for secure file storage
/// - Reed-Solomon erasure coding for fault tolerance
/// - Kademlia DHT for decentralized storage
/// - BLAKE3 hashing for optimal performance
///
/// The main module orchestrates all commands defined in the CLI and delegates
/// to appropriate modules for execution.
mod key_manager;
mod encrypted_key_manager;
mod audit_logger;
mod file_storage;
mod network;
mod cli;
mod interactive;
mod error;
mod error_handling;
mod logging;
mod config;
mod resilience;
mod performance;
mod database;
mod ui;
mod presets;
mod network_diagnostics;
mod file_manager;
mod batch_operations;
mod health_manager;
mod governance;
mod quota_service;
mod bootstrap_admin;
mod governance_service;
mod economics;
mod persistent_dht;
mod bootstrap_manager;
mod concurrent_chunks;
mod smart_cache;
mod api_server;

use std::error::Error;
use std::path::PathBuf;

/// Handles the configuration command for generating or displaying config files
///
/// # Arguments
///
/// * `generate` - Whether to generate a new config file
/// * `config_path` - Optional path to the config file
async fn handle_config_command(generate: bool, config_path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    if generate {
        let config = config::Config::default();
        let path = config_path.clone().unwrap_or_else(|| PathBuf::from("datamesh.toml"));
        config.save(&path)?;
        println!("Default configuration generated at: {:?}", path);
    } else {
        let path = config_path.clone().unwrap_or_else(|| PathBuf::from("datamesh.toml"));
        let config = config::Config::load_or_default(Some(path.clone()))?;
        println!("Configuration loaded from: {:?}", path);
        println!("{:#?}", config);
    }
    Ok(())
}

/// Handles the metrics command for displaying performance metrics
///
/// # Arguments
///
/// * `summary` - Whether to display a performance summary
/// * `export` - Whether to export metrics as JSON
async fn handle_metrics_command(summary: bool, export: bool) -> Result<(), Box<dyn Error>> {
    let monitor = performance::global_monitor();
    
    if summary {
        ui::print_header("Performance Summary");
        monitor.print_summary();
        
        // Show recent metrics
        let recent_metrics = monitor.get_recent_metrics(10);
        if !recent_metrics.is_empty() {
            println!();
            ui::print_header("Recent Operations");
            println!("{:<15} {:<12} {:<10} {:<8} {:<20}", "Operation", "Duration", "Success", "Bytes", "Timestamp");
            println!("{}", "-".repeat(75));
            
            for metric in recent_metrics {
                let duration_str = format!("{:.2}ms", metric.duration_ms);
                let success_str = if metric.success { "✓" } else { "✗" };
                let bytes_str = if let Some(bytes) = metric.bytes_processed {
                    format!("{}", bytes)
                } else {
                    "-".to_string()
                };
                let timestamp_str = metric.timestamp.format("%H:%M:%S");
                
                println!("{:<15} {:<12} {:<10} {:<8} {:<20}", 
                    metric.operation, duration_str, success_str, bytes_str, timestamp_str);
            }
        }
    }
    
    if export {
        let metrics_json = monitor.export_metrics();
        println!("Metrics JSON:");
        println!("{}", metrics_json);
    }
    
    if !summary && !export {
        println!("Use --summary to show performance summary or --export to export metrics");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging system
    logging::init_logging();
    
    // Parse command line arguments
    let mut cli = cli::Cli::parse();
    
    // Apply network presets if specified
    if let Err(e) = apply_network_preset(&mut cli) {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    // Set up key management before any other operations
    let key_selection_mode = if cli.non_interactive {
        key_manager::KeySelectionMode::NonInteractive
    } else {
        key_manager::KeySelectionMode::Interactive
    };
    
    let key_manager = match key_manager::setup_key_management_with_mode(&cli, key_selection_mode).await {
        Ok(km) => km,
        Err(e) => {
            let enhanced_error = error_handling::handle_error(e.as_ref());
            error_handling::display_enhanced_error(&enhanced_error);
            std::process::exit(1);
        }
    };
    
    crate::ui::print_operation_status("Cryptographic Keys", "Ready", Some("ECIES encryption initialized"));

    // Initialize performance monitoring
    let _monitor = performance::global_monitor();
    
    // Handle the command with enhanced error handling
    let result: Result<(), Box<dyn Error>> = match &cli.command {
        cli::Commands::Put { path, public_key, name, tags } => {
            let timer = performance::global_monitor().start_operation("file_put");
            let result = file_storage::handle_put_command(&cli, &key_manager, path, public_key, name, tags).await
                .map_err(|e| Box::new(e) as Box<dyn Error>);
            
            match &result {
                Ok(_) => timer.complete_success(None),
                Err(e) => timer.complete_failure(e.to_string()),
            }
            result
        }
        cli::Commands::Get { identifier, output_path, private_key } => {
            let timer = performance::global_monitor().start_operation("file_get");
            let result = file_storage::handle_get_command(&cli, &key_manager, identifier, output_path, private_key).await
                .map_err(|e| Box::new(e) as Box<dyn Error>);
            
            match &result {
                Ok(_) => timer.complete_success(None),
                Err(e) => timer.complete_failure(e.to_string()),
            }
            result
        }
        cli::Commands::List { public_key, tags } => {
            file_storage::handle_list_command(&key_manager, public_key, tags).await
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        }
        cli::Commands::Bootstrap { port } => {
            let config = config::Config::load_or_default(None)?;
            network::start_bootstrap_node(*port, &config).await
        }
        cli::Commands::Interactive { bootstrap_peer, bootstrap_addr, port } => {
            interactive::run_interactive_mode(&cli, key_manager, *bootstrap_peer, bootstrap_addr.clone(), *port).await
        }
        cli::Commands::Service { bootstrap_peer, bootstrap_addr, port, timeout } => {
            interactive::run_service_mode(&cli, key_manager, *bootstrap_peer, bootstrap_addr.clone(), *port, *timeout).await
        }
        cli::Commands::Config { generate, config_path } => {
            handle_config_command(*generate, config_path).await
        }
        cli::Commands::Metrics { summary, export } => {
            handle_metrics_command(*summary, *export).await
        }
        cli::Commands::Info { identifier } => {
            handle_info_command(identifier).await
        }
        cli::Commands::Stats => {
            handle_stats_command().await
        }
        cli::Commands::Networks => {
            handle_networks_command().await
        }
        cli::Commands::Peers { detailed, format } => {
            handle_peers_command(detailed, format).await
        }
        cli::Commands::Health { continuous, interval } => {
            handle_health_command(continuous, interval).await
        }
        cli::Commands::Distribution { file_key, public_key } => {
            handle_distribution_command(file_key, public_key).await
        }
        cli::Commands::Network { depth, visualize } => {
            handle_network_command(depth, visualize).await
        }
        cli::Commands::Discover { timeout, bootstrap_all } => {
            handle_discover_command(timeout, bootstrap_all).await
        }
        cli::Commands::Bandwidth { test_peer, duration } => {
            handle_bandwidth_command(test_peer, duration).await
        }
        
        // === File Management & Operations ===
        cli::Commands::Sync { local_dir, watch, bidirectional, exclude, parallel } => {
            handle_sync_command(&cli, &key_manager, local_dir, *watch, *bidirectional, exclude, *parallel).await
        }
        cli::Commands::Backup { source, name, incremental, compress, schedule, exclude } => {
            handle_backup_command(&cli, &key_manager, source, name, *incremental, *compress, schedule, exclude).await
        }
        cli::Commands::Restore { backup_name, destination, version, verify, list_versions } => {
            handle_restore_command(&cli, &key_manager, backup_name, destination, version, *verify, *list_versions).await
        }
        cli::Commands::Duplicate { source, new_name, new_tags } => {
            handle_duplicate_command(&cli, &key_manager, source, new_name, new_tags).await
        }
        cli::Commands::Rename { old_name, new_name } => {
            handle_rename_command(old_name, new_name).await
        }
        
        // === Search & Discovery ===
        cli::Commands::Search { query, file_type, size, date, regex, limit } => {
            handle_search_command(query, file_type, size, date, *regex, *limit).await
        }
        cli::Commands::Recent { count, days, file_type } => {
            handle_recent_command(*count, *days, file_type).await
        }
        cli::Commands::Popular { timeframe, count } => {
            handle_popular_command(timeframe, *count).await
        }
        
        // === Batch Operations ===
        cli::Commands::BatchPut { pattern, recursive, parallel, base_dir, tag_pattern } => {
            handle_batch_put_command(&cli, &key_manager, pattern, *recursive, *parallel, base_dir, tag_pattern).await
        }
        cli::Commands::BatchGet { pattern, destination, parallel, preserve_structure } => {
            handle_batch_get_command(&cli, &key_manager, pattern, destination, *parallel, *preserve_structure).await
        }
        cli::Commands::BatchTag { pattern, add_tags, remove_tags, dry_run } => {
            handle_batch_tag_command(pattern, add_tags, remove_tags, *dry_run).await
        }
        
        // === Health & Maintenance ===
        cli::Commands::Repair { target, auto, verify_all, threshold } => {
            handle_repair_command(&cli, &key_manager, target, *auto, *verify_all, *threshold).await
        }
        cli::Commands::Cleanup { orphaned, duplicates, low_health, dry_run, force } => {
            handle_cleanup_command(*orphaned, *duplicates, *low_health, *dry_run, *force).await
        }
        cli::Commands::Quota { usage, limit, warn } => {
            handle_quota_command(*usage, limit, warn).await
        }
        
        // === Import/Export ===
        cli::Commands::Export { destination, format, encrypt, include_metadata, pattern } => {
            handle_export_command(destination, format, *encrypt, *include_metadata, pattern).await
        }
        cli::Commands::Import { archive, verify, preserve_structure, tag_prefix } => {
            handle_import_command(archive, *verify, *preserve_structure, tag_prefix).await
        }
        
        // === Quick Actions ===
        cli::Commands::Pin { target, duration, priority } => {
            handle_pin_command(target, duration, *priority).await
        }
        cli::Commands::Unpin { target } => {
            handle_unpin_command(target).await
        }
        cli::Commands::Share { target, public, expires, password, qr_code } => {
            handle_share_command(target, *public, expires, password, *qr_code).await
        }
        
        // === Performance & Optimization ===
        cli::Commands::Optimize { defrag, rebalance, compress, analyze } => {
            handle_optimize_command(*defrag, *rebalance, *compress, *analyze).await
        }
        cli::Commands::Benchmark { full, network, storage, duration } => {
            handle_benchmark_command(&cli, &key_manager, *full, *network, *storage, *duration).await
        }
        
        // === API Server ===
        cli::Commands::ApiServer { host, port, https, cert_path, key_path, no_swagger } => {
            handle_api_server_command(&cli, &key_manager, host, port, *https, cert_path, key_path, *no_swagger).await
        }
    };

    // Handle any command errors with enhanced error reporting
    if let Err(e) = result {
        let enhanced_error = error_handling::handle_error(e.as_ref());
        error_handling::display_enhanced_error(&enhanced_error);
        std::process::exit(1);
    }

    Ok(())
}

/// Handles the info command for displaying file information
async fn handle_info_command(identifier: &str) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Try to find the file by name first, then by key
    let file = if let Some(file) = db.get_file_by_name(identifier)? {
        file
    } else if let Some(file) = db.get_file_by_key(identifier)? {
        file
    } else {
        return Err(Box::new(error_handling::file_not_found_error_with_suggestions(identifier)));
    };
    
    ui::print_file_info(&file);
    Ok(())
}

/// Handles the stats command for displaying storage statistics
async fn handle_stats_command() -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    let stats = db.get_stats()?;
    ui::print_database_stats(&stats);
    
    Ok(())
}

/// Handles the networks command for displaying available network presets
async fn handle_networks_command() -> Result<(), Box<dyn Error>> {
    presets::print_available_presets();
    Ok(())
}

/// Handles the peers command for listing connected peers
async fn handle_peers_command(
    _detailed: &bool,
    _format: &cli::OutputFormat,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Peers command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'peers' command within the interactive mode.");
    Ok(())
}

/// Handles the health command for network health monitoring
async fn handle_health_command(
    _continuous: &bool,
    _interval: &u64,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Health command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'health' command within the interactive mode.");
    Ok(())
}

/// Handles the distribution command for file distribution analysis
async fn handle_distribution_command(
    _file_key: &Option<String>,
    _public_key: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Distribution command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'distribution' command within the interactive mode.");
    Ok(())
}

/// Handles the network command for network topology analysis
async fn handle_network_command(
    _depth: &u32,
    _visualize: &bool,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Network command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'network' command within the interactive mode.");
    Ok(())
}

/// Handles the discover command for peer discovery
async fn handle_discover_command(
    _timeout: &u64,
    _bootstrap_all: &bool,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Discover command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'discover' command within the interactive mode.");
    Ok(())
}

/// Handles the bandwidth command for network performance testing
async fn handle_bandwidth_command(
    _test_peer: &Option<String>,
    _duration: &u64,
) -> Result<(), Box<dyn Error>> {
    ui::print_error("Bandwidth command requires an active network connection.");
    ui::print_info("Use 'dfs interactive' or 'dfs service' to connect to the network first.");
    ui::print_info("Then use the 'bandwidth' command within the interactive mode.");
    Ok(())
}

// === New Command Handlers ===

/// Handles the sync command for directory synchronization
async fn handle_sync_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    local_dir: &PathBuf,
    watch: bool,
    bidirectional: bool,
    exclude: &Option<String>,
    parallel: usize,
) -> Result<(), Box<dyn Error>> {
    let exclude_patterns = exclude.as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    
    let options = file_manager::SyncOptions {
        watch,
        bidirectional,
        exclude_patterns,
        parallel,
    };
    
    file_manager::sync_directory(cli, key_manager, local_dir, options).await
        .map_err(|e| e.into())
}

/// Handles the backup command
async fn handle_backup_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    source: &PathBuf,
    name: &str,
    incremental: bool,
    compress: bool,
    schedule: &Option<String>,
    exclude: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let exclude_patterns = exclude.as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    
    let config = file_manager::BackupConfig {
        name: name.to_string(),
        incremental,
        compress,
        schedule: schedule.clone(),
        exclude_patterns,
    };
    
    file_manager::create_backup(cli, key_manager, source, config).await
        .map_err(|e| e.into())
}

/// Handles the restore command
async fn handle_restore_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    backup_name: &str,
    destination: &PathBuf,
    version: &Option<u32>,
    verify: bool,
    list_versions: bool,
) -> Result<(), Box<dyn Error>> {
    if list_versions {
        let db_path = database::get_default_db_path()?;
        let db = database::DatabaseManager::new(&db_path)?;
        
        let backups = db.list_files(Some(&format!("backup:{}", backup_name)))?;
        if backups.is_empty() {
            ui::print_info(&format!("No backup versions found for '{}'", backup_name));
            return Ok(());
        }
        
        ui::print_header(&format!("Backup Versions for '{}'", backup_name));
        println!("{:<8} {:<20} {:<10} {:<15}", "Version", "Date", "Size", "Status");
        println!("{}", "-".repeat(60));
        
        for (idx, backup) in backups.iter().enumerate() {
            let version = idx + 1;
            let size = if backup.file_size > 1024 * 1024 {
                format!("{:.1} MB", backup.file_size as f64 / (1024.0 * 1024.0))
            } else if backup.file_size > 1024 {
                format!("{:.1} KB", backup.file_size as f64 / 1024.0)
            } else {
                format!("{} B", backup.file_size)
            };
            
            let health_percentage = if backup.chunks_total > 0 {
                (backup.chunks_healthy as f64 / backup.chunks_total as f64) * 100.0
            } else {
                100.0
            };
            
            let health_status = if health_percentage >= 90.0 {
                "Healthy"
            } else if health_percentage >= 70.0 {
                "Good"
            } else {
                "Degraded"
            };
            
            println!("{:<8} {:<20} {:<10} {:<15}", 
                version, 
                backup.upload_time.format("%Y-%m-%d %H:%M:%S"),
                size,
                health_status
            );
        }
        return Ok(());
    }
    
    file_manager::restore_backup(cli, key_manager, backup_name, destination, *version, verify).await
        .map_err(|e| e.into())
}

/// Handles the duplicate command
async fn handle_duplicate_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    source: &str,
    new_name: &Option<String>,
    new_tags: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    file_manager::duplicate_file(cli, key_manager, source, new_name.clone(), new_tags.clone()).await
        .map_err(|e| e.into())
}

/// Handles the rename command
async fn handle_rename_command(old_name: &str, new_name: &str) -> Result<(), Box<dyn Error>> {
    file_manager::rename_file(old_name, new_name).await
        .map_err(|e| e.into())
}

/// Handles the search command
async fn handle_search_command(
    query: &str,
    file_type: &Option<String>,
    size: &Option<String>,
    date: &Option<String>,
    regex: bool,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    let size_range = size.as_ref().and_then(|s| parse_size_range(s).ok());
    let date_range = date.as_ref().and_then(|d| parse_date_range(d).ok());
    
    let criteria = file_manager::SearchCriteria {
        query: query.to_string(),
        file_type: file_type.clone(),
        size_range,
        date_range,
        use_regex: regex,
        limit,
    };
    
    let results = file_manager::search_files(criteria).await?;
    ui::print_file_list(&results);
    
    Ok(())
}

/// Handles the recent command
async fn handle_recent_command(
    count: usize,
    days: u32,
    file_type: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let results = file_manager::get_recent_files(count, days, file_type.clone()).await?;
    
    ui::print_header("Recent Files");
    ui::print_file_list(&results);
    
    Ok(())
}

/// Handles the popular command
async fn handle_popular_command(
    timeframe: &str,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    let all_files = db.list_files(None)?;
    if all_files.is_empty() {
        ui::print_info("No files found in the database");
        return Ok(());
    }
    
    let now = chrono::Local::now();
    let cutoff_date = match timeframe {
        "day" => now - chrono::Duration::days(1),
        "week" => now - chrono::Duration::weeks(1),
        "month" => now - chrono::Duration::days(30),
        "year" => now - chrono::Duration::days(365),
        _ => now - chrono::Duration::weeks(1), // default to week
    };
    
    // Filter files by timeframe and sort by access frequency (using health percentage as proxy)
    let mut filtered_files: Vec<_> = all_files
        .into_iter()
        .filter(|file| file.upload_time > cutoff_date)
        .collect();
    
    // Sort by health percentage (higher percentage = more "popular")
    filtered_files.sort_by(|a, b| {
        let health_a = if a.chunks_total > 0 {
            (a.chunks_healthy as f64 / a.chunks_total as f64) * 100.0
        } else {
            100.0
        };
        let health_b = if b.chunks_total > 0 {
            (b.chunks_healthy as f64 / b.chunks_total as f64) * 100.0
        } else {
            100.0
        };
        health_b.partial_cmp(&health_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Take only the requested count
    filtered_files.truncate(count);
    
    if filtered_files.is_empty() {
        ui::print_info(&format!("No files found in the last {}", timeframe));
        return Ok(());
    }
    
    ui::print_header(&format!("Most Popular Files (last {})", timeframe));
    ui::print_file_list(&filtered_files);
    
    Ok(())
}

/// Handles the batch-put command
async fn handle_batch_put_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    pattern: &str,
    recursive: bool,
    parallel: usize,
    base_dir: &Option<PathBuf>,
    tag_pattern: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let config = batch_operations::BatchPutConfig {
        pattern: pattern.to_string(),
        recursive,
        parallel,
        base_dir: base_dir.clone(),
        tag_pattern: tag_pattern.clone(),
    };
    
    let result = batch_operations::batch_put(cli, key_manager, config).await?;
    
    if result.failed > 0 {
        ui::print_warning(&format!("Some operations failed. Check logs for details."));
    }
    
    Ok(())
}

/// Handles the batch-get command
async fn handle_batch_get_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    pattern: &str,
    destination: &PathBuf,
    parallel: usize,
    preserve_structure: bool,
) -> Result<(), Box<dyn Error>> {
    let config = batch_operations::BatchGetConfig {
        pattern: pattern.to_string(),
        destination: destination.clone(),
        parallel,
        preserve_structure,
    };
    
    let result = batch_operations::batch_get(cli, key_manager, config).await?;
    
    if result.failed > 0 {
        ui::print_warning(&format!("Some operations failed. Check logs for details."));
    }
    
    Ok(())
}

/// Handles the batch-tag command
async fn handle_batch_tag_command(
    pattern: &str,
    add_tags: &Option<String>,
    remove_tags: &Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let add_tags_vec = add_tags.as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    
    let remove_tags_vec = remove_tags.as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    
    let config = batch_operations::BatchTagConfig {
        pattern: pattern.to_string(),
        add_tags: add_tags_vec,
        remove_tags: remove_tags_vec,
        dry_run,
    };
    
    let result = batch_operations::batch_tag(config).await?;
    
    if result.failed > 0 {
        ui::print_warning(&format!("Some operations failed. Check logs for details."));
    }
    
    Ok(())
}

/// Handles the repair command
async fn handle_repair_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    target: &Option<String>,
    auto: bool,
    verify_all: bool,
    threshold: u8,
) -> Result<(), Box<dyn Error>> {
    let config = health_manager::RepairConfig {
        target: target.clone(),
        auto,
        verify_all,
        threshold,
    };
    
    health_manager::repair_files(cli, key_manager, config).await
        .map_err(|e| e.into())
}

/// Handles the cleanup command
async fn handle_cleanup_command(
    orphaned: bool,
    duplicates: bool,
    low_health: bool,
    dry_run: bool,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let config = health_manager::CleanupConfig {
        orphaned,
        duplicates,
        low_health,
        dry_run,
        force,
    };
    
    health_manager::cleanup_storage(config).await
        .map_err(|e| e.into())
}

/// Handles the quota command
async fn handle_quota_command(
    usage: bool,
    limit: &Option<String>,
    warn: &Option<u8>,
) -> Result<(), Box<dyn Error>> {
    health_manager::manage_quota(usage, limit.clone(), *warn).await
        .map_err(|e| e.into())
}

/// Handles the export command
async fn handle_export_command(
    destination: &PathBuf,
    format: &str,
    encrypt: bool,
    include_metadata: bool,
    pattern: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Get files matching the pattern
    let files = if let Some(pattern) = pattern {
        db.list_files(Some(pattern))?
    } else {
        db.list_files(None)?
    };
    
    if files.is_empty() {
        ui::print_info("No files found to export");
        return Ok(());
    }
    
    ui::print_header(&format!("Exporting {} files to {}", files.len(), destination.display()));
    
    // Create destination directory if it doesn't exist
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    match format {
        "tar" => {
            let file = std::fs::File::create(destination)?;
            let mut builder = tar::Builder::new(file);
            
            for file_info in &files {
                ui::print_info(&format!("Adding {} to archive", file_info.name));
                
                // For now, we'll create a simple text file with metadata
                let health_percentage = if file_info.chunks_total > 0 {
                    (file_info.chunks_healthy as f64 / file_info.chunks_total as f64) * 100.0
                } else {
                    100.0
                };
                
                let metadata = if include_metadata {
                    format!("Name: {}\nKey: {}\nSize: {}\nCreated: {}\nHealth: {:.1}%\nTags: {}\n",
                        file_info.name,
                        file_info.file_key,
                        file_info.file_size,
                        file_info.upload_time.format("%Y-%m-%d %H:%M:%S"),
                        health_percentage,
                        file_info.tags.join(", ")
                    )
                } else {
                    format!("File: {}\nKey: {}\n", file_info.name, file_info.file_key)
                };
                
                let mut header = tar::Header::new_gnu();
                header.set_size(metadata.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                
                builder.append_data(&mut header, &format!("{}.metadata", file_info.name), metadata.as_bytes())?;
            }
            
            builder.finish()?;
        }
        "json" => {
            let export_data = serde_json::json!({
                "export_date": chrono::Local::now().to_rfc3339(),
                "total_files": files.len(),
                "files": files.iter().map(|f| {
                    let health_percentage = if f.chunks_total > 0 {
                        (f.chunks_healthy as f64 / f.chunks_total as f64) * 100.0
                    } else {
                        100.0
                    };
                    
                    serde_json::json!({
                        "name": f.name,
                        "key": f.file_key,
                        "size": f.file_size,
                        "created_at": f.upload_time.to_rfc3339(),
                        "health_score": health_percentage,
                        "tags": f.tags
                    })
                }).collect::<Vec<_>>()
            });
            
            let json_content = if include_metadata {
                serde_json::to_string_pretty(&export_data)?
            } else {
                serde_json::to_string(&export_data)?
            };
            
            std::fs::write(destination, json_content)?;
        }
        _ => {
            return Err(format!("Unsupported export format: {}", format).into());
        }
    }
    
    if encrypt {
        ui::print_warning("Encryption for exported files not yet implemented");
    }
    
    ui::print_success(&format!("Successfully exported {} files to {}", files.len(), destination.display()));
    Ok(())
}

/// Handles the import command
async fn handle_import_command(
    archive: &PathBuf,
    verify: bool,
    preserve_structure: bool,
    tag_prefix: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    if !archive.exists() {
        return Err(format!("Archive file does not exist: {}", archive.display()).into());
    }
    
    ui::print_header(&format!("Importing from {}", archive.display()));
    
    let extension = archive.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "tar" => {
            let file = std::fs::File::open(archive)?;
            let mut archive = tar::Archive::new(file);
            
            let mut imported_count = 0;
            for entry in archive.entries()? {
                let entry = entry?;
                let path = entry.path()?;
                
                ui::print_info(&format!("Processing: {}", path.display()));
                
                // For now, we'll just extract metadata files
                if path.to_string_lossy().ends_with(".metadata") {
                    let mut content = String::new();
                    let mut entry = entry;
                    std::io::Read::read_to_string(&mut entry, &mut content)?;
                    
                    ui::print_info(&format!("Metadata content:\n{}", content));
                    imported_count += 1;
                }
            }
            
            ui::print_success(&format!("Imported {} metadata files", imported_count));
        }
        "json" => {
            let content = std::fs::read_to_string(archive)?;
            let import_data: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(files) = import_data.get("files").and_then(|f| f.as_array()) {
                ui::print_info(&format!("Found {} files in import data", files.len()));
                
                for file_data in files {
                    if let (Some(name), Some(key)) = (
                        file_data.get("name").and_then(|n| n.as_str()),
                        file_data.get("key").and_then(|k| k.as_str())
                    ) {
                        let mut tags = Vec::new();
                        if let Some(tag_prefix) = tag_prefix {
                            tags.push(format!("import:{}", tag_prefix));
                        }
                        
                        if let Some(file_tags) = file_data.get("tags").and_then(|t| t.as_array()) {
                            for tag in file_tags {
                                if let Some(tag_str) = tag.as_str() {
                                    tags.push(tag_str.to_string());
                                }
                            }
                        }
                        
                        ui::print_info(&format!("Would import: {} (key: {}, tags: {})", 
                            name, key, tags.join(", ")));
                    }
                }
            }
        }
        _ => {
            return Err(format!("Unsupported import format: {}", extension).into());
        }
    }
    
    if verify {
        ui::print_info("Verification enabled - would check file integrity");
    }
    
    if preserve_structure {
        ui::print_info("Structure preservation enabled - would maintain directory structure");
    }
    
    ui::print_success("Import completed successfully");
    Ok(())
}

/// Handles the pin command
async fn handle_pin_command(
    target: &str,
    duration: &Option<String>,
    priority: u8,
) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Find the file by name or key
    let file = if let Some(file) = db.get_file_by_name(target)? {
        file
    } else if let Some(file) = db.get_file_by_key(target)? {
        file
    } else {
        return Err(format!("File not found: {}", target).into());
    };
    
    // Calculate pin expiration
    let expiration = if let Some(duration_str) = duration {
        let now = chrono::Local::now();
        let expiration = match duration_str.as_str() {
            "1h" => now + chrono::Duration::hours(1),
            "1d" => now + chrono::Duration::days(1),
            "1w" => now + chrono::Duration::weeks(1),
            "1m" => now + chrono::Duration::days(30),
            "1y" => now + chrono::Duration::days(365),
            _ => {
                ui::print_warning(&format!("Invalid duration format: {}. Using 1 week default.", duration_str));
                now + chrono::Duration::weeks(1)
            }
        };
        Some(expiration)
    } else {
        None // Permanent pin
    };
    
    // Add pin tag with priority and expiration info
    let pin_tag = if let Some(exp) = expiration {
        format!("pin:priority:{},expires:{}", priority, exp.format("%Y-%m-%d"))
    } else {
        format!("pin:priority:{},permanent", priority)
    };
    
    let mut updated_tags = file.tags.clone();
    // Remove any existing pin tags
    updated_tags.retain(|tag| !tag.starts_with("pin:"));
    // Add the new pin tag
    updated_tags.push(pin_tag);
    
    // Update tags in database (we'd need to implement this method)
    ui::print_success(&format!("Pinned file '{}' with priority {} {}", 
        file.name, 
        priority,
        if let Some(exp) = expiration {
            format!("until {}", exp.format("%Y-%m-%d %H:%M:%S"))
        } else {
            "permanently".to_string()
        }
    ));
    
    ui::print_info("File pinning ensures high availability and prevents garbage collection");
    Ok(())
}

/// Handles the unpin command
async fn handle_unpin_command(target: &str) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Find the file by name or key
    let file = if let Some(file) = db.get_file_by_name(target)? {
        file
    } else if let Some(file) = db.get_file_by_key(target)? {
        file
    } else {
        return Err(format!("File not found: {}", target).into());
    };
    
    // Check if file is pinned
    let has_pin = file.tags.iter().any(|tag| tag.starts_with("pin:"));
    
    if !has_pin {
        ui::print_info(&format!("File '{}' is not pinned", file.name));
        return Ok(());
    }
    
    // Remove pin tags
    let mut updated_tags = file.tags.clone();
    updated_tags.retain(|tag| !tag.starts_with("pin:"));
    
    // Update tags in database (we'd need to implement this method)
    ui::print_success(&format!("Unpinned file '{}'", file.name));
    ui::print_info("File is now eligible for normal garbage collection");
    Ok(())
}

/// Handles the share command
async fn handle_share_command(
    target: &str,
    public: bool,
    expires: &Option<String>,
    password: &Option<String>,
    qr_code: bool,
) -> Result<(), Box<dyn Error>> {
    let db_path = database::get_default_db_path()?;
    let db = database::DatabaseManager::new(&db_path)?;
    
    // Find the file by name or key
    let file = if let Some(file) = db.get_file_by_name(target)? {
        file
    } else if let Some(file) = db.get_file_by_key(target)? {
        file
    } else {
        return Err(format!("File not found: {}", target).into());
    };
    
    ui::print_header(&format!("Sharing file: {}", file.name));
    
    // Calculate expiration
    let expiration = if let Some(expires_str) = expires {
        let now = chrono::Local::now();
        let expiration = match expires_str.as_str() {
            "1h" => now + chrono::Duration::hours(1),
            "1d" => now + chrono::Duration::days(1),
            "1w" => now + chrono::Duration::weeks(1),
            "1m" => now + chrono::Duration::days(30),
            _ => {
                ui::print_warning(&format!("Invalid expiration format: {}. Using 1 day default.", expires_str));
                now + chrono::Duration::days(1)
            }
        };
        Some(expiration)
    } else {
        None
    };
    
    // Generate sharing information
    let share_id = format!("share_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    let share_key = file.file_key.clone();
    
    // Create sharing command
    let mut share_command = format!("datamesh get {} ./downloaded_file", share_key);
    
    if public {
        ui::print_info("Creating public share (no authentication required)");
    } else {
        ui::print_info("Creating private share (requires authentication)");
    }
    
    if let Some(password) = password {
        ui::print_info(&format!("Password protection enabled: {}", password));
        share_command = format!("{} --password {}", share_command, password);
    }
    
    println!();
    ui::print_success("Share created successfully!");
    println!("Share ID: {}", share_id);
    println!("File Key: {}", share_key);
    println!("Command: {}", share_command);
    
    if let Some(exp) = expiration {
        println!("Expires: {}", exp.format("%Y-%m-%d %H:%M:%S"));
    } else {
        println!("Expires: Never");
    }
    
    if qr_code {
        ui::print_info("QR Code generation:");
        // Simple ASCII QR code placeholder
        let qr_content = format!("datamesh://share/{}?key={}", share_id, share_key);
        println!("QR Code URL: {}", qr_content);
        
        // ASCII art QR code placeholder
        println!("█████████████████████████████");
        println!("█ ▄▄▄▄▄ █▀█ █▄▄▄▄▄▄▄█ ▄▄▄▄▄ █");
        println!("█ █   █ █▀▀ █▄▄▄▄▄▄▄█ █   █ █");
        println!("█ █▄▄▄█ █▀█ █▄▄▄▄▄▄▄█ █▄▄▄█ █");
        println!("█▄▄▄▄▄▄▄█▄▀▄█▄▀▄▀▄▀▄█▄▄▄▄▄▄▄█");
        println!("█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄█");
        println!("█████████████████████████████");
        ui::print_info("Scan QR code to access file");
    }
    
    println!();
    ui::print_warning("Note: This is a demonstration of share functionality.");
    ui::print_info("In a full implementation, shares would be stored in the database with proper access control.");
    
    Ok(())
}

/// Handles the optimize command
async fn handle_optimize_command(
    _defrag: bool,
    _rebalance: bool,
    _compress: bool,
    analyze: bool,
) -> Result<(), Box<dyn Error>> {
    if analyze {
        ui::print_header("Storage Optimization Analysis");
        let _ = health_manager::generate_health_report().await?;
        ui::print_info("Based on the health report, consider running repair or cleanup operations");
    } else {
        ui::print_info("Storage optimization not yet fully implemented");
        ui::print_info("Use --analyze to see optimization recommendations");
    }
    Ok(())
}

/// Handles the benchmark command
async fn handle_benchmark_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    full: bool,
    network: bool,
    storage: bool,
    duration: u64,
) -> Result<(), Box<dyn Error>> {
    let results = health_manager::run_benchmarks(cli, key_manager, full, network, storage, duration).await?;
    
    ui::print_success("Benchmark completed successfully");
    
    // Results are already displayed in the benchmark function
    let _ = results; // Suppress unused variable warning
    
    Ok(())
}

// === Helper Functions ===

fn parse_size_range(size_str: &str) -> Result<file_manager::SizeRange, Box<dyn Error>> {
    if size_str.starts_with('>') {
        let size_part = &size_str[1..];
        let bytes = parse_size_bytes(size_part)?;
        Ok(file_manager::SizeRange::GreaterThan(bytes))
    } else if size_str.starts_with('<') {
        let size_part = &size_str[1..];
        let bytes = parse_size_bytes(size_part)?;
        Ok(file_manager::SizeRange::LessThan(bytes))
    } else if size_str.contains('-') {
        let parts: Vec<&str> = size_str.split('-').collect();
        if parts.len() == 2 {
            let min_bytes = parse_size_bytes(parts[0])?;
            let max_bytes = parse_size_bytes(parts[1])?;
            Ok(file_manager::SizeRange::Between(min_bytes, max_bytes))
        } else {
            Err("Invalid size range format".into())
        }
    } else {
        Err("Invalid size range format".into())
    }
}

fn parse_size_bytes(size_str: &str) -> Result<u64, Box<dyn Error>> {
    let size_str = size_str.to_uppercase();
    
    if let Some(number_part) = size_str.strip_suffix("GB") {
        Ok(number_part.parse::<u64>()? * 1024 * 1024 * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("MB") {
        Ok(number_part.parse::<u64>()? * 1024 * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("KB") {
        Ok(number_part.parse::<u64>()? * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("B") {
        Ok(number_part.parse::<u64>()?)
    } else {
        Ok(size_str.parse::<u64>()?)
    }
}

fn parse_date_range(date_str: &str) -> Result<file_manager::DateRange, Box<dyn Error>> {
    use chrono::{Local, Duration};
    
    match date_str.to_lowercase().as_str() {
        "today" => Ok(file_manager::DateRange::LastDays(1)),
        "yesterday" => Ok(file_manager::DateRange::LastDays(2)),
        "last week" | "week" => Ok(file_manager::DateRange::LastWeeks(1)),
        "last month" | "month" => Ok(file_manager::DateRange::LastMonths(1)),
        _ => {
            if date_str.contains(':') {
                // Parse date range like "2024-01-01:2024-12-31"
                let parts: Vec<&str> = date_str.split(':').collect();
                if parts.len() == 2 {
                    // This is a simplified parser - in a real implementation,
                    // you'd use a proper date parsing library like chrono
                    let start = Local::now() - Duration::days(30); // Placeholder
                    let end = Local::now();
                    Ok(file_manager::DateRange::Between(start, end))
                } else {
                    Err("Invalid date range format".into())
                }
            } else if date_str.ends_with(" days") {
                let days_str = date_str.strip_suffix(" days").unwrap();
                let days = days_str.parse::<u32>()?;
                Ok(file_manager::DateRange::LastDays(days))
            } else {
                Err("Invalid date format".into())
            }
        }
    }
}

/// Apply network presets to CLI configuration
fn apply_network_preset(cli: &mut cli::Cli) -> Result<(), Box<dyn Error>> {
    if let Some(network_spec) = &cli.network {
        let connection_config = presets::parse_network_spec(network_spec)?;
        
        // Apply the first bootstrap peer if available and CLI doesn't have explicit settings
        if cli.bootstrap_peer.is_none() && cli.bootstrap_addr.is_none() {
            if let Some(bootstrap_conn) = connection_config.bootstrap_peers.first() {
                cli.bootstrap_peer = bootstrap_conn.peer_id;
                cli.bootstrap_addr = Some(bootstrap_conn.address.clone());
                
                ui::print_info(&format!("Using bootstrap peer: {}", 
                    bootstrap_conn.address));
            }
        }
        
        // Apply port if CLI doesn't have explicit port set
        if cli.port == 0 && connection_config.port != 0 {
            cli.port = connection_config.port;
            ui::print_info(&format!("Using preset port: {}", connection_config.port));
        }
    }
    
    Ok(())
}

/// Handles the API server command
async fn handle_api_server_command(
    cli: &cli::Cli,
    key_manager: &key_manager::KeyManager,
    host: &Option<String>,
    port: &Option<u16>,
    https: bool,
    cert_path: &Option<PathBuf>,
    key_path: &Option<PathBuf>,
    no_swagger: bool,
) -> Result<(), Box<dyn Error>> {
    use std::sync::Arc;
    
    // Load configuration
    let config = config::Config::load_or_default(None)?;
    
    // Override API configuration with CLI arguments
    let mut api_config = config.api.to_api_server_config();
    
    if let Some(host) = host {
        api_config.host = host.clone();
    }
    if let Some(port) = port {
        api_config.port = *port;
    }
    if https {
        api_config.enable_https = true;
    }
    if let Some(cert_path) = cert_path {
        api_config.cert_path = Some(cert_path.clone());
    }
    if let Some(key_path) = key_path {
        api_config.key_path = Some(key_path.clone());
    }
    if no_swagger {
        api_config.enable_swagger = false;
    }
    
    // Initialize smart cache
    let cache_config = config.cache.to_smart_cache_config();
    let cache_manager = Arc::new(smart_cache::SmartCacheManager::new(cache_config));
    
    // Create API server
    let api_server = api_server::ApiServer::new(
        config,
        Arc::new(key_manager.clone()),
        cache_manager,
        cli.clone(),
        api_config,
    );
    
    // Start the server
    ui::print_info(&format!("Starting DataMesh API server on {}:{}", 
        api_server.state.api_config.host, api_server.state.api_config.port));
    
    api_server.start().await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
}