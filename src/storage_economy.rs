/// Storage Economy System
///
/// This module implements the storage-based economy where users can participate in the network by:
/// 1. Contributing 4x their desired storage space to earn network access
/// 2. Paying for premium access without contributing storage
/// 3. Continuous verification of contributed storage space

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{DfsError, DfsResult};

/// Storage contribution tier types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageTier {
    /// Free tier with no contribution
    Free {
        max_storage: u64,           // Maximum storage in bytes
    },
    /// Contributor tier - provides storage to earn credits
    Contributor {
        contributed_space: u64,     // Space contributed in bytes
        earned_storage: u64,        // Storage earned (contributed_space / 4)
        verification_path: PathBuf, // Path to contributed storage
        last_verified: DateTime<Utc>,
    },
    /// Premium tier - paid subscription
    Premium {
        max_storage: u64,           // Purchased storage in bytes
        subscription_expires: DateTime<Utc>,
        payment_method: String,
    },
    /// Enterprise tier - highest level access
    Enterprise {
        max_storage: u64,           // Unlimited or very high limit
        contract_expires: DateTime<Utc>,
        support_level: String,
    },
}

/// User storage status and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStorageProfile {
    pub user_id: String,
    pub tier: StorageTier,
    pub current_usage: u64,        // Current storage used in bytes
    pub upload_quota_used: u64,    // Upload quota used this period
    pub download_quota_used: u64,  // Download quota used this period
    pub last_activity: DateTime<Utc>,
    pub reputation_score: f64,     // 0.0 to 100.0
    pub violations: Vec<StorageViolation>,
}

/// Storage violations and infractions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageViolation {
    pub violation_id: String,
    pub violation_type: ViolationType,
    pub timestamp: DateTime<Utc>,
    pub severity: ViolationSeverity,
    pub description: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Storage contribution verification failed
    ContributionVerificationFailed,
    /// Exceeded storage quotas
    QuotaExceeded,
    /// Failed to maintain contribution
    ContributionMaintenance,
    /// Abuse of the system
    SystemAbuse,
    /// Payment failure
    PaymentFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Storage proof-of-space verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    pub proof_id: String,
    pub user_id: String,
    pub storage_path: PathBuf,
    pub claimed_space: u64,
    pub verified_space: u64,
    pub verification_timestamp: DateTime<Utc>,
    pub challenge_response: String,     // Cryptographic proof
    pub verification_status: ProofStatus,
    pub next_verification: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    Pending,
    Verified,
    Failed,
    Expired,
}

/// Storage verification challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    pub challenge_id: String,
    pub user_id: String,
    pub challenge_data: Vec<u8>,       // Random data to write/verify
    pub file_path: String,             // Specific file path to verify
    pub expected_response: String,     // Expected hash response
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Storage economy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEconomyConfig {
    /// Contribution ratio (how much to contribute to earn 1 unit)
    pub contribution_ratio: f64,       // Default: 4.0 (contribute 4GB to earn 1GB)
    
    /// Free tier limits
    pub free_tier_storage: u64,        // Default: 100MB
    pub free_tier_upload_quota: u64,   // Per month
    pub free_tier_download_quota: u64, // Per month
    
    /// Verification settings
    pub verification_interval_hours: u64,  // How often to verify storage
    pub verification_timeout_minutes: u64, // How long user has to respond
    pub max_failed_verifications: u32,     // Before penalization
    
    /// Pricing for premium tiers
    pub premium_price_per_gb_month: f64,   // USD per GB per month
    pub enterprise_multiplier: f64,        // Enterprise pricing multiplier
    
    /// Reputation system
    pub min_reputation_for_contributor: f64, // Minimum reputation to become contributor
    pub reputation_decay_rate: f64,          // Daily reputation decay
}

