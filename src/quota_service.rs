/// Quota Enforcement Service
///
/// This service integrates with the existing file storage system to enforce
/// resource quotas and fair usage policies as defined in the governance roadmap.
/// It provides middleware-like functionality to check quotas before operations.

use crate::governance::{
    UserResourceManager, UserId, UserUsage, QuotaError, UserQuota, PriorityLevel
};
use crate::error::{DfsResult, DfsError};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration as TokioDuration};

/// Service that enforces resource quotas and tracks usage
pub struct QuotaEnforcementService {
    resource_manager: Arc<UserResourceManager>,
    usage_tracker: Arc<RwLock<HashMap<UserId, UserUsage>>>,
    rate_limiter: Arc<RateLimiter>,
    bandwidth_limiter: Arc<BandwidthLimiter>,
    concurrent_ops: Arc<Mutex<HashMap<UserId, u8>>>,
}

/// Rate limiter for API requests
pub struct RateLimiter {
    windows: Arc<RwLock<HashMap<UserId, RateLimitWindow>>>,
}

/// Rate limiting window
#[derive(Debug, Clone)]
struct RateLimitWindow {
    requests: u32,
    window_start: DateTime<Utc>,
    window_duration: Duration,
}

/// Bandwidth limiter for upload/download operations
pub struct BandwidthLimiter {
    active_transfers: Arc<RwLock<HashMap<UserId, TransferState>>>,
}

/// Transfer state for bandwidth limiting
#[derive(Debug, Clone)]
struct TransferState {
    bytes_transferred: u64,
    start_time: DateTime<Utc>,
    max_mbps: f64,
    semaphore: Arc<Semaphore>,
}

/// File operation context for quota checking
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub user_id: UserId,
    pub operation_type: OperationType,
    pub file_size: Option<u64>,
    pub expected_bandwidth: Option<f64>,
}

/// Types of operations that require quota checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Upload,
    Download,
    List,
    Delete,
    Search,
    Metadata,
}

/// Usage statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub user_id: UserId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub storage_used: u64,
    pub bandwidth_used: u64,
    pub api_calls: u32,
    pub files_uploaded: u32,
    pub files_downloaded: u32,
    pub quota_violations: u32,
}

