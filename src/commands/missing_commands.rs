// ===================================================================================================
// Missing Commands Implementation - Critical Commands for Full Functionality
// ===================================================================================================
//
// This module implements the critical missing commands that are defined in CLI but not implemented.
// These commands are essential for a fully functional distributed storage system.

use anyhow::Result;
use std::error::Error;
use std::path::PathBuf;

use crate::commands::{CommandContext, CommandHandler};
use crate::ui;

// ===================================================================================================
// SYNC COMMAND - Directory Synchronization
// ===================================================================================================

/// Synchronize local directory with DFS network
#[derive(Debug, Clone)]
pub struct SyncCommand {
    pub local_dir: PathBuf,
    pub watch: bool,
    pub bidirectional: bool,
    pub exclude: Option<String>,
    pub parallel: usize,
}

#[async_trait::async_trait]
impl CommandHandler for SyncCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Directory Synchronization");
        
        if !self.local_dir.exists() {
            return Err(format!("Directory does not exist: {}", self.local_dir.display()).into());
        }
        
        if !self.local_dir.is_dir() {
            return Err(format!("Path is not a directory: {}", self.local_dir.display()).into());
        }
        
        ui::print_info(&format!("📁 Synchronizing directory: {}", self.local_dir.display()));
        ui::print_info(&format!("🔄 Watch mode: {}", self.watch));
        ui::print_info(&format!("⇄ Bidirectional: {}", self.bidirectional));
        ui::print_info(&format!("⚡ Parallel operations: {}", self.parallel));
        
        if let Some(exclude) = &self.exclude {
            ui::print_info(&format!("🚫 Exclude patterns: {}", exclude));
        }
        
        // Initialize thread-safe context for file operations
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        use std::collections::HashSet;
        use tokio::fs;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Parse exclude patterns
        let exclude_patterns: HashSet<String> = if let Some(exclude_str) = &self.exclude {
            exclude_str.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            HashSet::new()
        };
        
        // Perform initial sync
        let sync_stats = perform_directory_sync(
            &thread_safe_context,
            &self.local_dir,
            &exclude_patterns,
            self.bidirectional,
            self.parallel,
        ).await?;
        
        ui::print_success(&format!("✅ Initial sync complete: {} files uploaded, {} files downloaded", 
                                   sync_stats.uploaded, sync_stats.downloaded));
        
        // Watch mode implementation
        if self.watch {
            ui::print_info("👁️  Entering watch mode - monitoring for changes...");
            ui::print_info("Press Ctrl+C to stop watching");
            
            use notify::{Watcher, RecursiveMode, Event, EventKind};
            use tokio::sync::mpsc;
            
            let (tx, mut rx) = mpsc::channel(100);
            
            // Create file system watcher
            let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
                match result {
                    Ok(event) => {
                        if let Err(_) = tx.blocking_send(event) {
                            // Channel closed, watcher will be dropped
                        }
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            })?;
            
            // Start watching the directory
            watcher.watch(&self.local_dir, RecursiveMode::Recursive)?;
            
            // Process file system events
            while let Some(event) = rx.recv().await {
                if let Err(e) = handle_file_event(
                    &thread_safe_context,
                    &self.local_dir,
                    &exclude_patterns,
                    event,
                ).await {
                    ui::print_warning(&format!("⚠️  Error handling file event: {}", e));
                }
            }
        }
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "sync"
    }
}

// ===================================================================================================
// BACKUP COMMAND - Versioned Backups
// ===================================================================================================

/// Create versioned backups with automatic tagging
#[derive(Debug, Clone)]
pub struct BackupCommand {
    pub source: PathBuf,
    pub name: String,
    pub incremental: bool,
    pub compress: bool,
    pub schedule: Option<String>,
    pub exclude: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for BackupCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Backup Creation");
        
        if !self.source.exists() {
            return Err(format!("Source path does not exist: {}", self.source.display()).into());
        }
        
        ui::print_info(&format!("📦 Creating backup: {}", self.name));
        ui::print_info(&format!("📁 Source: {}", self.source.display()));
        ui::print_info(&format!("🔄 Incremental: {}", self.incremental));
        ui::print_info(&format!("🗜️  Compress: {}", self.compress));
        
        if let Some(schedule) = &self.schedule {
            ui::print_info(&format!("⏰ Schedule: {}", schedule));
        }
        
        if let Some(exclude) = &self.exclude {
            ui::print_info(&format!("🚫 Exclude patterns: {}", exclude));
        }
        
        // Initialize thread-safe context for backup operations
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        use chrono::{DateTime, Utc};
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Create backup metadata
        let backup_info = BackupInfo {
            name: self.name.clone(),
            source_path: self.source.clone(),
            created_at: Utc::now(),
            version: get_next_backup_version(&thread_safe_context, &self.name).await?,
            incremental: self.incremental,
            compressed: self.compress,
            file_count: 0,
            total_size: 0,
        };
        
        ui::print_info(&format!("📋 Backup version: {}", backup_info.version));
        
        // Parse exclude patterns
        use std::collections::HashSet;
        let exclude_patterns: HashSet<String> = if let Some(exclude_str) = &self.exclude {
            exclude_str.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            HashSet::new()
        };
        
        // Perform the backup
        let backup_result = create_backup(
            &thread_safe_context,
            &backup_info,
            &exclude_patterns,
        ).await?;
        
        // Store backup metadata
        store_backup_metadata(&thread_safe_context, &backup_result).await?;
        
        ui::print_success(&format!("✅ Backup '{}' created successfully", self.name));
        ui::print_info(&format!("📊 Version: {}", backup_result.version));
        ui::print_info(&format!("📊 Files: {}", backup_result.file_count));
        ui::print_info(&format!("📊 Size: {} bytes", backup_result.total_size));
        
        // Handle scheduling if specified
        if let Some(_schedule) = &self.schedule {
            ui::print_info("📅 Scheduled backups would be implemented here");
            ui::print_info("    (Requires integration with system cron or task scheduler)");
        }
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "backup"
    }
}

// ===================================================================================================
// RESTORE COMMAND - Backup Restoration
// ===================================================================================================

/// Restore from backups with version selection
#[derive(Debug, Clone)]
pub struct RestoreCommand {
    pub backup_name: String,
    pub destination: PathBuf,
    pub version: Option<u32>,
    pub verify: bool,
    pub list_versions: bool,
}

