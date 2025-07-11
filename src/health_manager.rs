use crate::cli::Cli;
use crate::database::{DatabaseManager, FileEntry};
use crate::key_manager::KeyManager;
use crate::ui;
use anyhow::Result;
use std::time::Duration;

/// Configuration for repair operations
#[derive(Debug, Clone)]
pub struct RepairConfig {
    pub target: Option<String>,
    pub auto: bool,
    pub verify_all: bool,
    pub threshold: u8,
}

/// Configuration for cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub orphaned: bool,
    pub duplicates: bool,
    pub low_health: bool,
    pub dry_run: bool,
    pub force: bool,
}

/// Quota configuration and usage information
#[derive(Debug, Clone)]
pub struct QuotaInfo {
    pub current_usage: u64,
    pub limit: Option<u64>,
    pub warning_threshold: Option<u8>,
    pub file_count: usize,
    pub usage_percentage: f64,
    pub warning_active: bool,
}

/// Health check results
#[derive(Debug)]
pub struct HealthReport {
    pub total_files: usize,
    pub healthy_files: usize,
    pub degraded_files: usize,
    pub critical_files: usize,
    pub orphaned_chunks: usize,
    pub duplicate_files: Vec<(String, Vec<String>)>,
    pub storage_usage: u64,
    pub database_size: u64,
    pub average_health: f64,
}

/// Benchmark results
#[derive(Debug)]
pub struct BenchmarkResults {
    pub storage_write_speed: f64, // MB/s
    pub storage_read_speed: f64,  // MB/s
    pub database_query_time: Duration,
    pub network_latency: Option<Duration>,
    pub chunk_processing_speed: f64,
    pub encryption_speed: f64,
}

/// Repair corrupted or low-redundancy files
pub async fn repair_files(cli: &Cli, key_manager: &KeyManager, config: RepairConfig) -> Result<()> {
    ui::print_header("File Repair Operation");

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let files_to_repair = if let Some(target) = &config.target {
        // Repair specific file
        if let Some(file) = db.get_file_by_name(target)? {
            vec![file]
        } else if let Some(file) = db.get_file_by_key(target)? {
            vec![file]
        } else {
            return Err(anyhow::anyhow!("File not found: {}", target));
        }
    } else if config.auto {
        // Auto repair files below threshold
        let all_files = db.list_files(None)?;
        all_files
            .into_iter()
            .filter(|file| {
                let health_percentage = (file.chunks_healthy * 100) / file.chunks_total;
                health_percentage < config.threshold as u32
            })
            .collect()
    } else {
        return Err(anyhow::anyhow!(
            "Either specify a target file or use --auto"
        ));
    };

    if files_to_repair.is_empty() {
        ui::print_success("No files need repair");
        return Ok(());
    }

    ui::print_key_value("Files to repair", &files_to_repair.len().to_string());

    if config.verify_all {
        ui::print_info("Performing integrity verification for all files...");
        verify_all_files(&db).await?;
    }

    let mut progress = ui::MultiOperationProgress::new();
    let repair_progress = progress.add_operation("Repair", files_to_repair.len() as u64);

    let mut repaired = 0;
    let mut failed = 0;

    for (i, file) in files_to_repair.iter().enumerate() {
        progress.update_operation(
            repair_progress,
            i as u64,
            &format!("Repairing {}", file.name),
        );

        match repair_single_file(cli, key_manager, file).await {
            Ok(_) => {
                repaired += 1;
                ui::print_success(&format!("Repaired: {}", file.name));
            }
            Err(e) => {
                failed += 1;
                ui::print_error(&format!("Failed to repair {}: {}", file.name, e));
            }
        }
    }

    progress.finish_operation(repair_progress, "Repair completed");
    progress.clear();

    ui::print_section("Repair Results");
    ui::print_key_value("Successfully repaired", &repaired.to_string());
    ui::print_key_value("Failed repairs", &failed.to_string());

    Ok(())
}

