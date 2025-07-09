/// DataMesh Core Integration Service
///
/// This module integrates all the advanced features of DataMesh into a cohesive
/// system, including load balancing, failover, performance optimization, billing,
/// and governance.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::load_balancer::{LoadBalancer, LoadBalancingStrategy, AutoScalingConfig};
use crate::failover::{FailoverManager, FailoverConfig};
use crate::performance_optimizer::{PerformanceOptimizer, OptimizationConfig};
use crate::billing_system::{BillingSystem, BillingConfig};
use crate::governance::GovernanceFramework;
use crate::economics::EconomicModel;
use crate::performance::PerformanceMonitor;
use crate::network_diagnostics::NetworkDiagnostics;
use crate::bootstrap_manager::BootstrapManager;
use crate::database::DatabaseManager;

/// Core DataMesh system configuration
#[derive(Debug, Clone)]
pub struct DataMeshConfig {
    pub load_balancing: LoadBalancingStrategy,
    pub auto_scaling: AutoScalingConfig,
    pub failover: FailoverConfig,
    pub performance_optimization: OptimizationConfig,
    pub billing: BillingConfig,
    pub governance_enabled: bool,
}

impl Default for DataMeshConfig {
    fn default() -> Self {
        Self {
            load_balancing: LoadBalancingStrategy::AdaptiveIntelligent,
            auto_scaling: AutoScalingConfig::default(),
            failover: FailoverConfig::default(),
            performance_optimization: OptimizationConfig::default(),
            billing: BillingConfig::default(),
            governance_enabled: true,
        }
    }
}

/// Core DataMesh system that integrates all advanced features
pub struct DataMeshCore {
    config: DataMeshConfig,
    database: Arc<DatabaseManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    network_diagnostics: Arc<NetworkDiagnostics>,
    bootstrap_manager: Arc<BootstrapManager>,
    economic_model: Arc<EconomicModel>,
    
    // Core systems
    load_balancer: Option<Arc<LoadBalancer>>,
    failover_manager: Option<Arc<FailoverManager>>,
    performance_optimizer: Option<Arc<PerformanceOptimizer>>,
    billing_system: Option<Arc<BillingSystem>>,
    governance_framework: Option<Arc<GovernanceFramework>>,
    
    // System state
    is_running: Arc<RwLock<bool>>,
}

impl DataMeshCore {
    /// Create a new DataMesh core system
    pub fn new(
        config: DataMeshConfig,
        database: Arc<DatabaseManager>,
        performance_monitor: Arc<PerformanceMonitor>,
        network_diagnostics: Arc<NetworkDiagnostics>,
        bootstrap_manager: Arc<BootstrapManager>,
    ) -> Self {
        let economic_model = Arc::new(EconomicModel::new());
        
        Self {
            config,
            database,
            performance_monitor,
            network_diagnostics,
            bootstrap_manager,
            economic_model,
            load_balancer: None,
            failover_manager: None,
            performance_optimizer: None,
            billing_system: None,
            governance_framework: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize all core systems
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing DataMesh core systems");

        // Initialize load balancer
        let load_balancer = Arc::new(LoadBalancer::new(
            self.config.load_balancing.clone(),
            self.config.auto_scaling.clone(),
            self.performance_monitor.clone(),
            self.network_diagnostics.clone(),
        ));
        self.load_balancer = Some(load_balancer.clone());

        // Initialize failover manager
        let failover_manager = Arc::new(FailoverManager::new(
            self.config.failover.clone(),
            self.bootstrap_manager.clone(),
            self.network_diagnostics.clone(),
        ));
        self.failover_manager = Some(failover_manager.clone());

        // Initialize performance optimizer
        let performance_optimizer = Arc::new(PerformanceOptimizer::new(
            self.config.performance_optimization.clone(),
            self.performance_monitor.clone(),
            self.network_diagnostics.clone(),
            load_balancer.clone(),
        ));
        self.performance_optimizer = Some(performance_optimizer);

        // Initialize billing system
        let billing_system = Arc::new(BillingSystem::new(
            self.config.billing.clone(),
            self.database.clone(),
            self.economic_model.clone(),
        ));
        self.billing_system = Some(billing_system);

        // Initialize governance framework if enabled
        if self.config.governance_enabled {
            let governance_framework = Arc::new(GovernanceFramework::new(
                self.database.clone(),
                self.bootstrap_manager.clone(),
            ));
            self.governance_framework = Some(governance_framework);
        }

        info!("DataMesh core systems initialized successfully");
        Ok(())
    }

    /// Start all core systems
    pub async fn start(&self) -> Result<()> {
        info!("Starting DataMesh core systems");

        // Mark system as running
        *self.is_running.write().await = true;

        // Start load balancer
        if let Some(load_balancer) = &self.load_balancer {
            load_balancer.start().await?;
        }

        // Start failover manager
        if let Some(failover_manager) = &self.failover_manager {
            failover_manager.start().await?;
        }

        // Start performance optimizer
        if let Some(performance_optimizer) = &self.performance_optimizer {
            performance_optimizer.start().await?;
        }

        // Start billing system
        if let Some(billing_system) = &self.billing_system {
            billing_system.start().await?;
        }

        // Start governance framework
        if let Some(governance_framework) = &self.governance_framework {
            governance_framework.start().await?;
        }

        info!("DataMesh core systems started successfully");
        Ok(())
    }

    /// Stop all core systems
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping DataMesh core systems");

        // Mark system as stopped
        *self.is_running.write().await = false;

        // In a full implementation, we would gracefully shut down all systems
        // For now, we'll just log the shutdown
        info!("DataMesh core systems stopped");
        Ok(())
    }

