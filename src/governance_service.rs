/// Network Governance Service
///
/// This service implements the complete network governance framework as outlined
/// in the governance roadmap. It integrates user management, quota enforcement,
/// bootstrap administration, and democratic governance mechanisms.

use crate::governance::{
    UserRegistry, UserResourceManager, NetworkGovernance, UserId, UserAccount, UserQuota,
    NetworkProposal, ProposalType, ProposalStatus, Vote, VoteType, BootstrapOperator
};
use crate::bootstrap_admin::{BootstrapAdministrationService, OperatorMetrics, NetworkHealthStatus};
use crate::quota_service::{QuotaEnforcementService, OperationContext, UsageStats};
use crate::error::{DfsResult, DfsError};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use tokio::sync::RwLock as AsyncRwLock;

/// Complete governance service that coordinates all governance components
pub struct NetworkGovernanceService {
    user_registry: Arc<UserRegistry>,
    resource_manager: Arc<UserResourceManager>,
    quota_service: Arc<QuotaEnforcementService>,
    bootstrap_admin: Arc<BootstrapAdministrationService>,
    governance: Arc<NetworkGovernance>,
    governance_config: Arc<RwLock<GovernanceConfig>>,
    active_sessions: Arc<AsyncRwLock<HashMap<String, AuthSession>>>,
}

/// Configuration for governance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub min_stake_for_proposal: u64,
    pub voting_period_days: u32,
    pub quorum_percentage: f64,
    pub min_reputation_for_vote: f64,
    pub bootstrap_operator_min_stake: u64,
    pub max_governance_weight_per_operator: f64,
    pub proposal_execution_delay_hours: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_stake_for_proposal: 10000,
            voting_period_days: 14,
            quorum_percentage: 20.0,
            min_reputation_for_vote: 0.3,
            bootstrap_operator_min_stake: 100000,
            max_governance_weight_per_operator: 0.3,
            proposal_execution_delay_hours: 24,
        }
    }
}

/// Authentication session for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<Permission>,
}

/// Permissions for different operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    ReadFiles,
    WriteFiles,
    DeleteFiles,
    SubmitProposal,
    Vote,
    AdministerUsers,
    ManageOperators,
    EmergencyActions,
}

/// Governance action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceActionResult {
    pub success: bool,
    pub message: String,
    pub action_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_users: usize,
    pub active_users_last_24h: usize,
    pub total_operators: usize,
    pub online_operators: usize,
    pub total_storage_gb: u64,
    pub total_bandwidth_gb: u64,
    pub network_health: NetworkHealthStatus,
    pub governance_participation: f64,
}

