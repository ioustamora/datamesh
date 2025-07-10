/// Error Handling Utilities
///
/// This module provides enhanced error handling with contextual information
/// and actionable suggestions for users.

use crate::error::{DfsError, EnhancedError};
use crate::ui;
use std::collections::HashMap;

/// Error severity levels for better error categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Critical errors that require immediate attention
    Critical,
    /// Errors that should be addressed but don't prevent operation
    Warning,
    /// Informational errors for debugging
    Info,
}

/// Batch error collector for operations that can produce multiple errors
#[derive(Debug)]
pub struct ErrorBatch {
    errors: Vec<EnhancedError>,
    context: String,
}

impl ErrorBatch {
    /// Create a new error batch with context
    pub fn new(context: String) -> Self {
        Self {
            errors: Vec::new(),
            context,
        }
    }
    
    /// Add an error to the batch
    pub fn add_error(&mut self, error: EnhancedError) {
        self.errors.push(error);
    }
    
    /// Get all errors in the batch
    pub fn errors(&self) -> &[EnhancedError] {
        &self.errors
    }
    
    /// Get errors grouped by severity
    pub fn errors_by_severity(&self) -> HashMap<ErrorSeverity, Vec<&EnhancedError>> {
        let mut grouped = HashMap::new();
        for error in &self.errors {
            let severity = get_error_severity(&error.error);
            grouped.entry(severity).or_insert_with(Vec::new).push(error);
        }
        grouped
    }
    
    /// Check if there are any critical errors
    pub fn has_critical_errors(&self) -> bool {
        self.errors.iter().any(|e| get_error_severity(&e.error) == ErrorSeverity::Critical)
    }
    
    /// Get the total error count
    pub fn count(&self) -> usize {
        self.errors.len()
    }
    
    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Get the severity level for a given error
pub fn get_error_severity(error: &DfsError) -> ErrorSeverity {
    match error {
        DfsError::Network(_) => ErrorSeverity::Critical,
        DfsError::FileNotFound(_) => ErrorSeverity::Warning,
        DfsError::Storage(_) => ErrorSeverity::Critical,
        DfsError::Database(_) => ErrorSeverity::Critical,
        DfsError::Crypto(_) => ErrorSeverity::Critical,
        DfsError::Io(io_err) => {
            match io_err.kind() {
                std::io::ErrorKind::PermissionDenied => ErrorSeverity::Critical,
                std::io::ErrorKind::NotFound => ErrorSeverity::Warning,
                std::io::ErrorKind::ConnectionRefused 
                | std::io::ErrorKind::ConnectionReset 
                | std::io::ErrorKind::ConnectionAborted => ErrorSeverity::Critical,
                _ => ErrorSeverity::Warning,
            }
        }
        DfsError::Generic(_) => ErrorSeverity::Info,
        DfsError::Serialization(_) => ErrorSeverity::Warning,
        DfsError::KeyManagement(_) => ErrorSeverity::Critical,
        DfsError::Share(_) => ErrorSeverity::Warning,
        DfsError::Export(_) => ErrorSeverity::Warning,
        DfsError::Import(_) => ErrorSeverity::Warning,
        DfsError::Authentication(_) => ErrorSeverity::Critical,
        DfsError::Config(_) => ErrorSeverity::Critical,
        DfsError::Backup(_) => ErrorSeverity::Warning,
    }
}

/// Display error batch with proper formatting
pub fn display_error_batch(batch: &ErrorBatch) {
    if batch.is_empty() {
        return;
    }
    
    ui::print_error(&format!("Error batch: {}", batch.context));
    
    let grouped = batch.errors_by_severity();
    
    // Display critical errors first
    if let Some(critical_errors) = grouped.get(&ErrorSeverity::Critical) {
        ui::print_error(&format!("Critical errors ({}): ", critical_errors.len()));
        for error in critical_errors {
            display_enhanced_error(error);
        }
    }
    
    // Then warnings
    if let Some(warnings) = grouped.get(&ErrorSeverity::Warning) {
        ui::print_warning(&format!("Warnings ({}): ", warnings.len()));
        for error in warnings {
            display_enhanced_error(error);
        }
    }
    
    // Finally info messages
    if let Some(info_errors) = grouped.get(&ErrorSeverity::Info) {
        ui::print_info(&format!("Info messages ({}): ", info_errors.len()));
        for error in info_errors {
            display_enhanced_error(error);
        }
    }
}

/// Enhanced error handler that provides context and suggestions
pub fn handle_error(error: &(dyn std::error::Error + 'static)) -> EnhancedError {
    // Try to downcast to known error types first for better handling
    let dfs_error = if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => DfsError::FileNotFound(error.to_string()),
            std::io::ErrorKind::PermissionDenied => DfsError::Io(
                std::io::Error::new(io_err.kind(), io_err.to_string())
            ),
            std::io::ErrorKind::ConnectionRefused 
            | std::io::ErrorKind::ConnectionReset 
            | std::io::ErrorKind::ConnectionAborted => DfsError::Network(error.to_string()),
            _ => DfsError::Io(
                std::io::Error::new(io_err.kind(), io_err.to_string())
            ),
        }
    } else if let Some(dfs_err) = error.downcast_ref::<DfsError>() {
        // If it's already a DfsError, recreate it to avoid cloning
        match dfs_err {
            DfsError::Io(io_err) => DfsError::Io(
                std::io::Error::new(io_err.kind(), io_err.to_string())
            ),
            DfsError::Network(msg) => DfsError::Network(msg.clone()),
            DfsError::Crypto(msg) => DfsError::Crypto(msg.clone()),
            DfsError::Serialization(msg) => DfsError::Serialization(msg.clone()),
            DfsError::KeyManagement(msg) => DfsError::KeyManagement(msg.clone()),
            DfsError::Storage(msg) => DfsError::Storage(msg.clone()),
            DfsError::FileNotFound(msg) => DfsError::FileNotFound(msg.clone()),
            DfsError::Database(msg) => DfsError::Database(msg.clone()),
            DfsError::Share(msg) => DfsError::Share(msg.clone()),
            DfsError::Export(msg) => DfsError::Export(msg.clone()),
            DfsError::Import(msg) => DfsError::Import(msg.clone()),
            DfsError::Generic(msg) => DfsError::Generic(msg.clone()),
            DfsError::Authentication(msg) => DfsError::Authentication(msg.clone()),
            DfsError::Config(msg) => DfsError::Config(msg.clone()),
            DfsError::Backup(msg) => DfsError::Backup(msg.clone()),
        }
    } else {
        // Fall back to string analysis for unknown error types
        classify_error_by_string(&error.to_string())
    };
    
