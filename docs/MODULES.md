# DataMesh Core Modules Documentation

This document describes the comprehensive module architecture of DataMesh, including core infrastructure, advanced monitoring, web interface, and governance systems.

## 📊 Database Module (`src/database.rs`)

**Status**: ✅ Production Ready

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

## 🔐 Security Modules

### Secure Random Module (`src/secure_random.rs`)

**Status**: ✅ Production Ready

**Purpose**: Cryptographically secure random number generation using OsRng.

### Key Features
- **Secure Nonces**: Generate 12-byte cryptographic nonces
- **Salt Generation**: 32-byte salts for password hashing
- **Arbitrary Length**: Generate secure bytes of any length
- **Range Generation**: Secure random numbers within ranges
- **Fill Operations**: Fill existing buffers with secure randomness

### Example Usage
```rust
use datamesh::secure_random;

// Generate secure nonce for encryption
let nonce = secure_random::generate_secure_nonce();

// Generate salt for password hashing
let salt = secure_random::generate_secure_salt();

// Fill buffer with secure random data
let mut buffer = vec![0u8; 256];
secure_random::fill_secure_bytes(&mut buffer);
```

### Key Rotation Module (`src/key_rotation.rs`)

**Status**: ✅ Production Ready

**Purpose**: Perfect forward secrecy through automatic key rotation.

### Key Features
- **Automatic Rotation**: Time-based and event-based key rotation
- **Manual Rotation**: On-demand key rotation for security events
- **Version Management**: Track key versions and history
- **Secure Storage**: Encrypted key storage with integrity verification
- **Configuration**: Flexible rotation policies and intervals

### Example Usage
```rust
use datamesh::key_rotation::{KeyRotationManager, KeyRotationConfig};

let config = KeyRotationConfig {
    automatic_rotation_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
    max_key_history: 10,
    require_manual_approval: false,
};

let rotation_manager = KeyRotationManager::new(config).await?;
rotation_manager.start_automatic_rotation().await?;

// Manual rotation for security events
rotation_manager.rotate_keys_manually("security_incident").await?;
```

### Secure Transport Module (`src/secure_transport.rs`)

**Status**: ✅ Production Ready

**Purpose**: Transport layer security with peer authentication.

### Key Features
- **TLS Configuration**: Secure transport configuration
- **Certificate Management**: Certificate generation and validation
- **Peer Authentication**: Mutual authentication between peers
- **Connection Tracking**: Monitor and validate peer connections
- **Security Policies**: Configurable security policies

### Thread-Safe Database Module (`src/thread_safe_database.rs`)

**Status**: ✅ Production Ready

**Purpose**: Thread-safe wrapper for database operations.

### Key Features
- **Concurrent Access**: Safe concurrent database operations
- **Connection Pooling**: Efficient connection management
- **Error Handling**: Thread-safe error propagation
- **Performance**: Optimized for high-concurrency scenarios

### Encrypted Key Manager Module (`src/encrypted_key_manager.rs`)

**Status**: ✅ Production Ready

**Purpose**: Password-protected key storage with advanced encryption.

### Key Features
- **AES-256-GCM**: Strong encryption for key storage
- **Argon2 Hashing**: Secure password-based key derivation
- **Integrity Verification**: Detect tampering and corruption
- **Secure Overwrite**: Secure deletion of sensitive data
- **Multiple Security Levels**: Different protection levels

### Example Usage
```rust
use datamesh::encrypted_key_manager::EncryptedKeyManager;

let encrypted_manager = EncryptedKeyManager::new("my-secure-key").await?;
encrypted_manager.store_with_password("strong_password").await?;

// Load with password
let loaded_manager = EncryptedKeyManager::load_with_password(
    &encrypted_file_path, 
    "strong_password"
).await?;
```

## 🎭 Actor System Modules

### Actor File Storage Module (`src/actor_file_storage.rs`)

**Status**: ✅ Production Ready

**Purpose**: Actor-based file operations for improved concurrency.

