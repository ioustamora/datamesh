use crate::cli::Cli;
use crate::database::{DatabaseManager, FileEntry};
use crate::file_storage;
use crate::key_manager::KeyManager;
use crate::ui;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Local};
use std::fs;
/// File Management Module
///
/// This module provides enhanced file management capabilities including:
/// - Synchronization with local directories
/// - Backup and restore operations
/// - File duplication and renaming
/// - Advanced file search and discovery
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// File synchronization options
#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub watch: bool,
    pub bidirectional: bool,
    pub exclude_patterns: Vec<String>,
    pub parallel: usize,
}

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub name: String,
    pub incremental: bool,
    pub compress: bool,
    pub schedule: Option<String>,
    pub exclude_patterns: Vec<String>,
}

/// Search criteria for file discovery
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub query: String,
    pub file_type: Option<String>,
    pub size_range: Option<SizeRange>,
    pub date_range: Option<DateRange>,
    pub use_regex: bool,
    pub limit: usize,
}

/// Size range filter
#[derive(Debug, Clone)]
pub enum SizeRange {
    LessThan(u64),
    GreaterThan(u64),
    Between(u64, u64),
}

/// Date range filter
#[derive(Debug, Clone)]
pub enum DateRange {
    LastDays(u32),
    LastWeeks(u32),
    LastMonths(u32),
    Between(DateTime<Local>, DateTime<Local>),
}

/// Synchronize a local directory with the DFS network
pub async fn sync_directory(
    cli: &Cli,
    key_manager: &KeyManager,
    local_dir: &Path,
    options: SyncOptions,
) -> Result<()> {
    ui::print_header(&format!("Synchronizing {}", local_dir.display()));

    if !local_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory does not exist: {}",
            local_dir.display()
        ));
    }

    let spinner = ui::create_spinner("Analyzing directory structure...");

    // Get list of local files
    let local_files = scan_directory(local_dir, &options.exclude_patterns)?;
    spinner.finish_with_message("Directory scanned");

    // Get list of remote files
    ui::print_operation_status("Remote Files", "Scanning", None);
    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    let remote_files = db.list_files(None)?;

    // Compare and determine actions
    let sync_plan = create_sync_plan(&local_files, &remote_files, options.bidirectional)?;

    ui::print_section("Synchronization Plan");
    ui::print_key_value("Files to upload", &sync_plan.uploads.len().to_string());
    ui::print_key_value("Files to download", &sync_plan.downloads.len().to_string());
    ui::print_key_value("Files to update", &sync_plan.updates.len().to_string());

    if ui::confirm_action("Proceed with synchronization?", true) {
        execute_sync_plan(cli, key_manager, &sync_plan, &options).await?;
        ui::print_success("Synchronization completed successfully");
    }

    if options.watch {
        ui::print_info("Starting file watcher (Ctrl+C to stop)...");
        start_file_watcher(cli, key_manager, local_dir, options).await?;
    }

    Ok(())
}

