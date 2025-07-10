/// DataMesh Governance Integration Example
///
/// This example demonstrates basic governance and administration system functionality
/// that is currently implemented in the DataMesh system.

use datamesh::governance::{AccountType, VerificationStatus};
use datamesh::economics::{EconomicModel, EconomicConfig};
use datamesh::error::DfsResult;
use uuid::Uuid;
use std::sync::Arc;

#[tokio::main]
async fn main() -> DfsResult<()> {
    println!("🚀 DataMesh Governance Integration Example");
    println!("==========================================");
    
    // 1. Economic Model Testing
    println!("\n1. Economic Model Testing");
    println!("-------------------------");
    
    let economic_model = Arc::new(EconomicModel::new());
    
    println!("✅ Economic model initialized");
    
    // Test economic configuration
    let config = EconomicConfig::default();
    println!("💰 Economic Configuration:");
    println!("   Storage Cost: ${:.3}/GB/month", config.storage_cost_per_gb_month);
    println!("   Bandwidth Cost: ${:.3}/GB", config.bandwidth_cost_per_gb);
    println!("   Staking Reward Rate: {:.1}% annually", config.staking_reward_rate_annual * 100.0);
    
    // 2. Account Type Testing
    println!("\n2. Account Types");
    println!("----------------");
    
    let account_types = [
        AccountType::Free {
            storage_gb: 5,
            bandwidth_gb_month: 100,
            api_calls_hour: 1000,
        },
        AccountType::Premium {
            storage_gb: 100,
            bandwidth_gb_month: 1000,
            api_calls_hour: 10000,
        },
        AccountType::Enterprise {
            storage_unlimited: true,
            bandwidth_unlimited: true,
            api_calls_unlimited: true,
            sla_guarantee: 0.999,
        },
    ];
    
    for account_type in &account_types {
        println!("   Account Type: {:?}", account_type);
    }
    
    // 3. Verification Status Testing
    println!("\n3. Verification Status");
    println!("----------------------");
    
    let verification_statuses = [
        VerificationStatus::Unverified,
        VerificationStatus::EmailVerified,
        VerificationStatus::PhoneVerified,
        VerificationStatus::KYCVerified,
    ];
    
    for status in &verification_statuses {
        println!("   Verification Status: {:?}", status);
    }
    
    // 4. UUID Generation for User IDs
    println!("\n4. User ID Generation");
    println!("--------------------");
    
    let user_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    for (i, user_id) in user_ids.iter().enumerate() {
        println!("   User {}: {}", i + 1, user_id);
    }
    
    // 5. Cost Calculations (using economic model)
    println!("\n5. Cost Calculations");
    println!("-------------------");
    
    let storage_gb = 10.0;
    let bandwidth_gb = 5.0;
    
    let storage_cost = storage_gb * config.storage_cost_per_gb_month;
    let bandwidth_cost = bandwidth_gb * config.bandwidth_cost_per_gb;
    let total_cost = storage_cost + bandwidth_cost;
    
    println!("   Storage ({} GB): ${:.2}", storage_gb, storage_cost);
    println!("   Bandwidth ({} GB): ${:.2}", bandwidth_gb, bandwidth_cost);
    println!("   Total Monthly Cost: ${:.2}", total_cost);
    
    // 6. Staking Calculations
    println!("\n6. Staking Calculations");
    println!("----------------------");
    
    let stake_amount = 10000.0; // 10k tokens
    let annual_reward = stake_amount * config.staking_reward_rate_annual;
    let monthly_reward = annual_reward / 12.0;
    let daily_reward = annual_reward / 365.0;
    
    println!("   Stake Amount: {} DMT", stake_amount);
    println!("   Annual Reward: {:.2} DMT", annual_reward);
    println!("   Monthly Reward: {:.2} DMT", monthly_reward);
    println!("   Daily Reward: {:.2} DMT", daily_reward);
    
    println!("\n🎉 Governance Integration Example Completed!");
    println!("============================================");
    
    Ok(())
}