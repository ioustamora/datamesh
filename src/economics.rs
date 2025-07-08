/// Economic Model and Token System
///
/// This module implements the economic model for the DataMesh network as outlined
/// in the governance roadmap. It includes token management, incentive mechanisms,
/// cost calculations, and reward distribution systems.

use crate::governance::{UserId, BootstrapOperator, NetworkService};
use crate::error::{DfsResult, DfsError};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// DataMesh Token (DMT) representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub symbol: String,
    pub name: String,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub decimal_places: u8,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            symbol: "DMT".to_string(),
            name: "DataMesh Token".to_string(),
            total_supply: 1_000_000_000, // 1 billion tokens
            circulating_supply: 100_000_000, // 100 million initially circulating
            decimal_places: 18,
        }
    }
}

/// Token balance for users and operators
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenBalance {
    pub user_id: UserId,
    pub balance: u64,
    pub staked: u64,
    pub locked: u64,
    pub last_updated: DateTime<Utc>,
}

/// Economic configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicConfig {
    // Storage pricing
    pub storage_cost_per_gb_month: f64,
    pub storage_reward_rate_per_gb_hour: f64,
    
    // Bandwidth pricing
    pub bandwidth_cost_per_gb: f64,
    pub bandwidth_reward_rate_per_gb: f64,
    
    // API pricing
    pub api_cost_per_thousand: f64,
    
    // Staking and rewards
    pub minimum_stake_amount: u64,
    pub staking_reward_rate_annual: f64,
    pub bootstrap_operator_min_stake: u64,
    
    // Token economics
    pub inflation_rate_annual: f64,
    pub burn_rate_on_fees: f64,
}

impl Default for EconomicConfig {
    fn default() -> Self {
        Self {
            storage_cost_per_gb_month: 0.10,
            storage_reward_rate_per_gb_hour: 0.000011, // ~$0.08/GB/month
            bandwidth_cost_per_gb: 0.05,
            bandwidth_reward_rate_per_gb: 0.03,
            api_cost_per_thousand: 0.01,
            minimum_stake_amount: 1000,
            staking_reward_rate_annual: 0.05, // 5% annual reward
            bootstrap_operator_min_stake: 100000,
            inflation_rate_annual: 0.02, // 2% annual inflation
            burn_rate_on_fees: 0.1, // 10% of fees burned
        }
    }
}

/// Transaction record for economic activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: Uuid,
    pub from_user: Option<UserId>,
    pub to_user: Option<UserId>,
    pub amount: u64,
    pub transaction_type: TransactionType,
    pub fee: u64,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    Reward,
    Fee,
    Burn,
    Mint,
    StoragePayment,
    BandwidthPayment,
    ApiPayment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

/// Reward calculation for network participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardCalculation {
    pub recipient: UserId,
    pub reward_type: RewardType,
    pub base_amount: u64,
    pub quality_multiplier: f64,
    pub uptime_multiplier: f64,
    pub total_reward: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    StorageProvision,
    BandwidthProvision,
    NetworkMaintenance,
    GovernanceParticipation,
    Staking,
}

/// Cost calculation for user operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculation {
    pub user_id: UserId,
    pub operation_type: String,
    pub resource_usage: ResourceUsage,
    pub base_cost: u64,
    pub discount_multiplier: f64,
    pub total_cost: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub storage_gb: f64,
    pub bandwidth_gb: f64,
    pub api_calls: u32,
    pub duration_hours: f64,
}

/// Economic management service
pub struct EconomicService {
    config: Arc<RwLock<EconomicConfig>>,
    token_info: Arc<RwLock<Token>>,
    balances: Arc<RwLock<HashMap<UserId, TokenBalance>>>,
    transactions: Arc<RwLock<Vec<Transaction>>>,
    staking_pools: Arc<RwLock<HashMap<UserId, StakingPool>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub pool_id: Uuid,
    pub owner: UserId,
    pub total_staked: u64,
    pub participants: HashMap<UserId, u64>,
    pub annual_reward_rate: f64,
    pub lock_period_days: u32,
    pub created_at: DateTime<Utc>,
}