### Key Features
- **Message Passing**: Async message-based file operations
- **Concurrency**: Handle multiple operations simultaneously
- **Error Isolation**: Isolate errors between operations
- **State Management**: Maintain consistent state across operations

### Network Actor Module (`src/network_actor.rs`)

**Status**: ✅ Production Ready

**Purpose**: Actor-based network operations and P2P communication.

### Key Features
- **P2P Management**: Handle peer-to-peer connections
- **Message Routing**: Route messages between network components
- **Connection State**: Track and manage connection states
- **Event Handling**: Process network events asynchronously

### Thread-Safe Command Context (`src/thread_safe_command_context.rs`)

**Status**: ✅ Production Ready

**Purpose**: Thread-safe context for command execution across actors.

### Key Features
- **Shared State**: Safe shared state between command handlers
- **Context Isolation**: Isolate command execution contexts
- **Resource Management**: Manage shared resources safely
- **Error Propagation**: Safe error handling across threads

## 📁 File Manager Module (`src/file_manager.rs`)

**Status**: ✅ Production Ready

**Purpose**: High-level file operations including sync, backup, and search.

### Key Features
- **Directory Sync**: Bidirectional sync with watch mode
- **Backup & Restore**: Versioned backups with incremental support
- **Advanced Search**: Multi-criteria search with regex support and relevance scoring
- **File Operations**: Duplicate, rename, and organize files
- **Recent Files**: Track recently accessed files with time-based filtering

### Search Implementation Details

The search system implements a comprehensive multi-layered approach:

1. **Database Layer**: SQL-based queries with optimized indexes
2. **Filtering Engine**: Size, date, and file type filters
3. **Regex Engine**: Optional pattern matching support
4. **Relevance Scoring**: Semantic matching with weighted scoring
5. **Caching**: Query result caching for performance

#### Search Criteria Structure
```rust
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub query: String,
    pub file_type: Option<String>,
    pub size_range: Option<SizeRange>,
    pub date_range: Option<DateRange>,
    pub use_regex: bool,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub enum SizeRange {
    LessThan(u64),
    GreaterThan(u64),
    Between(u64, u64),
}

#[derive(Debug, Clone)]
pub enum DateRange {
    LastDays(u32),
    LastWeeks(u32),
    LastMonths(u32),
    Between(DateTime<Local>, DateTime<Local>),
}
```

#### Relevance Scoring Algorithm
```rust
fn calculate_relevance_score(filename: &str, query: &str) -> f64 {
    // Exact match gets highest score
    if filename.to_lowercase().contains(&query.to_lowercase()) {
        return 1.0;
    }
    
    // Partial matching with word overlap scoring
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let filename_words: Vec<&str> = filename.split_whitespace().collect();
    
    // Calculate intersection and position weighting
    // Implementation includes fuzzy matching and relevance weighting
}
```

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

**Status**: ✅ Production Ready

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

**Status**: ✅ Production Ready

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

**Status**: ✅ Production Ready

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

**Status**: ✅ Production Ready

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

**Status**: ✅ Production Ready

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

**Status**: ✅ Production Ready

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

## 📊 Advanced Monitoring System (`src/monitoring/`)

**Status**: ✅ Production Ready

### Core Monitoring (`src/monitoring/mod.rs`)

**Purpose**: Central monitoring system with comprehensive metrics collection and analytics.

### Key Features
- **Real-time Metrics**: Comprehensive system performance monitoring
- **ML-based Analytics**: Predictive insights and optimization recommendations
- **Health Scoring**: Weighted system health calculation
- **Report Generation**: Comprehensive analytics reports
- **Auto-scaling**: Intelligent resource management

### Example Usage
```rust
use datamesh::monitoring::AdvancedMonitoringSystem;

let monitoring = AdvancedMonitoringSystem::new(config).await?;
monitoring.start().await?;

let metrics = monitoring.collect_comprehensive_metrics().await?;
let report = monitoring.generate_analytics_report(Duration::from_days(7)).await?;
```

