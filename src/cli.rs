/// Command Line Interface Module
///
/// This module defines the command-line interface for the DataMesh application using Clap.
/// It provides structured command parsing and validation for all DataMesh operations.
///
/// The CLI supports the following major commands:
/// - put: Store a file in the distributed network
/// - get: Retrieve a file from the network
/// - list: List files accessible with the current key
/// - bootstrap: Run as a bootstrap node for the DHT
/// - interactive: Run in interactive console mode
/// - service: Run as a background service
/// - config: Generate or display configuration
/// - metrics: Display performance metrics
///
/// Each command has its own set of arguments and options appropriate for its function.
use clap::{Parser, ValueEnum};
use libp2p::{Multiaddr, PeerId};
use std::path::PathBuf;

/// Output format for various commands
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Table format
    Table,
    /// JSON format
    Json,
}

/// Main CLI structure for the DataMesh application
#[derive(Parser, Debug, Clone)]
#[command(name = "datamesh")]
pub struct Cli {
    /// Optional peer ID of a bootstrap node to connect to
    #[arg(long)]
    pub bootstrap_peer: Option<PeerId>,
    /// Optional multiaddress of the bootstrap node
    #[arg(long)]
    pub bootstrap_addr: Option<Multiaddr>,
    /// Multiple bootstrap peers (format: peer_id@address)
    #[arg(long, help = "Multiple bootstrap peers (format: peer_id@address)")]
    pub bootstrap_peers: Option<Vec<String>>,
    /// Port to listen on (0 for random port)
    #[arg(long, default_value = "0")]
    pub port: u16,
    /// Network preset or custom connection (local, public, test, or peer_id@address)
    #[arg(long, help = "Network preset: local, public, test, or custom connection")]
    pub network: Option<String>,
    /// Path to the directory containing key files
    #[arg(long, help = "Path to keys directory")]
    pub keys_dir: Option<PathBuf>,
    /// Name of the specific key file to use
    #[arg(long, help = "Name of the key file to use")]
    pub key_name: Option<String>,
    /// Whether to run in non-interactive mode
    #[arg(long, help = "Non-interactive mode (auto-generate/select keys without prompts)")]
    pub non_interactive: bool,
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for the DFS application
#[derive(Parser, Debug, Clone)]
pub enum Commands {
    /// Store a file in the distributed network
    Put {
        /// Path to the file to store
        #[arg(value_name = "FILE")]
        path: PathBuf,
        /// Public key for encryption (optional)
        #[arg(long, help = "Public key hex string for encryption (if not specified, uses default key)")]
        public_key: Option<String>,
        /// Human-readable name for the file (optional)
        #[arg(long, help = "Human-readable name for the file (auto-generated if not specified)")]
        name: Option<String>,
        /// Tags to associate with the file (comma-separated)
        #[arg(long, help = "Tags to associate with the file (comma-separated)")]
        tags: Option<String>,
    },
    /// Retrieve a file from the network
    Get {
        /// File name or unique key identifying the file
        #[arg(value_name = "NAME_OR_KEY")]
        identifier: String,
        /// Path where the retrieved file will be saved
        #[arg(value_name = "OUTPUT_PATH")]
        output_path: PathBuf,
        /// Specific private key file to use for decryption
        #[arg(long, help = "Specific private key file to use for decryption")]
        private_key: Option<String>,
    },
    /// List files accessible with the current key
    List {
        /// Public key to list files for (optional)
        #[arg(long, help = "Public key hex string to list files for (if not specified, uses default key)")]
        public_key: Option<String>,
        /// Filter by tags (comma-separated)
        #[arg(long, help = "Filter files by tags (comma-separated)")]
        tags: Option<String>,
    },
    /// Run as a bootstrap node for the DHT
    Bootstrap {
        /// Port to listen on for bootstrap node
        #[arg(long, default_value = "40871")]
        port: u16,
    },
    /// Run in interactive console mode
    Interactive {
        /// Optional peer ID of a bootstrap node to connect to
        #[arg(long)]
        bootstrap_peer: Option<PeerId>,
        /// Optional multiaddress of the bootstrap node
        #[arg(long)]
        bootstrap_addr: Option<Multiaddr>,
        /// Port to listen on (0 for random port)
        #[arg(long, default_value = "0")]
        port: u16,
    },
    /// Run as a background service
    Service {
        /// Optional peer ID of a bootstrap node to connect to
        #[arg(long)]
        bootstrap_peer: Option<PeerId>,
        /// Optional multiaddress of the bootstrap node
        #[arg(long)]
        bootstrap_addr: Option<Multiaddr>,
        /// Port to listen on (0 for random port)
        #[arg(long, default_value = "0")]
        port: u16,
        /// Stop the service after a specified duration (for testing)
        #[arg(long, help = "Stop after specified seconds (for testing)")]
        timeout: Option<u64>,
    },
    /// Generate or display configuration
    Config {
        /// Generate default configuration file
        #[arg(long, help = "Generate default configuration file")]
        generate: bool,
        /// Path to the configuration file
        #[arg(long, help = "Path to configuration file")]
        config_path: Option<PathBuf>,
    },
    /// Display performance metrics
    Metrics {
        /// Show a summary of performance metrics
        #[arg(long, help = "Show performance summary")]
        summary: bool,
        /// Export metrics to a JSON file
        #[arg(long, help = "Export metrics to JSON")]
        export: bool,
    },
    /// Show detailed information about a file
    Info {
        /// File name or key to show information for
        #[arg(value_name = "NAME_OR_KEY")]
        identifier: String,
    },
    /// Show storage statistics
    Stats,
    /// List available network presets
    Networks,
    /// List and manage connected peers
    Peers {
        /// Show detailed peer information
        #[arg(long, help = "Show detailed peer information")]
        detailed: bool,
        /// Output format (json or table)
        #[arg(long, value_enum, default_value = "table", help = "Output format")]
        format: OutputFormat,
    },
    /// Analyze network topology and routing
    Network {
        /// Depth of network exploration
        #[arg(long, default_value = "2", help = "Depth of network exploration")]
        depth: u32,
        /// Visualize network structure
        #[arg(long, help = "Visualize network structure")]
        visualize: bool,
    },
    /// Discover new peers in the network
    Discover {
        /// Discovery timeout in seconds
        #[arg(long, default_value = "30", help = "Discovery timeout in seconds")]
        timeout: u64,
        /// Bootstrap from all known peers
        #[arg(long, help = "Bootstrap from all known peers")]
        bootstrap_all: bool,
    },
    /// Analyze file distribution across the network
    Distribution {
        /// Specific file key to analyze
        #[arg(long, help = "Specific file key to analyze")]
        file_key: Option<String>,
        /// Public key to filter files
        #[arg(long, help = "Public key to filter files")]
        public_key: Option<String>,
    },
    /// Monitor network health and performance
    Health {
        /// Continuous monitoring mode
        #[arg(long, help = "Continuous monitoring mode")]
        continuous: bool,
        /// Monitoring interval in seconds
        #[arg(long, default_value = "5", help = "Monitoring interval in seconds")]
        interval: u64,
    },
    /// Test network bandwidth and performance
    Bandwidth {
        /// Test bandwidth with specific peer
        #[arg(long, help = "Test bandwidth with specific peer")]
        test_peer: Option<String>,
        /// Test duration in seconds
        #[arg(long, default_value = "30", help = "Test duration in seconds")]
        duration: u64,
    },
    
