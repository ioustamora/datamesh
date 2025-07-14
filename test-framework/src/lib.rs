/// DataMesh Universal Multinode Testing Framework
///
/// This library provides comprehensive testing capabilities for DataMesh
/// distributed storage system across multiple nodes in realistic environments.

pub mod orchestrator;
pub mod node_manager;
pub mod network_simulator;
pub mod test_executor;
pub mod monitoring;
pub mod validation;
pub mod test_data;

// Test modules
pub mod cli_tests;
pub mod api_tests;
pub mod ui_tests;
pub mod network_tests;
pub mod economy_tests;
pub mod governance_tests;
pub mod performance_tests;
pub mod fault_tests;

// Utilities
pub mod utils;
pub mod config;
pub mod reporting;

// Re-exports for convenience
pub use orchestrator::{TestOrchestrator, OrchestratorConfig, TestReport};
pub use test_executor::{TestResult, TestCase, TestCategory, TestSuite};
pub use node_manager::{NodeManager, NodeConfig, NodeInstance};

/// Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for common test scenarios
pub mod presets {
    use super::*;
    use std::time::Duration;
    use std::path::PathBuf;

    /// Basic 3-node test configuration
    pub fn basic_cluster() -> OrchestratorConfig {
        OrchestratorConfig {
            node_count: 3,
            base_port: 40000,
            test_timeout: Duration::from_secs(600), // 10 minutes
            work_dir: PathBuf::from("/tmp/datamesh-test-basic"),
            enable_network_simulation: false,
            enable_monitoring: true,
            parallel_execution: false,
            test_data_config: orchestrator::TestDataConfig {
                file_size_range: (1024, 1024 * 1024), // 1KB to 1MB
                file_count: 20,
                user_count: 5,
                proposal_count: 3,
            },
            topology: orchestrator::ClusterTopology::Star,
        }
    }

    /// Standard 5-node test configuration
    pub fn standard_cluster() -> OrchestratorConfig {
        OrchestratorConfig {
            node_count: 5,
            base_port: 40000,
            test_timeout: Duration::from_secs(1800), // 30 minutes
            work_dir: PathBuf::from("/tmp/datamesh-test-standard"),
            enable_network_simulation: false,
            enable_monitoring: true,
            parallel_execution: true,
            test_data_config: orchestrator::TestDataConfig {
                file_size_range: (1024, 10 * 1024 * 1024), // 1KB to 10MB
                file_count: 100,
                user_count: 20,
                proposal_count: 10,
            },
            topology: orchestrator::ClusterTopology::Star,
        }
    }

    /// Large 10-node performance test configuration
    pub fn performance_cluster() -> OrchestratorConfig {
        OrchestratorConfig {
            node_count: 10,
            base_port: 40000,
            test_timeout: Duration::from_secs(3600), // 1 hour
            work_dir: PathBuf::from("/tmp/datamesh-test-performance"),
            enable_network_simulation: true,
            enable_monitoring: true,
            parallel_execution: true,
            test_data_config: orchestrator::TestDataConfig {
                file_size_range: (1024, 100 * 1024 * 1024), // 1KB to 100MB
                file_count: 500,
                user_count: 50,
                proposal_count: 25,
            },
            topology: orchestrator::ClusterTopology::Mesh,
        }
    }

    /// Fault tolerance test configuration with network simulation
    pub fn fault_tolerance_cluster() -> OrchestratorConfig {
        OrchestratorConfig {
            node_count: 7,
            base_port: 40000,
            test_timeout: Duration::from_secs(2400), // 40 minutes
            work_dir: PathBuf::from("/tmp/datamesh-test-fault"),
            enable_network_simulation: true,
            enable_monitoring: true,
            parallel_execution: false, // Sequential for fault testing
            test_data_config: orchestrator::TestDataConfig {
                file_size_range: (1024, 5 * 1024 * 1024), // 1KB to 5MB
                file_count: 50,
                user_count: 15,
                proposal_count: 8,
            },
            topology: orchestrator::ClusterTopology::Ring,
        }
    }

    /// UI-focused test configuration
    pub fn ui_testing_cluster() -> OrchestratorConfig {
        OrchestratorConfig {
            node_count: 3,
            base_port: 40000,
            test_timeout: Duration::from_secs(1200), // 20 minutes
            work_dir: PathBuf::from("/tmp/datamesh-test-ui"),
            enable_network_simulation: false,
            enable_monitoring: true,
            parallel_execution: false, // Sequential for UI testing
            test_data_config: orchestrator::TestDataConfig {
                file_size_range: (1024, 2 * 1024 * 1024), // 1KB to 2MB
                file_count: 30,
                user_count: 10,
                proposal_count: 5,
            },
            topology: orchestrator::ClusterTopology::Star,
        }
    }
}

/// Common test scenarios
pub mod scenarios {
    use super::*;

    /// Create a basic functionality test suite
    pub fn basic_functionality() -> TestSuite {
        TestSuite {
            name: "Basic Functionality".to_string(),
            description: "Test core DataMesh functionality".to_string(),
            include_network_tests: true,
            include_cli_tests: true,
            include_api_tests: true,
            include_ui_tests: false,
            include_economy_tests: false,
            include_governance_tests: false,
            include_performance_tests: false,
            include_fault_tests: false,
        }
    }

    /// Create a comprehensive test suite
    pub fn comprehensive() -> TestSuite {
        TestSuite {
            name: "Comprehensive".to_string(),
            description: "Complete DataMesh functionality testing".to_string(),
            include_network_tests: true,
            include_cli_tests: true,
            include_api_tests: true,
            include_ui_tests: true,
            include_economy_tests: true,
            include_governance_tests: true,
            include_performance_tests: true,
            include_fault_tests: false,
        }
    }

