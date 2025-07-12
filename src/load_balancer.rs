use anyhow::Result;
use serde::{Deserialize, Serialize};
/// Load Balancer and Auto-scaling Implementation
///
/// This module implements intelligent load balancing and auto-scaling capabilities
/// for the DataMesh network, ensuring optimal performance and resource utilization.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info};

use crate::network_diagnostics::NetworkDiagnostics;
use crate::performance::PerformanceMonitor;

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    ResourceBased,
    LatencyBased,
    AdaptiveIntelligent,
}

/// Node performance metrics for load balancing decisions
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub peer_id: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_latency: u64,
    pub active_connections: u32,
    pub storage_usage: f64,
    pub throughput: f64,
    pub success_rate: f64,
    pub last_updated: Instant,
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    pub enabled: bool,
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period: Duration,
    pub monitoring_interval: Duration,
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_nodes: 3,
            max_nodes: 20,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            cooldown_period: Duration::from_secs(300), // 5 minutes
            monitoring_interval: Duration::from_secs(30),
        }
    }
}

/// Load balancer state and management
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    node_metrics: Arc<RwLock<HashMap<String, NodeMetrics>>>,
    auto_scaling_config: AutoScalingConfig,
    last_scaling_action: Arc<RwLock<Option<Instant>>>,
    performance_monitor: Arc<PerformanceMonitor>,
    network_diagnostics: Arc<NetworkDiagnostics>,
    current_index: Arc<RwLock<usize>>, // For round-robin
}

impl LoadBalancer {
    /// Create a new load balancer instance
    pub fn new(
        strategy: LoadBalancingStrategy,
        auto_scaling_config: AutoScalingConfig,
        performance_monitor: Arc<PerformanceMonitor>,
        network_diagnostics: Arc<NetworkDiagnostics>,
    ) -> Self {
        Self {
            strategy,
            node_metrics: Arc::new(RwLock::new(HashMap::new())),
            auto_scaling_config,
            last_scaling_action: Arc::new(RwLock::new(None)),
            performance_monitor,
            network_diagnostics,
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Start the load balancer monitoring and auto-scaling
    pub async fn start(&self) -> Result<()> {
        info!("Starting load balancer with strategy: {:?}", self.strategy);

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start auto-scaling if enabled
        if self.auto_scaling_config.enabled {
            self.start_auto_scaling().await?;
        }

        Ok(())
    }

    /// Select the best node for a request based on current strategy
    pub async fn select_node(&self, request_type: &str) -> Result<Option<String>> {
        let metrics = self.node_metrics.read().await;

        if metrics.is_empty() {
            return Ok(None);
        }

        let selected_peer = match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin_selection(&metrics).await,
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin_selection(&metrics).await
            }
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections_selection(&metrics).await
            }
            LoadBalancingStrategy::ResourceBased => self.resource_based_selection(&metrics).await,
            LoadBalancingStrategy::LatencyBased => self.latency_based_selection(&metrics).await,
            LoadBalancingStrategy::AdaptiveIntelligent => {
                self.adaptive_intelligent_selection(&metrics, request_type)
                    .await
            }
        }?;

