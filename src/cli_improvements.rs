/// CLI UX Improvements Module
///
/// This module contains enhancements for the DataMesh CLI user experience.
/// It addresses common usability issues and provides better command organization,
/// output formatting, and user guidance.

use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Output format options for CLI commands
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format (default)
    Table,
    /// Machine-readable JSON format
    Json,
    /// Compact single-line format
    Compact,
    /// CSV format for data export
    Csv,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
}

/// Verbosity levels for CLI output
#[derive(Debug, Clone, ValueEnum)]
pub enum VerbosityLevel {
    /// Only show errors
    Quiet,
    /// Normal output
    Normal,
    /// Verbose output with additional details
    Verbose,
    /// Debug output with full details
    Debug,
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        VerbosityLevel::Normal
    }
}

/// Enhanced CLI structure with improved UX
#[derive(Parser, Debug, Clone)]
#[command(
    name = "datamesh",
    version = env!("CARGO_PKG_VERSION"),
    about = "DataMesh - Distributed File Storage System",
    long_about = "DataMesh is a distributed file storage system built on libp2p with encryption, redundancy, and governance features."
)]
pub struct EnhancedCli {
    /// Global output format
    #[arg(
        long,
        short = 'f',
        global = true,
        help = "Output format"
    )]
    pub format: Option<OutputFormat>,
    
    /// Global verbosity level
    #[arg(
        long,
        short = 'v',
        global = true,
        help = "Verbosity level",
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,
    
    /// Disable colored output
    #[arg(
        long,
        global = true,
        help = "Disable colored output"
    )]
    pub no_color: bool,
    
    /// Configuration file path
    #[arg(
        long,
        short = 'c',
        global = true,
        help = "Path to configuration file"
    )]
    pub config: Option<String>,
    
    /// Enable interactive confirmations
    #[arg(
        long,
        short = 'i',
        global = true,
        help = "Enable interactive confirmations"
    )]
    pub interactive: bool,
    
    /// Dry run mode (show what would be done)
    #[arg(
        long,
        global = true,
        help = "Show what would be done without executing"
    )]
    pub dry_run: bool,
    
    #[command(subcommand)]
    pub command: EnhancedCommands,
}

/// Enhanced command structure with better organization
#[derive(Subcommand, Debug, Clone)]
pub enum EnhancedCommands {
    /// File operations
    #[command(subcommand)]
    File(FileCommands),
    
    /// Network operations
    #[command(subcommand)]
    Network(NetworkCommands),
    
    /// System operations
    #[command(subcommand)]
    System(SystemCommands),
    
    /// User and governance operations
    #[command(subcommand)]
    Governance(GovernanceCommands),
    
    /// Quick access to common operations
    #[command(flatten)]
    Quick(QuickCommands),
}

/// File-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum FileCommands {
    /// Store a file in the network
    Put {
        /// Path to the file to store
        path: String,
        /// Custom name for the file
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// Tags to associate with the file
        #[arg(long, short = 't')]
        tags: Option<Vec<String>>,
        /// Public key for encryption
        #[arg(long)]
        public_key: Option<String>,
    },
    
    /// Retrieve a file from the network
    Get {
        /// File identifier or name
        identifier: String,
        /// Output path
        #[arg(short = 'o', long)]
        output: Option<String>,
        /// Private key for decryption
        #[arg(long)]
        private_key: Option<String>,
    },
    
    /// List files
    List {
        /// Filter by tags
        #[arg(long, short = 't')]
        tags: Option<Vec<String>>,
        /// Public key to list files for
        #[arg(long)]
        public_key: Option<String>,
        /// Show detailed information
        #[arg(long, short = 'l')]
        long: bool,
    },
    
    /// Search files
    Search {
        /// Search query
        query: String,
        /// Search in content (not just names)
        #[arg(long)]
        content: bool,
        /// Limit results
        #[arg(long, short = 'n', default_value = "10")]
        limit: usize,
    },
    
    /// Batch operations
    Batch {
        /// Operation type
        #[command(subcommand)]
        operation: BatchOperation,
    },
    
    /// File sharing and permissions
    Share {
        /// File identifier
        file: String,
        /// Share with specific users
        #[arg(long)]
        with: Option<Vec<String>>,
        /// Create public link
        #[arg(long)]
        public: bool,
        /// Set expiration time
        #[arg(long)]
        expires: Option<String>,
    },
}