impl EconomicService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(EconomicConfig::default())),
            token_info: Arc::new(RwLock::new(Token::default())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(Vec::new())),
            staking_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize user balance
    pub fn initialize_user_balance(&self, user_id: UserId, initial_balance: u64) -> DfsResult<()> {
        let mut balances = self.balances.write().unwrap();
        balances.insert(user_id, TokenBalance {
            user_id,
            balance: initial_balance,
            staked: 0,
            locked: 0,
            last_updated: Utc::now(),
        });
        Ok(())
    }

    /// Get user balance
    pub fn get_balance(&self, user_id: &UserId) -> Option<TokenBalance> {
        let balances = self.balances.read().unwrap();
        balances.get(user_id).cloned()
    }

    /// Transfer tokens between users
    pub fn transfer_tokens(&self, from: UserId, to: UserId, amount: u64) -> DfsResult<Transaction> {
        let mut balances = self.balances.write().unwrap();
        
        // Check sender balance
        let sender_balance = balances.get_mut(&from)
            .ok_or_else(|| DfsError::Generic("Sender not found".to_string()))?;
        
        if sender_balance.balance < amount {
            return Err(DfsError::Generic("Insufficient balance".to_string()));
        }

        // Calculate fee (1% of transfer amount)
        let fee = amount / 100;
        let net_amount = amount - fee;

        // Update sender balance
        sender_balance.balance -= amount;
        sender_balance.last_updated = Utc::now();

        // Update recipient balance
        let recipient_balance = balances.entry(to).or_insert_with(|| TokenBalance {
            user_id: to,
            balance: 0,
            staked: 0,
            locked: 0,
            last_updated: Utc::now(),
        });
        recipient_balance.balance += net_amount;
        recipient_balance.last_updated = Utc::now();

        // Create transaction record
        let transaction = Transaction {
            transaction_id: Uuid::new_v4(),
            from_user: Some(from),
            to_user: Some(to),
            amount,
            transaction_type: TransactionType::Transfer,
            fee,
            timestamp: Utc::now(),
            status: TransactionStatus::Confirmed,
            description: format!("Transfer of {} tokens", amount),
        };

        // Record transaction
        let mut transactions = self.transactions.write().unwrap();
        transactions.push(transaction.clone());

        // Burn a portion of the fee
        self.burn_tokens(fee / 10)?; // Burn 10% of fee

        Ok(transaction)
    }

    /// Stake tokens for rewards
    pub fn stake_tokens(&self, user_id: UserId, amount: u64) -> DfsResult<Transaction> {
        let mut balances = self.balances.write().unwrap();
        
        let balance = balances.get_mut(&user_id)
            .ok_or_else(|| DfsError::Generic("User not found".to_string()))?;
        
        if balance.balance < amount {
            return Err(DfsError::Generic("Insufficient balance".to_string()));
        }

        let config = self.config.read().unwrap();
        if amount < config.minimum_stake_amount {
            return Err(DfsError::Generic("Amount below minimum stake".to_string()));
        }

        // Move tokens from balance to staked
        balance.balance -= amount;
        balance.staked += amount;
        balance.last_updated = Utc::now();

        // Create transaction record
        let transaction = Transaction {
            transaction_id: Uuid::new_v4(),
            from_user: Some(user_id),
            to_user: None,
            amount,
            transaction_type: TransactionType::Stake,
            fee: 0,
            timestamp: Utc::now(),
            status: TransactionStatus::Confirmed,
            description: format!("Staked {} tokens", amount),
        };

        let mut transactions = self.transactions.write().unwrap();
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// Unstake tokens
    pub fn unstake_tokens(&self, user_id: UserId, amount: u64) -> DfsResult<Transaction> {
        let mut balances = self.balances.write().unwrap();
        
        let balance = balances.get_mut(&user_id)
            .ok_or_else(|| DfsError::Generic("User not found".to_string()))?;
        
        if balance.staked < amount {
            return Err(DfsError::Generic("Insufficient staked amount".to_string()));
        }

        // Move tokens from staked to balance
        balance.staked -= amount;
        balance.balance += amount;
        balance.last_updated = Utc::now();

        // Create transaction record
        let transaction = Transaction {
            transaction_id: Uuid::new_v4(),
            from_user: Some(user_id),
            to_user: None,
            amount,
            transaction_type: TransactionType::Unstake,
            fee: 0,
            timestamp: Utc::now(),
            status: TransactionStatus::Confirmed,
            description: format!("Unstaked {} tokens", amount),
        };

        let mut transactions = self.transactions.write().unwrap();
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// Calculate cost for user operations
    pub fn calculate_operation_cost(&self, user_id: UserId, usage: ResourceUsage) -> CostCalculation {
        let config = self.config.read().unwrap();
        
        let storage_cost = (usage.storage_gb * usage.duration_hours / 24.0 / 30.0) * config.storage_cost_per_gb_month;
        let bandwidth_cost = usage.bandwidth_gb * config.bandwidth_cost_per_gb;
        let api_cost = (usage.api_calls as f64 / 1000.0) * config.api_cost_per_thousand;
        
        let base_cost = ((storage_cost + bandwidth_cost + api_cost) * 1000.0) as u64; // Convert to token units
        
        // Apply discount based on user staking (simplified)
        let discount_multiplier = if let Some(balance) = self.get_balance(&user_id) {
            if balance.staked > 10000 {
                0.9 // 10% discount for stakers
            } else {
                1.0
            }
        } else {
            1.0
        };

        let total_cost = (base_cost as f64 * discount_multiplier) as u64;

        CostCalculation {
            user_id,
            operation_type: "mixed".to_string(),
            resource_usage: usage,
            base_cost,
            discount_multiplier,
            total_cost,
            timestamp: Utc::now(),
        }
    }

    /// Calculate rewards for network participants
    pub fn calculate_participant_rewards(&self, user_id: UserId, contribution: NodeContribution) -> RewardCalculation {
        let config = self.config.read().unwrap();
        
        let storage_reward = (contribution.storage_gb_hours * config.storage_reward_rate_per_gb_hour) as u64;
        let bandwidth_reward = (contribution.bandwidth_gb_transferred * config.bandwidth_reward_rate_per_gb) as u64;
        
        let base_amount = storage_reward + bandwidth_reward;
        
        // Apply quality and uptime multipliers
        let quality_multiplier = contribution.quality_score.max(0.1).min(2.0);
        let uptime_multiplier = contribution.uptime_percentage.max(0.1).min(1.0);
        
        let total_reward = (base_amount as f64 * quality_multiplier * uptime_multiplier) as u64;

        RewardCalculation {
            recipient: user_id,
            reward_type: RewardType::StorageProvision,
            base_amount,
            quality_multiplier,
            uptime_multiplier,
            total_reward,
            period_start: contribution.period_start,
            period_end: contribution.period_end,
        }
    }

    /// Distribute rewards to participants
    pub fn distribute_rewards(&self, reward: RewardCalculation) -> DfsResult<Transaction> {
        let mut balances = self.balances.write().unwrap();
        
        let balance = balances.entry(reward.recipient).or_insert_with(|| TokenBalance {
            user_id: reward.recipient,
            balance: 0,
            staked: 0,
            locked: 0,
            last_updated: Utc::now(),
        });
        
        balance.balance += reward.total_reward;
        balance.last_updated = Utc::now();

        // Create transaction record
        let transaction = Transaction {
            transaction_id: Uuid::new_v4(),
            from_user: None, // Minted reward
            to_user: Some(reward.recipient),
            amount: reward.total_reward,
            transaction_type: TransactionType::Reward,
            fee: 0,
            timestamp: Utc::now(),
            status: TransactionStatus::Confirmed,
            description: format!("Reward for {:?}", reward.reward_type),
        };

        let mut transactions = self.transactions.write().unwrap();
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// Burn tokens (deflationary mechanism)
    pub fn burn_tokens(&self, amount: u64) -> DfsResult<()> {
        let mut token_info = self.token_info.write().unwrap();
        token_info.circulating_supply = token_info.circulating_supply.saturating_sub(amount);
        
        // Record burn transaction
        let transaction = Transaction {
            transaction_id: Uuid::new_v4(),
            from_user: None,
            to_user: None,
            amount,
            transaction_type: TransactionType::Burn,
            fee: 0,
            timestamp: Utc::now(),
            status: TransactionStatus::Confirmed,
            description: format!("Burned {} tokens", amount),
        };

        let mut transactions = self.transactions.write().unwrap();
        transactions.push(transaction);

        Ok(())
    }

    /// Get transaction history for a user
    pub fn get_transaction_history(&self, user_id: &UserId) -> Vec<Transaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.iter()
            .filter(|tx| tx.from_user == Some(*user_id) || tx.to_user == Some(*user_id))
            .cloned()
            .collect()
    }

    /// Get network economic statistics
    pub fn get_economic_stats(&self) -> EconomicStats {
        let token_info = self.token_info.read().unwrap();
        let balances = self.balances.read().unwrap();
        let transactions = self.transactions.read().unwrap();

        let total_staked = balances.values().map(|b| b.staked).sum();
        let total_locked = balances.values().map(|b| b.locked).sum();
        let total_in_circulation = balances.values().map(|b| b.balance).sum::<u64>() + total_staked + total_locked;
        
        let recent_transactions = transactions.iter()
            .filter(|tx| tx.timestamp > Utc::now() - Duration::hours(24))
            .count();

        EconomicStats {
            token_info: token_info.clone(),
            total_staked,
            total_locked,
            total_in_circulation,
            active_accounts: balances.len(),
            recent_transactions_24h: recent_transactions,
            average_transaction_size: if !transactions.is_empty() {
                transactions.iter().map(|tx| tx.amount).sum::<u64>() / transactions.len() as u64
            } else {
                0
            },
        }
    }

    /// Update economic configuration
    pub fn update_config(&self, config: EconomicConfig) -> DfsResult<()> {
        let mut current_config = self.config.write().unwrap();
        *current_config = config;
        Ok(())
    }

    /// Get current economic configuration
    pub fn get_config(&self) -> EconomicConfig {
        self.config.read().unwrap().clone()
    }
}

/// Node contribution data for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContribution {
    pub storage_gb_hours: f64,
    pub bandwidth_gb_transferred: f64,
    pub uptime_percentage: f64,
    pub quality_score: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Economic statistics for the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicStats {
    pub token_info: Token,
    pub total_staked: u64,
    pub total_locked: u64,
    pub total_in_circulation: u64,
    pub active_accounts: usize,
    pub recent_transactions_24h: usize,
    pub average_transaction_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_transfer() {
        let service = EconomicService::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        
        // Initialize balances
        service.initialize_user_balance(user1, 1000).unwrap();
        service.initialize_user_balance(user2, 0).unwrap();
        
        // Transfer tokens
        let tx = service.transfer_tokens(user1, user2, 100).unwrap();
        
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.from_user, Some(user1));
        assert_eq!(tx.to_user, Some(user2));
        
        // Check balances
        let balance1 = service.get_balance(&user1).unwrap();
        let balance2 = service.get_balance(&user2).unwrap();
        
        assert_eq!(balance1.balance, 900); // 1000 - 100
        assert_eq!(balance2.balance, 99); // 100 - 1 (fee)
    }

    #[test]
    fn test_staking() {
        let service = EconomicService::new();
        let user = Uuid::new_v4();
        
        // Initialize balance
        service.initialize_user_balance(user, 10000).unwrap();
        
        // Stake tokens
        let tx = service.stake_tokens(user, 5000).unwrap();
        
        assert_eq!(tx.amount, 5000);
        assert_eq!(tx.transaction_type, TransactionType::Stake);
        
        // Check balance
        let balance = service.get_balance(&user).unwrap();
        assert_eq!(balance.balance, 5000);
        assert_eq!(balance.staked, 5000);
    }

    #[test]
    fn test_cost_calculation() {
        let service = EconomicService::new();
        let user = Uuid::new_v4();
        
        let usage = ResourceUsage {
            storage_gb: 10.0,
            bandwidth_gb: 5.0,
            api_calls: 1000,
            duration_hours: 24.0,
        };
        
        let cost = service.calculate_operation_cost(user, usage);
        
        assert!(cost.base_cost > 0);
        assert!(cost.total_cost > 0);
        assert_eq!(cost.discount_multiplier, 1.0); // No staking discount
    }

    #[test]
    fn test_reward_calculation() {
        let service = EconomicService::new();
        let user = Uuid::new_v4();
        
        let contribution = NodeContribution {
            storage_gb_hours: 1000.0,
            bandwidth_gb_transferred: 500.0,
            uptime_percentage: 0.99,
            quality_score: 0.9,
            period_start: Utc::now() - Duration::hours(24),
            period_end: Utc::now(),
        };
        
        let reward = service.calculate_participant_rewards(user, contribution);
        
        assert!(reward.base_amount > 0);
        assert!(reward.total_reward > 0);
        assert_eq!(reward.quality_multiplier, 0.9);
        assert_eq!(reward.uptime_multiplier, 0.99);
    }

    #[test]
    fn test_economic_stats() {
        let service = EconomicService::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        
        // Initialize balances
        service.initialize_user_balance(user1, 1000).unwrap();
        service.initialize_user_balance(user2, 2000).unwrap();
        
        // Stake some tokens
        service.stake_tokens(user1, 500).unwrap();
        
        let stats = service.get_economic_stats();
        
        assert_eq!(stats.active_accounts, 2);
        assert_eq!(stats.total_staked, 500);
        assert_eq!(stats.total_in_circulation, 3000); // 500 + 2000 + 500 (staked)
    }
}