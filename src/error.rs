// ===================================================================================================
// Error Handling System - Comprehensive Error Types and Management
// ===================================================================================================
//
// This module defines the comprehensive error handling system for DataMesh, providing
// structured error types that enable proper error categorization, user-friendly error
// messages, and effective debugging and troubleshooting.
//
// ## ERROR DESIGN PRINCIPLES
//
// ### 1. Semantic Error Types
// Each error variant represents a specific category of failure, allowing for:
// - Precise error handling based on error type
// - Targeted recovery strategies
// - Clear user feedback based on error context
// - Effective debugging and logging
//
// ### 2. Descriptive Error Messages
// All error variants include detailed string descriptions that provide:
// - Context about what operation failed
// - Specific details about the failure condition
// - Guidance for resolution where appropriate
// - Technical details for debugging
//
// ### 3. Error Chain Support
// The error system integrates with Rust's standard Error trait, enabling:
// - Error chain propagation for root cause analysis
// - Integration with external error handling libraries
// - Proper error source tracking
// - Standardized error formatting
//
// ### 4. User Experience Focus
// Error messages are designed to be helpful for both users and developers:
// - Clear, non-technical language for user-facing errors
// - Technical details preserved for debugging
// - Actionable suggestions where possible
// - Consistent formatting and presentation
//
// ## ERROR CATEGORIES
//
// The error system is organized into logical categories that map to system components:
//
// ### System-Level Errors
// - IO: File system operations, permissions, disk space
// - Network: Connectivity, timeouts, protocol errors
// - Database: Storage, query failures, corruption
// - Config: Invalid configuration, missing parameters
//
// ### Cryptographic Errors
// - Crypto: General cryptographic operation failures
// - Encryption: Specific encryption operation failures
// - Decryption: Specific decryption operation failures
// - KeyManagement: Key generation, storage, and retrieval
// - Authentication: Identity verification and authorization
//
// ### Storage System Errors
// - Storage: Distributed storage operation failures
// - FileNotFound: Specific file lookup failures
// - Share: File sharing and permission errors
// - Backup: Backup and recovery operation failures
//
// ### Data Processing Errors
// - Serialization: Data encoding for storage/transmission
// - Deserialization: Data decoding from storage/transmission
// - Encoding: General data encoding operations
// - NotFound: General resource lookup failures
//
// ### Application-Level Errors
// - Import: Data import operation failures
// - Export: Data export operation failures
// - Economics: Token and economic model errors
// - Generic: Catch-all for miscellaneous errors
//
// ===================================================================================================

use std::error::Error as StdError;
use std::fmt;

/// Comprehensive error types for the DataMesh distributed storage system.
///
/// This enum provides structured error handling across all system components,
/// enabling precise error categorization and appropriate user feedback.
/// Each variant includes a descriptive message that provides context about
/// the specific failure condition.
///
/// ## Usage Pattern
/// ```rust
/// match result {
///     Ok(value) => { /* handle success */ },
///     Err(DfsError::Network(msg)) => { /* handle network error */ },
///     Err(DfsError::Crypto(msg)) => { /* handle crypto error */ },
///     Err(other) => { /* handle other errors */ },
/// }
/// ```
///
/// ## Error Recovery
/// Different error types may require different recovery strategies:
/// - Network errors: Retry with backoff
/// - Crypto errors: Check key validity
/// - Storage errors: Verify peer connectivity
/// - IO errors: Check permissions and disk space
#[derive(Debug)]
pub enum DfsError {
    // ===== SYSTEM-LEVEL ERRORS =====
    
    /// File system I/O operation failures.
    /// 
    /// Covers file reading/writing, permission issues, disk space problems,
    /// and other file system related errors. Common causes include:
    /// - File not found or insufficient permissions
    /// - Disk full or I/O device errors
    /// - Path resolution failures
    /// - File system corruption
    Io(String),

    /// Network communication and connectivity failures.
    ///
    /// Includes libp2p errors, DHT operation failures, peer connectivity
    /// issues, and protocol-level errors. Common causes include:
    /// - Bootstrap peer unavailable
    /// - DHT operation timeouts
    /// - Network partitions or connectivity loss
    /// - Protocol version mismatches
    Network(String),

    /// Database operation failures.
    ///
    /// Covers SQLite errors, connection issues, query failures, and
    /// data integrity problems. Common causes include:
    /// - Database file corruption
    /// - Schema migration failures
    /// - Concurrent access conflicts
    /// - Disk space or permission issues
    Database(String),