/// Batch operation types
#[derive(Subcommand, Debug, Clone)]
pub enum BatchOperation {
    /// Upload multiple files
    Put {
        /// File patterns to upload
        patterns: Vec<String>,
        /// Preserve directory structure
        #[arg(long)]
        preserve_structure: bool,
    },
    /// Download multiple files
    Get {
        /// File patterns to download
        patterns: Vec<String>,
        /// Output directory
        #[arg(short = 'o', long)]
        output_dir: String,
    },
    /// Tag multiple files
    Tag {
        /// File patterns
        patterns: Vec<String>,
        /// Tags to add
        #[arg(long)]
        add: Option<Vec<String>>,
        /// Tags to remove
        #[arg(long)]
        remove: Option<Vec<String>>,
    },
}

/// Network-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum NetworkCommands {
    /// Show peer information
    Peers {
        /// Show detailed peer information
        #[arg(long, short = 'l')]
        long: bool,
        /// Filter by peer status
        #[arg(long)]
        status: Option<String>,
    },
    
    /// Network health and diagnostics
    Health {
        /// Run comprehensive health check
        #[arg(long)]
        full: bool,
        /// Monitor continuously
        #[arg(long, short = 'm')]
        monitor: bool,
    },
    
    /// Network topology analysis
    Topology {
        /// Show routing table
        #[arg(long)]
        routing: bool,
        /// Export to file
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    
    /// Bandwidth testing
    Bandwidth {
        /// Target peer for testing
        peer: Option<String>,
        /// Test duration in seconds
        #[arg(long, short = 'd', default_value = "10")]
        duration: u64,
    },
    
    /// Bootstrap node management
    Bootstrap {
        /// Bootstrap operation
        #[command(subcommand)]
        operation: BootstrapOperation,
    },
}

/// Bootstrap operations
#[derive(Subcommand, Debug, Clone)]
pub enum BootstrapOperation {
    /// Start bootstrap node
    Start {
        /// Port to listen on
        #[arg(long, short = 'p', default_value = "40871")]
        port: u16,
    },
    /// Stop bootstrap node
    Stop,
    /// List available bootstrap nodes
    List,
    /// Add bootstrap node
    Add {
        /// Peer address
        address: String,
    },
    /// Remove bootstrap node
    Remove {
        /// Peer ID
        peer_id: String,
    },
}

/// System-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum SystemCommands {
    /// Configuration management
    Config {
        #[command(subcommand)]
        operation: ConfigOperation,
    },
    
    /// Statistics and metrics
    Stats {
        /// Show detailed statistics
        #[arg(long, short = 'l')]
        long: bool,
        /// Watch mode (continuous updates)
        #[arg(long, short = 'w')]
        watch: bool,
    },
    
    /// Storage management
    Storage {
        #[command(subcommand)]
        operation: StorageOperation,
    },
    
    /// API server management
    Api {
        #[command(subcommand)]
        operation: ApiOperation,
    },
    
    /// Run system benchmarks
    Benchmark {
        /// Benchmark type
        #[arg(long, short = 't')]
        test_type: Option<Vec<String>>,
        /// Duration in seconds
        #[arg(long, short = 'd', default_value = "60")]
        duration: u64,
    },
}

/// Configuration operations
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigOperation {
    /// Show current configuration
    Show {
        /// Show only specific section
        section: Option<String>,
    },
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Generate default configuration
    Init {
        /// Output file
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },
    /// Validate configuration
    Validate {
        /// Configuration file to validate
        file: Option<String>,
    },
}

