/// DataMesh Governance Integration Example
///
/// This example demonstrates how to use the DataMesh governance and administration
/// system as implemented based on the roadmap. It shows:
/// - User registration and authentication
/// - Resource quota management
/// - Bootstrap operator registration
/// - Network governance proposals and voting
/// - Economic model usage
/// - Integration with the existing file storage system

use datamesh::governance_service::{NetworkGovernanceService, GovernanceConfig, AuthSession};
use datamesh::governance::{AccountType, ProposalType, VoteType, NetworkService, VerificationStatus};
use datamesh::bootstrap_admin::{OperatorRegistrationRequest, ServiceConfig, AdminActionType, AdminTarget};
use datamesh::quota_service::{OperationContext, OperationType};
use datamesh::economics::{EconomicService, ResourceUsage, NodeContribution};
use datamesh::error::DfsResult;
use uuid::Uuid;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> DfsResult<()> {
    // Initialize the governance service
    let governance_service = Arc::new(NetworkGovernanceService::new());
    let economic_service = Arc::new(EconomicService::new());
    
    println!("🚀 DataMesh Governance Integration Example");
    println!("==========================================");
    
    // 1. User Registration and Authentication
    println!("\n1. User Registration and Authentication");
    println!("--------------------------------------");
    
    let user = governance_service.register_user(
        "alice@example.com".to_string(),
        "04a1b2c3d4...".to_string() // Mock public key
    ).await?;
    
    println!("✅ Registered user: {}", user.email);
    println!("   User ID: {}", user.user_id);
    println!("   Account Type: {:?}", user.account_type);
    println!("   Reputation: {:.2}", user.reputation_score);
    
    // Initialize user balance in economic system
    economic_service.initialize_user_balance(user.user_id, 10000)?;
    println!("💰 Initialized user balance: 10,000 DMT");
    
    // Authenticate user
    let session = governance_service.authenticate_user(&user.user_id, "04a1b2c3d4...").await?;
    println!("🔐 User authenticated, session: {}", session.session_id);
    println!("   Permissions: {:?}", session.permissions);
    
    // 2. Resource Quota Management
    println!("\n2. Resource Quota Management");
    println!("----------------------------");
    
    // Check quota for file upload
    let upload_context = OperationContext {
        user_id: user.user_id,
        operation_type: OperationType::Upload,
        file_size: Some(1024 * 1024), // 1MB file
        expected_bandwidth: None,
    };
    
    match governance_service.check_operation_permission(&session.session_id, &upload_context).await {
        Ok(()) => println!("✅ Upload operation allowed"),
        Err(e) => println!("❌ Upload operation denied: {}", e),
    }
    
    // Simulate file upload and quota usage
    let quota_service = governance_service.quota_service();
    let upload_guard = quota_service.start_operation(&upload_context).await?;
    
    // Simulate upload process
    sleep(Duration::from_millis(100)).await;
    
    upload_guard.complete(Some(1024 * 1024)).await;
    println!("📁 File upload completed, quota updated");
    
    // Get usage statistics
    let usage_stats = quota_service.get_usage_stats(&user.user_id, 30).await;
    println!("📊 Usage stats: {} files, {} bytes", usage_stats.files_uploaded, usage_stats.storage_used);
    
    // 3. Bootstrap Operator Registration
    println!("\n3. Bootstrap Operator Registration");
    println!("----------------------------------");
    
    let operator_request = OperatorRegistrationRequest {
        legal_name: "DataMesh Foundation".to_string(),
        contact_email: "admin@datamesh.foundation".to_string(),
        jurisdiction: "Estonia".to_string(),
        stake_amount: 1000000, // 1M DMT
        proposed_services: vec![NetworkService::Storage, NetworkService::Bandwidth],
        technical_contact: "tech@datamesh.foundation".to_string(),
        service_level_agreement: "99.9% uptime guarantee".to_string(),
    };
    
    let bootstrap_admin = governance_service.bootstrap_admin();
    let operator = bootstrap_admin.register_operator(operator_request, "12D3Ko...".to_string()).await?;
    
    println!("🏢 Bootstrap operator registered:");
    println!("   Operator ID: {}", operator.operator_id);
    println!("   Jurisdiction: {}", operator.jurisdiction);
    println!("   Stake: {} DMT", operator.stake);
    println!("   Governance Weight: {:.3}", operator.governance_weight);
    
    // Register storage service
    let storage_config = ServiceConfig::Storage {
        capacity_gb: 1000,
        redundancy_factor: 3,
        data_retention_days: 365,
    };
    
    let service_registration = bootstrap_admin.register_service(
        &operator.operator_id,
        NetworkService::Storage,
        storage_config,
    ).await?;
    
    println!("💾 Storage service registered: {}", service_registration.registration_id);
    
    // 4. Network Governance - Proposal and Voting
    println!("\n4. Network Governance - Proposal and Voting");
    println!("--------------------------------------------");
    
    // First, we need to increase user reputation to allow proposal submission
    let mut updated_user = user.clone();
    updated_user.reputation_score = 0.8; // High reputation
    updated_user.verification_status = VerificationStatus::KYCVerified;
    governance_service.user_registry.update_user(updated_user)?;
    
    // Re-authenticate to get updated permissions
    let session = governance_service.authenticate_user(&user.user_id, "04a1b2c3d4...").await?;
    
    // Submit a governance proposal
    let proposal = governance_service.submit_proposal(
        &session.session_id,
        "Increase Free Tier Storage Limit".to_string(),
        "Proposal to increase free tier storage from 5GB to 10GB to improve user adoption".to_string(),
        ProposalType::QuotaModification,
    ).await?;
    
    println!("🗳️  Governance proposal submitted:");
    println!("   Proposal ID: {}", proposal.proposal_id);
    println!("   Title: {}", proposal.title);
    println!("   Type: {:?}", proposal.proposal_type);
    println!("   Status: {:?}", proposal.status);
    
    // Vote on the proposal
    let vote = governance_service.vote_on_proposal(
        &session.session_id,
        proposal.proposal_id,
        VoteType::For,
    ).await?;
    
    println!("✅ Vote cast:");
    println!("   Vote ID: {}", vote.vote_id);
    println!("   Vote Type: {:?}", vote.vote_type);
    println!("   Stake Weight: {}", vote.stake_weight);
    
    // 5. Economic Model Integration
    println!("\n5. Economic Model Integration");
    println!("-----------------------------");
    
    // Transfer tokens between users
    let user2 = governance_service.register_user(
        "bob@example.com".to_string(),
        "04d5e6f7g8...".to_string(),
    ).await?;
    
    economic_service.initialize_user_balance(user2.user_id, 5000)?;
    
    let transfer_tx = economic_service.transfer_tokens(user.user_id, user2.user_id, 1000)?;
    println!("💸 Token transfer executed:");
    println!("   Transaction ID: {}", transfer_tx.transaction_id);
    println!("   Amount: {} DMT", transfer_tx.amount);
    println!("   Fee: {} DMT", transfer_tx.fee);
    
    // Stake tokens for rewards
    let stake_tx = economic_service.stake_tokens(user.user_id, 5000)?;
    println!("🔒 Tokens staked:");
    println!("   Transaction ID: {}", stake_tx.transaction_id);
    println!("   Amount: {} DMT", stake_tx.amount);
    
    // Calculate operation cost
    let usage = ResourceUsage {
        storage_gb: 5.0,
        bandwidth_gb: 10.0,
        api_calls: 500,
        duration_hours: 24.0,
    };
    
    let cost = economic_service.calculate_operation_cost(user.user_id, usage);
    println!("💰 Operation cost calculation:");
    println!("   Base Cost: {} DMT", cost.base_cost);
    println!("   Discount: {:.1}%", (1.0 - cost.discount_multiplier) * 100.0);
    println!("   Total Cost: {} DMT", cost.total_cost);
    
    // Calculate node rewards
    let contribution = NodeContribution {
        storage_gb_hours: 1000.0,
        bandwidth_gb_transferred: 500.0,
        uptime_percentage: 0.995,
        quality_score: 0.9,
        period_start: chrono::Utc::now() - chrono::Duration::hours(24),
        period_end: chrono::Utc::now(),
    };
    
    let reward = economic_service.calculate_participant_rewards(user.user_id, contribution);
    println!("🎁 Node reward calculation:");
    println!("   Base Reward: {} DMT", reward.base_amount);
    println!("   Quality Multiplier: {:.2}x", reward.quality_multiplier);
    println!("   Uptime Multiplier: {:.3}x", reward.uptime_multiplier);
    println!("   Total Reward: {} DMT", reward.total_reward);
    
    // Distribute the reward
    let reward_tx = economic_service.distribute_rewards(reward)?;
    println!("   Reward distributed: {}", reward_tx.transaction_id);
    
    // 6. Administrative Actions
    println!("\n6. Administrative Actions");
    println!("-------------------------");
    
    // Increase operator reputation to allow admin actions
    let operators = bootstrap_admin.get_operators();
    if let Some(mut op) = operators.first().cloned() {
        op.reputation_score = 0.9; // High reputation for admin actions
        
        let admin_action = bootstrap_admin.execute_admin_action(
            &op.operator_id,
            AdminActionType::ApproveUser,
            AdminTarget::User(user.user_id),
            "User verification completed".to_string(),
        ).await?;
        
        println!("🔨 Administrative action executed:");
        println!("   Action ID: {}", admin_action.action_id);
        println!("   Action Type: {:?}", admin_action.action_type);
        println!("   Reason: {}", admin_action.reason);
    }
    
    // 7. Network Statistics and Health
    println!("\n7. Network Statistics and Health");
    println!("--------------------------------");
    
    let network_stats = governance_service.get_network_stats().await;
    println!("📈 Network Statistics:");
    println!("   Total Users: {}", network_stats.total_users);
    println!("   Active Users (24h): {}", network_stats.active_users_last_24h);
    println!("   Total Operators: {}", network_stats.total_operators);
    println!("   Online Operators: {}", network_stats.online_operators);
    println!("   Governance Participation: {:.1}%", network_stats.governance_participation);
    
    let health_check = governance_service.health_check().await;
    println!("🏥 System Health: {}", health_check.overall_health);
    
    let economic_stats = economic_service.get_economic_stats();
    println!("💹 Economic Statistics:");
    println!("   Total Supply: {} DMT", economic_stats.token_info.total_supply);
    println!("   Circulating Supply: {} DMT", economic_stats.token_info.circulating_supply);
    println!("   Total Staked: {} DMT", economic_stats.total_staked);
    println!("   Active Accounts: {}", economic_stats.active_accounts);
    
    // 8. User Dashboard
    println!("\n8. User Dashboard");
    println!("----------------");
    
    let dashboard = governance_service.get_user_dashboard(&session.session_id).await?;
    println!("👤 User Dashboard for {}:", dashboard.user_account.email);
    println!("   Reputation: {:.2}", dashboard.user_account.reputation_score);
    println!("   Verification: {:?}", dashboard.user_account.verification_status);
    println!("   Storage Used: {} bytes", dashboard.usage_stats.storage_used);
    println!("   Bandwidth Used: {} GB", dashboard.usage_stats.bandwidth_used);
    println!("   Active Proposals: {}", dashboard.active_proposals.len());
    println!("   Permissions: {:?}", dashboard.permissions);
    
    // 9. Configuration Management
    println!("\n9. Configuration Management");
    println!("---------------------------");
    
    let config = governance_service.get_governance_config();
    println!("⚙️  Current Governance Configuration:");
    println!("   Min Stake for Proposal: {} DMT", config.min_stake_for_proposal);
    println!("   Voting Period: {} days", config.voting_period_days);
    println!("   Quorum Percentage: {:.1}%", config.quorum_percentage);
    println!("   Min Reputation for Vote: {:.2}", config.min_reputation_for_vote);
    
    let economic_config = economic_service.get_config();
    println!("💰 Economic Configuration:");
    println!("   Storage Cost: ${:.3}/GB/month", economic_config.storage_cost_per_gb_month);
    println!("   Bandwidth Cost: ${:.3}/GB", economic_config.bandwidth_cost_per_gb);
    println!("   Staking Reward Rate: {:.1}% annually", economic_config.staking_reward_rate_annual * 100.0);
    
    println!("\n🎉 Governance Integration Example Completed!");
    println!("============================================");
    
    Ok(())
}