### Metrics Collection (`src/monitoring/metrics.rs`)

**Purpose**: Intelligent data gathering with minimal performance impact.

### Key Features
- **Multi-source Collection**: System, network, storage, user, and governance metrics
- **Performance Optimization**: Minimal overhead metrics collection
- **Real-time Caching**: Efficient data aggregation and storage
- **Custom Metrics**: Extensible metrics framework

### Example Usage
```rust
use datamesh::monitoring::metrics::MetricsCollector;

let collector = MetricsCollector::new(collection_interval).await?;
let metrics = collector.collect_all_metrics().await?;
```

### Time Series Database (`src/monitoring/time_series.rs`)

**Purpose**: High-performance historical data storage and querying.

### Key Features
- **Data Compression**: Efficient storage with compression algorithms
- **Advanced Querying**: Complex time-series queries with aggregations
- **Retention Management**: Automated data lifecycle management
- **Performance Optimization**: Indexing and caching for fast queries

### Example Usage
```rust
use datamesh::monitoring::time_series::{TimeSeriesDB, TimeSeriesQuery};

let db = TimeSeriesDB::new(retention_period).await?;
let query = TimeSeriesQuery {
    metric_names: vec!["cpu_usage".to_string()],
    start_time: Utc::now() - Duration::from_hours(24),
    end_time: Utc::now(),
    aggregation: Some(AggregationFunction::Average),
};
let data = db.query(&query).await?;
```

### Intelligent Alerting (`src/monitoring/alerts.rs`)

**Purpose**: ML-based anomaly detection and intelligent alerting system.

### Key Features
- **Anomaly Detection**: Machine learning-based pattern recognition
- **Escalation Management**: Intelligent alert routing and escalation
- **Correlation Analysis**: Multi-metric correlation and pattern detection
- **Notification Routing**: Multiple notification channels with preferences

### Example Usage
```rust
use datamesh::monitoring::alerts::{AlertManager, AlertRule};

let alert_manager = AlertManager::new(cooldown_period).await?;
let rule = AlertRule::new("high_cpu")
    .condition(AlertCondition::Threshold {
        metric: "cpu_usage".to_string(),
        threshold: 80.0,
    })
    .severity(AlertSeverity::Warning);
alert_manager.register_rule(rule).await?;
```

### Analytics Engine (`src/monitoring/analytics.rs`)

**Purpose**: Advanced analytics with ML-based insights and predictive capabilities.

### Key Features
- **Predictive Analytics**: ML models for performance prediction
- **Pattern Recognition**: Automated usage pattern detection
- **Optimization Recommendations**: AI-driven optimization suggestions
- **Performance Insights**: Deep performance analysis and trending

### Example Usage
```rust
use datamesh::monitoring::analytics::AnalyticsEngine;

let engine = AnalyticsEngine::new(analysis_window).await?;
let insights = engine.generate_insights_report(Duration::from_days(30)).await?;
let recommendations = engine.get_optimization_recommendations().await?;
```

### Real-time Dashboard (`src/monitoring/dashboard.rs`)

**Purpose**: Interactive monitoring dashboard with real-time visualization.

### Key Features
- **Live Data Updates**: Real-time metrics visualization
- **Customizable Widgets**: User-configurable dashboard layouts
- **Export Capabilities**: Data export in multiple formats
- **User Preferences**: Personalized dashboard configurations

### Example Usage
```rust
use datamesh::monitoring::dashboard::MonitoringDashboard;

let dashboard = MonitoringDashboard::new(refresh_interval).await?;
let data = dashboard.get_dashboard_data().await?;
let export = dashboard.export_dashboard_data(ExportFormat::JSON, time_range).await?;
```

## 🌐 Web Interface (`web-interface/`)

**Status**: ✅ Production Ready

### Vue.js Frontend Application

**Purpose**: Modern web interface for DataMesh with comprehensive functionality.