    /// Get system status
    pub async fn get_status(&self) -> Result<DataMeshStatus> {
        let is_running = *self.is_running.read().await;
        
        let load_balancer_stats = if let Some(load_balancer) = &self.load_balancer {
            Some(load_balancer.get_statistics().await?)
        } else {
            None
        };

        let failover_stats = if let Some(failover_manager) = &self.failover_manager {
            Some(failover_manager.get_failover_stats().await?)
        } else {
            None
        };

        let performance_stats = if let Some(performance_optimizer) = &self.performance_optimizer {
            Some(performance_optimizer.get_performance_stats().await?)
        } else {
            None
        };

        let billing_stats = if let Some(billing_system) = &self.billing_system {
            Some(billing_system.get_billing_stats().await?)
        } else {
            None
        };

        Ok(DataMeshStatus {
            is_running,
            load_balancer_stats,
            failover_stats,
            performance_stats,
            billing_stats,
        })
    }

    /// Get performance recommendations
    pub async fn get_performance_recommendations(&self) -> Result<Vec<crate::performance_optimizer::OptimizationRecommendation>> {
        if let Some(performance_optimizer) = &self.performance_optimizer {
            performance_optimizer.get_recommendations().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Apply performance optimization
    pub async fn apply_optimization(&self, category: &str) -> Result<()> {
        if let Some(performance_optimizer) = &self.performance_optimizer {
            performance_optimizer.apply_optimization(category).await?;
        }
        Ok(())
    }

    /// Select optimal node for request
    pub async fn select_node(&self, request_type: &str) -> Result<Option<String>> {
        if let Some(load_balancer) = &self.load_balancer {
            load_balancer.select_node(request_type).await
        } else {
            Ok(None)
        }
    }

    /// Record request success for learning
    pub async fn record_request_success(&self, peer_id: &str) -> Result<()> {
        if let Some(failover_manager) = &self.failover_manager {
            failover_manager.record_success(peer_id).await?;
        }
        Ok(())
    }

    /// Record request failure for learning
    pub async fn record_request_failure(&self, peer_id: &str, error: &str) -> Result<()> {
        if let Some(failover_manager) = &self.failover_manager {
            failover_manager.record_failure(peer_id, error).await?;
        }
        Ok(())
    }

    /// Record usage for billing
    pub async fn record_usage(
        &self,
        user_id: crate::governance::UserId,
        resource_type: crate::billing_system::ResourceType,
        amount: f64,
        unit: String,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        if let Some(billing_system) = &self.billing_system {
            billing_system.record_usage(user_id, resource_type, amount, unit, metadata).await?;
        }
        Ok(())
    }

    /// Get billing statistics
    pub async fn get_billing_statistics(&self) -> Result<Option<crate::billing_system::BillingStats>> {
        if let Some(billing_system) = &self.billing_system {
            Ok(Some(billing_system.get_billing_stats().await?))
        } else {
            Ok(None)
        }
    }

    /// Create subscription
    pub async fn create_subscription(
        &self,
        user_id: crate::governance::UserId,
        tier: crate::billing_system::SubscriptionTier,
        billing_cycle: crate::billing_system::BillingCycle,
        payment_method: crate::billing_system::PaymentMethod,
    ) -> Result<Option<crate::billing_system::Subscription>> {
        if let Some(billing_system) = &self.billing_system {
            Ok(Some(billing_system.create_subscription(user_id, tier, billing_cycle, payment_method).await?))
        } else {
            Ok(None)
        }
    }

    /// Get healthy nodes
    pub async fn get_healthy_nodes(&self) -> Result<Vec<String>> {
        if let Some(failover_manager) = &self.failover_manager {
            failover_manager.get_healthy_nodes().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if node is available
    pub async fn is_node_available(&self, peer_id: &str) -> Result<bool> {
        if let Some(failover_manager) = &self.failover_manager {
            failover_manager.is_node_available(peer_id).await
        } else {
            Ok(true) // Default to available if no failover manager
        }
    }
}

/// DataMesh system status
#[derive(Debug)]
pub struct DataMeshStatus {
    pub is_running: bool,
    pub load_balancer_stats: Option<crate::load_balancer::LoadBalancerStats>,
    pub failover_stats: Option<crate::failover::FailoverStats>,
    pub performance_stats: Option<crate::performance_optimizer::PerformanceStats>,
    pub billing_stats: Option<crate::billing_system::BillingStats>,
}

/// Create default DataMesh core system
pub async fn create_default_core(
    database: Arc<DatabaseManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    network_diagnostics: Arc<NetworkDiagnostics>,
    bootstrap_manager: Arc<BootstrapManager>,
) -> Result<DataMeshCore> {
    let config = DataMeshConfig::default();
    let mut core = DataMeshCore::new(
        config,
        database,
        performance_monitor,
        network_diagnostics,
        bootstrap_manager,
    );

    core.initialize().await?;
    Ok(core)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::get_default_db_path;

    #[tokio::test]
    async fn test_datamesh_core_creation() {
        let db_path = get_default_db_path().unwrap();
        let database = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let network_diagnostics = Arc::new(NetworkDiagnostics::new());
        let bootstrap_manager = Arc::new(BootstrapManager::new());

        let core = create_default_core(
            database,
            performance_monitor,
            network_diagnostics,
            bootstrap_manager,
        ).await;

        assert!(core.is_ok());
    }
}