impl Default for StorageEconomyConfig {
    fn default() -> Self {
        Self {
            contribution_ratio: 4.0,
            free_tier_storage: 100 * 1024 * 1024,        // 100MB
            free_tier_upload_quota: 1024 * 1024 * 1024,  // 1GB per month
            free_tier_download_quota: 2 * 1024 * 1024 * 1024, // 2GB per month
            verification_interval_hours: 24,
            verification_timeout_minutes: 60,
            max_failed_verifications: 3,
            premium_price_per_gb_month: 0.10,
            enterprise_multiplier: 2.0,
            min_reputation_for_contributor: 75.0,
            reputation_decay_rate: 0.1,
        }
    }
}

/// Main storage economy manager
pub struct StorageEconomyManager {
    config: StorageEconomyConfig,
    user_profiles: Arc<RwLock<HashMap<String, UserStorageProfile>>>,
    storage_proofs: Arc<RwLock<HashMap<String, StorageProof>>>,
    active_challenges: Arc<RwLock<HashMap<String, StorageChallenge>>>,
    database: Arc<crate::thread_safe_database::ThreadSafeDatabaseManager>,
}

impl StorageEconomyManager {
    /// Create a new storage economy manager
    pub fn new(
        config: StorageEconomyConfig,
        database: Arc<crate::thread_safe_database::ThreadSafeDatabaseManager>,
    ) -> Self {
        Self {
            config,
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            storage_proofs: Arc::new(RwLock::new(HashMap::new())),
            active_challenges: Arc::new(RwLock::new(HashMap::new())),
            database,
        }
    }

    /// Get user storage profile
    pub async fn get_user_profile(&self, user_id: &str) -> DfsResult<Option<UserStorageProfile>> {
        let profiles = self.user_profiles.read().await;
        Ok(profiles.get(user_id).cloned())
    }

    /// Create or update user storage profile
    pub async fn create_user_profile(&self, user_id: &str, tier: StorageTier) -> DfsResult<()> {
        let profile = UserStorageProfile {
            user_id: user_id.to_string(),
            tier,
            current_usage: 0,
            upload_quota_used: 0,
            download_quota_used: 0,
            last_activity: Utc::now(),
            reputation_score: 80.0, // Start with decent reputation
            violations: Vec::new(),
        };

        let mut profiles = self.user_profiles.write().await;
        profiles.insert(user_id.to_string(), profile);
        
        // Persist to database
        self.save_user_profile_to_db(&profiles[user_id]).await?;
        
        Ok(())
    }

    /// Check if user can upload given amount of data
    pub async fn can_user_upload(&self, user_id: &str, data_size: u64) -> DfsResult<bool> {
        let profile = self.get_user_profile(user_id).await?
            .ok_or_else(|| DfsError::Storage("User profile not found".to_string()))?;

        let max_storage = self.get_max_storage_for_tier(&profile.tier);
        let would_exceed = profile.current_usage + data_size > max_storage;

        if would_exceed {
            return Ok(false);
        }

        // Check monthly upload quota
        let upload_quota = self.get_upload_quota_for_tier(&profile.tier);
        Ok(profile.upload_quota_used + data_size <= upload_quota)
    }

    /// Update user storage usage
    pub async fn update_user_usage(&self, user_id: &str, size_delta: i64, is_upload: bool) -> DfsResult<()> {
        let mut profiles = self.user_profiles.write().await;
        
        if let Some(profile) = profiles.get_mut(user_id) {
            if size_delta > 0 {
                profile.current_usage += size_delta as u64;
                if is_upload {
                    profile.upload_quota_used += size_delta as u64;
                }
            } else {
                profile.current_usage = profile.current_usage.saturating_sub((-size_delta) as u64);
            }
            
            profile.last_activity = Utc::now();
            
            // Save to database
            self.save_user_profile_to_db(profile).await?;
        }

        Ok(())
    }

