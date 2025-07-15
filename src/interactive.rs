/// Interactive Mode Module
///
/// This module provides interactive and service mode functionality for the DFS application.
/// It implements:
/// - An interactive console for user commands
/// - A service mode for background operation
/// - Command parsing and execution
/// - Network event handling
///
/// The interactive mode provides a command-line interface for users to perform
/// file operations, check network status, and manage keys in real-time.
///
/// The service mode runs DFS as a background process, maintaining DHT connectivity
/// and providing persistent storage capabilities.
use anyhow::Result;
use futures::stream::StreamExt;
use libp2p::kad::{Event as KademliaEvent, GetRecordOk, QueryResult};
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::cli::Cli;
use crate::config::Config;
use crate::database::{self, DatabaseManager};
use crate::error_handling;
use crate::file_storage::{FileRetrieval, StoredFile, PUB_DATA_SHARDS, PUB_PARITY_SHARDS};
use crate::key_manager::{get_default_keys_dir, KeyManager};
use crate::network::{create_swarm_and_connect_multi_bootstrap, MyBehaviourEvent};
use crate::network_diagnostics;
use crate::ui;
// Essential UX enhancement functions are defined inline below

/// Number of data shards for Reed-Solomon erasure coding
const DATA_SHARDS: usize = 4;

/// Simple Levenshtein distance implementation for typo suggestions
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

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Parsed command structure
#[derive(Debug, Default)]
struct ParsedCommand {
    command: String,
    args: Vec<String>,
    flags: HashMap<String, String>,
    errors: Vec<String>,
    suggestions: Vec<String>,
}

/// Smart command parser with validation and suggestions
fn parse_command_smart(input: &str, valid_commands: &[&str]) -> ParsedCommand {
    let mut parsed = ParsedCommand::default();

    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() {
        parsed.errors.push("Empty command".to_string());
        return parsed;
    }

    parsed.command = parts[0].to_string();

    // Validate command exists
    if !valid_commands.contains(&parsed.command.as_str()) {
        let suggestions = suggest_similar_commands(&parsed.command, valid_commands);
        if !suggestions.is_empty() {
            parsed.errors.push(format!(
                "Unknown command '{}'. Did you mean: {}?",
                parsed.command,
                suggestions.join(", ")
            ));
            parsed.suggestions = suggestions;
        } else {
            parsed.errors.push(format!(
                "Unknown command '{}'. Type 'help' for available commands.",
                parsed.command
            ));
        }
        return parsed;
    }

    // Parse arguments and flags
    let mut i = 1;
    while i < parts.len() {
        if parts[i].starts_with("--") {
            if i + 1 < parts.len() && !parts[i + 1].starts_with("--") {
                parsed
                    .flags
                    .insert(parts[i][2..].to_string(), parts[i + 1].to_string());
                i += 2;
            } else {
                parsed
                    .flags
                    .insert(parts[i][2..].to_string(), "true".to_string());
                i += 1;
            }
        } else {
            parsed.args.push(parts[i].to_string());
            i += 1;
        }
    }

    // Validate command-specific requirements
    validate_command_requirements(&mut parsed);

    parsed
}

/// Suggest similar commands based on Levenshtein distance
fn suggest_similar_commands(input: &str, valid_commands: &[&str]) -> Vec<String> {
    valid_commands
        .iter()
        .filter(|cmd| levenshtein_distance(cmd, input) <= 2)
        .map(|s| s.to_string())
        .collect()
}

/// Validate command-specific requirements
fn validate_command_requirements(parsed: &mut ParsedCommand) {
    match parsed.command.as_str() {
        "put" => {
            if parsed.args.is_empty() {
                parsed
                    .errors
                    .push("Put command requires a file path".to_string());
                parsed
                    .suggestions
                    .push("Usage: put <file> [--name <alias>] [--tags <tag1,tag2>]".to_string());
            } else if !std::path::Path::new(&parsed.args[0]).exists() {
                parsed
                    .errors
                    .push(format!("File '{}' does not exist", parsed.args[0]));
                parsed
                    .suggestions
                    .push("Check the file path and ensure the file exists".to_string());
            }
        }
        "get" => {
            if parsed.args.len() < 2 {
                parsed
                    .errors
                    .push("Get command requires file identifier and output path".to_string());
                parsed
                    .suggestions
                    .push("Usage: get <name_or_key> <output_path>".to_string());
            }
        }
        "info" => {
            if parsed.args.is_empty() {
                parsed
                    .errors
                    .push("Info command requires a file identifier".to_string());
                parsed
                    .suggestions
                    .push("Usage: info <name_or_key>".to_string());
            }
        }
        _ => {}
    }
}

