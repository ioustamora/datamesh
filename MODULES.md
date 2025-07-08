# DataMesh Core Modules Documentation

This document describes the newly implemented core modules that complete the DataMesh distributed data storage system.

## 📊 Database Module (`src/database.rs`)

**Purpose**: SQLite-based persistent storage for file metadata and system state.

### Key Features
- **File Metadata Storage**: Store file names, keys, tags, and health information
- **Advanced Querying**: Search by name, key, tags, size, and date ranges
- **Unique Name Generation**: Automatic conflict resolution for file names
- **Statistics**: Database health and usage statistics
- **Tag Management**: Flexible tagging system for file organization

### Example Usage
```rust
use datamesh::database::DatabaseManager;

let db = DatabaseManager::new(&db_path)?;

// Store a file
let file_id = db.store_file(
    "my-document",
    "file-key-hash", 
    "document.pdf",
    1024 * 1024, // size in bytes
    upload_time,
    &vec!["work".to_string(), "important".to_string()],
    "public-key-hex"
)?;

// Retrieve by name
let file = db.get_file_by_name("my-document")?;

// Search with tags
let tagged_files = db.list_files(Some("work"))?;
```

## 📁 File Manager Module (`src/file_manager.rs`)

**Purpose**: High-level file operations including sync, backup, and search.

### Key Features
- **Directory Sync**: Bidirectional sync with watch mode
- **Backup & Restore**: Versioned backups with incremental support
- **Advanced Search**: Multi-criteria search with regex support
- **File Operations**: Duplicate, rename, and organize files
- **Recent Files**: Track recently accessed files

### Example Usage
```rust
use datamesh::file_manager::{sync_directory, SyncOptions, SearchCriteria};

// Sync local directory
let options = SyncOptions {
    watch: true,
    bidirectional: true,
    exclude_patterns: vec!["*.tmp".to_string()],
    parallel: 3,
};
sync_directory(&cli, &key_manager, &local_dir, options).await?;

// Advanced search
let criteria = SearchCriteria {
    query: "document".to_string(),
    file_type: Some("pdf".to_string()),
    size_range: Some(SizeRange::GreaterThan(1024 * 1024)), // > 1MB
    date_range: Some(DateRange::LastWeeks(2)),
    use_regex: false,
    limit: 50,
};
let results = search_files(criteria).await?;
```

## 🔄 Batch Operations Module (`src/batch_operations.rs`)

**Purpose**: Efficient bulk operations with parallel processing.

### Key Features
- **Batch Upload**: Upload multiple files with glob patterns
- **Batch Download**: Download files with structure preservation  
- **Bulk Tagging**: Add/remove tags from multiple files
- **Progress Tracking**: Real-time progress for all operations
- **Error Handling**: Continue processing despite individual failures

### Example Usage
```rust
use datamesh::batch_operations::{batch_put, BatchPutConfig};

let config = BatchPutConfig {
    pattern: "documents/*.pdf".to_string(),
    recursive: true,
    parallel: 5,
    base_dir: Some(PathBuf::from("./data")),
    tag_pattern: Some("type:{ext},folder:{path}".to_string()),
};

let result = batch_put(&cli, &key_manager, config).await?;
println!("Uploaded {} files, {} failed", result.successful, result.failed);
```

## 🏥 Health Manager Module (`src/health_manager.rs`)

**Purpose**: System health monitoring, repair, and maintenance.

### Key Features
- **File Repair**: Reconstruct damaged files using Reed-Solomon
- **Storage Cleanup**: Remove orphaned data and duplicates
- **Quota Management**: Set and monitor storage limits
- **Performance Benchmarks**: Test system performance
- **Health Reports**: Comprehensive system health analysis

### Example Usage
```rust
use datamesh::health_manager::{repair_files, RepairConfig, generate_health_report};

// Repair files below threshold
let repair_config = RepairConfig {
    target: None, // All files
    auto: true,
    verify_all: false,
    threshold: 75, // Repair if health < 75%
};
repair_files(&cli, &key_manager, repair_config).await?;

// Generate health report
let report = generate_health_report().await?;
println!("System health: {:.1}%", report.average_health);
```

## 🌐 Network Diagnostics Module (`src/network_diagnostics.rs`)

**Purpose**: Network analysis, peer management, and performance monitoring.

### Key Features
- **Peer Statistics**: Track connection quality and reputation
- **Bandwidth Testing**: Measure network performance
- **Topology Analysis**: Visualize network structure
- **Health Monitoring**: Continuous network health tracking
- **Discovery**: Find and connect to new peers

### Example Usage
```rust
use datamesh::network_diagnostics::{handle_peers_command, handle_bandwidth_command};

// List connected peers
handle_peers_command(&mut swarm, true, &OutputFormat::Table).await?;

// Test bandwidth
handle_bandwidth_command(&mut swarm, &None, 30).await?;

// Monitor health continuously
handle_health_command(&mut swarm, true, 5).await?;
```

## ⚙️ Network Presets Module (`src/presets.rs`)