    /// Initiate storage contribution
    pub async fn initiate_storage_contribution(
        &self,
        user_id: &str,
        storage_path: PathBuf,
        claimed_space: u64,
    ) -> DfsResult<String> {
        // Verify user has minimum reputation
        let profile = self.get_user_profile(user_id).await?
            .ok_or_else(|| DfsError::Storage("User profile not found".to_string()))?;

        if profile.reputation_score < self.config.min_reputation_for_contributor {
            return Err(DfsError::Storage(format!(
                "Minimum reputation {} required for contribution",
                self.config.min_reputation_for_contributor
            )));
        }

        // Verify storage path is accessible and has claimed space
        let actual_space = self.verify_storage_space(&storage_path).await?;
        
        if actual_space < claimed_space {
            return Err(DfsError::Storage(format!(
                "Claimed space {} exceeds actual available space {}",
                claimed_space, actual_space
            )));
        }

        // Create storage proof
        let proof_id = Uuid::new_v4().to_string();
        let proof = StorageProof {
            proof_id: proof_id.clone(),
            user_id: user_id.to_string(),
            storage_path: storage_path.clone(),
            claimed_space,
            verified_space: actual_space,
            verification_timestamp: Utc::now(),
            challenge_response: String::new(),
            verification_status: ProofStatus::Pending,
            next_verification: Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64),
        };

        // Store proof
        {
            let mut proofs = self.storage_proofs.write().await;
            proofs.insert(proof_id.clone(), proof);
        }

        // Update user tier to contributor
        let earned_storage = (claimed_space as f64 / self.config.contribution_ratio) as u64;
        let new_tier = StorageTier::Contributor {
            contributed_space: claimed_space,
            earned_storage,
            verification_path: storage_path,
            last_verified: Utc::now(),
        };

        self.update_user_tier(user_id, new_tier).await?;

