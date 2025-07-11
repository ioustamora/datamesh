/// Comprehensive Audit Logging System
///
/// This module implements enterprise-grade audit logging for all key operations
/// and security events in the DataMesh system. It provides comprehensive
/// logging, anomaly detection, and compliance monitoring capabilities.
use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Types of audit events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuditEventType {
    KeyGeneration,
    KeyAccess,
    KeyRotation,
    KeyDeletion,
    EncryptionOperation,
    DecryptionOperation,
    AuthenticationAttempt,
    PolicyViolation,
    FileUpload,
    FileDownload,
    FileDelete,
    ConfigChange,
    SystemStart,
    SystemStop,
    NetworkEvent,
    SecurityAlert,
}

/// Operation result for audit logging
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationResult {
    Success,
    Failure { error: String },
    Warning { message: String },
}

/// Client information for audit context
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub user_id: String,
    pub process_id: u32,
    pub hostname: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Comprehensive audit event structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub resource_id: String,
    pub operation: String,
    pub result: OperationResult,
    pub client_info: ClientInfo,
    pub metadata: serde_json::Value,
    pub severity: EventSeverity,
    pub session_id: Option<String>,
}

/// Event severity levels
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit query structure for searching logs
#[derive(Debug, Clone)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub user_id: Option<String>,
    pub resource_id: Option<String>,
    pub severity: Option<EventSeverity>,
    pub limit: Option<usize>,
}

/// Audit logger trait for different storage backends
pub trait AuditLogger: Send + Sync {
    fn log_operation(&self, event: AuditEvent) -> Result<()>;
    fn query_logs(&self, query: AuditQuery) -> Result<Vec<AuditEvent>>;
    fn get_recent_events(&self, limit: usize) -> Result<Vec<AuditEvent>>;
    fn get_events_by_user(&self, user_id: &str, limit: usize) -> Result<Vec<AuditEvent>>;
    fn get_security_alerts(&self, limit: usize) -> Result<Vec<AuditEvent>>;
}

/// File-based audit logger implementation
pub struct FileAuditLogger {
    log_file_path: std::path::PathBuf,
    events_cache: Arc<Mutex<Vec<AuditEvent>>>,
    max_cache_size: usize,
}

impl FileAuditLogger {
    pub fn new(log_file_path: std::path::PathBuf) -> Result<Self> {
        // Ensure log directory exists
        if let Some(parent) = log_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Set secure file permissions
        let logger = Self {
            log_file_path,
            events_cache: Arc::new(Mutex::new(Vec::new())),
            max_cache_size: 10000, // Keep last 10k events in memory
        };

        logger.ensure_log_file_permissions()?;

        Ok(logger)
    }

    fn ensure_log_file_permissions(&self) -> Result<()> {
        // Create file if it doesn't exist
        if !self.log_file_path.exists() {
            fs::File::create(&self.log_file_path)?;
        }

        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.log_file_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.log_file_path, perms)?;
        }

        Ok(())
    }

    fn add_to_cache(&self, event: AuditEvent) {
        let mut cache = self.events_cache.lock().unwrap();
        cache.push(event);

        // Trim cache if too large
        if cache.len() > self.max_cache_size {
            let trim_count = cache.len() - self.max_cache_size;
            cache.drain(0..trim_count);
        }
    }

    fn search_cache(&self, query: &AuditQuery) -> Vec<AuditEvent> {
        let cache = self.events_cache.lock().unwrap();

        cache
            .iter()
            .filter(|event| {
                // Filter by time range
                if let Some(start) = query.start_time {
                    if event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if event.timestamp > end {
                        return false;
                    }
                }

                // Filter by event types
                if let Some(ref types) = query.event_types {
                    if !types.iter().any(|t| {
                        std::mem::discriminant(t) == std::mem::discriminant(&event.event_type)
                    }) {
                        return false;
                    }
                }

                // Filter by user ID
                if let Some(ref user_id) = query.user_id {
                    if event.user_id != *user_id {
                        return false;
                    }
                }

                // Filter by resource ID
                if let Some(ref resource_id) = query.resource_id {
                    if event.resource_id != *resource_id {
                        return false;
                    }
                }

                // Filter by severity
                if let Some(ref severity) = query.severity {
                    if event.severity != *severity {
                        return false;
                    }
                }

                true
            })
            .take(query.limit.unwrap_or(100))
            .cloned()
            .collect()
    }
}

