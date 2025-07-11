use anyhow::Result;
use serde::{Deserialize, Serialize};
/// Quota Service Module
///
/// This module implements user quota enforcement for concurrent operations
/// as outlined in the DataMesh Application & Network Improvements Roadmap.
/// It provides:
/// - User-based operation quotas and limits
/// - Rate limiting for concurrent operations
/// - Bandwidth quota consumption tracking
/// - Account type-based service differentiation
/// - Fair usage policy enforcement
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// User account types with different service levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    /// Free tier with limited operations
    Free,
    /// Premium tier with higher limits
    Premium,
    /// Enterprise tier with maximum operations
    Enterprise,
}

/// User quota configuration based on account type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuotaConfig {
    /// Account type
    pub account_type: AccountType,
    /// Maximum concurrent chunk operations
    pub max_concurrent_operations: usize,
    /// Maximum bandwidth per hour (in bytes)
    pub max_bandwidth_per_hour: u64,
    /// Maximum storage space (in bytes)
    pub max_storage_space: u64,
    /// Maximum file size (in bytes)
    pub max_file_size: u64,
    /// Rate limit: operations per minute
    pub operations_per_minute: u32,
    /// Priority level for resource allocation
    pub priority_level: u8,
}

impl Default for UserQuotaConfig {
    fn default() -> Self {
        Self::for_account_type(AccountType::Free)
    }
}

impl UserQuotaConfig {
    /// Create quota configuration for specific account type
    pub fn for_account_type(account_type: AccountType) -> Self {
        match account_type {
            AccountType::Free => Self {
                account_type,
                max_concurrent_operations: 2,
                max_bandwidth_per_hour: 100 * 1024 * 1024, // 100MB/hour
                max_storage_space: 1024 * 1024 * 1024,     // 1GB
                max_file_size: 10 * 1024 * 1024,           // 10MB
                operations_per_minute: 10,
                priority_level: 3,
            },
            AccountType::Premium => Self {
                account_type,
                max_concurrent_operations: 8,
                max_bandwidth_per_hour: 1024 * 1024 * 1024, // 1GB/hour
                max_storage_space: 100 * 1024 * 1024 * 1024, // 100GB
                max_file_size: 100 * 1024 * 1024,           // 100MB
                operations_per_minute: 60,
                priority_level: 2,
            },
            AccountType::Enterprise => Self {
                account_type,
                max_concurrent_operations: 20,
                max_bandwidth_per_hour: 10 * 1024 * 1024 * 1024, // 10GB/hour
                max_storage_space: 1024 * 1024 * 1024 * 1024,    // 1TB
                max_file_size: 1024 * 1024 * 1024,               // 1GB
                operations_per_minute: 300,
                priority_level: 1,
            },
        }
    }
}

/// Quota enforcement result
#[derive(Debug, Clone)]
pub enum QuotaResult {
    /// Operation is allowed
    Allowed,
    /// Operation denied due to quota exceeded
    Denied {
        reason: String,
        retry_after: Option<Duration>,
    },
}

/// Current usage tracking for a user
#[derive(Debug, Clone)]
pub struct UserUsage {
    /// Current concurrent operations
    pub current_operations: usize,
    /// Bandwidth used in current hour (in bytes)
    pub bandwidth_used_hour: u64,
    /// Storage space used (in bytes)
    pub storage_used: u64,
    /// Operations performed in current minute
    pub operations_this_minute: u32,
    /// Last operation timestamp
    pub last_operation: Instant,
    /// Bandwidth usage reset time
    pub bandwidth_reset_time: Instant,
    /// Operations count reset time
    pub operations_reset_time: Instant,
}

impl Default for UserUsage {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            current_operations: 0,
            bandwidth_used_hour: 0,
            storage_used: 0,
            operations_this_minute: 0,
            last_operation: now,
            bandwidth_reset_time: now,
            operations_reset_time: now,
        }
    }
}

/// Quota service for managing user resource usage
pub struct QuotaService {
    /// User quota configurations
    user_quotas: Arc<RwLock<HashMap<String, UserQuotaConfig>>>,
    /// Current user usage tracking
    user_usage: Arc<RwLock<HashMap<String, UserUsage>>>,
    /// Enable quota enforcement
    enabled: bool,
}