/// Enhanced interactive welcome message
fn print_interactive_welcome_enhanced(peer_id: &str, public_key: &str) {
    use colored::*;
    ui::print_header("🚀 DataMesh Interactive Console");

    ui::print_key_value("Peer ID", &format!("{}...", &peer_id[..16]));
    ui::print_key_value("Public Key", &format!("{}...", &public_key[..16]));

    ui::print_section("🗂️ File Operations");
    println!(
        "  {} - Store files with encryption",
        "put <file>".green().bold()
    );
    println!(
        "  {} - Retrieve files by name/key",
        "get <name> <output>".green().bold()
    );
    println!("  {} - Browse your stored files", "list".green().bold());

    ui::print_section("📊 Information & Status");
    println!("  {} - Show file details", "info <name>".cyan().bold());
    println!("  {} - Storage statistics", "stats".cyan().bold());
    println!("  {} - Network connection status", "status".cyan().bold());
    println!("  {} - List encryption keys", "keys".cyan().bold());

    ui::print_section("🌐 Network Diagnostics");
    println!("  {} - Connected peers", "peers".blue().bold());
    println!("  {} - Network health check", "health".blue().bold());
    println!("  {} - Topology analysis", "network".blue().bold());
    println!("  {} - Discover new peers", "discover".blue().bold());

    ui::print_section("🔧 Advanced Features");
    println!(
        "  {} - Interactive command builder",
        "wizard".yellow().bold()
    );
    println!("  {} - Command examples", "examples".yellow().bold());
    println!("  {} - Session summary", "summary".yellow().bold());

    ui::print_section("💡 Quick Tips");
    println!("  • Use shortcuts: ls=list, s=status, p=peers, q=quit");
    println!("  • Type 'help <command>' for detailed usage");
    println!("  • Type 'examples' to see common workflows");
    println!("  • Type 'wizard' for guided command building");

    ui::print_separator();
}

/// Enhanced help system with contextual information
fn print_interactive_help_enhanced() {
    use colored::*;
    ui::print_header("🆘 DataMesh Help System");

    ui::print_section("🗂️ File Operations");
    println!(
        "  {} - Store files with encryption",
        "put <file>".green().bold()
    );
    println!(
        "  {} - Retrieve files by name/key",
        "get <name> <output>".green().bold()
    );
    println!("  {} - Browse your stored files", "list".green().bold());

    ui::print_section("📊 Information & Status");
    println!("  {} - Show file details", "info <name>".cyan().bold());
    println!("  {} - Storage statistics", "stats".cyan().bold());
    println!("  {} - Network connection status", "status".cyan().bold());
    println!("  {} - List encryption keys", "keys".cyan().bold());

    ui::print_section("🌐 Network Diagnostics");
    println!("  {} - Connected peers", "peers".blue().bold());
    println!("  {} - Network health check", "health".blue().bold());
    println!("  {} - Topology analysis", "network".blue().bold());
    println!("  {} - Discover new peers", "discover".blue().bold());
    println!(
        "  {} - File distribution analysis",
        "distribution".blue().bold()
    );
    println!("  {} - Bandwidth testing", "bandwidth".blue().bold());

    ui::print_section("🔧 Utilities");
    println!(
        "  {} - Interactive command builder",
        "wizard".yellow().bold()
    );
    println!("  {} - Command examples", "examples".yellow().bold());
    println!("  {} - Session summary", "summary".yellow().bold());

    println!("\n💡 Type 'help <command>' for detailed usage");
    println!("🚀 Type 'examples' to see common workflows");
    println!("🎯 Type 'wizard' for guided command building");
}

/// Show contextual help for specific commands
fn show_contextual_help(command: &str) {
    match command {
        "put" => {
            ui::print_section("📤 File Upload Help");
            ui::print_list_item("Basic usage: put <file>", None);
            ui::print_list_item("With alias: put <file> --name my-file", None);
            ui::print_list_item("With tags: put <file> --tags work,important", None);
            ui::print_list_item("Specific encryption: put <file> --public-key <hex>", None);
            println!("💡 Tip: Use relative paths for better organization");
            println!(
                "📝 Example: put ./documents/report.pdf --name quarterly-report --tags work,2024"
            );
        }
        "get" => {
            ui::print_section("📥 File Download Help");
            ui::print_list_item("By name: get my-file ./output.txt", None);
            ui::print_list_item("By key: get 1a2b3c4d ./output.txt", None);
            ui::print_list_item(
                "Specific key: get my-file ./output.txt --private-key my-key",
                None,
            );
            println!("💡 Tip: Use 'list' to see available files first");
            println!("📝 Example: get quarterly-report ./downloads/report.pdf");
        }
        "list" => {
            ui::print_section("📋 File Listing Help");
            ui::print_list_item("All files: list", None);
            ui::print_list_item("By tags: list --tags work", None);
            ui::print_list_item("Specific key: list --public-key <hex>", None);
            println!("💡 Tip: Use tags to organize your files");
            println!("📝 Example: list --tags work,important");
        }
        "info" => {
            ui::print_section("ℹ️ File Information Help");
            ui::print_list_item("By name: info my-file", None);
            ui::print_list_item("By key: info 1a2b3c4d", None);
            println!("💡 Tip: Shows file health, size, and metadata");
            println!("📝 Example: info quarterly-report");
        }
        "status" => {
            ui::print_section("📊 Network Status Help");
            ui::print_list_item("Shows peer connections and network health", None);
            ui::print_list_item("Displays listening addresses", None);
            println!("💡 Tip: Use this to check connectivity issues");
        }
        "peers" => {
            ui::print_section("👥 Peer Management Help");
            ui::print_list_item("Basic: peers", None);
            ui::print_list_item("Detailed: peers --detailed", None);
            println!("💡 Tip: Shows connection quality and reputation");
        }
        "health" => {
            ui::print_section("🏥 Network Health Help");
            ui::print_list_item("One-time check: health", None);
            ui::print_list_item("Continuous: health --continuous", None);
            ui::print_list_item("Custom interval: health --continuous --interval 10", None);
            println!("💡 Tip: Use continuous mode for monitoring");
        }
        _ => {
            ui::print_info("Type 'help <command>' for specific command help");
            ui::print_info("Available commands: put, get, list, info, stats, status, peers, health, network, discover");
        }
    }
}