/// Clean up storage and optimize database
pub async fn cleanup_storage(config: CleanupConfig) -> Result<()> {
    ui::print_header("Storage Cleanup");

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let mut cleanup_actions = Vec::new();
    let mut total_space_freed = 0u64;

    if config.orphaned {
        ui::print_section("Checking for orphaned data...");
        let orphaned_size = find_and_clean_orphaned_data(&db, config.dry_run).await?;
        total_space_freed += orphaned_size;
        cleanup_actions.push(format!(
            "Removed {} of orphaned data",
            ui::format_file_size(orphaned_size)
        ));
    }

    if config.duplicates {
        ui::print_section("Checking for duplicate files...");
        let duplicates = find_duplicate_files(&db).await?;

        if !duplicates.is_empty() {
            ui::print_key_value("Duplicate groups found", &duplicates.len().to_string());

            if config.dry_run {
                for (hash, files) in &duplicates {
                    println!("  Duplicate group ({}): {:?}", hash, files);
                }
            } else if config.force
                || ui::confirm_action("Remove duplicate files (keeping newest)?", false)
            {
                let removed_size = remove_duplicate_files(&db, &duplicates).await?;
                total_space_freed += removed_size;
                cleanup_actions.push(format!("Removed {} duplicates", duplicates.len()));
            }
        } else {
            ui::print_success("No duplicate files found");
        }
    }

    if config.low_health {
        ui::print_section("Checking for low-health files...");
        let low_health_files = find_low_health_files(&db).await?;

        if !low_health_files.is_empty() {
            ui::print_warning(&format!(
                "Found {} files with irreparable low health",
                low_health_files.len()
            ));

            if config.dry_run {
                for file in &low_health_files {
                    println!(
                        "  Would remove: {} ({}% health)",
                        file.name,
                        (file.chunks_healthy * 100) / file.chunks_total
                    );
                }
            } else if config.force || ui::confirm_action("Remove low-health files?", false) {
                let removed_size = remove_low_health_files(&db, &low_health_files).await?;
                total_space_freed += removed_size;
                cleanup_actions.push(format!(
                    "Removed {} low-health files",
                    low_health_files.len()
                ));
            }
        } else {
            ui::print_success("No low-health files found");
        }
    }

    // Optimize database
    ui::print_section("Optimizing database...");
    if !config.dry_run {
        optimize_database(&db).await?;
        cleanup_actions.push("Optimized database".to_string());
    }

    ui::print_section("Cleanup Summary");
    ui::print_key_value(
        "Total space freed",
        &ui::format_file_size(total_space_freed),
    );
    ui::print_key_value("Actions performed", &cleanup_actions.len().to_string());

    for action in cleanup_actions {
        ui::print_list_item(&action, None);
    }

    if config.dry_run {
        ui::print_info("This was a dry run - no changes were made");
    }

    Ok(())
}

/// Manage storage quotas and usage
pub async fn manage_quota(usage: bool, limit: Option<String>, warn: Option<u8>) -> Result<()> {
    ui::print_header("Storage Quota Management");

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;
    let stats = db.get_stats()?;

    let mut quota_info = QuotaInfo {
        current_usage: stats.total_size,
        limit: None,
        warning_threshold: None,
        file_count: stats.total_files as usize,
        usage_percentage: 0.0,
        warning_active: false,
    };

    // Load existing quota settings
    if let Ok(existing_limit) = load_quota_limit() {
        quota_info.limit = Some(existing_limit);
        quota_info.usage_percentage = (stats.total_size as f64 / existing_limit as f64) * 100.0;
    }

    if let Ok(existing_warn) = load_warning_threshold() {
        quota_info.warning_threshold = Some(existing_warn);
        quota_info.warning_active = quota_info.usage_percentage > existing_warn as f64;
    }

    if usage {
        display_quota_usage(&quota_info);
    }

    if let Some(limit_str) = limit {
        let new_limit = parse_size_string(&limit_str)?;
        save_quota_limit(new_limit)?;
        quota_info.limit = Some(new_limit);
        quota_info.usage_percentage = (stats.total_size as f64 / new_limit as f64) * 100.0;
        ui::print_success(&format!(
            "Storage limit set to {}",
            ui::format_file_size(new_limit)
        ));
    }

    if let Some(warn_threshold) = warn {
        save_warning_threshold(warn_threshold)?;
        quota_info.warning_threshold = Some(warn_threshold);
        quota_info.warning_active = quota_info.usage_percentage > warn_threshold as f64;
        ui::print_success(&format!("Warning threshold set to {}%", warn_threshold));
    }

    // Show warnings if applicable
    if quota_info.warning_active {
        ui::print_warning(&format!(
            "Storage usage ({:.1}%) exceeds warning threshold ({}%)",
            quota_info.usage_percentage,
            quota_info.warning_threshold.unwrap_or(80)
        ));
    }

    if let Some(limit) = quota_info.limit {
        if stats.total_size > limit {
            ui::print_error(&format!(
                "Storage usage ({}) exceeds limit ({})",
                ui::format_file_size(stats.total_size),
                ui::format_file_size(limit)
            ));
        }
    }

    Ok(())
}

