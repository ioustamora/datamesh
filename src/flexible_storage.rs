use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::error::Result;
use crate::database::DatabaseManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexibleStorageManager {
    pub storage_tiers: Vec<FlexibleStorageTier>,
    pub burst_policies: HashMap<String, BurstPolicy>,
    pub priority_queue: PriorityQueue,
    pub bandwidth_allocator: BandwidthAllocator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexibleStorageTier {
    pub id: String,
    pub name: String,
    pub base_storage: u64,
    pub burst_allowance: u64,
    pub contribution_ratio: f64,
    pub priority_level: u8,
    pub bandwidth_allocation: u64,
    pub price_per_gb: f64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstPolicy {
    pub max_burst_duration: Duration,
    pub burst_price_multiplier: f64,
    pub cooldown_period: Duration,
    pub auto_upgrade_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityQueue {
    pub queues: HashMap<u8, Vec<QueuedOperation>>,
    pub processing_weights: HashMap<u8, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedOperation {
    pub user_id: String,
    pub operation_type: OperationType,
    pub file_id: String,
    pub timestamp: DateTime<Utc>,
    pub priority_level: u8,
    pub estimated_completion: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Upload,
    Download,
    Retrieve,
    Backup,
    Restore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthAllocator {
    pub total_bandwidth: u64,
    pub tier_allocations: HashMap<String, BandwidthAllocation>,
    pub dynamic_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthAllocation {
    pub guaranteed_bandwidth: u64,
    pub burstable_bandwidth: u64,
    pub current_usage: u64,
    pub usage_history: Vec<BandwidthUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub timestamp: DateTime<Utc>,
    pub usage: u64,
    pub peak_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStorageState {
    pub user_id: String,
    pub current_tier: String,
    pub storage_used: u64,
    pub burst_used: u64,
    pub contribution_provided: u64,
    pub priority_level: u8,
    pub bandwidth_quota: u64,
    pub last_burst_usage: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub current_efficiency: f64,
    pub potential_savings: f64,
    pub recommended_action: String,
    pub confidence: f64,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    TierUpgrade,
    TierDowngrade,
    BurstOptimization,
    ContributionIncrease,
    BandwidthReallocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimization {
    pub recommended_tier: String,
    pub burst_capacity: u64,
    pub cost_savings: f64,
    pub performance_improvement: f64,
}

impl FlexibleStorageManager {
    pub fn new() -> Self {
        Self {
            storage_tiers: Self::create_default_tiers(),
            burst_policies: Self::create_default_burst_policies(),
            priority_queue: PriorityQueue::new(),
            bandwidth_allocator: BandwidthAllocator::new(),
        }
    }

    fn create_default_tiers() -> Vec<FlexibleStorageTier> {
        vec![
            FlexibleStorageTier {
                id: "free".to_string(),
                name: "Free".to_string(),
                base_storage: 5_000_000_000, // 5GB
                burst_allowance: 1_000_000_000, // 1GB
                contribution_ratio: 4.0,
                priority_level: 1,
                bandwidth_allocation: 10_000_000, // 10MB/s
                price_per_gb: 0.0,
                features: vec!["Basic encryption".to_string(), "Standard support".to_string()],
            },
            FlexibleStorageTier {
                id: "basic".to_string(),
                name: "Basic".to_string(),
                base_storage: 100_000_000_000, // 100GB
                burst_allowance: 20_000_000_000, // 20GB
                contribution_ratio: 3.0,
                priority_level: 2,
                bandwidth_allocation: 50_000_000, // 50MB/s
                price_per_gb: 0.10,
                features: vec!["Enhanced encryption".to_string(), "Priority support".to_string()],
            },
            FlexibleStorageTier {
                id: "pro".to_string(),
                name: "Pro".to_string(),
                base_storage: 1_000_000_000_000, // 1TB
                burst_allowance: 200_000_000_000, // 200GB
                contribution_ratio: 2.5,
                priority_level: 3,
                bandwidth_allocation: 100_000_000, // 100MB/s
                price_per_gb: 0.08,
                features: vec!["Advanced encryption".to_string(), "Premium support".to_string(), "Analytics dashboard".to_string()],
            },
            FlexibleStorageTier {
                id: "enterprise".to_string(),
                name: "Enterprise".to_string(),
                base_storage: 10_000_000_000_000, // 10TB
                burst_allowance: 2_000_000_000_000, // 2TB
                contribution_ratio: 2.0,
                priority_level: 4,
                bandwidth_allocation: 500_000_000, // 500MB/s
                price_per_gb: 0.05,
                features: vec!["Enterprise encryption".to_string(), "24/7 support".to_string(), "Custom analytics".to_string(), "API access".to_string()],
            },
        ]
    }

    fn create_default_burst_policies() -> HashMap<String, BurstPolicy> {
        let mut policies = HashMap::new();
        
        policies.insert("free".to_string(), BurstPolicy {
            max_burst_duration: Duration::hours(1),
            burst_price_multiplier: 2.0,
            cooldown_period: Duration::hours(24),
            auto_upgrade_threshold: 3_000_000_000, // 3GB
        });

        policies.insert("basic".to_string(), BurstPolicy {
            max_burst_duration: Duration::hours(6),
            burst_price_multiplier: 1.5,
            cooldown_period: Duration::hours(12),
            auto_upgrade_threshold: 50_000_000_000, // 50GB
        });

        policies.insert("pro".to_string(), BurstPolicy {
            max_burst_duration: Duration::hours(24),
            burst_price_multiplier: 1.2,
            cooldown_period: Duration::hours(6),
            auto_upgrade_threshold: 500_000_000_000, // 500GB
        });

        policies.insert("enterprise".to_string(), BurstPolicy {
            max_burst_duration: Duration::days(7),
            burst_price_multiplier: 1.0,
            cooldown_period: Duration::hours(1),
            auto_upgrade_threshold: 5_000_000_000_000, // 5TB
        });

        policies
    }

    /// Calculate optimal storage tier for a user based on usage patterns
    pub async fn calculate_optimal_tier(&self, user_id: &str, usage_history: &[StorageUsage]) -> Result<TierRecommendation> {
        let current_state = self.get_user_storage_state(user_id).await?;
        let usage_analysis = self.analyze_usage_patterns(usage_history)?;

        let current_tier = self.get_tier_by_id(&current_state.current_tier)?;
        let efficiency = self.calculate_tier_efficiency(&current_state, &usage_analysis);

        let recommendation = if efficiency < 0.6 {
            // User is overpaying
            let optimal_tier = self.find_optimal_tier_for_usage(&usage_analysis)?;
            TierRecommendation::Downgrade {
                current_tier: current_tier.clone(),
                suggested_tier: optimal_tier.clone(),
                potential_savings: self.calculate_savings(&current_tier, &optimal_tier, &usage_analysis),
                confidence: self.calculate_confidence(&usage_analysis),
            }
        } else if usage_analysis.burst_frequency > 0.3 {
            // User frequently exceeds limits
            let optimal_tier = self.find_tier_for_burst_usage(&usage_analysis)?;
            TierRecommendation::Upgrade {
                current_tier: current_tier.clone(),
                suggested_tier: optimal_tier.clone(),
                benefits: self.calculate_upgrade_benefits(&current_tier, &optimal_tier),
                urgency: self.calculate_urgency(&usage_analysis),
                potential_savings: 150.0, // Stub value
            }
        } else {
            TierRecommendation::Optimal {
                current_tier: current_tier.clone(),
                efficiency_score: efficiency,
                optimization_tips: self.generate_optimization_tips(&usage_analysis),
            }
        };

        Ok(recommendation)
    }

    /// Handle burst storage requests
    pub async fn handle_burst_request(&mut self, user_id: &str, additional_storage: u64) -> Result<BurstResponse> {
        let user_state = self.get_user_storage_state(user_id).await?;
        let tier = self.get_tier_by_id(&user_state.current_tier)?;
        let policy = self.burst_policies.get(&tier.id).unwrap();

        // Check if user can use burst storage
        if let Some(last_burst) = user_state.last_burst_usage {
            if Utc::now() - last_burst < policy.cooldown_period {
                return Ok(BurstResponse::Denied {
                    reason: "Cooldown period active".to_string(),
                    available_at: last_burst + policy.cooldown_period,
                });
            }
        }

        if user_state.burst_used + additional_storage > tier.burst_allowance {
            return Ok(BurstResponse::Denied {
                reason: "Exceeds burst allowance".to_string(),
                available_at: Utc::now() + Duration::hours(1),
            });
        }

        // Calculate cost
        let cost = additional_storage as f64 * tier.price_per_gb * policy.burst_price_multiplier;

        // Check for auto-upgrade suggestion
        let should_suggest_upgrade = additional_storage > policy.auto_upgrade_threshold;

        Ok(BurstResponse::Approved {
            granted_storage: additional_storage,
            cost,
            duration: policy.max_burst_duration,
            suggest_upgrade: should_suggest_upgrade,
        })
    }

    /// Allocate bandwidth based on tier and current usage
    pub async fn allocate_bandwidth(&mut self, user_id: &str, operation: &QueuedOperation) -> Result<BandwidthAllocation> {
        let user_state = self.get_user_storage_state(user_id).await?;
        let tier = self.get_tier_by_id(&user_state.current_tier)?;

        // Get current bandwidth allocation
        let allocation = self.bandwidth_allocator.tier_allocations
            .get(&tier.id)
            .cloned()
            .unwrap_or_else(|| BandwidthAllocation::default());

        // Calculate available bandwidth
        let available_bandwidth = allocation.guaranteed_bandwidth.saturating_sub(allocation.current_usage);

        // Check if we can provide burstable bandwidth
        let burstable_available = if available_bandwidth < allocation.guaranteed_bandwidth / 2 {
            allocation.burstable_bandwidth.saturating_sub(allocation.current_usage)
        } else {
            0
        };

        Ok(BandwidthAllocation {
            guaranteed_bandwidth: available_bandwidth,
            burstable_bandwidth: burstable_available,
            current_usage: allocation.current_usage,
            usage_history: allocation.usage_history,
        })
    }

    /// Add operation to priority queue
    pub async fn queue_operation(&mut self, user_id: &str, operation: QueuedOperation) -> Result<QueuePosition> {
        let user_state = self.get_user_storage_state(user_id).await?;
        let tier = self.get_tier_by_id(&user_state.current_tier)?;

        let priority_level = tier.priority_level;
        let queue = self.priority_queue.queues.entry(priority_level).or_insert_with(Vec::new);

        // Insert operation maintaining timestamp order
        let position = queue.len();
        queue.push(operation);

        // Sort by timestamp within priority level
        queue.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(QueuePosition {
            priority_level,
            position_in_queue: position,
            estimated_wait_time: self.estimate_wait_time(priority_level, position),
        })
    }

    /// Generate storage optimization suggestions
    pub async fn generate_optimization_suggestions(&self, user_id: &str) -> Result<Vec<StorageOptimizationSuggestion>> {
        let user_state = self.get_user_storage_state(user_id).await?;
        let usage_history = self.get_usage_history(user_id).await?;
        let usage_analysis = self.analyze_usage_patterns(&usage_history)?;

        let mut suggestions = Vec::new();

        // Tier optimization
        if let Ok(tier_rec) = self.calculate_optimal_tier(user_id, &usage_history).await {
            match tier_rec {
                TierRecommendation::Upgrade { potential_savings, .. } => {
                    suggestions.push(StorageOptimizationSuggestion {
                        suggestion_type: SuggestionType::TierUpgrade,
                        current_efficiency: usage_analysis.efficiency,
                        potential_savings,
                        recommended_action: "Upgrade to next tier for better value".to_string(),
                        confidence: 0.8,
                        implementation_steps: vec![
                            "Review current usage patterns".to_string(),
                            "Calculate cost-benefit analysis".to_string(),
                            "Upgrade tier in account settings".to_string(),
                        ],
                    });
                }
                TierRecommendation::Downgrade { potential_savings, .. } => {
                    suggestions.push(StorageOptimizationSuggestion {
                        suggestion_type: SuggestionType::TierDowngrade,
                        current_efficiency: usage_analysis.efficiency,
                        potential_savings,
                        recommended_action: "Downgrade to save costs".to_string(),
                        confidence: 0.9,
                        implementation_steps: vec![
                            "Archive unused files".to_string(),
                            "Optimize storage usage".to_string(),
                            "Downgrade tier in account settings".to_string(),
                        ],
                    });
                }
                _ => {}
            }
        }

        // Contribution optimization
        if user_state.contribution_provided < user_state.storage_used * 2.0 as u64 {
            suggestions.push(StorageOptimizationSuggestion {
                suggestion_type: SuggestionType::ContributionIncrease,
                current_efficiency: usage_analysis.efficiency,
                potential_savings: usage_analysis.potential_contribution_savings,
                recommended_action: "Increase storage contribution for better rates".to_string(),
                confidence: 0.7,
                implementation_steps: vec![
                    "Allocate more local storage".to_string(),
                    "Configure contribution settings".to_string(),
                    "Monitor contribution rewards".to_string(),
                ],
            });
        }

        Ok(suggestions)
    }

    // Helper methods
    async fn get_user_storage_state(&self, user_id: &str) -> Result<UserStorageState> {
        // In a real implementation, this would query the database
        Ok(UserStorageState {
            user_id: user_id.to_string(),
            current_tier: "basic".to_string(),
            storage_used: 50_000_000_000, // 50GB
            burst_used: 0,
            contribution_provided: 150_000_000_000, // 150GB
            priority_level: 2,
            bandwidth_quota: 50_000_000, // 50MB/s
            last_burst_usage: None,
        })
    }

    fn get_tier_by_id(&self, tier_id: &str) -> Result<&FlexibleStorageTier> {
        self.storage_tiers.iter()
            .find(|t| t.id == tier_id)
            .ok_or_else(|| format!("Tier not found: {}", tier_id).into())
    }

    fn analyze_usage_patterns(&self, history: &[StorageUsage]) -> Result<UsageAnalysis> {
        let total_usage: u64 = history.iter().map(|u| u.storage_used).sum();
        let avg_usage = total_usage as f64 / history.len() as f64;
        
        let burst_events = history.iter().filter(|u| u.burst_used > 0).count();
        let burst_frequency = burst_events as f64 / history.len() as f64;

        Ok(UsageAnalysis {
            average_usage: avg_usage,
            peak_usage: history.iter().map(|u| u.storage_used).max().unwrap_or(0),
            burst_frequency,
            efficiency: self.calculate_efficiency(history),
            growth_trend: self.calculate_growth_trend(history),
            potential_contribution_savings: avg_usage * 0.2, // 20% savings potential
        })
    }

    fn calculate_efficiency(&self, _history: &[StorageUsage]) -> f64 {
        // Simplified efficiency calculation
        0.75
    }

    fn calculate_growth_trend(&self, _history: &[StorageUsage]) -> f64 {
        // Simplified growth trend calculation
        0.05 // 5% growth
    }

    fn calculate_tier_efficiency(&self, _state: &UserStorageState, analysis: &UsageAnalysis) -> f64 {
        // Simplified efficiency calculation
        analysis.efficiency
    }

    fn find_optimal_tier_for_usage(&self, analysis: &UsageAnalysis) -> Result<FlexibleStorageTier> {
        for tier in &self.storage_tiers {
            if tier.base_storage as f64 >= analysis.average_usage * 1.2 {
                return Ok(tier.clone());
            }
        }
        Ok(self.storage_tiers.last().unwrap().clone())
    }

    fn find_tier_for_burst_usage(&self, analysis: &UsageAnalysis) -> Result<FlexibleStorageTier> {
        for tier in &self.storage_tiers {
            if tier.base_storage as f64 >= analysis.peak_usage as f64 {
                return Ok(tier.clone());
            }
        }
        Ok(self.storage_tiers.last().unwrap().clone())
    }

    fn calculate_savings(&self, _current: &FlexibleStorageTier, _optimal: &FlexibleStorageTier, _analysis: &UsageAnalysis) -> f64 {
        // Simplified savings calculation
        25.0
    }

    fn calculate_confidence(&self, _analysis: &UsageAnalysis) -> f64 {
        0.85
    }

    fn calculate_upgrade_benefits(&self, _current: &FlexibleStorageTier, _optimal: &FlexibleStorageTier) -> Vec<String> {
        vec![
            "Increased storage capacity".to_string(),
            "Higher priority processing".to_string(),
            "Better bandwidth allocation".to_string(),
        ]
    }

    fn calculate_urgency(&self, analysis: &UsageAnalysis) -> UrgencyLevel {
        if analysis.burst_frequency > 0.5 {
            UrgencyLevel::High
        } else if analysis.burst_frequency > 0.2 {
            UrgencyLevel::Medium
        } else {
            UrgencyLevel::Low
        }
    }

    fn generate_optimization_tips(&self, _analysis: &UsageAnalysis) -> Vec<String> {
        vec![
            "Consider increasing storage contribution for better rates".to_string(),
            "Monitor usage patterns to optimize costs".to_string(),
            "Use burst storage strategically for temporary needs".to_string(),
        ]
    }

    fn estimate_wait_time(&self, priority_level: u8, position: usize) -> Duration {
        let base_time = Duration::minutes(5);
        let priority_multiplier = match priority_level {
            4 => 0.5,
            3 => 0.7,
            2 => 1.0,
            1 => 1.5,
            _ => 2.0,
        };
        
        base_time * (position as i32 + 1) * priority_multiplier as i32
    }

    async fn get_usage_history(&self, _user_id: &str) -> Result<Vec<StorageUsage>> {
        // In a real implementation, this would query the database
        Ok(vec![
            StorageUsage {
                timestamp: Utc::now() - Duration::days(30),
                storage_used: 40_000_000_000,
                burst_used: 0,
                bandwidth_used: 1_000_000_000,
            },
            StorageUsage {
                timestamp: Utc::now() - Duration::days(15),
                storage_used: 45_000_000_000,
                burst_used: 5_000_000_000,
                bandwidth_used: 1_500_000_000,
            },
            StorageUsage {
                timestamp: Utc::now() - Duration::days(7),
                storage_used: 50_000_000_000,
                burst_used: 0,
                bandwidth_used: 2_000_000_000,
            },
        ])
    }

    /// Optimize user storage based on usage patterns
    pub async fn optimize_user_storage(&self, user_id: &str, current_usage: u64) -> Result<StorageOptimization> {
        // Stub implementation for compilation
        let recommended_tier = if current_usage > 1_000_000_000_000 {
            "enterprise".to_string()
        } else if current_usage > 100_000_000_000 {
            "pro".to_string()
        } else if current_usage > 5_000_000_000 {
            "basic".to_string()
        } else {
            "free".to_string()
        };

        Ok(StorageOptimization {
            recommended_tier,
            burst_capacity: current_usage / 10,
            cost_savings: 50.0,
            performance_improvement: 25.0,
        })
    }

    /// Recommend storage tier based on access patterns
    pub async fn recommend_tier(&self, user_id: &str, access_patterns: &[String]) -> Result<TierRecommendation> {
        // Stub implementation for compilation
        let potential_savings = 100.0;
        Ok(TierRecommendation::Upgrade { 
            current_tier: self.storage_tiers[0].clone(),
            suggested_tier: self.storage_tiers[1].clone(),
            benefits: vec!["Better performance".to_string()],
            urgency: UrgencyLevel::Medium,
            potential_savings,
        })
    }
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            processing_weights: {
                let mut weights = HashMap::new();
                weights.insert(4, 2.0); // Enterprise - highest priority
                weights.insert(3, 1.5); // Pro
                weights.insert(2, 1.0); // Basic
                weights.insert(1, 0.5); // Free - lowest priority
                weights
            },
        }
    }
}

impl BandwidthAllocator {
    pub fn new() -> Self {
        Self {
            total_bandwidth: 10_000_000_000, // 10GB/s total
            tier_allocations: HashMap::new(),
            dynamic_scaling: true,
        }
    }
}

impl Default for BandwidthAllocation {
    fn default() -> Self {
        Self {
            guaranteed_bandwidth: 10_000_000, // 10MB/s
            burstable_bandwidth: 50_000_000,  // 50MB/s
            current_usage: 0,
            usage_history: Vec::new(),
        }
    }
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub timestamp: DateTime<Utc>,
    pub storage_used: u64,
    pub burst_used: u64,
    pub bandwidth_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalysis {
    pub average_usage: f64,
    pub peak_usage: u64,
    pub burst_frequency: f64,
    pub efficiency: f64,
    pub growth_trend: f64,
    pub potential_contribution_savings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierRecommendation {
    Upgrade {
        current_tier: FlexibleStorageTier,
        suggested_tier: FlexibleStorageTier,
        benefits: Vec<String>,
        urgency: UrgencyLevel,
        potential_savings: f64,
    },
    Downgrade {
        current_tier: FlexibleStorageTier,
        suggested_tier: FlexibleStorageTier,
        potential_savings: f64,
        confidence: f64,
    },
    Optimal {
        current_tier: FlexibleStorageTier,
        efficiency_score: f64,
        optimization_tips: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BurstResponse {
    Approved {
        granted_storage: u64,
        cost: f64,
        duration: Duration,
        suggest_upgrade: bool,
    },
    Denied {
        reason: String,
        available_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePosition {
    pub priority_level: u8,
    pub position_in_queue: usize,
    pub estimated_wait_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flexible_storage_tier_calculation() {
        let manager = FlexibleStorageManager::new();
        let usage_history = vec![
            StorageUsage {
                timestamp: Utc::now() - Duration::days(30),
                storage_used: 40_000_000_000,
                burst_used: 0,
                bandwidth_used: 1_000_000_000,
            },
        ];

        let recommendation = manager.calculate_optimal_tier("test_user", &usage_history).await.unwrap();
        
        match recommendation {
            TierRecommendation::Optimal { efficiency_score, .. } => {
                assert!(efficiency_score > 0.0);
            }
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_burst_storage_request() {
        let mut manager = FlexibleStorageManager::new();
        let response = manager.handle_burst_request("test_user", 5_000_000_000).await.unwrap();
        
        match response {
            BurstResponse::Approved { granted_storage, .. } => {
                assert_eq!(granted_storage, 5_000_000_000);
            }
            _ => panic!("Expected approved burst request"),
        }
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let mut manager = FlexibleStorageManager::new();
        let operation = QueuedOperation {
            user_id: "test_user".to_string(),
            operation_type: OperationType::Upload,
            file_id: "test_file".to_string(),
            timestamp: Utc::now(),
            priority_level: 2,
            estimated_completion: Duration::minutes(5),
        };

        let position = manager.queue_operation("test_user", operation).await.unwrap();
        assert_eq!(position.priority_level, 2);
    }
}