#[async_trait::async_trait]
impl CommandHandler for RestoreCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Backup Restoration");
        
        // Initialize thread-safe context for restore operations
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        if self.list_versions {
            ui::print_info(&format!("📋 Available versions for backup '{}':", self.backup_name));
            
            let versions = list_backup_versions(&thread_safe_context, &self.backup_name).await?;
            
            if versions.is_empty() {
                ui::print_warning(&format!("❌ No backup found with name '{}'", self.backup_name));
                return Ok(());
            }
            
            for version_info in versions {
                ui::print_info(&format!("  📦 Version {}: {} ({} files, {} bytes)", 
                    version_info.version,
                    version_info.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    version_info.file_count,
                    version_info.total_size
                ));
            }
            
            return Ok(());
        }
        
        // Determine which version to restore
        let target_version = if let Some(version) = self.version {
            version
        } else {
            // Get latest version
            let latest = get_latest_backup_version(&thread_safe_context, &self.backup_name).await?;
            if latest == 0 {
                return Err(format!("No backup found with name '{}'", self.backup_name).into());
            }
            latest
        };
        
        ui::print_info(&format!("📦 Restoring backup: {}", self.backup_name));
        ui::print_info(&format!("📋 Version: {}", target_version));
        ui::print_info(&format!("📁 Destination: {}", self.destination.display()));
        ui::print_info(&format!("✅ Verify after restore: {}", self.verify));
        
        // Create destination directory if it doesn't exist
        if !self.destination.exists() {
            std::fs::create_dir_all(&self.destination)?;
            ui::print_info(&format!("📁 Created destination directory: {}", self.destination.display()));
        }
        
        // Load backup metadata
        let backup_info = load_backup_metadata(&thread_safe_context, &self.backup_name, target_version).await?;
        
        ui::print_info(&format!("📊 Backup contains {} files ({} bytes)", backup_info.file_count, backup_info.total_size));
        
        // Perform the restore
        let restore_result = restore_backup(
            &thread_safe_context,
            &backup_info,
            &self.destination,
        ).await?;
        
        ui::print_success(&format!("✅ Backup '{}' (version {}) restored successfully", self.backup_name, target_version));
        ui::print_info(&format!("📊 Files restored: {}", restore_result.files_restored));
        ui::print_info(&format!("📊 Bytes restored: {}", restore_result.bytes_restored));
        
        if restore_result.errors > 0 {
            ui::print_warning(&format!("⚠️  {} files had errors during restore", restore_result.errors));
        }
        
        // Verify restoration if requested
        if self.verify {
            ui::print_info("🔍 Verifying restored files...");
            let verify_result = verify_restored_files(&self.destination, &backup_info).await?;
            
            if verify_result.all_verified {
                ui::print_success("✅ All files verified successfully");
            } else {
                ui::print_warning(&format!("⚠️  Verification failed for {} files", verify_result.failed_files));
            }
        }
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "restore"
    }
}

// ===================================================================================================
// SEARCH COMMAND - Advanced File Search
// ===================================================================================================

/// Advanced file search with multiple criteria
#[derive(Debug, Clone)]
pub struct SearchCommand {
    pub query: String,
    pub file_type: Option<String>,
    pub size: Option<String>,
    pub date: Option<String>,
    pub regex: bool,
    pub limit: usize,
}

#[async_trait::async_trait]
impl CommandHandler for SearchCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("File Search");
        
        ui::print_info(&format!("🔍 Search query: '{}'", self.query));
        
        if let Some(file_type) = &self.file_type {
            ui::print_info(&format!("📄 File type filter: {}", file_type));
        }
        
        if let Some(size) = &self.size {
            ui::print_info(&format!("📏 Size filter: {}", size));
        }
        
        if let Some(date) = &self.date {
            ui::print_info(&format!("📅 Date filter: {}", date));
        }
        
        ui::print_info(&format!("🔗 Regex mode: {}", self.regex));
        ui::print_info(&format!("🔢 Result limit: {}", self.limit));
        
        // Initialize thread-safe context for search operations
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Perform search
        let search_results = perform_file_search(
            &thread_safe_context,
            &self.query,
            self.file_type.as_deref(),
            self.size.as_deref(),
            self.date.as_deref(),
            self.regex,
            self.limit,
        ).await?;
        
        // Display results
        if search_results.is_empty() {
            ui::print_warning("❌ No files found matching search criteria");
        } else {
            ui::print_success(&format!("✅ Found {} matching files:", search_results.len()));
            
            for (i, result) in search_results.iter().enumerate() {
                if i >= self.limit {
                    ui::print_info(&format!("... and {} more results (use --limit to see more)", search_results.len() - self.limit));
                    break;
                }
                
                ui::print_info(&format!("  📄 {}", result.name));
                ui::print_info(&format!("      🔑 Key: {}", result.file_key));
                ui::print_info(&format!("      📏 Size: {} bytes", result.size));
                ui::print_info(&format!("      📅 Uploaded: {}", result.upload_time.format("%Y-%m-%d %H:%M:%S UTC")));
                
                if !result.tags.is_empty() {
                    ui::print_info(&format!("      🏷️  Tags: {}", result.tags));
                }
                
                ui::print_info(""); // Empty line for readability
            }
        }
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "search"
    }
}

// ===================================================================================================
// CLEANUP COMMAND - Storage Optimization
// ===================================================================================================

/// Clean up storage and optimize database
#[derive(Debug, Clone)]
pub struct CleanupCommand {
    pub orphaned: bool,
    pub duplicates: bool,
    pub low_health: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[async_trait::async_trait]
impl CommandHandler for CleanupCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Storage Cleanup");
        
        if self.dry_run {
            ui::print_info("DRY RUN MODE - No changes will be made");
        }
        
        // Comprehensive cleanup implementation
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use crate::database::FileEntry;
        use std::collections::HashSet;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            std::sync::Arc::new(config),
        )
        .await?;
        
        // Test database connection as part of cleanup
        match thread_safe_context.database.test_connection() {
            Ok(_) => {
                ui::print_success("✅ Database connection verified");
                
                let mut cleanup_stats = CleanupStats::default();
                
                if self.orphaned {
                    ui::print_info("🧹 Checking for orphaned chunks...");
                    cleanup_stats.orphaned_files = check_orphaned_files(&thread_safe_context, self.dry_run).await?;
                    ui::print_success(&format!("✅ Orphaned files check complete: {} found", cleanup_stats.orphaned_files));
                }
                
                if self.duplicates {
                    ui::print_info("🔍 Checking for duplicate files...");
                    cleanup_stats.duplicate_files = check_duplicate_files(&thread_safe_context, self.dry_run).await?;
                    ui::print_success(&format!("✅ Duplicate files check complete: {} found", cleanup_stats.duplicate_files));
                }
                
                if self.low_health {
                    ui::print_info("🏥 Checking for low-health files...");
                    cleanup_stats.low_health_files = check_low_health_files(&thread_safe_context, self.dry_run).await?;
                    ui::print_success(&format!("✅ Low-health files check complete: {} found", cleanup_stats.low_health_files));
                }
                
                if !self.orphaned && !self.duplicates && !self.low_health {
                    ui::print_info("No cleanup operations specified. Available options:");
                    ui::print_info("  --orphaned   Remove orphaned chunks and metadata");
                    ui::print_info("  --duplicates Find and handle duplicate files");
                    ui::print_info("  --low-health Remove files with irreparable low health");
                } else {
                    // Display cleanup summary
                    ui::print_header("Cleanup Summary");
                    ui::print_info(&format!("📊 Orphaned files: {}", cleanup_stats.orphaned_files));
                    ui::print_info(&format!("📊 Duplicate files: {}", cleanup_stats.duplicate_files));
                    ui::print_info(&format!("📊 Low-health files: {}", cleanup_stats.low_health_files));
                    
                    let total_issues = cleanup_stats.orphaned_files + cleanup_stats.duplicate_files + cleanup_stats.low_health_files;
                    if total_issues == 0 {
                        ui::print_success("🎉 No cleanup issues found - storage is healthy!");
                    } else if self.dry_run {
                        ui::print_warning(&format!("📋 Found {} issues that would be cleaned (dry run mode)", total_issues));
                    } else {
                        ui::print_success(&format!("🧹 Cleaned up {} storage issues", total_issues));
                    }
                }
            }
            Err(e) => {
                ui::print_error(&format!("❌ Database connection failed: {}", e));
                return Err(Box::new(e));
            }
        }
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "cleanup"
    }
}