**Purpose**: Simplified network configuration with preset options.

### Key Features
- **Built-in Presets**: Local, public, and test network configurations
- **Custom Presets**: Save and load user-defined configurations
- **Auto-Discovery**: Automatic peer discovery for local networks
- **Connection Resolution**: Parse various network specification formats

### Example Usage
```rust
use datamesh::presets::{NetworkPresets, parse_network_spec};

// Use built-in preset
let presets = NetworkPresets::new();
let config = presets.apply_preset("local")?;

// Parse custom network spec
let custom_config = parse_network_spec("peer_id@/ip4/192.168.1.100/tcp/40871")?;

// List available presets
presets.print_available_presets();
```

## 🛠️ Enhanced Error Handling (`src/error_handling.rs`)

**Purpose**: User-friendly error messages with actionable suggestions.

### Key Features
- **Contextual Errors**: Enhanced error messages with context
- **Actionable Suggestions**: Specific suggestions for each error type
- **Error Batching**: Collect and display multiple errors
- **Severity Levels**: Categorize errors by importance
- **Operation Context**: Tailor suggestions based on the operation

### Example Usage
```rust
use datamesh::error_handling::{handle_error, operation_error_with_context};

// Handle generic error
let enhanced = handle_error(&some_error);
display_enhanced_error(&enhanced);

// Add operation context
let contextual = operation_error_with_context("put", &file_error);
display_enhanced_error(&contextual);
```

## 🎨 Enhanced UI Components (`src/ui.rs`)

**Purpose**: Professional CLI interface with progress bars and formatting.

### Key Features
- **Progress Bars**: Single and multi-operation progress tracking
- **Colored Output**: Status-based color coding
- **Interactive Prompts**: User confirmations and input
- **Formatted Tables**: Data display in table format
- **File Listings**: Specialized file display with health indicators

### Example Usage
```rust
use datamesh::ui;

// Progress tracking
let mut progress = ui::MultiOperationProgress::new();
let op_id = progress.add_operation("Upload", 100);
progress.update_operation(op_id, 50, "Uploading file 50/100");

// Status messages
ui::print_success("Operation completed successfully");
ui::print_error("Something went wrong");
ui::print_warning("Check your configuration");

// Interactive confirmation
if ui::confirm_action("Delete all files?", false) {
    // User confirmed
}
```

## 🚀 Integration Examples

### Complete File Upload Workflow
```rust
// 1. Initialize database
let db = DatabaseManager::new(&get_default_db_path()?)?;

// 2. Set up network with preset
let presets = NetworkPresets::new();
let network_config = presets.apply_preset("local")?;

// 3. Batch upload with progress
let batch_config = BatchPutConfig {
    pattern: "documents/**/*".to_string(),
    recursive: true,
    parallel: 3,
    base_dir: Some(PathBuf::from("./data")),
    tag_pattern: Some("uploaded:{date},type:{ext}".to_string()),
};

let result = batch_put(&cli, &key_manager, batch_config).await?;

// 4. Health check and optimization
let health_report = generate_health_report().await?;
if health_report.average_health < 80.0 {
    ui::print_warning("System health below 80%, running cleanup...");
    let cleanup_config = CleanupConfig {
        orphaned: true,
        duplicates: true,
        low_health: false,
        dry_run: false,
        force: true,
    };
    cleanup_storage(cleanup_config).await?;
}
```

### Advanced Search and Analytics
```rust
// Complex search with multiple criteria
let search_results = search_files(SearchCriteria {
    query: "important".to_string(),
    file_type: Some("pdf".to_string()),
    size_range: Some(SizeRange::Between(1024 * 1024, 100 * 1024 * 1024)),
    date_range: Some(DateRange::LastMonths(6)),
    use_regex: false,
    limit: 100,
}).await?;

// Analyze results
ui::print_file_list(&search_results);

// Generate distribution analysis
handle_distribution_command(&mut swarm, &None, &Some("work".to_string())).await?;
```

## 📈 Performance & Monitoring

All modules include comprehensive logging, error handling, and performance monitoring:

- **Tracing**: Structured logging for debugging and monitoring
- **Metrics**: Operation timing and success rates
- **Health Checks**: Continuous system health monitoring
- **Benchmarking**: Performance testing capabilities

## 🔧 Testing

Comprehensive test suite covering:
- Unit tests for each module
- Integration tests for module interactions
- Error handling scenarios
- Performance benchmarks

Run tests with:
```bash
cargo test
cargo test --release --test integration_tests
```

## 📚 Dependencies

All modules use production-ready dependencies:
- `tokio` - Async runtime
- `anyhow` - Error handling
- `rusqlite` - SQLite database
- `chrono` - Date/time handling
- `serde` - Serialization
- `indicatif` - Progress bars
- `colored` - Terminal colors

The modules are designed to be:
- **Async-first**: Non-blocking operations
- **Error-resilient**: Graceful error handling
- **User-friendly**: Clear feedback and suggestions
- **Performant**: Parallel processing where beneficial
- **Maintainable**: Well-documented and tested