/// Create a backup of files or directories
pub async fn create_backup(
    cli: &Cli,
    key_manager: &KeyManager,
    source: &Path,
    config: BackupConfig,
) -> Result<()> {
    ui::print_header(&format!("Creating Backup: {}", config.name));

    let backup_tag = format!("backup:{}", config.name);
    let timestamp_tag = format!("backup-date:{}", Local::now().format("%Y-%m-%d"));

    if source.is_file() {
        // Single file backup
        let tags = format!("{},{},single-file", backup_tag, timestamp_tag);
        let file_name = format!(
            "{}-{}",
            config.name,
            source.file_name().unwrap().to_string_lossy()
        );

        file_storage::handle_put_command(
            cli,
            key_manager,
            &source.to_path_buf(),
            &None,
            &Some(file_name),
            &Some(tags),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    } else if source.is_dir() {
        // Directory backup
        let files = scan_directory(source, &config.exclude_patterns)?;

        ui::print_key_value("Files to backup", &files.len().to_string());
        ui::print_key_value(
            "Backup type",
            if config.incremental {
                "Incremental"
            } else {
                "Full"
            },
        );

        if config.incremental {
            // Check for previous backup
            let db_path = crate::database::get_default_db_path()?;
            let db = DatabaseManager::new(&db_path)?;
            let previous_files = db.list_files_by_tag(&backup_tag)?;

            ui::print_key_value("Previous backup files", &previous_files.len().to_string());
        }

        let mut progress = ui::MultiOperationProgress::new();
        let backup_op = progress.add_operation("Backup Progress", files.len() as u64);

        for (i, file_path) in files.iter().enumerate() {
            let relative_path = file_path
                .strip_prefix(source)
                .unwrap_or(file_path)
                .to_string_lossy();

            let file_tags = format!("{},{},path:{}", backup_tag, timestamp_tag, relative_path);
            let file_name = format!("{}-{}", config.name, relative_path.replace('/', "-"));

            progress.update_operation(
                backup_op,
                i as u64,
                &format!("Backing up {}", relative_path),
            );

            if let Err(e) = file_storage::handle_put_command(
                cli,
                key_manager,
                file_path,
                &None,
                &Some(file_name),
                &Some(file_tags),
            )
            .await
            {
                ui::print_error(&format!("Failed to backup {}: {}", relative_path, e));
            }
        }

        progress.finish_operation(backup_op, "Backup completed");
        progress.clear();
    }

    ui::print_success(&format!("Backup '{}' created successfully", config.name));
    Ok(())
}

/// Restore files from a backup
pub async fn restore_backup(
    cli: &Cli,
    key_manager: &KeyManager,
    backup_name: &str,
    destination: &Path,
    _version: Option<u32>,
    verify: bool,
) -> Result<()> {
    ui::print_header(&format!("Restoring Backup: {}", backup_name));

    let backup_tag = format!("backup:{}", backup_name);

    // Get backup files from database
    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    let backup_files = db.list_files_by_tag(&backup_tag)?;

    if backup_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No backup found with name: {}",
            backup_name
        ));
    }

    ui::print_key_value("Backup files found", &backup_files.len().to_string());

    // Create destination directory
    fs::create_dir_all(destination).context("Failed to create destination directory")?;

    let mut progress = ui::MultiOperationProgress::new();
    let restore_op = progress.add_operation("Restore Progress", backup_files.len() as u64);

    for (i, file_entry) in backup_files.iter().enumerate() {
        progress.update_operation(
            restore_op,
            i as u64,
            &format!("Restoring {}", file_entry.name),
        );

        // Extract original path from tags
        let output_path =
            if let Some(path_tag) = file_entry.tags.iter().find(|tag| tag.starts_with("path:")) {
                let relative_path = path_tag.strip_prefix("path:").unwrap();
                destination.join(relative_path)
            } else {
                destination.join(&file_entry.original_filename)
            };

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Restore file
        if let Err(e) = file_storage::handle_get_command(
            cli,
            key_manager,
            &file_entry.name,
            &output_path,
            &None,
        )
        .await
        {
            ui::print_error(&format!("Failed to restore {}: {}", file_entry.name, e));
        }

        if verify {
            // Verify file integrity
            verify_file_integrity(&output_path, &file_entry)?;
        }
    }

    progress.finish_operation(restore_op, "Restore completed");
    progress.clear();

    ui::print_success(&format!(
        "Backup '{}' restored to {}",
        backup_name,
        destination.display()
    ));
    Ok(())
}

/// Search for files using advanced criteria
pub async fn search_files(criteria: SearchCriteria) -> Result<Vec<FileEntry>> {
    ui::print_header("File Search");
    ui::print_key_value("Query", &criteria.query);

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let spinner = ui::create_spinner("Searching files...");

    // Get all files first
    let all_files = db.list_files(None)?;

    // Apply filters
    let mut results = Vec::new();

    for file in all_files {
        if matches_search_criteria(&file, &criteria)? {
            results.push(file);
            if results.len() >= criteria.limit {
                break;
            }
        }
    }

    spinner.finish_with_message("Search completed");

    // Sort by relevance
    results.sort_by(|a, b| {
        calculate_relevance_score(&a.name, &criteria.query)
            .partial_cmp(&calculate_relevance_score(&b.name, &criteria.query))
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });

    Ok(results)
}

