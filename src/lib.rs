/// DataMesh Library
///
/// This library provides the core functionality for the DataMesh distributed
/// storage system, including key management, file storage, networking, and
/// database operations.

pub mod secure_random;
pub mod secure_transport;
pub mod key_rotation;
pub mod key_manager;
pub mod encrypted_key_manager;
pub mod audit_logger;
pub mod file_storage;
pub mod actor_file_storage;
pub mod network;
pub mod network_actor;
pub mod cli;
pub mod commands;
pub mod interactive;
pub mod error;
pub mod error_handling;
pub mod logging;
pub mod config;
pub mod resilience;
pub mod performance;
pub mod database;
pub mod thread_safe_database;
pub mod thread_safe_command_context;
pub mod thread_safe_file_commands;
pub mod ui;
pub mod presets;
pub mod network_diagnostics;
pub mod file_manager;
pub mod batch_operations;
pub mod health_manager;
pub mod governance;
pub mod quota_service;
pub mod bootstrap_admin;
pub mod governance_service;
pub mod economics;
pub mod persistent_dht;
pub mod bootstrap_manager;
pub mod concurrent_chunks;
pub mod smart_cache;
pub mod api_server;
pub mod load_balancer;
pub mod failover;
pub mod performance_optimizer;
pub mod billing_system;
pub mod datamesh_core;
pub mod advanced_commands;
pub mod backup_system;

pub use key_manager::KeyManager;
pub use database::DatabaseManager;
pub use error::{DfsError, DfsResult};
pub use performance::PerformanceMonitor;
pub use backup_system::{BackupSystem, BackupConfig, BackupType, BackupDestination, RestoreOptions};