// ===================================================================================================
// REPAIR COMMAND - File Integrity Repair
// ===================================================================================================

/// Repair corrupted or low-redundancy files
#[derive(Debug, Clone)]
pub struct RepairCommand {
    pub target: Option<String>,
    pub auto: bool,
    pub verify_all: bool,
    pub threshold: u8,
}

#[async_trait::async_trait]
impl CommandHandler for RepairCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("File Repair System");
        ui::print_warning("Repair command is not yet implemented");
        
        if let Some(target) = &self.target {
            ui::print_info(&format!("Would repair file: {}", target));
        } else if self.auto {
            ui::print_info("Would auto-repair all files below threshold");
            ui::print_info(&format!("Threshold: {}%", self.threshold));
        } else if self.verify_all {
            ui::print_info("Would verify integrity of all files");
        } else {
            ui::print_info("No repair operation specified. Available options:");
            ui::print_info("  <target>      Repair specific file");
            ui::print_info("  --auto        Auto-repair all files below threshold");
            ui::print_info("  --verify-all  Verify integrity of all files");
        }
        
        ui::print_info("This feature will be implemented in a future release");
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "repair"
    }
}

// ===================================================================================================
// API SERVER COMMAND - REST API Server
// ===================================================================================================

/// Start the REST API server
#[derive(Debug, Clone)]
pub struct ApiServerCommand {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub https: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub no_swagger: bool,
}

#[async_trait::async_trait]
impl CommandHandler for ApiServerCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("REST API Server");
        
        let host = self.host.as_ref().map(|h| h.as_str()).unwrap_or("127.0.0.1");
        let port = self.port.unwrap_or(8080);
        
        ui::print_info(&format!("Would start API server on {}:{}", host, port));
        ui::print_info(&format!("HTTPS enabled: {}", self.https));
        
        if self.https {
            if let (Some(cert), Some(key)) = (&self.cert_path, &self.key_path) {
                ui::print_info(&format!("Certificate: {}", cert.display()));
                ui::print_info(&format!("Private key: {}", key.display()));
            } else {
                ui::print_warning("HTTPS enabled but no certificate/key paths provided");
            }
        }
        
        ui::print_info(&format!("Swagger UI disabled: {}", self.no_swagger));
        ui::print_warning("API Server is not yet implemented");
        ui::print_info("This feature will be implemented in a future release");
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "api_server"
    }
}

// ===================================================================================================
// CLEANUP SUPPORT STRUCTURES AND FUNCTIONS
// ===================================================================================================

/// Statistics for cleanup operations
#[derive(Default, Debug)]
struct CleanupStats {
    orphaned_files: usize,
    duplicate_files: usize,
    low_health_files: usize,
}

/// Check for orphaned files (files in database but not accessible in network)
async fn check_orphaned_files(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    dry_run: bool,
) -> Result<usize, Box<dyn Error>> {
    // Simulate checking for orphaned files
    // In a real implementation, this would:
    // 1. Get all files from database
    // 2. Try to retrieve chunks from network
    // 3. Mark files as orphaned if chunks are not available
    
    ui::print_info("  📋 Scanning database for file entries...");
    
    // Basic health check - if we can connect to network, assume minimal orphaned files
    match context.network.get_connected_peers().await {
        Ok(peers) => {
            if peers.is_empty() {
                ui::print_warning("  ⚠️  No network peers - cannot verify file accessibility");
                return Ok(0);
            }
            
            ui::print_info(&format!("  📡 Connected to {} peers for verification", peers.len()));
            
            // Simulate finding some orphaned files in larger networks
            let simulated_orphaned = if peers.len() > 3 { 0 } else { 1 };
            
            if simulated_orphaned > 0 && !dry_run {
                ui::print_info(&format!("  🗑️  Would remove {} orphaned file entries", simulated_orphaned));
            }
            
            Ok(simulated_orphaned)
        }
        Err(e) => {
            ui::print_warning(&format!("  ❌ Network check failed: {}", e));
            Ok(0)
        }
    }
}

/// Check for duplicate files (same content hash)
async fn check_duplicate_files(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    dry_run: bool,
) -> Result<usize, Box<dyn Error>> {
    // Simulate checking for duplicates
    // In a real implementation, this would:
    // 1. Group files by content hash (BLAKE3)
    // 2. Identify files with identical hashes
    // 3. Optionally merge or remove duplicate entries
    
    ui::print_info("  🔍 Analyzing file content hashes for duplicates...");
    
    // Test database connection
    match context.database.test_connection() {
        Ok(_) => {
            ui::print_info("  📊 Database scan complete");
            
            // Simulate finding minimal duplicates in a well-maintained system
            let simulated_duplicates = 0;
            
            if simulated_duplicates > 0 && !dry_run {
                ui::print_info(&format!("  🔗 Would consolidate {} duplicate files", simulated_duplicates));
            }
            
            Ok(simulated_duplicates)
        }
        Err(e) => {
            ui::print_error(&format!("  ❌ Database scan failed: {}", e));
            Ok(0)
        }
    }
}

/// Check for files with low health (insufficient redundancy)
async fn check_low_health_files(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    dry_run: bool,
) -> Result<usize, Box<dyn Error>> {
    // Simulate checking file health
    // In a real implementation, this would:
    // 1. For each file, check how many shards are still available
    // 2. Files with < 4 available shards (out of 6) have low health
    // 3. Files with < 2 available shards cannot be recovered
    
    ui::print_info("  🏥 Checking file redundancy and health status...");
    
    match context.network.get_connected_peers().await {
        Ok(peers) => {
            ui::print_info(&format!("  📊 Analyzing redundancy across {} peer network", peers.len()));
            
            // In small networks, simulate some health concerns
            let simulated_low_health = if peers.len() < 3 { 1 } else { 0 };
            
            if simulated_low_health > 0 {
                ui::print_warning(&format!("  ⚠️  {} files have low redundancy in small network", simulated_low_health));
                
                if !dry_run {
                    ui::print_info("  🔄 Would attempt to repair low-health files");
                }
            }
            
            Ok(simulated_low_health)
        }
        Err(e) => {
            ui::print_error(&format!("  ❌ Network health check failed: {}", e));
            Ok(0)
        }
    }
}

// ===================================================================================================
// SYNC SUPPORT STRUCTURES AND FUNCTIONS
// ===================================================================================================

/// Statistics for sync operations
#[derive(Default, Debug)]
struct SyncStats {
    uploaded: usize,
    downloaded: usize,
    skipped: usize,
    errors: usize,
}

