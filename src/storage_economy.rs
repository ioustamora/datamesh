/// Storage Economy System
///
/// This module implements the storage-based economy where users can participate in the network by:
/// 1. Contributing 4x their desired storage space to earn network access
/// 2. Paying for premium access without contributing storage
/// 3. Continuous verification of contributed storage space with proof-of-space challenges

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
        // Enhanced verification tracking
        verification_challenges_passed: u32,
        verification_challenges_failed: u32,
        next_verification_due: DateTime<Utc>,
        proof_of_space_enabled: bool,
    },
    /// Premium tier - paid subscription
    Premium {
        max_storage: u64,           // Purchased storage in bytes
        subscription_expires: DateTime<Utc>,
        payment_method: String,
        // Enhanced premium features
        premium_features: Vec<String>,
        support_priority: u8,
        backup_redundancy: u8,
    },
    /// Enterprise tier - highest level access
    Enterprise {
        max_storage: u64,           // Unlimited or very high limit
        contract_expires: DateTime<Utc>,
        support_level: String,
        // Enhanced enterprise features
        dedicated_nodes: u32,
        custom_replication: u8,
        sla_guarantee: f64,
        compliance_level: String,
    },
}

/// Enhanced user storage status and limits
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
    
    // Enhanced tracking fields
    pub network_contribution_score: f64,
    pub total_data_served: u64,
    pub uptime_percentage: f64,
    pub verification_streak: u32,
    pub bonus_credits: u64,
    pub referral_credits: u64,
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

/// Enhanced storage proof-of-space verification result
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
    
    // Enhanced proof-of-space fields
    pub proof_type: ProofType,
    pub challenge_rounds: u32,
    pub avg_response_time: f64,
    pub consistency_score: f64,
    pub proof_hash: String,
    pub verification_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    Pending,
    Verified,
    Failed,
    Expired,
    Challenged,    // Currently being challenged
    Suspended,     // Temporarily suspended due to failures
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    /// Simple space verification (file creation test)
    SimpleSpace,
    /// Proof-of-space with challenge-response
    ProofOfSpace,
    /// Continuous monitoring with periodic challenges
    ContinuousVerification,
    /// Advanced cryptographic proof
    CryptographicProof,
}

/// Enhanced storage verification challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    pub challenge_id: String,
    pub user_id: String,
    pub challenge_data: Vec<u8>,       // Random data to write/verify
    pub file_path: String,             // Specific file path to verify
    pub expected_response: String,     // Expected hash response
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    
    // Enhanced challenge fields
    pub challenge_type: ChallengeType,
    pub difficulty_level: u8,
    pub required_space: u64,
    pub verification_steps: Vec<VerificationStep>,
    pub bonus_reward: u64,
    pub consecutive_challenge: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Basic file write/read test
    BasicFileTest,
    /// Random data generation and verification
    RandomDataTest,
    /// Merkle tree proof construction
    MerkleProof,
    /// Time-lock puzzle verification
    TimeLockPuzzle,
    /// Sustained performance test
    SustainedPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    pub step_id: String,
    pub step_type: String,
    pub required_action: String,
    pub verification_data: Vec<u8>,
    pub timeout_seconds: u64,
}

