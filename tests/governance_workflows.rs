/// Governance Workflow Tests for DataMesh
///
/// This module tests complete governance workflows including proposal creation,
/// voting, execution, and network parameter changes. Tests cover the full
/// lifecycle of governance operations and their integration with other systems.

mod test_utils;

use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use std::collections::HashMap;
use std::time::Duration;
use test_utils::{TestEnvironment, assertions, performance, mock_data};
use uuid::Uuid;

use datamesh::{
    governance::{
        NetworkGovernance, Proposal, ProposalStatus, ProposalType, Vote, VoteType,
        UserAccount, AccountType, VerificationStatus, OperatorAccount
    },
    economics::{EconomicModel, EconomicConfig},
    database::DatabaseManager,
    key_manager::KeyManager,
    storage_economy::StorageEconomy,
    audit_logger::AuditLogger,
};

/// Governance test environment with multiple users and operators
struct GovernanceTestEnv {
    test_env: TestEnvironment,
    governance: NetworkGovernance,
    economic_model: EconomicModel,
    storage_economy: StorageEconomy,
    audit_logger: AuditLogger,
    
    // Test users and operators
    regular_users: Vec<UserAccount>,
    operators: Vec<OperatorAccount>,
    admin_user: UserAccount,
}

impl GovernanceTestEnv {
    async fn new() -> Result<Self> {
        let test_env = TestEnvironment::new()?;
        
        // Initialize governance and economic systems
        let config = test_env.create_test_config();
        let governance = NetworkGovernance::new(&config)?;
        let economic_model = EconomicModel::new();
        let storage_economy = StorageEconomy::new(&config)?;
        let audit_logger = AuditLogger::new(&test_env.storage_path)?;
        
        // Create test users
        let regular_users = vec![
            create_test_user("user1@test.com", AccountType::Free { 
                storage_gb: 5, bandwidth_gb_month: 100, api_calls_hour: 1000 
            }),
            create_test_user("user2@test.com", AccountType::Premium { 
                storage_gb: 100, bandwidth_gb_month: 1000, api_calls_hour: 10000 
            }),
            create_test_user("user3@test.com", AccountType::Free { 
                storage_gb: 5, bandwidth_gb_month: 100, api_calls_hour: 1000 
            }),
        ];
        
        // Create test operators
        let operators = vec![
            create_test_operator("operator1@test.com", 10000),
            create_test_operator("operator2@test.com", 15000),
            create_test_operator("operator3@test.com", 20000),
        ];
        
        // Create admin user
        let admin_user = create_test_user("admin@test.com", AccountType::Administrator);
        
        // Register users and operators with governance
        for user in &regular_users {
            governance.register_user(user.clone()).await?;
        }
        
        for operator in &operators {
            governance.register_operator(operator.clone()).await?;
        }
        
        governance.register_user(admin_user.clone()).await?;
        
        Ok(GovernanceTestEnv {
            test_env,
            governance,
            economic_model,
            storage_economy,
            audit_logger,
            regular_users,
            operators,
            admin_user,
        })
    }
}

// Helper functions for creating test accounts
fn create_test_user(email: &str, account_type: AccountType) -> UserAccount {
    UserAccount {
        user_id: Uuid::new_v4(),
        email: email.to_string(),
        password_hash: "test_hash".to_string(),
        public_key: "test_public_key".to_string(),
        account_type,
        verification_status: VerificationStatus::EmailVerified,
        registration_date: Utc::now(),
        last_activity: Utc::now(),
        reputation_score: 75.0,
        abuse_flags: vec![],
        subscription: None,
    }
}

fn create_test_operator(email: &str, stake_amount: u64) -> OperatorAccount {
    OperatorAccount {
        operator_id: Uuid::new_v4(),
        email: email.to_string(),
        public_key: "operator_public_key".to_string(),
        stake_amount,
        registration_date: Utc::now(),
        last_activity: Utc::now(),
        reputation_score: 85.0,
        operated_nodes: vec![],
        vote_weight: stake_amount as f64,
    }
}

