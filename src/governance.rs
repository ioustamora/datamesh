use crate::error::{DfsError, DfsResult};

/// Governance-specific error type
#[derive(Debug, Clone)]
pub enum GovernanceError {
    UserNotFound,
    InvalidCredentials,
    InsufficientPermissions,
    TokenExpired,
    InvalidToken,
    QuotaExceeded,
    NetworkError(String),
    DatabaseError(String),
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::UserNotFound => write!(f, "User not found"),
            GovernanceError::InvalidCredentials => write!(f, "Invalid credentials"),
            GovernanceError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            GovernanceError::TokenExpired => write!(f, "Token expired"),
            GovernanceError::InvalidToken => write!(f, "Invalid token"),
            GovernanceError::QuotaExceeded => write!(f, "Quota exceeded"),
            GovernanceError::NetworkError(e) => write!(f, "Network error: {}", e),
            GovernanceError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for GovernanceError {}
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
/// Network Governance Module
///
/// This module implements the network governance framework for DataMesh as outlined
/// in the roadmap. It provides user authentication, resource quotas, bootstrap node
/// administration, and democratic governance mechanisms.
///
/// Key Features:
/// - User authentication and account management
/// - Resource quotas and fair usage enforcement
/// - Bootstrap operator management
/// - Network governance through voting
/// - Token-based economic model
/// - Abuse detection and response
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

// ===== User Authentication & Management =====

/// User identification type
pub type UserId = Uuid;

/// JWT claims for user authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,   // Subject (user ID)
    pub email: String, // User email
    pub exp: usize,    // Expiration time
    pub iat: usize,    // Issued at
    pub iss: String,   // Issuer
    pub aud: String,   // Audience
    pub role: String,  // User role (user, admin, operator)
}

/// User account in the DataMesh network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub user_id: UserId,
    pub email: String,
    pub password_hash: String,
    pub public_key: String,
    pub account_type: AccountType,
    pub registration_date: DateTime<Utc>,
    pub verification_status: VerificationStatus,
    pub reputation_score: f64,
    pub abuse_flags: Vec<AbuseFlag>,
    pub subscription: Option<Subscription>,
    pub last_activity: DateTime<Utc>,
}

/// Account types with different resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Free {
        storage_gb: u8,
        bandwidth_gb_month: u16,
        api_calls_hour: u16,
    },
    Premium {
        storage_gb: u16,
        bandwidth_gb_month: u32,
        api_calls_hour: u32,
    },
    Enterprise {
        storage_unlimited: bool,
        bandwidth_unlimited: bool,
        api_calls_unlimited: bool,
        sla_guarantee: f64,
    },
}

/// User verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    EmailVerified,
    IdentityVerified,
}

/// Abuse flag for content moderation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseFlag {
    pub flag_id: Uuid,
    pub flag_type: AbuseType,
    pub reported_by: UserId,
    pub report_date: DateTime<Utc>,
    pub description: String,
    pub status: AbuseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbuseType {
    Spam,
    Malware,
    Copyright,
    Harassment,
    IllegalContent,
    ResourceAbuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbuseStatus {
    Pending,
    Investigating,
    Resolved,
    Dismissed,
}

/// Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub subscription_id: Uuid,
    pub plan_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub auto_renew: bool,
    pub payment_method: String,
}

// ===== Resource Quotas & Fair Usage =====

/// Resource quota for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuota {
    pub user_id: UserId,

    // Storage limits
    pub max_storage_bytes: u64,
    pub max_files: u32,
    pub max_file_size: u64,

    // Bandwidth limits
    pub max_upload_mbps: f64,
    pub max_download_mbps: f64,
    pub monthly_transfer_gb: u64,

    // API limits
    pub max_requests_per_hour: u32,
    pub max_concurrent_operations: u8,

    // Time-based limits
    pub quota_reset_date: DateTime<Utc>,
    pub priority_level: PriorityLevel,
}