/// Storage operations
#[derive(Subcommand, Debug, Clone)]
pub enum StorageOperation {
    /// Clean up storage
    Cleanup {
        /// Remove orphaned files
        #[arg(long)]
        orphaned: bool,
        /// Compact database
        #[arg(long)]
        compact: bool,
    },
    /// Repair storage
    Repair {
        /// Check file integrity
        #[arg(long)]
        integrity: bool,
        /// Repair corrupted files
        #[arg(long)]
        fix: bool,
    },
    /// Optimize storage
    Optimize {
        /// Defragment storage
        #[arg(long)]
        defrag: bool,
        /// Rebalance chunks
        #[arg(long)]
        rebalance: bool,
    },
    /// Show quota information
    Quota {
        /// Show detailed quota breakdown
        #[arg(long, short = 'l')]
        long: bool,
    },
}

/// API server operations
#[derive(Subcommand, Debug, Clone)]
pub enum ApiOperation {
    /// Start API server
    Start {
        /// Port to listen on
        #[arg(long, short = 'p', default_value = "8080")]
        port: u16,
        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },
    /// Stop API server
    Stop,
    /// Show API server status
    Status,
    /// Generate API documentation
    Docs {
        /// Output format
        #[arg(long, default_value = "openapi")]
        format: String,
        /// Output file
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
}

/// Governance-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum GovernanceCommands {
    /// User management
    User {
        #[command(subcommand)]
        operation: UserOperation,
    },
    
    /// Proposal management
    Proposal {
        #[command(subcommand)]
        operation: ProposalOperation,
    },
    
    /// Voting operations
    Vote {
        /// Proposal ID
        proposal_id: String,
        /// Vote (yes/no/abstain)
        vote: String,
        /// Voting reason
        #[arg(long)]
        reason: Option<String>,
    },
    
    /// Network economics
    Economics {
        #[command(subcommand)]
        operation: EconomicsOperation,
    },
}

/// User operations
#[derive(Subcommand, Debug, Clone)]
pub enum UserOperation {
    /// Register new user
    Register {
        /// Email address
        email: String,
        /// User password
        #[arg(long)]
        password: Option<String>,
    },
    /// Login user
    Login {
        /// Email address
        email: String,
        /// User password
        #[arg(long)]
        password: Option<String>,
    },
    /// Show user profile
    Profile {
        /// User ID (defaults to current user)
        user_id: Option<String>,
    },
    /// Update user profile
    Update {
        /// New email
        #[arg(long)]
        email: Option<String>,
        /// Change password
        #[arg(long)]
        password: bool,
    },
}

/// Proposal operations
#[derive(Subcommand, Debug, Clone)]
pub enum ProposalOperation {
    /// List proposals
    List {
        /// Show only active proposals
        #[arg(long)]
        active: bool,
        /// Filter by type
        #[arg(long)]
        proposal_type: Option<String>,
    },
    /// Create new proposal
    Create {
        /// Proposal title
        title: String,
        /// Proposal description
        description: String,
        /// Proposal type
        #[arg(long)]
        proposal_type: String,
    },
    /// Show proposal details
    Show {
        /// Proposal ID
        proposal_id: String,
    },
}

/// Economics operations
#[derive(Subcommand, Debug, Clone)]
pub enum EconomicsOperation {
    /// Show balance
    Balance {
        /// User ID (defaults to current user)
        user_id: Option<String>,
    },
    /// Transfer tokens
    Transfer {
        /// Recipient user ID
        to: String,
        /// Amount to transfer
        amount: f64,
        /// Transfer memo
        #[arg(long)]
        memo: Option<String>,
    },
    /// Stake tokens
    Stake {
        /// Amount to stake
        amount: f64,
        /// Staking duration
        #[arg(long)]
        duration: Option<String>,
    },
    /// Show transaction history
    History {
        /// Number of transactions to show
        #[arg(long, short = 'n', default_value = "10")]
        limit: usize,
    },
}

