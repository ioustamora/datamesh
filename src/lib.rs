/// DataMesh Library
///
/// This library provides the core functionality for the DataMesh distributed
/// storage system, including key management, file storage, networking, and
/// database operations.

pub mod key_manager;
pub mod encrypted_key_manager;
pub mod audit_logger;
pub mod file_storage;
pub mod network;
pub mod cli;
pub mod interactive;
pub mod error;
pub mod error_handling;
pub mod logging;
pub mod config;
pub mod resilience;
pub mod performance;
pub mod database;
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

pub use key_manager::KeyManager;
pub use database::DatabaseManager;
pub use error::{DfsError, DfsResult};
pub use performance::PerformanceMonitor;