### Key Features
- **Responsive Design**: Mobile-friendly interface with modern UI
- **Real-time Updates**: WebSocket integration for live data
- **File Management**: Drag-and-drop uploads with progress tracking
- **Analytics Dashboard**: Interactive charts and metrics visualization
- **User Authentication**: Secure login and session management
- **Multi-language Support**: Internationalization ready

### Core Views
- **Dashboard.vue**: Main system overview with key metrics
- **Analytics.vue**: Comprehensive analytics with interactive charts
- **FileManager.vue**: File management with upload/download capabilities
- **Governance.vue**: Network governance and voting interface
- **Profile.vue**: User profile and settings management

### State Management
- **auth.js**: Authentication state and user management
- **files.js**: File operations and metadata management
- **governance.js**: Governance proposals and voting
- **websocket.js**: Real-time data synchronization

### API Integration
- **api.js**: RESTful API client with authentication
- **Real-time updates**: WebSocket integration for live data
- **Error handling**: Comprehensive error handling and user feedback

## 🏛️ Governance System (`src/governance.rs`, `src/governance_service.rs`)

**Status**: ✅ Production Ready

### Network Governance (`src/governance.rs`)

**Purpose**: Democratic network governance with bootstrap node administration.

### Key Features
- **Proposal System**: Democratic proposal submission and voting
- **Bootstrap Administration**: Operator management and permissions
- **Token Economics**: Governance token distribution and staking
- **Voting Mechanisms**: Weighted voting with quorum requirements

### Example Usage
```rust
use datamesh::governance::{GovernanceManager, Proposal};

let governance = GovernanceManager::new().await?;
let proposal = Proposal::new(
    "Network Upgrade",
    "Proposal to upgrade network protocol",
    ProposalType::NetworkUpgrade,
);
governance.submit_proposal(proposal).await?;
```

### Governance Service (`src/governance_service.rs`)

**Purpose**: API service for governance operations and proposal management.

### Key Features
- **Proposal Management**: Create, vote on, and execute proposals
- **Operator Administration**: Bootstrap node operator management
- **Voting Interface**: Web-based voting with real-time updates
- **Governance Analytics**: Participation metrics and voting patterns

## 💰 Economic System (`src/economics.rs`, `src/quota_service.rs`)

**Status**: ✅ Production Ready

### Token Economics (`src/economics.rs`)

**Purpose**: Network economics with token-based incentives and rewards.

### Key Features
- **Token System**: Native token with staking and rewards
- **Incentive Engine**: Reward calculations for node operators
- **Cost Calculator**: Fair usage pricing and cost estimation
- **Reward Distribution**: Automated token distribution system

### Quota Service (`src/quota_service.rs`)

**Purpose**: User quota management and fair usage enforcement.

### Key Features
- **Tiered Quotas**: Free, premium, and enterprise account tiers
- **Usage Tracking**: Real-time usage monitoring and enforcement
- **Rate Limiting**: API and bandwidth rate limiting
- **Billing Integration**: Usage-based billing and subscription management

### Example Usage
```rust
use datamesh::quota_service::{QuotaManager, UserQuota};

let quota_manager = QuotaManager::new().await?;
let quota = quota_manager.get_user_quota(&user_id).await?;
quota_manager.enforce_upload_quota(&user_id, file_size).await?;
```

## 📝 Audit & Logging (`src/audit_logger.rs`)

**Status**: ✅ Production Ready

### Audit Logger (`src/audit_logger.rs`)

**Purpose**: Comprehensive audit logging for compliance and security.

### Key Features
- **Comprehensive Logging**: All user actions and system events
- **Compliance Support**: GDPR, HIPAA, and regulatory compliance
- **Security Monitoring**: Authentication and authorization logging
- **Performance Tracking**: Operation timing and success rates