/// Priority levels for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Current usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsage {
    pub user_id: UserId,
    pub storage_bytes: u64,
    pub file_count: u32,
    pub bandwidth_used_gb: u64,
    pub api_calls_today: u32,
    pub concurrent_operations: u8,
    pub last_updated: DateTime<Utc>,
}

/// Quota enforcement errors
#[derive(Debug, thiserror::Error)]
pub enum QuotaError {
    #[error("Storage quota exceeded: {current} bytes used, {limit} bytes limit, {requested} bytes requested")]
    StorageQuotaExceeded {
        current: u64,
        limit: u64,
        requested: u64,
    },
    #[error("File too large: {size} bytes, limit is {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },
    #[error("File count exceeded: {current} files, limit is {limit} files")]
    FileCountExceeded { current: u32, limit: u32 },
    #[error("Bandwidth quota exceeded: {current_gb} GB used, {limit_gb} GB limit")]
    BandwidthQuotaExceeded { current_gb: u64, limit_gb: u64 },
    #[error(
        "Rate limit exceeded: {requests_made} requests made, {limit} limit. Reset at {reset_time}"
    )]
    RateLimitExceeded {
        requests_made: u32,
        limit: u32,
        reset_time: DateTime<Utc>,
    },
}

// ===== Bootstrap Node Administration =====

/// Bootstrap node operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapOperator {
    pub operator_id: Uuid,
    pub peer_id: String,
    pub stake: u64,
    pub jurisdiction: String,
    pub governance_weight: f64,
    pub reputation_score: f64,
    pub services: Vec<NetworkService>,
    pub registration_date: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

/// Services provided by bootstrap operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkService {
    Storage,
    Bandwidth,
    BootstrapRelay,
    ContentDelivery,
    Monitoring,
}

// ===== Network Governance Framework =====

/// Network governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProposal {
    pub proposal_id: Uuid,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub author: UserId,
    pub status: ProposalStatus,
    pub votes_for: u64,
    pub votes_against: u64,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    NetworkUpgrade,
    FeeAdjustment,
    QuotaModification,
    OperatorRegistration,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
}

/// Vote on a governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub vote_id: Uuid,
    pub proposal_id: Uuid,
    pub voter_id: UserId,
    pub vote: VoteType,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

// ===== Core Management Systems =====

/// User registry manages all user accounts
pub struct UserRegistry {
    users: Arc<RwLock<HashMap<UserId, UserAccount>>>,
    email_index: Arc<RwLock<HashMap<String, UserId>>>,
}

