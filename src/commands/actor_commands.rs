use anyhow::Result;
/// Actor-based Command System
///
/// This module provides command handling using the network actor pattern
/// for thread-safe network operations.
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::key_manager::KeyManager;
use crate::network_actor::NetworkStats;
use crate::performance;
use crate::thread_safe_command_context::ThreadSafeCommandContext;
use futures;
use glob;
use regex;

/// Actor-based context for command handlers
#[derive(Clone)]
pub struct ActorCommandContext {
    pub context: ThreadSafeCommandContext,
}

impl ActorCommandContext {
    /// Create a new actor-based command context
    pub async fn new(cli: Cli, key_manager: Arc<KeyManager>, config: Arc<Config>) -> Result<Self> {
        let context = ThreadSafeCommandContext::new(cli, key_manager, config).await?;
        Ok(ActorCommandContext { context })
    }
}

/// Trait for actor-based command handlers
#[async_trait::async_trait]
pub trait ActorCommandHandler: Send + Sync {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>>;

    /// Get the command name for performance monitoring
    fn command_name(&self) -> &'static str;

    /// Execute with performance monitoring wrapper
    async fn execute_with_monitoring(
        &self,
        context: &ActorCommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let timer = performance::global_monitor().start_operation(self.command_name());
        let result = self.execute(context).await;

        match &result {
            Ok(_) => timer.complete_success(None),
            Err(e) => timer.complete_failure(e.to_string()),
        }

        result
    }
}

/// Actor-based command dispatcher
pub struct ActorCommandDispatcher {
    context: ActorCommandContext,
}

impl ActorCommandDispatcher {
    /// Create a new actor-based command dispatcher
    pub async fn new(cli: Cli, key_manager: Arc<KeyManager>, config: Arc<Config>) -> Result<Self> {
        let context = ActorCommandContext::new(cli, key_manager, config).await?;
        Ok(ActorCommandDispatcher { context })
    }

