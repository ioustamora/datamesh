/// Batch Operations Module
///
/// This module provides batch operations for multiple file handling:
/// - Batch upload with patterns and parallel processing
/// - Batch download with filtering
/// - Bulk tag operations
/// - Progress tracking for large operations

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::database::{DatabaseManager, FileEntry};
use crate::key_manager::KeyManager;
use crate::file_storage;
use crate::cli::Cli;
use crate::ui;

/// Configuration for batch put operations
#[derive(Debug, Clone)]
pub struct BatchPutConfig {
    pub pattern: String,
    pub recursive: bool,
    pub parallel: usize,
    pub base_dir: Option<PathBuf>,
    pub tag_pattern: Option<String>,
}

/// Configuration for batch get operations
#[derive(Debug, Clone)]
pub struct BatchGetConfig {
    pub pattern: String,
    pub destination: PathBuf,
    pub parallel: usize,
    pub preserve_structure: bool,
}

/// Configuration for batch tag operations
#[derive(Debug, Clone)]
pub struct BatchTagConfig {
    pub pattern: String,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub dry_run: bool,
}

/// Results from a batch operation
#[derive(Debug)]
pub struct BatchResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

/// Upload multiple files matching patterns
pub async fn batch_put(
    cli: &Cli,
    key_manager: &KeyManager,
    config: BatchPutConfig,
) -> Result<BatchResult> {
    ui::print_header("Batch Upload");
    ui::print_key_value("Pattern", &config.pattern);
    ui::print_key_value("Parallel operations", &config.parallel.to_string());
    
    let start_time = std::time::Instant::now();
    let base_dir = config.base_dir.as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| Path::new("."));
    
    // Find matching files
    let spinner = ui::create_spinner("Scanning for files...");
    let matching_files = find_matching_files(base_dir, &config.pattern, config.recursive)?;
    spinner.finish_with_message("Files found");
    
    if matching_files.is_empty() {
        ui::print_warning("No files found matching the pattern");
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    ui::print_key_value("Files to upload", &matching_files.len().to_string());
    
    if !ui::confirm_action("Proceed with batch upload?", true) {
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    // Set up progress tracking
    let mut progress = ui::MultiOperationProgress::new();
    let upload_progress = progress.add_operation("Batch Upload", matching_files.len() as u64);
    
    // Process files in parallel batches
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    let chunks: Vec<_> = matching_files.chunks(config.parallel).collect();
    
    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        let mut upload_futures = Vec::new();
        
        for (_file_idx, file_path) in chunk.iter().enumerate() {
            let cli_clone = cli.clone();
            let key_manager_clone = key_manager.clone();
            let file_path_clone = file_path.clone();
            let config_clone = config.clone();
            let base_dir_clone = base_dir.to_path_buf();
            
            let future = async move {
                upload_single_file(
                    &cli_clone,
                    &key_manager_clone,
                    &file_path_clone,
                    &config_clone,
                    &base_dir_clone,
                ).await
            };
            
            upload_futures.push(future);
        }
        
        // Wait for this batch to complete using join_all
        let results = futures::future::join_all(upload_futures).await;
        
        for (_idx, result) in results.into_iter().enumerate() {
            let current_progress = chunk_idx * config.parallel + successful + failed + 1;
            progress.update_operation(
                upload_progress,
                current_progress as u64,
                &format!("Processing file {}/{}", current_progress, matching_files.len())
            );
            
            match result {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(e.to_string());
                }
            }
        }
    }
    
    progress.finish_operation(upload_progress, &format!("Completed: {} successful, {} failed", successful, failed));
    progress.clear();
    
    ui::print_section("Batch Upload Results");
    ui::print_key_value("Successful uploads", &successful.to_string());
    ui::print_key_value("Failed uploads", &failed.to_string());
    ui::print_key_value("Duration", &format!("{:.2}s", start_time.elapsed().as_secs_f64()));
    
    if !errors.is_empty() {
        ui::print_warning("Errors encountered:");
        for (i, error) in errors.iter().enumerate().take(5) {
            println!("  {}: {}", i + 1, error);
        }
        if errors.len() > 5 {
            println!("  ... and {} more errors", errors.len() - 5);
        }
    }
    
    Ok(BatchResult {
        successful,
        failed,
        errors,
        duration: start_time.elapsed(),
    })
}