    let enhanced = EnhancedError::new(dfs_error);
    add_context_and_suggestions(enhanced)
}


/// Classify errors based on string content (fallback method)
fn classify_error_by_string(error_str: &str) -> DfsError {
    let error_str = error_str.to_lowercase();
    
    if error_str.contains("network") || error_str.contains("connection") {
        DfsError::Network(error_str.to_string())
    } else if error_str.contains("file") && error_str.contains("not found") {
        DfsError::FileNotFound(error_str.to_string())
    } else if error_str.contains("permission") || error_str.contains("access") {
        DfsError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            error_str.to_string()
        ))
    } else if error_str.contains("storage") && error_str.contains("already taken") {
        DfsError::Storage(error_str.to_string())
    } else if error_str.contains("database") || error_str.contains("sql") {
        DfsError::Database(error_str.to_string())
    } else {
        DfsError::Generic(error_str.to_string())
    }
}

/// Add context and suggestions based on error type
pub fn add_context_and_suggestions(mut error: EnhancedError) -> EnhancedError {
    match &error.error {
        DfsError::Network(_) => {
            error = error
                .with_context("Network connectivity issue".to_string())
                .with_suggestions(vec![
                    "Check if bootstrap nodes are running".to_string(),
                    "Verify network connectivity".to_string(),
                    "Try using --bootstrap-peer and --bootstrap-addr options".to_string(),
                    "Run 'dfs bootstrap' in another terminal".to_string(),
                ]);
        },
        DfsError::FileNotFound(msg) => {
            if msg.contains("File not found:") {
                error = error
                    .with_context("File identifier not recognized".to_string())
                    .with_suggestions(vec![
                        "Check the file name or key for typos".to_string(),
                        "Use 'dfs list' to see available files".to_string(),
                        "Use 'dfs info <name>' to get file details".to_string(),
                    ]);
            } else {
                error = error
                    .with_context("File system access issue".to_string())
                    .with_suggestions(vec![
                        "Check if the file path exists".to_string(),
                        "Verify file permissions".to_string(),
                        "Use absolute path if needed".to_string(),
                    ]);
            }
        },
        DfsError::Io(io_error) => {
            match io_error.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    error = error
                        .with_context("Permission denied".to_string())
                        .with_suggestions(vec![
                            "Check file/directory permissions".to_string(),
                            "Run with appropriate user privileges".to_string(),
                            "Ensure the directory is writable".to_string(),
                        ]);
                },
                std::io::ErrorKind::NotFound => {
                    error = error
                        .with_context("File or directory not found".to_string())
                        .with_suggestions(vec![
                            "Check the file path spelling".to_string(),
                            "Ensure the file exists".to_string(),
                            "Use absolute path if needed".to_string(),
                        ]);
                },
                std::io::ErrorKind::AlreadyExists => {
                    error = error
                        .with_context("File already exists".to_string())
                        .with_suggestions(vec![
                            "Choose a different output filename".to_string(),
                            "Remove the existing file first".to_string(),
                            "Use a different directory".to_string(),
                        ]);
                },
                _ => {
                    error = error
                        .with_context("I/O operation failed".to_string())
                        .with_suggestions(vec![
                            "Check disk space availability".to_string(),
                            "Verify file system health".to_string(),
                            "Try the operation again".to_string(),
                        ]);
                }
            }
        },
        DfsError::Storage(msg) => {
            if msg.contains("already taken") {
                error = error
                    .with_context("File name conflict".to_string())
                    .with_suggestions(vec![
                        "Choose a different name with --name option".to_string(),
                        "Use 'dfs list' to see existing file names".to_string(),
                        "Let DFS auto-generate a name by omitting --name".to_string(),
                    ]);
            } else if msg.contains("Reed-Solomon") {
                error = error
                    .with_context("Data integrity issue".to_string())
                    .with_suggestions(vec![
                        "Check if enough peers are online".to_string(),
                        "Retry the operation".to_string(),
                        "Verify file hasn't been corrupted".to_string(),
                    ]);
            } else {
                error = error
                    .with_context("Storage operation failed".to_string())
                    .with_suggestions(vec![
                        "Check network connectivity".to_string(),
                        "Ensure sufficient peers are available".to_string(),
                        "Try the operation again".to_string(),
                    ]);
            }
        },
        DfsError::Crypto(_) => {
            error = error
                .with_context("Cryptographic operation failed".to_string())
                .with_suggestions(vec![
                    "Check if the key file is corrupted".to_string(),
                    "Verify the correct private key is being used".to_string(),
                    "Try regenerating keys if they're damaged".to_string(),
                ]);
        },
        DfsError::Database(_) => {
            error = error
                .with_context("Database operation failed".to_string())
                .with_suggestions(vec![
                    "Check if database file is accessible".to_string(),
                    "Verify disk space availability".to_string(),
                    "Try running the command again".to_string(),
                ]);
        },
        DfsError::KeyManagement(_) => {
            error = error
                .with_context("Key management issue".to_string())
                .with_suggestions(vec![
                    "Check if keys directory exists and is writable".to_string(),
                    "Use --keys-dir to specify custom location".to_string(),
                    "Run in --non-interactive mode to auto-generate keys".to_string(),
                ]);
        },
        _ => {
            error = error
                .with_suggestions(vec![
                    "Try the operation again".to_string(),
                    "Check the command syntax with --help".to_string(),
                    "Verify all required parameters are provided".to_string(),
                ]);
        }
    }
    
    error
}