/// Get recently uploaded/accessed files
pub async fn get_recent_files(
    count: usize,
    days: u32,
    file_type: Option<String>,
) -> Result<Vec<FileEntry>> {
    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let cutoff_date = Local::now() - ChronoDuration::days(days as i64);
    let all_files = db.list_files(None)?;

    let mut recent_files: Vec<FileEntry> = all_files
        .into_iter()
        .filter(|file| file.upload_time >= cutoff_date)
        .filter(|file| {
            if let Some(ref ft) = file_type {
                file.original_filename.ends_with(&format!(".{}", ft))
            } else {
                true
            }
        })
        .collect();

    recent_files.sort_by(|a, b| b.upload_time.cmp(&a.upload_time));
    recent_files.truncate(count);

    Ok(recent_files)
}

/// Duplicate a file with new name/tags
pub async fn duplicate_file(
    cli: &Cli,
    key_manager: &KeyManager,
    source: &str,
    new_name: Option<String>,
    new_tags: Option<String>,
) -> Result<()> {
    ui::print_header("File Duplication");

    // First get the source file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "dfs_duplicate_{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
    ));

    ui::print_operation_status("Source File", "Downloading", Some(source));
    file_storage::handle_get_command(cli, key_manager, source, &temp_file, &None)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Upload as new file
    let final_name = new_name.unwrap_or_else(|| format!("{}-copy", source));
    ui::print_operation_status("Duplicate", "Creating", Some(&final_name));

    file_storage::handle_put_command(
        cli,
        key_manager,
        &temp_file,
        &None,
        &Some(final_name.clone()),
        &new_tags,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Cleanup
    let _ = fs::remove_file(&temp_file);

    ui::print_success(&format!("File duplicated as '{}'", final_name));
    Ok(())
}

/// Find duplicate files based on similar file sizes (approximation for now)
pub async fn find_duplicate_files(min_size: u64) -> Result<Vec<FileEntry>> {
    let db_path = crate::database::get_default_db_path()?;
    let db = crate::database::DatabaseManager::new(&db_path)?;
    
    // For now, we'll find files with similar sizes as an approximation
    // In a real implementation, this would use actual content hashes
    let files = db.list_files(None)?;
    
    let mut potential_duplicates = Vec::new();
    let mut size_groups: std::collections::HashMap<u64, Vec<FileEntry>> = std::collections::HashMap::new();
    
    // Group files by size
    for file in files {
        if file.file_size >= min_size {
            size_groups.entry(file.file_size).or_insert_with(Vec::new).push(file);
        }
    }
    
    // Add files from groups with multiple entries
    for (_, group) in size_groups {
        if group.len() > 1 {
            potential_duplicates.extend(group);
        }
    }
    
    Ok(potential_duplicates)
}

/// Rename a file (metadata-only operation)
pub async fn rename_file(old_name: &str, new_name: &str) -> Result<()> {
    ui::print_header("File Rename");

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    ui::print_key_value("Old name", old_name);
    ui::print_key_value("New name", new_name);

    if ui::confirm_action("Proceed with rename?", true) {
        db.rename_file(old_name, new_name)?;
        ui::print_success("File renamed successfully");
    }

    Ok(())
}

// Helper functions

fn scan_directory(dir: &Path, exclude_patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    fn scan_recursive(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        exclude_patterns: &[String],
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check exclude patterns
            let path_str = path.to_string_lossy();
            if exclude_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            }) {
                continue;
            }

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                scan_recursive(&path, files, exclude_patterns)?;
            }
        }
        Ok(())
    }

    scan_recursive(dir, &mut files, exclude_patterns)?;
    Ok(files)
}

struct SyncPlan {
    uploads: Vec<PathBuf>,
    downloads: Vec<FileEntry>,
    updates: Vec<(PathBuf, FileEntry)>,
}

fn create_sync_plan(
    local_files: &[PathBuf],
    remote_files: &[FileEntry],
    bidirectional: bool,
) -> Result<SyncPlan> {
    let mut plan = SyncPlan {
        uploads: Vec::new(),
        downloads: Vec::new(),
        updates: Vec::new(),
    };

    // Files to upload (local files not in remote)
    for local_file in local_files {
        let filename = local_file.file_name().unwrap().to_string_lossy();
        if !remote_files
            .iter()
            .any(|rf| rf.original_filename == filename)
        {
            plan.uploads.push(local_file.clone());
        }
    }

    if bidirectional {
        // Files to download (remote files not in local)
        for remote_file in remote_files {
            if !local_files.iter().any(|lf| {
                lf.file_name().unwrap().to_string_lossy() == remote_file.original_filename
            }) {
                plan.downloads.push(remote_file.clone());
            }
        }
    }

    Ok(plan)
}