impl AuditLogger for FileAuditLogger {
    fn log_operation(&self, event: AuditEvent) -> Result<()> {
        // Add to in-memory cache
        self.add_to_cache(event.clone());

        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        let log_line = format!("{}\n", serde_json::to_string(&event)?);
        file.write_all(log_line.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    fn query_logs(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        // For simple implementation, search in-memory cache
        // In production, this would search the log file or database
        Ok(self.search_cache(&query))
    }

    fn get_recent_events(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        let cache = self.events_cache.lock().unwrap();
        let events = cache.iter().rev().take(limit).cloned().collect();
        Ok(events)
    }

    fn get_events_by_user(&self, user_id: &str, limit: usize) -> Result<Vec<AuditEvent>> {
        let query = AuditQuery {
            start_time: None,
            end_time: None,
            event_types: None,
            user_id: Some(user_id.to_string()),
            resource_id: None,
            severity: None,
            limit: Some(limit),
        };
        self.query_logs(query)
    }

    fn get_security_alerts(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        let query = AuditQuery {
            start_time: None,
            end_time: None,
            event_types: Some(vec![
                AuditEventType::AuthenticationAttempt,
                AuditEventType::PolicyViolation,
                AuditEventType::SecurityAlert,
            ]),
            user_id: None,
            resource_id: None,
            severity: Some(EventSeverity::High),
            limit: Some(limit),
        };
        self.query_logs(query)
    }
}

/// Key operation structure for audit logging
#[derive(Debug, Clone)]
pub struct KeyOperation {
    pub operation_type: AuditEventType,
    pub user_id: String,
    pub key_id: String,
    pub operation_name: String,
    pub result: OperationResult,
    pub client_info: ClientInfo,
    pub duration: Duration,
    pub data_size: Option<u64>,
    pub source_ip: Option<String>,
}

/// Anomaly detection system
pub struct AnomalyDetector {
    baseline_behavior: BaselineBehavior,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct BaselineBehavior {
    pub normal_hours: (u8, u8), // Start and end hour for normal operations
    pub typical_locations: Vec<String>, // Typical IP ranges or locations
    pub average_operations_per_hour: f64,
    pub typical_operation_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_failed_attempts: u32,
    pub max_operations_per_minute: u32,
    pub suspicious_operation_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    UnusualAccess,
    BruteForce,
    OffHoursAccess,
    SuspiciousLocation,
    AnomalousVolume,
    UnusualDuration,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            baseline_behavior: BaselineBehavior {
                normal_hours: (8, 18), // 8 AM to 6 PM
                typical_locations: vec!["192.168.".to_string(), "10.0.".to_string()],
                average_operations_per_hour: 50.0,
                typical_operation_duration: Duration::from_millis(500),
            },
            alert_thresholds: AlertThresholds {
                max_failed_attempts: 5,
                max_operations_per_minute: 100,
                suspicious_operation_duration: Duration::from_secs(30),
            },
        }
    }

    pub fn analyze_event(&self, event: &AuditEvent) -> Result<Option<AlertType>> {
        let current_hour = event.timestamp.hour() as u8;

        // Check for off-hours access
        if current_hour < self.baseline_behavior.normal_hours.0
            || current_hour > self.baseline_behavior.normal_hours.1
        {
            return Ok(Some(AlertType::OffHoursAccess));
        }

        // Check for suspicious location
        if let Some(ref source_ip) = event.client_info.source_ip {
            let is_typical = self
                .baseline_behavior
                .typical_locations
                .iter()
                .any(|range| source_ip.starts_with(range));

            if !is_typical {
                return Ok(Some(AlertType::SuspiciousLocation));
            }
        }

        // Check for failure patterns (would need more context/state)
        if matches!(event.result, OperationResult::Failure { .. }) {
            return Ok(Some(AlertType::UnusualAccess));
        }

        Ok(None)
    }

    pub fn trigger_alert(&self, alert_type: AlertType, event: &AuditEvent) -> Result<()> {
        println!(
            "🚨 Security Alert: {:?} detected for user {} at {}",
            alert_type, event.user_id, event.timestamp
        );

        // In production, this would send alerts to security monitoring systems
        // For now, we'll just log the alert
        Ok(())
    }
}

/// Compliance monitoring system
pub struct ComplianceMonitor {
    compliance_rules: Vec<ComplianceRule>,
}

#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub name: String,
    pub description: String,
    pub required_events: Vec<AuditEventType>,
    pub retention_period: Duration,
    pub encryption_required: bool,
}

impl ComplianceMonitor {
    pub fn new() -> Self {
        Self {
            compliance_rules: vec![
                ComplianceRule {
                    name: "SOX_COMPLIANCE".to_string(),
                    description: "Sarbanes-Oxley Act compliance".to_string(),
                    required_events: vec![
                        AuditEventType::KeyAccess,
                        AuditEventType::KeyDeletion,
                        AuditEventType::FileUpload,
                        AuditEventType::FileDownload,
                    ],
                    retention_period: Duration::from_secs(7 * 365 * 24 * 3600), // 7 years
                    encryption_required: true,
                },
                ComplianceRule {
                    name: "GDPR_COMPLIANCE".to_string(),
                    description: "General Data Protection Regulation compliance".to_string(),
                    required_events: vec![
                        AuditEventType::KeyAccess,
                        AuditEventType::FileDownload,
                        AuditEventType::KeyDeletion,
                    ],
                    retention_period: Duration::from_secs(6 * 365 * 24 * 3600), // 6 years
                    encryption_required: true,
                },
            ],
        }
    }

