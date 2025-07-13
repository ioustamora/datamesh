use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::error::Result;
use crate::economics::EconomicService;
use crate::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPricingEngine {
    market_analyzer: MarketAnalyzer,
    demand_predictor: DemandPredictor,
    pricing_optimizer: PricingOptimizer,
    regional_adjustments: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalyzer {
    supply_metrics: SupplyMetrics,
    demand_metrics: DemandMetrics,
    historical_data: Vec<MarketSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandPredictor {
    prediction_model: PredictionModel,
    confidence_threshold: f64,
    update_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingOptimizer {
    base_prices: HashMap<String, f64>,
    multiplier_ranges: MultiplierRanges,
    optimization_targets: OptimizationTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRecommendation {
    pub resource_type: String,
    pub base_price: f64,
    pub multiplier: f64,
    pub final_price: f64,
    pub confidence: f64,
    pub valid_until: DateTime<Utc>,
    pub factors: Vec<PricingFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingFactor {
    pub name: String,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyMetrics {
    pub total_storage_capacity: u64,
    pub available_storage: u64,
    pub network_bandwidth: u64,
    pub active_nodes: u32,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandMetrics {
    pub storage_requests: u64,
    pub bandwidth_usage: u64,
    pub peak_times: Vec<DateTime<Utc>>,
    pub growth_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub timestamp: DateTime<Utc>,
    pub supply: SupplyMetrics,
    pub demand: DemandMetrics,
    pub price_point: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionModel {
    pub algorithm: String,
    pub accuracy: f64,
    pub last_trained: DateTime<Utc>,
    pub training_data_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplierRanges {
    pub min_multiplier: f64,
    pub max_multiplier: f64,
    pub optimal_range: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTargets {
    pub target_utilization: f64,
    pub revenue_goal: f64,
    pub user_satisfaction_threshold: f64,
}

impl DynamicPricingEngine {
    pub fn new() -> Self {
        Self {
            market_analyzer: MarketAnalyzer::new(),
            demand_predictor: DemandPredictor::new(),
            pricing_optimizer: PricingOptimizer::new(),
            regional_adjustments: HashMap::new(),
        }
    }

    /// Calculate optimal pricing based on current network conditions
    pub async fn calculate_optimal_pricing(&self, 
        resource_type: &str,
        region: Option<&str>
    ) -> Result<PricingRecommendation> {
        // Analyze current market conditions
        let supply = self.market_analyzer.get_current_supply().await?;
        let demand = self.market_analyzer.get_current_demand().await?;
        
        // Predict future demand
        let demand_prediction = self.demand_predictor.predict_demand(resource_type, Duration::hours(24)).await?;
        
        // Calculate base pricing factors
        let supply_demand_ratio = supply.available_storage as f64 / demand.storage_requests as f64;
        let base_multiplier = self.calculate_supply_demand_multiplier(supply_demand_ratio);
        
        // Apply regional adjustments
        let regional_multiplier = region
            .and_then(|r| self.regional_adjustments.get(r))
            .unwrap_or(&1.0);
        
        // Calculate final pricing
        let base_price = self.pricing_optimizer.get_base_price(resource_type)?;
        let final_multiplier = base_multiplier * regional_multiplier;
        let final_price = base_price * final_multiplier;
        
        // Calculate confidence based on data quality
        let confidence = self.calculate_confidence(&supply, &demand, &demand_prediction);
        
        // Compile pricing factors
        let factors = vec![
            PricingFactor {
                name: "Supply/Demand Ratio".to_string(),
                impact: base_multiplier - 1.0,
                description: format!("Current ratio: {:.2}", supply_demand_ratio),
            },
            PricingFactor {
                name: "Regional Adjustment".to_string(),
                impact: regional_multiplier - 1.0,
                description: format!("Region: {}", region.unwrap_or("global")),
            },
            PricingFactor {
                name: "Network Reliability".to_string(),
                impact: (supply.reliability_score - 0.95) * 0.1,
                description: format!("Reliability: {:.1}%", supply.reliability_score * 100.0),
            },
        ];
        
        Ok(PricingRecommendation {
            resource_type: resource_type.to_string(),
            base_price,
            multiplier: final_multiplier,
            final_price,
            confidence,
            valid_until: Utc::now() + Duration::hours(1),
            factors,
        })
    }

    /// Update pricing based on real-time network changes
    pub async fn update_dynamic_pricing(&mut self, network_stats: &NetworkStats) -> Result<()> {
        // Update market analyzer with new data
        self.market_analyzer.update_metrics(network_stats).await?;
        
        // Retrain demand predictor if needed
        if self.demand_predictor.should_retrain() {
            self.demand_predictor.retrain().await?;
        }
        
        // Optimize pricing parameters
        self.pricing_optimizer.optimize_parameters().await?;
        
        Ok(())
    }

    fn calculate_supply_demand_multiplier(&self, ratio: f64) -> f64 {
        // Higher ratio (more supply) = lower prices
        // Lower ratio (less supply) = higher prices
        match ratio {
            r if r > 2.0 => 0.8,  // Abundant supply, reduce prices
            r if r > 1.5 => 0.9,  // Good supply
            r if r > 1.0 => 1.0,  // Balanced
            r if r > 0.5 => 1.2,  // High demand
            _ => 1.5,             // Very high demand, increase prices
        }
    }

    fn calculate_confidence(&self, supply: &SupplyMetrics, demand: &DemandMetrics, prediction: &DemandPrediction) -> f64 {
        let data_quality = (supply.active_nodes as f64 / 100.0).min(1.0);
        let prediction_confidence = prediction.confidence;
        let historical_accuracy = self.demand_predictor.prediction_model.accuracy;
        
        (data_quality + prediction_confidence + historical_accuracy) / 3.0
    }
}

impl MarketAnalyzer {
    pub fn new() -> Self {
        Self {
            supply_metrics: SupplyMetrics::default(),
            demand_metrics: DemandMetrics::default(),
            historical_data: Vec::new(),
        }
    }

    pub async fn get_current_supply(&self) -> Result<SupplyMetrics> {
        // In a real implementation, this would query the network
        Ok(self.supply_metrics.clone())
    }

    pub async fn get_current_demand(&self) -> Result<DemandMetrics> {
        // In a real implementation, this would query usage statistics
        Ok(self.demand_metrics.clone())
    }

    pub async fn update_metrics(&mut self, network_stats: &NetworkStats) -> Result<()> {
        // Update supply metrics
        self.supply_metrics.total_storage_capacity = network_stats.total_storage;
        self.supply_metrics.available_storage = network_stats.available_storage;
        self.supply_metrics.network_bandwidth = network_stats.total_bandwidth;
        self.supply_metrics.active_nodes = network_stats.active_nodes;
        self.supply_metrics.reliability_score = network_stats.reliability_score;

        // Update demand metrics
        self.demand_metrics.storage_requests = network_stats.storage_requests;
        self.demand_metrics.bandwidth_usage = network_stats.bandwidth_usage;
        
        // Add to historical data
        self.historical_data.push(MarketSnapshot {
            timestamp: Utc::now(),
            supply: self.supply_metrics.clone(),
            demand: self.demand_metrics.clone(),
            price_point: network_stats.current_price,
        });

        // Keep only last 1000 snapshots
        if self.historical_data.len() > 1000 {
            self.historical_data.drain(0..100);
        }

        Ok(())
    }
}

impl DemandPredictor {
    pub fn new() -> Self {
        Self {
            prediction_model: PredictionModel {
                algorithm: "Linear Regression".to_string(),
                accuracy: 0.85,
                last_trained: Utc::now(),
                training_data_size: 1000,
            },
            confidence_threshold: 0.7,
            update_interval: Duration::hours(6),
        }
    }

    pub async fn predict_demand(&self, resource_type: &str, horizon: Duration) -> Result<DemandPrediction> {
        // Simplified prediction logic
        let base_demand = match resource_type {
            "storage" => 1000.0,
            "bandwidth" => 500.0,
            _ => 100.0,
        };

        let growth_factor = 1.0 + (horizon.num_hours() as f64 * 0.01);
        let predicted_demand = base_demand * growth_factor;

        Ok(DemandPrediction {
            resource_type: resource_type.to_string(),
            predicted_value: predicted_demand,
            confidence: self.prediction_model.accuracy,
            prediction_horizon: horizon,
            factors: vec![
                "Historical growth patterns".to_string(),
                "Seasonal adjustments".to_string(),
                "Network capacity trends".to_string(),
            ],
        })
    }

    pub fn should_retrain(&self) -> bool {
        Utc::now() - self.prediction_model.last_trained > self.update_interval
    }

    pub async fn retrain(&mut self) -> Result<()> {
        // In a real implementation, this would retrain the ML model
        self.prediction_model.last_trained = Utc::now();
        self.prediction_model.accuracy = 0.87; // Simulated improvement
        Ok(())
    }
}

impl PricingOptimizer {
    pub fn new() -> Self {
        let mut base_prices = HashMap::new();
        base_prices.insert("storage".to_string(), 0.001); // per GB
        base_prices.insert("bandwidth".to_string(), 0.01); // per GB
        base_prices.insert("priority".to_string(), 0.05); // per request

        Self {
            base_prices,
            multiplier_ranges: MultiplierRanges {
                min_multiplier: 0.5,
                max_multiplier: 2.0,
                optimal_range: (0.8, 1.2),
            },
            optimization_targets: OptimizationTargets {
                target_utilization: 0.85,
                revenue_goal: 10000.0,
                user_satisfaction_threshold: 0.8,
            },
        }
    }

    pub fn get_base_price(&self, resource_type: &str) -> Result<f64> {
        self.base_prices.get(resource_type)
            .copied()
            .ok_or_else(|| format!("Unknown resource type: {}", resource_type).into())
    }

    pub async fn optimize_parameters(&mut self) -> Result<()> {
        // Simplified optimization logic
        // In a real implementation, this would use ML algorithms
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandPrediction {
    pub resource_type: String,
    pub predicted_value: f64,
    pub confidence: f64,
    pub prediction_horizon: Duration,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_storage: u64,
    pub available_storage: u64,
    pub total_bandwidth: u64,
    pub bandwidth_usage: u64,
    pub active_nodes: u32,
    pub reliability_score: f64,
    pub storage_requests: u64,
    pub current_price: f64,
}

impl Default for SupplyMetrics {
    fn default() -> Self {
        Self {
            total_storage_capacity: 1000000000, // 1TB
            available_storage: 500000000,       // 500GB
            network_bandwidth: 1000000000,      // 1Gbps
            active_nodes: 100,
            reliability_score: 0.95,
        }
    }
}

impl Default for DemandMetrics {
    fn default() -> Self {
        Self {
            storage_requests: 1000,
            bandwidth_usage: 500000000, // 500MB
            peak_times: Vec::new(),
            growth_rate: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dynamic_pricing_calculation() {
        let engine = DynamicPricingEngine::new();
        let recommendation = engine.calculate_optimal_pricing("storage", Some("us-west")).await.unwrap();
        
        assert!(recommendation.final_price > 0.0);
        assert!(recommendation.confidence > 0.0);
        assert!(recommendation.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_demand_prediction() {
        let predictor = DemandPredictor::new();
        let prediction = predictor.predict_demand("storage", Duration::hours(24)).await.unwrap();
        
        assert!(prediction.predicted_value > 0.0);
        assert!(prediction.confidence > 0.0);
    }
}