impl QuotaEnforcementService {
    pub fn new(resource_manager: Arc<UserResourceManager>) -> Self {
        Self {
            resource_manager,
            usage_tracker: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::new()),
            bandwidth_limiter: Arc::new(BandwidthLimiter::new()),
            concurrent_ops: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a user can perform an operation
    pub async fn check_operation_allowed(&self, context: &OperationContext) -> Result<(), QuotaError> {
        // Check rate limits
        self.rate_limiter.check_rate_limit(&context.user_id).await?;
        
        // Check concurrent operations
        self.check_concurrent_operations(&context.user_id).await?;
        
        // Check specific operation quotas
        match context.operation_type {
            OperationType::Upload => {
                if let Some(file_size) = context.file_size {
                    self.check_upload_quota(&context.user_id, file_size).await?;
                }
            }
            OperationType::Download => {
                if let Some(file_size) = context.file_size {
                    self.check_download_quota(&context.user_id, file_size).await?;
                }
            }
            _ => {
                // Other operations just need rate limiting
            }
        }
        
        Ok(())
    }

    /// Record that an operation has started
    pub async fn start_operation(&self, context: &OperationContext) -> DfsResult<OperationGuard> {
        // Increment concurrent operations
        {
            let mut concurrent_ops = self.concurrent_ops.lock().unwrap();
            let count = concurrent_ops.entry(context.user_id).or_insert(0);
            *count += 1;
        }

        // For bandwidth-intensive operations, acquire bandwidth allocation
        let bandwidth_guard = if let Some(bandwidth) = context.expected_bandwidth {
            Some(self.bandwidth_limiter.acquire_bandwidth(&context.user_id, bandwidth).await?)
        } else {
            None
        };

        Ok(OperationGuard {
            user_id: context.user_id,
            operation_type: context.operation_type.clone(),
            service: self.clone(),
            bandwidth_guard,
            start_time: Utc::now(),
        })
    }

    /// Check upload quota specifically
    async fn check_upload_quota(&self, user_id: &UserId, file_size: u64) -> Result<(), QuotaError> {
        let quota = self.resource_manager.get_quota(user_id)
            .ok_or_else(|| QuotaError::StorageQuotaExceeded { 
                current: 0, 
                limit: 0, 
                requested: file_size 
            })?;

        let usage = self.get_current_usage(user_id).await;

        // Check storage quota
        if usage.storage_bytes + file_size > quota.max_storage_bytes {
            return Err(QuotaError::StorageQuotaExceeded {
                current: usage.storage_bytes,
                limit: quota.max_storage_bytes,
                requested: file_size,
            });
        }

        // Check file size limit
        if file_size > quota.max_file_size {
            return Err(QuotaError::FileTooLarge {
                size: file_size,
                limit: quota.max_file_size,
            });
        }

        // Check file count limit
        if usage.file_count >= quota.max_files {
            return Err(QuotaError::FileCountExceeded {
                current: usage.file_count,
                limit: quota.max_files,
            });
        }

        Ok(())
    }

    /// Check download quota specifically
    async fn check_download_quota(&self, user_id: &UserId, file_size: u64) -> Result<(), QuotaError> {
        let quota = self.resource_manager.get_quota(user_id)
            .ok_or_else(|| QuotaError::BandwidthQuotaExceeded { 
                current_gb: 0, 
                limit_gb: 0 
            })?;

        let usage = self.get_current_usage(user_id).await;

        // Check bandwidth quota
        let file_size_gb = file_size / (1024 * 1024 * 1024);
        if usage.bandwidth_used_gb + file_size_gb > quota.monthly_transfer_gb {
            return Err(QuotaError::BandwidthQuotaExceeded {
                current_gb: usage.bandwidth_used_gb,
                limit_gb: quota.monthly_transfer_gb,
            });
        }

        Ok(())
    }

    /// Check concurrent operations limit
    async fn check_concurrent_operations(&self, user_id: &UserId) -> Result<(), QuotaError> {
        let quota = self.resource_manager.get_quota(user_id)
            .ok_or_else(|| QuotaError::RateLimitExceeded {
                requests_made: 0,
                limit: 0,
                reset_time: Utc::now(),
            })?;

        let concurrent_ops = self.concurrent_ops.lock().unwrap();
        let current_count = concurrent_ops.get(user_id).copied().unwrap_or(0);

        if current_count >= quota.max_concurrent_operations {
            return Err(QuotaError::RateLimitExceeded {
                requests_made: current_count as u32,
                limit: quota.max_concurrent_operations as u32,
                reset_time: Utc::now() + Duration::minutes(5),
            });
        }

        Ok(())
    }

    /// Get current usage for a user
    async fn get_current_usage(&self, user_id: &UserId) -> UserUsage {
        let usage_tracker = self.usage_tracker.read().unwrap();
        usage_tracker.get(user_id).cloned().unwrap_or_else(|| UserUsage {
            user_id: *user_id,
            ..Default::default()
        })
    }

    /// Update usage statistics
    pub async fn update_usage(&self, user_id: &UserId, operation: &OperationType, bytes_processed: Option<u64>) {
        let mut usage_tracker = self.usage_tracker.write().unwrap();
        let usage = usage_tracker.entry(*user_id).or_insert_with(|| UserUsage {
            user_id: *user_id,
            ..Default::default()
        });

        match operation {
            OperationType::Upload => {
                usage.file_count += 1;
                if let Some(bytes) = bytes_processed {
                    usage.storage_bytes += bytes;
                }
            }
            OperationType::Download => {
                if let Some(bytes) = bytes_processed {
                    usage.bandwidth_used_gb += bytes / (1024 * 1024 * 1024);
                }
            }
            _ => {}
        }

        usage.api_calls_today += 1;
        usage.last_updated = Utc::now();

        // Also update in the resource manager
        self.resource_manager.update_usage(*user_id, usage.clone());
    }

    /// Get usage statistics for a user
    pub async fn get_usage_stats(&self, user_id: &UserId, period_days: u32) -> UsageStats {
        let usage = self.get_current_usage(user_id).await;
        let period_start = Utc::now() - Duration::days(period_days as i64);
        
        UsageStats {
            user_id: *user_id,
            period_start,
            period_end: Utc::now(),
            storage_used: usage.storage_bytes,
            bandwidth_used: usage.bandwidth_used_gb,
            api_calls: usage.api_calls_today,
            files_uploaded: usage.file_count,
            files_downloaded: 0, // This would need to be tracked separately
            quota_violations: 0, // This would need to be tracked
        }
    }

    /// Clean up expired usage data
    pub async fn cleanup_expired_data(&self) {
        let cutoff_time = Utc::now() - Duration::days(30);
        let mut usage_tracker = self.usage_tracker.write().unwrap();
        
        usage_tracker.retain(|_, usage| usage.last_updated > cutoff_time);
    }
}

impl Clone for QuotaEnforcementService {
    fn clone(&self) -> Self {
        Self {
            resource_manager: self.resource_manager.clone(),
            usage_tracker: self.usage_tracker.clone(),
            rate_limiter: self.rate_limiter.clone(),
            bandwidth_limiter: self.bandwidth_limiter.clone(),
            concurrent_ops: self.concurrent_ops.clone(),
        }
    }
}

/// RAII guard for operations
pub struct OperationGuard {
    user_id: UserId,
    operation_type: OperationType,
    service: QuotaEnforcementService,
    bandwidth_guard: Option<BandwidthGuard>,
    start_time: DateTime<Utc>,
}

impl OperationGuard {
    /// Complete the operation successfully
    pub async fn complete(self, bytes_processed: Option<u64>) {
        // Update usage statistics
        self.service.update_usage(&self.user_id, &self.operation_type, bytes_processed).await;
        
        // The Drop impl will handle cleanup
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        // Decrement concurrent operations
        let mut concurrent_ops = self.service.concurrent_ops.lock().unwrap();
        if let Some(count) = concurrent_ops.get_mut(&self.user_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                concurrent_ops.remove(&self.user_id);
            }
        }
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, user_id: &UserId) -> Result<(), QuotaError> {
        let now = Utc::now();
        let mut windows = self.windows.write().unwrap();
        
        let window = windows.entry(*user_id).or_insert_with(|| RateLimitWindow {
            requests: 0,
            window_start: now,
            window_duration: Duration::hours(1),
        });

        // Reset window if expired
        if now - window.window_start > window.window_duration {
            window.requests = 0;
            window.window_start = now;
        }

        // For now, use a simple 1000 requests per hour limit
        // In a real implementation, this would come from user quotas
        if window.requests >= 1000 {
            return Err(QuotaError::RateLimitExceeded {
                requests_made: window.requests,
                limit: 1000,
                reset_time: window.window_start + window.window_duration,
            });
        }

        window.requests += 1;
        Ok(())
    }
}