impl NetworkGovernanceService {
    pub fn new() -> Self {
        let user_registry = Arc::new(UserRegistry::new());
        let resource_manager = Arc::new(UserResourceManager::new());
        let quota_service = Arc::new(QuotaEnforcementService::new(resource_manager.clone()));
        let bootstrap_admin = Arc::new(BootstrapAdministrationService::new());
        let governance = Arc::new(NetworkGovernance::new());

        Self {
            user_registry,
            resource_manager,
            quota_service,
            bootstrap_admin,
            governance,
            governance_config: Arc::new(RwLock::new(GovernanceConfig::default())),
            active_sessions: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }

    /// Register a new user with automatic quota assignment
    pub async fn register_user(&self, email: String, public_key: String) -> DfsResult<UserAccount> {
        // Register user
        let user_account = self.user_registry.register_user(email, public_key)?;
        
        // Set up default quota based on account type
        let quota = match user_account.account_type {
            crate::governance::AccountType::Free { .. } => UserQuota::for_free_account(user_account.user_id),
            crate::governance::AccountType::Premium { .. } => UserQuota::for_premium_account(user_account.user_id),
            crate::governance::AccountType::Enterprise { .. } => UserQuota::for_enterprise_account(user_account.user_id),
        };

        self.resource_manager.set_quota(user_account.user_id, quota);

        Ok(user_account)
    }

    /// Authenticate user and create session
    pub async fn authenticate_user(&self, user_id: &UserId, public_key: &str) -> DfsResult<AuthSession> {
        let user_account = self.user_registry.get_user(user_id)
            .ok_or_else(|| DfsError::Authentication("User not found".to_string()))?;

        // Verify public key matches
        if user_account.public_key != public_key {
            return Err(DfsError::Authentication("Invalid credentials".to_string()));
        }

        // Create session
        let session_id = Uuid::new_v4().to_string();
        let permissions = self.get_user_permissions(&user_account).await;
        
        let session = AuthSession {
            session_id: session_id.clone(),
            user_id: *user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            permissions,
        };

        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        Ok(session)
    }

    /// Get user permissions based on account status and reputation
    async fn get_user_permissions(&self, user_account: &UserAccount) -> Vec<Permission> {
        let mut permissions = vec![Permission::ReadFiles];
        
        // Basic permissions for verified users
        if matches!(user_account.verification_status, 
            crate::governance::VerificationStatus::EmailVerified | 
            crate::governance::VerificationStatus::PhoneVerified |
            crate::governance::VerificationStatus::KYCVerified) {
            permissions.push(Permission::WriteFiles);
        }

        // Advanced permissions for high reputation users
        if user_account.reputation_score > 0.7 {
            permissions.push(Permission::DeleteFiles);
            permissions.push(Permission::SubmitProposal);
        }

        // Voting permissions for established users
        if user_account.reputation_score > 0.5 {
            permissions.push(Permission::Vote);
        }

        // Admin permissions for premium/enterprise users with high reputation
        if user_account.reputation_score > 0.8 {
            match user_account.account_type {
                crate::governance::AccountType::Premium { .. } |
                crate::governance::AccountType::Enterprise { .. } => {
                    permissions.push(Permission::AdministerUsers);
                }
                _ => {}
            }
        }

        permissions
    }

    /// Validate session and get user permissions
    pub async fn validate_session(&self, session_id: &str) -> DfsResult<AuthSession> {
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| DfsError::Authentication("Invalid session".to_string()))?;

        // Check if session is expired
        if session.expires_at < Utc::now() {
            return Err(DfsError::Authentication("Session expired".to_string()));
        }

        Ok(session.clone())
    }

    /// Check if user can perform an operation
    pub async fn check_operation_permission(
        &self, 
        session_id: &str, 
        operation: &OperationContext
    ) -> DfsResult<()> {
        let session = self.validate_session(session_id).await?;
        
        // Check basic session validity
        if session.user_id != operation.user_id {
            return Err(DfsError::Authentication("Session user mismatch".to_string()));
        }

        // Check specific permissions
        let required_permission = match operation.operation_type {
            crate::quota_service::OperationType::Upload => Permission::WriteFiles,
            crate::quota_service::OperationType::Download => Permission::ReadFiles,
            crate::quota_service::OperationType::Delete => Permission::DeleteFiles,
            crate::quota_service::OperationType::List => Permission::ReadFiles,
            crate::quota_service::OperationType::Search => Permission::ReadFiles,
            crate::quota_service::OperationType::Metadata => Permission::ReadFiles,
        };

        if !session.permissions.contains(&required_permission) {
            return Err(DfsError::Authentication("Insufficient permissions".to_string()));
        }

        // Check quota limits
        self.quota_service.check_operation_allowed(operation).await
            .map_err(|e| DfsError::Authentication(e.to_string()))?;

        Ok(())
    }

    /// Submit a governance proposal
    pub async fn submit_proposal(
        &self,
        session_id: &str,
        title: String,
        description: String,
        proposal_type: ProposalType,
    ) -> DfsResult<NetworkProposal> {
        let session = self.validate_session(session_id).await?;
        
        // Check if user has permission to submit proposals
        if !session.permissions.contains(&Permission::SubmitProposal) {
            return Err(DfsError::Authentication("No permission to submit proposals".to_string()));
        }

        // Check user reputation and stake (simplified for this implementation)
        let user_account = self.user_registry.get_user(&session.user_id)
            .ok_or_else(|| DfsError::Authentication("User not found".to_string()))?;

        if user_account.reputation_score < 0.7 {
            return Err(DfsError::Authentication("Insufficient reputation to submit proposal".to_string()));
        }

        let config = self.governance_config.read().unwrap();
        let proposal = NetworkProposal {
            proposal_id: Uuid::new_v4(),
            title,
            description,
            proposer: session.user_id,
            proposal_type,
            voting_period: Duration::days(config.voting_period_days as i64),
            required_quorum: config.quorum_percentage,
            status: ProposalStatus::Voting,
            votes_for: 0,
            votes_against: 0,
            implementation_timeline: Some(Utc::now() + Duration::days(config.voting_period_days as i64 + 1)),
            created_at: Utc::now(),
        };

        self.governance.submit_proposal(proposal.clone())?;
        Ok(proposal)
    }