### Example Usage
```rust
use datamesh::audit_logger::{AuditLogger, AuditEvent};

let logger = AuditLogger::new().await?;
logger.log_user_action(&user_id, "file_upload", &metadata).await?;
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

// Advanced monitoring integration
let monitoring = AdvancedMonitoringSystem::new(config).await?;
monitoring.start().await?;

// Real-time analytics
let insights = monitoring.analytics_engine.generate_insights_report(Duration::from_days(7)).await?;
let recommendations = monitoring.get_optimization_recommendations().await?;

// Dashboard integration
let dashboard = MonitoringDashboard::new(Duration::from_secs(5)).await?;
let live_data = dashboard.get_dashboard_data().await?;
```

### Web Interface Integration
```rust
// Start REST API server
let api_server = ApiServer::new(config).await?;
api_server.start().await?;

// WebSocket for real-time updates
let websocket_handler = WebSocketHandler::new();
websocket_handler.start_broadcasting().await?;

// User authentication and quotas
let auth_service = AuthenticationService::new().await?;
let quota_service = QuotaService::new().await?;

// Governance integration
let governance = GovernanceManager::new().await?;
governance.start_proposal_monitoring().await?;
```

## 📈 Performance & Monitoring

All modules include comprehensive performance monitoring and observability:

- **Advanced Monitoring**: ML-based monitoring with predictive analytics
- **Time-series Database**: High-performance historical data storage
- **Intelligent Alerting**: Anomaly detection with smart notifications
- **Real-time Dashboard**: Interactive monitoring with customizable widgets
- **Analytics Engine**: Performance insights and optimization recommendations
- **Audit Logging**: Comprehensive audit trails for compliance
- **Health Scoring**: Weighted system health calculation
- **Auto-scaling**: Intelligent resource management and optimization

## 🔧 Testing

Comprehensive test suite covering:
- **Unit Tests**: Each module with comprehensive coverage
- **Integration Tests**: Module interactions and workflows
- **End-to-End Tests**: Complete system functionality
- **Performance Tests**: Benchmarking and load testing
- **Web Interface Tests**: Frontend component and integration testing
- **Monitoring Tests**: Analytics and alerting system validation
- **Governance Tests**: Proposal and voting system validation

Run tests with:
```bash
# Core system tests
cargo test
cargo test --release --test integration_tests

# Web interface tests
cd web-interface && npm test

# Comprehensive cluster tests
cd examples && ./comprehensive_cluster_test.sh
```

## 📚 Dependencies

### Backend Dependencies
- `tokio` - Async runtime for concurrent operations
- `anyhow` - Error handling and context
- `rusqlite` - SQLite database for metadata
- `rocksdb` - High-performance key-value store
- `chrono` - Date/time handling and serialization
- `serde` - Serialization framework
- `axum` - Modern web framework for REST API
- `uuid` - Unique identifier generation
- `blake3` - Fast cryptographic hashing
- `indicatif` - Progress bars and user feedback
- `colored` - Terminal colors and formatting
- `tracing` - Structured logging and observability

### Frontend Dependencies
- `vue` - Progressive JavaScript framework
- `element-plus` - Vue.js UI component library
- `pinia` - State management for Vue.js
- `chart.js` - Interactive charts and visualization
- `axios` - HTTP client for API communication
- `dayjs` - Date manipulation and formatting
- `vite` - Fast build tool and dev server

### Monitoring Dependencies
- `prometheus` - Metrics collection and monitoring
- `grafana` - Data visualization and dashboards
- `sled` - Embedded database for time-series data
- `bincode` - Binary serialization for performance
- `lru` - LRU cache implementation
- `futures` - Async utilities and combinators

### Architecture Principles

The modules are designed with these principles:
- **Async-first**: Non-blocking operations throughout
- **Error-resilient**: Graceful error handling and recovery
- **User-friendly**: Clear feedback and actionable suggestions
- **Performant**: Parallel processing and optimization
- **Maintainable**: Well-documented and thoroughly tested
- **Modular**: Loosely coupled components with clear interfaces
- **Scalable**: Designed for production workloads
- **Observable**: Comprehensive monitoring and logging
- **Secure**: Security-first design with audit trails