impl UserRegistry {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_user(
        &self,
        email: String,
        password: String,
        public_key: String,
    ) -> DfsResult<UserAccount> {
        let mut users = self.users.write().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire user registry lock: {}", e))
        })?;
        let mut email_index = self.email_index.write().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire email index lock: {}", e))
        })?;

        // Check if email already exists
        if email_index.contains_key(&email) {
            return Err(crate::error::DfsError::Authentication(
                "Email already registered".to_string(),
            ));
        }

        // Hash password
        let password_hash = self.hash_password(&password)?;

        let user_id = Uuid::new_v4();
        let user_account = UserAccount {
            user_id,
            email: email.clone(),
            password_hash,
            public_key,
            account_type: AccountType::Free {
                storage_gb: 5,
                bandwidth_gb_month: 50,
                api_calls_hour: 100,
            },
            registration_date: Utc::now(),
            verification_status: VerificationStatus::Unverified,
            reputation_score: 0.5,
            abuse_flags: Vec::new(),
            subscription: None,
            last_activity: Utc::now(),
        };

        users.insert(user_id, user_account.clone());
        email_index.insert(email, user_id);

        Ok(user_account)
    }

    /// Authenticate user with email and password
    pub fn authenticate_user(&self, email: &str, password: &str) -> DfsResult<UserAccount> {
        let user = self.get_user_by_email(email)?
            .ok_or_else(|| {
                crate::error::DfsError::Authentication("Invalid credentials".to_string())
            })?;

        if self.verify_password(password, &user.password_hash)? {
            // Update last activity
            let mut updated_user = user.clone();
            updated_user.last_activity = Utc::now();
            self.update_user(updated_user.clone())?;
            Ok(updated_user)
        } else {
            Err(crate::error::DfsError::Authentication(
                "Invalid credentials".to_string(),
            ))
        }
    }

    /// Hash a password using Argon2
    fn hash_password(&self, password: &str) -> DfsResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| {
                crate::error::DfsError::Authentication(format!("Password hashing failed: {}", e))
            })
    }

    /// Verify a password against its hash
    fn verify_password_hash(&self, password: &str, hash: &str) -> DfsResult<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            crate::error::DfsError::Authentication(format!("Invalid password hash: {}", e))
        })?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn get_user(&self, user_id: &UserId) -> DfsResult<Option<UserAccount>> {
        let users = self.users.read().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire user registry read lock: {}", e))
        })?;
        Ok(users.get(user_id).cloned())
    }

    pub fn get_user_by_email(&self, email: &str) -> DfsResult<Option<UserAccount>> {
        let email_index = self.email_index.read().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire email index read lock: {}", e))
        })?;
        if let Some(user_id) = email_index.get(email) {
            self.get_user(user_id)
        } else {
            Ok(None)
        }
    }

    pub fn update_user(&self, user_account: UserAccount) -> DfsResult<()> {
        let mut users = self.users.write().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire user registry write lock: {}", e))
        })?;
        users.insert(user_account.user_id, user_account);
        Ok(())
    }

    pub fn list_users(&self) -> DfsResult<Vec<UserAccount>> {
        let users = self.users.read().map_err(|e| {
            crate::error::DfsError::Authentication(format!("Failed to acquire user registry read lock: {}", e))
        })?;
        Ok(users.values().cloned().collect())
    }

    /// Update user profile
    pub fn update_user_profile(
        &self,
        user_id: &UserId,
        email: String,
        display_name: Option<String>,
    ) -> Result<UserAccount, GovernanceError> {
        let mut users = self.users.write().unwrap();
        
        if let Some(account) = users.get_mut(user_id) {
            account.email = email;
            // In a real implementation, you'd also update display_name field
            Ok(account.clone())
        } else {
            Err(GovernanceError::UserNotFound)
        }
    }

    /// Verify user password
    pub fn verify_password(&self, user_id: &UserId, password: &str) -> Result<bool, GovernanceError> {
        let users = self.users.read().unwrap();
        
        if let Some(_account) = users.get(user_id) {
            // In a real implementation, you'd hash the password and compare
            // For now, just return true for demo purposes
            Ok(true)
        } else {
            Err(GovernanceError::UserNotFound)
        }
    }

    /// Update user password
    pub fn update_password(&self, user_id: &UserId, new_password: &str) -> Result<(), GovernanceError> {
        let mut users = self.users.write().unwrap();
        
        if let Some(_account) = users.get_mut(user_id) {
            // In a real implementation, you'd hash and store the password
            // For now, just return success for demo purposes
            Ok(())
        } else {
            Err(GovernanceError::UserNotFound)
        }
    }
}

/// JWT authentication service
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    jwt_config: crate::api_server::JwtConfig,
}

impl AuthService {
    pub fn new(jwt_config: &crate::api_server::JwtConfig) -> DfsResult<Self> {
        // Validate secret strength (minimum 32 bytes for HS256)
        if jwt_config.secret.len() < 32 {
            return Err(DfsError::Authentication(format!(
                "JWT secret must be at least 32 characters long for security. Current length: {}",
                jwt_config.secret.len()
            )));
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = jwt_config.leeway_seconds;
        validation.set_issuer(&[jwt_config.issuer.clone()]);
        validation.set_audience(&[jwt_config.audience.clone()]);

        Ok(Self {
            encoding_key: EncodingKey::from_secret(jwt_config.secret.as_ref()),
            decoding_key: DecodingKey::from_secret(jwt_config.secret.as_ref()),
            validation,
            jwt_config: jwt_config.clone(),
        })
    }

    /// Legacy constructor for backward compatibility
    pub fn new_with_secret(secret: &str) -> DfsResult<Self> {
        let jwt_config = crate::api_server::JwtConfig {
            secret: secret.to_string(),
            issuer: "datamesh.local".to_string(),
            audience: "datamesh-api".to_string(),
            expiry_hours: 24,
            leeway_seconds: 60,
        };
        Self::new(&jwt_config)
    }

    /// Generate JWT token for authenticated user
    pub fn generate_token(&self, user: &UserAccount) -> DfsResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.jwt_config.expiry_hours as i64);