        Ok(proof_id)
    }

    /// Generate storage verification challenge
    pub async fn generate_storage_challenge(&self, user_id: &str) -> DfsResult<StorageChallenge> {
        let proof = self.get_user_storage_proof(user_id).await?
            .ok_or_else(|| DfsError::Storage("No storage contribution found".to_string()))?;

        let challenge_id = Uuid::new_v4().to_string();
        let challenge_data: Vec<u8> = (0..1024).map(|_| fastrand::u8(..)).collect();
        
        // Generate unique file path for challenge
        let file_path = format!("datamesh_challenge_{}_{}.tmp", user_id, challenge_id);
        
        // Expected response is hash of challenge data
        let expected_response = blake3::hash(&challenge_data).to_hex().to_string();

        let challenge = StorageChallenge {
            challenge_id: challenge_id.clone(),
            user_id: user_id.to_string(),
            challenge_data,
            file_path,
            expected_response,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(self.config.verification_timeout_minutes as i64),
        };

        // Store active challenge
        {
            let mut challenges = self.active_challenges.write().await;
            challenges.insert(challenge_id, challenge.clone());
        }

        Ok(challenge)
    }

    /// Verify storage challenge response
    pub async fn verify_challenge_response(
        &self,
        challenge_id: &str,
        response_hash: &str,
    ) -> DfsResult<bool> {
        let challenge = {
            let challenges = self.active_challenges.read().await;
            challenges.get(challenge_id).cloned()
        };

        let challenge = challenge
            .ok_or_else(|| DfsError::Storage("Challenge not found".to_string()))?;

        // Check if challenge has expired
        if Utc::now() > challenge.expires_at {
            self.handle_failed_verification(&challenge.user_id, "Challenge expired").await?;
            return Ok(false);
        }

        // Verify response
        let is_valid = response_hash == challenge.expected_response;

        if is_valid {
            self.handle_successful_verification(&challenge.user_id).await?;
        } else {
            self.handle_failed_verification(&challenge.user_id, "Invalid challenge response").await?;
        }

        // Remove challenge from active challenges
        {
            let mut challenges = self.active_challenges.write().await;
            challenges.remove(challenge_id);
        }

        Ok(is_valid)
    }

    /// Get storage statistics for user
    pub async fn get_user_storage_stats(&self, user_id: &str) -> DfsResult<UserStorageStats> {
        let profile = self.get_user_profile(user_id).await?
            .ok_or_else(|| DfsError::Storage("User profile not found".to_string()))?;

        let max_storage = self.get_max_storage_for_tier(&profile.tier);
        let upload_quota = self.get_upload_quota_for_tier(&profile.tier);
        let download_quota = self.get_download_quota_for_tier(&profile.tier);

        Ok(UserStorageStats {
            current_usage: profile.current_usage,
            max_storage,
            upload_quota_used: profile.upload_quota_used,
            upload_quota_limit: upload_quota,
            download_quota_used: profile.download_quota_used,
            download_quota_limit: download_quota,
            storage_tier: format!("{:?}", profile.tier),
            reputation_score: profile.reputation_score,
            violations_count: profile.violations.len(),
            can_contribute: profile.reputation_score >= self.config.min_reputation_for_contributor,
        })
    }

    /// Private helper methods
    
    async fn verify_storage_space(&self, path: &PathBuf) -> DfsResult<u64> {
        use std::fs;
        
        let metadata = fs::metadata(path)
            .map_err(|e| DfsError::Io(format!("Cannot access storage path: {}", e)))?;

        if !metadata.is_dir() {
            return Err(DfsError::Io("Storage path must be a directory".to_string()));
        }

        // Get available space using platform-specific methods
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;
            
            let path_c = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|_| DfsError::Io("Invalid path".to_string()))?;
            
            let mut statvfs: libc::statvfs = unsafe { mem::zeroed() };
            let result = unsafe { libc::statvfs(path_c.as_ptr(), &mut statvfs) };
            
            if result == 0 {
                let available_bytes = statvfs.f_bavail * statvfs.f_frsize;
                Ok(available_bytes)
            } else {
                Err(DfsError::Io("Failed to get filesystem statistics".to_string()))
            }
        }
        
        #[cfg(windows)]
        {
            // Windows implementation would go here
            // For now, return a placeholder
            Ok(1024 * 1024 * 1024) // 1GB placeholder
        }
    }

    async fn get_user_storage_proof(&self, user_id: &str) -> DfsResult<Option<StorageProof>> {
        let proofs = self.storage_proofs.read().await;
        Ok(proofs.values().find(|p| p.user_id == user_id).cloned())
    }

    async fn handle_successful_verification(&self, user_id: &str) -> DfsResult<()> {
        // Update proof status
        {
            let mut proofs = self.storage_proofs.write().await;
            if let Some(proof) = proofs.values_mut().find(|p| p.user_id == user_id) {
                proof.verification_status = ProofStatus::Verified;
                proof.verification_timestamp = Utc::now();
                proof.next_verification = Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64);
            }
        }

        // Update user reputation (small boost for successful verification)
        self.update_user_reputation(user_id, 1.0).await?;

        Ok(())
    }

    async fn handle_failed_verification(&self, user_id: &str, reason: &str) -> DfsResult<()> {
        // Record violation
        let violation = StorageViolation {
            violation_id: Uuid::new_v4().to_string(),
            violation_type: ViolationType::ContributionVerificationFailed,
            timestamp: Utc::now(),
            severity: ViolationSeverity::Medium,
            description: reason.to_string(),
            resolved: false,
        };

        // Add violation to user profile
        {
            let mut profiles = self.user_profiles.write().await;
            if let Some(profile) = profiles.get_mut(user_id) {
                profile.violations.push(violation);
                
                // Reduce reputation
                profile.reputation_score = (profile.reputation_score - 5.0).max(0.0);
                
                // Check if user should be downgraded
                if profile.violations.iter()
                    .filter(|v| matches!(v.violation_type, ViolationType::ContributionVerificationFailed))
                    .count() >= self.config.max_failed_verifications as usize
                {
                    // Downgrade to free tier
                    profile.tier = StorageTier::Free {
                        max_storage: self.config.free_tier_storage,
                    };
                }
            }
        }

        Ok(())
    }

    async fn update_user_tier(&self, user_id: &str, new_tier: StorageTier) -> DfsResult<()> {
        let mut profiles = self.user_profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            profile.tier = new_tier;
            self.save_user_profile_to_db(profile).await?;
        }
        Ok(())
    }

    async fn update_user_reputation(&self, user_id: &str, delta: f64) -> DfsResult<()> {
        let mut profiles = self.user_profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            profile.reputation_score = (profile.reputation_score + delta).clamp(0.0, 100.0);
            self.save_user_profile_to_db(profile).await?;
        }
        Ok(())
    }

    async fn save_user_profile_to_db(&self, profile: &UserStorageProfile) -> DfsResult<()> {
        // Save to database - implement based on your database schema
        // For now, this is a placeholder
        Ok(())
    }

    fn get_max_storage_for_tier(&self, tier: &StorageTier) -> u64 {
        match tier {
            StorageTier::Free { max_storage } => *max_storage,
            StorageTier::Contributor { earned_storage, .. } => *earned_storage,
            StorageTier::Premium { max_storage, .. } => *max_storage,
            StorageTier::Enterprise { max_storage, .. } => *max_storage,
        }
    }

    fn get_upload_quota_for_tier(&self, tier: &StorageTier) -> u64 {
        match tier {
            StorageTier::Free { .. } => self.config.free_tier_upload_quota,
            StorageTier::Contributor { earned_storage, .. } => earned_storage * 2, // 2x upload quota
            StorageTier::Premium { max_storage, .. } => max_storage * 3, // 3x upload quota
            StorageTier::Enterprise { .. } => u64::MAX, // Unlimited
        }
    }

    fn get_download_quota_for_tier(&self, tier: &StorageTier) -> u64 {
        match tier {
            StorageTier::Free { .. } => self.config.free_tier_download_quota,
            StorageTier::Contributor { earned_storage, .. } => earned_storage * 4, // 4x download quota
            StorageTier::Premium { max_storage, .. } => max_storage * 5, // 5x download quota
            StorageTier::Enterprise { .. } => u64::MAX, // Unlimited
        }
    }
}