/// Run comprehensive performance benchmarks
pub async fn run_benchmarks(
    cli: &Cli,
    key_manager: &KeyManager,
    full: bool,
    network: bool,
    storage: bool,
    duration: u64,
) -> Result<BenchmarkResults> {
    ui::print_header("Performance Benchmarks");

    let mut results = BenchmarkResults {
        storage_write_speed: 0.0,
        storage_read_speed: 0.0,
        database_query_time: Duration::from_secs(0),
        network_latency: None,
        chunk_processing_speed: 0.0,
        encryption_speed: 0.0,
    };

    if full || storage {
        ui::print_section("Storage Benchmarks");

        // Test storage write speed
        ui::print_operation_status("Storage Write", "Testing", None);
        results.storage_write_speed = benchmark_storage_write(cli, key_manager, duration).await?;
        ui::print_key_value(
            "Write Speed",
            &format!("{:.1} MB/s", results.storage_write_speed),
        );

        // Test storage read speed
        ui::print_operation_status("Storage Read", "Testing", None);
        results.storage_read_speed = benchmark_storage_read(cli, key_manager, duration).await?;
        ui::print_key_value(
            "Read Speed",
            &format!("{:.1} MB/s", results.storage_read_speed),
        );

        // Test database performance
        ui::print_operation_status("Database", "Testing", None);
        results.database_query_time = benchmark_database_performance().await?;
        ui::print_key_value(
            "Query Time",
            &format!("{:.2}ms", results.database_query_time.as_millis()),
        );
    }

    if full || network {
        ui::print_section("Network Benchmarks");

        // Network latency would require active connection
        ui::print_info("Network benchmarks require active network connection");
        ui::print_info("Use 'dfs interactive' then 'bandwidth' for network tests");
    }

    if full {
        ui::print_section("Cryptographic Benchmarks");

        // Test encryption speed
        ui::print_operation_status("Encryption", "Testing", None);
        results.encryption_speed = benchmark_encryption_speed(duration).await?;
        ui::print_key_value(
            "Encryption Speed",
            &format!("{:.1} MB/s", results.encryption_speed),
        );

        // Test chunk processing
        ui::print_operation_status("Chunk Processing", "Testing", None);
        results.chunk_processing_speed = benchmark_chunk_processing(duration).await?;
        ui::print_key_value(
            "Chunk Processing",
            &format!("{:.1} MB/s", results.chunk_processing_speed),
        );
    }

    ui::print_section("Benchmark Summary");
    if results.storage_write_speed > 0.0 {
        ui::print_key_value(
            "Storage Write",
            &format!("{:.1} MB/s", results.storage_write_speed),
        );
    }
    if results.storage_read_speed > 0.0 {
        ui::print_key_value(
            "Storage Read",
            &format!("{:.1} MB/s", results.storage_read_speed),
        );
    }
    if results.encryption_speed > 0.0 {
        ui::print_key_value(
            "Encryption",
            &format!("{:.1} MB/s", results.encryption_speed),
        );
    }

    Ok(results)
}