/// Quick access commands (commonly used operations)
#[derive(Parser, Debug, Clone)]
pub struct QuickCommands {
    /// Interactive mode
    #[command(subcommand)]
    pub quick: Option<QuickOperation>,
}

/// Quick operations
#[derive(Subcommand, Debug, Clone)]
pub enum QuickOperation {
    /// Start interactive shell
    Interactive,
    /// Run as service
    Service {
        /// Service operation
        #[command(subcommand)]
        operation: ServiceOperation,
    },
    /// Show system status
    Status,
    /// Show help for getting started
    Guide {
        /// Guide topic
        topic: Option<String>,
    },
}

/// Service operations
#[derive(Subcommand, Debug, Clone)]
pub enum ServiceOperation {
    /// Start service
    Start {
        /// Run in foreground
        #[arg(long)]
        foreground: bool,
    },
    /// Stop service
    Stop,
    /// Restart service
    Restart,
    /// Show service status
    Status,
    /// Show service logs
    Logs {
        /// Follow logs
        #[arg(long, short = 'f')]
        follow: bool,
        /// Number of lines to show
        #[arg(long, short = 'n', default_value = "100")]
        lines: usize,
    },
}

/// Command completion suggestions
pub struct CommandCompletion {
    commands: Vec<String>,
    descriptions: HashMap<String, String>,
}

impl CommandCompletion {
    /// Create new command completion helper
    pub fn new() -> Self {
        let mut commands = Vec::new();
        let mut descriptions = HashMap::new();
        
        // Core commands
        commands.extend_from_slice(&[
            "file", "network", "system", "governance",
            "interactive", "service", "status", "guide"
        ]);
        
        // File subcommands
        commands.extend_from_slice(&[
            "file put", "file get", "file list", "file search", "file batch", "file share"
        ]);
        
        // Network subcommands
        commands.extend_from_slice(&[
            "network peers", "network health", "network topology", "network bandwidth", "network bootstrap"
        ]);
        
        // System subcommands
        commands.extend_from_slice(&[
            "system config", "system stats", "system storage", "system api", "system benchmark"
        ]);
        
        // Add descriptions
        descriptions.insert("file".to_string(), "File operations (put, get, list, search)".to_string());
        descriptions.insert("network".to_string(), "Network operations (peers, health, topology)".to_string());
        descriptions.insert("system".to_string(), "System operations (config, stats, storage)".to_string());
        descriptions.insert("governance".to_string(), "Governance operations (users, proposals, voting)".to_string());
        descriptions.insert("interactive".to_string(), "Start interactive shell".to_string());
        descriptions.insert("service".to_string(), "Service management".to_string());
        descriptions.insert("status".to_string(), "Show system status".to_string());
        descriptions.insert("guide".to_string(), "Getting started guide".to_string());
        
        Self {
            commands,
            descriptions,
        }
    }
    
    /// Get command suggestions based on input
    pub fn suggest(&self, input: &str) -> Vec<(String, String)> {
        let input_lower = input.to_lowercase();
        
        self.commands
            .iter()
            .filter(|cmd| cmd.to_lowercase().starts_with(&input_lower))
            .map(|cmd| {
                let desc = self.descriptions
                    .get(*cmd)
                    .cloned()
                    .unwrap_or_else(|| "No description available".to_string());
                (cmd.clone(), desc)
            })
            .take(10)
            .collect()
    }
    
    /// Get fuzzy command suggestions using Levenshtein distance
    pub fn fuzzy_suggest(&self, input: &str) -> Vec<(String, String, usize)> {
        let mut suggestions: Vec<_> = self.commands
            .iter()
            .map(|cmd| {
                let distance = levenshtein_distance(input, cmd);
                let desc = self.descriptions
                    .get(*cmd)
                    .cloned()
                    .unwrap_or_else(|| "No description available".to_string());
                (cmd.clone(), desc, distance)
            })
            .filter(|(_, _, distance)| *distance <= 3) // Only show close matches
            .collect();
        
        suggestions.sort_by_key(|(_, _, distance)| *distance);
        suggestions.truncate(5);
        suggestions
    }
}