        let role = match user.account_type {
            AccountType::Free { .. } | AccountType::Premium { .. } => "user",
            AccountType::Enterprise { .. } => "enterprise",
        };

        let claims = AuthClaims {
            sub: user.user_id.to_string(),
            email: user.email.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.jwt_config.issuer.clone(),
            aud: self.jwt_config.audience.clone(),
            role: role.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            crate::error::DfsError::Authentication(format!("Token generation failed: {}", e))
        })
    }

    /// Validate and decode JWT token
    pub fn validate_token(&self, token: &str) -> DfsResult<AuthClaims> {
        decode::<AuthClaims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| {
                crate::error::DfsError::Authentication(format!("Token validation failed: {}", e))
            })
    }

    /// Extract user ID from token
    pub fn get_user_id_from_token(&self, token: &str) -> Result<UserId, GovernanceError> {
        // In a real implementation, you'd decode and validate the JWT
        // For now, just return a mock user ID
        Ok(Uuid::new_v4())
    }

    /// Validate refresh token
    pub fn validate_refresh_token(&self, refresh_token: &str) -> Result<UserId, GovernanceError> {
        // In a real implementation, you'd validate the refresh token
        // For now, just return a mock user ID
        Ok(Uuid::new_v4())
    }

    /// Generate simple access token
    pub fn generate_simple_token(&self, user_id: &UserId) -> Result<String, GovernanceError> {
        // In a real implementation, you'd generate a proper JWT
        // For now, just return a mock token
        Ok(format!("token_{}", user_id.to_string()))
    }

    // ...existing methods...
}

/// Resource manager handles quotas and usage tracking
pub struct UserResourceManager {
    user_quotas: Arc<RwLock<HashMap<UserId, UserQuota>>>,
    usage_tracker: Arc<RwLock<HashMap<UserId, UserUsage>>>,
}

impl UserResourceManager {
    pub fn new() -> Self {
        Self {
            user_quotas: Arc::new(RwLock::new(HashMap::new())),
            usage_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_quota(&self, user_id: UserId, quota: UserQuota) -> DfsResult<()> {
        let mut quotas = self.user_quotas.write().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire quota write lock: {}", e))
        })?;
        quotas.insert(user_id, quota);
        Ok(())
    }