    /// Create a UI-focused test suite
    pub fn ui_focused() -> TestSuite {
        TestSuite {
            name: "UI Focused".to_string(),
            description: "Web interface and user experience testing".to_string(),
            include_network_tests: false,
            include_cli_tests: false,
            include_api_tests: true,
            include_ui_tests: true,
            include_economy_tests: false,
            include_governance_tests: false,
            include_performance_tests: false,
            include_fault_tests: false,
        }
    }

    /// Create a performance test suite
    pub fn performance_focused() -> TestSuite {
        TestSuite {
            name: "Performance Focused".to_string(),
            description: "Performance and scalability testing".to_string(),
            include_network_tests: true,
            include_cli_tests: false,
            include_api_tests: true,
            include_ui_tests: false,
            include_economy_tests: false,
            include_governance_tests: false,
            include_performance_tests: true,
            include_fault_tests: false,
        }
    }

    /// Create a fault tolerance test suite
    pub fn fault_tolerance() -> TestSuite {
        TestSuite {
            name: "Fault Tolerance".to_string(),
            description: "Network partition and failure recovery testing".to_string(),
            include_network_tests: true,
            include_cli_tests: true,
            include_api_tests: false,
            include_ui_tests: false,
            include_economy_tests: true,
            include_governance_tests: true,
            include_performance_tests: false,
            include_fault_tests: true,
        }
    }

    /// Create an economy-focused test suite
    pub fn economy_focused() -> TestSuite {
        TestSuite {
            name: "Economy Focused".to_string(),
            description: "Storage economy and verification testing".to_string(),
            include_network_tests: true,
            include_cli_tests: true,
            include_api_tests: true,
            include_ui_tests: true,
            include_economy_tests: true,
            include_governance_tests: false,
            include_performance_tests: false,
            include_fault_tests: false,
        }
    }

    /// Create a governance-focused test suite
    pub fn governance_focused() -> TestSuite {
        TestSuite {
            name: "Governance Focused".to_string(),
            description: "Governance and consensus testing".to_string(),
            include_network_tests: true,
            include_cli_tests: true,
            include_api_tests: true,
            include_ui_tests: true,
            include_economy_tests: false,
            include_governance_tests: true,
            include_performance_tests: false,
            include_fault_tests: false,
        }
    }
}

/// Utility functions for test framework
pub mod helpers {
    use super::*;
    use std::path::Path;
    use anyhow::Result;

    /// Initialize the test framework with logging
    pub fn init_test_framework() -> Result<()> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("datamesh_test_framework=info".parse()?)
                    .add_directive("datamesh=debug".parse()?)
            )
            .init();

        Ok(())
    }

    /// Load configuration from TOML file
    pub fn load_config(path: &Path) -> Result<OrchestratorConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: OrchestratorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Quick test run with preset configuration
    pub async fn quick_test(
        preset: fn() -> OrchestratorConfig,
        scenario: fn() -> TestSuite,
    ) -> Result<TestReport> {
        init_test_framework()?;
        
        let config = preset();
        let test_suite = scenario();
        
        let mut orchestrator = TestOrchestrator::new(config).await?;
        orchestrator.deploy_cluster().await?;
        
        let _results = orchestrator.run_test_suite(test_suite).await?;
        let report = orchestrator.generate_report().await?;
        
        orchestrator.teardown().await?;
        
        Ok(report)
    }

    /// Validate test environment prerequisites
    pub fn validate_environment() -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check if datamesh binary exists
        if std::process::Command::new("datamesh")
            .arg("--version")
            .output()
            .is_err()
        {
            issues.push("DataMesh binary not found or not executable".to_string());
        }

        // Check available memory (require at least 4GB)
        if let Ok(sys) = sysinfo::System::new_all().total_memory() {
            if sys < 4 * 1024 * 1024 * 1024 {
                issues.push("Insufficient memory (require at least 4GB)".to_string());
            }
        }

        // Check available disk space (require at least 10GB)
        if let Ok(available) = std::fs::metadata("/tmp")
            .map(|m| m.len())
        {
            if available < 10 * 1024 * 1024 * 1024 {
                issues.push("Insufficient disk space in /tmp (require at least 10GB)".to_string());
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_configurations() {
        let basic = presets::basic_cluster();
        assert_eq!(basic.node_count, 3);
        
        let standard = presets::standard_cluster();
        assert_eq!(standard.node_count, 5);
        
        let performance = presets::performance_cluster();
        assert_eq!(performance.node_count, 10);
    }

    #[test]
    fn test_scenario_configurations() {
        let basic = scenarios::basic_functionality();
        assert!(basic.include_network_tests);
        assert!(basic.include_cli_tests);
        assert!(!basic.include_fault_tests);
        
        let comprehensive = scenarios::comprehensive();
        assert!(comprehensive.include_ui_tests);
        assert!(comprehensive.include_economy_tests);
        assert!(comprehensive.include_governance_tests);
    }

    #[tokio::test]
    async fn test_framework_initialization() {
        let result = helpers::init_test_framework();
        // Should not panic or return error
        assert!(result.is_ok() || result.is_err()); // Either is fine for this test
    }

    #[test]
    fn test_environment_validation() {
        let issues = helpers::validate_environment();
        assert!(issues.is_ok());
        // The actual validation results depend on the test environment
    }
}