/// Simple Levenshtein distance calculation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }
    
    matrix[len1][len2]
}

/// Enhanced output formatter
pub struct OutputFormatter {
    format: OutputFormat,
    no_color: bool,
    verbosity: VerbosityLevel,
}

impl OutputFormatter {
    /// Create new output formatter
    pub fn new(format: OutputFormat, no_color: bool, verbosity: VerbosityLevel) -> Self {
        Self {
            format,
            no_color,
            verbosity,
        }
    }
    
    /// Format output based on configured format
    pub fn format_output(&self, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
        match self.format {
            OutputFormat::Json => {
                Ok(serde_json::to_string_pretty(data)?)
            },
            OutputFormat::Table => {
                self.format_table(data)
            },
            OutputFormat::Compact => {
                self.format_compact(data)
            },
            OutputFormat::Csv => {
                self.format_csv(data)
            },
        }
    }
    
    /// Format data as table
    fn format_table(&self, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
        // Table formatting implementation
        match data {
            Value::Array(items) => {
                if items.is_empty() {
                    return Ok("No data available".to_string());
                }
                
                // Extract headers from first item
                let headers = match &items[0] {
                    Value::Object(obj) => obj.keys().cloned().collect::<Vec<_>>(),
                    _ => return Ok(serde_json::to_string_pretty(data)?),
                };
                
                let mut table = String::new();
                
                // Header row
                table.push_str(&format!("{}\n", headers.join(" | ")));
                table.push_str(&format!("{}\n", "-".repeat(headers.len() * 10)));
                
                // Data rows
                for item in items {
                    if let Value::Object(obj) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| {
                                obj.get(h)
                                    .map(|v| format_value(v))
                                    .unwrap_or_else(|| "-".to_string())
                            })
                            .collect();
                        table.push_str(&format!("{}\n", row.join(" | ")));
                    }
                }
                
                Ok(table)
            },
            Value::Object(_) => {
                // Single object as key-value pairs
                Ok(format_key_value(data))
            },
            _ => Ok(data.to_string()),
        }
    }
    
    /// Format data in compact mode
    fn format_compact(&self, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            Value::Array(items) => {
                let compact: Vec<String> = items
                    .iter()
                    .map(|item| {
                        if let Value::Object(obj) = item {
                            // Show first few key fields
                            let key_fields = ["id", "name", "status", "type"];
                            let summary: Vec<String> = key_fields
                                .iter()
                                .filter_map(|&field| {
                                    obj.get(field).map(|v| format!("{}:{}", field, format_value(v)))
                                })
                                .collect();
                            summary.join(" ")
                        } else {
                            format_value(item)
                        }
                    })
                    .collect();
                Ok(compact.join("\n"))
            },
            _ => Ok(format_value(data)),
        }
    }
    
    /// Format data as CSV
    fn format_csv(&self, data: &Value) -> Result<String, Box<dyn std::error::Error>> {
        match data {
            Value::Array(items) => {
                if items.is_empty() {
                    return Ok(String::new());
                }
                
                // Extract headers
                let headers = match &items[0] {
                    Value::Object(obj) => obj.keys().cloned().collect::<Vec<_>>(),
                    _ => return Err("Cannot format non-object array as CSV".into()),
                };
                
                let mut csv = String::new();
                csv.push_str(&format!("{}\n", headers.join(",")));
                
                for item in items {
                    if let Value::Object(obj) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| {
                                obj.get(h)
                                    .map(|v| format_csv_value(v))
                                    .unwrap_or_else(|| String::new())
                            })
                            .collect();
                        csv.push_str(&format!("{}\n", row.join(",")));
                    }
                }
                
                Ok(csv)
            },
            _ => Err("Cannot format non-array as CSV".into()),
        }
    }
    
    /// Apply color formatting if enabled
    pub fn colorize(&self, text: &str, color: &str) -> String {
        if self.no_color {
            text.to_string()
        } else {
            match color {
                "red" => format!("\x1b[31m{}\x1b[0m", text),
                "green" => format!("\x1b[32m{}\x1b[0m", text),
                "yellow" => format!("\x1b[33m{}\x1b[0m", text),
                "blue" => format!("\x1b[34m{}\x1b[0m", text),
                "bold" => format!("\x1b[1m{}\x1b[0m", text),
                _ => text.to_string(),
            }
        }
    }
}

