pub mod actor_file_storage;
pub mod advanced_commands;
pub mod api_server;
pub mod audit_logger;
pub mod backup_system;
pub mod batch_operations;
pub mod billing_system;
pub mod bootstrap_admin;
pub mod bootstrap_manager;
pub mod cli;
pub mod commands;
pub mod concurrent_chunks;
pub mod config;
pub mod database;
pub mod datamesh_core;
pub mod economics;
pub mod encrypted_key_manager;
pub mod error;
pub mod error_handling;
pub mod failover;
pub mod file_manager;
pub mod file_storage;
pub mod governance;
pub mod governance_service;
pub mod health_manager;
pub mod high_performance;
pub mod interactive;
pub mod key_manager;
pub mod key_rotation;
pub mod load_balancer;
pub mod logging;
pub mod monitoring;
pub mod network;
pub mod network_actor;
pub mod network_diagnostics;
pub mod performance;
pub mod performance_optimizer;
pub mod persistent_dht;
pub mod presets;
pub mod quorum_manager;
pub mod quota_service;
pub mod resilience;
/// DataMesh Library
///
/// This library provides the core functionality for the DataMesh distributed
/// storage system, including key management, file storage, networking, and
/// database operations.
pub mod secure_random;
pub mod secure_transport;
pub mod smart_cache;
pub mod thread_safe_command_context;
pub mod thread_safe_database;
pub mod thread_safe_file_commands;
pub mod ui;

pub use backup_system::{
    BackupConfig, BackupDestination, BackupSystem, BackupType, RestoreOptions,
};
pub use database::DatabaseManager;
pub use error::{DfsError, DfsResult};
pub use key_manager::KeyManager;
pub use performance::PerformanceMonitor;