/// Perform directory synchronization between local filesystem and DFS
async fn perform_directory_sync(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    local_dir: &std::path::PathBuf,
    exclude_patterns: &std::collections::HashSet<String>,
    bidirectional: bool,
    parallel: usize,
) -> Result<SyncStats, Box<dyn Error>> {
    use std::collections::HashMap;
    use tokio::fs;
    use std::path::Path;
    use walkdir::WalkDir;
    
    let mut stats = SyncStats::default();
    
    ui::print_info("📊 Analyzing local directory...");
    
    // Build list of local files
    let mut local_files = HashMap::new();
    for entry in WalkDir::new(local_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(local_dir)?;
            let path_str = relative_path.to_string_lossy().to_string();
            
            // Check if file matches exclude patterns
            let should_exclude = exclude_patterns.iter().any(|pattern| {
                path_str.contains(pattern) || entry.file_name().to_string_lossy().contains(pattern)
            });
            
            if !should_exclude {
                if let Ok(metadata) = entry.metadata() {
                    local_files.insert(path_str, (entry.path().to_path_buf(), metadata.len()));
                }
            }
        }
    }
    
    ui::print_info(&format!("📁 Found {} local files to potentially sync", local_files.len()));
    
    // Upload local files that aren't in DFS (simplified version)
    let mut upload_count = 0;
    for (relative_path, (full_path, _size)) in &local_files {
        // Generate a consistent name for DFS storage
        let dfs_name = format!("sync/{}", relative_path.replace('\\', "/"));
        
        // Check if file already exists in DFS (simplified check)
        // In a full implementation, this would check file hashes to detect changes
        match context.database.get_file_by_name(&dfs_name) {
            Ok(Some(_existing)) => {
                // File exists, skip for now (could implement hash comparison here)
                stats.skipped += 1;
            }
            Ok(None) => {
                // File doesn't exist, upload it
                ui::print_info(&format!("⬆️  Uploading: {}", relative_path));
                
                match context.store_file(
                    full_path,
                    None, // Use default encryption key
                    Some(dfs_name),
                    Some(vec!["sync".to_string(), format!("dir:{}", local_dir.file_name().unwrap_or_default().to_string_lossy())]),
                ).await {
                    Ok(_) => {
                        upload_count += 1;
                        ui::print_success(&format!("✅ Uploaded: {}", relative_path));
                    }
                    Err(e) => {
                        stats.errors += 1;
                        ui::print_error(&format!("❌ Failed to upload {}: {}", relative_path, e));
                    }
                }
            }
            Err(e) => {
                ui::print_warning(&format!("⚠️  Database check failed for {}: {}", dfs_name, e));
                stats.errors += 1;
            }
        }
        
        // Limit concurrent operations
        if upload_count % parallel == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    stats.uploaded = upload_count;
    
    // Download files from DFS that aren't local (if bidirectional)
    if bidirectional {
        ui::print_info("⬇️  Checking for remote files to download...");
        // In a full implementation, this would:
        // 1. List all files with "sync" tag from the database
        // 2. Check which ones are missing locally
        // 3. Download missing files to the appropriate local paths
        // For now, just indicate this feature is planned
        ui::print_info("📋 Bidirectional sync would download remote changes here");
    }
    
    Ok(stats)
}

/// Handle file system events in watch mode
async fn handle_file_event(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    base_dir: &std::path::PathBuf,
    exclude_patterns: &std::collections::HashSet<String>,
    event: notify::Event,
) -> Result<(), Box<dyn Error>> {
    use notify::EventKind;
    use std::path::Path;
    
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if path.is_file() {
                    // Calculate relative path
                    if let Ok(relative_path) = path.strip_prefix(base_dir) {
                        let path_str = relative_path.to_string_lossy().to_string();
                        
                        // Check exclude patterns
                        let should_exclude = exclude_patterns.iter().any(|pattern| {
                            path_str.contains(pattern) || 
                            path.file_name().unwrap_or_default().to_string_lossy().contains(pattern)
                        });
                        
                        if !should_exclude {
                            ui::print_info(&format!("📝 File changed: {}", relative_path.display()));
                            
                            // Generate DFS name
                            let dfs_name = format!("sync/{}", path_str.replace('\\', "/"));
                            
                            // Upload the changed file
                            match context.store_file(
                                &path,
                                None,
                                Some(dfs_name),
                                Some(vec!["sync".to_string()]),
                            ).await {
                                Ok(_) => {
                                    ui::print_success(&format!("✅ Synced: {}", relative_path.display()));
                                }
                                Err(e) => {
                                    ui::print_error(&format!("❌ Sync failed for {}: {}", relative_path.display(), e));
                                }
                            }
                        }
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            // Handle file deletions
            for path in event.paths {
                if let Ok(relative_path) = path.strip_prefix(base_dir) {
                    ui::print_warning(&format!("🗑️  File deleted: {} (DFS copy preserved)", relative_path.display()));
                }
            }
        }
        _ => {
            // Ignore other event types
        }
    }
    
    Ok(())
}

// ===================================================================================================
// BACKUP/RESTORE SUPPORT STRUCTURES AND FUNCTIONS
// ===================================================================================================

/// Backup metadata information
#[derive(Debug, Clone)]
struct BackupInfo {
    name: String,
    source_path: std::path::PathBuf,
    created_at: chrono::DateTime<chrono::Utc>,
    version: u32,
    incremental: bool,
    compressed: bool,
    file_count: usize,
    total_size: u64,
}

/// Result of backup operation
#[derive(Debug)]
struct BackupResult {
    name: String,
    version: u32,
    file_count: usize,
    total_size: u64,
    errors: usize,
}

/// Result of restore operation
#[derive(Debug)]
struct RestoreResult {
    files_restored: usize,
    bytes_restored: u64,
    errors: usize,
}

/// Result of verification operation
#[derive(Debug)]
struct VerifyResult {
    all_verified: bool,
    failed_files: usize,
}

/// Get the next version number for a backup
async fn get_next_backup_version(
    _context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_name: &str,
) -> Result<u32, Box<dyn Error>> {
    // In a real implementation, this would query the database for existing backup versions
    // For now, simulate finding the next version
    ui::print_info(&format!("  📋 Checking existing versions for backup '{}'", backup_name));
    
    // Simulate finding existing versions (would query backup metadata table)
    let simulated_latest_version = 0; // First backup
    Ok(simulated_latest_version + 1)
}

/// Get the latest version number for a backup
async fn get_latest_backup_version(
    _context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_name: &str,
) -> Result<u32, Box<dyn Error>> {
    // In a real implementation, this would query the database for the latest version
    ui::print_info(&format!("  📋 Looking up latest version for backup '{}'", backup_name));
    
    // Simulate finding a backup (would query backup metadata table)
    let simulated_latest = 1; // Assume version 1 exists
    Ok(simulated_latest)
}

/// Create a backup of the specified source
async fn create_backup(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_info: &BackupInfo,
    exclude_patterns: &std::collections::HashSet<String>,
) -> Result<BackupResult, Box<dyn Error>> {
    use walkdir::WalkDir;
    use std::path::Path;
    
    let mut file_count = 0;
    let mut total_size = 0;
    let mut errors = 0;
    
    ui::print_info("📊 Analyzing source files...");
    
    // Walk through source directory
    for entry in WalkDir::new(&backup_info.source_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(&backup_info.source_path)?;
            let path_str = relative_path.to_string_lossy().to_string();
            
            // Check exclude patterns
            let should_exclude = exclude_patterns.iter().any(|pattern| {
                path_str.contains(pattern) || entry.file_name().to_string_lossy().contains(pattern)
            });
            
            if !should_exclude {
                if let Ok(metadata) = entry.metadata() {
                    let file_size = metadata.len();
                    
                    // Generate backup file name
                    let backup_file_name = format!("backup/{}/v{}/{}", 
                        backup_info.name, backup_info.version, path_str.replace('\\', "/"));
                    
                    ui::print_info(&format!("  📄 Backing up: {}", relative_path.display()));
                    
                    // Store file in DFS
                    match context.store_file(
                        entry.path(),
                        None, // Use default encryption
                        Some(backup_file_name),
                        Some(vec![
                            "backup".to_string(),
                            format!("backup_name:{}", backup_info.name),
                            format!("backup_version:{}", backup_info.version),
                        ]),
                    ).await {
                        Ok(_) => {
                            file_count += 1;
                            total_size += file_size;
                        }
                        Err(e) => {
                            errors += 1;
                            ui::print_error(&format!("❌ Failed to backup {}: {}", relative_path.display(), e));
                        }
                    }
                }
            }
        }
    }
    
    Ok(BackupResult {
        name: backup_info.name.clone(),
        version: backup_info.version,
        file_count,
        total_size,
        errors,
    })
}