/// Display common workflow examples
fn print_command_examples() {
    ui::print_header("📚 Common Workflows");

    ui::print_section("🚀 Getting Started");
    ui::print_list_item("Check network status:", Some(&["status", "peers"]));

    ui::print_list_item(
        "Store your first file:",
        Some(&[
            "put document.pdf --name my-document --tags work,important",
            "list",
            "info my-document",
        ]),
    );

    ui::print_section("📁 File Management");
    ui::print_list_item(
        "Organize with tags:",
        Some(&[
            "put report.pdf --name q1-report --tags quarterly,2024,finance",
            "put presentation.pptx --name q1-presentation --tags quarterly,2024,marketing",
            "list --tags quarterly",
        ]),
    );

    ui::print_list_item(
        "Retrieve files:",
        Some(&[
            "list --tags work",
            "get my-document ./downloads/document.pdf",
            "info my-document",
        ]),
    );

    ui::print_section("🌐 Network Diagnostics");
    ui::print_list_item(
        "Monitor network health:",
        Some(&[
            "status",
            "peers --detailed",
            "health --continuous --interval 5",
            "network --depth 2",
        ]),
    );

    println!("\n💡 Use 'wizard' for guided command building");
    println!("🆘 Use 'help <command>' for detailed command help");
}

/// Start the interactive command wizard
fn start_command_wizard() {
    use colored::*;
    ui::print_header("🧙 DataMesh Command Wizard");

    ui::print_section("What would you like to do?");
    println!("1. {} - Store a file", "Upload".green().bold());
    println!(
        "2. {} - Find and download a file",
        "Download".green().bold()
    );
    println!("3. {} - Check network status", "Monitor".cyan().bold());
    println!("4. {} - Search for files", "Search".blue().bold());
    println!("5. {} - Manage storage", "Maintenance".yellow().bold());

    ui::print_info("🎯 Copy and paste these commands at the prompt:");
    ui::print_list_item(
        "Upload: put <your-file> --name <friendly-name> --tags <tag1,tag2>",
        None,
    );
    ui::print_list_item("Download: get <file-name> <output-path>", None);
    ui::print_list_item("Monitor: status", None);
    ui::print_list_item("Search: list --tags <tag>", None);
    ui::print_list_item("Maintenance: stats", None);
}

/// Valid commands for interactive mode
const VALID_COMMANDS: &[&str] = &[
    "put",
    "get",
    "list",
    "info",
    "stats",
    "status",
    "peers",
    "health",
    "network",
    "discover",
    "distribution",
    "bandwidth",
    "keys",
    "help",
    "quit",
    "exit",
    "wizard",
    "examples",
];

/// Interactive session state
pub struct InteractiveSession {
    history: Vec<String>,
    shortcuts: HashMap<String, String>,
    last_command: Option<String>,
    session_stats: SessionStats,
    user_id: String,
    command_history: Vec<String>,
    start_time: chrono::DateTime<chrono::Utc>,
}

/// Session statistics
#[derive(Default)]
struct SessionStats {
    commands_executed: usize,
    files_uploaded: usize,
    files_downloaded: usize,
    errors_encountered: usize,
}

/// Command completer for suggestions
struct CommandCompleter {
    commands: Vec<String>,
    context_help: HashMap<String, Vec<String>>,
}

impl CommandCompleter {
    fn new() -> Self {
        let commands = VALID_COMMANDS.iter().map(|s| s.to_string()).collect();

        let mut context_help = HashMap::new();
        context_help.insert(
            "put".to_string(),
            vec![
                "--name <alias>".to_string(),
                "--tags <tag1,tag2>".to_string(),
                "--public-key <hex>".to_string(),
            ],
        );
        context_help.insert("get".to_string(), vec!["--private-key <name>".to_string()]);
        context_help.insert(
            "list".to_string(),
            vec!["--tags <tag>".to_string(), "--public-key <hex>".to_string()],
        );

        Self {
            commands,
            context_help,
        }
    }

    fn suggest_similar_commands(&self, input: &str) -> Vec<String> {
        self.commands
            .iter()
            .filter(|cmd| levenshtein_distance(cmd, input) <= 2)
            .cloned()
            .collect()
    }
}

impl InteractiveSession {
    pub fn new() -> Self {
        let mut session = Self {
            history: Vec::new(),
            shortcuts: HashMap::new(),
            last_command: None,
            session_stats: SessionStats::default(),
            user_id: "default_user".to_string(),
            command_history: Vec::new(),
            start_time: chrono::Utc::now(),
        };
        session.add_shortcuts();
        session
    }

    fn add_shortcuts(&mut self) {
        // Unix-like shortcuts
        self.shortcuts.insert("ls".to_string(), "list".to_string());
        self.shortcuts
            .insert("ll".to_string(), "list --detailed".to_string());
        self.shortcuts
            .insert("pwd".to_string(), "status".to_string());
        self.shortcuts.insert("q".to_string(), "quit".to_string());
        self.shortcuts.insert("?".to_string(), "help".to_string());

        // DataMesh-specific shortcuts
        self.shortcuts.insert("s".to_string(), "status".to_string());
        self.shortcuts.insert("p".to_string(), "peers".to_string());
        self.shortcuts.insert("h".to_string(), "health".to_string());

        // Repeat last command
        self.shortcuts.insert("".to_string(), "".to_string()); // Special case for !!
    }