    pub fn get_quota(&self, user_id: &UserId) -> DfsResult<Option<UserQuota>> {
        let quotas = self.user_quotas.read().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire quota read lock: {}", e))
        })?;
        Ok(quotas.get(user_id).cloned())
    }

    pub fn update_usage(&self, user_id: UserId, usage: UserUsage) -> DfsResult<()> {
        let mut usage_tracker = self.usage_tracker.write().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire usage tracker write lock: {}", e))
        })?;
        usage_tracker.insert(user_id, usage);
        Ok(())
    }

    pub fn get_usage(&self, user_id: &UserId) -> DfsResult<Option<UserUsage>> {
        let usage_tracker = self.usage_tracker.read().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire usage tracker read lock: {}", e))
        })?;
        Ok(usage_tracker.get(user_id).cloned())
    }

    pub fn check_storage_quota(
        &self,
        user_id: &UserId,
        additional_bytes: u64,
    ) -> Result<(), QuotaError> {
        let quota = self
            .get_quota(user_id)
            .map_err(|_| QuotaError::StorageQuotaExceeded {
                current: 0,
                limit: 0,
                requested: additional_bytes,
            })?
            .ok_or(QuotaError::StorageQuotaExceeded {
                current: 0,
                limit: 0,
                requested: additional_bytes,
            })?;

        let usage = self.get_usage(user_id)
            .map_err(|_| QuotaError::StorageQuotaExceeded {
                current: 0,
                limit: 0,
                requested: additional_bytes,
            })?
            .unwrap_or_default();

        if usage.storage_bytes + additional_bytes > quota.max_storage_bytes {
            return Err(QuotaError::StorageQuotaExceeded {
                current: usage.storage_bytes,
                limit: quota.max_storage_bytes,
                requested: additional_bytes,
            });
        }

        Ok(())
    }

    pub fn check_rate_limit(&self, user_id: &UserId) -> Result<(), QuotaError> {
        let quota = self
            .get_quota(user_id)
            .map_err(|_| QuotaError::RateLimitExceeded {
                requests_made: 0,
                limit: 0,
                reset_time: Utc::now(),
            })?
            .ok_or(QuotaError::RateLimitExceeded {
                requests_made: 0,
                limit: 0,
                reset_time: Utc::now(),
            })?;

        let usage = self.get_usage(user_id)
            .map_err(|_| QuotaError::RateLimitExceeded {
                requests_made: 0,
                limit: 0,
                reset_time: Utc::now(),
            })?
            .unwrap_or_default();

        if usage.api_calls_today >= quota.max_requests_per_hour {
            return Err(QuotaError::RateLimitExceeded {
                requests_made: usage.api_calls_today,
                limit: quota.max_requests_per_hour,
                reset_time: quota.quota_reset_date,
            });
        }

        Ok(())
    }
}

/// Network governance system
pub struct NetworkGovernance {
    proposals: Arc<RwLock<HashMap<Uuid, NetworkProposal>>>,
    votes: Arc<RwLock<HashMap<Uuid, Vec<Vote>>>>,
    bootstrap_operators: Arc<RwLock<HashMap<Uuid, BootstrapOperator>>>,
}

impl NetworkGovernance {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_operators: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn submit_proposal(&self, proposal: NetworkProposal) -> DfsResult<Uuid> {
        let mut proposals = self.proposals.write().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire proposals write lock: {}", e))
        })?;
        let proposal_id = proposal.proposal_id;
        proposals.insert(proposal_id, proposal);
        Ok(proposal_id)
    }

    pub fn vote_on_proposal(&self, proposal_id: Uuid, vote: Vote) -> DfsResult<()> {
        let mut votes = self.votes.write().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire votes write lock: {}", e))
        })?;
        votes.entry(proposal_id).or_insert_with(Vec::new).push(vote);
        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &Uuid) -> DfsResult<Option<NetworkProposal>> {
        let proposals = self.proposals.read().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire proposals read lock: {}", e))
        })?;
        Ok(proposals.get(proposal_id).cloned())
    }

    pub fn list_proposals(&self) -> DfsResult<Vec<NetworkProposal>> {
        let proposals = self.proposals.read().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire proposals read lock: {}", e))
        })?;
        Ok(proposals.values().cloned().collect())
    }

    pub fn add_bootstrap_operator(&self, operator: BootstrapOperator) -> DfsResult<()> {
        let mut operators = self.bootstrap_operators.write().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire operators write lock: {}", e))
        })?;
        operators.insert(operator.operator_id, operator);
        Ok(())
    }

    pub fn get_bootstrap_operators(&self) -> DfsResult<Vec<BootstrapOperator>> {
        let operators = self.bootstrap_operators.read().map_err(|e| {
            crate::error::DfsError::Storage(format!("Failed to acquire operators read lock: {}", e))
        })?;
        Ok(operators.values().cloned().collect())
    }
}

// ===== Default quota implementations =====