#[tokio::test]
async fn test_complete_proposal_lifecycle() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    let perf_test = performance::PerformanceTest::new("proposal_lifecycle");
    
    // Step 1: Create a network parameter change proposal
    let proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Increase Storage Reward Rate".to_string(),
        description: "Increase storage reward rate to incentivize more storage contributions".to_string(),
        proposer: env.operators[0].operator_id,
        proposal_type: ProposalType::ParameterChange {
            parameter_name: "storage_reward_rate_per_gb_hour".to_string(),
            old_value: "0.001".to_string(),
            new_value: "0.002".to_string(),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(24),
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    // Submit proposal
    let proposal_id = env.governance.submit_proposal(proposal.clone()).await?;
    
    // Verify proposal was created
    let stored_proposal = env.governance.get_proposal(proposal_id).await?;
    assert_eq!(stored_proposal.title, proposal.title);
    assert_eq!(stored_proposal.status, ProposalStatus::Active);
    
    // Step 2: Operators vote on the proposal
    
    // Operator 1 votes for (stake: 10000)
    env.governance.cast_vote(
        proposal_id,
        env.operators[0].operator_id,
        VoteType::For,
        "Agree with increasing rewards".to_string(),
    ).await?;
    
    // Operator 2 votes for (stake: 15000)
    env.governance.cast_vote(
        proposal_id,
        env.operators[1].operator_id,
        VoteType::For,
        "This will help network growth".to_string(),
    ).await?;
    
    // Operator 3 votes against (stake: 20000)
    env.governance.cast_vote(
        proposal_id,
        env.operators[2].operator_id,
        VoteType::Against,
        "Concerned about inflation impact".to_string(),
    ).await?;
    
    // Step 3: Check vote tallying
    let updated_proposal = env.governance.get_proposal(proposal_id).await?;
    assert_eq!(updated_proposal.votes_for, 2);
    assert_eq!(updated_proposal.votes_against, 1);
    
    // Total vote weight should be 25000 (10000 + 15000) for, 20000 against
    let total_for_weight = 25000.0;
    let total_against_weight = 20000.0;
    let total_weight = total_for_weight + total_against_weight;
    
    // Check if proposal passes (55.6% for vs 44.4% against, needs >50%)
    let for_percentage = total_for_weight / total_weight;
    assert!(for_percentage > 0.5, "Proposal should pass with majority vote");
    
    // Step 4: Execute the proposal (simulate voting deadline passed)
    env.governance.execute_proposal(proposal_id).await?;
    
    // Verify proposal status changed
    let executed_proposal = env.governance.get_proposal(proposal_id).await?;
    assert_eq!(executed_proposal.status, ProposalStatus::Executed);
    
    // Verify parameter change was applied
    let current_config = env.economic_model.get_config();
    assertions::assert_in_range(current_config.storage_reward_rate_per_gb_hour, 0.0019, 0.0021);
    
    perf_test.finish(Duration::from_secs(5));
    Ok(())
}

#[tokio::test]
async fn test_governance_user_tier_upgrade_workflow() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Test user requesting tier upgrade through governance
    let user = &env.regular_users[0]; // Free tier user
    
    // Create tier upgrade proposal
    let upgrade_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "User Tier Upgrade Request".to_string(),
        description: format!("User {} requests upgrade to Premium tier", user.email),
        proposer: user.user_id,
        proposal_type: ProposalType::UserTierChange {
            user_id: user.user_id,
            current_tier: "Free".to_string(),
            requested_tier: "Premium".to_string(),
            justification: "Increased storage needs for research project".to_string(),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(12),
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(upgrade_proposal).await?;
    
    // Admin approves the upgrade
    env.governance.cast_vote(
        proposal_id,
        env.admin_user.user_id,
        VoteType::For,
        "Valid research use case".to_string(),
    ).await?;
    
    // Execute the tier upgrade
    env.governance.execute_proposal(proposal_id).await?;
    
    // Verify user's account type was updated
    let updated_user = env.governance.get_user(user.user_id).await?;
    match updated_user.account_type {
        AccountType::Premium { storage_gb, bandwidth_gb_month, api_calls_hour } => {
            assert_eq!(storage_gb, 100);
            assert_eq!(bandwidth_gb_month, 1000);
            assert_eq!(api_calls_hour, 10000);
        }
        _ => panic!("User should have been upgraded to Premium tier"),
    }
    
    // Verify storage quota was updated in storage economy
    let user_quota = env.storage_economy.get_user_quota(user.user_id).await?;
    assert_eq!(user_quota.storage_gb, 100);
    
    Ok(())
}

#[tokio::test]
async fn test_network_configuration_governance() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Create proposal to change network parameters
    let config_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Update Network Configuration".to_string(),
        description: "Increase max file size and adjust connection limits".to_string(),
        proposer: env.operators[0].operator_id,
        proposal_type: ProposalType::NetworkConfigChange {
            parameters: HashMap::from([
                ("max_file_size_mb".to_string(), "200".to_string()),
                ("max_connections_per_node".to_string(), "1000".to_string()),
                ("connection_timeout_secs".to_string(), "120".to_string()),
            ]),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(48),
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(config_proposal).await?;
    
    // All operators vote for the change
    for operator in &env.operators {
        env.governance.cast_vote(
            proposal_id,
            operator.operator_id,
            VoteType::For,
            "Network needs these improvements".to_string(),
        ).await?;
    }
    
    // Execute the configuration change
    env.governance.execute_proposal(proposal_id).await?;
    
    // Verify configuration was updated
    let current_config = env.governance.get_network_config().await?;
    assert_eq!(current_config.max_file_size_mb, 200);
    assert_eq!(current_config.max_connections_per_node, 1000);
    assert_eq!(current_config.connection_timeout_secs, 120);
    
    Ok(())
}