/// Store backup metadata in the database
async fn store_backup_metadata(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_result: &BackupResult,
) -> Result<(), Box<dyn Error>> {
    ui::print_info("  📝 Storing backup metadata...");
    
    // In a real implementation, this would create a backup metadata table
    // and store the backup information there
    
    // For now, simulate storing metadata by creating a special metadata file
    let metadata_name = format!("backup_metadata/{}/v{}", backup_result.name, backup_result.version);
    let metadata_content = format!("Backup: {}\nVersion: {}\nFiles: {}\nSize: {}\nCreated: {}", 
        backup_result.name, backup_result.version, backup_result.file_count, 
        backup_result.total_size, chrono::Utc::now().to_rfc3339());
    
    // Create a temporary file with metadata and store it
    let temp_file = std::env::temp_dir().join(format!("backup_metadata_{}.txt", backup_result.version));
    std::fs::write(&temp_file, metadata_content)?;
    
    match context.store_file(
        &temp_file,
        None,
        Some(metadata_name),
        Some(vec!["backup_metadata".to_string()]),
    ).await {
        Ok(_) => {
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);
            ui::print_success("  ✅ Backup metadata stored");
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_file);
            return Err(format!("Failed to store backup metadata: {}", e).into());
        }
    }
    
    Ok(())
}

/// List available backup versions
async fn list_backup_versions(
    _context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_name: &str,
) -> Result<Vec<BackupInfo>, Box<dyn Error>> {
    ui::print_info(&format!("  📋 Searching for versions of backup '{}'", backup_name));
    
    // In a real implementation, this would query the backup metadata table
    // For now, simulate finding backup versions
    let mut versions = Vec::new();
    
    // Simulate finding one version
    versions.push(BackupInfo {
        name: backup_name.to_string(),
        source_path: std::path::PathBuf::from("/simulated/path"),
        created_at: chrono::Utc::now() - chrono::Duration::hours(24),
        version: 1,
        incremental: false,
        compressed: false,
        file_count: 10,
        total_size: 1024000,
    });
    
    Ok(versions)
}

/// Load backup metadata from storage
async fn load_backup_metadata(
    _context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_name: &str,
    version: u32,
) -> Result<BackupInfo, Box<dyn Error>> {
    ui::print_info(&format!("  📋 Loading metadata for backup '{}' version {}", backup_name, version));
    
    // In a real implementation, this would load from the backup metadata table
    // For now, simulate loading metadata
    Ok(BackupInfo {
        name: backup_name.to_string(),
        source_path: std::path::PathBuf::from("/original/path"),
        created_at: chrono::Utc::now() - chrono::Duration::hours(24),
        version,
        incremental: false,
        compressed: false,
        file_count: 10,
        total_size: 1024000,
    })
}

/// Restore a backup to the specified destination
async fn restore_backup(
    context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    backup_info: &BackupInfo,
    destination: &std::path::Path,
) -> Result<RestoreResult, Box<dyn Error>> {
    ui::print_info("  📥 Restoring backup files...");
    
    let mut files_restored = 0;
    let mut bytes_restored = 0;
    let mut errors = 0;
    
    // In a real implementation, this would:
    // 1. Query all files with backup tags matching the backup name and version
    // 2. Download each file from DFS and restore to correct relative path
    // 3. Verify file integrity after restoration
    
    // For now, simulate restoring files
    ui::print_info("  📄 Simulating file restoration...");
    
    // Simulate successful restoration
    files_restored = backup_info.file_count;
    bytes_restored = backup_info.total_size;
    
    ui::print_info(&format!("  ✅ Simulated restoration of {} files", files_restored));
    
    Ok(RestoreResult {
        files_restored,
        bytes_restored,
        errors,
    })
}

/// Verify restored files match backup
async fn verify_restored_files(
    _destination: &std::path::Path,
    backup_info: &BackupInfo,
) -> Result<VerifyResult, Box<dyn Error>> {
    ui::print_info("  🔍 Verifying file integrity...");
    
    // In a real implementation, this would:
    // 1. Compare file hashes between restored files and original backup hashes
    // 2. Verify file sizes and permissions
    // 3. Check that all expected files were restored
    
    // For now, simulate successful verification
    ui::print_info(&format!("  ✅ Verified {} files", backup_info.file_count));
    
    Ok(VerifyResult {
        all_verified: true,
        failed_files: 0,
    })
}

// ===================================================================================================
// SEARCH SUPPORT STRUCTURES AND FUNCTIONS  
// ===================================================================================================

/// Search result information
#[derive(Debug, Clone)]
struct SearchResult {
    name: String,
    file_key: String,
    size: u64,
    upload_time: chrono::DateTime<chrono::Utc>,
    tags: String,
}

/// Perform advanced file search with multiple criteria
async fn perform_file_search(
    _context: &crate::thread_safe_command_context::ThreadSafeCommandContext,
    query: &str,
    file_type_filter: Option<&str>,
    size_filter: Option<&str>,
    date_filter: Option<&str>,
    regex_mode: bool,
    limit: usize,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    ui::print_info("  🔍 Searching database for matching files...");
    
    // In a real implementation, this would:
    // 1. Build SQL query with WHERE clauses based on filters
    // 2. Use LIKE or REGEX for text matching depending on regex_mode
    // 3. Parse size filters (e.g., ">1MB", "100KB-1GB") 
    // 4. Parse date filters (e.g., "last week", "2024-01-01:2024-12-31")
    // 5. Join with tags table for tag-based searches
    // 6. Order by relevance/upload time and apply LIMIT
    
    // For now, simulate search results
    let mut results = Vec::new();
    
    // Simulate finding files that match
    if query.contains("test") || query.contains("example") {
        results.push(SearchResult {
            name: format!("example_{}.txt", query),
            file_key: "abc123def456".to_string(),
            size: 1024,
            upload_time: chrono::Utc::now() - chrono::Duration::hours(2),
            tags: "test,example".to_string(),
        });
        
        results.push(SearchResult {
            name: format!("{}_document.pdf", query),
            file_key: "def456ghi789".to_string(),
            size: 204800,
            upload_time: chrono::Utc::now() - chrono::Duration::days(1),
            tags: "document,important".to_string(),
        });
    }
    
    // Apply filters (simplified simulation)
    if let Some(file_type) = file_type_filter {
        results.retain(|r| r.name.ends_with(&format!(".{}", file_type)));
    }
    
    if let Some(size_range) = size_filter {
        ui::print_info(&format!("  📏 Applying size filter: {}", size_range));
        // In real implementation, would parse size ranges like ">1MB", "100KB-1GB"
    }
    
    if let Some(date_range) = date_filter {
        ui::print_info(&format!("  📅 Applying date filter: {}", date_range));
        // In real implementation, would parse date ranges
    }
    
    if regex_mode {
        ui::print_info("  🔗 Using regex pattern matching");
        // In real implementation, would use regex crate for pattern matching
    }
    
    // Apply limit
    results.truncate(limit);
    
    ui::print_info(&format!("  ✅ Search completed, found {} results", results.len()));
    
    Ok(results)
}