    // === File Management & Operations ===
    
    /// Synchronize a local directory with the DFS network
    Sync {
        /// Local directory to synchronize
        #[arg(value_name = "LOCAL_DIR")]
        local_dir: PathBuf,
        /// Continuously monitor for changes
        #[arg(long, help = "Continuously monitor for changes")]
        watch: bool,
        /// Two-way synchronization (upload changes, download remote updates)
        #[arg(long, help = "Two-way synchronization")]
        bidirectional: bool,
        /// Exclude files matching patterns (comma-separated)
        #[arg(long, help = "Exclude files matching patterns (comma-separated)")]
        exclude: Option<String>,
        /// Number of parallel operations
        #[arg(long, default_value = "3", help = "Number of parallel operations")]
        parallel: usize,
    },
    
    /// Create versioned backups with automatic tagging
    Backup {
        /// Source directory or file to backup
        #[arg(value_name = "SOURCE")]
        source: PathBuf,
        /// Name for this backup
        #[arg(long, help = "Name for this backup")]
        name: String,
        /// Only backup changed files since last backup
        #[arg(long, help = "Incremental backup (only changed files)")]
        incremental: bool,
        /// Compress files before storing
        #[arg(long, help = "Compress files before storing")]
        compress: bool,
        /// Schedule automatic backups (cron format)
        #[arg(long, help = "Schedule automatic backups (cron format)")]
        schedule: Option<String>,
        /// Exclude files matching patterns
        #[arg(long, help = "Exclude files matching patterns (comma-separated)")]
        exclude: Option<String>,
    },
    