    pub fn check_compliance(&self, event: &AuditEvent) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        for rule in &self.compliance_rules {
            // Check if event type is required for this rule
            if rule.required_events.iter().any(|required| {
                std::mem::discriminant(required) == std::mem::discriminant(&event.event_type)
            }) {
                // Verify event meets compliance requirements
                if rule.encryption_required && event.metadata.get("encrypted").is_none() {
                    violations.push(format!("Rule {} requires encryption metadata", rule.name));
                }

                // Additional compliance checks would go here
            }
        }

        Ok(violations)
    }
}

/// Main audit system that coordinates all components
pub struct KeyOperationAuditor {
    audit_logger: Box<dyn AuditLogger>,
    anomaly_detector: AnomalyDetector,
    compliance_monitor: ComplianceMonitor,
}

impl KeyOperationAuditor {
    pub fn new(audit_logger: Box<dyn AuditLogger>) -> Self {
        Self {
            audit_logger,
            anomaly_detector: AnomalyDetector::new(),
            compliance_monitor: ComplianceMonitor::new(),
        }
    }

    pub fn log_key_operation(&self, op: KeyOperation) -> Result<()> {
        let audit_event = AuditEvent {
            timestamp: Utc::now(),
            event_type: op.operation_type,
            user_id: op.user_id,
            resource_id: op.key_id,
            operation: op.operation_name,
            result: op.result,
            client_info: op.client_info,
            metadata: json!({
                "duration_ms": op.duration.as_millis(),
                "data_size": op.data_size,
                "source_ip": op.source_ip,
            }),
            severity: EventSeverity::Medium,
            session_id: None,
        };

        // Log the event
        self.audit_logger.log_operation(audit_event.clone())?;

        // Check for anomalies
        if let Some(alert_type) = self.anomaly_detector.analyze_event(&audit_event)? {
            self.anomaly_detector
                .trigger_alert(alert_type, &audit_event)?;
        }

        // Compliance monitoring
        let violations = self.compliance_monitor.check_compliance(&audit_event)?;
        if !violations.is_empty() {
            for violation in violations {
                println!("⚠️  Compliance violation: {}", violation);
            }
        }

        Ok(())
    }

    pub fn get_audit_summary(&self) -> Result<AuditSummary> {
        let recent_events = self.audit_logger.get_recent_events(100)?;
        let security_alerts = self.audit_logger.get_security_alerts(50)?;

        let total_events = recent_events.len();
        let failed_operations = recent_events
            .iter()
            .filter(|e| matches!(e.result, OperationResult::Failure { .. }))
            .count();

        let success_rate = if total_events > 0 {
            ((total_events - failed_operations) as f64 / total_events as f64) * 100.0
        } else {
            100.0
        };

        Ok(AuditSummary {
            total_events,
            failed_operations,
            success_rate,
            security_alerts: security_alerts.len(),
            recent_events,
        })
    }
}

/// Summary of audit activity
#[derive(Debug, Serialize)]
pub struct AuditSummary {
    pub total_events: usize,
    pub failed_operations: usize,
    pub success_rate: f64,
    pub security_alerts: usize,
    pub recent_events: Vec<AuditEvent>,
}

/// Helper function to create default client info
pub fn create_client_info(user_id: String) -> ClientInfo {
    ClientInfo {
        user_id,
        process_id: std::process::id(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        source_ip: None,
        user_agent: Some("DataMesh/1.0".to_string()),
    }
}

/// Get default audit log path
pub fn get_default_audit_log_path() -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let datamesh_dir = config_dir.join("datamesh");
    fs::create_dir_all(&datamesh_dir)?;

    Ok(datamesh_dir.join("audit.log"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_audit_logger() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = FileAuditLogger::new(log_path).unwrap();

        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: AuditEventType::KeyAccess,
            user_id: "test_user".to_string(),
            resource_id: "test_key".to_string(),
            operation: "load_key".to_string(),
            result: OperationResult::Success,
            client_info: create_client_info("test_user".to_string()),
            metadata: json!({"test": "data"}),
            severity: EventSeverity::Low,
            session_id: None,
        };

        logger.log_operation(event.clone()).unwrap();

        let recent = logger.get_recent_events(10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].user_id, "test_user");
    }

    #[test]
    fn test_anomaly_detection() {
        let detector = AnomalyDetector::new();

        // Test off-hours access
        let mut event = AuditEvent {
            timestamp: Utc::now().with_hour(2).unwrap().with_minute(0).unwrap(), // 2 AM
            event_type: AuditEventType::KeyAccess,
            user_id: "test_user".to_string(),
            resource_id: "test_key".to_string(),
            operation: "load_key".to_string(),
            result: OperationResult::Success,
            client_info: create_client_info("test_user".to_string()),
            metadata: json!({}),
            severity: EventSeverity::Low,
            session_id: None,
        };

        let alert = detector.analyze_event(&event).unwrap();
        assert!(matches!(alert, Some(AlertType::OffHoursAccess)));
    }
}