    fn resolve_command(&mut self, input: &str) -> String {
        if input == "!!" {
            self.last_command
                .clone()
                .unwrap_or_else(|| "help".to_string())
        } else {
            self.shortcuts
                .get(input)
                .cloned()
                .unwrap_or_else(|| input.to_string())
        }
    }

    fn add_to_history(&mut self, command: &str) {
        self.history.push(command.to_string());
        self.last_command = Some(command.to_string());

        // Limit history size
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    fn increment_stat(&mut self, stat: &str) {
        self.session_stats.commands_executed += 1;
        match stat {
            "upload" => self.session_stats.files_uploaded += 1,
            "download" => self.session_stats.files_downloaded += 1,
            "error" => self.session_stats.errors_encountered += 1,
            _ => {}
        }
    }

    fn show_session_summary(&self) {
        ui::print_section("📊 Session Summary");
        ui::print_key_value(
            "Commands Executed",
            &self.session_stats.commands_executed.to_string(),
        );
        ui::print_key_value(
            "Files Uploaded",
            &self.session_stats.files_uploaded.to_string(),
        );
        ui::print_key_value(
            "Files Downloaded",
            &self.session_stats.files_downloaded.to_string(),
        );

        if self.session_stats.errors_encountered > 0 {
            ui::print_key_value(
                "Errors Encountered",
                &self.session_stats.errors_encountered.to_string(),
            );
        }

        if !self.history.is_empty() {
            println!("\n🕒 Recent Commands:");
            for (i, cmd) in self.history.iter().rev().take(5).enumerate() {
                println!("  {}. {}", i + 1, cmd);
            }
        }
    }

    /// Get the command history
    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }
}