    /// Restore from backups with version selection
    Restore {
        /// Backup name to restore from
        #[arg(value_name = "BACKUP_NAME")]
        backup_name: String,
        /// Destination directory for restored files
        #[arg(value_name = "DESTINATION")]
        destination: PathBuf,
        /// Restore specific backup version (default: latest)
        #[arg(long, help = "Restore specific backup version")]
        version: Option<u32>,
        /// Verify integrity after restore
        #[arg(long, help = "Verify integrity after restore")]
        verify: bool,
        /// List available backup versions
        #[arg(long, help = "List available backup versions")]
        list_versions: bool,
    },
    
    /// Create copies of existing files
    Duplicate {
        /// File name or key to duplicate
        #[arg(value_name = "NAME_OR_KEY")]
        source: String,
        /// New name for the duplicate
        #[arg(long, help = "New name for the duplicate")]
        new_name: Option<String>,
        /// New tags for the duplicate
        #[arg(long, help = "New tags for the duplicate (comma-separated)")]
        new_tags: Option<String>,
    },
    
    /// Rename files without re-uploading
    Rename {
        /// Current file name
        #[arg(value_name = "OLD_NAME")]
        old_name: String,
        /// New file name
        #[arg(value_name = "NEW_NAME")]
        new_name: String,
    },
    
    // === Search & Discovery ===
    
    /// Advanced file search with multiple criteria
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        /// Filter by file type or extension
        #[arg(long, help = "Filter by file type or extension")]
        file_type: Option<String>,
        /// Size range filter (e.g., '1MB-10MB', '>5GB', '<100KB')
        #[arg(long, help = "Size range filter")]
        size: Option<String>,
        /// Date range filter (e.g., 'last week', '2024-01-01:2024-12-31')
        #[arg(long, help = "Date range filter")]
        date: Option<String>,
        /// Use regex patterns in search
        #[arg(long, help = "Use regex patterns in search")]
        regex: bool,
        /// Limit number of results
        #[arg(long, default_value = "50", help = "Limit number of results")]
        limit: usize,
    },
    
    /// Show recently uploaded/accessed files
    Recent {
        /// Number of files to show
        #[arg(long, default_value = "20", help = "Number of files to show")]
        count: usize,
        /// Show files from last N days
        #[arg(long, default_value = "7", help = "Show files from last N days")]
        days: u32,
        /// Filter by file type
        #[arg(long, help = "Filter by file type")]
        file_type: Option<String>,
    },
    
    /// Show most frequently accessed files
    Popular {
        /// Time frame for popularity calculation
        #[arg(long, default_value = "week", help = "Time frame (day, week, month, year)")]
        timeframe: String,
        /// Number of files to show
        #[arg(long, default_value = "20", help = "Number of files to show")]
        count: usize,
    },
    
    // === Batch Operations ===
    
    /// Upload multiple files matching patterns
    BatchPut {
        /// File pattern to match
        #[arg(value_name = "PATTERN")]
        pattern: String,
        /// Include subdirectories
        #[arg(long, help = "Include subdirectories")]
        recursive: bool,
        /// Number of concurrent uploads
        #[arg(long, default_value = "3", help = "Number of concurrent uploads")]
        parallel: usize,
        /// Base directory for relative paths
        #[arg(long, help = "Base directory for relative paths")]
        base_dir: Option<PathBuf>,
        /// Tag pattern (use {name}, {ext}, {path} placeholders)
        #[arg(long, help = "Tag pattern with placeholders")]
        tag_pattern: Option<String>,
    },
    
    /// Download multiple files to local directory
    BatchGet {
        /// File pattern to match
        #[arg(value_name = "PATTERN")]
        pattern: String,
        /// Destination directory
        #[arg(value_name = "DESTINATION")]
        destination: PathBuf,
        /// Number of concurrent downloads
        #[arg(long, default_value = "3", help = "Number of concurrent downloads")]
        parallel: usize,
        /// Preserve directory structure
        #[arg(long, help = "Preserve directory structure")]
        preserve_structure: bool,
    },
    