/// Display an enhanced error with suggestions
pub fn display_enhanced_error(error: &EnhancedError) {
    let error_msg = if let Some(context) = &error.context {
        format!("{}: {}", error.error, context)
    } else {
        error.error.to_string()
    };
    
    if error.suggestions.is_empty() {
        ui::print_error(&error_msg);
    } else {
        let suggestions: Vec<&str> = error.suggestions.iter().map(|s| s.as_str()).collect();
        ui::print_error_with_suggestions(&error_msg, &suggestions);
    }
}

/// Helper macro for creating enhanced errors with context
#[macro_export]
macro_rules! enhanced_error {
    ($error_type:expr, $msg:expr) => {
        crate::error_handling::add_context_and_suggestions(
            crate::error::EnhancedError::new($error_type)
        )
    };
    ($error_type:expr, $msg:expr, $context:expr) => {
        crate::error_handling::add_context_and_suggestions(
            crate::error::EnhancedError::new($error_type)
                .with_context($context.to_string())
        )
    };
}

/// Helper function to create network error with suggestions
pub fn network_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Network(message.to_string()))
    )
}

/// Helper function to create file not found error with suggestions  
pub fn file_not_found_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::FileNotFound(message.to_string()))
    )
}

/// Helper function to create storage error with suggestions
pub fn storage_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Storage(message.to_string()))
    )
}

/// Helper function to create share error with suggestions
pub fn share_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Share(message.to_string()))
    )
}

/// Helper function to create export error with suggestions
pub fn export_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Export(message.to_string()))
    )
}

/// Helper function to create import error with suggestions
pub fn import_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Import(message.to_string()))
    )
}

/// Helper function to create database error with suggestions
pub fn database_error_with_suggestions(message: &str) -> EnhancedError {
    add_context_and_suggestions(
        EnhancedError::new(DfsError::Database(message.to_string()))
    )
}

/// Helper function to create contextual error for specific operations
pub fn operation_error_with_context(operation: &str, error: &(dyn std::error::Error + 'static)) -> EnhancedError {
    let mut enhanced = handle_error(error);
    
    match operation {
        "put" => {
            enhanced = enhanced.with_context("File upload failed".to_string());
            if !enhanced.suggestions.iter().any(|s| s.contains("file size") || s.contains("permissions")) {
                enhanced = enhanced.with_suggestions(vec![
                    "Check file size and permissions".to_string(),
                    "Ensure network connectivity".to_string(),
                    "Verify sufficient storage space".to_string(),
                ]);
            }
        },
        "get" => {
            enhanced = enhanced.with_context("File download failed".to_string());
            if !enhanced.suggestions.iter().any(|s| s.contains("identifier")) {
                enhanced = enhanced.with_suggestions(vec![
                    "Verify file identifier is correct".to_string(),
                    "Check network connectivity".to_string(),
                    "Try different output directory".to_string(),
                ]);
            }
        },
        "list" => {
            enhanced = enhanced.with_context("File listing failed".to_string());
        },
        _ => {
            enhanced = enhanced.with_context(format!("Operation '{}' failed", operation));
        }
    }
    
    enhanced
}