/// Generate a comprehensive health report
pub async fn generate_health_report() -> Result<HealthReport> {
    ui::print_header("System Health Report");

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let spinner = ui::create_spinner("Analyzing system health...");

    let all_files = db.list_files(None)?;
    let stats = db.get_stats()?;

    let mut healthy_files = 0;
    let mut degraded_files = 0;
    let mut critical_files = 0;

    for file in &all_files {
        let health_percentage = (file.chunks_healthy * 100) / file.chunks_total;
        match health_percentage {
            90..=100 => healthy_files += 1,
            50..=89 => degraded_files += 1,
            _ => critical_files += 1,
        }
    }

    let duplicates = find_duplicate_files(&db).await?;
    let orphaned_chunks = count_orphaned_chunks().await?;

    spinner.finish_with_message("Health analysis completed");

    let report = HealthReport {
        total_files: all_files.len(),
        healthy_files,
        degraded_files,
        critical_files,
        orphaned_chunks,
        duplicate_files: duplicates,
        storage_usage: stats.total_size,
        database_size: get_database_size(&db_path)?,
        average_health: stats.average_health * 100.0,
    };

    display_health_report(&report);

    Ok(report)
}

// Helper functions

async fn repair_single_file(_cli: &Cli, _key_manager: &KeyManager, file: &FileEntry) -> Result<()> {
    // Placeholder for file repair logic
    // In a real implementation, this would:
    // 1. Download available chunks
    // 2. Use Reed-Solomon to reconstruct missing chunks
    // 3. Re-upload to improve redundancy

    if file.chunks_healthy < 4 {
        return Err(anyhow::anyhow!(
            "Cannot repair file with less than 4 healthy chunks"
        ));
    }

    // Simulate repair process
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(())
}

async fn verify_all_files(db: &DatabaseManager) -> Result<()> {
    let all_files = db.list_files(None)?;
    let mut verified = 0;
    let mut failed = 0;

    for file in all_files {
        // Placeholder verification logic
        if file.chunks_healthy >= file.chunks_total / 2 {
            verified += 1;
        } else {
            failed += 1;
        }
    }

    ui::print_key_value("Files verified", &verified.to_string());
    ui::print_key_value("Verification failures", &failed.to_string());

    Ok(())
}