/// Format a JSON value as string
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

/// Format a JSON value for CSV (with proper escaping)
fn format_csv_value(value: &Value) -> String {
    let s = format_value(value);
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

/// Format object as key-value pairs
fn format_key_value(value: &Value) -> String {
    match value {
        Value::Object(obj) => {
            let mut result = String::new();
            for (key, val) in obj {
                result.push_str(&format!("{}: {}\n", key, format_value(val)));
            }
            result
        },
        _ => format_value(value),
    }
}

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    message: String,
    spinner_chars: Vec<char>,
    current_frame: usize,
}

impl ProgressIndicator {
    /// Create new progress indicator
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            spinner_chars: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_frame: 0,
        }
    }
    
    /// Update progress display
    pub fn tick(&mut self) {
        print!("\r{} {}", 
               self.spinner_chars[self.current_frame], 
               self.message);
        self.current_frame = (self.current_frame + 1) % self.spinner_chars.len();
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    
    /// Finish progress with success message
    pub fn finish_with_message(&self, message: &str) {
        println!("\r✓ {}", message);
    }
    
    /// Finish progress with error message
    pub fn finish_with_error(&self, message: &str) {
        println!("\r✗ {}", message);
    }
}

/// CLI help improvement with examples and tips
pub struct HelpBuilder {
    command: String,
    examples: Vec<(String, String)>,
    tips: Vec<String>,
    see_also: Vec<String>,
}

impl HelpBuilder {
    /// Create new help builder
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            examples: Vec::new(),
            tips: Vec::new(),
            see_also: Vec::new(),
        }
    }
    
    /// Add example usage
    pub fn example(mut self, command: &str, description: &str) -> Self {
        self.examples.push((command.to_string(), description.to_string()));
        self
    }
    
    /// Add tip
    pub fn tip(mut self, tip: &str) -> Self {
        self.tips.push(tip.to_string());
        self
    }
    
    /// Add related command
    pub fn see_also(mut self, command: &str) -> Self {
        self.see_also.push(command.to_string());
        self
    }
    
    /// Build enhanced help text
    pub fn build(&self) -> String {
        let mut help = String::new();
        
        if !self.examples.is_empty() {
            help.push_str("\nEXAMPLES:\n");
            for (cmd, desc) in &self.examples {
                help.push_str(&format!("  {} - {}\n", cmd, desc));
            }
        }
        
        if !self.tips.is_empty() {
            help.push_str("\nTIPS:\n");
            for tip in &self.tips {
                help.push_str(&format!("  • {}\n", tip));
            }
        }
        
        if !self.see_also.is_empty() {
            help.push_str("\nSEE ALSO:\n");
            help.push_str(&format!("  {}\n", self.see_also.join(", ")));
        }
        
        help
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "world"), 4);
    }
    
    #[test]
    fn test_command_completion() {
        let completion = CommandCompletion::new();
        let suggestions = completion.suggest("file");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|(cmd, _)| cmd.starts_with("file")));
    }
    
    #[test]
    fn test_output_formatter() {
        let formatter = OutputFormatter::new(
            OutputFormat::Json,
            false,
            VerbosityLevel::Normal
        );
        
        let data = serde_json::json!({
            "name": "test",
            "value": 42
        });
        
        let result = formatter.format_output(&data);
        assert!(result.is_ok());
    }
}