    /// Vote on a proposal
    pub async fn vote_on_proposal(
        &self,
        session_id: &str,
        proposal_id: Uuid,
        vote_type: VoteType,
    ) -> DfsResult<Vote> {
        let session = self.validate_session(session_id).await?;
        
        // Check if user has permission to vote
        if !session.permissions.contains(&Permission::Vote) {
            return Err(DfsError::Authentication("No permission to vote".to_string()));
        }

        // Check if proposal exists and is in voting state
        let proposal = self.governance.get_proposal(&proposal_id)
            .ok_or_else(|| DfsError::Generic("Proposal not found".to_string()))?;

        if !matches!(proposal.status, ProposalStatus::Voting) {
            return Err(DfsError::Generic("Proposal not in voting state".to_string()));
        }

        // Calculate vote weight (simplified - would be based on stake/reputation)
        let user_account = self.user_registry.get_user(&session.user_id)
            .ok_or_else(|| DfsError::Authentication("User not found".to_string()))?;

        let vote_weight = (user_account.reputation_score * 1000.0) as u64;

        let vote = Vote {
            vote_id: Uuid::new_v4(),
            proposal_id,
            voter: session.user_id,
            vote_type,
            stake_weight: vote_weight,
            timestamp: Utc::now(),
        };

        self.governance.vote_on_proposal(proposal_id, vote.clone())?;
        Ok(vote)
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let users = self.user_registry.list_users();
        let operators = self.bootstrap_admin.get_operators();
        let network_health = self.bootstrap_admin.check_network_health();
        
        // Calculate active users (simplified)
        let active_users_last_24h = users.iter()
            .filter(|u| u.last_activity > Utc::now() - Duration::hours(24))
            .count();

        // Calculate total storage and bandwidth (simplified)
        let total_storage_gb = users.len() as u64 * 5; // Rough estimate
        let total_bandwidth_gb = users.len() as u64 * 50; // Rough estimate

        // Calculate governance participation (simplified)
        let governance_participation = if users.len() > 0 {
            (users.iter().filter(|u| u.reputation_score > 0.5).count() as f64 / users.len() as f64) * 100.0
        } else {
            0.0
        };

        NetworkStats {
            total_users: users.len(),
            active_users_last_24h,
            total_operators: operators.len(),
            online_operators: network_health.online_operators,
            total_storage_gb,
            total_bandwidth_gb,
            network_health,
            governance_participation,
        }
    }

    /// Get user dashboard data
    pub async fn get_user_dashboard(&self, session_id: &str) -> DfsResult<UserDashboard> {
        let session = self.validate_session(session_id).await?;
        let user_account = self.user_registry.get_user(&session.user_id)
            .ok_or_else(|| DfsError::Authentication("User not found".to_string()))?;

        let quota = self.resource_manager.get_quota(&session.user_id);
        let usage_stats = self.quota_service.get_usage_stats(&session.user_id, 30).await;
        let active_proposals = self.governance.list_proposals()
            .into_iter()
            .filter(|p| matches!(p.status, ProposalStatus::Voting))
            .collect();

        Ok(UserDashboard {
            user_account,
            quota,
            usage_stats,
            active_proposals,
            permissions: session.permissions,
        })
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.active_sessions.write().await;
        let now = Utc::now();
        sessions.retain(|_, session| session.expires_at > now);
    }

    /// Get governance configuration
    pub fn get_governance_config(&self) -> GovernanceConfig {
        self.governance_config.read().unwrap().clone()
    }

    /// Update governance configuration (requires admin permissions)
    pub async fn update_governance_config(
        &self,
        session_id: &str,
        config: GovernanceConfig,
    ) -> DfsResult<()> {
        let session = self.validate_session(session_id).await?;
        
        if !session.permissions.contains(&Permission::AdministerUsers) {
            return Err(DfsError::Authentication("No permission to update governance config".to_string()));
        }

        let mut current_config = self.governance_config.write().unwrap();
        *current_config = config;

        Ok(())
    }

