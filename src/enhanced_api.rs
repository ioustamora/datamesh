use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use crate::dynamic_pricing::DynamicPricingEngine;
use crate::flexible_storage::FlexibleStorageManager;
use crate::gamification::ContributionGameification;
use crate::intelligent_cli::IntelligentCLIAssistant;
use crate::pricing_assistant::PricingAssistant;
use crate::error::Result;
use crate::database::DatabaseManager;
use crate::economics::EconomicService;

#[derive(Debug, Clone)]
pub struct EnhancedApiServer {
    pub pricing_engine: Arc<RwLock<DynamicPricingEngine>>,
    pub storage_manager: Arc<RwLock<FlexibleStorageManager>>,
    pub gamification: Arc<RwLock<ContributionGameification>>,
    pub cli_assistant: Arc<RwLock<IntelligentCLIAssistant>>,
    pub pricing_assistant: Arc<RwLock<PricingAssistant>>,
    pub database: Arc<DatabaseManager>,
    pub economics: Arc<EconomicService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingRequest {
    pub user_id: String,
    pub storage_size: u64,
    pub access_frequency: f64,
    pub duration: std::time::Duration,
    pub tier: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingResponse {
    pub base_price: f64,
    pub dynamic_adjustment: f64,
    pub final_price: f64,
    pub tier_recommendation: Option<String>,
    pub savings_opportunities: Vec<String>,
    pub market_comparison: MarketComparison,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketComparison {
    pub position: String,
    pub percentile: f64,
    pub competitive_advantage: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageOptimizationRequest {
    pub user_id: String,
    pub current_usage: u64,
    pub access_patterns: Vec<String>,
    pub performance_requirements: PerformanceRequirements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub latency_target: f64,
    pub throughput_target: f64,
    pub availability_target: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageOptimizationResponse {
    pub recommended_tier: String,
    pub burst_capacity: u64,
    pub cost_savings: f64,
    pub performance_improvement: f64,
    pub migration_plan: MigrationPlan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub steps: Vec<MigrationStep>,
    pub estimated_downtime: std::time::Duration,
    pub rollback_strategy: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStep {
    pub step_number: u32,
    pub description: String,
    pub estimated_duration: std::time::Duration,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GamificationRequest {
    pub user_id: String,
    pub action: String,
    pub context: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GamificationResponse {
    pub points_earned: u64,
    pub new_level: u32,
    pub achievements_unlocked: Vec<Achievement>,
    pub leaderboard_position: u32,
    pub next_milestone: Milestone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub rarity: String,
    pub reward: Reward,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reward {
    pub reward_type: String,
    pub value: f64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub progress: f64,
    pub target: f64,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentAssistRequest {
    pub user_id: String,
    pub query: String,
    pub context: QueryContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryContext {
    pub current_command: Option<String>,
    pub previous_commands: Vec<String>,
    pub user_level: String,
    pub preferred_style: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentAssistResponse {
    pub response: String,
    pub suggested_commands: Vec<SuggestedCommand>,
    pub learning_tips: Vec<String>,
    pub related_documentation: Vec<DocumentationLink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedCommand {
    pub command: String,
    pub description: String,
    pub confidence: f64,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationLink {
    pub title: String,
    pub url: String,
    pub relevance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingAssistantRequest {
    pub user_id: String,
    pub analysis_type: String,
    pub timeframe: Option<std::time::Duration>,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingAssistantResponse {
    pub analysis_results: serde_json::Value,
    pub recommendations: Vec<String>,
    pub cost_predictions: Vec<CostPrediction>,
    pub savings_opportunities: Vec<SavingsOpportunity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostPrediction {
    pub timeframe: std::time::Duration,
    pub predicted_cost: f64,
    pub confidence_level: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavingsOpportunity {
    pub category: String,
    pub potential_savings: f64,
    pub implementation_effort: String,
    pub roi_timeframe: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardDataRequest {
    pub user_id: String,
    pub timeframe: String,
    pub metrics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardDataResponse {
    pub metrics: std::collections::HashMap<String, MetricData>,
    pub insights: Vec<Insight>,
    pub alerts: Vec<Alert>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricData {
    pub current_value: f64,
    pub previous_value: f64,
    pub trend: String,
    pub historical_data: Vec<DataPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub action_items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub category: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub estimated_impact: f64,
}

impl EnhancedApiServer {
    pub fn new(
        pricing_engine: DynamicPricingEngine,
        storage_manager: FlexibleStorageManager,
        gamification: ContributionGameification,
        cli_assistant: IntelligentCLIAssistant,
        pricing_assistant: PricingAssistant,
        database: Arc<DatabaseManager>,
        economics: Arc<EconomicService>,
    ) -> Self {
        Self {
            pricing_engine: Arc::new(RwLock::new(pricing_engine)),
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            gamification: Arc::new(RwLock::new(gamification)),
            cli_assistant: Arc::new(RwLock::new(cli_assistant)),
            pricing_assistant: Arc::new(RwLock::new(pricing_assistant)),
            database,
            economics,
        }
    }

    pub async fn start_server(&self, port: u16) -> Result<()> {
        let routes = self.build_routes();
        
        println!("Starting Enhanced API Server on port {}", port);
        warp::serve(routes)
            .run(([0, 0, 0, 0], port))
            .await;
        
        Ok(())
    }

    fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let pricing_routes = self.pricing_routes();
        let storage_routes = self.storage_routes();
        let gamification_routes = self.gamification_routes();
        let assistant_routes = self.assistant_routes();
        let dashboard_routes = self.dashboard_routes();

        pricing_routes
            .or(storage_routes)
            .or(gamification_routes)
            .or(assistant_routes)
            .or(dashboard_routes)
            .with(cors)
    }

    fn pricing_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let pricing_engine = self.pricing_engine.clone();
        let pricing_assistant_predict = self.pricing_assistant.clone();
        let pricing_assistant_market = self.pricing_assistant.clone();

        // Dynamic pricing calculation
        let calculate_price = warp::path!("api" / "v1" / "pricing" / "calculate")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || pricing_engine.clone()))
            .and_then(Self::calculate_dynamic_price);

        // Price predictions
        let predict_price = warp::path!("api" / "v1" / "pricing" / "predict")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || pricing_assistant_predict.clone()))
            .and_then(Self::predict_future_pricing);

        // Market analysis
        let market_analysis = warp::path!("api" / "v1" / "pricing" / "market-analysis")
            .and(warp::get())
            .and(warp::any().map(move || pricing_assistant_market.clone()))
            .and_then(Self::get_market_analysis);

        calculate_price.or(predict_price).or(market_analysis)
    }

    fn storage_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let storage_manager_optimize = self.storage_manager.clone();
        let storage_manager_recommend = self.storage_manager.clone();

        // Storage optimization
        let optimize_storage = warp::path!("api" / "v1" / "storage" / "optimize")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || storage_manager_optimize.clone()))
            .and_then(Self::optimize_storage);

        // Tier recommendations
        let recommend_tier = warp::path!("api" / "v1" / "storage" / "recommend-tier")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || storage_manager_recommend.clone()))
            .and_then(Self::recommend_storage_tier);

        optimize_storage.or(recommend_tier)
    }

    fn gamification_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let gamification_record = self.gamification.clone();
        let gamification_progress = self.gamification.clone();
        let gamification_leaderboard = self.gamification.clone();

        // Record user action
        let record_action = warp::path!("api" / "v1" / "gamification" / "action")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || gamification_record.clone()))
            .and_then(Self::record_gamification_action);

        // Get user progress
        let get_progress = warp::path!("api" / "v1" / "gamification" / "progress" / String)
            .and(warp::get())
            .and(warp::any().map(move || gamification_progress.clone()))
            .and_then(Self::get_user_progress);

        // Get leaderboard
        let get_leaderboard = warp::path!("api" / "v1" / "gamification" / "leaderboard")
            .and(warp::get())
            .and(warp::any().map(move || gamification_leaderboard.clone()))
            .and_then(Self::get_leaderboard);

        record_action.or(get_progress).or(get_leaderboard)
    }

    fn assistant_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let cli_assistant = self.cli_assistant.clone();
        let pricing_assistant = self.pricing_assistant.clone();

        // CLI assistance
        let cli_help = warp::path!("api" / "v1" / "assistant" / "cli")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || cli_assistant.clone()))
            .and_then(Self::get_cli_assistance);

        // Pricing assistance
        let pricing_help = warp::path!("api" / "v1" / "assistant" / "pricing")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || pricing_assistant.clone()))
            .and_then(Self::get_pricing_assistance);

        cli_help.or(pricing_help)
    }

    fn dashboard_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        // Simplified dashboard routes to avoid complex trait bounds
        warp::path("api")
            .and(warp::path("v1"))
            .and(warp::path("dashboard"))
            .and(warp::path("data"))
            .and(warp::post())
            .and(warp::body::json())
            .map(|request: serde_json::Value| {
                warp::reply::json(&serde_json::json!({
                    "success": true,
                    "data": "Dashboard data placeholder"
                }))
            })
    }

    // Route handlers
    async fn calculate_dynamic_price(
        request: PricingRequest,
        pricing_engine: Arc<RwLock<DynamicPricingEngine>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let engine = pricing_engine.read().await;
        
        let pricing_recommendation = engine.calculate_optimal_pricing("storage", request.region.as_deref()).await
            .map_err(|_| warp::reject::custom(ApiError::pricing_error()))?;

        let base_price = pricing_recommendation.base_price;
        let dynamic_adjustment = pricing_recommendation.final_price - pricing_recommendation.base_price;
        let final_price = pricing_recommendation.final_price;

        let response = PricingResponse {
            base_price,
            dynamic_adjustment,
            final_price,
            tier_recommendation: Some("Standard".to_string()),
            savings_opportunities: vec!["Consider upgrading to Premium for better rates".to_string()],
            market_comparison: MarketComparison {
                position: "Competitive".to_string(),
                percentile: 75.0,
                competitive_advantage: vec!["Lower latency".to_string(), "Better reliability".to_string()],
            },
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn predict_future_pricing(
        request: PricingAssistantRequest,
        pricing_assistant: Arc<RwLock<PricingAssistant>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let assistant = pricing_assistant.read().await;
        
        let predictions = assistant.predict_future_costs(&request.user_id, chrono::Duration::from_std(request.timeframe.unwrap_or(std::time::Duration::from_secs(30 * 24 * 3600))).unwrap()).await
            .map_err(|_| warp::reject::custom(ApiError::prediction_error()))?;

        let response = PricingAssistantResponse {
            analysis_results: serde_json::json!(predictions),
            recommendations: vec!["Consider optimizing storage access patterns".to_string()],
            cost_predictions: vec![CostPrediction {
                timeframe: std::time::Duration::from_secs(30 * 24 * 3600),
                predicted_cost: predictions.predicted_cost,
                confidence_level: predictions.confidence_level,
            }],
            savings_opportunities: vec![SavingsOpportunity {
                category: "Storage Optimization".to_string(),
                potential_savings: 150.0,
                implementation_effort: "Medium".to_string(),
                roi_timeframe: std::time::Duration::from_secs(7 * 24 * 3600),
            }],
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn get_market_analysis(
        pricing_assistant: Arc<RwLock<PricingAssistant>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let assistant = pricing_assistant.read().await;
        
        let market_analysis = assistant.provide_market_insights().await
            .map_err(|_| warp::reject::custom(ApiError::analysis_error()))?;

        Ok(warp::reply::json(&Self::success_response(market_analysis)))
    }

    async fn optimize_storage(
        request: StorageOptimizationRequest,
        storage_manager: Arc<RwLock<FlexibleStorageManager>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let manager = storage_manager.read().await;
        
        let optimization = manager.optimize_user_storage(&request.user_id, request.current_usage).await
            .map_err(|_| warp::reject::custom(ApiError::optimization_error()))?;

        let response = StorageOptimizationResponse {
            recommended_tier: optimization.recommended_tier,
            burst_capacity: optimization.burst_capacity,
            cost_savings: optimization.cost_savings,
            performance_improvement: optimization.performance_improvement,
            migration_plan: MigrationPlan {
                steps: vec![
                    MigrationStep {
                        step_number: 1,
                        description: "Backup current data".to_string(),
                        estimated_duration: std::time::Duration::from_secs(3600),
                        dependencies: vec![],
                    },
                    MigrationStep {
                        step_number: 2,
                        description: "Migrate to new tier".to_string(),
                        estimated_duration: std::time::Duration::from_secs(1800),
                        dependencies: vec!["backup_complete".to_string()],
                    },
                ],
                estimated_downtime: std::time::Duration::from_secs(300),
                rollback_strategy: "Automatic rollback if migration fails".to_string(),
            },
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn recommend_storage_tier(
        request: StorageOptimizationRequest,
        storage_manager: Arc<RwLock<FlexibleStorageManager>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let manager = storage_manager.read().await;
        
        let recommendation = manager.recommend_tier(&request.user_id, &request.access_patterns).await
            .map_err(|_| warp::reject::custom(ApiError::recommendation_error()))?;

        Ok(warp::reply::json(&Self::success_response(recommendation)))
    }

    async fn record_gamification_action(
        request: GamificationRequest,
        gamification: Arc<RwLock<ContributionGameification>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let mut game = gamification.write().await;
        
        let result = game.record_contribution(&request.user_id, &request.action, 1.0).await
            .map_err(|_| warp::reject::custom(ApiError::gamification_error()))?;

        let response = GamificationResponse {
            points_earned: result.points_earned,
            new_level: result.new_level,
            achievements_unlocked: result.achievements_unlocked.iter().map(|a| Achievement {
                id: a.id.clone(),
                name: a.name.clone(),
                description: a.description.clone(),
                icon: a.icon.clone(),
                rarity: format!("{:?}", a.tier),
                reward: Reward {
                    reward_type: "tokens".to_string(),
                    value: a.rewards.tokens as f64,
                    description: format!("Tokens: {}, Storage: {} bytes", a.rewards.tokens, a.rewards.storage_bonus),
                },
            }).collect(),
            leaderboard_position: result.leaderboard_position,
            next_milestone: Milestone {
                name: "Next Level".to_string(),
                progress: result.progress_to_next_level,
                target: 100.0,
                estimated_completion: chrono::Utc::now() + chrono::Duration::days(7),
            },
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn get_user_progress(
        user_id: String,
        gamification: Arc<RwLock<ContributionGameification>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let game = gamification.read().await;
        
        let progress = game.get_user_progress(&user_id).await
            .map_err(|_| warp::reject::custom(ApiError::gamification_error()))?;

        Ok(warp::reply::json(&Self::success_response(progress)))
    }

    async fn get_leaderboard(
        gamification: Arc<RwLock<ContributionGameification>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let game = gamification.read().await;
        
        let leaderboard = game.get_leaderboard(10).await
            .map_err(|_| warp::reject::custom(ApiError::gamification_error()))?;

        Ok(warp::reply::json(&Self::success_response(leaderboard)))
    }

    async fn get_cli_assistance(
        request: IntelligentAssistRequest,
        cli_assistant: Arc<RwLock<IntelligentCLIAssistant>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let assistant = cli_assistant.read().await;
        
        // Create a mock InteractiveSession for the API call
        let mock_session = crate::interactive::InteractiveSession::new();
        let assistance = assistant.provide_assistance(&request.query, &mock_session).await
            .map_err(|_| warp::reject::custom(ApiError::assistance_error()))?;

        let response = IntelligentAssistResponse {
            response: "Help provided".to_string(), // Stub response
            suggested_commands: vec![], // Stub suggestions
            learning_tips: vec!["Practice regularly".to_string()], // Stub tips
            related_documentation: vec![], // Stub documentation
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn get_pricing_assistance(
        request: PricingAssistantRequest,
        pricing_assistant: Arc<RwLock<PricingAssistant>>,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let mut assistant = pricing_assistant.write().await;
        
        let recommendations = assistant.analyze_user_costs(&request.user_id).await
            .map_err(|_| warp::reject::custom(ApiError::assistance_error()))?;

        let response = PricingAssistantResponse {
            analysis_results: serde_json::json!(recommendations),
            recommendations: recommendations.iter().map(|r| r.title.clone()).collect(),
            cost_predictions: vec![],
            savings_opportunities: vec![],
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn get_dashboard_data(
        request: DashboardDataRequest,
        api_server: Self,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        // Collect data from various sources
        let mut metrics = std::collections::HashMap::new();
        
        // Add metrics based on request
        for metric in &request.metrics {
            let metric_data = MetricData {
                current_value: 100.0,
                previous_value: 95.0,
                trend: "up".to_string(),
                historical_data: vec![
                    DataPoint {
                        timestamp: chrono::Utc::now() - chrono::Duration::hours(24),
                        value: 95.0,
                    },
                    DataPoint {
                        timestamp: chrono::Utc::now(),
                        value: 100.0,
                    },
                ],
            };
            metrics.insert(metric.clone(), metric_data);
        }

        let response = DashboardDataResponse {
            metrics,
            insights: vec![
                Insight {
                    title: "Storage Usage Optimization".to_string(),
                    description: "You could save 15% by optimizing your storage tier".to_string(),
                    category: "cost".to_string(),
                    severity: "medium".to_string(),
                    action_items: vec!["Review storage tiers".to_string()],
                },
            ],
            alerts: vec![],
            recommendations: vec![
                Recommendation {
                    id: "rec1".to_string(),
                    title: "Upgrade Storage Tier".to_string(),
                    description: "Upgrading to Premium tier will reduce your costs".to_string(),
                    category: "optimization".to_string(),
                    priority: "high".to_string(),
                    estimated_impact: 150.0,
                },
            ],
        };

        Ok(warp::reply::json(&Self::success_response(response)))
    }

    async fn get_real_time_metrics(
        user_id: String,
        api_server: Self,
    ) -> std::result::Result<impl Reply, warp::Rejection> {
        let metrics: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        Ok(warp::reply::json(&Self::success_response(metrics)))
    }

    // Helper methods
    fn success_response<T: Serialize>(data: T) -> ApiResponse<T> {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn error_response(error: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug)]
struct ApiError {
    message: String,
}

impl ApiError {
    fn pricing_error() -> Self {
        Self {
            message: "Pricing calculation failed".to_string(),
        }
    }

    fn prediction_error() -> Self {
        Self {
            message: "Price prediction failed".to_string(),
        }
    }

    fn analysis_error() -> Self {
        Self {
            message: "Analysis failed".to_string(),
        }
    }

    fn optimization_error() -> Self {
        Self {
            message: "Storage optimization failed".to_string(),
        }
    }

    fn recommendation_error() -> Self {
        Self {
            message: "Recommendation generation failed".to_string(),
        }
    }

    fn gamification_error() -> Self {
        Self {
            message: "Gamification operation failed".to_string(),
        }
    }

    fn assistance_error() -> Self {
        Self {
            message: "Assistance request failed".to_string(),
        }
    }
}

impl warp::reject::Reject for ApiError {}
