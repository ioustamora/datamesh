use super::alerts::Alert;
use super::{PerformanceTrend, SystemMetrics};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Real-time monitoring dashboard data aggregator
/// Provides comprehensive dashboard data with real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub current_metrics: SystemMetrics,
    pub recent_alerts: Vec<Alert>,
    pub performance_trends: Vec<PerformanceTrend>,
    pub system_health: SystemHealth,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_score: f64,
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub recommendations: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub health_score: f64,
    pub status: HealthStatus,
    pub metrics: HashMap<String, f64>,
    pub issues: Vec<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Real-time dashboard for monitoring system
/// Provides live data updates and interactive visualization
pub struct MonitoringDashboard {
    refresh_interval: Duration,
    data_cache: Arc<RwLock<DashboardCache>>,
    widget_configs: Arc<RwLock<Vec<WidgetConfig>>>,
    user_preferences: Arc<RwLock<HashMap<String, UserPreferences>>>,
    alert_subscriptions: Arc<RwLock<HashMap<String, Vec<AlertSubscription>>>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCache {
    pub live_data: DashboardData,
    pub historical_snapshots: Vec<HistoricalSnapshot>,
    pub aggregated_metrics: AggregatedMetrics,
    pub alert_statistics: AlertStatistics,
    pub performance_summaries: Vec<PerformanceSummary>,
    pub last_updated: DateTime<Utc>,
    pub cache_expiry: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub widget_id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub metrics: Vec<String>,
    pub refresh_rate: Duration,
    pub display_options: DisplayOptions,
    pub filters: Vec<MetricFilter>,
    pub position: WidgetPosition,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub user_id: String,
    pub dashboard_layout: DashboardLayout,
    pub default_time_range: Duration,
    pub notification_settings: NotificationSettings,
    pub custom_widgets: Vec<CustomWidget>,
    pub theme: DashboardTheme,
    pub auto_refresh: bool,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSubscription {
    pub user_id: String,
    pub alert_types: Vec<String>,
    pub severity_threshold: AlertSeverity,
    pub delivery_method: DeliveryMethod,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: SystemMetrics,
    pub health_score: f64,
    pub active_alerts: u32,
    pub performance_grade: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub hourly_averages: HashMap<String, Vec<f64>>,
    pub daily_summaries: HashMap<String, DailySummary>,
    pub weekly_trends: HashMap<String, WeeklyTrend>,
    pub monthly_reports: HashMap<String, MonthlyReport>,
    pub peak_usage_times: Vec<PeakUsage>,
    pub performance_baselines: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts_today: u32,
    pub critical_alerts_today: u32,
    pub resolved_alerts_today: u32,
    pub average_resolution_time: Duration,
    pub alert_trend: Vec<AlertTrendPoint>,
    pub top_alert_sources: Vec<AlertSource>,
    pub false_positive_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub time_period: (DateTime<Utc>, DateTime<Utc>),
    pub average_response_time: f64,
    pub throughput: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub availability: f64,
    pub user_satisfaction: f64,
    pub key_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    Table,
    Metric,
    Alert,
    Heatmap,
    Histogram,
    Scatter,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    pub show_legend: bool,
    pub show_grid: bool,
    pub color_scheme: ColorScheme,
    pub chart_style: ChartStyle,
    pub animation_enabled: bool,
    pub data_points_limit: u32,
    pub real_time_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFilter {
    pub metric_name: String,
    pub filter_type: FilterType,
    pub filter_value: String,
    pub operator: FilterOperator,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub z_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub layout_name: String,
    pub widget_positions: HashMap<String, WidgetPosition>,
    pub grid_size: (u32, u32),
    pub responsive: bool,
    pub saved_views: Vec<SavedView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub sms_enabled: bool,
    pub slack_enabled: bool,
    pub notification_frequency: NotificationFrequency,
    pub quiet_hours: Option<QuietHours>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomWidget {
    pub widget_id: String,
    pub widget_name: String,
    pub query: String,
    pub visualization_type: WidgetType,
    pub refresh_interval: Duration,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    Light,
    Dark,
    Auto,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryMethod {
    Email,
    Push,
    SMS,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: DateTime<Utc>,
    pub average_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub total_samples: u32,
    pub trend_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyTrend {
    pub week_start: DateTime<Utc>,
    pub trend_percentage: f64,
    pub volatility: f64,
    pub peak_day: u8,
    pub lowest_day: u8,
    pub weekly_average: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub month: DateTime<Utc>,
    pub monthly_average: f64,
    pub growth_rate: f64,
    pub performance_grade: String,
    pub key_events: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakUsage {
    pub metric_name: String,
    pub peak_time: DateTime<Utc>,
    pub peak_value: f64,
    pub duration: Duration,
    pub frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub alert_count: u32,
    pub severity_distribution: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSource {
    pub source_name: String,
    pub alert_count: u32,
    pub average_severity: f64,
    pub resolution_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Monochrome,
    Rainbow,
    Corporate,
    Accessibility,
    Custom(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartStyle {
    Minimal,
    Detailed,
    Professional,
    Colorful,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Range,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub view_name: String,
    pub view_config: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationFrequency {
    Immediate,
    Every5Minutes,
    Every15Minutes,
    Every30Minutes,
    Hourly,
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_time: String, // Format: "HH:MM"
    pub end_time: String,   // Format: "HH:MM"
    pub timezone: String,
    pub enabled: bool,
}

impl Default for DashboardCache {
    fn default() -> Self {
        Self {
            live_data: DashboardData {
                current_metrics: SystemMetrics {
                    timestamp: Utc::now(),
                    node_id: "dashboard-node".to_string(),
                    throughput_mbps: 0.0,
                    avg_response_time_ms: 0.0,
                    success_rate: 0.0,
                    active_connections: 0,
                    request_queue_length: 0,
                    error_rate: 0.0,
                    total_files: 0,
                    total_size_bytes: 0,
                    storage_efficiency: 0.0,
                    redundancy_factor: 0.0,
                    chunk_availability: 0.0,
                    deduplication_ratio: 0.0,
                    peer_count: 0,
                    dht_size: 0,
                    network_health_score: 0.0,
                    bootstrap_node_count: 0,
                    consensus_participation: 0.0,
                    memory_usage_mb: 0,
                    cpu_usage_percent: 0.0,
                    disk_usage_gb: 0,
                    network_io_mbps: 0.0,
                    uptime_seconds: 0,
                    active_users: 0,
                    new_registrations: 0,
                    user_satisfaction_score: 0.0,
                    support_tickets: 0,
                    active_proposals: 0,
                    voting_participation: 0.0,
                    operator_reputation_avg: 0.0,
                    governance_health: 0.0,
                    custom_metrics: HashMap::new(),
                },
                recent_alerts: vec![],
                performance_trends: vec![],
                system_health: SystemHealth {
                    overall_score: 0.0,
                    status: HealthStatus::Unknown,
                    components: vec![],
                    recommendations: vec![],
                    last_updated: Utc::now(),
                },
                timestamp: Utc::now(),
            },
            historical_snapshots: vec![],
            aggregated_metrics: AggregatedMetrics {
                hourly_averages: HashMap::new(),
                daily_summaries: HashMap::new(),
                weekly_trends: HashMap::new(),
                monthly_reports: HashMap::new(),
                peak_usage_times: vec![],
                performance_baselines: HashMap::new(),
            },
            alert_statistics: AlertStatistics {
                total_alerts_today: 0,
                critical_alerts_today: 0,
                resolved_alerts_today: 0,
                average_resolution_time: Duration::from_secs(0),
                alert_trend: vec![],
                top_alert_sources: vec![],
                false_positive_rate: 0.0,
            },
            performance_summaries: vec![],
            last_updated: Utc::now(),
            cache_expiry: Utc::now() + Duration::from_secs(300),
        }
    }
}

impl MonitoringDashboard {
    pub async fn new(refresh_interval: Duration) -> Result<Self> {
        Ok(Self {
            refresh_interval,
            data_cache: Arc::new(RwLock::new(DashboardCache::default())),
            widget_configs: Arc::new(RwLock::new(Self::default_widget_configs())),
            user_preferences: Arc::new(RwLock::new(HashMap::new())),
            alert_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the dashboard service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        *running = true;

        // Start dashboard update loop
        self.start_dashboard_loop().await?;

        tracing::info!("Monitoring Dashboard started successfully");
        Ok(())
    }

    /// Stop the dashboard service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        *running = false;

        tracing::info!("Monitoring Dashboard stopped");
        Ok(())
    }

    /// Get current dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let cache = self.data_cache.read().await;

        // Check if cache is still valid
        if cache.cache_expiry > Utc::now() {
            Ok(cache.live_data.clone())
        } else {
            // Cache expired, return current data and trigger refresh
            Ok(cache.live_data.clone())
        }
    }

    /// Get dashboard configuration for a user
    pub async fn get_user_dashboard_config(&self, user_id: &str) -> Result<UserDashboardConfig> {
        let preferences = self.user_preferences.read().await;
        let widgets = self.widget_configs.read().await;

        let user_prefs = preferences.get(user_id).cloned().unwrap_or_else(|| {
            UserPreferences {
                user_id: user_id.to_string(),
                dashboard_layout: DashboardLayout {
                    layout_name: "Default".to_string(),
                    widget_positions: HashMap::new(),
                    grid_size: (12, 8),
                    responsive: true,
                    saved_views: vec![],
                },
                default_time_range: Duration::from_secs(24 * 60 * 60), // 24 hours
                notification_settings: NotificationSettings {
                    email_enabled: true,
                    push_enabled: false,
                    sms_enabled: false,
                    slack_enabled: false,
                    notification_frequency: NotificationFrequency::Immediate,
                    quiet_hours: None,
                },
                custom_widgets: vec![],
                theme: DashboardTheme::Light,
                auto_refresh: true,
                last_modified: Utc::now(),
            }
        });

        Ok(UserDashboardConfig {
            user_preferences: user_prefs,
            available_widgets: widgets.clone(),
            dashboard_capabilities: DashboardCapabilities {
                supports_real_time: true,
                supports_historical: true,
                supports_alerts: true,
                supports_custom_widgets: true,
                max_widgets: 50,
                max_custom_queries: 20,
            },
        })
    }

    /// Update user dashboard preferences
    pub async fn update_user_preferences(
        &self,
        user_id: &str,
        preferences: UserPreferences,
    ) -> Result<()> {
        let mut user_prefs = self.user_preferences.write().await;
        user_prefs.insert(user_id.to_string(), preferences);

        tracing::info!("Updated dashboard preferences for user: {}", user_id);
        Ok(())
    }

    /// Subscribe user to alert notifications
    pub async fn subscribe_to_alerts(
        &self,
        user_id: &str,
        subscription: AlertSubscription,
    ) -> Result<()> {
        let mut subscriptions = self.alert_subscriptions.write().await;

        subscriptions
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(subscription);

        tracing::info!("Added alert subscription for user: {}", user_id);
        Ok(())
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        let cache = self.data_cache.read().await;
        Ok(cache.live_data.system_health.clone())
    }

    /// Get aggregated metrics for time range
    pub async fn get_aggregated_metrics(&self, _time_range: Duration) -> Result<AggregatedMetrics> {
        let cache = self.data_cache.read().await;

        // Filter metrics based on time range
        let filtered_metrics = cache.aggregated_metrics.clone();

        // Apply time range filtering logic here
        // For now, return all metrics
        Ok(filtered_metrics)
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(&self) -> Result<AlertStatistics> {
        let cache = self.data_cache.read().await;
        Ok(cache.alert_statistics.clone())
    }

    /// Export dashboard data
    pub async fn export_dashboard_data(
        &self,
        format: ExportFormat,
        time_range: Duration,
    ) -> Result<ExportResult> {
        let cache = self.data_cache.read().await;

        let export_data = DashboardExportData {
            export_timestamp: Utc::now(),
            time_range,
            current_metrics: cache.live_data.current_metrics.clone(),
            historical_snapshots: cache.historical_snapshots.clone(),
            aggregated_metrics: cache.aggregated_metrics.clone(),
            alert_statistics: cache.alert_statistics.clone(),
            performance_summaries: cache.performance_summaries.clone(),
        };

        let exported_data = match format {
            ExportFormat::JSON => serde_json::to_string_pretty(&export_data)?,
            ExportFormat::CSV => self.convert_to_csv(&export_data).await?,
            ExportFormat::Excel => self.convert_to_excel(&export_data).await?,
            ExportFormat::PDF => self.convert_to_pdf(&export_data).await?,
        };

        let filename = format!(
            "dashboard_export_{}.{}",
            Utc::now().format("%Y%m%d_%H%M%S"),
            format.file_extension()
        );

        let data_size = exported_data.len();
        Ok(ExportResult {
            format,
            data: exported_data,
            filename,
            size_bytes: data_size,
            generated_at: Utc::now(),
        })
    }

    // Private helper methods
    fn default_widget_configs() -> Vec<WidgetConfig> {
        vec![
            WidgetConfig {
                widget_id: "system_health".to_string(),
                widget_type: WidgetType::Gauge,
                title: "System Health".to_string(),
                metrics: vec!["overall_health_score".to_string()],
                refresh_rate: Duration::from_secs(5),
                display_options: DisplayOptions {
                    show_legend: false,
                    show_grid: false,
                    color_scheme: ColorScheme::Default,
                    chart_style: ChartStyle::Minimal,
                    animation_enabled: true,
                    data_points_limit: 100,
                    real_time_updates: true,
                },
                filters: vec![],
                position: WidgetPosition {
                    x: 0,
                    y: 0,
                    width: 3,
                    height: 2,
                    z_index: 1,
                },
                enabled: true,
            },
            WidgetConfig {
                widget_id: "performance_chart".to_string(),
                widget_type: WidgetType::LineChart,
                title: "Performance Metrics".to_string(),
                metrics: vec![
                    "avg_response_time_ms".to_string(),
                    "throughput_mbps".to_string(),
                    "success_rate".to_string(),
                ],
                refresh_rate: Duration::from_secs(30),
                display_options: DisplayOptions {
                    show_legend: true,
                    show_grid: true,
                    color_scheme: ColorScheme::Default,
                    chart_style: ChartStyle::Professional,
                    animation_enabled: true,
                    data_points_limit: 200,
                    real_time_updates: true,
                },
                filters: vec![],
                position: WidgetPosition {
                    x: 3,
                    y: 0,
                    width: 6,
                    height: 4,
                    z_index: 1,
                },
                enabled: true,
            },
            WidgetConfig {
                widget_id: "alerts_table".to_string(),
                widget_type: WidgetType::Table,
                title: "Recent Alerts".to_string(),
                metrics: vec!["recent_alerts".to_string()],
                refresh_rate: Duration::from_secs(10),
                display_options: DisplayOptions {
                    show_legend: false,
                    show_grid: true,
                    color_scheme: ColorScheme::Default,
                    chart_style: ChartStyle::Detailed,
                    animation_enabled: false,
                    data_points_limit: 50,
                    real_time_updates: true,
                },
                filters: vec![],
                position: WidgetPosition {
                    x: 9,
                    y: 0,
                    width: 3,
                    height: 4,
                    z_index: 1,
                },
                enabled: true,
            },
            WidgetConfig {
                widget_id: "resource_usage".to_string(),
                widget_type: WidgetType::BarChart,
                title: "Resource Usage".to_string(),
                metrics: vec![
                    "cpu_usage_percent".to_string(),
                    "memory_usage_mb".to_string(),
                    "disk_usage_gb".to_string(),
                ],
                refresh_rate: Duration::from_secs(15),
                display_options: DisplayOptions {
                    show_legend: true,
                    show_grid: true,
                    color_scheme: ColorScheme::Default,
                    chart_style: ChartStyle::Colorful,
                    animation_enabled: true,
                    data_points_limit: 100,
                    real_time_updates: true,
                },
                filters: vec![],
                position: WidgetPosition {
                    x: 0,
                    y: 4,
                    width: 6,
                    height: 3,
                    z_index: 1,
                },
                enabled: true,
            },
            WidgetConfig {
                widget_id: "network_topology".to_string(),
                widget_type: WidgetType::Custom,
                title: "Network Topology".to_string(),
                metrics: vec![
                    "peer_count".to_string(),
                    "network_health_score".to_string(),
                    "consensus_participation".to_string(),
                ],
                refresh_rate: Duration::from_secs(60),
                display_options: DisplayOptions {
                    show_legend: false,
                    show_grid: false,
                    color_scheme: ColorScheme::Default,
                    chart_style: ChartStyle::Minimal,
                    animation_enabled: true,
                    data_points_limit: 1000,
                    real_time_updates: true,
                },
                filters: vec![],
                position: WidgetPosition {
                    x: 6,
                    y: 4,
                    width: 6,
                    height: 3,
                    z_index: 1,
                },
                enabled: true,
            },
        ]
    }

    async fn start_dashboard_loop(&self) -> Result<()> {
        let data_cache = self.data_cache.clone();
        let refresh_interval = self.refresh_interval;
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refresh_interval);

            loop {
                interval.tick().await;

                let running = is_running.read().await;
                if !*running {
                    break;
                }
                drop(running);

                if let Err(e) = Self::update_dashboard_data(&data_cache).await {
                    tracing::error!("Error updating dashboard data: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn update_dashboard_data(data_cache: &Arc<RwLock<DashboardCache>>) -> Result<()> {
        let mut cache = data_cache.write().await;

        // Update timestamps
        cache.last_updated = Utc::now();
        cache.cache_expiry = Utc::now() + Duration::from_secs(300);

        // Update live data (simplified - would integrate with actual monitoring system)
        cache.live_data.timestamp = Utc::now();
        cache.live_data.system_health.last_updated = Utc::now();

        // Add historical snapshot
        let current_metrics = cache.live_data.current_metrics.clone();
        let health_score = cache.live_data.system_health.overall_score;
        let active_alerts = cache.live_data.recent_alerts.len() as u32;

        cache.historical_snapshots.push(HistoricalSnapshot {
            timestamp: Utc::now(),
            metrics: current_metrics,
            health_score,
            active_alerts,
            performance_grade: "Good".to_string(),
        });

        // Keep only recent snapshots
        if cache.historical_snapshots.len() > 1000 {
            cache.historical_snapshots.remove(0);
        }

        tracing::debug!("Updated dashboard data");
        Ok(())
    }

    async fn convert_to_csv(&self, _data: &DashboardExportData) -> Result<String> {
        // Simplified CSV conversion
        Ok("timestamp,metric,value\n2024-01-01T00:00:00Z,cpu_usage,50.0\n".to_string())
    }

    async fn convert_to_excel(&self, _data: &DashboardExportData) -> Result<String> {
        // Simplified Excel conversion (would use actual Excel library)
        Ok("Excel format not implemented".to_string())
    }

    async fn convert_to_pdf(&self, _data: &DashboardExportData) -> Result<String> {
        // Simplified PDF conversion (would use actual PDF library)
        Ok("PDF format not implemented".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboardConfig {
    pub user_preferences: UserPreferences,
    pub available_widgets: Vec<WidgetConfig>,
    pub dashboard_capabilities: DashboardCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCapabilities {
    pub supports_real_time: bool,
    pub supports_historical: bool,
    pub supports_alerts: bool,
    pub supports_custom_widgets: bool,
    pub max_widgets: u32,
    pub max_custom_queries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardExportData {
    pub export_timestamp: DateTime<Utc>,
    pub time_range: Duration,
    pub current_metrics: SystemMetrics,
    pub historical_snapshots: Vec<HistoricalSnapshot>,
    pub aggregated_metrics: AggregatedMetrics,
    pub alert_statistics: AlertStatistics,
    pub performance_summaries: Vec<PerformanceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    CSV,
    Excel,
    PDF,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub data: String,
    pub filename: String,
    pub size_bytes: usize,
    pub generated_at: DateTime<Utc>,
}

impl ExportFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            ExportFormat::JSON => "json",
            ExportFormat::CSV => "csv",
            ExportFormat::Excel => "xlsx",
            ExportFormat::PDF => "pdf",
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Warning => write!(f, "Warning"),
            HealthStatus::Critical => write!(f, "Critical"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
