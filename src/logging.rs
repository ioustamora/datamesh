use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Initialize logging system for the DataMesh application
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default log level based on debug/release build
        if cfg!(debug_assertions) {
            EnvFilter::new("datamesh=debug,libp2p=info")
        } else {
            EnvFilter::new("datamesh=info,libp2p=warn")
        }
    });

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    info!("DataMesh logging initialized");
    Ok(())
}

/// Initialize logging with fallback behavior if setup fails
pub fn init_logging_safe() {
    if let Err(e) = init_logging() {
        eprintln!(
            "Warning: Failed to set up advanced logging: {}. Using basic logging.",
            e
        );
        // Fallback to basic println-style logging if tracing setup fails
        eprintln!("DataMesh logging initialized with fallback");
    }
}

/// Log network events
pub fn log_network_event(event: &str, details: &str) {
    info!(target: "dfs::network", "{}: {}", event, details);
}

/// Log file operations
pub fn log_file_operation(operation: &str, file_name: &str, details: &str) {
    info!(target: "dfs::file", "{} {}: {}", operation, file_name, details);
}

/// Log key management operations
pub fn log_key_operation(operation: &str, key_name: &str) {
    info!(target: "dfs::keys", "{}: {}", operation, key_name);
}

/// Log error with context
pub fn log_error_with_context(context: &str, error: &dyn std::error::Error) {
    error!(target: "dfs::error", "{}: {}", context, error);
}

/// Log performance metrics
pub fn log_performance(operation: &str, duration_ms: u64, bytes: Option<usize>) {
    if let Some(b) = bytes {
        info!(target: "dfs::perf", "{} completed in {}ms, {} bytes", operation, duration_ms, b);
    } else {
        info!(target: "dfs::perf", "{} completed in {}ms", operation, duration_ms);
    }
}
