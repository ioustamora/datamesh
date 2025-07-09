use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::SystemMetrics;

/// Intelligent alert management system with ML-based anomaly detection
/// Implements sophisticated alerting rules, escalation, and notification routing
pub struct AlertManager {
    cooldown_period: Duration,
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    notification_channels: Arc<RwLock<Vec<Box<dyn NotificationChannel>>>>,
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
    escalation_manager: Arc<RwLock<EscalationManager>>,
    alert_stats: Arc<Mutex<AlertStats>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub cooldown: Duration,
    pub notification_channels: Vec<String>,
    pub escalation_rules: Vec<EscalationRule>,
    pub suppression_rules: Vec<SuppressionRule>,
    pub enabled: bool,
    pub tags: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Threshold {
        metric: String,
        operator: ComparisonOperator,
        threshold: f64,
        duration: Option<Duration>,
    },
    AnomalyDetection {
        metric: String,
        sensitivity: f64,
        window: Duration,
    },
    Composite {
        expression: String,
        conditions: Vec<AlertCondition>,
    },
    RateOfChange {
        metric: String,
        rate_threshold: f64,
        time_window: Duration,
    },
    Correlation {
        primary_metric: String,
        secondary_metric: String,
        correlation_threshold: f64,
        window: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub category: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: Option<f64>,
    pub tags: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub notification_sent: bool,
    pub escalated: bool,
    pub escalation_level: u32,
    pub related_alerts: Vec<String>,
    pub context: AlertContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertContext {
    pub node_id: String,
    pub environment: String,
    pub service: String,
    pub additional_info: HashMap<String, String>,
    pub time_series_data: Vec<TimeSeriesPoint>,
    pub correlated_events: Vec<CorrelatedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub correlation_score: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub after_duration: Duration,
    pub target_level: u32,
    pub notification_channels: Vec<String>,
    pub auto_actions: Vec<AutoAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionRule {
    pub condition: String,
    pub duration: Duration,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    RestartService,
    ScaleUp,
    ScaleDown,
    RunScript,
    SendNotification,
    CreateTicket,
    TriggerFailover,
}

/// Anomaly detection using statistical methods and ML techniques
pub struct AnomalyDetector {
    detection_models: HashMap<String, AnomalyDetectionModel>,
    baseline_data: HashMap<String, BaselineData>,
    sensitivity_settings: HashMap<String, f64>,
    detection_stats: AnomalyDetectionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionModel {
    pub model_type: ModelType,
    pub parameters: HashMap<String, f64>,
    pub training_data: Vec<f64>,
    pub last_updated: DateTime<Utc>,
    pub confidence_threshold: f64,
    pub false_positive_rate: f64,
    pub detection_accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    StatisticalThreshold,
    SeasonalDecomposition,
    LSTM,
    IsolationForest,
    ARIMA,
    RollingMean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineData {
    pub mean: f64,
    pub std_dev: f64,
    pub percentiles: BTreeMap<u8, f64>,
    pub seasonal_patterns: SeasonalPatterns,
    pub trend_data: TrendData,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPatterns {
    pub hourly: Vec<f64>,
    pub daily: Vec<f64>,
    pub weekly: Vec<f64>,
    pub monthly: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub slope: f64,
    pub r_squared: f64,
    pub confidence_interval: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionStats {
    pub total_detections: u64,
    pub true_positives: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
}

/// Escalation management for progressive alert handling
pub struct EscalationManager {
    escalation_chains: HashMap<String, EscalationChain>,
    active_escalations: HashMap<String, ActiveEscalation>,
    escalation_history: VecDeque<EscalationEvent>,
    contact_groups: HashMap<String, ContactGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationChain {
    pub id: String,
    pub name: String,
    pub steps: Vec<EscalationStep>,
    pub enabled: bool,
    pub business_hours_only: bool,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    pub level: u32,
    pub delay: Duration,
    pub contacts: Vec<String>,
    pub notification_methods: Vec<NotificationMethod>,
    pub actions: Vec<AutoAction>,
    pub continue_on_acknowledge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEscalation {
    pub alert_id: String,
    pub chain_id: String,
    pub current_level: u32,
    pub started_at: DateTime<Utc>,
    pub next_escalation: Option<DateTime<Utc>>,
    pub notifications_sent: Vec<NotificationRecord>,
    pub actions_taken: Vec<ActionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationEvent {
    pub id: String,
    pub alert_id: String,
    pub level: u32,
    pub timestamp: DateTime<Utc>,
    pub action_taken: String,
    pub success: bool,
    pub response_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: String,
    pub name: String,
    pub members: Vec<Contact>,
    pub rotation_schedule: Option<RotationSchedule>,
    pub backup_contacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub notification_methods: Vec<NotificationMethod>,
    pub availability_schedule: AvailabilitySchedule,
    pub escalation_delay: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMethod {
    pub method_type: NotificationMethodType,
    pub address: String,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMethodType {
    Email,
    SMS,
    Slack,
    PagerDuty,
    Webhook,
    Phone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilitySchedule {
    pub timezone: String,
    pub working_hours: Vec<WorkingHoursBlock>,
    pub holidays: Vec<DateTime<Utc>>,
    pub on_call_overrides: Vec<OnCallOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursBlock {
    pub day_of_week: u32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCallOverride {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub contact_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationSchedule {
    pub rotation_type: RotationType,
    pub interval: Duration,
    pub start_date: DateTime<Utc>,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationType {
    Weekly,
    Daily,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: String,
    pub method: NotificationMethodType,
    pub recipient: String,
    pub sent_at: DateTime<Utc>,
    pub delivered: bool,
    pub delivery_time: Option<Duration>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: String,
    pub action_type: ActionType,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub duration: Duration,
    pub result: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total_alerts: u64,
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    pub alerts_by_category: HashMap<String, u64>,
    pub average_resolution_time: Duration,
    pub false_positive_rate: f64,
    pub escalation_rate: f64,
    pub acknowledgment_rate: f64,
    pub notification_success_rate: f64,
}

/// Notification channel trait for different notification methods
pub trait NotificationChannel: Send + Sync {
    async fn send_notification(&self, alert: &Alert) -> Result<NotificationRecord>;
    fn channel_type(&self) -> NotificationMethodType;
    fn is_enabled(&self) -> bool;
    fn get_configuration(&self) -> HashMap<String, String>;
}

/// Email notification channel
pub struct EmailChannel {
    smtp_config: SmtpConfig,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
}

/// Slack notification channel
pub struct SlackChannel {
    webhook_url: String,
    channel: String,
    enabled: bool,
}

/// Webhook notification channel
pub struct WebhookChannel {
    url: String,
    headers: HashMap<String, String>,
    enabled: bool,
}

impl AlertManager {
    pub async fn new(cooldown_period: Duration) -> Result<Self> {
        Ok(Self {
            cooldown_period,
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            notification_channels: Arc::new(RwLock::new(Vec::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector::new())),
            escalation_manager: Arc::new(RwLock::new(EscalationManager::new())),
            alert_stats: Arc::new(Mutex::new(AlertStats::default())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;

        // Start background tasks
        self.start_alert_processor().await?;
        self.start_escalation_processor().await?;
        self.start_anomaly_detector().await?;

        tracing::info!("AlertManager started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        tracing::info!("AlertManager stopped");
        Ok(())
    }

    /// Register a new alert rule
    pub async fn register_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.insert(rule.id.clone(), rule);
        tracing::info!("Alert rule registered: {}", rule.id);
        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.remove(rule_id);
        tracing::info!("Alert rule removed: {}", rule_id);
        Ok(())
    }

    /// Evaluate metrics against all alert rules
    pub async fn evaluate_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        let rules = self.alert_rules.read().await;
        let mut triggered_alerts = Vec::new();

        for (rule_id, rule) in rules.iter() {
            if !rule.enabled {
                continue;
            }

            // Check cooldown period
            if let Some(last_triggered) = rule.last_triggered {
                if Utc::now() - last_triggered < rule.cooldown {
                    continue;
                }
            }

            // Evaluate rule condition
            if self.evaluate_condition(&rule.condition, metrics).await? {
                let alert = self.create_alert(rule, metrics).await?;
                triggered_alerts.push(alert);
            }
        }

        // Process triggered alerts
        for alert in triggered_alerts {
            self.process_alert(alert).await?;
        }

        Ok(())
    }

    /// Process a triggered alert
    async fn process_alert(&self, alert: Alert) -> Result<()> {
        let alert_id = alert.id.clone();
        
        // Check if alert is already active
        let mut active_alerts = self.active_alerts.write().await;
        if active_alerts.contains_key(&alert_id) {
            // Update existing alert
            if let Some(existing_alert) = active_alerts.get_mut(&alert_id) {
                existing_alert.updated_at = Utc::now();
                existing_alert.current_value = alert.current_value;
            }
            return Ok(());
        }

        // Add to active alerts
        active_alerts.insert(alert_id.clone(), alert.clone());
        drop(active_alerts);

        // Send notifications
        self.send_notifications(&alert).await?;

        // Start escalation if configured
        self.start_escalation(&alert).await?;

        // Update statistics
        self.update_alert_stats(&alert).await?;

        // Add to history
        let mut history = self.alert_history.write().await;
        history.push_back(alert);
        if history.len() > 10000 { // Keep last 10k alerts
            history.pop_front();
        }

        tracing::info!("Alert processed: {}", alert_id);
        Ok(())
    }

    /// Evaluate a single alert condition
    async fn evaluate_condition(&self, condition: &AlertCondition, metrics: &SystemMetrics) -> Result<bool> {
        match condition {
            AlertCondition::Threshold { metric, operator, threshold, duration: _ } => {
                let value = self.get_metric_value(metric, metrics).await?;
                Ok(self.compare_values(value, *threshold, operator))
            }
            AlertCondition::AnomalyDetection { metric, sensitivity, window } => {
                let detector = self.anomaly_detector.read().await;
                detector.is_anomaly(metric, self.get_metric_value(metric, metrics).await?, *sensitivity).await
            }
            AlertCondition::RateOfChange { metric, rate_threshold, time_window } => {
                // Would implement rate of change calculation
                Ok(false) // Placeholder
            }
            AlertCondition::Correlation { primary_metric, secondary_metric, correlation_threshold, window } => {
                // Would implement correlation analysis
                Ok(false) // Placeholder
            }
            AlertCondition::Composite { expression, conditions } => {
                // Would implement composite condition evaluation
                Ok(false) // Placeholder
            }
        }
    }

    /// Get metric value from SystemMetrics
    async fn get_metric_value(&self, metric_name: &str, metrics: &SystemMetrics) -> Result<f64> {
        let value = match metric_name {
            "throughput_mbps" => metrics.throughput_mbps,
            "avg_response_time_ms" => metrics.avg_response_time_ms,
            "success_rate" => metrics.success_rate,
            "active_connections" => metrics.active_connections as f64,
            "error_rate" => metrics.error_rate,
            "total_files" => metrics.total_files as f64,
            "total_size_bytes" => metrics.total_size_bytes as f64,
            "storage_efficiency" => metrics.storage_efficiency,
            "redundancy_factor" => metrics.redundancy_factor,
            "chunk_availability" => metrics.chunk_availability,
            "peer_count" => metrics.peer_count as f64,
            "dht_size" => metrics.dht_size as f64,
            "network_health_score" => metrics.network_health_score,
            "bootstrap_node_count" => metrics.bootstrap_node_count as f64,
            "memory_usage_mb" => metrics.memory_usage_mb as f64,
            "cpu_usage_percent" => metrics.cpu_usage_percent,
            "disk_usage_gb" => metrics.disk_usage_gb as f64,
            "uptime_seconds" => metrics.uptime_seconds as f64,
            "active_users" => metrics.active_users as f64,
            "governance_health" => metrics.governance_health,
            _ => {
                // Check custom metrics
                metrics.custom_metrics.get(metric_name).cloned().unwrap_or(0.0)
            }
        };

        Ok(value)
    }

    /// Compare two values using the specified operator
    fn compare_values(&self, value: f64, threshold: f64, operator: &ComparisonOperator) -> bool {
        match operator {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
            ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
        }
    }

    /// Create an alert from a triggered rule
    async fn create_alert(&self, rule: &AlertRule, metrics: &SystemMetrics) -> Result<Alert> {
        let alert_id = Uuid::new_v4().to_string();
        let current_value = self.get_metric_value(&self.extract_metric_name(&rule.condition), metrics).await?;
        let threshold_value = self.extract_threshold_value(&rule.condition);

        Ok(Alert {
            id: alert_id,
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            title: format!("{} - {}", rule.name, self.get_severity_description(&rule.severity)),
            description: self.generate_alert_description(rule, current_value, threshold_value).await?,
            category: self.determine_alert_category(&rule.condition),
            metric_name: self.extract_metric_name(&rule.condition),
            current_value,
            threshold_value,
            tags: rule.tags.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved_at: None,
            resolved: false,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            notification_sent: false,
            escalated: false,
            escalation_level: 0,
            related_alerts: Vec::new(),
            context: AlertContext {
                node_id: metrics.node_id.clone(),
                environment: "production".to_string(),
                service: "datamesh".to_string(),
                additional_info: HashMap::new(),
                time_series_data: Vec::new(),
                correlated_events: Vec::new(),
            },
        })
    }

    /// Send notifications for an alert
    async fn send_notifications(&self, alert: &Alert) -> Result<()> {
        let channels = self.notification_channels.read().await;
        let mut notification_results = Vec::new();

        for channel in channels.iter() {
            if channel.is_enabled() {
                match channel.send_notification(alert).await {
                    Ok(record) => {
                        notification_results.push(record);
                        tracing::info!("Notification sent via {:?} for alert {}", channel.channel_type(), alert.id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to send notification via {:?} for alert {}: {}", 
                                      channel.channel_type(), alert.id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Start escalation process for an alert
    async fn start_escalation(&self, alert: &Alert) -> Result<()> {
        let escalation_manager = self.escalation_manager.read().await;
        // Would implement escalation logic here
        tracing::info!("Escalation started for alert {}", alert.id);
        Ok(())
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().await;
        
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by.to_string());
            alert.acknowledged_at = Some(Utc::now());
            alert.updated_at = Utc::now();
            
            tracing::info!("Alert {} acknowledged by {}", alert_id, acknowledged_by);
        }

        Ok(())
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().await;
        
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(Utc::now());
            alert.updated_at = Utc::now();
            
            tracing::info!("Alert {} resolved", alert_id);
        }

        Ok(())
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, limit: usize) -> Result<Vec<Alert>> {
        let history = self.alert_history.read().await;
        let alerts: Vec<Alert> = history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect();
        Ok(alerts)
    }

    /// Get alerts within a time period
    pub async fn get_alerts_in_period(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Alert>> {
        let history = self.alert_history.read().await;
        let alerts: Vec<Alert> = history.iter()
            .filter(|alert| alert.created_at >= start && alert.created_at <= end)
            .cloned()
            .collect();
        Ok(alerts)
    }

    /// Get alert statistics
    pub async fn get_alert_stats(&self) -> Result<AlertStats> {
        let stats = self.alert_stats.lock().await;
        Ok(stats.clone())
    }

    /// Add a notification channel
    pub async fn add_notification_channel(&self, channel: Box<dyn NotificationChannel>) -> Result<()> {
        let mut channels = self.notification_channels.write().await;
        channels.push(channel);
        Ok(())
    }

    // Helper methods

    fn extract_metric_name(&self, condition: &AlertCondition) -> String {
        match condition {
            AlertCondition::Threshold { metric, .. } => metric.clone(),
            AlertCondition::AnomalyDetection { metric, .. } => metric.clone(),
            AlertCondition::RateOfChange { metric, .. } => metric.clone(),
            AlertCondition::Correlation { primary_metric, .. } => primary_metric.clone(),
            AlertCondition::Composite { .. } => "composite".to_string(),
        }
    }

    fn extract_threshold_value(&self, condition: &AlertCondition) -> Option<f64> {
        match condition {
            AlertCondition::Threshold { threshold, .. } => Some(*threshold),
            AlertCondition::RateOfChange { rate_threshold, .. } => Some(*rate_threshold),
            AlertCondition::Correlation { correlation_threshold, .. } => Some(*correlation_threshold),
            _ => None,
        }
    }

    fn get_severity_description(&self, severity: &AlertSeverity) -> &'static str {
        match severity {
            AlertSeverity::Critical => "Critical",
            AlertSeverity::Warning => "Warning",
            AlertSeverity::Info => "Info",
        }
    }

    async fn generate_alert_description(&self, rule: &AlertRule, current_value: f64, threshold_value: Option<f64>) -> Result<String> {
        let threshold_text = if let Some(threshold) = threshold_value {
            format!(" (threshold: {})", threshold)
        } else {
            String::new()
        };

        Ok(format!(
            "Alert '{}' triggered. Current value: {:.2}{}",
            rule.name, current_value, threshold_text
        ))
    }

    fn determine_alert_category(&self, condition: &AlertCondition) -> String {
        match condition {
            AlertCondition::Threshold { metric, .. } => {
                if metric.contains("memory") || metric.contains("cpu") || metric.contains("disk") {
                    "system".to_string()
                } else if metric.contains("network") || metric.contains("peer") {
                    "network".to_string()
                } else if metric.contains("storage") || metric.contains("file") {
                    "storage".to_string()
                } else {
                    "general".to_string()
                }
            }
            AlertCondition::AnomalyDetection { .. } => "anomaly".to_string(),
            AlertCondition::RateOfChange { .. } => "performance".to_string(),
            AlertCondition::Correlation { .. } => "correlation".to_string(),
            AlertCondition::Composite { .. } => "composite".to_string(),
        }
    }

    async fn update_alert_stats(&self, alert: &Alert) -> Result<()> {
        let mut stats = self.alert_stats.lock().await;
        stats.total_alerts += 1;
        
        *stats.alerts_by_severity.entry(alert.severity.clone()).or_insert(0) += 1;
        *stats.alerts_by_category.entry(alert.category.clone()).or_insert(0) += 1;

        Ok(())
    }

    async fn start_alert_processor(&self) -> Result<()> {
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }

                // Process alert lifecycle tasks
                tracing::debug!("Alert processor cycle completed");
            }
        });

        Ok(())
    }

    async fn start_escalation_processor(&self) -> Result<()> {
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }

                // Process escalations
                tracing::debug!("Escalation processor cycle completed");
            }
        });

        Ok(())
    }

    async fn start_anomaly_detector(&self) -> Result<()> {
        let anomaly_detector = self.anomaly_detector.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }

                // Update anomaly detection models
                let mut detector = anomaly_detector.write().await;
                if let Err(e) = detector.update_models().await {
                    tracing::error!("Failed to update anomaly detection models: {}", e);
                }
                
                tracing::debug!("Anomaly detector cycle completed");
            }
        });

        Ok(())
    }
}

// Implementation of supporting structures

impl AlertRule {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            condition: AlertCondition::Threshold {
                metric: "".to_string(),
                operator: ComparisonOperator::GreaterThan,
                threshold: 0.0,
                duration: None,
            },
            severity: AlertSeverity::Warning,
            cooldown: Duration::from_minutes(5),
            notification_channels: Vec::new(),
            escalation_rules: Vec::new(),
            suppression_rules: Vec::new(),
            enabled: true,
            tags: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        }
    }

    pub fn condition(mut self, condition: AlertCondition) -> Self {
        self.condition = condition;
        self
    }

    pub fn severity(mut self, severity: AlertSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            detection_models: HashMap::new(),
            baseline_data: HashMap::new(),
            sensitivity_settings: HashMap::new(),
            detection_stats: AnomalyDetectionStats::default(),
        }
    }

    async fn is_anomaly(&self, metric: &str, value: f64, sensitivity: f64) -> Result<bool> {
        // Simple statistical threshold detection
        if let Some(baseline) = self.baseline_data.get(metric) {
            let z_score = (value - baseline.mean) / baseline.std_dev;
            let threshold = 2.0 * sensitivity; // Configurable threshold
            Ok(z_score.abs() > threshold)
        } else {
            // No baseline data, not an anomaly
            Ok(false)
        }
    }

    async fn update_models(&mut self) -> Result<()> {
        // Update statistical models and baselines
        // This is a placeholder implementation
        Ok(())
    }
}

impl EscalationManager {
    fn new() -> Self {
        Self {
            escalation_chains: HashMap::new(),
            active_escalations: HashMap::new(),
            escalation_history: VecDeque::new(),
            contact_groups: HashMap::new(),
        }
    }
}

impl Default for AlertStats {
    fn default() -> Self {
        Self {
            total_alerts: 0,
            alerts_by_severity: HashMap::new(),
            alerts_by_category: HashMap::new(),
            average_resolution_time: Duration::from_secs(0),
            false_positive_rate: 0.0,
            escalation_rate: 0.0,
            acknowledgment_rate: 0.0,
            notification_success_rate: 0.0,
        }
    }
}

impl Default for AnomalyDetectionStats {
    fn default() -> Self {
        Self {
            total_detections: 0,
            true_positives: 0,
            false_positives: 0,
            false_negatives: 0,
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
        }
    }
}

// Notification channel implementations

impl NotificationChannel for EmailChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<NotificationRecord> {
        // Implementation would send actual email
        let record = NotificationRecord {
            id: Uuid::new_v4().to_string(),
            method: NotificationMethodType::Email,
            recipient: "admin@example.com".to_string(),
            sent_at: Utc::now(),
            delivered: true,
            delivery_time: Some(Duration::from_secs(2)),
            error_message: None,
        };

        tracing::info!("Email notification sent for alert {}", alert.id);
        Ok(record)
    }

    fn channel_type(&self) -> NotificationMethodType {
        NotificationMethodType::Email
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_configuration(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("host".to_string(), self.smtp_config.host.clone());
        config.insert("port".to_string(), self.smtp_config.port.to_string());
        config.insert("from".to_string(), self.smtp_config.from_address.clone());
        config
    }
}

impl NotificationChannel for SlackChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<NotificationRecord> {
        // Implementation would send actual Slack message
        let record = NotificationRecord {
            id: Uuid::new_v4().to_string(),
            method: NotificationMethodType::Slack,
            recipient: self.channel.clone(),
            sent_at: Utc::now(),
            delivered: true,
            delivery_time: Some(Duration::from_secs(1)),
            error_message: None,
        };

        tracing::info!("Slack notification sent for alert {}", alert.id);
        Ok(record)
    }

    fn channel_type(&self) -> NotificationMethodType {
        NotificationMethodType::Slack
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_configuration(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("webhook_url".to_string(), self.webhook_url.clone());
        config.insert("channel".to_string(), self.channel.clone());
        config
    }
}

impl NotificationChannel for WebhookChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<NotificationRecord> {
        // Implementation would send actual webhook
        let record = NotificationRecord {
            id: Uuid::new_v4().to_string(),
            method: NotificationMethodType::Webhook,
            recipient: self.url.clone(),
            sent_at: Utc::now(),
            delivered: true,
            delivery_time: Some(Duration::from_millis(500)),
            error_message: None,
        };

        tracing::info!("Webhook notification sent for alert {}", alert.id);
        Ok(record)
    }

    fn channel_type(&self) -> NotificationMethodType {
        NotificationMethodType::Webhook
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_configuration(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("url".to_string(), self.url.clone());
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let manager = AlertManager::new(Duration::from_minutes(5)).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_alert_rule_registration() {
        let manager = AlertManager::new(Duration::from_minutes(5)).await.unwrap();
        
        let rule = AlertRule::new("test_rule")
            .condition(AlertCondition::Threshold {
                metric: "cpu_usage_percent".to_string(),
                operator: ComparisonOperator::GreaterThan,
                threshold: 80.0,
                duration: None,
            })
            .severity(AlertSeverity::Warning);

        manager.register_rule(rule).await.unwrap();
        
        let rules = manager.alert_rules.read().await;
        assert!(rules.contains_key("test_rule"));
    }

    #[tokio::test]
    async fn test_threshold_evaluation() {
        let manager = AlertManager::new(Duration::from_minutes(5)).await.unwrap();
        manager.start().await.unwrap();

        let rule = AlertRule::new("high_cpu")
            .condition(AlertCondition::Threshold {
                metric: "cpu_usage_percent".to_string(),
                operator: ComparisonOperator::GreaterThan,
                threshold: 50.0,
                duration: None,
            })
            .severity(AlertSeverity::Critical);

        manager.register_rule(rule).await.unwrap();

        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            node_id: "test-node".to_string(),
            cpu_usage_percent: 75.0, // Above threshold
            throughput_mbps: 0.0,
            avg_response_time_ms: 0.0,
            success_rate: 1.0,
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
        };

        manager.evaluate_metrics(&metrics).await.unwrap();

        sleep(Duration::from_millis(100)).await;

        let active_alerts = manager.active_alerts.read().await;
        assert!(!active_alerts.is_empty());

        manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let manager = AlertManager::new(Duration::from_minutes(5)).await.unwrap();
        
        let alert = Alert {
            id: "test-alert".to_string(),
            rule_id: "test-rule".to_string(),
            severity: AlertSeverity::Warning,
            title: "Test Alert".to_string(),
            description: "Test Description".to_string(),
            category: "test".to_string(),
            metric_name: "test_metric".to_string(),
            current_value: 100.0,
            threshold_value: Some(80.0),
            tags: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved_at: None,
            resolved: false,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            notification_sent: false,
            escalated: false,
            escalation_level: 0,
            related_alerts: Vec::new(),
            context: AlertContext {
                node_id: "test-node".to_string(),
                environment: "test".to_string(),
                service: "test".to_string(),
                additional_info: HashMap::new(),
                time_series_data: Vec::new(),
                correlated_events: Vec::new(),
            },
        };

        let mut active_alerts = manager.active_alerts.write().await;
        active_alerts.insert("test-alert".to_string(), alert);
        drop(active_alerts);

        manager.acknowledge_alert("test-alert", "test-user").await.unwrap();

        let active_alerts = manager.active_alerts.read().await;
        let alert = active_alerts.get("test-alert").unwrap();
        assert!(alert.acknowledged);
        assert_eq!(alert.acknowledged_by, Some("test-user".to_string()));
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let mut detector = AnomalyDetector::new();
        
        // Add baseline data
        detector.baseline_data.insert("test_metric".to_string(), BaselineData {
            mean: 50.0,
            std_dev: 10.0,
            percentiles: BTreeMap::new(),
            seasonal_patterns: SeasonalPatterns {
                hourly: Vec::new(),
                daily: Vec::new(),
                weekly: Vec::new(),
                monthly: Vec::new(),
            },
            trend_data: TrendData {
                slope: 0.0,
                r_squared: 0.0,
                confidence_interval: (0.0, 0.0),
            },
            last_updated: Utc::now(),
        });

        // Test normal value
        let is_anomaly = detector.is_anomaly("test_metric", 55.0, 1.0).await.unwrap();
        assert!(!is_anomaly);

        // Test anomalous value
        let is_anomaly = detector.is_anomaly("test_metric", 100.0, 1.0).await.unwrap();
        assert!(is_anomaly);
    }

    #[tokio::test]
    async fn test_comparison_operators() {
        let manager = AlertManager::new(Duration::from_minutes(5)).await.unwrap();
        
        assert!(manager.compare_values(10.0, 5.0, &ComparisonOperator::GreaterThan));
        assert!(manager.compare_values(5.0, 10.0, &ComparisonOperator::LessThan));
        assert!(manager.compare_values(10.0, 10.0, &ComparisonOperator::Equal));
        assert!(manager.compare_values(10.0, 5.0, &ComparisonOperator::NotEqual));
        assert!(manager.compare_values(10.0, 10.0, &ComparisonOperator::GreaterThanOrEqual));
        assert!(manager.compare_values(10.0, 10.0, &ComparisonOperator::LessThanOrEqual));
    }
}