    /// System configuration errors.
    ///
    /// Includes invalid configuration files, missing required parameters,
    /// and configuration validation failures. Common causes include:
    /// - Invalid TOML syntax in configuration files
    /// - Missing required configuration parameters
    /// - Invalid network or security settings
    /// - Configuration file permission issues
    Config(String),

    /// Alternative configuration error variant for compatibility.
    ///
    /// Provides an alternative naming convention for configuration errors
    /// to maintain compatibility with different parts of the codebase.
    Configuration(String),

    /// Bad request or invalid input parameters.
    ///
    /// Covers invalid user input, malformed requests, invalid parameters,
    /// and other client-side errors. Common causes include:
    /// - Invalid proposal types or vote values
    /// - Malformed file names or paths
    /// - Out-of-range parameters
    /// - Invalid user input format
    BadRequest(String),

    // ===== CRYPTOGRAPHIC ERRORS =====

    /// General cryptographic operation failures.
    ///
    /// Covers ECIES encryption/decryption, key generation, and other
    /// cryptographic operations. Common causes include:
    /// - Invalid key material or corrupted keys
    /// - Cryptographic library errors
    /// - Hardware security module failures
    /// - Random number generation issues
    Crypto(String),

    /// Specific encryption operation failures.
    ///
    /// Dedicated error type for encryption operations, providing more
    /// specific error context for encryption-related failures.
    Encryption(String),

    /// Specific decryption operation failures.
    ///
    /// Dedicated error type for decryption operations, often indicating
    /// key mismatches or corrupted encrypted data.
    Decryption(String),

    /// Cryptographic key management failures.
    ///
    /// Covers key generation, storage, retrieval, and validation errors.
    /// Common causes include:
    /// - Key file corruption or unavailability
    /// - Invalid key formats or unsupported key types
    /// - Key derivation failures
    /// - Hardware security module communication errors
    KeyManagement(String),

    /// Authentication and authorization failures.
    ///
    /// Covers identity verification, access control, and permission
    /// validation errors. Common causes include:
    /// - Invalid credentials or expired tokens
    /// - Insufficient permissions for requested operations
    /// - Authentication service unavailability
    /// - Authorization policy violations
    Authentication(String),

    // ===== STORAGE SYSTEM ERRORS =====

    /// Distributed file storage operation failures.
    ///
    /// Covers Reed-Solomon encoding, DHT storage, quorum failures, and
    /// other distributed storage specific errors. Common causes include:
    /// - Insufficient peers for quorum requirements
    /// - Reed-Solomon encoding/decoding failures
    /// - Shard distribution or retrieval failures
    /// - Peer connectivity issues during storage operations
    Storage(String),

    /// File lookup and retrieval failures.
    ///
    /// Specific error type for file not found conditions, providing
    /// clear indication that a requested file is not available.
    FileNotFound(String),

    /// File sharing and permission errors.
    ///
    /// Covers file sharing operations, permission management, and
    /// access control for shared files. Common causes include:
    /// - Invalid sharing permissions or expired shares
    /// - Recipient key unavailability
    /// - Share metadata corruption
    /// - Access policy violations
    Share(String),

    /// Backup and recovery operation failures.
    ///
    /// Covers backup creation, restoration, and backup data integrity
    /// issues. Common causes include:
    /// - Backup storage unavailability
    /// - Backup data corruption or incompatibility
    /// - Insufficient space for backup operations
    /// - Backup encryption/decryption failures
    Backup(String),

    // ===== DATA PROCESSING ERRORS =====

    /// Data serialization failures.
    ///
    /// Covers encoding of data structures for storage or transmission.
    /// Common causes include:
    /// - Invalid data structure format
    /// - Serialization library errors
    /// - Memory allocation failures during serialization
    /// - Data size limitations exceeded
    Serialization(String),

    /// Data deserialization failures.
    ///
    /// Covers decoding of stored or transmitted data structures.
    /// Common causes include:
    /// - Corrupted or incompatible data format
    /// - Version compatibility issues
    /// - Truncated or incomplete data
    /// - Schema validation failures
    Deserialization(String),

    /// General data encoding operation failures.
    ///
    /// Covers various data encoding operations beyond serialization.
    /// Common causes include:
    /// - Character encoding issues
    /// - Base64 or hex encoding errors
    /// - Compression/decompression failures
    /// - Format conversion errors
    Encoding(String),

