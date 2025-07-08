/// User Interface Module
///
/// This module provides enhanced user interface components including:
/// - Progress bars for file operations
/// - Colored output for better readability
/// - Formatted file listings
/// - Status indicators and feedback
/// - Error message formatting
/// - Interactive prompts and confirmations
/// - Real-time status displays
/// - Professional-grade CLI experience

use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;
use std::io::{self, Write};

use crate::database::{DatabaseStats, FileEntry};
use crate::network_diagnostics::{
    PeerInfo, NetworkHealth, FileDistribution, BandwidthTest, NetworkTopology, DiscoveryResult
};

/// Progress bar manager for file operations
pub struct ProgressManager {
    bar: ProgressBar,
}

impl ProgressManager {
    /// Create a new progress bar for file upload
    pub fn new_upload(file_size: u64) -> Self {
        let bar = ProgressBar::new(file_size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message("Uploading");
        
        Self { bar }
    }

    /// Create a new progress bar for file download
    pub fn new_download(total_chunks: u64) -> Self {
        let bar = ProgressBar::new(total_chunks);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({msg})")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message("Downloading");
        
        Self { bar }
    }

    /// Update progress
    pub fn set_position(&self, position: u64) {
        self.bar.set_position(position);
    }

    /// Set progress message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        self.bar.finish();
    }

    /// Finish with a success message
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }
}

/// Enhanced progress manager for multi-operation tasks
pub struct MultiOperationProgress {
    multi: MultiProgress,
    operations: Vec<ProgressBar>,
}

impl MultiOperationProgress {
    /// Create a new multi-operation progress manager
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            operations: Vec::new(),
        }
    }

    /// Add a new operation with progress tracking
    pub fn add_operation(&mut self, name: &str, total: u64) -> usize {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{spinner:.green}} [{{elapsed_precise}}] {{prefix:.bold.cyan}} [{{bar:40.cyan/blue}}] {{pos}}/{{len}} ({{eta}}) {{msg}}"))
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_prefix(name.to_string());
        
        let index = self.operations.len();
        self.operations.push(pb);
        index
    }

    /// Update operation progress
    pub fn update_operation(&self, index: usize, position: u64, message: &str) {
        if let Some(pb) = self.operations.get(index) {
            pb.set_position(position);
            pb.set_message(message.to_string());
        }
    }

    /// Finish an operation
    pub fn finish_operation(&self, index: usize, message: &str) {
        if let Some(pb) = self.operations.get(index) {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Clear all progress bars
    pub fn clear(&self) {
        self.multi.clear().unwrap_or(());
    }
}

/// Interactive confirmation prompt
pub fn confirm_action(message: &str, default: bool) -> bool {
    let default_char = if default { "Y/n" } else { "y/N" };
    print!("{} {} [{}]: ", "?".yellow().bold(), message, default_char);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let input = input.trim().to_lowercase();
            if input.is_empty() {
                default
            } else {
                matches!(input.as_str(), "y" | "yes")
            }
        }
        Err(_) => default,
    }
}

/// Display a formatted header with decorations
pub fn print_header(title: &str) {
    let width = 80;
    let title_len = title.len();
    let padding = (width - title_len - 2) / 2;
    
    println!();
    println!("{}", "═".repeat(width).bright_cyan());
    println!("{}{} {}{}", 
        " ".repeat(padding), 
        "│".bright_cyan(), 
        title.bold().bright_white(),
        " ".repeat(width - padding - title_len - 2)
    );
    println!("{}", "═".repeat(width).bright_cyan());
    println!();
}

/// Display a section header
pub fn print_section(title: &str) {
    println!();
    println!("{}", format!("▶ {}", title).bold().bright_blue());
    println!("{}", "─".repeat(title.len() + 2).bright_blue());
}