impl BandwidthLimiter {
    pub fn new() -> Self {
        Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn acquire_bandwidth(&self, user_id: &UserId, mbps: f64) -> DfsResult<BandwidthGuard> {
        let mut transfers = self.active_transfers.write().unwrap();
        
        let transfer_state = transfers.entry(*user_id).or_insert_with(|| TransferState {
            bytes_transferred: 0,
            start_time: Utc::now(),
            max_mbps: mbps,
            semaphore: Arc::new(Semaphore::new(1)),
        });

        // For simplicity, we'll just track the state
        // In a real implementation, this would enforce bandwidth limits
        Ok(BandwidthGuard {
            user_id: *user_id,
            limiter: self.clone(),
        })
    }
}

impl Clone for BandwidthLimiter {
    fn clone(&self) -> Self {
        Self {
            active_transfers: self.active_transfers.clone(),
        }
    }
}

/// Guard for bandwidth allocation
pub struct BandwidthGuard {
    user_id: UserId,
    limiter: BandwidthLimiter,
}

impl Drop for BandwidthGuard {
    fn drop(&mut self) {
        // Clean up bandwidth allocation
        let mut transfers = self.limiter.active_transfers.write().unwrap();
        transfers.remove(&self.user_id);
    }
}

/// Middleware function to enforce quotas on file operations
pub async fn enforce_quota_middleware<F, R>(
    service: &QuotaEnforcementService,
    context: OperationContext,
    operation: F,
) -> Result<R, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<R, Box<dyn std::error::Error>>>,
{
    // Check if operation is allowed
    service.check_operation_allowed(&context).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Start the operation
    let guard = service.start_operation(&context).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Execute the operation
    let result = operation.await;

    // Complete the operation (this will update usage stats)
    match &result {
        Ok(_) => guard.complete(context.file_size).await,
        Err(_) => {
            // Operation failed, still complete the guard but don't update usage
            guard.complete(None).await;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::{UserRegistry, UserResourceManager};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_quota_enforcement() {
        let resource_manager = Arc::new(UserResourceManager::new());
        let service = QuotaEnforcementService::new(resource_manager.clone());
        
        let user_id = Uuid::new_v4();
        let quota = UserQuota::for_free_account(user_id);
        resource_manager.set_quota(user_id, quota);

        // Test upload quota
        let context = OperationContext {
            user_id,
            operation_type: OperationType::Upload,
            file_size: Some(1024 * 1024), // 1MB
            expected_bandwidth: None,
        };

        let result = service.check_operation_allowed(&context).await;
        assert!(result.is_ok());

        // Test file too large
        let context = OperationContext {
            user_id,
            operation_type: OperationType::Upload,
            file_size: Some(200 * 1024 * 1024), // 200MB (exceeds 100MB limit)
            expected_bandwidth: None,
        };

        let result = service.check_operation_allowed(&context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let rate_limiter = RateLimiter::new();
        let user_id = Uuid::new_v4();

        // First request should succeed
        let result = rate_limiter.check_rate_limit(&user_id).await;
        assert!(result.is_ok());

        // Simulate many requests
        for _ in 0..999 {
            let _ = rate_limiter.check_rate_limit(&user_id).await;
        }

        // This should fail (1001st request)
        let result = rate_limiter.check_rate_limit(&user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operation_guard() {
        let resource_manager = Arc::new(UserResourceManager::new());
        let service = QuotaEnforcementService::new(resource_manager.clone());
        
        let user_id = Uuid::new_v4();
        let quota = UserQuota::for_free_account(user_id);
        resource_manager.set_quota(user_id, quota);

        let context = OperationContext {
            user_id,
            operation_type: OperationType::Upload,
            file_size: Some(1024),
            expected_bandwidth: None,
        };

        // Test that concurrent operations are tracked
        let guard = service.start_operation(&context).await.unwrap();
        
        {
            let concurrent_ops = service.concurrent_ops.lock().unwrap();
            assert_eq!(concurrent_ops.get(&user_id), Some(&1));
        }

        guard.complete(Some(1024)).await;

        // After completion, concurrent operations should be decremented
        {
            let concurrent_ops = service.concurrent_ops.lock().unwrap();
            assert_eq!(concurrent_ops.get(&user_id), Some(&0));
        }
    }
}