async fn execute_sync_plan(
    cli: &Cli,
    key_manager: &KeyManager,
    plan: &SyncPlan,
    _options: &SyncOptions,
) -> Result<()> {
    // Upload files
    if !plan.uploads.is_empty() {
        ui::print_section("Uploading Files");
        let mut progress = ui::MultiOperationProgress::new();
        let upload_op = progress.add_operation("Upload", plan.uploads.len() as u64);

        for (i, file_path) in plan.uploads.iter().enumerate() {
            let filename = file_path.file_name().unwrap().to_string_lossy();
            progress.update_operation(upload_op, i as u64, &format!("Uploading {}", filename));

            if let Err(e) = file_storage::handle_put_command(
                cli,
                key_manager,
                file_path,
                &None,
                &Some(filename.to_string()),
                &Some("sync".to_string()),
            )
            .await
            {
                ui::print_error(&format!("Failed to upload {}: {}", filename, e));
            }
        }

        progress.finish_operation(upload_op, "Uploads completed");
        progress.clear();
    }

    // Download files
    if !plan.downloads.is_empty() {
        ui::print_section("Downloading Files");
        let mut progress = ui::MultiOperationProgress::new();
        let download_op = progress.add_operation("Download", plan.downloads.len() as u64);

        for (i, remote_file) in plan.downloads.iter().enumerate() {
            progress.update_operation(
                download_op,
                i as u64,
                &format!("Downloading {}", remote_file.name),
            );

            let local_path = PathBuf::from(&remote_file.original_filename);
            if let Err(e) = file_storage::handle_get_command(
                cli,
                key_manager,
                &remote_file.name,
                &local_path,
                &None,
            )
            .await
            {
                ui::print_error(&format!("Failed to download {}: {}", remote_file.name, e));
            }
        }

        progress.finish_operation(download_op, "Downloads completed");
        progress.clear();
    }

    Ok(())
}

async fn start_file_watcher(
    cli: &Cli,
    key_manager: &KeyManager,
    local_dir: &Path,
    options: SyncOptions,
) -> Result<()> {
    use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    ui::print_info("Starting file watcher...");
    ui::print_info("Press Ctrl+C to stop watching");

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    // Create a watcher
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Watch the directory
    watcher.watch(local_dir, RecursiveMode::Recursive)?;

    // Clone data for the async task
    let cli_clone = cli.clone();
    let key_manager_clone = key_manager.clone();
    let local_dir_clone = local_dir.to_path_buf();
    let exclude_patterns = options.exclude_patterns.clone();

    // Process events in a separate task
    let handle = tokio::task::spawn_blocking(move || loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let Err(e) = handle_file_event(
                    &cli_clone,
                    &key_manager_clone,
                    &local_dir_clone,
                    event,
                    &exclude_patterns,
                ) {
                    ui::print_error(&format!("Error handling file event: {}", e));
                }
            }
            Ok(Err(e)) => {
                ui::print_error(&format!("Watch error: {}", e));
            }
            Err(_) => {
                ui::print_info("File watcher stopped");
                break;
            }
        }
    });

    // Wait for the task to complete or be interrupted
    match handle.await {
        Ok(_) => ui::print_info("File watching completed"),
        Err(e) => ui::print_error(&format!("File watcher task error: {}", e)),
    }

    Ok(())
}

