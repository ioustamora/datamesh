use std::fmt;
use std::error::Error as StdError;

/// Custom error types for the DFS system
#[derive(Debug)]
pub enum DfsError {
    /// IO related errors
    Io(String),
    /// Network related errors
    Network(String),
    /// Encryption/Decryption errors
    Crypto(String),
    /// Serialization errors
    Serialization(String),
    /// Key management errors
    KeyManagement(String),
    /// File storage errors
    Storage(String),
    /// File not found errors
    FileNotFound(String),
    /// Database errors
    Database(String),
    /// Share operation errors
    Share(String),
    /// Export operation errors
    Export(String),
    /// Import operation errors
    Import(String),
    /// Encryption errors
    Encryption(String),
    /// Decryption errors
    Decryption(String),
    /// Encoding errors
    Encoding(String),
    /// Deserialization errors
    Deserialization(String),
    /// Not found errors
    NotFound(String),
    /// Configuration errors
    Config(String),
    /// Authentication and authorization errors
    Authentication(String),
    /// Backup operation errors
    Backup(String),
    /// Generic errors
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
            DfsError::Backup(e) => write!(f, "Backup error: {}", e),
            DfsError::Generic(e) => write!(f, "Error: {}", e),
            DfsError::Encryption(e) => write!(f, "Encryption error: {}", e),
            DfsError::Decryption(e) => write!(f, "Decryption error: {}", e),
            DfsError::Encoding(e) => write!(f, "Encoding error: {}", e),
            DfsError::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            DfsError::NotFound(e) => write!(f, "Not found: {}", e),
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