/// Run the interactive console mode
///
/// # Arguments
///
/// * `cli` - Command line arguments
/// * `key_manager` - Key manager instance
/// * `bootstrap_peer` - Optional bootstrap peer ID
/// * `bootstrap_addr` - Optional bootstrap peer address
/// * `port` - Port to listen on
pub async fn run_interactive_mode(
    cli: &Cli,
    key_manager: KeyManager,
    bootstrap_peer: Option<PeerId>,
    bootstrap_addr: Option<Multiaddr>,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    // Create a modified CLI for network connection
    let mut network_cli = cli.clone();
    network_cli.bootstrap_peer = bootstrap_peer;
    network_cli.bootstrap_addr = bootstrap_addr.clone();
    if port > 0 {
        network_cli.port = port;
    }

    let config = Config::load_or_default(None)?;
    let mut swarm = create_swarm_and_connect_multi_bootstrap(&network_cli, &config).await?;

    if port > 0 {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
        swarm.listen_on(listen_addr.parse()?)?;
    }

    if let (Some(peer), Some(addr)) = (bootstrap_peer, bootstrap_addr) {
        swarm.behaviour_mut().kad.add_address(&peer, addr);
        println!("Connecting to bootstrap peer: {}", peer);
    }

    // Initialize database for interactive mode
    let db_path = database::get_default_db_path()?;
    let db = DatabaseManager::new(&db_path)?;

    // Initialize interactive session
    let mut session = InteractiveSession::new();
    let _completer = CommandCompleter::new();

    // Display enhanced welcome message
    print_interactive_welcome_enhanced(
        &swarm.local_peer_id().to_string(),
        &key_manager.key_info.public_key_hex,
    );

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut pending_file_retrieval: Option<FileRetrieval> = None;

    loop {
        // Handle network events
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        ui::print_info(&format!("Listening on {}", address));
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let connected_peers: Vec<_> = swarm.connected_peers().collect();
                        ui::print_success(&format!("Connected to peer: {}", peer_id));
                        ui::print_connection_status(connected_peers.len());
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        let connected_peers: Vec<_> = swarm.connected_peers().collect();
                        ui::print_warning(&format!("Disconnected from peer: {}", peer_id));
                        ui::print_connection_status(connected_peers.len());
                    }
                    SwarmEvent::Behaviour(behaviour_event) => {
                        handle_behaviour_event(behaviour_event, &mut pending_file_retrieval, &key_manager, cli);
                    }
                    _ => {}
                }
            }
            line = lines.next_line() => {
                match line? {
                    Some(input) => {
                        let input = input.trim();
                        if input.is_empty() {
                            continue;
                        }

                        // Resolve shortcuts and parse command
                        let resolved_input = session.resolve_command(input);
                        let parsed = parse_command_smart(&resolved_input, VALID_COMMANDS);

                        // Handle parsing errors
                        if !parsed.errors.is_empty() {
                            for error in &parsed.errors {
                                ui::print_error(error);
                            }
                            if !parsed.suggestions.is_empty() {
                                ui::print_info("💡 Suggestions:");
                                for suggestion in &parsed.suggestions {
                                    ui::print_list_item(suggestion, None);
                                }
                            }
                            continue;
                        }

                        // Add to history
                        session.add_to_history(&resolved_input);
                        session.increment_stat("command");

                        match parsed.command.as_str() {
                            "put" => {
                                if parsed.args.is_empty() {
                                    ui::print_error("Usage: put <file> [--name <alias>] [--tags <tag1,tag2>] [--public-key <hex>]");
                                    ui::print_info("Example: put document.pdf --name my-document --tags work,important");
                                    continue;
                                }

                                let file_path = PathBuf::from(&parsed.args[0]);
                                if !file_path.exists() {
                                    ui::print_error(&format!("File not found: {}", file_path.display()));
                                    continue;
                                }

                                if !file_path.is_file() {
                                    ui::print_error(&format!("Path is not a file: {}", file_path.display()));
                                    continue;
                                }

                                let public_key = parsed.flags.get("public-key").cloned();
                                let name = parsed.flags.get("name").cloned();
                                let tags = parsed.flags.get("tags").cloned();

                                // Show operation confirmation
                                let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                                ui::print_confirmation(
                                    "Uploading file",
                                    &format!("{} ({})", file_path.display(), ui::format_file_size(file_size))
                                );

                                match handle_put_interactive(&mut swarm, &file_path, &key_manager, &public_key, &name, &tags, &db).await {
                                    Ok(file_name) => ui::print_success(&format!("File stored successfully as '{}'", file_name)),
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("put", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "get" => {
                                if parsed.args.len() < 2 {
                                    ui::print_error("Usage: get <name_or_key> <output_path> [--private-key <name>]");
                                    ui::print_info("Example: get my-document ./downloaded-doc.pdf");
                                    continue;
                                }

                                let output_path = PathBuf::from(&parsed.args[1]);
                                if output_path.exists() {
                                    ui::print_warning(&format!("Output file already exists: {}", output_path.display()));
                                    ui::print_info("File will be overwritten if download succeeds");
                                }

                                // Check if parent directory exists
                                if let Some(parent) = output_path.parent() {
                                    if !parent.exists() {
                                        ui::print_error(&format!("Output directory does not exist: {}", parent.display()));
                                        continue;
                                    }
                                }

                                let identifier = parsed.args[0].clone();
                                let private_key = parsed.flags.get("private-key").cloned();

                                // Try to resolve identifier to a file key using database
                                let file_key = if let Ok(Some(file_entry)) = db.get_file_by_name(&identifier) {
                                    ui::print_info(&format!("Found file '{}' in database", identifier));
                                    file_entry.file_key
                                } else if let Ok(Some(_)) = db.get_file_by_key(&identifier) {
                                    ui::print_info("Using provided file key");
                                    identifier.clone()
                                } else {
                                    ui::print_info("Treating as direct file key");
                                    identifier.clone()
                                };

                                // Show operation confirmation
                                ui::print_confirmation(
                                    "Retrieving file",
                                    &format!("{} → {}", identifier, output_path.display())
                                );

                                match handle_get_interactive(&mut swarm, file_key, output_path, &mut pending_file_retrieval, &private_key).await {
                                    Ok(_) => ui::print_info("File retrieval initiated..."),
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("get", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "list" => {
                                let public_key = parsed.flags.get("public-key").cloned();
                                let tags = parsed.flags.get("tags").cloned();

                                match handle_list_interactive(&db, &key_manager, &public_key, &tags) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("list", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "keys" => {
                                println!("Available keys:");
                                let keys_dir = get_default_keys_dir().unwrap_or_else(|_| PathBuf::from("./keys"));
                                match KeyManager::list_keys(&keys_dir) {
                                    Ok(keys) => {
                                        for key_name in keys {
                                            if let Ok(info) = KeyManager::get_key_info(&keys_dir, &key_name) {
                                                println!("  {} (created: {}, public: {}...)",
                                                    info.name,
                                                    info.created.format("%Y-%m-%d %H:%M:%S"),
                                                    &info.public_key_hex[..16]
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => println!("Error listing keys: {}", e),
                                }
                            }
                            "quit" | "exit" => {
                                session.show_session_summary();
                                ui::print_success("Goodbye!");
                                return Ok(());
                            }
                            "info" => {
                                if parsed.args.is_empty() {
                                    ui::print_error("Usage: info <name_or_key>");
                                    continue;
                                }

                                let identifier = &parsed.args[0];
                                match handle_info_interactive(&db, identifier) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::handle_error(e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "stats" => {
                                match handle_stats_interactive(&db) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::handle_error(e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "status" => {
                                let connected_peers: Vec<_> = swarm.connected_peers().collect();
                                let listening_addrs: Vec<String> = swarm.listeners()
                                    .map(|addr| addr.to_string())
                                    .collect();

                                ui::print_detailed_network_status(
                                    &swarm.local_peer_id().to_string(),
                                    &listening_addrs,
                                    connected_peers.len()
                                );
                            }
                            "peers" => {
                                let detailed = parsed.flags.contains_key("detailed");
                                let format = crate::cli::OutputFormat::Table;

                                match network_diagnostics::handle_peers_command(&mut swarm, detailed, &format).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("peers", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "health" => {
                                let continuous = parsed.flags.contains_key("continuous");
                                let interval = parsed.flags.get("interval")
                                    .and_then(|v| v.parse().ok())
                                    .unwrap_or(5);

                                match network_diagnostics::handle_health_command(&mut swarm, continuous, interval).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("health", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "network" => {
                                let depth = parsed.flags.get("depth")
                                    .and_then(|v| v.parse().ok())
                                    .unwrap_or(2);
                                let visualize = parsed.flags.contains_key("visualize");

                                match network_diagnostics::handle_network_command(&mut swarm, depth, visualize).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("network", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "discover" => {
                                let timeout = parsed.flags.get("timeout")
                                    .and_then(|v| v.parse().ok())
                                    .unwrap_or(30);
                                let bootstrap_all = parsed.flags.contains_key("bootstrap-all");

                                match network_diagnostics::handle_discover_command(&mut swarm, timeout, bootstrap_all).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("discover", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "distribution" => {
                                let file_key = parsed.flags.get("file-key").cloned();
                                let public_key = parsed.flags.get("public-key").cloned();

                                match network_diagnostics::handle_distribution_command(&mut swarm, &file_key, &public_key).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("distribution", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "bandwidth" => {
                                let test_peer = parsed.flags.get("test-peer").cloned();
                                let duration = parsed.flags.get("duration")
                                    .and_then(|v| v.parse().ok())
                                    .unwrap_or(30);

                                match network_diagnostics::handle_bandwidth_command(&mut swarm, &test_peer, duration).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        let enhanced_error = error_handling::operation_error_with_context("bandwidth", e.as_ref());
                                        error_handling::display_enhanced_error(&enhanced_error);
                                    }
                                }
                            }
                            "help" => {
                                if parsed.args.is_empty() {
                                    print_interactive_help_enhanced();
                                } else {
                                    show_contextual_help(&parsed.args[0]);
                                }
                            }
                            "wizard" => {
                                start_command_wizard();
                            }
                            "examples" => {
                                print_command_examples();
                            }
                            "summary" => {
                                session.show_session_summary();
                            }
                            _ => {
                                ui::print_error(&format!("Unknown command: '{}'", parsed.command));
                                ui::print_info("Type 'help' for available commands or 'examples' for common workflows.");
                            }
                        }
                    }
                    None => break, // EOF
                }
            }
        }
    }

    Ok(())
}

async fn handle_put_interactive(
    swarm: &mut libp2p::swarm::Swarm<crate::network::MyBehaviour>,
    file_path: &PathBuf,
    key_manager: &KeyManager,
    public_key: &Option<String>,
    name: &Option<String>,
    tags: &Option<String>,
    db: &DatabaseManager,
) -> Result<String, Box<dyn Error>> {
    use crate::key_manager::get_encryption_key;
    use chrono::Local;
    use ecies::encrypt;
    use libp2p::kad::{Quorum, Record, RecordKey};
    use reed_solomon_erasure::galois_8::ReedSolomon;
    use std::fs;

    let file_data = fs::read(file_path)?;
    let file_key = RecordKey::new(&blake3::hash(&file_data).as_bytes());

    // Get the encryption key (either specified public key or default)
    let (encryption_public_key, public_key_hex) = get_encryption_key(public_key, key_manager)?;
    let encrypted_data = encrypt(&encryption_public_key.serialize(), &file_data)
        .map_err(|e| anyhow::anyhow!("Encryption error: {:?}", e))?;

    // Create Reed-Solomon encoder
    let r = ReedSolomon::new(PUB_DATA_SHARDS, PUB_PARITY_SHARDS)?;
    let chunk_size = (encrypted_data.len() + PUB_DATA_SHARDS - 1) / PUB_DATA_SHARDS;

    // Create shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; PUB_DATA_SHARDS + PUB_PARITY_SHARDS];

    // Fill data shards
    for (i, shard) in shards.iter_mut().enumerate().take(PUB_DATA_SHARDS) {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, encrypted_data.len());
        if start < encrypted_data.len() {
            shard[..end - start].copy_from_slice(&encrypted_data[start..end]);
        }
    }

    // Encode to create parity shards
    r.encode(&mut shards)?;

    // Store each shard
    let mut chunk_keys = Vec::new();
    for shard in shards {
        let chunk_key = RecordKey::new(&blake3::hash(&shard).as_bytes());
        chunk_keys.push(chunk_key.as_ref().to_vec()); // Store as Vec<u8>
        let record = Record {
            key: chunk_key,
            value: shard,
            publisher: None,
            expires: None,
        };
        swarm.behaviour_mut().kad.put_record(record, Quorum::One)?;
    }

    // Generate or use provided name
    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_name = if let Some(provided_name) = name {
        if db.is_name_taken(provided_name)? {
            return Err(format!(
                "Name '{}' is already taken. Please choose a different name.",
                provided_name
            )
            .into());
        }
        provided_name.clone()
    } else {
        db.generate_unique_name(&original_filename)?
    };

    // Parse tags
    let file_tags = if let Some(tag_str) = tags {
        tag_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Store file metadata in DHT
    let stored_file = StoredFile {
        chunk_keys,
        encryption_key: key_manager.key.serialize().to_vec(),
        file_size: file_data.len(),
        public_key_hex: public_key_hex.clone(),
        file_name: original_filename.clone(),
        stored_at: Local::now(),
    };
    let record = Record {
        key: file_key.clone(),
        value: serde_json::to_vec(&stored_file)?,
        publisher: None,
        expires: None,
    };
    swarm.behaviour_mut().kad.put_record(record, Quorum::One)?;

    // Store in database
    let file_size = file_data.len() as u64;
    let upload_time = Local::now();
    db.store_file(
        &file_name,
        &hex::encode(file_key.as_ref()),
        &original_filename,
        file_size,
        upload_time,
        &file_tags,
        &public_key_hex,
    )?;

    Ok(file_name)
}

async fn handle_get_interactive(
    swarm: &mut libp2p::swarm::Swarm<crate::network::MyBehaviour>,
    key: String,
    output_path: PathBuf,
    pending_file_retrieval: &mut Option<FileRetrieval>,
    _private_key: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    use chrono::Local;
    use libp2p::kad::RecordKey;

    let key_bytes = hex::decode(key)?;
    let record_key = RecordKey::from(key_bytes);
    swarm.behaviour_mut().kad.get_record(record_key);

    // Initialize file retrieval state
    *pending_file_retrieval = Some(FileRetrieval {
        stored_file: StoredFile {
            chunk_keys: Vec::new(),
            encryption_key: Vec::new(),
            file_size: 0,
            public_key_hex: String::new(),
            file_name: String::new(),
            stored_at: Local::now(),
        },
        chunks: HashMap::new(),
        output_path,
    });

    Ok(())
}

fn handle_behaviour_event(
    event: MyBehaviourEvent,
    pending_file_retrieval: &mut Option<FileRetrieval>,
    _key_manager: &KeyManager,
    _cli: &Cli,
) {
    match event {
        MyBehaviourEvent::Kad(kad_event) => {
            match kad_event {
                KademliaEvent::OutboundQueryProgressed { result, .. } => {
                    match result {
                        QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                            let record = &peer_record.record;

                            // Try to parse as StoredFile metadata first
                            if let Ok(stored_file) =
                                serde_json::from_slice::<StoredFile>(&record.value)
                            {
                                ui::print_operation_status(
                                    "File Metadata",
                                    "Found",
                                    Some(&format!(
                                        "Retrieving {} chunks",
                                        stored_file.chunk_keys.len()
                                    )),
                                );

                                ui::print_key_value("File Name", &stored_file.file_name);
                                ui::print_key_value(
                                    "Stored At",
                                    &stored_file
                                        .stored_at
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string(),
                                );
                                ui::print_key_value(
                                    "Encryption Key",
                                    &format!("{}...", &stored_file.public_key_hex[..16]),
                                );

                                // Update file retrieval state - we can't call swarm methods here,
                                // so we'll just update the state
                                if let Some(ref mut retrieval) = pending_file_retrieval {
                                    retrieval.stored_file = stored_file.clone();

                                    // Note: In interactive mode, we would need to queue chunk requests
                                    // for the next swarm poll cycle
                                }
                            } else {
                                // This might be a chunk
                                if let Some(ref mut retrieval) = pending_file_retrieval {
                                    // Check if this record key matches any of our expected chunk keys
                                    let record_key_bytes = record.key.as_ref().to_vec();
                                    if retrieval.stored_file.chunk_keys.contains(&record_key_bytes)
                                    {
                                        retrieval
                                            .chunks
                                            .insert(record.key.clone(), record.value.clone());
                                        ui::print_progress(
                                            retrieval.chunks.len(),
                                            retrieval.stored_file.chunk_keys.len(),
                                            "Downloading chunks",
                                        );

                                        // Check if we have all chunks needed for reconstruction
                                        if retrieval.chunks.len() >= DATA_SHARDS {
                                            if let Err(e) = reconstruct_file_interactive(
                                                retrieval,
                                                _key_manager,
                                                _cli,
                                            ) {
                                                println!("Failed to reconstruct file: {:?}", e);
                                            } else {
                                                println!("File reconstruction complete!");
                                                // Clear the retrieval state
                                                // *pending_file_retrieval = None; // Can't do this in interactive mode easily
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn reconstruct_file_interactive(
    retrieval: &mut FileRetrieval,
    _key_manager: &KeyManager,
    _cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    use ecies::{decrypt, SecretKey};
    use libp2p::kad::RecordKey;
    use reed_solomon_erasure::galois_8::ReedSolomon;
    use std::fs;

    let r = ReedSolomon::new(PUB_DATA_SHARDS, PUB_PARITY_SHARDS)?;

    // Prepare shards for reconstruction
    let mut shards: Vec<Option<Vec<u8>>> = vec![None; PUB_DATA_SHARDS + PUB_PARITY_SHARDS];

    // Fill shards with available chunks
    for (i, chunk_key_bytes) in retrieval.stored_file.chunk_keys.iter().enumerate() {
        let chunk_key = RecordKey::from(chunk_key_bytes.clone());
        if let Some(chunk_data) = retrieval.chunks.get(&chunk_key) {
            shards[i] = Some(chunk_data.clone());
        }
    }

    // Reconstruct missing shards if needed
    r.reconstruct(&mut shards)?;

    // Combine data shards to reconstruct the encrypted file
    let mut encrypted_data = Vec::new();
    for shard_opt in shards.iter().take(PUB_DATA_SHARDS) {
        if let Some(shard) = shard_opt {
            encrypted_data.extend_from_slice(shard);
        }
    }

    // Decrypt the data using the stored key
    let decryption_key = SecretKey::parse_slice(&retrieval.stored_file.encryption_key)
        .map_err(|e| anyhow::anyhow!("Failed to parse encryption key: {:?}", e))?;

    let decrypted_data = decrypt(&decryption_key.serialize(), &encrypted_data)
        .map_err(|e| anyhow::anyhow!("Decryption error: {:?}", e))?;

    // Trim to original file size
    let final_data = if decrypted_data.len() > retrieval.stored_file.file_size {
        &decrypted_data[..retrieval.stored_file.file_size]
    } else {
        &decrypted_data
    };

    // Write to output file
    fs::write(&retrieval.output_path, final_data)?;
    println!(
        "File successfully retrieved and saved to: {:?}",
        retrieval.output_path
    );
    println!("Original file name: {}", retrieval.stored_file.file_name);
    println!(
        "Stored at: {}",
        retrieval.stored_file.stored_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Encrypted with public key: {}",
        retrieval.stored_file.public_key_hex
    );

    Ok(())
}

// Service mode - runs the DFS node without interactive input
pub async fn run_service_mode(
    cli: &Cli,
    key_manager: KeyManager,
    bootstrap_peer: Option<PeerId>,
    bootstrap_addr: Option<Multiaddr>,
    port: u16,
    timeout: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    // Create a modified CLI for network connection
    let mut network_cli = cli.clone();
    network_cli.bootstrap_peer = bootstrap_peer;
    network_cli.bootstrap_addr = bootstrap_addr.clone();
    if port > 0 {
        network_cli.port = port;
    }

    let config = Config::load_or_default(None)?;
    let mut swarm = create_swarm_and_connect_multi_bootstrap(&network_cli, &config).await?;

    if port > 0 {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
        swarm.listen_on(listen_addr.parse()?)?;
    }

    if let (Some(peer), Some(addr)) = (bootstrap_peer, bootstrap_addr) {
        swarm.behaviour_mut().kad.add_address(&peer, addr);
        println!("Connecting to bootstrap peer: {}", peer);

        // Bootstrap the DHT to improve connectivity
        swarm.behaviour_mut().kad.bootstrap().ok();
    }

    println!("DFS Service Mode");
    println!("================");
    println!("Peer ID: {:?}", swarm.local_peer_id());
    println!(
        "Default public key: {}",
        key_manager.key_info.public_key_hex
    );
    println!("Service running... (use Ctrl+C to stop)");

    let mut pending_file_retrieval: Option<FileRetrieval> = None;
    let start_time = std::time::Instant::now();
    let mut last_bootstrap = std::time::Instant::now();

    loop {
        // Check timeout if specified
        if let Some(timeout_secs) = timeout {
            if start_time.elapsed().as_secs() >= timeout_secs {
                println!(
                    "Service timeout reached ({}s), shutting down...",
                    timeout_secs
                );
                break;
            }
        }

        // Periodically re-bootstrap the DHT to maintain connectivity
        if last_bootstrap.elapsed().as_secs() >= 30 {
            // More frequent bootstrapping
            println!("Performing DHT bootstrap to maintain connectivity...");
            swarm.behaviour_mut().kad.bootstrap().ok();
            last_bootstrap = std::time::Instant::now();

            // Print current peer count for diagnostics
            let connected_peers: Vec<_> = swarm.connected_peers().collect();
            println!("Currently connected to {} peers", connected_peers.len());
        }

        // Handle network events
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Service node connected to peer: {}", peer_id);
                    }
                    SwarmEvent::Behaviour(behaviour_event) => {
                        handle_behaviour_event(behaviour_event, &mut pending_file_retrieval, &key_manager, cli);
                    }
                    _ => {}
                }
            }
            // Add a small delay to prevent busy waiting
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Continue the loop
            }
        }
    }

    println!("DFS service stopped");
    Ok(())
}

/// Handle list command in interactive mode
fn handle_list_interactive(
    db: &DatabaseManager,
    key_manager: &KeyManager,
    public_key: &Option<String>,
    tags: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    // Parse tag filter
    let tag_filter = tags.as_ref().map(|t| t.as_str());

    // Get files from database
    let files = db.list_files(tag_filter)?;

    if let Some(pk) = public_key {
        // Filter by public key if specified
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|f| f.public_key_hex == *pk)
            .collect();
        ui::print_file_list(&filtered_files);
    } else {
        // Show all files for this user's default key
        let target_public_key = &key_manager.key_info.public_key_hex;
        let filtered_files: Vec<_> = files
            .into_iter()
            .filter(|f| f.public_key_hex == *target_public_key)
            .collect();
        ui::print_file_list(&filtered_files);
    }

    Ok(())
}

/// Handle info command in interactive mode
fn handle_info_interactive(db: &DatabaseManager, identifier: &str) -> Result<(), Box<dyn Error>> {
    // Try to find the file by name first, then by key
    let file = if let Some(file) = db.get_file_by_name(identifier)? {
        file
    } else if let Some(file) = db.get_file_by_key(identifier)? {
        file
    } else {
        ui::print_error(&format!("File not found: {}", identifier));
        ui::print_info("Use 'list' to see available files");
        return Ok(());
    };

    ui::print_file_info(&file);
    Ok(())
}

/// Handle stats command in interactive mode
fn handle_stats_interactive(db: &DatabaseManager) -> Result<(), Box<dyn Error>> {
    let stats = db.get_stats()?;
    ui::print_database_stats(&stats);
    Ok(())
}