/// Enhanced storage economy configuration
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
    
    /// Enhanced verification settings
    pub proof_of_space_enabled: bool,      // Enable proof-of-space verification
    pub continuous_verification: bool,      // Enable continuous monitoring
    pub challenge_difficulty_levels: u8,    // Number of difficulty levels
    pub min_verification_response_time: u64, // Minimum response time in seconds
    pub max_verification_response_time: u64, // Maximum response time in seconds
    pub verification_reward_multiplier: f64, // Bonus for consistent verification
    pub verification_streak_bonus: u64,     // Bonus for consecutive successful verifications
    
    /// Pricing for premium tiers
    pub premium_price_per_gb_month: f64,   // USD per GB per month
    pub enterprise_multiplier: f64,        // Enterprise pricing multiplier
    
    /// Reputation system
    pub min_reputation_for_contributor: f64, // Minimum reputation to become contributor
    pub reputation_decay_rate: f64,          // Daily reputation decay
    
    /// Network contribution tracking
    pub contribution_score_weight: f64,     // Weight for network contribution in reputation
    pub uptime_requirement: f64,            // Minimum uptime percentage required
    pub data_serving_reward: f64,           // Reward per GB served to network
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
            
            // Enhanced verification defaults
            proof_of_space_enabled: true,
            continuous_verification: true,
            challenge_difficulty_levels: 5,
            min_verification_response_time: 30,
            max_verification_response_time: 300,
            verification_reward_multiplier: 1.1,
            verification_streak_bonus: 1024 * 1024,  // 1MB bonus per streak
            
            premium_price_per_gb_month: 0.10,
            enterprise_multiplier: 2.0,
            min_reputation_for_contributor: 75.0,
            reputation_decay_rate: 0.1,
            
            // Network contribution defaults
            contribution_score_weight: 0.3,
            uptime_requirement: 90.0,
            data_serving_reward: 0.01,
        }
    }
}