impl UserQuota {
    pub fn for_free_account(user_id: UserId) -> Self {
        Self {
            user_id,
            max_storage_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            max_files: 1000,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_upload_mbps: 10.0,
            max_download_mbps: 50.0,
            monthly_transfer_gb: 50,
            max_requests_per_hour: 100,
            max_concurrent_operations: 5,
            quota_reset_date: Utc::now() + Duration::days(30),
            priority_level: PriorityLevel::Low,
        }
    }

    pub fn for_premium_account(user_id: UserId) -> Self {
        Self {
            user_id,
            max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            max_files: 10000,
            max_file_size: 1024 * 1024 * 1024, // 1GB
            max_upload_mbps: 100.0,
            max_download_mbps: 500.0,
            monthly_transfer_gb: 1000,
            max_requests_per_hour: 10000,
            max_concurrent_operations: 20,
            quota_reset_date: Utc::now() + Duration::days(30),
            priority_level: PriorityLevel::High,
        }
    }

    pub fn for_enterprise_account(user_id: UserId) -> Self {
        Self {
            user_id,
            max_storage_bytes: u64::MAX, // Unlimited
            max_files: u32::MAX,
            max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
            max_upload_mbps: 1000.0,
            max_download_mbps: 5000.0,
            monthly_transfer_gb: u64::MAX,
            max_requests_per_hour: u32::MAX,
            max_concurrent_operations: 100,
            quota_reset_date: Utc::now() + Duration::days(30),
            priority_level: PriorityLevel::Critical,
        }
    }
}

impl Default for UserUsage {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            storage_bytes: 0,
            file_count: 0,
            bandwidth_used_gb: 0,
            api_calls_today: 0,
            concurrent_operations: 0,
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        let registry = UserRegistry::new();
        let result = registry.register_user(
            "test@example.com".to_string(),
            "password123".to_string(),
            "pubkey123".to_string(),
        );
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.public_key, "pubkey123");
        assert!(matches!(user.account_type, AccountType::Free { .. }));
    }

    #[test]
    fn test_duplicate_email_registration() {
        let registry = UserRegistry::new();
        let _ = registry.register_user(
            "test@example.com".to_string(),
            "password123".to_string(),
            "pubkey123".to_string(),
        );

        let result = registry.register_user(
            "test@example.com".to_string(),
            "password456".to_string(),
            "pubkey456".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_quota_enforcement() {
        let manager = UserResourceManager::new();
        let user_id = Uuid::new_v4();

        let quota = UserQuota::for_free_account(user_id);
        manager.set_quota(user_id, quota);

        // Test storage quota
        let result = manager.check_storage_quota(&user_id, 6 * 1024 * 1024 * 1024); // 6GB
        assert!(result.is_err());

        let result = manager.check_storage_quota(&user_id, 1 * 1024 * 1024 * 1024); // 1GB
        assert!(result.is_ok());
    }

    #[test]
    fn test_governance_proposal() {
        let governance = NetworkGovernance::new();
        let proposal = NetworkProposal {
            proposal_id: Uuid::new_v4(),
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposer: Uuid::new_v4(),
            proposal_type: ProposalType::NetworkUpgrade,
            voting_period: Duration::days(14),
            required_quorum: 0.2,
            status: ProposalStatus::Draft,
            votes_for: 0,
            votes_against: 0,
            implementation_timeline: None,
            created_at: Utc::now(),
        };

        let result = governance.submit_proposal(proposal.clone());
        assert!(result.is_ok());

        let retrieved = governance.get_proposal(&proposal.proposal_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Proposal");
    }
}

/// Governance framework wrapper for integration
pub struct GovernanceFramework {
    governance: NetworkGovernance,
}

impl GovernanceFramework {
    pub fn new(
        _database: Arc<crate::database::DatabaseManager>,
        _bootstrap_manager: Arc<crate::bootstrap_manager::BootstrapManager>,
    ) -> Self {
        // Create a simple governance system for now
        Self {
            governance: NetworkGovernance::new(),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // Start governance processes
        Ok(())
    }
}