    /// Bulk tag operations
    BatchTag {
        /// File pattern to match
        #[arg(value_name = "PATTERN")]
        pattern: String,
        /// Tags to add (comma-separated)
        #[arg(long, help = "Tags to add (comma-separated)")]
        add_tags: Option<String>,
        /// Tags to remove (comma-separated)
        #[arg(long, help = "Tags to remove (comma-separated)")]
        remove_tags: Option<String>,
        /// Show matching files without making changes
        #[arg(long, help = "Show matching files without making changes")]
        dry_run: bool,
    },
    
    // === Health & Maintenance ===
    
    /// Repair corrupted or low-redundancy files
    Repair {
        /// Specific file name or key to repair
        #[arg(value_name = "NAME_OR_KEY")]
        target: Option<String>,
        /// Automatically repair all files below threshold
        #[arg(long, help = "Automatically repair all files below threshold")]
        auto: bool,
        /// Verify integrity of all files
        #[arg(long, help = "Verify integrity of all files")]
        verify_all: bool,
        /// Minimum health threshold for auto repair
        #[arg(long, default_value = "50", help = "Minimum health threshold for auto repair")]
        threshold: u8,
    },
    
    /// Clean up storage and optimize database
    Cleanup {
        /// Remove orphaned chunks and metadata
        #[arg(long, help = "Remove orphaned chunks and metadata")]
        orphaned: bool,
        /// Find and handle duplicate files
        #[arg(long, help = "Find and handle duplicate files")]
        duplicates: bool,
        /// Remove files with irreparable low health
        #[arg(long, help = "Remove files with irreparable low health")]
        low_health: bool,
        /// Show what would be cleaned without doing it
        #[arg(long, help = "Show what would be cleaned without doing it")]
        dry_run: bool,
        /// Force cleanup without confirmation
        #[arg(long, help = "Force cleanup without confirmation")]
        force: bool,
    },
    
    /// Manage storage quotas and usage
    Quota {
        /// Show current usage
        #[arg(long, help = "Show current usage")]
        usage: bool,
        /// Set storage limit
        #[arg(long, help = "Set storage limit (e.g., 100GB, 1TB)")]
        limit: Option<String>,
        /// Warning threshold percentage
        #[arg(long, help = "Warning threshold percentage")]
        warn: Option<u8>,
    },
    
    // === Import/Export ===
    
    /// Export files to standard archive formats
    Export {
        /// Destination file for export
        #[arg(value_name = "DESTINATION")]
        destination: PathBuf,
        /// Archive format (tar, zip)
        #[arg(long, default_value = "tar", help = "Archive format")]
        format: String,
        /// Encrypt the export archive
        #[arg(long, help = "Encrypt the export archive")]
        encrypt: bool,
        /// Include DFS metadata (tags, health status)
        #[arg(long, help = "Include DFS metadata")]
        include_metadata: bool,
        /// File pattern to export
        #[arg(long, help = "File pattern to export (default: all files)")]
        pattern: Option<String>,
    },
    
    /// Import from standard archive formats
    Import {
        /// Archive file to import
        #[arg(value_name = "ARCHIVE")]
        archive: PathBuf,
        /// Verify files after import
        #[arg(long, help = "Verify files after import")]
        verify: bool,
        /// Maintain directory structure as tags
        #[arg(long, help = "Maintain directory structure as tags")]
        preserve_structure: bool,
        /// Tag prefix for imported files
        #[arg(long, help = "Tag prefix for imported files")]
        tag_prefix: Option<String>,
    },
    
    // === Quick Actions ===
    
    /// Pin important files for guaranteed availability
    Pin {
        /// File name or key to pin
        #[arg(value_name = "NAME_OR_KEY")]
        target: String,
        /// Pin duration (e.g., '30 days', '1 week', 'permanent')
        #[arg(long, help = "Pin duration")]
        duration: Option<String>,
        /// Pin priority (1-10, higher = more important)
        #[arg(long, default_value = "5", help = "Pin priority")]
        priority: u8,
    },
    
    /// Remove pin from files
    Unpin {
        /// File name or key to unpin
        #[arg(value_name = "NAME_OR_KEY")]
        target: String,
    },
    
