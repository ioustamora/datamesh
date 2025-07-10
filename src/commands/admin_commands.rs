/// Administrative command handlers
/// 
/// This module contains handlers for administrative operations:
/// config, metrics, networks

use std::error::Error;
use std::path::PathBuf;
use anyhow::Result;

use crate::commands::{CommandHandler, CommandContext};
use crate::{config, performance, presets};

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
            let path = self.config_path.clone().unwrap_or_else(|| PathBuf::from("datamesh.toml"));
            config::generate_config(&path)?;
            println!("Generated default configuration at: {}", path.display());
        } else {
            let path = self.config_path.clone().unwrap_or_else(|| PathBuf::from("datamesh.toml"));
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
            println!("Total Operations: {}", summary.total_operations);
            println!("Success Rate: {:.1}%", summary.success_rate * 100.0);
            println!("Average Duration: {:.2}ms", summary.avg_duration);
            
            if !summary.recent_metrics.is_empty() {
                println!("\n📈 Recent Operations:");
                println!("{:<15} {:<12} {:<10} {:<8} {:<20}", 
                    "Operation", "Duration", "Success", "Bytes", "Timestamp");
                println!("{}", "─".repeat(70));
                
                for metric in &summary.recent_metrics {
                    let duration_str = if let Some(duration) = metric.duration {
                        format!("{:.2}ms", duration)
                    } else {
                        "pending".to_string()
                    };
                    let success_str = if metric.success { "✓" } else { "✗" };
                    let bytes_str = if let Some(bytes) = metric.bytes_processed {
                        format!("{}", bytes)
                    } else {
                        "-".to_string()
                    };
                    let timestamp_str = metric.timestamp.format("%H:%M:%S");
                    
                    println!("{:<15} {:<12} {:<10} {:<8} {:<20}", 
                        metric.operation, duration_str, success_str, bytes_str, timestamp_str);
                }
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