/// Display operation status with icon
pub fn print_operation_status(operation: &str, status: &str, details: Option<&str>) {
    let (icon, color) = match status.to_lowercase().as_str() {
        "success" | "completed" | "ok" => ("✓", Color::Green),
        "error" | "failed" | "fail" => ("✗", Color::Red),
        "warning" | "warn" => ("⚠", Color::Yellow),
        "info" | "running" | "in_progress" => ("ℹ", Color::Cyan),
        "pending" | "waiting" => ("⏳", Color::Yellow),
        _ => ("•", Color::White),
    };

    let status_text = format!("{} {}", icon, operation).color(color).bold();
    
    if let Some(details) = details {
        println!("  {} {}", status_text, details.dimmed());
    } else {
        println!("  {}", status_text);
    }
}

/// Display a step in a process
pub fn print_step(step_num: usize, total_steps: usize, description: &str) {
    let progress = format!("[{}/{}]", step_num, total_steps);
    println!("  {} {} {}", 
        progress.bright_cyan().bold(),
        "→".bright_blue(),
        description
    );
}

/// Display key-value information in a formatted way
pub fn print_key_value(key: &str, value: &str) {
    println!("  {}: {}", 
        key.bold().bright_white(), 
        value.bright_green()
    );
}

/// Display a list of items with bullets
pub fn print_list_item(item: &str, sub_items: Option<&[&str]>) {
    println!("  {} {}", "•".bright_blue(), item);
    
    if let Some(sub_items) = sub_items {
        for sub_item in sub_items {
            println!("    {} {}", "◦".bright_cyan(), sub_item.dimmed());
        }
    }
}

/// Display a table with headers and rows
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if headers.is_empty() || rows.is_empty() {
        return;
    }

    // Calculate column widths
    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Print header
    print!("  ");
    for (i, header) in headers.iter().enumerate() {
        print!("{:<width$}", header.bold().bright_cyan(), width = col_widths[i] + 2);
    }
    println!();

    // Print separator
    print!("  ");
    for width in &col_widths {
        print!("{}", "─".repeat(width + 2));
    }
    println!();

    // Print rows
    for row in rows {
        print!("  ");
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                print!("{:<width$}", cell, width = col_widths[i] + 2);
            }
        }
        println!();
    }
    println!();
}

/// Display a spinner for long-running operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Display network connection status
pub fn print_network_status(connected_peers: usize, bootstrap_connected: bool) {
    print_section("Network Status");
    
    print_operation_status(
        "Peer Connections", 
        if connected_peers > 0 { "Connected" } else { "Searching" },
        Some(&format!("{} peers connected", connected_peers))
    );
    
    let bootstrap_status = if bootstrap_connected { "Connected" } else { "Disconnected" };
    print_operation_status("Bootstrap Node", bootstrap_status, None);
}