impl QuotaService {
    /// Create a new quota service
    pub fn new(enabled: bool) -> Self {
        Self {
            user_quotas: Arc::new(RwLock::new(HashMap::new())),
            user_usage: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }

    /// Set user quota configuration
    pub async fn set_user_quota(&self, user_id: &str, quota_config: UserQuotaConfig) {
        let mut quotas = self.user_quotas.write().await;
        quotas.insert(user_id.to_string(), quota_config);

        info!(
            "Updated quota for user {}: {:?}",
            user_id,
            quotas.get(user_id)
        );
    }

    /// Get user quota configuration
    pub async fn get_user_quota(&self, user_id: &str) -> UserQuotaConfig {
        if !self.enabled {
            return UserQuotaConfig::for_account_type(AccountType::Enterprise);
        }

        let quotas = self.user_quotas.read().await;
        quotas.get(user_id).cloned().unwrap_or_default()
    }

    /// Check if user can perform a concurrent operation
    pub async fn can_perform_operation(
        &self,
        user_id: &str,
        operation_type: &str,
        data_size: u64,
    ) -> Result<QuotaResult> {
        if !self.enabled {
            return Ok(QuotaResult::Allowed);
        }

        let quota = self.get_user_quota(user_id).await;
        let mut usage = self.get_user_usage(user_id).await;

        // Update usage counters (reset if time windows have passed)
        self.update_usage_counters(&mut usage).await;

        // Check concurrent operations limit
        if usage.current_operations >= quota.max_concurrent_operations {
            return Ok(QuotaResult::Denied {
                reason: format!(
                    "Maximum concurrent operations exceeded ({}/{})",
                    usage.current_operations, quota.max_concurrent_operations
                ),
                retry_after: Some(Duration::from_secs(30)),
            });
        }

        // Check bandwidth quota
        if usage.bandwidth_used_hour + data_size > quota.max_bandwidth_per_hour {
            let retry_after = self.calculate_bandwidth_reset_time(&usage).await;
            return Ok(QuotaResult::Denied {
                reason: format!(
                    "Bandwidth quota exceeded ({}/{} bytes/hour)",
                    usage.bandwidth_used_hour, quota.max_bandwidth_per_hour
                ),
                retry_after: Some(retry_after),
            });
        }

        // Check rate limiting
        if usage.operations_this_minute >= quota.operations_per_minute {
            let retry_after = self.calculate_rate_limit_reset_time(&usage).await;
            return Ok(QuotaResult::Denied {
                reason: format!(
                    "Rate limit exceeded ({}/{} operations/minute)",
                    usage.operations_this_minute, quota.operations_per_minute
                ),
                retry_after: Some(retry_after),
            });
        }

        // Check storage quota for upload operations
        if operation_type == "upload" && usage.storage_used + data_size > quota.max_storage_space {
            return Ok(QuotaResult::Denied {
                reason: format!(
                    "Storage quota exceeded ({}/{} bytes)",
                    usage.storage_used, quota.max_storage_space
                ),
                retry_after: None,
            });
        }

        // Check file size limit
        if operation_type == "upload" && data_size > quota.max_file_size {
            return Ok(QuotaResult::Denied {
                reason: format!(
                    "File size exceeds limit ({}/{} bytes)",
                    data_size, quota.max_file_size
                ),
                retry_after: None,
            });
        }

        debug!(
            "Operation allowed for user {}: {} ({}B)",
            user_id, operation_type, data_size
        );

        Ok(QuotaResult::Allowed)
    }

    /// Reserve resources for an operation
    pub async fn reserve_operation(
        &self,
        user_id: &str,
        operation_type: &str,
        data_size: u64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut usage_map = self.user_usage.write().await;
        let usage = usage_map.entry(user_id.to_string()).or_default();

        // Reserve concurrent operation slot
        usage.current_operations += 1;

        // Reserve bandwidth
        usage.bandwidth_used_hour += data_size;

        // Increment operation counter
        usage.operations_this_minute += 1;

        // Update storage usage for uploads
        if operation_type == "upload" {
            usage.storage_used += data_size;
        }

        usage.last_operation = Instant::now();

        info!(
            "Reserved resources for user {}: {} concurrent ops, {}B bandwidth, {}B storage",
            user_id, usage.current_operations, usage.bandwidth_used_hour, usage.storage_used
        );

        Ok(())
    }

    /// Release resources after operation completion
    pub async fn release_operation(&self, user_id: &str, _operation_type: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut usage_map = self.user_usage.write().await;
        if let Some(usage) = usage_map.get_mut(user_id) {
            if usage.current_operations > 0 {
                usage.current_operations -= 1;
            }

            debug!(
                "Released operation for user {}: {} concurrent ops remaining",
                user_id, usage.current_operations
            );
        }

        Ok(())
    }

    /// Get current user usage
    pub async fn get_user_usage(&self, user_id: &str) -> UserUsage {
        let usage_map = self.user_usage.read().await;
        usage_map.get(user_id).cloned().unwrap_or_default()
    }

    /// Update usage counters and reset if time windows have passed
    async fn update_usage_counters(&self, usage: &mut UserUsage) {
        let now = Instant::now();

        // Reset bandwidth counter if hour has passed
        if now.duration_since(usage.bandwidth_reset_time) >= Duration::from_secs(3600) {
            usage.bandwidth_used_hour = 0;
            usage.bandwidth_reset_time = now;
        }

        // Reset operations counter if minute has passed
        if now.duration_since(usage.operations_reset_time) >= Duration::from_secs(60) {
            usage.operations_this_minute = 0;
            usage.operations_reset_time = now;
        }
    }

    /// Calculate time until bandwidth quota resets
    async fn calculate_bandwidth_reset_time(&self, usage: &UserUsage) -> Duration {
        let elapsed = usage.bandwidth_reset_time.elapsed();
        if elapsed < Duration::from_secs(3600) {
            Duration::from_secs(3600) - elapsed
        } else {
            Duration::from_secs(0)
        }
    }

    /// Calculate time until rate limit resets
    async fn calculate_rate_limit_reset_time(&self, usage: &UserUsage) -> Duration {
        let elapsed = usage.operations_reset_time.elapsed();
        if elapsed < Duration::from_secs(60) {
            Duration::from_secs(60) - elapsed
        } else {
            Duration::from_secs(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quota_service_creation() {
        let quota_service = QuotaService::new(true);
        assert!(quota_service.enabled);

        let usage = quota_service.get_user_usage("test_user").await;
        assert_eq!(usage.current_operations, 0);
    }

    #[tokio::test]
    async fn test_user_quota_configuration() {
        let quota_service = QuotaService::new(true);

        // Test default quota (Free tier)
        let default_quota = quota_service.get_user_quota("test_user").await;
        assert_eq!(default_quota.account_type, AccountType::Free);
        assert_eq!(default_quota.max_concurrent_operations, 2);

        // Test setting premium quota
        let premium_quota = UserQuotaConfig::for_account_type(AccountType::Premium);
        quota_service
            .set_user_quota("premium_user", premium_quota.clone())
            .await;

        let retrieved_quota = quota_service.get_user_quota("premium_user").await;
        assert_eq!(retrieved_quota.account_type, AccountType::Premium);
        assert_eq!(retrieved_quota.max_concurrent_operations, 8);
    }

    #[tokio::test]
    async fn test_concurrent_operations_limit() {
        let quota_service = QuotaService::new(true);

        // Set free tier quota (2 concurrent operations)
        let free_quota = UserQuotaConfig::for_account_type(AccountType::Free);
        quota_service.set_user_quota("free_user", free_quota).await;

        // First operation should be allowed
        let result = quota_service
            .can_perform_operation("free_user", "download", 1024)
            .await
            .unwrap();
        assert!(matches!(result, QuotaResult::Allowed));

        // Reserve the operation
        quota_service
            .reserve_operation("free_user", "download", 1024)
            .await
            .unwrap();

        // Second operation should be allowed
        let result = quota_service
            .can_perform_operation("free_user", "download", 1024)
            .await
            .unwrap();
        assert!(matches!(result, QuotaResult::Allowed));

        // Reserve the second operation
        quota_service
            .reserve_operation("free_user", "download", 1024)
            .await
            .unwrap();

        // Third operation should be denied
        let result = quota_service
            .can_perform_operation("free_user", "download", 1024)
            .await
            .unwrap();
        assert!(matches!(result, QuotaResult::Denied { .. }));
    }
}