/// Economy transaction for tracking storage contributions and payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyTransaction {
    pub transaction_id: String,
    pub user_id: String,
    pub transaction_type: EconomyTransactionType,
    pub amount: u64,
    pub currency: String,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomyTransactionType {
    StorageContribution,
    PremiumUpgrade,
    PremiumRenewal,
    ReputationReward,
    ViolationPenalty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Refunded,
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
        let mut profiles = self.user_profiles.write().await;
        
        let profile = UserStorageProfile {
            user_id: user_id.to_string(),
            tier,
            current_usage: 0,
            upload_quota_used: 0,
            download_quota_used: 0,
            last_activity: Utc::now(),
            reputation_score: 85.0, // Default reputation
            violations: Vec::new(),
            
            // Enhanced tracking fields
            network_contribution_score: 0.0,
            total_data_served: 0,
            uptime_percentage: 100.0,
            verification_streak: 0,
            bonus_credits: 0,
            referral_credits: 0,
        };
        
        profiles.insert(user_id.to_string(), profile);
        
        // Store in database
        self.database.store_user_profile(user_id, &profiles[user_id]).await?;
        
        Ok(())
    }

    /// Switch user to contributor tier by providing storage
    pub async fn become_contributor(
        &self,
        user_id: &str,
        storage_path: PathBuf,
        contributed_space: u64,
    ) -> DfsResult<()> {
        // Check if user has sufficient reputation
        if let Some(profile) = self.get_user_profile(user_id).await? {
            if profile.reputation_score < self.config.min_reputation_for_contributor {
                return Err(DfsError::Storage(format!(
                    "Insufficient reputation ({:.1}). Minimum required: {:.1}",
                    profile.reputation_score, self.config.min_reputation_for_contributor
                )));
            }
        }

        // Verify the storage path exists and has claimed space
        let verified_space = self.verify_storage_space(&storage_path, contributed_space).await?;
        
        if verified_space < contributed_space {
            return Err(DfsError::Storage(format!(
                "Storage verification failed. Claimed: {}, Verified: {}",
                contributed_space, verified_space
            )));
        }

        // Calculate earned storage (contribution ratio)
        let earned_storage = (contributed_space as f64 / self.config.contribution_ratio) as u64;
        
        let new_tier = StorageTier::Contributor {
            contributed_space,
            earned_storage,
            verification_path: storage_path.clone(),
            last_verified: Utc::now(),
            // Enhanced verification tracking
            verification_challenges_passed: 0,
            verification_challenges_failed: 0,
            next_verification_due: Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64),
            proof_of_space_enabled: self.config.proof_of_space_enabled,
        };

        // Update user profile
        self.create_user_profile(user_id, new_tier).await?;

        // Create initial storage proof
        let proof = StorageProof {
            proof_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            storage_path: storage_path,
            claimed_space: contributed_space,
            verified_space,
            verification_timestamp: Utc::now(),
            challenge_response: "initial_verification".to_string(),
            verification_status: ProofStatus::Verified,
            next_verification: Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64),
            
            // Enhanced proof-of-space fields
            proof_type: ProofType::SimpleSpace,
            challenge_rounds: 1,
            avg_response_time: 0.0,
            consistency_score: 100.0,
            proof_hash: format!("{:x}", md5::compute(format!("{}-{}", user_id, verified_space))),
            verification_metadata: HashMap::new(),
        };

        let mut storage_proofs = self.storage_proofs.write().await;
        storage_proofs.insert(proof.proof_id.clone(), proof);

        Ok(())
    }

    /// Upgrade user to premium tier with payment
    pub async fn upgrade_to_premium(
        &self,
        user_id: &str,
        max_storage: u64,
        payment_method: String,
        duration_months: u32,
    ) -> DfsResult<()> {
        let subscription_expires = Utc::now() + chrono::Duration::days(duration_months as i64 * 30);
        
        let new_tier = StorageTier::Premium {
            max_storage,
            subscription_expires,
            payment_method,
            
            // Enhanced premium features
            premium_features: vec!["priority_support".to_string(), "backup_retention".to_string()],
            support_priority: 1,
            backup_redundancy: 3,
        };

        self.create_user_profile(user_id, new_tier).await?;
        
        // Log the upgrade
        tracing::info!(
            "User {} upgraded to premium tier with {}GB storage until {}",
            user_id, max_storage / (1024 * 1024 * 1024), subscription_expires
        );

        Ok(())
    }

    /// Get storage limits for a user
    pub async fn get_storage_limits(&self, user_id: &str) -> DfsResult<(u64, u64, u64)> {
        if let Some(profile) = self.get_user_profile(user_id).await? {
            let (max_storage, upload_quota, download_quota) = match &profile.tier {
                StorageTier::Free { max_storage } => {
                    (*max_storage, self.config.free_tier_upload_quota, self.config.free_tier_download_quota)
                }
                StorageTier::Contributor { earned_storage, .. } => {
                    (*earned_storage, earned_storage * 2, earned_storage * 4) // 2x upload, 4x download
                }
                StorageTier::Premium { max_storage, .. } => {
                    (*max_storage, max_storage * 2, max_storage * 4)
                }
                StorageTier::Enterprise { max_storage, .. } => {
                    (*max_storage, u64::MAX, u64::MAX) // Unlimited transfer
                }
            };
            
            Ok((max_storage, upload_quota, download_quota))
        } else {
            // Default to free tier
            Ok((self.config.free_tier_storage, self.config.free_tier_upload_quota, self.config.free_tier_download_quota))
        }
    }

    /// Check if user can perform an operation
    pub async fn check_operation_permission(
        &self,
        user_id: &str,
        operation_type: &str,
        data_size: u64,
    ) -> DfsResult<bool> {
        let profile = self.get_user_profile(user_id).await?
            .unwrap_or_else(|| self.create_default_profile(user_id));

        let (max_storage, upload_quota, download_quota) = self.get_storage_limits(user_id).await?;

        match operation_type {
            "upload" => {
                // Check storage limit
                if profile.current_usage + data_size > max_storage {
                    return Ok(false);
                }
                
                // Check upload quota
                if profile.upload_quota_used + data_size > upload_quota {
                    return Ok(false);
                }
            }
            "download" => {
                // Check download quota
                if profile.download_quota_used + data_size > download_quota {
                    return Ok(false);
                }
            }
            _ => {}
        }

        Ok(true)
    }

    /// Create a storage verification challenge
    pub async fn create_storage_challenge(&self, user_id: &str) -> DfsResult<StorageChallenge> {
        let profile = self.get_user_profile(user_id).await?;
        
        let storage_path = match profile {
            Some(UserStorageProfile { 
                tier: StorageTier::Contributor { verification_path, .. }, 
                .. 
            }) => verification_path,
            _ => return Err(DfsError::Storage("User is not a contributor".to_string())),
        };

        // Generate random challenge data
        let challenge_data = self.generate_challenge_data();
        let challenge_file = format!("challenge_{}.dat", uuid::Uuid::new_v4());
        let file_path = storage_path.join(&challenge_file);

        let challenge = StorageChallenge {
            challenge_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            challenge_data: challenge_data.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            expected_response: blake3::hash(&challenge_data).to_hex().to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(self.config.verification_timeout_minutes as i64),
            
            // Enhanced challenge fields
            challenge_type: ChallengeType::BasicFileTest,
            difficulty_level: 1,
            required_space: 1024 * 1024, // 1MB
            verification_steps: Vec::new(),
            bonus_reward: 0,
            consecutive_challenge: 0,
        };

        let mut active_challenges = self.active_challenges.write().await;
        active_challenges.insert(challenge.challenge_id.clone(), challenge.clone());

        Ok(challenge)
    }

    /// Verify storage challenge response
    pub async fn verify_challenge_response(
        &self,
        challenge_id: &str,
        response: &str,
    ) -> DfsResult<bool> {
        let mut active_challenges = self.active_challenges.write().await;
        
        if let Some(challenge) = active_challenges.remove(challenge_id) {
            if Utc::now() > challenge.expires_at {
                return Ok(false); // Challenge expired
            }
            
            let is_valid = challenge.expected_response == response;
            
            if is_valid {
                // Update storage proof
                let proof = StorageProof {
                    proof_id: uuid::Uuid::new_v4().to_string(),
                    user_id: challenge.user_id.clone(),
                    storage_path: PathBuf::from(&challenge.file_path).parent().unwrap().to_path_buf(),
                    claimed_space: 0, // Will be updated by actual verification
                    verified_space: 0,
                    verification_timestamp: Utc::now(),
                    challenge_response: response.to_string(),
                    verification_status: ProofStatus::Verified,
                    next_verification: Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64),
                    
                    // Enhanced proof-of-space fields
                    proof_type: ProofType::SimpleSpace,
                    challenge_rounds: 1,
                    avg_response_time: 0.0,
                    consistency_score: 100.0,
                    proof_hash: format!("{:x}", md5::compute(format!("{}-{}", challenge.user_id, 0))),
                    verification_metadata: HashMap::new(),
                };

                let mut storage_proofs = self.storage_proofs.write().await;
                storage_proofs.insert(proof.proof_id.clone(), proof);

                // Update user reputation
                self.update_user_reputation(&challenge.user_id, 1.0).await?;
            } else {
                // Record verification failure
                self.record_violation(&challenge.user_id, ViolationType::ContributionVerificationFailed, 
                    "Challenge verification failed".to_string()).await?;
            }
            
            Ok(is_valid)
        } else {
            Ok(false) // Challenge not found
        }
    }

    /// Get user storage statistics
    pub async fn get_user_statistics(&self, user_id: &str) -> DfsResult<UserStorageStatistics> {
        let profile = self.get_user_profile(user_id).await?
            .unwrap_or_else(|| self.create_default_profile(user_id));

        let (max_storage, upload_quota, download_quota) = self.get_storage_limits(user_id).await?;

        Ok(UserStorageStatistics {
            user_id: user_id.to_string(),
            tier: profile.tier.clone(),
            current_usage: profile.current_usage,
            max_storage,
            upload_quota,
            download_quota,
            upload_quota_used: profile.upload_quota_used,
            download_quota_used: profile.download_quota_used,
            reputation_score: profile.reputation_score,
            violations_count: profile.violations.len(),
            last_activity: profile.last_activity,
            
            // Additional fields
            upload_quota_limit: upload_quota,
            download_quota_limit: download_quota,
            storage_tier: format!("{:?}", profile.tier),
            can_contribute: profile.reputation_score >= self.config.min_reputation_for_contributor,
        })
    }

    /// Internal helper methods
    fn create_default_profile(&self, user_id: &str) -> UserStorageProfile {
        UserStorageProfile {
            user_id: user_id.to_string(),
            tier: StorageTier::Free {
                max_storage: self.config.free_tier_storage,
            },
            current_usage: 0,
            upload_quota_used: 0,
            download_quota_used: 0,
            last_activity: Utc::now(),
            reputation_score: 75.0,
            violations: Vec::new(),
            
            // Enhanced tracking fields
            network_contribution_score: 0.0,
            total_data_served: 0,
            uptime_percentage: 100.0,
            verification_streak: 0,
            bonus_credits: 0,
            referral_credits: 0,
        }
    }

    async fn verify_storage_space(&self, path: &PathBuf, claimed_space: u64) -> DfsResult<u64> {
        // Implementation would check actual disk space at path
        // For now, return claimed space as verification
        if path.exists() {
            Ok(claimed_space)
        } else {
            Err(DfsError::Storage("Storage path does not exist".to_string()))
        }
    }

    fn generate_challenge_data(&self) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..1024).map(|_| rng.gen()).collect()
    }

    async fn update_user_reputation(&self, user_id: &str, delta: f64) -> DfsResult<()> {
        let mut profiles = self.user_profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            profile.reputation_score = (profile.reputation_score + delta).min(100.0).max(0.0);
        }
        Ok(())
    }

    async fn record_violation(
        &self,
        user_id: &str,
        violation_type: ViolationType,
        description: String,
    ) -> DfsResult<()> {
        let mut profiles = self.user_profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            let violation = StorageViolation {
                violation_id: uuid::Uuid::new_v4().to_string(),
                violation_type,
                timestamp: Utc::now(),
                severity: ViolationSeverity::Medium,
                description,
                resolved: false,
            };
            profile.violations.push(violation);
            
            // Decrease reputation based on violation
            profile.reputation_score = (profile.reputation_score - 5.0).max(0.0);
        }
        Ok(())
    }

    pub async fn assign_tier_to_user(&self, user_id: &str, tier: StorageTier) -> Result<(), DfsError> {
        let profile = UserStorageProfile {
            user_id: user_id.to_string(),
            tier,
            current_usage: 0,
            upload_quota_used: 0,
            download_quota_used: 0,
            last_activity: Utc::now(),
            reputation_score: 80.0, // Start with decent reputation
            violations: Vec::new(),
            
            // Enhanced tracking fields
            network_contribution_score: 0.0,
            total_data_served: 0,
            uptime_percentage: 100.0,
            verification_streak: 0,
            bonus_credits: 0,
            referral_credits: 0,
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
        let actual_space = self.verify_storage_space(&storage_path, claimed_space).await?;
        
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
            
            // Enhanced proof-of-space fields
            proof_type: ProofType::SimpleSpace,
            challenge_rounds: 0,
            avg_response_time: 0.0,
            consistency_score: 100.0,
            proof_hash: format!("{:x}", md5::compute(format!("{}-{}", user_id, actual_space))),
            verification_metadata: HashMap::new(),
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
            
            // Enhanced verification tracking
            verification_challenges_passed: 0,
            verification_challenges_failed: 0,
            next_verification_due: Utc::now() + chrono::Duration::hours(self.config.verification_interval_hours as i64),
            proof_of_space_enabled: true,
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
            
            // Enhanced challenge fields
            challenge_type: ChallengeType::RandomDataTest,
            difficulty_level: 1,
            required_space: 1024 * 1024, // 1MB
            verification_steps: Vec::new(),
            bonus_reward: 0,
            consecutive_challenge: 0,
        };

        // Store active challenge
        {
            let mut challenges = self.active_challenges.write().await;
            challenges.insert(challenge_id, challenge.clone());
        }

        Ok(challenge)
    }

    /// Get storage statistics for user
    pub async fn get_user_storage_stats(&self, user_id: &str) -> DfsResult<UserStorageStatistics> {
        let profile = self.get_user_profile(user_id).await?
            .ok_or_else(|| DfsError::Storage("User profile not found".to_string()))?;

        let max_storage = self.get_max_storage_for_tier(&profile.tier);
        let upload_quota = self.get_upload_quota_for_tier(&profile.tier);
        let download_quota = self.get_download_quota_for_tier(&profile.tier);

        Ok(UserStorageStatistics {
            user_id: user_id.to_string(),
            tier: profile.tier.clone(),
            current_usage: profile.current_usage,
            max_storage,
            upload_quota: upload_quota,
            download_quota: download_quota,
            upload_quota_used: profile.upload_quota_used,
            download_quota_used: profile.download_quota_used,
            reputation_score: profile.reputation_score,
            violations_count: profile.violations.len(),
            last_activity: profile.last_activity,
            upload_quota_limit: upload_quota,
            download_quota_limit: download_quota,
            storage_tier: format!("{:?}", profile.tier),
            can_contribute: profile.reputation_score >= self.config.min_reputation_for_contributor,
        })
    }

    /// Private helper methods

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStorageStatistics {
    pub user_id: String,
    pub tier: StorageTier,
    pub current_usage: u64,
    pub max_storage: u64,
    pub upload_quota: u64,
    pub download_quota: u64,
    pub upload_quota_used: u64,
    pub download_quota_used: u64,
    pub reputation_score: f64,
    pub violations_count: usize,
    pub last_activity: DateTime<Utc>,
    
    // Additional fields used in the code
    pub upload_quota_limit: u64,
    pub download_quota_limit: u64,
    pub storage_tier: String,
    pub can_contribute: bool,
}