/// Display file operation summary
pub fn print_file_operation_summary(operation: &str, files_processed: usize, total_files: usize, 
                                   duration: Duration, errors: usize) {
    print_section(&format!("{} Summary", operation));
    
    print_key_value("Files Processed", &format!("{}/{}", files_processed, total_files));
    print_key_value("Success Rate", &format!("{:.1}%", 
        (files_processed - errors) as f64 / total_files as f64 * 100.0));
    print_key_value("Duration", &format_duration(duration));
    
    if errors > 0 {
        print_key_value("Errors", &errors.to_string());
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if size == 0 {
        return "0 B".to_string();
    }
    
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size_f /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes", secs / 60)
    } else if secs < 86400 {
        format!("{} hours", secs / 3600)
    } else {
        format!("{} days", secs / 86400)
    }
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

/// Print an error message
pub fn print_error(message: &str) {
    println!("{} {}", "❌".red(), message.red());
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow());
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

/// Print a formatted file listing
pub fn print_file_list(files: &[FileEntry]) {
    if files.is_empty() {
        print_info("No files found");
        return;
    }

    println!("{}", "📋 Your Files:".bold().cyan());
    println!();
    
    for file in files {
        let health_indicator = if file.chunks_healthy == file.chunks_total {
            "✅".green()
        } else if file.chunks_healthy >= 4 {
            "⚠️".yellow()
        } else {
            "❌".red()
        };
        
        let size_str = format_file_size(file.file_size);
        let age = chrono::Local::now().signed_duration_since(file.upload_time);
        let age_str = if age.num_days() > 0 {
            format!("{} days ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{} hours ago", age.num_hours())
        } else if age.num_minutes() > 0 {
            format!("{} minutes ago", age.num_minutes())
        } else {
            "just now".to_string()
        };
        
        println!("  {} {} ({})", health_indicator, file.name.bold(), file.original_filename.dimmed());
        println!("    {} • {} • {}", size_str, age_str, format!("{}% healthy", (file.chunks_healthy * 100 / file.chunks_total)).dimmed());
        
        if !file.tags.is_empty() {
            let tags_str = file.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            println!("    {}", tags_str.dimmed());
        }
        
        println!();
    }
}

/// Print file information
pub fn print_file_info(file: &FileEntry) {
    println!("{}", format!("📄 {}", file.name).bold().cyan());
    println!("├─ Original: {}", file.original_filename);
    println!("├─ Size: {}", format_file_size(file.file_size));
    println!("├─ Uploaded: {}", file.upload_time.format("%Y-%m-%d %H:%M:%S"));
    println!("├─ Health: {}/{} chunks ({}%)", 
             file.chunks_healthy, 
             file.chunks_total,
             file.chunks_healthy * 100 / file.chunks_total);
    println!("├─ Key: {}", file.file_key.dimmed());
    
    if !file.tags.is_empty() {
        println!("├─ Tags: {}", file.tags.join(", "));
    }
    
    let health_status = if file.chunks_healthy == file.chunks_total {
        "✅ Fully redundant".green()
    } else if file.chunks_healthy >= 4 {
        "⚠️ Reduced redundancy".yellow()
    } else {
        "❌ At risk".red()
    };
    
    println!("└─ Status: {}", health_status);
}

/// Print database statistics
pub fn print_database_stats(stats: &DatabaseStats) {
    println!("{}", "📊 DFS Storage Statistics".bold().cyan());
    println!("├─ Total files: {}", stats.total_files);
    println!("├─ Total size: {}", format_file_size(stats.total_size));
    println!("└─ Average health: {:.1}%", stats.average_health * 100.0);
}

/// Print a formatted error with suggestions
pub fn print_error_with_suggestions(error: &str, suggestions: &[&str]) {
    print_error(error);
    
    if !suggestions.is_empty() {
        println!("{}", "💡 Suggestions:".bold().yellow());
        for suggestion in suggestions {
            println!("   • {}", suggestion);
        }
    }
}

/// Print detailed network status with peer information
pub fn print_detailed_network_status(peer_id: &str, listening_addresses: &[String], connected_peers: usize) {
    println!("{}", "🌐 Network Status".bold().cyan());
    println!("├─ Peer ID: {}", peer_id.dimmed());
    println!("├─ Listening on:");
    for addr in listening_addresses {
        println!("│  └─ {}", addr.dimmed());
    }
    
    let connection_status = if connected_peers == 0 {
        "❌ Disconnected".red()
    } else if connected_peers < 3 {
        "⚠️ Limited connectivity".yellow()
    } else {
        "✅ Well connected".green()
    };
    
    println!("└─ Connected peers: {} ({})", connected_peers, connection_status);
}

/// Print connection status indicator
pub fn print_connection_status(connected_peers: usize) {
    let status = if connected_peers == 0 {
        "❌ Offline".red()
    } else if connected_peers < 3 {
        format!("⚠️ {} peers", connected_peers).yellow()
    } else {
        format!("✅ {} peers", connected_peers).green()
    };
    
    println!("Network: {}", status);
}

/// Print a separator line
pub fn print_separator() {
    println!("{}", "─".repeat(50).dimmed());
}

/// Print welcome message for interactive mode
pub fn print_interactive_welcome(peer_id: &str, public_key: &str) {
    println!("{}", "🚀 DFS Interactive Console".bold().cyan());
    print_separator();
    println!("Peer ID: {}", peer_id.dimmed());
    println!("Public Key: {}...", &public_key[..16].dimmed());
    println!();
    println!("{}", "Available Commands:".bold());
    println!("  {} <file> [--name <alias>] [--tags <tag1,tag2>] - Store a file", "put".bold().green());
    println!("  {} <name|key> <output>                          - Retrieve a file", "get".bold().green());
    println!("  {} [--tags <tag>]                               - List your files", "list".bold().green());
    println!("  {} <name|key>                                   - Show file details", "info".bold().green());
    println!("  {}                                              - Show available keys", "keys".bold().green());
    println!("  {}                                              - Show storage statistics", "stats".bold().green());
    println!("  {}                                             - Show network status", "status".bold().green());
    println!("  {} [--detailed]                                - Show connected peers", "peers".bold().cyan());
    println!("  {} [--continuous] [--interval <sec>]           - Monitor network health", "health".bold().cyan());
    println!("  {} [--depth <n>] [--visualize]                 - Analyze network topology", "network".bold().cyan());
    println!("  {} [--timeout <sec>] [--bootstrap-all]         - Discover new peers", "discover".bold().cyan());
    println!("  {} [--file-key <key>] [--public-key <key>]     - Analyze file distribution", "distribution".bold().cyan());
    println!("  {} [--test-peer <id>] [--duration <sec>]       - Test network bandwidth", "bandwidth".bold().cyan());
    println!("  {}                                              - Show this help", "help".bold().green());
    println!("  {}                                              - Exit", "quit".bold().green());
    println!();
    println!("{}", "Enter commands below:".dimmed());
    print_separator();
}

/// Get a spinner for long operations
pub fn get_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Print operation confirmation
pub fn print_confirmation(operation: &str, details: &str) {
    println!("{} {} {}", "📋".cyan(), operation.bold(), details.dimmed());
}

/// Print operation progress
pub fn print_progress(current: usize, total: usize, operation: &str) {
    let percentage = (current as f64 / total as f64 * 100.0) as u8;
    let bar_length = 20;
    let filled = (current * bar_length / total).min(bar_length);
    let empty = bar_length - filled;
    
    let bar = format!("{}{}",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed()
    );
    
    println!("{} {} [{}] {}% ({}/{})", 
        "🔄".cyan(), 
        operation, 
        bar, 
        percentage, 
        current, 
        total
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30 seconds");
        assert_eq!(format_duration(Duration::from_secs(90)), "1 minutes");
        assert_eq!(format_duration(Duration::from_secs(3700)), "1 hours");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1 days");
    }
}

// Network Diagnostics UI Functions

/// Print peer information in table format
pub fn print_peer_table(peers: &[PeerInfo], detailed: bool) {
    if peers.is_empty() {
        print_info("No peers connected");
        return;
    }

    println!("{}", "👥 Connected Peers".bold().cyan());
    print_separator();
    
    if detailed {
        for peer in peers {
            println!("Peer: {}", peer.peer_id.bold());
            println!("├─ Addresses:");
            for addr in &peer.addresses {
                println!("│  └─ {}", addr.dimmed());
            }
            println!("├─ Connected: {} ({})", 
                     peer.connected_at.format("%Y-%m-%d %H:%M:%S"),
                     format_duration(peer.connection_duration));
            println!("├─ Operations: {} successful, {} failed", 
                     peer.successful_ops, peer.failed_ops);
            println!("├─ Avg Response: {}ms", peer.avg_response_time);
            println!("├─ Last Seen: {}", peer.last_seen.format("%Y-%m-%d %H:%M:%S"));
            println!("└─ Reputation: {}%", peer.reputation);
            println!();
        }
    } else {
        println!("{:<52} {:<10} {:<8}", "Peer ID", "Duration", "Status");
        println!("{}", "─".repeat(80).dimmed());
        
        for peer in peers {
            let duration_str = format_duration(peer.connection_duration);
            let status = if peer.reputation >= 80 {
                "✅ Good".green()
            } else if peer.reputation >= 60 {
                "⚠️ Fair".yellow()
            } else {
                "❌ Poor".red()
            };
            
            println!("{:<52} {:<10} {}", 
                     peer.peer_id.dimmed(),
                     duration_str,
                     status);
        }
    }
}

/// Print network health information
pub fn print_network_health(health: &NetworkHealth) {
    println!("{}", "🏥 Network Health".bold().cyan());
    print_separator();
    
    let peer_status = if health.connected_peers >= 5 {
        "✅ Excellent".green()
    } else if health.connected_peers >= 3 {
        "⚠️ Good".yellow()
    } else if health.connected_peers > 0 {
        "⚠️ Limited".yellow()
    } else {
        "❌ Isolated".red()
    };
    
    println!("Connected Peers: {} ({})", health.connected_peers, peer_status);
    println!("Bootstrap Peers: {}", health.active_bootstrap_peers);
    println!("Routing Table: {} entries", health.routing_table_size);
    println!("Avg Response: {}ms", health.avg_response_time);
    println!("Uptime: {:.1}%", health.uptime_percentage);
    println!("Operations (1h): {} successful, {} failed", 
             health.successful_ops_last_hour,
             health.failed_ops_last_hour);
    
    let stability_status = if health.stability_score >= 90 {
        "✅ Excellent".green()
    } else if health.stability_score >= 70 {
        "⚠️ Good".yellow()
    } else {
        "❌ Poor".red()
    };
    
    println!("Stability: {}% ({})", health.stability_score, stability_status);
}

/// Print file distribution information
pub fn print_file_distribution(distribution: &FileDistribution) {
    println!("{}", "📊 File Distribution".bold().cyan());
    print_separator();
    
    println!("File Key: {}", distribution.file_key.dimmed());
    println!("Total Chunks: {}", distribution.total_chunks);
    println!("Available Chunks: {}", distribution.available_chunks);
    println!("Replication Factor: {:.2}", distribution.replication_factor);
    println!("Fault Tolerance: {} chunks can be lost", distribution.fault_tolerance);
    
    if !distribution.chunk_locations.is_empty() {
        println!("\nChunk Locations:");
        for (peer_id, chunks) in &distribution.chunk_locations {
            println!("  {}: chunks {:?}", 
                     peer_id.dimmed(),
                     chunks);
        }
    }
    
    let health_status = if distribution.available_chunks == distribution.total_chunks {
        "✅ Fully Available".green()
    } else if distribution.available_chunks >= 4 {
        "⚠️ Recoverable".yellow()
    } else {
        "❌ At Risk".red()
    };
    
    println!("\nStatus: {}", health_status);
}

/// Print multiple file distributions
pub fn print_file_distributions(distributions: &[FileDistribution]) {
    if distributions.is_empty() {
        print_info("No file distributions found");
        return;
    }

    println!("{}", "📊 File Distribution Summary".bold().cyan());
    print_separator();
    
    println!("{:<64} {:<8} {:<8} {:<10}", "File Key", "Chunks", "Avail", "Status");
    println!("{}", "─".repeat(100).dimmed());
    
    for dist in distributions {
        let status = if dist.available_chunks == dist.total_chunks {
            "✅ Full".green()
        } else if dist.available_chunks >= 4 {
            "⚠️ Partial".yellow()
        } else {
            "❌ Risk".red()
        };
        
        println!("{:<64} {:<8} {:<8} {}", 
                 format!("{}...", &dist.file_key[..16]).dimmed(),
                 dist.total_chunks,
                 dist.available_chunks,
                 status);
    }
}

/// Print network topology information
pub fn print_network_topology(topology: &NetworkTopology) {
    println!("{}", "🕸️ Network Topology".bold().cyan());
    print_separator();
    
    println!("Local Peer: {}", topology.local_peer.dimmed());
    println!("Direct Neighbors: {}", topology.neighbors.len());
    println!("Routing Buckets: {}", topology.routing_buckets.len());
    println!("Estimated Diameter: {}", topology.estimated_diameter);
    println!("Total Reachable: {}", topology.total_reachable);
    
    if !topology.neighbors.is_empty() {
        println!("\nDirect Neighbors:");
        for neighbor in &topology.neighbors {
            println!("  └─ {}", neighbor.dimmed());
        }
    }
}

/// Print network visualization (ASCII art representation)
pub fn print_network_visualization(topology: &NetworkTopology) {
    println!("{}", "🎨 Network Visualization".bold().cyan());
    print_separator();
    
    println!("        ┌─ {}", "YOU".bold().green());
    println!("        │");
    
    for (i, neighbor) in topology.neighbors.iter().enumerate() {
        let connector = if i == topology.neighbors.len() - 1 { "└" } else { "├" };
        let short_id = if neighbor.len() >= 8 { &neighbor[..8] } else { neighbor };
        println!("        {}─ {}", connector, format!("{}...", short_id).dimmed());
    }
    
    if topology.neighbors.is_empty() {
        println!("        └─ {}", "No connections".red());
    }
}

/// Print peer discovery results
pub fn print_discovery_result(result: &DiscoveryResult) {
    println!("{}", "🔍 Peer Discovery Results".bold().cyan());
    print_separator();
    
    println!("New Peers Found: {}", result.new_peers.len());
    println!("Total Discovered: {}", result.total_discovered);
    println!("Discovery Time: {}", format_duration(result.discovery_duration));
    println!("Success Rate: {:.1}%", result.success_rate * 100.0);
    
    if !result.new_peers.is_empty() {
        println!("\nNewly Discovered Peers:");
        for peer in &result.new_peers {
            println!("  └─ {}", peer.peer_id.dimmed());
        }
    }
}

/// Print bandwidth test results
pub fn print_bandwidth_test(test: &BandwidthTest) {
    println!("{}", "🚀 Bandwidth Test Results".bold().cyan());
    print_separator();
    
    println!("Test Peer: {}", test.peer_id.dimmed());
    println!("Download Speed: {}/s", format_file_size(test.download_speed));
    println!("Upload Speed: {}/s", format_file_size(test.upload_speed));
    println!("Round Trip Time: {}ms", test.rtt);
    println!("Packet Loss: {:.2}%", test.packet_loss);
    println!("Test Duration: {}", format_duration(test.duration));
    
    let quality = if test.rtt < 50 && test.packet_loss < 1.0 {
        "✅ Excellent".green()
    } else if test.rtt < 200 && test.packet_loss < 5.0 {
        "⚠️ Good".yellow()
    } else {
        "❌ Poor".red()
    };
    
    println!("Connection Quality: {}", quality);
}

/// Print multiple bandwidth test results
pub fn print_bandwidth_tests(tests: &[BandwidthTest]) {
    if tests.is_empty() {
        print_info("No bandwidth tests performed");
        return;
    }

    println!("{}", "🚀 Bandwidth Test Summary".bold().cyan());
    print_separator();
    
    println!("{:<52} {:<12} {:<12} {:<8}", "Peer ID", "Download", "Upload", "RTT");
    println!("{}", "─".repeat(90).dimmed());
    
    for test in tests {
        let short_id = if test.peer_id.len() >= 8 { &test.peer_id[..8] } else { &test.peer_id };
        println!("{:<52} {:<12} {:<12} {}ms", 
                 format!("{}...", short_id).dimmed(),
                 format_file_size(test.download_speed),
                 format_file_size(test.upload_speed),
                 test.rtt);
    }
}