/// User storage statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct UserStorageStats {
    pub current_usage: u64,
    pub max_storage: u64,
    pub upload_quota_used: u64,
    pub upload_quota_limit: u64,
    pub download_quota_used: u64,
    pub download_quota_limit: u64,
    pub storage_tier: String,
    pub reputation_score: f64,
    pub violations_count: usize,
    pub can_contribute: bool,
}

/// Background storage verification daemon
pub struct StorageVerificationDaemon {
    economy_manager: Arc<StorageEconomyManager>,
    verification_interval: std::time::Duration,
}

impl StorageVerificationDaemon {
    pub fn new(
        economy_manager: Arc<StorageEconomyManager>,
        verification_interval_hours: u64,
    ) -> Self {
        Self {
            economy_manager,
            verification_interval: std::time::Duration::from_secs(verification_interval_hours * 3600),
        }
    }

    /// Start the verification daemon
    pub async fn start(&self) {
        let mut interval = tokio::time::interval(self.verification_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.run_verification_cycle().await {
                tracing::error!("Storage verification cycle failed: {}", e);
            }
        }
    }

    async fn run_verification_cycle(&self) -> DfsResult<()> {
        tracing::info!("Starting storage verification cycle");

        // Get all contributor users that need verification
        let profiles = self.economy_manager.user_profiles.read().await;
        let users_to_verify: Vec<String> = profiles
            .iter()
            .filter_map(|(user_id, profile)| {
                match &profile.tier {
                    StorageTier::Contributor { last_verified, .. } => {
                        let next_verification = *last_verified + chrono::Duration::hours(
                            self.economy_manager.config.verification_interval_hours as i64
                        );
                        
                        if Utc::now() >= next_verification {
                            Some(user_id.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect();
        
        drop(profiles); // Release the lock

        tracing::info!("Found {} users requiring verification", users_to_verify.len());

        // Generate challenges for users
        for user_id in users_to_verify {
            match self.economy_manager.generate_storage_challenge(&user_id).await {
                Ok(challenge) => {
                    tracing::info!("Generated storage challenge for user {}", user_id);
                    // In a real implementation, you would send the challenge to the user
                    // via WebSocket, email, or API notification
                }
                Err(e) => {
                    tracing::error!("Failed to generate challenge for user {}: {}", user_id, e);
                }
            }
        }

        Ok(())
    }
}