/// Example of integrating governance with existing file operations
async fn governance_integrated_file_upload(
    governance_service: &NetworkGovernanceService,
    session_id: &str,
    file_data: Vec<u8>,
    file_name: &str,
) -> DfsResult<()> {
    // Extract user ID from session
    let session = governance_service.validate_session(session_id).await?;
    
    // Create operation context
    let context = OperationContext {
        user_id: session.user_id,
        operation_type: OperationType::Upload,
        file_size: Some(file_data.len() as u64),
        expected_bandwidth: Some(1.0), // 1 Mbps
    };
    
    // Check permissions and quotas
    governance_service.check_operation_permission(session_id, &context).await?;
    
    // Start the operation (this will track quota usage)
    let quota_service = governance_service.quota_service();
    let guard = quota_service.start_operation(&context).await?;
    
    // Simulate file upload process
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // TODO: Integrate with actual file storage system
    // let file_key = file_storage::store_file(file_data, file_name).await?;
    
    // Complete the operation (this updates usage statistics)
    guard.complete(Some(file_data.len() as u64)).await;
    
    println!("✅ File '{}' uploaded successfully with governance integration", file_name);
    
    Ok(())
}

/// Example of bootstrap operator onboarding process
async fn bootstrap_operator_onboarding_process(
    governance_service: &NetworkGovernanceService,
    economic_service: &EconomicService,
) -> DfsResult<()> {
    println!("\n🚀 Bootstrap Operator Onboarding Process");
    println!("========================================");
    
    // Step 1: Operator submits registration
    let registration_request = OperatorRegistrationRequest {
        legal_name: "Global DataMesh Services Ltd".to_string(),
        contact_email: "contact@globaldm.com".to_string(),
        jurisdiction: "Switzerland".to_string(),
        stake_amount: 2000000, // 2M DMT
        proposed_services: vec![
            NetworkService::Storage,
            NetworkService::Bandwidth,
            NetworkService::ContentDelivery,
        ],
        technical_contact: "tech@globaldm.com".to_string(),
        service_level_agreement: "99.95% uptime, 24/7 support".to_string(),
    };
    
    // Step 2: Register operator
    let bootstrap_admin = governance_service.bootstrap_admin();
    let operator = bootstrap_admin.register_operator(
        registration_request,
        "12D3KooWH5e...".to_string(),
    ).await?;
    
    println!("✅ Operator registered: {}", operator.operator_id);
    
    // Step 3: Initialize economic stake
    economic_service.initialize_user_balance(
        Uuid::new_v4(), // Would be operator's user ID
        operator.stake,
    )?;
    
    // Step 4: Register services
    let services = vec![
        (NetworkService::Storage, ServiceConfig::Storage {
            capacity_gb: 10000,
            redundancy_factor: 3,
            data_retention_days: 2555, // 7 years
        }),
        (NetworkService::Bandwidth, ServiceConfig::Bandwidth {
            max_mbps: 1000.0,
            data_transfer_limit_gb: None,
        }),
        (NetworkService::ContentDelivery, ServiceConfig::ContentDelivery {
            cache_size_gb: 1000,
            supported_regions: vec!["EU".to_string(), "NA".to_string()],
        }),
    ];
    
    for (service_type, config) in services {
        let registration = bootstrap_admin.register_service(
            &operator.operator_id,
            service_type.clone(),
            config,
        ).await?;
        
        println!("✅ Service registered: {:?} - {}", service_type, registration.registration_id);
    }
    
    // Step 5: Start monitoring and metrics collection
    // This would be done by the actual node software
    println!("📊 Operator onboarding complete. Monitoring active.");
    
    Ok(())
}