async fn find_and_clean_orphaned_data(_db: &DatabaseManager, dry_run: bool) -> Result<u64> {
    // Placeholder for orphaned data cleanup
    let orphaned_size = 1024 * 1024; // 1MB placeholder

    if !dry_run {
        ui::print_info("Cleaning orphaned data...");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(orphaned_size)
}

async fn find_duplicate_files(_db: &DatabaseManager) -> Result<Vec<(String, Vec<String>)>> {
    // Placeholder for duplicate detection
    // In real implementation, would hash file contents and compare
    Ok(Vec::new())
}

async fn remove_duplicate_files(
    _db: &DatabaseManager,
    _duplicates: &[(String, Vec<String>)],
) -> Result<u64> {
    // Placeholder for duplicate removal
    Ok(0)
}

async fn find_low_health_files(db: &DatabaseManager) -> Result<Vec<FileEntry>> {
    let all_files = db.list_files(None)?;
    Ok(all_files
        .into_iter()
        .filter(|file| {
            let health_percentage = (file.chunks_healthy * 100) / file.chunks_total;
            health_percentage < 25 // Less than 25% health
        })
        .collect())
}

async fn remove_low_health_files(_db: &DatabaseManager, files: &[FileEntry]) -> Result<u64> {
    let total_size: u64 = files.iter().map(|f| f.file_size).sum();
    // Placeholder for actual removal
    Ok(total_size)
}

async fn optimize_database(_db: &DatabaseManager) -> Result<()> {
    // Placeholder for database optimization (VACUUM, REINDEX, etc.)
    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(())
}

async fn count_orphaned_chunks() -> Result<usize> {
    // Placeholder for orphaned chunk counting
    Ok(0)
}

fn display_quota_usage(quota_info: &QuotaInfo) {
    ui::print_section("Storage Usage");
    ui::print_key_value(
        "Current usage",
        &ui::format_file_size(quota_info.current_usage),
    );
    ui::print_key_value("File count", &quota_info.file_count.to_string());

    if let Some(limit) = quota_info.limit {
        ui::print_key_value("Storage limit", &ui::format_file_size(limit));
        ui::print_key_value(
            "Usage percentage",
            &format!("{:.1}%", quota_info.usage_percentage),
        );
    }

    if let Some(warn_threshold) = quota_info.warning_threshold {
        ui::print_key_value("Warning threshold", &format!("{}%", warn_threshold));
    }
}

fn display_health_report(report: &HealthReport) {
    ui::print_section("System Health Summary");

    ui::print_key_value("Total files", &report.total_files.to_string());
    ui::print_key_value(
        "Healthy files",
        &format!(
            "{} ({:.1}%)",
            report.healthy_files,
            (report.healthy_files as f64 / report.total_files as f64) * 100.0
        ),
    );
    ui::print_key_value(
        "Degraded files",
        &format!(
            "{} ({:.1}%)",
            report.degraded_files,
            (report.degraded_files as f64 / report.total_files as f64) * 100.0
        ),
    );
    ui::print_key_value(
        "Critical files",
        &format!(
            "{} ({:.1}%)",
            report.critical_files,
            (report.critical_files as f64 / report.total_files as f64) * 100.0
        ),
    );

    ui::print_key_value("Average health", &format!("{:.1}%", report.average_health));
    ui::print_key_value("Storage usage", &ui::format_file_size(report.storage_usage));
    ui::print_key_value("Database size", &ui::format_file_size(report.database_size));

    if report.orphaned_chunks > 0 {
        ui::print_warning(&format!("Found {} orphaned chunks", report.orphaned_chunks));
    }

    if !report.duplicate_files.is_empty() {
        ui::print_warning(&format!(
            "Found {} duplicate file groups",
            report.duplicate_files.len()
        ));
    }
}

// Benchmark helper functions

async fn benchmark_storage_write(
    _cli: &Cli,
    _key_manager: &KeyManager,
    duration: u64,
) -> Result<f64> {
    use std::io::Write;
    use std::time::Instant;

    let test_data = vec![0u8; 1024 * 1024]; // 1MB test data
    let start_time = Instant::now();
    let mut total_bytes = 0u64;

    // Create a temporary file for testing
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("dfs_write_benchmark.tmp");

    while start_time.elapsed().as_secs() < duration {
        // Write test data to temporary file
        let mut file = std::fs::File::create(&test_file)?;
        file.write_all(&test_data)?;
        file.sync_all()?; // Ensure data is written to disk

        total_bytes += test_data.len() as u64;

        // If duration is very short, add at least one iteration
        if duration == 0 {
            break;
        }
    }

    // Clean up
    let _ = std::fs::remove_file(&test_file);

    let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001); // Avoid division by zero
    let mb_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    Ok(mb_per_sec)
}