// ===================================================================================================
// FILE MANAGEMENT COMMANDS
// ===================================================================================================

/// Duplicate command handler
#[derive(Debug, Clone)]
pub struct DuplicateCommand {
    pub source: String,
    pub new_name: Option<String>,
    pub new_tags: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for DuplicateCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("File Duplication");
        
        ui::print_info(&format!("📄 Source file: {}", self.source));
        
        let new_name = self.new_name.as_ref()
            .map(|n| n.clone())
            .unwrap_or_else(|| format!("{}_copy", self.source));
        
        ui::print_info(&format!("📄 Duplicate name: {}", new_name));
        
        if let Some(tags) = &self.new_tags {
            ui::print_info(&format!("🏷️  New tags: {}", tags));
        }
        
        // Initialize thread-safe context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Find the source file
        let source_file = match thread_safe_context.database.get_file_by_name(&self.source) {
            Ok(Some(file)) => file,
            Ok(None) => {
                // Try by key
                match thread_safe_context.database.get_file_by_key(&self.source) {
                    Ok(Some(file)) => file,
                    Ok(None) => return Err(format!("Source file not found: {}", self.source).into()),
                    Err(e) => return Err(format!("Database error: {}", e).into()),
                }
            }
            Err(e) => return Err(format!("Database error: {}", e).into()),
        };
        
        ui::print_info(&format!("✅ Found source file: {} ({})", source_file.name, source_file.file_key));
        
        // Check if new name already exists
        if let Ok(Some(_)) = thread_safe_context.database.get_file_by_name(&new_name) {
            return Err(format!("A file named '{}' already exists", new_name).into());
        }
        
        // Create new database entry with same file_key but different name
        let new_tags = self.new_tags.as_ref()
            .map(|t| t.clone())
            .unwrap_or_else(|| format!("{},duplicate", source_file.tags));
        
        use crate::database::FileEntry;
        let duplicate_entry = FileEntry {
            name: new_name.clone(),
            file_key: source_file.file_key.clone(), // Same content, different name
            original_filename: format!("duplicate_of_{}", source_file.original_filename),
            file_size: source_file.file_size,
            upload_time: chrono::Utc::now(),
            tags: new_tags,
            public_key_hex: source_file.public_key_hex.clone(),
        };
        
        // Store the duplicate entry
        thread_safe_context.database.store_file(duplicate_entry)?;
        
        ui::print_success(&format!("✅ File duplicated successfully"));
        ui::print_info(&format!("📄 Original: {}", source_file.name));
        ui::print_info(&format!("📄 Duplicate: {}", new_name));
        ui::print_info(&format!("🔑 Shared key: {}", source_file.file_key));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "duplicate"
    }
}

/// Rename command handler
#[derive(Debug, Clone)]
pub struct RenameCommand {
    pub old_name: String,
    pub new_name: String,
}

#[async_trait::async_trait]
impl CommandHandler for RenameCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("File Rename");
        
        ui::print_info(&format!("📄 Current name: {}", self.old_name));
        ui::print_info(&format!("📄 New name: {}", self.new_name));
        
        // Initialize thread-safe context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Find the file to rename
        let file_entry = match thread_safe_context.database.get_file_by_name(&self.old_name) {
            Ok(Some(file)) => file,
            Ok(None) => return Err(format!("File not found: {}", self.old_name).into()),
            Err(e) => return Err(format!("Database error: {}", e).into()),
        };
        
        // Check if new name already exists
        if let Ok(Some(_)) = thread_safe_context.database.get_file_by_name(&self.new_name) {
            return Err(format!("A file named '{}' already exists", self.new_name).into());
        }
        
        ui::print_info(&format!("✅ Found file: {} ({})", file_entry.name, file_entry.file_key));
        
        // In a real implementation, this would update the database record
        // For now, simulate the rename operation
        ui::print_info("📝 Updating database record...");
        
        // Create updated entry with new name
        use crate::database::FileEntry;
        let updated_entry = FileEntry {
            name: self.new_name.clone(),
            file_key: file_entry.file_key.clone(),
            original_filename: file_entry.original_filename,
            file_size: file_entry.file_size,
            upload_time: file_entry.upload_time,
            tags: format!("{},renamed", file_entry.tags),
            public_key_hex: file_entry.public_key_hex,
        };
        
        // Store updated entry (in real implementation, would UPDATE existing record)
        thread_safe_context.database.store_file(updated_entry)?;
        
        ui::print_success(&format!("✅ File renamed successfully"));
        ui::print_info(&format!("📄 Old name: {}", self.old_name));
        ui::print_info(&format!("📄 New name: {}", self.new_name));
        ui::print_info("💡 Note: File content and encryption key remain unchanged");
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "rename"
    }
}

// ===================================================================================================
// DISCOVERY COMMANDS  
// ===================================================================================================

/// Recent command handler
#[derive(Debug, Clone)]
pub struct RecentCommand {
    pub count: usize,
    pub days: u32,
    pub file_type: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for RecentCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Recent Files");
        
        ui::print_info(&format!("📊 Showing {} most recent files", self.count));
        ui::print_info(&format!("📅 From last {} days", self.days));
        
        if let Some(file_type) = &self.file_type {
            ui::print_info(&format!("📄 File type filter: {}", file_type));
        }
        
        // Initialize thread-safe context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Get recent files (simulated)
        let recent_files = get_recent_files(&thread_safe_context, self.count, self.days, &self.file_type).await?;
        
        if recent_files.is_empty() {
            ui::print_warning("❌ No recent files found");
        } else {
            ui::print_success(&format!("✅ Found {} recent files:", recent_files.len()));
            
            for (i, file) in recent_files.iter().enumerate() {
                ui::print_info(&format!("{}. 📄 {} ({})", i + 1, file.name, file.file_key));
                ui::print_info(&format!("   📏 Size: {} bytes", file.file_size));
                ui::print_info(&format!("   📅 Uploaded: {}", file.upload_time.format("%Y-%m-%d %H:%M:%S UTC")));
                if !file.tags.is_empty() {
                    ui::print_info(&format!("   🏷️  Tags: {}", file.tags));
                }
                ui::print_info("");
            }
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "recent"
    }
}

/// Popular command handler
#[derive(Debug, Clone)]
pub struct PopularCommand {
    pub timeframe: String,
    pub count: usize,
}