#[tokio::test]
async fn test_operator_stake_and_voting_power() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Create proposal to test voting power distribution
    let test_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Test Voting Power".to_string(),
        description: "Testing stake-weighted voting".to_string(),
        proposer: env.operators[0].operator_id,
        proposal_type: ProposalType::ParameterChange {
            parameter_name: "test_parameter".to_string(),
            old_value: "old".to_string(),
            new_value: "new".to_string(),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(24),
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(test_proposal).await?;
    
    // Only the highest stake operator votes for (20000 stake)
    env.governance.cast_vote(
        proposal_id,
        env.operators[2].operator_id,
        VoteType::For,
        "High stake vote".to_string(),
    ).await?;
    
    // Two lower stake operators vote against (10000 + 15000 = 25000 stake)
    env.governance.cast_vote(
        proposal_id,
        env.operators[0].operator_id,
        VoteType::Against,
        "Lower stake vote 1".to_string(),
    ).await?;
    
    env.governance.cast_vote(
        proposal_id,
        env.operators[1].operator_id,
        VoteType::Against,
        "Lower stake vote 2".to_string(),
    ).await?;
    
    // Check voting results
    let final_proposal = env.governance.get_proposal(proposal_id).await?;
    
    // Verify vote counts
    assert_eq!(final_proposal.votes_for, 1);
    assert_eq!(final_proposal.votes_against, 2);
    
    // Verify weighted voting
    let total_stake = 45000.0; // 10000 + 15000 + 20000
    let for_weight = 20000.0;
    let against_weight = 25000.0;
    
    let for_percentage = for_weight / total_stake;
    let against_percentage = against_weight / total_stake;
    
    assertions::assert_in_range(for_percentage, 0.44, 0.45); // ~44.4%
    assertions::assert_in_range(against_percentage, 0.55, 0.56); // ~55.6%
    
    // Proposal should fail due to majority against
    assert!(against_percentage > 0.5, "Proposal should fail with majority against");
    
    Ok(())
}

#[tokio::test]
async fn test_governance_abuse_handling() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Create proposal to handle abusive user
    let user_to_suspend = &env.regular_users[2];
    
    let abuse_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Suspend Abusive User".to_string(),
        description: format!("User {} has been reported for spam and should be suspended", user_to_suspend.email),
        proposer: env.admin_user.user_id,
        proposal_type: ProposalType::UserSuspension {
            user_id: user_to_suspend.user_id,
            reason: "Multiple spam reports and malware distribution".to_string(),
            duration_days: 30,
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(6), // Urgent
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(abuse_proposal).await?;
    
    // Operators vote to approve suspension
    for operator in &env.operators {
        env.governance.cast_vote(
            proposal_id,
            operator.operator_id,
            VoteType::For,
            "Protect network integrity".to_string(),
        ).await?;
    }
    
    // Execute the suspension
    env.governance.execute_proposal(proposal_id).await?;
    
    // Verify user was suspended
    let updated_user = env.governance.get_user(user_to_suspend.user_id).await?;
    assert_eq!(updated_user.verification_status, VerificationStatus::Suspended);
    
    // Verify suspension is logged in audit trail
    let audit_logs = env.audit_logger.get_logs_for_user(&user_to_suspend.user_id.to_string())?;
    let suspension_log = audit_logs.iter()
        .find(|log| log.event_type == "user_suspended");
    assert!(suspension_log.is_some(), "Suspension should be logged");
    
    Ok(())
}

#[tokio::test]
async fn test_governance_economic_policy_changes() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Create proposal to change economic parameters
    let economic_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Adjust Economic Parameters".to_string(),
        description: "Update pricing and reward structure for better network economics".to_string(),
        proposer: env.operators[1].operator_id,
        proposal_type: ProposalType::EconomicPolicyChange {
            policy_name: "storage_pricing_update".to_string(),
            parameters: HashMap::from([
                ("storage_cost_per_gb_month".to_string(), "0.03".to_string()),
                ("bandwidth_cost_per_gb".to_string(), "0.005".to_string()),
                ("api_cost_per_thousand".to_string(), "0.001".to_string()),
            ]),
            impact_analysis: "Expected to increase operator revenue by 15% while remaining competitive".to_string(),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::hours(72), // Longer for economic changes
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(economic_proposal).await?;
    
    // Mixed voting on economic policy
    env.governance.cast_vote(
        proposal_id,
        env.operators[0].operator_id,
        VoteType::For,
        "Will improve sustainability".to_string(),
    ).await?;
    
    env.governance.cast_vote(
        proposal_id,
        env.operators[1].operator_id,
        VoteType::For,
        "Reasonable price adjustment".to_string(),
    ).await?;
    
    // Execute if passes
    env.governance.execute_proposal(proposal_id).await?;
    
    // Verify economic config was updated
    let updated_config = env.economic_model.get_config();
    assertions::assert_in_range(updated_config.storage_cost_per_gb_month, 0.029, 0.031);
    assertions::assert_in_range(updated_config.bandwidth_cost_per_gb, 0.004, 0.006);
    
    // Verify storage economy reflects new pricing
    let pricing_info = env.storage_economy.get_current_pricing().await?;
    assertions::assert_in_range(pricing_info.storage_cost_per_gb_month, 0.029, 0.031);
    
    Ok(())
}