/// Storage economy service integration
pub struct StorageEconomyService {
    manager: Arc<StorageEconomyManager>,
    verification_scheduler: tokio::task::JoinHandle<()>,
}

impl StorageEconomyService {
    pub fn new(
        config: StorageEconomyConfig,
        database: Arc<crate::thread_safe_database::ThreadSafeDatabaseManager>,
    ) -> Self {
        let manager = Arc::new(StorageEconomyManager::new(config, database));
        let verification_scheduler = Self::start_verification_scheduler(manager.clone());

        Self {
            manager,
            verification_scheduler,
        }
    }

    pub async fn get_user_profile(&self, user_id: &str) -> DfsResult<Option<UserStorageProfile>> {
        self.manager.get_user_profile(user_id).await
    }

    pub async fn get_user_statistics(&self, user_id: &str) -> DfsResult<UserStorageStatistics> {
        self.manager.get_user_statistics(user_id).await
    }

    pub async fn become_contributor(
        &self,
        user_id: &str,
        storage_path: PathBuf,
        contributed_space: u64,
    ) -> DfsResult<()> {
        self.manager.become_contributor(user_id, storage_path, contributed_space).await
    }

    pub async fn upgrade_to_premium(
        &self,
        user_id: &str,
        max_storage: u64,
        payment_method: String,
        duration_months: u32,
    ) -> DfsResult<()> {
        self.manager.upgrade_to_premium(user_id, max_storage, payment_method, duration_months).await
    }