/// Example of governance proposal lifecycle
async fn governance_proposal_lifecycle(
    governance_service: &NetworkGovernanceService,
) -> DfsResult<()> {
    println!("\n🗳️  Governance Proposal Lifecycle");
    println!("================================");
    
    // Create a high-reputation user for proposal submission
    let proposer = governance_service.register_user(
        "proposer@datamesh.org".to_string(),
        "04proposer123...".to_string(),
    ).await?;
    
    // Set high reputation
    let mut proposer_account = proposer.clone();
    proposer_account.reputation_score = 0.9;
    proposer_account.verification_status = VerificationStatus::KYCVerified;
    governance_service.user_registry.update_user(proposer_account)?;
    
    // Authenticate proposer
    let proposer_session = governance_service.authenticate_user(
        &proposer.user_id,
        "04proposer123...",
    ).await?;
    
    // Submit proposal
    let proposal = governance_service.submit_proposal(
        &proposer_session.session_id,
        "Implement Content Moderation Framework".to_string(),
        "Proposal to implement automated content moderation with community appeals process".to_string(),
        ProposalType::SecurityPolicy,
    ).await?;
    
    println!("📝 Proposal submitted: {}", proposal.title);
    println!("   ID: {}", proposal.proposal_id);
    
    // Create voters
    let voters = vec![
        ("voter1@datamesh.org", "04voter1...", VoteType::For),
        ("voter2@datamesh.org", "04voter2...", VoteType::For),
        ("voter3@datamesh.org", "04voter3...", VoteType::Against),
    ];
    
    for (email, pubkey, vote_type) in voters {
        let voter = governance_service.register_user(
            email.to_string(),
            pubkey.to_string(),
        ).await?;
        
        // Set reputation to allow voting
        let mut voter_account = voter.clone();
        voter_account.reputation_score = 0.6;
        governance_service.user_registry.update_user(voter_account)?;
        
        let voter_session = governance_service.authenticate_user(&voter.user_id, pubkey).await?;
        
        let vote = governance_service.vote_on_proposal(
            &voter_session.session_id,
            proposal.proposal_id,
            vote_type.clone(),
        ).await?;
        
        println!("🗳️  Vote cast by {}: {:?}", email, vote_type);
    }
    
    println!("✅ Proposal lifecycle demonstration complete");
    
    Ok(())
}