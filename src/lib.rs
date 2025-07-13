// ===================================================================================================
// DataMesh Library - Core Distributed Storage System
// ===================================================================================================
//
// This library provides the complete functionality for the DataMesh distributed storage system.
// It's designed to be both a standalone application and a library that can be embedded in other
// Rust applications requiring secure, fault-tolerant distributed file storage.
//
// ## KEY DESIGN PRINCIPLES
//
// ### 1. Thread Safety First
// All core components are designed to be thread-safe from the ground up:
// - Actor-based networking isolates libp2p Swarm operations
// - Thread-safe wrappers for database and file operations
// - Async/await throughout for non-blocking I/O
//
// ### 2. Security by Default
// - ECIES encryption for all stored data
// - BLAKE3 hashing for integrity verification
// - Secure key management with multiple key support
// - Transport-level security for network communications
//
// ### 3. Fault Tolerance
// - Reed-Solomon erasure coding (4+2 shards) for data redundancy
// - Automatic failover and recovery mechanisms
// - Resilient network connectivity with bootstrap peer management
// - Health monitoring and self-healing capabilities
//
// ### 4. Performance Optimization
// - Concurrent chunk processing for large files
// - Intelligent caching systems
// - Performance monitoring and automatic optimization
// - Zero-copy operations where possible
//
// ### 5. Modular Architecture
// Each module has a specific responsibility and well-defined interfaces:
// - Storage layer handles file operations and encryption
// - Network layer manages P2P communication and DHT operations
// - Command layer provides CLI and API interfaces
// - Monitoring layer tracks performance and health
//
// ===================================================================================================

// ===== CORE STORAGE AND FILE MANAGEMENT =====
pub mod actor_file_storage;       // Actor-based file storage with thread-safe network communication
pub mod file_storage;             // Traditional file storage implementation with Reed-Solomon erasure coding
pub mod file_manager;             // High-level file management operations and metadata handling
pub mod concurrent_chunks;        // Parallel chunk processing for optimal large file performance
pub mod backup_system;            // Comprehensive data backup and recovery system
pub mod storage_economy;          // Storage economy system with contribution tiers and verification

// ===== NETWORK AND P2P COMMUNICATION =====
pub mod network;                  // Core libp2p networking setup and Kademlia DHT configuration
pub mod network_actor;            // Actor-based thread-safe network communication layer
pub mod network_diagnostics;      // Network health monitoring and diagnostic tools
pub mod bootstrap_manager;        // Bootstrap peer management and automatic connectivity
pub mod persistent_dht;           // Persistent DHT state management for faster startup
pub mod secure_transport;         // Secure network transport layer with encryption

// ===== COMMAND LINE INTERFACE AND USER INTERACTION =====
pub mod cli;                      // Command line argument parsing and validation using clap
pub mod commands;                 // Command handler implementations with clean separation
pub mod interactive;              // Interactive mode and user interface components
pub mod ui;                       // User interface utilities, progress indicators, and formatting
pub mod advanced_commands;        // Advanced administrative commands for power users

// ===== SECURITY AND CRYPTOGRAPHY =====
pub mod key_manager;              // ECIES key management and cryptographic operations
pub mod encrypted_key_manager;    // Enhanced key management with additional encryption layers
pub mod secure_random;            // Cryptographically secure random number generation
pub mod key_rotation;             // Automatic key rotation and security policy enforcement

// ===== SYSTEM CONFIGURATION AND MANAGEMENT =====
pub mod config;                   // Configuration management with TOML support and validation
pub mod presets;                  // Network preset configurations for easy deployment
pub mod database;                 // SQLite database operations for metadata and file tracking
pub mod thread_safe_database;     // Thread-safe database wrapper with connection pooling

// ===== PERFORMANCE AND MONITORING =====
pub mod performance;              // Performance monitoring and metrics collection
pub mod performance_optimizer;    // Automatic performance optimization and tuning
pub mod monitoring;               // Comprehensive system monitoring with time-series data
pub mod logging;                  // Structured logging and tracing with multiple output formats

// ===== ADVANCED FEATURES AND ADMINISTRATION =====
pub mod bootstrap_admin;          // Bootstrap node administration and management tools
pub mod api_server;               // HTTP API server for web interface and REST operations
pub mod governance;               // Decentralized governance system for network administration
pub mod governance_service;       // Governance service implementation with voting mechanisms

// ===== RELIABILITY AND FAULT TOLERANCE =====
pub mod error;                    // Core error types and comprehensive error handling
pub mod error_handling;           // Enhanced error handling with user-friendly messages
pub mod failover;                 // Automatic failover mechanisms for high availability
pub mod resilience;               // System resilience and recovery from partial failures
pub mod health_manager;           // Node health monitoring and automated recovery

// ===== RESOURCE MANAGEMENT =====
pub mod quota_service;            // Storage quota management and enforcement
pub mod economics;                // Economic model and incentive mechanisms
pub mod billing_system;           // Usage tracking and billing for commercial deployments
pub mod load_balancer;            // Intelligent request load balancing and distribution
pub mod quorum_manager;           // Quorum calculation and management for storage operations

// ===== SYSTEM UTILITIES AND HELPERS =====
pub mod thread_safe_command_context;  // Thread-safe command execution context
pub mod thread_safe_file_commands;    // Thread-safe file operation wrappers
pub mod smart_cache;                  // Intelligent caching system with LRU and TTL support
pub mod audit_logger;                 // Security audit logging and compliance tracking
pub mod batch_operations;             // Batch processing capabilities for bulk operations
pub mod datamesh_core;                // Core system functionality and shared utilities
pub mod websocket;                    // WebSocket functionality for real-time updates

// ===== ECONOMY & UX ENHANCEMENT MODULES =====
pub mod dynamic_pricing;          // Dynamic pricing models for storage and retrieval
pub mod flexible_storage;         // Flexible storage solutions and configurations
pub mod gamification;             // Gamification elements to encourage network participation
pub mod intelligent_cli;          // Intelligent command-line interface enhancements
pub mod pricing_assistant;        // Pricing assistant module for cost estimation and optimization
pub mod enhanced_api;            // Enhanced API module for extended functionality

// ===== CONVENIENCE RE-EXPORTS =====
// These re-exports provide easy access to the most commonly used types
// and functions when using DataMesh as a library

/// Backup system components for data protection and recovery
pub use backup_system::{
    BackupConfig, BackupDestination, BackupSystem, BackupType, RestoreOptions,
};

/// Database management for metadata and file tracking
pub use database::DatabaseManager;

/// Core error types and result types for error handling
pub use error::{DfsError, DfsResult};

/// Cryptographic key management for ECIES operations
pub use key_manager::KeyManager;

/// Performance monitoring and metrics collection
pub use performance::PerformanceMonitor;