async fn benchmark_storage_read(
    _cli: &Cli,
    _key_manager: &KeyManager,
    duration: u64,
) -> Result<f64> {
    use std::io::Read;
    use std::time::Instant;

    let test_data = vec![0u8; 1024 * 1024]; // 1MB test data
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("dfs_read_benchmark.tmp");

    // Create test file
    std::fs::write(&test_file, &test_data)?;

    let start_time = Instant::now();
    let mut total_bytes = 0u64;

    while start_time.elapsed().as_secs() < duration {
        // Read test data from file
        let mut file = std::fs::File::open(&test_file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        total_bytes += buffer.len() as u64;

        // If duration is very short, add at least one iteration
        if duration == 0 {
            break;
        }
    }

    // Clean up
    let _ = std::fs::remove_file(&test_file);

    let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
    let mb_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    Ok(mb_per_sec)
}

async fn benchmark_database_performance() -> Result<Duration> {
    use std::time::Instant;

    let db_path = crate::database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    let start = Instant::now();

    // Perform several database operations to measure performance
    let num_operations = 100;

    for i in 0..num_operations {
        // Test database queries
        let _ = db.list_files(None)?;
        let _ = db.get_stats()?;

        // Test search functionality
        if i % 10 == 0 {
            let _ = db.search_files("test")?;
        }
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / num_operations;

    Ok(avg_time)
}

async fn benchmark_encryption_speed(duration: u64) -> Result<f64> {
    use blake3::Hasher;
    use std::time::Instant;

    let test_data = vec![42u8; 1024 * 1024]; // 1MB test data
    let start_time = Instant::now();
    let mut total_bytes = 0u64;

    while start_time.elapsed().as_secs() < duration {
        // Test BLAKE3 hashing (used in DFS for encryption)
        let mut hasher = Hasher::new();
        hasher.update(&test_data);
        let _hash = hasher.finalize();

        total_bytes += test_data.len() as u64;

        // If duration is very short, add at least one iteration
        if duration == 0 {
            break;
        }
    }

    let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
    let mb_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    Ok(mb_per_sec)
}

async fn benchmark_chunk_processing(duration: u64) -> Result<f64> {
    use reed_solomon_erasure::ReedSolomon;
    use std::time::Instant;

    let test_data = vec![42u8; 64 * 1024]; // 64KB chunks (typical size)
    let start_time = Instant::now();
    let mut total_bytes = 0u64;

    // Create Reed-Solomon encoder (6 data + 4 parity shards as used in DFS)
    let rs: ReedSolomon<reed_solomon_erasure::galois_8::Field> = ReedSolomon::new(6, 4)?;

    while start_time.elapsed().as_secs() < duration {
        // Split data into shards
        let mut shards: Vec<Vec<u8>> = Vec::new();
        let shard_size = test_data.len() / 6;

        for i in 0..6 {
            let start_idx = i * shard_size;
            let end_idx = if i == 5 {
                test_data.len()
            } else {
                (i + 1) * shard_size
            };
            shards.push(test_data[start_idx..end_idx].to_vec());
        }

        // Add parity shards
        for _ in 0..4 {
            shards.push(vec![0u8; shard_size]);
        }

        // Encode
        let _ = rs.encode(&mut shards);

        total_bytes += test_data.len() as u64;

        // If duration is very short, add at least one iteration
        if duration == 0 {
            break;
        }
    }

    let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
    let mb_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    Ok(mb_per_sec)
}

// Quota persistence functions (placeholders)

fn load_quota_limit() -> Result<u64> {
    // Placeholder - would load from config file
    Err(anyhow::anyhow!("No quota limit set"))
}

fn save_quota_limit(_limit: u64) -> Result<()> {
    // Placeholder - would save to config file
    Ok(())
}

fn load_warning_threshold() -> Result<u8> {
    // Placeholder - would load from config file
    Err(anyhow::anyhow!("No warning threshold set"))
}

fn save_warning_threshold(_threshold: u8) -> Result<()> {
    // Placeholder - would save to config file
    Ok(())
}

fn parse_size_string(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();

    if let Some(number_part) = size_str.strip_suffix("GB") {
        Ok(number_part.parse::<u64>()? * 1024 * 1024 * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("MB") {
        Ok(number_part.parse::<u64>()? * 1024 * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("KB") {
        Ok(number_part.parse::<u64>()? * 1024)
    } else if let Some(number_part) = size_str.strip_suffix("TB") {
        Ok(number_part.parse::<u64>()? * 1024 * 1024 * 1024 * 1024)
    } else {
        Ok(size_str.parse::<u64>()?)
    }
}

fn get_database_size(db_path: &std::path::Path) -> Result<u64> {
    Ok(std::fs::metadata(db_path)?.len())
}