/// Download multiple files matching patterns
pub async fn batch_get(
    cli: &Cli,
    key_manager: &KeyManager,
    config: BatchGetConfig,
) -> Result<BatchResult> {
    ui::print_header("Batch Download");
    ui::print_key_value("Pattern", &config.pattern);
    ui::print_key_value("Destination", &config.destination.display().to_string());
    
    let start_time = std::time::Instant::now();
    
    // Find matching files in database
    let spinner = ui::create_spinner("Searching for files...");
    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    let all_files = db.list_files(None)?;
    
    let matching_files: Vec<FileEntry> = all_files
        .into_iter()
        .filter(|file| matches_pattern(&file.name, &config.pattern) || 
                      matches_pattern(&file.original_filename, &config.pattern))
        .collect();
    
    spinner.finish_with_message("Files found");
    
    if matching_files.is_empty() {
        ui::print_warning("No files found matching the pattern");
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    ui::print_key_value("Files to download", &matching_files.len().to_string());
    
    // Create destination directory
    std::fs::create_dir_all(&config.destination)
        .context("Failed to create destination directory")?;
    
    if !ui::confirm_action("Proceed with batch download?", true) {
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    // Set up progress tracking
    let mut progress = ui::MultiOperationProgress::new();
    let download_progress = progress.add_operation("Batch Download", matching_files.len() as u64);
    
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    // Process files in parallel batches
    let chunks: Vec<_> = matching_files.chunks(config.parallel).collect();
    
    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        let mut download_futures = Vec::new();
        
        for file_entry in chunk.iter() {
            let cli_clone = cli.clone();
            let key_manager_clone = key_manager.clone();
            let file_entry_clone = file_entry.clone();
            let config_clone = config.clone();
            
            let future = async move {
                download_single_file(
                    &cli_clone,
                    &key_manager_clone,
                    &file_entry_clone,
                    &config_clone,
                ).await
            };
            
            download_futures.push(future);
        }
        
        // Wait for this batch to complete using join_all
        let results = futures::future::join_all(download_futures).await;
        
        for (_idx, result) in results.into_iter().enumerate() {
            let current_progress = chunk_idx * config.parallel + successful + failed + 1;
            progress.update_operation(
                download_progress,
                current_progress as u64,
                &format!("Processing file {}/{}", current_progress, matching_files.len())
            );
            
            match result {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(e.to_string());
                }
            }
        }
    }
    
    progress.finish_operation(download_progress, &format!("Completed: {} successful, {} failed", successful, failed));
    progress.clear();
    
    ui::print_section("Batch Download Results");
    ui::print_key_value("Successful downloads", &successful.to_string());
    ui::print_key_value("Failed downloads", &failed.to_string());
    ui::print_key_value("Duration", &format!("{:.2}s", start_time.elapsed().as_secs_f64()));
    
    Ok(BatchResult {
        successful,
        failed,
        errors,
        duration: start_time.elapsed(),
    })
}