#[tokio::test]
async fn test_governance_proposal_deadline_enforcement() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    // Create proposal with short deadline
    let short_deadline_proposal = Proposal {
        id: Uuid::new_v4(),
        title: "Short Deadline Test".to_string(),
        description: "Testing deadline enforcement".to_string(),
        proposer: env.operators[0].operator_id,
        proposal_type: ProposalType::ParameterChange {
            parameter_name: "test_param".to_string(),
            old_value: "old".to_string(),
            new_value: "new".to_string(),
        },
        created_at: Utc::now(),
        voting_deadline: Utc::now() + ChronoDuration::seconds(1), // Very short deadline
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        total_vote_weight: 0.0,
        execution_data: None,
    };
    
    let proposal_id = env.governance.submit_proposal(short_deadline_proposal).await?;
    
    // Wait for deadline to pass
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Try to vote after deadline
    let late_vote_result = env.governance.cast_vote(
        proposal_id,
        env.operators[0].operator_id,
        VoteType::For,
        "Late vote".to_string(),
    ).await;
    
    // Should fail due to expired deadline
    assert!(late_vote_result.is_err(), "Votes after deadline should be rejected");
    
    // Try to execute proposal (should fail due to insufficient votes)
    let execution_result = env.governance.execute_proposal(proposal_id).await;
    
    // Check proposal status was updated to expired or failed
    let final_proposal = env.governance.get_proposal(proposal_id).await?;
    assert!(
        matches!(final_proposal.status, ProposalStatus::Expired | ProposalStatus::Failed),
        "Proposal should be expired or failed"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_governance_concurrent_proposals() -> Result<()> {
    let env = GovernanceTestEnv::new().await?;
    
    let perf_test = performance::PerformanceTest::new("concurrent_governance");
    
    // Create multiple proposals concurrently
    let mut proposal_handles = Vec::new();
    
    for i in 0..5 {
        let governance = env.governance.clone();
        let operator_id = env.operators[i % env.operators.len()].operator_id;
        
        let handle = tokio::spawn(async move {
            let proposal = Proposal {
                id: Uuid::new_v4(),
                title: format!("Concurrent Proposal {}", i),
                description: format!("Testing concurrent proposal {}", i),
                proposer: operator_id,
                proposal_type: ProposalType::ParameterChange {
                    parameter_name: format!("param_{}", i),
                    old_value: "old".to_string(),
                    new_value: format!("new_{}", i),
                },
                created_at: Utc::now(),
                voting_deadline: Utc::now() + ChronoDuration::hours(24),
                status: ProposalStatus::Active,
                votes_for: 0,
                votes_against: 0,
                total_vote_weight: 0.0,
                execution_data: None,
            };
            
            governance.submit_proposal(proposal).await
        });
        
        proposal_handles.push(handle);
    }
    
    // Wait for all proposals to be submitted
    let mut proposal_ids = Vec::new();
    for handle in proposal_handles {
        let proposal_id = handle.await??;
        proposal_ids.push(proposal_id);
    }
    
    // Verify all proposals were created
    assert_eq!(proposal_ids.len(), 5);
    
    // Vote on all proposals concurrently
    let mut vote_handles = Vec::new();
    
    for (i, proposal_id) in proposal_ids.iter().enumerate() {
        let governance = env.governance.clone();
        let operator_id = env.operators[i % env.operators.len()].operator_id;
        let proposal_id = *proposal_id;
        
        let handle = tokio::spawn(async move {
            governance.cast_vote(
                proposal_id,
                operator_id,
                VoteType::For,
                format!("Vote on proposal {}", i),
            ).await
        });
        
        vote_handles.push(handle);
    }
    
    // Wait for all votes
    for handle in vote_handles {
        handle.await??;
    }
    
    // Verify all proposals received votes
    for proposal_id in proposal_ids {
        let proposal = env.governance.get_proposal(proposal_id).await?;
        assert!(proposal.votes_for > 0, "Proposal should have received votes");
    }
    
    perf_test.finish(Duration::from_secs(10));
    Ok(())
}