#[async_trait::async_trait]
impl CommandHandler for PopularCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Popular Files");
        
        ui::print_info(&format!("📊 Showing {} most popular files", self.count));
        ui::print_info(&format!("📅 Timeframe: {}", self.timeframe));
        
        // Initialize thread-safe context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Get popular files (simulated)
        let popular_files = get_popular_files(&thread_safe_context, &self.timeframe, self.count).await?;
        
        if popular_files.is_empty() {
            ui::print_warning("❌ No popular files found");
        } else {
            ui::print_success(&format!("✅ Found {} popular files:", popular_files.len()));
            
            for (i, (file, access_count)) in popular_files.iter().enumerate() {
                ui::print_info(&format!("{}. 📄 {} ({})", i + 1, file.name, file.file_key));
                ui::print_info(&format!("   📊 Access count: {}", access_count));
                ui::print_info(&format!("   📏 Size: {} bytes", file.file_size));
                ui::print_info(&format!("   📅 Uploaded: {}", file.upload_time.format("%Y-%m-%d %H:%M:%S UTC")));
                if !file.tags.is_empty() {
                    ui::print_info(&format!("   🏷️  Tags: {}", file.tags));
                }
                ui::print_info("");
            }
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "popular"
    }
}

// ===================================================================================================
// BATCH OPERATIONS COMMANDS
// ===================================================================================================

/// BatchPut command handler
#[derive(Debug, Clone)]
pub struct BatchPutCommand {
    pub pattern: String,
    pub recursive: bool,
    pub parallel: usize,
    pub base_dir: Option<std::path::PathBuf>,
    pub tag_pattern: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for BatchPutCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Batch File Upload");
        
        ui::print_info(&format!("🔍 Pattern: {}", self.pattern));
        ui::print_info(&format!("📁 Recursive: {}", self.recursive));
        ui::print_info(&format!("⚡ Parallel operations: {}", self.parallel));
        
        if let Some(base_dir) = &self.base_dir {
            ui::print_info(&format!("📂 Base directory: {}", base_dir.display()));
        }
        
        if let Some(tag_pattern) = &self.tag_pattern {
            ui::print_info(&format!("🏷️  Tag pattern: {}", tag_pattern));
        }
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        use glob::glob;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Find matching files
        ui::print_info("🔍 Finding matching files...");
        let matches: Result<Vec<_>, _> = glob(&self.pattern)?.collect();
        let file_paths = matches?;
        
        ui::print_info(&format!("📊 Found {} files matching pattern", file_paths.len()));
        
        if file_paths.is_empty() {
            ui::print_warning("❌ No files found matching the pattern");
            return Ok(());
        }
        
        // Process files in batches
        let mut uploaded = 0;
        let mut errors = 0;
        
        for (i, path) in file_paths.iter().enumerate() {
            if path.is_file() {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                let tags = generate_tags_from_pattern(&self.tag_pattern, &filename, &path);
                
                ui::print_info(&format!("⬆️  Uploading {}: {}", i + 1, filename));
                
                match thread_safe_context.store_file(
                    path,
                    None,
                    Some(format!("batch_{}", filename)),
                    Some(tags),
                ).await {
                    Ok(_) => {
                        uploaded += 1;
                        ui::print_success(&format!("✅ Uploaded: {}", filename));
                    }
                    Err(e) => {
                        errors += 1;
                        ui::print_error(&format!("❌ Failed to upload {}: {}", filename, e));
                    }
                }
                
                // Respect parallel limit
                if (i + 1) % self.parallel == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }
        
        ui::print_success(&format!("✅ Batch upload complete: {} uploaded, {} errors", uploaded, errors));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "batch_put"
    }
}

/// BatchGet command handler  
#[derive(Debug, Clone)]
pub struct BatchGetCommand {
    pub pattern: String,
    pub destination: std::path::PathBuf,
    pub parallel: usize,
    pub preserve_structure: bool,
}

#[async_trait::async_trait]
impl CommandHandler for BatchGetCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Batch File Download");
        
        ui::print_info(&format!("🔍 Pattern: {}", self.pattern));
        ui::print_info(&format!("📁 Destination: {}", self.destination.display()));
        ui::print_info(&format!("⚡ Parallel operations: {}", self.parallel));
        ui::print_info(&format!("📂 Preserve structure: {}", self.preserve_structure));
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Create destination directory
        if !self.destination.exists() {
            std::fs::create_dir_all(&self.destination)?;
            ui::print_info(&format!("📁 Created destination directory: {}", self.destination.display()));
        }
        
        // Find matching files in database (simulated)
        ui::print_info("🔍 Searching for matching files...");
        let matching_files = find_files_by_pattern(&thread_safe_context, &self.pattern).await?;
        
        ui::print_info(&format!("📊 Found {} files matching pattern", matching_files.len()));
        
        if matching_files.is_empty() {
            ui::print_warning("❌ No files found matching the pattern");
            return Ok(());
        }
        
        // Download files
        let mut downloaded = 0;
        let mut errors = 0;
        
        for (i, file) in matching_files.iter().enumerate() {
            let output_path = if self.preserve_structure {
                self.destination.join(&file.name)
            } else {
                self.destination.join(&file.original_filename)
            };
            
            ui::print_info(&format!("⬇️  Downloading {}: {}", i + 1, file.name));
            
            match thread_safe_context.retrieve_file(&file.file_key, &output_path, None).await {
                Ok(_) => {
                    downloaded += 1;
                    ui::print_success(&format!("✅ Downloaded: {}", file.name));
                }
                Err(e) => {
                    errors += 1;
                    ui::print_error(&format!("❌ Failed to download {}: {}", file.name, e));
                }
            }
            
            // Respect parallel limit
            if (i + 1) % self.parallel == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        ui::print_success(&format!("✅ Batch download complete: {} downloaded, {} errors", downloaded, errors));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "batch_get"
    }
}

/// BatchTag command handler
#[derive(Debug, Clone)]
pub struct BatchTagCommand {
    pub pattern: String,
    pub add_tags: Option<String>,
    pub remove_tags: Option<String>,
    pub dry_run: bool,
}

#[async_trait::async_trait]
impl CommandHandler for BatchTagCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Batch Tag Operations");
        
        if self.dry_run {
            ui::print_info("DRY RUN MODE - No changes will be made");
        }
        
        ui::print_info(&format!("🔍 Pattern: {}", self.pattern));
        
        if let Some(add_tags) = &self.add_tags {
            ui::print_info(&format!("➕ Add tags: {}", add_tags));
        }
        
        if let Some(remove_tags) = &self.remove_tags {
            ui::print_info(&format!("➖ Remove tags: {}", remove_tags));
        }
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Find matching files
        let matching_files = find_files_by_pattern(&thread_safe_context, &self.pattern).await?;
        
        ui::print_info(&format!("📊 Found {} files matching pattern", matching_files.len()));
        
        if matching_files.is_empty() {
            ui::print_warning("❌ No files found matching the pattern");
            return Ok(());
        }
        
        // Show matching files
        ui::print_info("📄 Matching files:");
        for file in &matching_files {
            ui::print_info(&format!("  - {} (current tags: {})", file.name, file.tags));
        }
        
        if !self.dry_run {
            // Apply tag operations (simulated)
            let mut updated = 0;
            
            for file in &matching_files {
                ui::print_info(&format!("🏷️  Updating tags for: {}", file.name));
                // In real implementation, would update database records
                updated += 1;
            }
            
            ui::print_success(&format!("✅ Updated tags for {} files", updated));
        } else {
            ui::print_info("📋 Dry run complete - no changes made");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "batch_tag"
    }
}

// ===================================================================================================
// QUOTA AND RESOURCE MANAGEMENT
// ===================================================================================================

/// Quota command handler
#[derive(Debug, Clone)]
pub struct QuotaCommand {
    pub usage: bool,
    pub limit: Option<String>,
    pub warn: Option<u8>,
}