    /// General resource lookup failures.
    ///
    /// Generic error type for resource not found conditions beyond
    /// specific file lookups. Common causes include:
    /// - Peer ID not found in routing table
    /// - Configuration parameter not found
    /// - Service endpoint unavailability
    /// - Resource identifier invalid or expired
    NotFound(String),

    // ===== APPLICATION-LEVEL ERRORS =====

    /// Data import operation failures.
    ///
    /// Covers importing external data into the system. Common causes include:
    /// - Invalid import data format
    /// - Import source unavailability
    /// - Data validation failures during import
    /// - Storage space limitations
    Import(String),

    /// Data export operation failures.
    ///
    /// Covers exporting system data to external formats. Common causes include:
    /// - Export destination unavailability
    /// - Insufficient permissions for export operations
    /// - Data format conversion errors
    /// - Export size limitations exceeded
    Export(String),

    /// Economic model and token management errors.
    ///
    /// Covers token operations, economic incentives, and payment processing.
    /// Common causes include:
    /// - Insufficient token balance
    /// - Invalid economic transactions
    /// - Token calculation errors
    /// - Economic policy violations
    Economics(String),

    /// Generic catch-all error type.
    ///
    /// Used for errors that don't fit into other specific categories
    /// or for temporary error handling during development.
    Generic(String),
}

/// Enhanced error with context and suggestions
#[derive(Debug)]
pub struct EnhancedError {
    pub error: DfsError,
    pub context: Option<String>,
    pub suggestions: Vec<String>,
}

impl EnhancedError {
    pub fn new(error: DfsError) -> Self {
        Self {
            error,
            context: None,
            suggestions: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
}

impl fmt::Display for DfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DfsError::Io(e) => write!(f, "IO error: {}", e),
            DfsError::Network(e) => write!(f, "Network error: {}", e),
            DfsError::Crypto(e) => write!(f, "Cryptographic error: {}", e),
            DfsError::Serialization(e) => write!(f, "Serialization error: {}", e),
            DfsError::KeyManagement(e) => write!(f, "Key management error: {}", e),
            DfsError::Storage(e) => write!(f, "Storage error: {}", e),
            DfsError::FileNotFound(e) => write!(f, "File not found: {}", e),
            DfsError::Database(e) => write!(f, "Database error: {}", e),
            DfsError::Share(e) => write!(f, "Share error: {}", e),
            DfsError::Export(e) => write!(f, "Export error: {}", e),
            DfsError::Import(e) => write!(f, "Import error: {}", e),
            DfsError::Authentication(e) => write!(f, "Authentication error: {}", e),
            DfsError::Config(e) => write!(f, "Configuration error: {}", e),
            DfsError::Configuration(e) => write!(f, "Configuration error: {}", e),
            DfsError::Backup(e) => write!(f, "Backup error: {}", e),
            DfsError::Economics(e) => write!(f, "Economics error: {}", e),
            DfsError::Generic(e) => write!(f, "Error: {}", e),
            DfsError::Encryption(e) => write!(f, "Encryption error: {}", e),
            DfsError::Decryption(e) => write!(f, "Decryption error: {}", e),
            DfsError::Encoding(e) => write!(f, "Encoding error: {}", e),
            DfsError::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            DfsError::NotFound(e) => write!(f, "Not found: {}", e),
            DfsError::BadRequest(e) => write!(f, "Bad request: {}", e),
        }
    }
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

impl StdError for EnhancedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl StdError for DfsError {}

impl From<std::io::Error> for DfsError {
    fn from(error: std::io::Error) -> Self {
        DfsError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for DfsError {
    fn from(error: serde_json::Error) -> Self {
        DfsError::Serialization(error.to_string())
    }
}

impl From<hex::FromHexError> for DfsError {
    fn from(error: hex::FromHexError) -> Self {
        DfsError::Serialization(format!("Hex decode error: {}", error))
    }
}

impl From<reed_solomon_erasure::Error> for DfsError {
    fn from(error: reed_solomon_erasure::Error) -> Self {
        DfsError::Storage(format!("Reed-Solomon error: {:?}", error))
    }
}

impl From<anyhow::Error> for DfsError {
    fn from(error: anyhow::Error) -> Self {
        DfsError::Generic(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for DfsError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        DfsError::Generic(error.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for DfsError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        DfsError::Generic(error.to_string())
    }
}

impl From<libp2p::kad::store::Error> for DfsError {
    fn from(error: libp2p::kad::store::Error) -> Self {
        DfsError::Network(format!("Kademlia store error: {:?}", error))
    }
}

/// Result type alias for DFS operations
pub type DfsResult<T> = Result<T, DfsError>;
