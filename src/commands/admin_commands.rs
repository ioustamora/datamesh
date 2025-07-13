use anyhow::Result;
/// Administrative command handlers
///
/// This module contains handlers for administrative operations:
/// config, metrics, networks, shell completion
use std::error::Error;
use std::path::PathBuf;

use crate::commands::{CommandContext, CommandHandler};
use crate::{config, performance, presets};
use clap_complete::{generate, Shell};
use clap::CommandFactory;
use colored::Colorize;

/// Config command handler
#[derive(Debug, Clone)]
pub struct ConfigCommand {
    pub generate: bool,
    pub config_path: Option<PathBuf>,
}

#[async_trait::async_trait]
impl CommandHandler for ConfigCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        if self.generate {
            let path = self
                .config_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("datamesh.toml"));
            config::generate_config(&path)?;
            println!("Generated default configuration at: {}", path.display());
        } else {
            let path = self
                .config_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("datamesh.toml"));
            let config = config::Config::load_or_default(Some(path.clone()))?;
            println!("Configuration loaded from: {}", path.display());
            println!("{:#?}", config);
        }
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "admin_config"
    }
}

/// Metrics command handler
#[derive(Debug, Clone)]
pub struct MetricsCommand {
    pub summary: bool,
    pub export: bool,
}

#[async_trait::async_trait]
impl CommandHandler for MetricsCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        let monitor = performance::global_monitor();

        if self.summary {
            let summary = monitor.get_summary();
            println!("📊 Performance Summary");
            println!("─────────────────────");

            // Calculate totals across all operations
            let total_operations: usize = summary.values().map(|s| s.total_operations).sum();
            let total_successful: usize = summary.values().map(|s| s.successful_operations).sum();
            let overall_success_rate = if total_operations > 0 {
                (total_successful as f64 / total_operations as f64) * 100.0
            } else {
                0.0
            };
            let avg_duration: f64 = if !summary.is_empty() {
                summary.values().map(|s| s.avg_duration_ms).sum::<f64>() / summary.len() as f64
            } else {
                0.0
            };

            println!("Total Operations: {}", total_operations);
            println!("Success Rate: {:.1}%", overall_success_rate);
            println!("Average Duration: {:.2}ms", avg_duration);

            // Show recent operations from the performance monitor
            let recent_metrics = monitor.get_recent_metrics(10);
            if !recent_metrics.is_empty() {
                println!("\n📈 Recent Operations:");
                println!(
                    "{:<15} {:<12} {:<10} {:<8} {:<20}",
                    "Operation", "Duration", "Success", "Bytes", "Timestamp"
                );
                println!("{}", "─".repeat(70));

                for metric in &recent_metrics {
                    let duration_str = format!("{:.2}ms", metric.duration_ms);
                    let success_str = if metric.success { "✓" } else { "✗" };
                    let bytes_str = if let Some(bytes) = metric.bytes_processed {
                        format!("{}", bytes)
                    } else {
                        "-".to_string()
                    };
                    let timestamp_str = metric.timestamp.format("%H:%M:%S");

                    println!(
                        "{:<15} {:<12} {:<10} {:<8} {:<20}",
                        metric.operation, duration_str, success_str, bytes_str, timestamp_str
                    );
                }
            } else {
                println!("\nNo recent operations recorded");
            }
        }

        if self.export {
            let metrics_json = monitor.export_metrics();
            println!("Metrics JSON:");
            println!("{}", metrics_json);
        }

        if !self.summary && !self.export {
            println!("Use --summary to show performance summary or --export to export metrics");
        }

        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "admin_metrics"
    }
}

/// Networks command handler
#[derive(Debug, Clone)]
pub struct NetworksCommand;

#[async_trait::async_trait]
impl CommandHandler for NetworksCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        println!("🌐 Available Network Configurations");
        println!("────────────────────────────────────");

        let networks = presets::get_available_networks();
        for (name, network) in networks {
            println!("\n📡 {}", name);
            println!("   Description: {}", network.description);
            println!("   Bootstrap Nodes: {}", network.bootstrap_nodes.len());

            for (i, node) in network.bootstrap_nodes.iter().enumerate() {
                println!("   {}. {}", i + 1, node);
            }

            if let Some(ref features) = network.features {
                println!("   Features: {}", features.join(", "));
            }
        }

        println!("\n💡 Use --network <name> to connect to a specific network");
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "admin_networks"
    }
}

/// Shell completion command handler
#[derive(Debug, Clone)]
pub struct CompletionCommand {
    pub shell: Shell,
}

#[async_trait::async_trait]
impl CommandHandler for CompletionCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        let mut cmd = crate::cli::Cli::command();
        let bin_name = cmd.get_name().to_string();
        
        generate(self.shell, &mut cmd, bin_name, &mut std::io::stdout());
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "admin_completion"
    }
}

/// Help command handler
#[derive(Debug, Clone)]
pub struct HelpCommand;

#[async_trait::async_trait]
impl CommandHandler for HelpCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<(), Box<dyn Error>> {
        use crate::ui;
        
        println!("{}", "DataMesh Quick Help".bold().cyan());
        println!("{}", "===================".cyan());
        println!();
        
        ui::print_shortcuts();
        println!();
        
        println!("{}", "Common Workflows:".bold().green());
        ui::print_command_hint("datamesh put myfile.txt", "Store a file");
        ui::print_command_hint("datamesh get myfile.txt ./", "Download a file");
        ui::print_command_hint("datamesh ls", "List your files");
        ui::print_command_hint("datamesh find 'keyword'", "Search for files");
        ui::print_command_hint("datamesh stats", "Show storage statistics");
        ui::print_command_hint("datamesh peers", "Show connected peers");
        println!();
        
        println!("{}", "Getting Started:".bold().yellow());
        println!("  1. First run: {} to start a bootstrap node", "datamesh bootstrap".cyan());
        println!("  2. In another terminal: {} to connect", "datamesh interactive".cyan());
        println!("  3. Use {} to see all commands", "datamesh --help".cyan());
        println!();
        
        println!("{}", "Shell Completion:".bold().blue());
        println!("  Add to ~/.bashrc: {}", "eval \"$(datamesh generate-completion bash)\"".cyan());
        println!("  Add to ~/.zshrc:  {}", "eval \"$(datamesh generate-completion zsh)\"".cyan());
        
        Ok(())
    }

    fn command_name(&self) -> &'static str {
        "admin_help"
    }
}