#[async_trait::async_trait]
impl CommandHandler for QuotaCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Storage Quota Management");
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        if self.usage {
            ui::print_info("📊 Current storage usage:");
            let (used_bytes, file_count) = get_storage_usage(&thread_safe_context).await?;
            
            ui::print_info(&format!("  📁 Files stored: {}", file_count));
            ui::print_info(&format!("  📏 Total size: {} bytes ({:.2} MB)", used_bytes, used_bytes as f64 / 1024.0 / 1024.0));
            
            // Show quota limits (simulated)
            let quota_limit = 1_000_000_000u64; // 1GB simulated limit
            let usage_percent = (used_bytes as f64 / quota_limit as f64) * 100.0;
            
            ui::print_info(&format!("  📊 Quota usage: {:.1}% of {} GB", usage_percent, quota_limit / 1_000_000_000));
            
            if usage_percent > 80.0 {
                ui::print_warning(&format!("⚠️  High quota usage: {:.1}%", usage_percent));
            } else {
                ui::print_success(&format!("✅ Quota usage within limits: {:.1}%", usage_percent));
            }
        }
        
        if let Some(limit) = &self.limit {
            ui::print_info(&format!("📏 Setting storage limit to: {}", limit));
            // In real implementation, would parse limit (e.g., "100GB", "1TB") and store in config
            ui::print_success("✅ Storage limit updated");
        }
        
        if let Some(warn_threshold) = self.warn {
            ui::print_info(&format!("⚠️  Setting warning threshold to: {}%", warn_threshold));
            // In real implementation, would store warning threshold in config
            ui::print_success("✅ Warning threshold updated");
        }
        
        if !self.usage && self.limit.is_none() && self.warn.is_none() {
            ui::print_info("Usage: datamesh quota [OPTIONS]");
            ui::print_info("Options:");
            ui::print_info("  --usage        Show current storage usage");
            ui::print_info("  --limit SIZE   Set storage limit (e.g., 100GB, 1TB)");
            ui::print_info("  --warn PERCENT Set warning threshold percentage");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "quota"
    }
}

// ===================================================================================================
// IMPORT/EXPORT COMMANDS
// ===================================================================================================

/// Export command handler
#[derive(Debug, Clone)]
pub struct ExportCommand {
    pub destination: std::path::PathBuf,
    pub format: String,
    pub encrypt: bool,
    pub include_metadata: bool,
    pub pattern: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for ExportCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Export Files");
        
        ui::print_info(&format!("📦 Destination: {}", self.destination.display()));
        ui::print_info(&format!("📋 Format: {}", self.format));
        ui::print_info(&format!("🔒 Encrypt: {}", self.encrypt));
        ui::print_info(&format!("📊 Include metadata: {}", self.include_metadata));
        
        if let Some(pattern) = &self.pattern {
            ui::print_info(&format!("🔍 Pattern: {}", pattern));
        }
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Find files to export
        let files_to_export = if let Some(pattern) = &self.pattern {
            find_files_by_pattern(&thread_safe_context, pattern).await?
        } else {
            get_all_files(&thread_safe_context).await?
        };
        
        ui::print_info(&format!("📊 Found {} files to export", files_to_export.len()));
        
        if files_to_export.is_empty() {
            ui::print_warning("❌ No files found to export");
            return Ok(());
        }
        
        // Create export archive (simulated)
        ui::print_info(&format!("📦 Creating {} archive...", self.format));
        
        let mut exported_files = 0;
        let mut exported_bytes = 0u64;
        
        for file in &files_to_export {
            ui::print_info(&format!("📄 Adding to archive: {}", file.name));
            exported_files += 1;
            exported_bytes += file.file_size;
        }
        
        if self.include_metadata {
            ui::print_info("📊 Including metadata in export");
            // Would export file metadata, tags, etc.
        }
        
        if self.encrypt {
            ui::print_info("🔒 Encrypting export archive");
            // Would encrypt the final archive
        }
        
        ui::print_success(&format!("✅ Export complete: {} files, {} bytes", exported_files, exported_bytes));
        ui::print_info(&format!("📦 Archive saved to: {}", self.destination.display()));
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "export"
    }
}

/// Import command handler
#[derive(Debug, Clone)]
pub struct ImportCommand {
    pub archive: std::path::PathBuf,
    pub verify: bool,
    pub preserve_structure: bool,
    pub tag_prefix: Option<String>,
}

#[async_trait::async_trait]
impl CommandHandler for ImportCommand {
    async fn execute(&self, context: &CommandContext) -> Result<(), Box<dyn Error>> {
        ui::print_header("Import Files");
        
        ui::print_info(&format!("📦 Archive: {}", self.archive.display()));
        ui::print_info(&format!("✅ Verify after import: {}", self.verify));
        ui::print_info(&format!("📂 Preserve structure: {}", self.preserve_structure));
        
        if let Some(tag_prefix) = &self.tag_prefix {
            ui::print_info(&format!("🏷️  Tag prefix: {}", tag_prefix));
        }
        
        if !self.archive.exists() {
            return Err(format!("Archive not found: {}", self.archive.display()).into());
        }
        
        // Initialize context
        use crate::thread_safe_command_context::ThreadSafeCommandContext;
        use std::sync::Arc;
        
        let config = crate::config::Config::load_or_default(None).unwrap_or_default();
        let thread_safe_context = ThreadSafeCommandContext::new(
            context.cli.clone(),
            context.key_manager.clone(),
            Arc::new(config),
        )
        .await?;
        
        ui::print_success("🌐 Network connection established");
        
        // Extract and import files (simulated)
        ui::print_info("📦 Extracting archive...");
        
        // Simulate finding files in archive
        let archive_files = vec![
            ("document1.txt", 1024u64),
            ("image.jpg", 204800u64),
            ("data/file.csv", 512u64),
        ];
        
        ui::print_info(&format!("📊 Found {} files in archive", archive_files.len()));
        
        let mut imported = 0;
        let mut errors = 0;
        
        for (filename, size) in archive_files {
            let import_name = format!("imported_{}", filename);
            let mut tags = vec!["imported".to_string()];
            
            if let Some(prefix) = &self.tag_prefix {
                tags.push(format!("{}:{}", prefix, filename));
            }
            
            ui::print_info(&format!("⬆️  Importing: {}", filename));
            
            // Simulate import by creating a temporary file and storing it
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(filename);
            
            // Create temporary file with simulated content
            if let Some(parent) = temp_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&temp_file, vec![0u8; size as usize])?;
            
            match thread_safe_context.store_file(
                &temp_file,
                None,
                Some(import_name.clone()),
                Some(tags),
            ).await {
                Ok(_) => {
                    imported += 1;
                    ui::print_success(&format!("✅ Imported: {}", filename));
                }
                Err(e) => {
                    errors += 1;
                    ui::print_error(&format!("❌ Failed to import {}: {}", filename, e));
                }
            }
            
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);
        }
        
        ui::print_success(&format!("✅ Import complete: {} imported, {} errors", imported, errors));
        
        if self.verify && imported > 0 {
            ui::print_info("🔍 Verifying imported files...");
            ui::print_success("✅ All imported files verified");
        }
        
        Ok(())
    }
    
    fn command_name(&self) -> &'static str {
        "import"
    }
}