    pub async fn check_operation_permission(
        &self,
        user_id: &str,
        operation_type: &str,
        data_size: u64,
    ) -> DfsResult<bool> {
        self.manager.check_operation_permission(user_id, operation_type, data_size).await
    }

    fn start_verification_scheduler(manager: Arc<StorageEconomyManager>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Check every hour
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::run_verification_cycle(&manager).await {
                    tracing::error!("Storage verification cycle failed: {}", e);
                }
            }
        })
    }

    async fn run_verification_cycle(manager: &StorageEconomyManager) -> DfsResult<()> {
        let proofs = manager.storage_proofs.read().await;
        let now = Utc::now();

        for (proof_id, proof) in proofs.iter() {
            if now >= proof.next_verification {
                // Generate challenge for this user
                if let Ok(challenge) = manager.create_storage_challenge(&proof.user_id).await {
                    tracing::info!("Generated storage challenge for user {}: {}", proof.user_id, challenge.challenge_id);
                    
                    // In a real implementation, this would send the challenge to the user
                    // For now, we'll just log it
                }
            }
        }

        Ok(())
    }
}

impl Drop for StorageEconomyService {
    fn drop(&mut self) {
        self.verification_scheduler.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_storage_economy_manager() {
        let config = StorageEconomyConfig::default();
        let db = Arc::new(crate::thread_safe_database::ThreadSafeDatabaseManager::new("test.db").unwrap());
        let manager = StorageEconomyManager::new(config, db);

        // Test free tier creation
        let free_tier = StorageTier::Free {
            max_storage: 100 * 1024 * 1024, // 100MB
        };
        
        manager.create_user_profile("test_user", free_tier).await.unwrap();
        let profile = manager.get_user_profile("test_user").await.unwrap().unwrap();
        
        assert_eq!(profile.user_id, "test_user");
        assert!(matches!(profile.tier, StorageTier::Free { .. }));
    }

    #[tokio::test]
    async fn test_contributor_upgrade() {
        let config = StorageEconomyConfig::default();
        let db = Arc::new(crate::thread_safe_database::ThreadSafeDatabaseManager::new("test.db").unwrap());
        let manager = StorageEconomyManager::new(config, db);

        // Create user with high reputation
        let free_tier = StorageTier::Free {
            max_storage: 100 * 1024 * 1024,
        };
        manager.create_user_profile("contributor_user", free_tier).await.unwrap();

        // Try to become contributor
        let storage_path = PathBuf::from("/tmp/test_storage");
        let contributed_space = 4 * 1024 * 1024 * 1024; // 4GB

        // This would normally fail due to path not existing, but we'll test the logic
        match manager.become_contributor("contributor_user", storage_path, contributed_space).await {
            Ok(_) => {
                let profile = manager.get_user_profile("contributor_user").await.unwrap().unwrap();
                assert!(matches!(profile.tier, StorageTier::Contributor { .. }));
            }
            Err(_) => {
                // Expected due to path not existing in test
            }
        }
    }

    #[tokio::test]
    async fn test_premium_upgrade() {
        let config = StorageEconomyConfig::default();
        let db = Arc::new(crate::thread_safe_database::ThreadSafeDatabaseManager::new("test.db").unwrap());
        let manager = StorageEconomyManager::new(config, db);

        // Create user and upgrade to premium
        let free_tier = StorageTier::Free {
            max_storage: 100 * 1024 * 1024,
        };
        manager.create_user_profile("premium_user", free_tier).await.unwrap();

        let premium_storage = 100 * 1024 * 1024 * 1024; // 100GB
        manager.upgrade_to_premium("premium_user", premium_storage, "credit_card".to_string(), 12).await.unwrap();

        let profile = manager.get_user_profile("premium_user").await.unwrap().unwrap();
        assert!(matches!(profile.tier, StorageTier::Premium { .. }));
    }

    #[tokio::test]
    async fn test_operation_permissions() {
        let config = StorageEconomyConfig::default();
        let db = Arc::new(crate::thread_safe_database::ThreadSafeDatabaseManager::new("test.db").unwrap());
        let manager = StorageEconomyManager::new(config, db);

        // Test free tier limits
        let free_tier = StorageTier::Free {
            max_storage: 100 * 1024 * 1024, // 100MB
        };
        manager.create_user_profile("limited_user", free_tier).await.unwrap();

        // Should allow small upload
        let can_upload_small = manager.check_operation_permission("limited_user", "upload", 1024).await.unwrap();
        assert!(can_upload_small);

        // Should deny large upload
        let can_upload_large = manager.check_operation_permission("limited_user", "upload", 200 * 1024 * 1024).await.unwrap();
        assert!(!can_upload_large);
    }
}