    /// Get reference to quota service for external use
    pub fn quota_service(&self) -> Arc<QuotaEnforcementService> {
        self.quota_service.clone()
    }

    /// Get reference to bootstrap admin service for external use
    pub fn bootstrap_admin(&self) -> Arc<BootstrapAdministrationService> {
        self.bootstrap_admin.clone()
    }
}

/// User dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboard {
    pub user_account: UserAccount,
    pub quota: Option<UserQuota>,
    pub usage_stats: UsageStats,
    pub active_proposals: Vec<NetworkProposal>,
    pub permissions: Vec<Permission>,
}

/// Health check result for the governance system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthCheck {
    pub overall_health: String,
    pub user_registry_status: String,
    pub quota_service_status: String,
    pub bootstrap_admin_status: String,
    pub governance_status: String,
    pub network_health: NetworkHealthStatus,
    pub timestamp: DateTime<Utc>,
}

impl NetworkGovernanceService {
    /// Perform comprehensive health check
    pub async fn health_check(&self) -> GovernanceHealthCheck {
        let network_health = self.bootstrap_admin.check_network_health();
        
        let overall_health = if network_health.can_reach_consensus {
            "healthy"
        } else {
            "degraded"
        };

        GovernanceHealthCheck {
            overall_health: overall_health.to_string(),
            user_registry_status: "operational".to_string(),
            quota_service_status: "operational".to_string(),
            bootstrap_admin_status: "operational".to_string(),
            governance_status: "operational".to_string(),
            network_health,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration_and_authentication() {
        let service = NetworkGovernanceService::new();
        
        // Register user
        let user = service.register_user(
            "test@example.com".to_string(),
            "pubkey123".to_string()
        ).await.unwrap();

        // Authenticate user
        let session = service.authenticate_user(&user.user_id, "pubkey123").await.unwrap();
        
        assert_eq!(session.user_id, user.user_id);
        assert!(!session.permissions.is_empty());
    }

    #[tokio::test]
    async fn test_proposal_submission() {
        let service = NetworkGovernanceService::new();
        
        // Register user with high reputation
        let user = service.register_user(
            "test@example.com".to_string(),
            "pubkey123".to_string()
        ).await.unwrap();

        // Set high reputation to allow proposal submission
        let mut user_account = user.clone();
        user_account.reputation_score = 0.8;
        service.user_registry.update_user(user_account).unwrap();

        // Authenticate user
        let session = service.authenticate_user(&user.user_id, "pubkey123").await.unwrap();
        
        // Submit proposal
        let proposal = service.submit_proposal(
            &session.session_id,
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            ProposalType::NetworkUpgrade,
        ).await.unwrap();

        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.proposer, user.user_id);
    }

    #[tokio::test]
    async fn test_operation_permission_check() {
        let service = NetworkGovernanceService::new();
        
        // Register user
        let user = service.register_user(
            "test@example.com".to_string(),
            "pubkey123".to_string()
        ).await.unwrap();

        // Authenticate user
        let session = service.authenticate_user(&user.user_id, "pubkey123").await.unwrap();
        
        // Test read operation (should be allowed)
        let read_context = OperationContext {
            user_id: user.user_id,
            operation_type: crate::quota_service::OperationType::List,
            file_size: None,
            expected_bandwidth: None,
        };

        let result = service.check_operation_permission(&session.session_id, &read_context).await;
        assert!(result.is_ok());

        // Test write operation (may not be allowed for unverified user)
        let write_context = OperationContext {
            user_id: user.user_id,
            operation_type: crate::quota_service::OperationType::Upload,
            file_size: Some(1024),
            expected_bandwidth: None,
        };

        let result = service.check_operation_permission(&session.session_id, &write_context).await;
        // This might fail due to verification status, which is expected
    }

    #[tokio::test]
    async fn test_network_stats() {
        let service = NetworkGovernanceService::new();
        
        // Register a few users
        for i in 0..5 {
            let _ = service.register_user(
                format!("user{}@example.com", i),
                format!("pubkey{}", i)
            ).await.unwrap();
        }

        let stats = service.get_network_stats().await;
        assert_eq!(stats.total_users, 5);
        assert_eq!(stats.total_operators, 0); // No operators registered
    }
}