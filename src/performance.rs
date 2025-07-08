use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Performance metrics for DFS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub bytes_processed: Option<usize>,
    pub success: bool,
    pub error_type: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Performance monitor for tracking operation metrics
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    operation_counters: Arc<Mutex<HashMap<String, OperationStats>>>,
}

#[derive(Debug, Clone)]
struct OperationStats {
    total_operations: usize,
    successful_operations: usize,
    total_duration_ms: u64,
    total_bytes: usize,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            operation_counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start timing an operation
    pub fn start_operation(&self, operation: &str) -> OperationTimer {
        OperationTimer {
            operation: operation.to_string(),
            start_time: Instant::now(),
            monitor: self.clone(),
        }
    }

    /// Record a completed operation
    pub fn record_operation(
        &self,
        operation: &str,
        duration: Duration,
        bytes_processed: Option<usize>,
        success: bool,
        error_type: Option<String>,
    ) {
        let metric = PerformanceMetrics {
            operation: operation.to_string(),
            duration_ms: duration.as_millis() as u64,
            bytes_processed,
            success,
            error_type: error_type.clone(),  // Clone to avoid borrow checker issues
            timestamp: chrono::Local::now(),
        };

        // Log the metric
        if success {
            if let Some(bytes) = bytes_processed {
                info!(
                    target: "dfs::performance",
                    "Operation '{}' completed in {}ms, {} bytes processed",
                    operation,
                    metric.duration_ms,
                    bytes
                );
            } else {
                info!(
                    target: "dfs::performance",
                    "Operation '{}' completed in {}ms",
                    operation,
                    metric.duration_ms
                );
            }
        } else {
            info!(
                target: "dfs::performance",
                "Operation '{}' failed after {}ms: {:?}",
                operation,
                metric.duration_ms,
                error_type
            );
        }

        // Store the metric
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.push(metric);

            // Keep only the last 1000 metrics to prevent memory bloat
            if metrics.len() > 1000 {
                metrics.remove(0);
            }
        }

        // Update operation counters
        {
            let mut counters = self.operation_counters.lock().unwrap();
            let stats = counters.entry(operation.to_string()).or_insert(OperationStats {
                total_operations: 0,
                successful_operations: 0,
                total_duration_ms: 0,
                total_bytes: 0,
            });

            stats.total_operations += 1;
            if success {
                stats.successful_operations += 1;
            }
            stats.total_duration_ms += duration.as_millis() as u64;
            if let Some(bytes) = bytes_processed {
                stats.total_bytes += bytes;
            }
        }
    }

    /// Get summary statistics for all operations
    pub fn get_summary(&self) -> HashMap<String, OperationSummary> {
        let counters = self.operation_counters.lock().unwrap();
        let mut summary = HashMap::new();

        for (operation, stats) in counters.iter() {
            let success_rate = if stats.total_operations > 0 {
                (stats.successful_operations as f64 / stats.total_operations as f64) * 100.0
            } else {
                0.0
            };

            let avg_duration_ms = if stats.total_operations > 0 {
                stats.total_duration_ms as f64 / stats.total_operations as f64
            } else {
                0.0
            };

            let avg_throughput_bps = if stats.total_duration_ms > 0 && stats.total_bytes > 0 {
                (stats.total_bytes as f64 * 1000.0) / stats.total_duration_ms as f64
            } else {
                0.0
            };

            summary.insert(operation.clone(), OperationSummary {
                operation: operation.clone(),
                total_operations: stats.total_operations,
                successful_operations: stats.successful_operations,
                success_rate,
                total_duration_ms: stats.total_duration_ms,
                avg_duration_ms,
                total_bytes: stats.total_bytes,
                avg_throughput_bps,
            });
        }

        summary
    }

    /// Get recent metrics (last N operations)
    pub fn get_recent_metrics(&self, count: usize) -> Vec<PerformanceMetrics> {
        let metrics = self.metrics.lock().unwrap();
        let start = if metrics.len() > count {
            metrics.len() - count
        } else {
            0
        };
        metrics[start..].to_vec()
    }

    /// Print performance summary to console
    pub fn print_summary(&self) {
        let summary = self.get_summary();
        
        println!("\n🚀 Performance Summary");
        println!("======================");
        
        for (operation, stats) in summary.iter() {
            println!("📊 Operation: {}", operation);
            println!("   Total: {} operations", stats.total_operations);
            println!("   Success Rate: {:.1}%", stats.success_rate);
            println!("   Avg Duration: {:.1}ms", stats.avg_duration_ms);
            if stats.total_bytes > 0 {
                println!("   Total Data: {} bytes", stats.total_bytes);
                println!("   Avg Throughput: {:.1} bytes/sec", stats.avg_throughput_bps);
            }
            println!();
        }
    }

    /// Export metrics to JSON for external analysis
    pub fn export_metrics(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        serde_json::to_string_pretty(&*metrics).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Timer for tracking operation duration
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
    monitor: PerformanceMonitor,
}

impl OperationTimer {
    /// Complete the operation successfully
    pub fn complete_success(self, bytes_processed: Option<usize>) {
        let duration = self.start_time.elapsed();
        self.monitor.record_operation(
            &self.operation,
            duration,
            bytes_processed,
            true,
            None,
        );
    }

    /// Complete the operation with failure
    pub fn complete_failure(self, error_type: String) {
        let duration = self.start_time.elapsed();
        self.monitor.record_operation(
            &self.operation,
            duration,
            None,
            false,
            Some(error_type),
        );
    }

    /// Complete the operation with custom result
    pub fn complete(self, success: bool, bytes_processed: Option<usize>, error_type: Option<String>) {
        let duration = self.start_time.elapsed();
        self.monitor.record_operation(
            &self.operation,
            duration,
            bytes_processed,
            success,
            error_type,
        );
    }
}

/// Summary statistics for an operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    pub operation: String,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub total_bytes: usize,
    pub avg_throughput_bps: f64,
}

/// Global performance monitor instance
static GLOBAL_MONITOR: std::sync::OnceLock<PerformanceMonitor> = std::sync::OnceLock::new();

/// Get or initialize the global performance monitor
pub fn global_monitor() -> &'static PerformanceMonitor {
    GLOBAL_MONITOR.get_or_init(PerformanceMonitor::new)
}

/// Convenience function to start timing an operation globally
pub fn start_operation(operation: &str) -> OperationTimer {
    global_monitor().start_operation(operation)
}

/// Convenience macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($op:expr, $block:block) => {
        {
            let timer = $crate::performance::start_operation($op);
            let result = $block;
            match &result {
                Ok(_) => timer.complete_success(None),
                Err(e) => timer.complete_failure(e.to_string()),
            }
            result
        }
    };
    ($op:expr, $bytes:expr, $block:block) => {
        {
            let timer = $crate::performance::start_operation($op);
            let result = $block;
            match &result {
                Ok(_) => timer.complete_success(Some($bytes)),
                Err(e) => timer.complete_failure(e.to_string()),
            }
            result
        }
    };
}