    /// Generate sharing links or keys
    Share {
        /// File name or key to share
        #[arg(value_name = "NAME_OR_KEY")]
        target: String,
        /// Create public (unencrypted) share
        #[arg(long, help = "Create public (unencrypted) share")]
        public: bool,
        /// Set expiration time
        #[arg(long, help = "Set expiration time (e.g., '1 week', '30 days')")]
        expires: Option<String>,
        /// Password protect the share
        #[arg(long, help = "Password protect the share")]
        password: Option<String>,
        /// Generate QR code for share link
        #[arg(long, help = "Generate QR code for share link")]
        qr_code: bool,
    },
    
    // === Performance & Optimization ===
    
    /// Optimize storage and network performance
    Optimize {
        /// Defragment local database
        #[arg(long, help = "Defragment local database")]
        defrag: bool,
        /// Rebalance chunk distribution
        #[arg(long, help = "Rebalance chunk distribution")]
        rebalance: bool,
        /// Compress rarely accessed files
        #[arg(long, help = "Compress rarely accessed files")]
        compress: bool,
        /// Show optimization recommendations
        #[arg(long, help = "Show optimization recommendations")]
        analyze: bool,
    },
    
    /// Run comprehensive performance benchmarks
    Benchmark {
        /// Run full benchmark suite
        #[arg(long, help = "Run full benchmark suite")]
        full: bool,
        /// Test network performance only
        #[arg(long, help = "Test network performance only")]
        network: bool,
        /// Test storage performance only
        #[arg(long, help = "Test storage performance only")]
        storage: bool,
        /// Duration for each test in seconds
        #[arg(long, default_value = "30", help = "Duration for each test")]
        duration: u64,
    },
    
    // === API Server ===
    
    /// Start the REST API server
    ApiServer {
        /// Server host address
        #[arg(long, default_value = "127.0.0.1", help = "Server host address")]
        host: Option<String>,
        /// Server port
        #[arg(long, default_value = "8080", help = "Server port")]
        port: Option<u16>,
        /// Enable HTTPS
        #[arg(long, help = "Enable HTTPS")]
        https: bool,
        /// Path to TLS certificate file
        #[arg(long, help = "Path to TLS certificate file")]
        cert_path: Option<PathBuf>,
        /// Path to TLS private key file
        #[arg(long, help = "Path to TLS private key file")]
        key_path: Option<PathBuf>,
        /// Disable Swagger UI
        #[arg(long, help = "Disable Swagger UI")]
        no_swagger: bool,
    },
}

impl Cli {
    /// Parse command-line arguments and return a Cli instance
    pub fn parse() -> Self {
        Parser::parse()
    }

    /// Parse bootstrap peers from CLI format (peer_id@address)
    pub fn parse_bootstrap_peers(&self) -> Result<Vec<crate::bootstrap_manager::BootstrapPeer>, Box<dyn std::error::Error>> {
        use crate::bootstrap_manager::BootstrapPeer;
        use std::str::FromStr;
        
        let mut peers = Vec::new();
        
        if let Some(ref bootstrap_peers) = self.bootstrap_peers {
            for peer_str in bootstrap_peers {
                let parts: Vec<&str> = peer_str.split('@').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid bootstrap peer format '{}'. Expected format: peer_id@address", peer_str).into());
                }
                
                let peer_id = PeerId::from_str(parts[0])?;
                let address = Multiaddr::from_str(parts[1])?;
                
                let peer = BootstrapPeer::new(peer_id, vec![address])
                    .with_priority(1); // CLI peers get high priority
                
                peers.push(peer);
            }
        }
        
        Ok(peers)
    }

    /// Get all bootstrap peers (CLI format + individual peer/addr)
    pub fn get_all_bootstrap_peers(&self) -> Result<Vec<crate::bootstrap_manager::BootstrapPeer>, Box<dyn std::error::Error>> {
        use crate::bootstrap_manager::BootstrapPeer;
        
        let mut peers = self.parse_bootstrap_peers()?;
        
        // Add individual peer/addr if specified
        if let (Some(peer_id), Some(addr)) = (self.bootstrap_peer, self.bootstrap_addr.clone()) {
            let peer = BootstrapPeer::new(peer_id, vec![addr])
                .with_priority(1); // CLI peers get high priority
            peers.push(peer);
        }
        
        Ok(peers)
    }
}