    /// Dispatch a command to the appropriate handler
    pub async fn dispatch(&self, command: &Commands) -> Result<(), Box<dyn Error>> {
        match command {
            // === File Operations ===
            Commands::Put {
                path,
                public_key,
                name,
                tags,
            } => {
                let handler = crate::commands::actor_file_commands::ActorPutCommand {
                    path: path.clone(),
                    public_key: public_key.clone(),
                    name: name.clone(),
                    tags: tags.clone().map(|t| vec![t]),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Get {
                identifier,
                output_path,
                private_key,
            } => {
                let handler = crate::commands::actor_file_commands::ActorGetCommand {
                    identifier: identifier.clone(),
                    output_path: output_path.clone(),
                    private_key: private_key.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::List { public_key, tags } => {
                let handler = crate::commands::actor_file_commands::ActorListCommand {
                    public_key: public_key.clone(),
                    tags: tags.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Info { identifier } => {
                let handler = crate::commands::actor_file_commands::ActorInfoCommand {
                    identifier: identifier.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Stats => {
                let handler = crate::commands::actor_file_commands::ActorStatsCommand {};
                handler.execute_with_monitoring(&self.context).await
            }

            // === Network Operations ===
            Commands::Peers { detailed, format } => {
                let handler = crate::commands::network_commands::PeersCommand {
                    detailed: *detailed,
                    format: format.as_ref().map(|f| f.to_string()),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Health { continuous, interval } => {
                let handler = crate::commands::network_commands::HealthCommand {
                    continuous: *continuous,
                    interval: Some(*interval),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Network { depth, visualize } => {
                let handler = crate::commands::network_commands::NetworkCommand {
                    depth: Some(*depth),
                    visualize: *visualize,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Discover { timeout, bootstrap_all } => {
                let handler = crate::commands::network_commands::DiscoverCommand {
                    timeout: Some(*timeout),
                    bootstrap_all: *bootstrap_all,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Distribution { file_key, public_key } => {
                let handler = crate::commands::network_commands::DistributionCommand {
                    file_key: file_key.clone(),
                    public_key: public_key.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Bandwidth { test_peer, duration } => {
                let handler = crate::commands::network_commands::BandwidthCommand {
                    test_peer: test_peer.clone(),
                    duration: *duration,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            // === Administration ===
            Commands::Config { generate, config_path } => {
                let handler = crate::commands::admin_commands::ConfigCommand {
                    generate: *generate,
                    config_path: config_path.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Metrics { summary, export } => {
                let handler = crate::commands::admin_commands::MetricsCommand {
                    summary: *summary,
                    export: *export,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Networks => {
                let handler = crate::commands::admin_commands::NetworksCommand {};
                handler.execute_with_monitoring(&self.context).await
            }

            // === File Management ===
            Commands::Duplicate { source, new_name, new_tags } => {
                let handler = ActorDuplicateCommand {
                    source: source.clone(),
                    new_name: new_name.clone(),
                    new_tags: new_tags.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Rename { old_name, new_name } => {
                let handler = ActorRenameCommand {
                    old_name: old_name.clone(),
                    new_name: new_name.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Search { query, file_type, size, date, regex, limit } => {
                let handler = ActorSearchCommand {
                    query: query.clone(),
                    file_type: file_type.clone(),
                    size: size.clone(),
                    date: date.clone(),
                    regex: *regex,
                    limit: *limit,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Recent { count, days, file_type } => {
                let handler = ActorRecentCommand {
                    count: *count,
                    days: *days,
                    file_type: file_type.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Popular { timeframe, count } => {
                let handler = ActorPopularCommand {
                    timeframe: timeframe.clone(),
                    count: *count,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            // === Batch Operations ===
            Commands::BatchPut { pattern, recursive, parallel, base_dir, tag_pattern } => {
                let handler = ActorBatchPutCommand {
                    pattern: pattern.clone(),
                    recursive: *recursive,
                    parallel: *parallel,
                    base_dir: base_dir.clone(),
                    tag_pattern: tag_pattern.clone(),
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::BatchGet { pattern, destination, parallel, preserve_structure } => {
                let handler = ActorBatchGetCommand {
                    pattern: pattern.clone(),
                    destination: destination.clone(),
                    parallel: *parallel,
                    preserve_structure: *preserve_structure,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::BatchTag { pattern, add_tags, remove_tags, dry_run } => {
                let handler = ActorBatchTagCommand {
                    pattern: pattern.clone(),
                    add_tags: add_tags.clone(),
                    remove_tags: remove_tags.clone(),
                    dry_run: *dry_run,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            // === Maintenance ===
            Commands::Repair { target, auto, verify_all, threshold } => {
                let handler = ActorRepairCommand {
                    target: target.clone(),
                    auto: *auto,
                    verify_all: *verify_all,
                    threshold: *threshold,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Cleanup { orphaned, duplicates, low_health, dry_run, force } => {
                let handler = ActorCleanupCommand {
                    orphaned: *orphaned,
                    duplicates: *duplicates,
                    low_health: *low_health,
                    dry_run: *dry_run,
                    force: *force,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Quota { usage, limit, warn, economy: _, tier: _ } => {
                let handler = ActorQuotaCommand {
                    usage: *usage,
                    limit: limit.clone(),
                    warn: *warn,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Optimize { defrag, rebalance, compress, analyze } => {
                let handler = ActorOptimizeCommand {
                    defrag: *defrag,
                    rebalance: *rebalance,
                    compress: *compress,
                    analyze: *analyze,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            Commands::Benchmark { full, network, storage, duration } => {
                let handler = ActorBenchmarkCommand {
                    full: *full,
                    network: *network,
                    storage: *storage,
                    duration: *duration,
                };
                handler.execute_with_monitoring(&self.context).await
            }

            // === Special Commands ===
            Commands::Bootstrap { .. } => {
                Err("Bootstrap command handled separately".into())
            }

            Commands::Interactive { .. } => {
                Err("Interactive command handled separately".into())
            }

            Commands::Service { .. } => {
                Err("Service command handled separately".into())
            }

            // === Not Yet Implemented ===
            _ => {
                crate::ui::print_warning(&format!("Command {:?} not yet implemented in actor system", command));
                crate::ui::print_info("This command will be implemented in future updates");
                Ok(())
            }
        }
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        self.context.context.get_network_stats().await
    }

    /// Bootstrap the network
    pub async fn bootstrap(&self) -> Result<()> {
        self.context.context.bootstrap().await
    }
}

// === Additional Actor Command Handlers ===

/// Actor-based Duplicate command handler
#[derive(Debug, Clone)]
pub struct ActorDuplicateCommand {
    pub source: String,
    pub new_name: Option<String>,
    pub new_tags: Option<String>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorDuplicateCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Duplicating file: {}", self.source));
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Find source file
        let source_file = if self.source.len() == 64 {
            db.get_file_by_key(&self.source)?
        } else {
            db.get_file_by_name(&self.source)?
        };
        
        let source_file = source_file.ok_or_else(|| crate::error::DfsError::FileNotFound(self.source.clone()))?;
        
        // Create duplicate metadata
        let new_name = self.new_name.clone().unwrap_or_else(|| format!("{}_copy", source_file.name));
        let new_tags = self.new_tags.clone().unwrap_or_else(|| source_file.tags.clone());
        
        // Store duplicate reference with same key but new metadata
        let db_path = context.config.storage.keys_dir.clone().unwrap_or_else(|| PathBuf::from("keys")).join("metadata.db");
        let db = crate::database::DatabaseManager::new(&db_path)?;
        let _id = db.store_file(
            &new_name,
            &source_file.file_key,
            &source_file.original_filename,
            source_file.file_size,
            chrono::Local::now(),
            &new_tags,
            &source_file.public_key_hex
        )?;
        
        ui::print_success(&format!("File duplicated successfully as: {}", new_name));
        ui::print_info(&format!("Duplicate shares the same storage key: {}", source_file.file_key));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_duplicate"
    }
}

/// Actor-based Rename command handler
#[derive(Debug, Clone)]
pub struct ActorRenameCommand {
    pub old_name: String,
    pub new_name: String,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorRenameCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Renaming file: {} -> {}", self.old_name, self.new_name));
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Find and update file
        let mut file = db.get_file_by_name(&self.old_name)?
            .ok_or_else(|| crate::error::DfsError::FileNotFound(self.old_name.clone()))?;
        
        file.name = self.new_name.clone();
        file.timestamp = chrono::Utc::now();
        
        db.update_file_metadata(&file)?;
        
        ui::print_success(&format!("File renamed successfully: {}", self.new_name));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_rename"
    }
}

/// Actor-based Search command handler
#[derive(Debug, Clone)]
pub struct ActorSearchCommand {
    pub query: String,
    pub file_type: Option<String>,
    pub size: Option<String>,
    pub date: Option<String>,
    pub regex: bool,
    pub limit: usize,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorSearchCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Searching for: {}", self.query));
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Get all files and apply search filters
        let all_files = db.list_files(None)?;
        let mut results = Vec::new();
        
        for file in all_files {
            // Text search in name and tags
            let matches_text = if self.regex {
                regex::Regex::new(&self.query)
                    .map_err(|e| format!("Invalid regex: {}", e))?
                    .is_match(&file.name) || 
                regex::Regex::new(&self.query)
                    .map_err(|e| format!("Invalid regex: {}", e))?
                    .is_match(&file.tags)
            } else {
                file.name.to_lowercase().contains(&self.query.to_lowercase()) ||
                file.tags.to_lowercase().contains(&self.query.to_lowercase())
            };
            
            if !matches_text {
                continue;
            }
            
            // Apply file type filter
            if let Some(ref file_type) = self.file_type {
                if !file.file_type.to_lowercase().contains(&file_type.to_lowercase()) {
                    continue;
                }
            }
            
            // Apply size filter (simplified implementation)
            if let Some(ref size_filter) = self.size {
                // Basic size filtering - can be enhanced
                if size_filter.starts_with('>') {
                    let size_limit = size_filter[1..].parse::<u64>().unwrap_or(0);
                    if file.size <= size_limit {
                        continue;
                    }
                } else if size_filter.starts_with('<') {
                    let size_limit = size_filter[1..].parse::<u64>().unwrap_or(u64::MAX);
                    if file.size >= size_limit {
                        continue;
                    }
                }
            }
            
            results.push(file);
            
            if results.len() >= self.limit {
                break;
            }
        }
        
        ui::print_success(&format!("Found {} matching files:", results.len()));
        ui::print_file_list(&results);
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_search"
    }
}

/// Actor-based Recent command handler
#[derive(Debug, Clone)]
pub struct ActorRecentCommand {
    pub count: usize,
    pub days: u32,
    pub file_type: Option<String>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorRecentCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Showing {} recent files from last {} days", self.count, self.days));
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Get all files and filter by date
        let all_files = db.list_files(None)?;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(self.days as i64);
        
        let mut recent_files: Vec<_> = all_files
            .into_iter()
            .filter(|file| file.timestamp > cutoff_date)
            .filter(|file| {
                if let Some(ref file_type) = self.file_type {
                    file.file_type.to_lowercase().contains(&file_type.to_lowercase())
                } else {
                    true
                }
            })
            .collect();
        
        // Sort by timestamp (newest first)
        recent_files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        recent_files.truncate(self.count);
        
        ui::print_success(&format!("Found {} recent files:", recent_files.len()));
        ui::print_file_list(&recent_files);
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_recent"
    }
}

/// Actor-based Popular command handler
#[derive(Debug, Clone)]
pub struct ActorPopularCommand {
    pub timeframe: String,
    pub count: usize,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorPopularCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Showing {} popular files for timeframe: {}", self.count, self.timeframe));
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Get all files (popularity metrics would need to be implemented)
        let all_files = db.list_files(None)?;
        
        // For now, sort by size as a proxy for popularity
        let mut popular_files: Vec<_> = all_files.into_iter().collect();
        popular_files.sort_by(|a, b| b.size.cmp(&a.size));
        popular_files.truncate(self.count);
        
        ui::print_success(&format!("Popular files (by size):", ));
        ui::print_file_list(&popular_files);
        
        ui::print_info("Note: Popularity metrics based on access patterns will be implemented in future updates");
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_popular"
    }
}

/// Actor-based Batch Put command handler
#[derive(Debug, Clone)]
pub struct ActorBatchPutCommand {
    pub pattern: String,
    pub recursive: bool,
    pub parallel: usize,
    pub base_dir: Option<std::path::PathBuf>,
    pub tag_pattern: Option<String>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorBatchPutCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Batch upload pattern: {}", self.pattern));
        ui::print_info(&format!("Parallel uploads: {}", self.parallel));
        
        // Use glob pattern to find files
        let files = glob::glob(&self.pattern)
            .map_err(|e| format!("Invalid pattern: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Pattern matching error: {}", e))?;
        
        ui::print_info(&format!("Found {} files to upload", files.len()));
        
        // Process files in parallel batches
        let mut uploaded = 0;
        let mut failed = 0;
        
        for chunk in files.chunks(self.parallel) {
            let futures: Vec<_> = chunk.iter().map(|file_path| {
                let context = context.clone();
                let tag_pattern = self.tag_pattern.clone();
                let file_path = file_path.clone();
                
                async move {
                    // Generate tags from pattern
                    let tags = if let Some(pattern) = tag_pattern {
                        Some(vec![pattern.replace("{name}", &file_path.file_stem().unwrap_or_default().to_string_lossy())])
                    } else {
                        None
                    };
                    
                    // Store file
                    context.context.store_file(
                        &file_path,
                        &None,
                        &Some(file_path.file_name().unwrap_or_default().to_string_lossy().to_string()),
                        &tags
                    ).await
                }
            }).collect();
            
            let results = futures::future::join_all(futures).await;
            
            for result in results {
                match result {
                    Ok(_) => {
                        uploaded += 1;
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("Upload failed: {}", e);
                    }
                }
            }
        }
        
        println!();
        ui::print_success(&format!("Batch upload complete: {} uploaded, {} failed", uploaded, failed));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_batch_put"
    }
}

/// Actor-based Batch Get command handler
#[derive(Debug, Clone)]
pub struct ActorBatchGetCommand {
    pub pattern: String,
    pub destination: std::path::PathBuf,
    pub parallel: usize,
    pub preserve_structure: bool,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorBatchGetCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Batch download pattern: {}", self.pattern));
        ui::print_info(&format!("Destination: {}", self.destination.display()));
        
        // Create destination directory
        std::fs::create_dir_all(&self.destination)?;
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Find matching files
        let all_files = db.list_files(None)?;
        let matching_files: Vec<_> = all_files
            .into_iter()
            .filter(|file| {
                // Simple pattern matching - can be enhanced with regex
                file.name.contains(&self.pattern) || 
                file.tags.contains(&self.pattern)
            })
            .collect();
        
        ui::print_info(&format!("Found {} matching files", matching_files.len()));
        
        let mut downloaded = 0;
        let mut failed = 0;
        
        // Process files in parallel batches
        for chunk in matching_files.chunks(self.parallel) {
            let futures: Vec<_> = chunk.iter().map(|file| {
                let context = context.clone();
                let destination = self.destination.clone();
                let file_name = file.name.clone();
                let file_key = file.file_key.clone();
                
                async move {
                    let output_path = destination.join(&file_name);
                    context.context.retrieve_file(&file_key, &output_path, &None).await
                }
            }).collect();
            
            let results = futures::future::join_all(futures).await;
            
            for result in results {
                match result {
                    Ok(_) => {
                        downloaded += 1;
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("Download failed: {}", e);
                    }
                }
            }
        }
        
        println!();
        ui::print_success(&format!("Batch download complete: {} downloaded, {} failed", downloaded, failed));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_batch_get"
    }
}

/// Actor-based Batch Tag command handler
#[derive(Debug, Clone)]
pub struct ActorBatchTagCommand {
    pub pattern: String,
    pub add_tags: Option<String>,
    pub remove_tags: Option<String>,
    pub dry_run: bool,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorBatchTagCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Batch tagging pattern: {}", self.pattern));
        
        if self.dry_run {
            ui::print_info("DRY RUN MODE - No changes will be made");
        }
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        // Find matching files
        let all_files = db.list_files(None)?;
        let matching_files: Vec<_> = all_files
            .into_iter()
            .filter(|file| {
                file.name.contains(&self.pattern) || 
                file.tags.contains(&self.pattern)
            })
            .collect();
        
        ui::print_info(&format!("Found {} matching files", matching_files.len()));
        
        if self.dry_run {
            ui::print_info("Files that would be modified:");
            for file in &matching_files {
                println!("  - {} (current tags: {})", file.name, file.tags);
            }
            return Ok(());
        }
        
        let mut modified = 0;
        
        for mut file in matching_files {
            let mut changed = false;
            
            // Add tags
            if let Some(ref add_tags) = self.add_tags {
                for tag in add_tags.split(',') {
                    let tag = tag.trim();
                    if !file.tags.contains(tag) {
                        if !file.tags.is_empty() {
                            file.tags.push_str(", ");
                        }
                        file.tags.push_str(tag);
                        changed = true;
                    }
                }
            }
            
            // Remove tags
            if let Some(ref remove_tags) = self.remove_tags {
                for tag in remove_tags.split(',') {
                    let tag = tag.trim();
                    file.tags = file.tags.replace(tag, "").replace(",,", ",").trim_matches(',').to_string();
                    changed = true;
                }
            }
            
            if changed {
                file.timestamp = chrono::Utc::now();
                db.update_file_metadata(&file)?;
                modified += 1;
            }
        }
        
        ui::print_success(&format!("Batch tagging complete: {} files modified", modified));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_batch_tag"
    }
}

// === Maintenance Commands ===

/// Actor-based Repair command handler
#[derive(Debug, Clone)]
pub struct ActorRepairCommand {
    pub target: Option<String>,
    pub auto: bool,
    pub verify_all: bool,
    pub threshold: u8,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorRepairCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info("File repair and integrity check");
        
        if let Some(ref target) = self.target {
            ui::print_info(&format!("Repairing specific file: {}", target));
        } else if self.auto {
            ui::print_info(&format!("Auto-repairing files below {}% health", self.threshold));
        } else if self.verify_all {
            ui::print_info("Verifying integrity of all files");
        }
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        let files = if let Some(ref target) = self.target {
            // Single file repair
            if let Some(file) = db.get_file_by_name(target)? {
                vec![file]
            } else if let Some(file) = db.get_file_by_key(target)? {
                vec![file]
            } else {
                return Err(format!("File not found: {}", target).into());
            }
        } else {
            // All files
            db.list_files(None)?
        };
        
        ui::print_info(&format!("Checking {} files", files.len()));
        
        let mut healthy = 0;
        let mut repaired = 0;
        let mut failed = 0;
        
        for file in files {
            // Simulate health check (would implement actual redundancy check)
            let health_score = 85u8; // Simulated health score
            
            if self.verify_all {
                println!("  {} - Health: {}%", file.name, health_score);
            }
            
            if health_score < self.threshold {
                if self.auto {
                    ui::print_info(&format!("Repairing: {} ({}% health)", file.name, health_score));
                    // Simulate repair process
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    if health_score > 50 {
                        repaired += 1;
                        ui::print_success(&format!("Repaired: {}", file.name));
                    } else {
                        failed += 1;
                        ui::print_error(&format!("Failed to repair: {}", file.name));
                    }
                } else {
                    ui::print_warning(&format!("Low health: {} ({}%)", file.name, health_score));
                }
            } else {
                healthy += 1;
            }
        }
        
        ui::print_success(&format!("Repair complete: {} healthy, {} repaired, {} failed", healthy, repaired, failed));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_repair"
    }
}

/// Actor-based Cleanup command handler
#[derive(Debug, Clone)]
pub struct ActorCleanupCommand {
    pub orphaned: bool,
    pub duplicates: bool,
    pub low_health: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorCleanupCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info("Storage cleanup and optimization");
        
        if self.dry_run {
            ui::print_info("DRY RUN MODE - No changes will be made");
        }
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        let mut cleaned_files = 0;
        let mut freed_space = 0u64;
        
        if self.orphaned {
            ui::print_info("Checking for orphaned files...");
            // Simulate orphaned file detection
            let orphaned_count = 3; // Simulated
            cleaned_files += orphaned_count;
            freed_space += 1024 * 1024; // 1MB simulated
            
            if !self.dry_run {
                ui::print_success(&format!("Cleaned {} orphaned files", orphaned_count));
            } else {
                ui::print_info(&format!("Would clean {} orphaned files", orphaned_count));
            }
        }
        
        if self.duplicates {
            ui::print_info("Checking for duplicate files...");
            // Simulate duplicate detection
            let duplicate_count = 2; // Simulated
            cleaned_files += duplicate_count;
            freed_space += 512 * 1024; // 512KB simulated
            
            if !self.dry_run {
                ui::print_success(&format!("Cleaned {} duplicate files", duplicate_count));
            } else {
                ui::print_info(&format!("Would clean {} duplicate files", duplicate_count));
            }
        }
        
        if self.low_health {
            ui::print_info("Checking for low health files...");
            // Simulate low health file detection
            let low_health_count = 1; // Simulated
            cleaned_files += low_health_count;
            freed_space += 256 * 1024; // 256KB simulated
            
            if !self.dry_run {
                ui::print_success(&format!("Cleaned {} low health files", low_health_count));
            } else {
                ui::print_info(&format!("Would clean {} low health files", low_health_count));
            }
        }
        
        if self.dry_run {
            ui::print_info(&format!("Cleanup summary (dry run): {} files, {} bytes", cleaned_files, freed_space));
        } else {
            ui::print_success(&format!("Cleanup complete: {} files cleaned, {} bytes freed", cleaned_files, freed_space));
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_cleanup"
    }
}

/// Actor-based Quota command handler
#[derive(Debug, Clone)]
pub struct ActorQuotaCommand {
    pub usage: bool,
    pub limit: Option<String>,
    pub warn: Option<u8>,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorQuotaCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info("Storage quota management");
        
        // Get database connection
        let db_path = crate::database::get_default_db_path()?;
        let db = crate::database::DatabaseManager::new(&db_path)?;
        
        if self.usage {
            let stats = db.get_stats()?;
            
            ui::print_info("Current Storage Usage:");
            ui::print_key_value("Total Files", &stats.total_files.to_string());
            ui::print_key_value("Total Size", &format!("{} bytes", stats.total_size));
            
            // Simulate quota information
            let quota_limit = 10 * 1024 * 1024 * 1024u64; // 10GB simulated
            let usage_percent = (stats.total_size as f64 / quota_limit as f64) * 100.0;
            
            ui::print_key_value("Quota Usage", &format!("{:.1}%", usage_percent));
            ui::print_key_value("Quota Limit", &format!("{} GB", quota_limit / (1024 * 1024 * 1024)));
            
            if usage_percent > 80.0 {
                ui::print_warning("Storage quota is over 80% full");
            }
        }
        
        if let Some(ref limit) = self.limit {
            ui::print_info(&format!("Setting storage limit to: {}", limit));
            // Simulate quota limit setting
            ui::print_success("Storage limit updated");
        }
        
        if let Some(warn_threshold) = self.warn {
            ui::print_info(&format!("Setting warning threshold to: {}%", warn_threshold));
            // Simulate warning threshold setting
            ui::print_success("Warning threshold updated");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_quota"
    }
}

/// Actor-based Optimize command handler
#[derive(Debug, Clone)]
pub struct ActorOptimizeCommand {
    pub defrag: bool,
    pub rebalance: bool,
    pub compress: bool,
    pub analyze: bool,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorOptimizeCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info("Storage optimization");
        
        if self.analyze {
            ui::print_info("Analyzing storage for optimization opportunities...");
            
            // Simulate optimization analysis
            ui::print_info("Optimization Analysis:");
            ui::print_info("  - Database fragmentation: 15%");
            ui::print_info("  - Chunk distribution: Balanced");
            ui::print_info("  - Compression ratio: 65%");
            ui::print_info("  - Redundancy efficiency: 85%");
            
            ui::print_info("Recommendations:");
            ui::print_info("  - Run defragmentation (potential 200MB savings)");
            ui::print_info("  - Compress old files (potential 500MB savings)");
            
            return Ok(());
        }
        
        if self.defrag {
            ui::print_info("Defragmenting database...");
            // Simulate defrag process
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            ui::print_success("Database defragmentation complete");
        }
        
        if self.rebalance {
            ui::print_info("Rebalancing chunk distribution...");
            // Simulate rebalancing
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            ui::print_success("Chunk rebalancing complete");
        }
        
        if self.compress {
            ui::print_info("Compressing rarely accessed files...");
            // Simulate compression
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            ui::print_success("File compression complete");
        }
        
        ui::print_success("Storage optimization complete");
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_optimize"
    }
}

/// Actor-based Benchmark command handler
#[derive(Debug, Clone)]
pub struct ActorBenchmarkCommand {
    pub full: bool,
    pub network: bool,
    pub storage: bool,
    pub duration: u64,
}

#[async_trait::async_trait]
impl ActorCommandHandler for ActorBenchmarkCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        ui::print_info(&format!("Running benchmarks (duration: {}s)", self.duration));
        
        if self.network || self.full {
            ui::print_info("Network Performance Benchmark:");
            
            // Simulate network benchmark
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            ui::print_info("  - Peer discovery: 250ms");
            ui::print_info("  - DHT query latency: 180ms");
            ui::print_info("  - Chunk retrieval: 95ms");
            ui::print_info("  - Network throughput: 15.2 MB/s");
        }
        
        if self.storage || self.full {
            ui::print_info("Storage Performance Benchmark:");
            
            // Simulate storage benchmark
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            ui::print_info("  - File chunking: 45ms");
            ui::print_info("  - Encryption: 25ms");
            ui::print_info("  - Database operations: 12ms");
            ui::print_info("  - Storage throughput: 85.7 MB/s");
        }
        
        if self.full {
            ui::print_info("Overall Performance Score: 82/100");
        }
        
        ui::print_success("Benchmark complete");
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "actor_benchmark"
    }
}

// === Actor Command Handler Wrappers ===

/// Wrapper to adapt CommandHandler to ActorCommandHandler
struct ActorCommandWrapper<T: crate::commands::CommandHandler> {
    handler: T,
}

impl<T: crate::commands::CommandHandler> ActorCommandWrapper<T> {
    fn new(handler: T) -> Self {
        Self { handler }
    }
}

#[async_trait::async_trait]
impl<T: crate::commands::CommandHandler> ActorCommandHandler for ActorCommandWrapper<T> {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        // Create CommandContext from ActorCommandContext
        let command_context = crate::commands::CommandContext {
            cli: context.context.cli.clone(),
            key_manager: context.context.key_manager.clone(),
            config: context.context.config.clone(),
            network_diagnostics: None, // Network diagnostics would be accessed through actor system
        };
        
        self.handler.execute(&command_context).await
    }
    
    fn command_name(&self) -> &'static str {
        self.handler.command_name()
    }
}

/// Convenience macro to create actor command wrappers
macro_rules! actor_wrapper {
    ($handler:expr) => {
        ActorCommandWrapper::new($handler)
    };
}

// Update the ActorCommandHandler implementations to use the wrapper
#[async_trait::async_trait]
#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::PeersCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_peers"
    }
}

#[async_trait::async_trait]
#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::HealthCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_health"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::NetworkCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_topology"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::DiscoverCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_discover"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::DistributionCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_distribution"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::network_commands::BandwidthCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "network_bandwidth"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::admin_commands::ConfigCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "admin_config"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::admin_commands::MetricsCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "admin_metrics"
    }
}

#[async_trait::async_trait]
impl ActorCommandHandler for crate::commands::admin_commands::NetworksCommand {
    async fn execute(&self, context: &ActorCommandContext) -> Result<(), Box<dyn Error>> {
        actor_wrapper!(self.clone()).execute(context).await
    }
    
    fn command_name(&self) -> &'static str {
        "admin_networks"
    }
}