fn handle_file_event(
    _cli: &Cli,
    _key_manager: &KeyManager,
    local_dir: &Path,
    event: notify::Event,
    exclude_patterns: &[String],
) -> Result<()> {
    match event.kind {
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
            for path in event.paths {
                if path.is_file() {
                    // Check if file should be excluded
                    let relative_path = path.strip_prefix(local_dir).unwrap_or(&path);
                    let path_str = relative_path.to_string_lossy();

                    let should_exclude = exclude_patterns.iter().any(|pattern| {
                        glob::Pattern::new(pattern)
                            .map(|p| p.matches(&path_str))
                            .unwrap_or(false)
                    });

                    if !should_exclude {
                        ui::print_info(&format!("File changed: {}", path.display()));

                        // Upload the file (in a real implementation, this would be async)
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        // Note: This is a simplified sync version
                        // In practice, you'd want to batch changes and handle them asynchronously
                        ui::print_info(&format!("Would upload: {}", filename));
                    }
                }
            }
        }
        notify::EventKind::Remove(_) => {
            for path in event.paths {
                ui::print_info(&format!("File removed: {}", path.display()));
                // Handle file removal - maybe mark as deleted in database
            }
        }
        _ => {
            // Ignore other event types
        }
    }

    Ok(())
}

fn matches_search_criteria(file: &FileEntry, criteria: &SearchCriteria) -> Result<bool> {
    // Check query match
    let query_match = if criteria.use_regex {
        regex::Regex::new(&criteria.query)?.is_match(&file.name)
            || regex::Regex::new(&criteria.query)?.is_match(&file.original_filename)
    } else {
        file.name
            .to_lowercase()
            .contains(&criteria.query.to_lowercase())
            || file
                .original_filename
                .to_lowercase()
                .contains(&criteria.query.to_lowercase())
            || file
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&criteria.query.to_lowercase()))
    };

    if !query_match {
        return Ok(false);
    }

    // Check file type
    if let Some(ref file_type) = criteria.file_type {
        if !file.original_filename.ends_with(&format!(".{}", file_type)) {
            return Ok(false);
        }
    }

    // Check size range
    if let Some(ref size_range) = criteria.size_range {
        match size_range {
            SizeRange::LessThan(limit) => {
                if file.file_size >= *limit {
                    return Ok(false);
                }
            }
            SizeRange::GreaterThan(limit) => {
                if file.file_size <= *limit {
                    return Ok(false);
                }
            }
            SizeRange::Between(min, max) => {
                if file.file_size < *min || file.file_size > *max {
                    return Ok(false);
                }
            }
        }
    }

    // Check date range
    if let Some(ref date_range) = criteria.date_range {
        let now = Local::now();
        match date_range {
            DateRange::LastDays(days) => {
                let cutoff = now - ChronoDuration::days(*days as i64);
                if file.upload_time < cutoff {
                    return Ok(false);
                }
            }
            DateRange::LastWeeks(weeks) => {
                let cutoff = now - ChronoDuration::weeks(*weeks as i64);
                if file.upload_time < cutoff {
                    return Ok(false);
                }
            }
            DateRange::LastMonths(months) => {
                let cutoff = now - ChronoDuration::days((*months as i64) * 30);
                if file.upload_time < cutoff {
                    return Ok(false);
                }
            }
            DateRange::Between(start, end) => {
                if file.upload_time < *start || file.upload_time > *end {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

fn calculate_relevance_score(filename: &str, query: &str) -> f64 {
    let filename_lower = filename.to_lowercase();
    let query_lower = query.to_lowercase();

    if filename_lower == query_lower {
        return 100.0;
    }

    if filename_lower.starts_with(&query_lower) {
        return 90.0;
    }

    if filename_lower.contains(&query_lower) {
        return 50.0;
    }

    // Basic fuzzy matching score
    let mut score = 0.0;
    let query_chars: Vec<char> = query_lower.chars().collect();
    let filename_chars: Vec<char> = filename_lower.chars().collect();

    let mut query_idx = 0;
    for filename_char in filename_chars {
        if query_idx < query_chars.len() && filename_char == query_chars[query_idx] {
            score += 1.0;
            query_idx += 1;
        }
    }

    score / query_chars.len() as f64 * 20.0
}

fn verify_file_integrity(file_path: &Path, file_entry: &FileEntry) -> Result<()> {
    let metadata = fs::metadata(file_path)?;
    if metadata.len() != file_entry.file_size {
        return Err(anyhow::anyhow!(
            "File size mismatch: expected {}, got {}",
            file_entry.file_size,
            metadata.len()
        ));
    }
    Ok(())
}