        Ok(selected_peer)
    }

    /// Round-robin node selection
    async fn round_robin_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> Result<Option<String>> {
        let nodes: Vec<String> = metrics.keys().cloned().collect();
        if nodes.is_empty() {
            return Ok(None);
        }

        let mut index = self.current_index.write().await;
        let selected = nodes[*index % nodes.len()].clone();
        *index += 1;

        Ok(Some(selected))
    }

    /// Weighted round-robin based on node performance
    async fn weighted_round_robin_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> Result<Option<String>> {
        let mut weighted_nodes = Vec::new();

        for (peer_id, metric) in metrics.iter() {
            // Calculate weight based on performance metrics
            let weight = self.calculate_node_weight(metric);
            for _ in 0..weight {
                weighted_nodes.push(peer_id.clone());
            }
        }

        if weighted_nodes.is_empty() {
            return Ok(None);
        }

        let mut index = self.current_index.write().await;
        let selected = weighted_nodes[*index % weighted_nodes.len()].clone();
        *index += 1;

        Ok(Some(selected))
    }

    /// Select node with least active connections
    async fn least_connections_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> Result<Option<String>> {
        let selected = metrics
            .iter()
            .min_by_key(|(_, metric)| metric.active_connections)
            .map(|(peer_id, _)| peer_id.clone());

        Ok(selected)
    }

    /// Select node based on available resources
    async fn resource_based_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> Result<Option<String>> {
        let selected = metrics
            .iter()
            .min_by(|(_, a), (_, b)| {
                let a_load = (a.cpu_usage + a.memory_usage + a.storage_usage) / 3.0;
                let b_load = (b.cpu_usage + b.memory_usage + b.storage_usage) / 3.0;
                a_load
                    .partial_cmp(&b_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(peer_id, _)| peer_id.clone());

        Ok(selected)
    }

    /// Select node with lowest latency
    async fn latency_based_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> Result<Option<String>> {
        let selected = metrics
            .iter()
            .min_by_key(|(_, metric)| metric.network_latency)
            .map(|(peer_id, _)| peer_id.clone());

        Ok(selected)
    }

    /// Adaptive intelligent selection based on request type and ML predictions
    async fn adaptive_intelligent_selection(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
        request_type: &str,
    ) -> Result<Option<String>> {
        let mut scores = HashMap::new();

        for (peer_id, metric) in metrics.iter() {
            let score = self.calculate_adaptive_score(metric, request_type).await;
            scores.insert(peer_id.clone(), score);
        }

        let selected = scores
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(peer_id, _)| peer_id.clone());

        Ok(selected)
    }

    /// Calculate adaptive score for intelligent selection
    async fn calculate_adaptive_score(&self, metric: &NodeMetrics, request_type: &str) -> f64 {
        let mut score = 0.0;

        // Base performance score
        score += (1.0 - metric.cpu_usage) * 0.3;
        score += (1.0 - metric.memory_usage) * 0.2;
        score += (1.0 - metric.storage_usage) * 0.1;
        score += metric.success_rate * 0.2;

        // Latency penalty
        score -= (metric.network_latency as f64 / 1000.0) * 0.1;

        // Connection load penalty
        score -= (metric.active_connections as f64 / 100.0) * 0.1;

        // Request type specific optimizations
        match request_type {
            "upload" => {
                score += (1.0 - metric.storage_usage) * 0.2;
                score += metric.throughput * 0.1;
            }
            "download" => {
                score += metric.throughput * 0.3;
                score -= (metric.network_latency as f64 / 1000.0) * 0.2;
            }
            "search" => {
                score += (1.0 - metric.cpu_usage) * 0.4;
                score += (1.0 - metric.memory_usage) * 0.3;
            }
            _ => {}
        }

        score.max(0.0).min(1.0)
    }

    /// Calculate node weight for weighted round-robin
    fn calculate_node_weight(&self, metric: &NodeMetrics) -> usize {
        let performance_score = (metric.success_rate
            * (1.0 - metric.cpu_usage)
            * (1.0 - metric.memory_usage)
            * (1.0 - metric.storage_usage)
            * metric.throughput)
            / (metric.network_latency as f64 + 1.0);

        ((performance_score * 10.0) as usize).max(1)
    }

    /// Start metrics collection from nodes
    async fn start_metrics_collection(&self) -> Result<()> {
        let node_metrics = self.node_metrics.clone();
        let network_diagnostics = self.network_diagnostics.clone();
        let performance_monitor = self.performance_monitor.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                if let Err(e) = Self::collect_node_metrics(
                    &node_metrics,
                    &network_diagnostics,
                    &performance_monitor,
                )
                .await
                {
                    error!("Failed to collect node metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Collect metrics from all nodes
    async fn collect_node_metrics(
        node_metrics: &Arc<RwLock<HashMap<String, NodeMetrics>>>,
        network_diagnostics: &Arc<NetworkDiagnostics>,
        _performance_monitor: &Arc<PerformanceMonitor>,
    ) -> Result<()> {
        let mut metrics = node_metrics.write().await;

        // This would typically query actual nodes for their metrics
        // For now, we'll simulate with network diagnostics data
        let peers = network_diagnostics.get_active_peers().await;

        for peer_id in peers {
            let peer_str = peer_id.to_string();

            // Simulate metrics collection (in real implementation, this would query the actual node)
            let node_metric = NodeMetrics {
                peer_id: peer_str.clone(),
                cpu_usage: fastrand::f64() * 0.8, // Simulate CPU usage
                memory_usage: fastrand::f64() * 0.7, // Simulate memory usage
                network_latency: network_diagnostics.get_avg_response_time(peer_id),
                active_connections: fastrand::u32(1..50),
                storage_usage: fastrand::f64() * 0.6,
                throughput: fastrand::f64() * 100.0,
                success_rate: 0.95 + fastrand::f64() * 0.05,
                last_updated: Instant::now(),
            };

            metrics.insert(peer_str, node_metric);
        }

        // Remove stale metrics
        let now = Instant::now();
        metrics
            .retain(|_, metric| now.duration_since(metric.last_updated) < Duration::from_secs(300));

        Ok(())
    }

    /// Start auto-scaling monitoring
    async fn start_auto_scaling(&self) -> Result<()> {
        let node_metrics = self.node_metrics.clone();
        let config = self.auto_scaling_config.clone();
        let last_scaling_action = self.last_scaling_action.clone();

        tokio::spawn(async move {
            let mut interval = interval(config.monitoring_interval);

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::evaluate_scaling_decision(&node_metrics, &config, &last_scaling_action)
                        .await
                {
                    error!("Auto-scaling evaluation failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Evaluate whether scaling action is needed
    async fn evaluate_scaling_decision(
        node_metrics: &Arc<RwLock<HashMap<String, NodeMetrics>>>,
        config: &AutoScalingConfig,
        last_scaling_action: &Arc<RwLock<Option<Instant>>>,
    ) -> Result<()> {
        let metrics = node_metrics.read().await;
        let node_count = metrics.len() as u32;

        if node_count == 0 {
            return Ok(());
        }

        // Check cooldown period
        let last_action = last_scaling_action.read().await;
        if let Some(last_time) = *last_action {
            if Instant::now().duration_since(last_time) < config.cooldown_period {
                return Ok(());
            }
        }
        drop(last_action);

        // Calculate average load
        let total_load: f64 = metrics
            .values()
            .map(|m| (m.cpu_usage + m.memory_usage + m.storage_usage) / 3.0)
            .sum();
        let avg_load = total_load / node_count as f64;

        // Scaling decision
        if avg_load > config.scale_up_threshold && node_count < config.max_nodes {
            info!(
                "Auto-scaling: Scaling up (avg_load: {:.2}, nodes: {})",
                avg_load, node_count
            );
            Self::trigger_scale_up().await?;
            *last_scaling_action.write().await = Some(Instant::now());
        } else if avg_load < config.scale_down_threshold && node_count > config.min_nodes {
            info!(
                "Auto-scaling: Scaling down (avg_load: {:.2}, nodes: {})",
                avg_load, node_count
            );
            Self::trigger_scale_down().await?;
            *last_scaling_action.write().await = Some(Instant::now());
        }

        Ok(())
    }

    /// Trigger scale-up action
    async fn trigger_scale_up() -> Result<()> {
        // This would trigger actual node creation in a real implementation
        // For now, we'll just log the action
        info!("Triggering scale-up action - would create new nodes");
        Ok(())
    }

    /// Trigger scale-down action
    async fn trigger_scale_down() -> Result<()> {
        // This would trigger actual node removal in a real implementation
        // For now, we'll just log the action
        info!("Triggering scale-down action - would remove excess nodes");
        Ok(())
    }

    /// Get current load balancer statistics
    pub async fn get_statistics(&self) -> Result<LoadBalancerStats> {
        let metrics = self.node_metrics.read().await;
        let node_count = metrics.len();

        let total_load: f64 = metrics
            .values()
            .map(|m| (m.cpu_usage + m.memory_usage + m.storage_usage) / 3.0)
            .sum();
        let avg_load = if node_count > 0 {
            total_load / node_count as f64
        } else {
            0.0
        };

        let total_connections: u32 = metrics.values().map(|m| m.active_connections).sum();
        let avg_latency = if node_count > 0 {
            metrics.values().map(|m| m.network_latency).sum::<u64>() / node_count as u64
        } else {
            0
        };

        Ok(LoadBalancerStats {
            strategy: self.strategy.clone(),
            node_count,
            average_load: avg_load,
            total_connections,
            average_latency: avg_latency,
            auto_scaling_enabled: self.auto_scaling_config.enabled,
        })
    }
}

/// Load balancer statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub strategy: LoadBalancingStrategy,
    pub node_count: usize,
    pub average_load: f64,
    pub total_connections: u32,
    pub average_latency: u64,
    pub auto_scaling_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin_selection() {
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let network_diagnostics = Arc::new(NetworkDiagnostics::new());
        let load_balancer = LoadBalancer::new(
            LoadBalancingStrategy::RoundRobin,
            AutoScalingConfig::default(),
            performance_monitor,
            network_diagnostics,
        );

        // Add some test metrics
        let mut metrics = load_balancer.node_metrics.write().await;
        metrics.insert(
            "node1".to_string(),
            NodeMetrics {
                peer_id: "node1".to_string(),
                cpu_usage: 0.5,
                memory_usage: 0.4,
                network_latency: 100,
                active_connections: 10,
                storage_usage: 0.3,
                throughput: 50.0,
                success_rate: 0.95,
                last_updated: Instant::now(),
            },
        );
        metrics.insert(
            "node2".to_string(),
            NodeMetrics {
                peer_id: "node2".to_string(),
                cpu_usage: 0.6,
                memory_usage: 0.5,
                network_latency: 120,
                active_connections: 15,
                storage_usage: 0.4,
                throughput: 45.0,
                success_rate: 0.93,
                last_updated: Instant::now(),
            },
        );
        drop(metrics);

        // Test round-robin selection
        let node1 = load_balancer.select_node("test").await.unwrap();
        let node2 = load_balancer.select_node("test").await.unwrap();
        let node3 = load_balancer.select_node("test").await.unwrap();

        assert!(node1.is_some());
        assert!(node2.is_some());
        assert!(node3.is_some());
        assert_eq!(node1, node3); // Should cycle back to first node
    }

    #[tokio::test]
    async fn test_least_connections_selection() {
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let network_diagnostics = Arc::new(NetworkDiagnostics::new());
        let load_balancer = LoadBalancer::new(
            LoadBalancingStrategy::LeastConnections,
            AutoScalingConfig::default(),
            performance_monitor,
            network_diagnostics,
        );

        // Add test metrics with different connection counts
        let mut metrics = load_balancer.node_metrics.write().await;
        metrics.insert(
            "node1".to_string(),
            NodeMetrics {
                peer_id: "node1".to_string(),
                cpu_usage: 0.5,
                memory_usage: 0.4,
                network_latency: 100,
                active_connections: 5, // Lower connections
                storage_usage: 0.3,
                throughput: 50.0,
                success_rate: 0.95,
                last_updated: Instant::now(),
            },
        );
        metrics.insert(
            "node2".to_string(),
            NodeMetrics {
                peer_id: "node2".to_string(),
                cpu_usage: 0.6,
                memory_usage: 0.5,
                network_latency: 120,
                active_connections: 20, // Higher connections
                storage_usage: 0.4,
                throughput: 45.0,
                success_rate: 0.93,
                last_updated: Instant::now(),
            },
        );
        drop(metrics);

        // Should select node with least connections
        let selected = load_balancer.select_node("test").await.unwrap();
        assert_eq!(selected, Some("node1".to_string()));
    }
}