/// Bulk tag operations on multiple files
pub async fn batch_tag(config: BatchTagConfig) -> Result<BatchResult> {
    ui::print_header("Batch Tag Operation");
    ui::print_key_value("Pattern", &config.pattern);
    
    if !config.add_tags.is_empty() {
        ui::print_key_value("Tags to add", &config.add_tags.join(", "));
    }
    if !config.remove_tags.is_empty() {
        ui::print_key_value("Tags to remove", &config.remove_tags.join(", "));
    }
    
    let start_time = std::time::Instant::now();
    
    // Find matching files
    let spinner = ui::create_spinner("Searching for files...");
    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    let all_files = db.list_files(None)?;
    
    let matching_files: Vec<FileEntry> = all_files
        .into_iter()
        .filter(|file| matches_pattern(&file.name, &config.pattern) || 
                      matches_pattern(&file.original_filename, &config.pattern))
        .collect();
    
    spinner.finish_with_message("Files found");
    
    if matching_files.is_empty() {
        ui::print_warning("No files found matching the pattern");
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    ui::print_key_value("Files to modify", &matching_files.len().to_string());
    
    if config.dry_run {
        ui::print_section("Dry Run - Files that would be modified:");
        for file in &matching_files {
            let new_tags = calculate_new_tags(&file.tags, &config.add_tags, &config.remove_tags);
            println!("  {} -> {}", file.name, new_tags.join(", "));
        }
        return Ok(BatchResult {
            successful: matching_files.len(),
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    if !ui::confirm_action("Proceed with tag modifications?", true) {
        return Ok(BatchResult {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        });
    }
    
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for file in &matching_files {
        let new_tags = calculate_new_tags(&file.tags, &config.add_tags, &config.remove_tags);
        
        match db.update_file_tags(&file.name, &new_tags) {
            Ok(_) => successful += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Failed to update {}: {}", file.name, e));
            }
        }
    }
    
    ui::print_section("Batch Tag Results");
    ui::print_key_value("Successfully modified", &successful.to_string());
    ui::print_key_value("Failed modifications", &failed.to_string());
    ui::print_key_value("Duration", &format!("{:.2}s", start_time.elapsed().as_secs_f64()));
    
    Ok(BatchResult {
        successful,
        failed,
        errors,
        duration: start_time.elapsed(),
    })
}

// Helper functions

fn find_matching_files(base_dir: &Path, pattern: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if recursive {
        find_files_recursive(base_dir, pattern, &mut files)?;
    } else {
        // Non-recursive: only check direct children
        for entry in std::fs::read_dir(base_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    if matches_pattern(&filename.to_string_lossy(), pattern) {
                        files.push(path);
                    }
                }
            }
        }
    }
    
    Ok(files)
}

fn find_files_recursive(dir: &Path, pattern: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(filename) = path.file_name() {
                if matches_pattern(&filename.to_string_lossy(), pattern) {
                    files.push(path);
                }
            }
        } else if path.is_dir() {
            find_files_recursive(&path, pattern, files)?;
        }
    }
    Ok(())
}

fn matches_pattern(text: &str, pattern: &str) -> bool {
    // Simple glob pattern matching
    if pattern.contains('*') || pattern.contains('?') {
        glob::Pattern::new(pattern)
            .map(|p| p.matches(text))
            .unwrap_or(false)
    } else {
        text.contains(pattern)
    }
}

async fn upload_single_file(
    cli: &Cli,
    key_manager: &KeyManager,
    file_path: &Path,
    config: &BatchPutConfig,
    base_dir: &Path,
) -> Result<()> {
    let relative_path = file_path.strip_prefix(base_dir)
        .unwrap_or(file_path);
    
    let name = relative_path.to_string_lossy().replace('/', "-");
    let tags = generate_tags_from_pattern(relative_path, &config.tag_pattern)?;
    
    file_storage::handle_put_command(
        cli,
        key_manager,
        &file_path.to_path_buf(),
        &None,
        &Some(name),
        &tags,
    ).await.map_err(|e| anyhow::anyhow!(e))
}

async fn download_single_file(
    cli: &Cli,
    key_manager: &KeyManager,
    file_entry: &FileEntry,
    config: &BatchGetConfig,
) -> Result<()> {
    let output_path = if config.preserve_structure {
        // Try to recreate directory structure from tags
        if let Some(path_tag) = file_entry.tags.iter().find(|tag| tag.starts_with("path:")) {
            let relative_path = path_tag.strip_prefix("path:").unwrap();
            config.destination.join(relative_path)
        } else {
            config.destination.join(&file_entry.original_filename)
        }
    } else {
        config.destination.join(&file_entry.original_filename)
    };
    
    // Create parent directories
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    file_storage::handle_get_command(
        cli,
        key_manager,
        &file_entry.name,
        &output_path,
        &None,
    ).await.map_err(|e| anyhow::anyhow!(e))
}

fn generate_tags_from_pattern(relative_path: &Path, pattern: &Option<String>) -> Result<Option<String>> {
    if let Some(pattern) = pattern {
        let filename = relative_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        let extension = relative_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let path_str = relative_path.to_string_lossy();
        
        let tags = pattern
            .replace("{name}", filename)
            .replace("{ext}", extension)
            .replace("{path}", &path_str);
        
        Ok(Some(tags))
    } else {
        Ok(None)
    }
}

fn calculate_new_tags(
    current_tags: &[String],
    add_tags: &[String],
    remove_tags: &[String],
) -> Vec<String> {
    let mut new_tags: Vec<String> = current_tags
        .iter()
        .filter(|tag| !remove_tags.contains(tag))
        .cloned()
        .collect();
    
    for tag in add_tags {
        if !new_tags.contains(tag) {
            new_tags.push(tag.clone());
        }
    }
    
    new_tags
}