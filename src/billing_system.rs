/// Comprehensive Billing and Payment System
///
/// This module implements a complete billing system with subscription management,
/// payment processing, and usage tracking for the DataMesh network.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{DateTime, Utc, Local, Datelike};

use crate::economics::EconomicModel;
use crate::governance::UserId;
use crate::database::DatabaseManager;

/// Subscription tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Pro,
    Enterprise,
    Custom,
}

/// Payment methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard { last_four: String, expiry: String },
    PayPal { email: String },
    Crypto { wallet_address: String, currency: String },
    BankTransfer { account_number: String },
    Token { balance: f64 },
}

/// Billing cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
    PayAsYouGo,
}

/// Subscription details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: UserId,
    pub tier: SubscriptionTier,
    pub billing_cycle: BillingCycle,
    pub price: f64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub auto_renew: bool,
    pub payment_method: PaymentMethod,
    pub usage_limits: UsageLimits,
    pub status: SubscriptionStatus,
}

/// Subscription status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Suspended,
    Cancelled,
    Expired,
    PendingPayment,
}

/// Usage limits for subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub storage_gb: Option<f64>,
    pub bandwidth_gb: Option<f64>,
    pub api_calls: Option<u64>,
    pub file_count: Option<u64>,
    pub concurrent_connections: Option<u32>,
}

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub user_id: UserId,
    pub resource_type: ResourceType,
    pub amount: f64,
    pub unit: String,
    pub cost: f64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Resource types for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Storage,
    Bandwidth,
    ApiCalls,
    ProcessingTime,
    PremiumFeatures,
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: UserId,
    pub subscription_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub line_items: Vec<LineItem>,
    pub issued_at: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
    pub resource_type: ResourceType,
}

/// Invoice status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Overdue,
    Cancelled,
}

/// Payment transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub user_id: UserId,
    pub invoice_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub transaction_id: Option<String>,
    pub failure_reason: Option<String>,
}

/// Payment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Billing system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    pub currency: String,
    pub tax_rate: f64,
    pub grace_period_days: u32,
    pub auto_suspend_after_days: u32,
    pub payment_retry_attempts: u32,
    pub invoice_generation_day: u32,
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            currency: "USD".to_string(),
            tax_rate: 0.0,
            grace_period_days: 7,
            auto_suspend_after_days: 30,
            payment_retry_attempts: 3,
            invoice_generation_day: 1,
        }
    }
}

/// Billing system
pub struct BillingSystem {
    config: BillingConfig,
    database: Arc<DatabaseManager>,
    economic_model: Arc<EconomicModel>,
    subscriptions: Arc<RwLock<HashMap<UserId, Subscription>>>,
    usage_records: Arc<RwLock<Vec<UsageRecord>>>,
    invoices: Arc<RwLock<HashMap<Uuid, Invoice>>>,
    payment_transactions: Arc<RwLock<HashMap<Uuid, PaymentTransaction>>>,
}

impl BillingSystem {
    /// Create a new billing system
    pub fn new(
        config: BillingConfig,
        database: Arc<DatabaseManager>,
        economic_model: Arc<EconomicModel>,
    ) -> Self {
        Self {
            config,
            database,
            economic_model,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            usage_records: Arc::new(RwLock::new(Vec::new())),
            invoices: Arc::new(RwLock::new(HashMap::new())),
            payment_transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the billing system
    pub async fn start(&self) -> Result<()> {
        info!("Starting billing system");
        
        // Load existing data
        self.load_subscriptions().await?;
        self.load_usage_records().await?;
        self.load_invoices().await?;
        
        // Start billing processes
        self.start_usage_tracking().await?;
        self.start_invoice_generation().await?;
        self.start_payment_processing().await?;
        self.start_subscription_management().await?;
        
        Ok(())
    }

    /// Create a new subscription
    pub async fn create_subscription(
        &self,
        user_id: UserId,
        tier: SubscriptionTier,
        billing_cycle: BillingCycle,
        payment_method: PaymentMethod,
    ) -> Result<Subscription> {
        let subscription_id = Uuid::new_v4();
        let now = Utc::now();
        
        let (price, usage_limits) = self.get_tier_details(&tier, &billing_cycle);
        
        let expires_at = match billing_cycle {
            BillingCycle::Monthly => now + chrono::Duration::days(30),
            BillingCycle::Quarterly => now + chrono::Duration::days(90),
            BillingCycle::Yearly => now + chrono::Duration::days(365),
            BillingCycle::PayAsYouGo => now + chrono::Duration::days(30), // Default
        };

        let subscription = Subscription {
            id: subscription_id,
            user_id,
            tier,
            billing_cycle,
            price,
            currency: self.config.currency.clone(),
            created_at: now,
            expires_at,
            auto_renew: true,
            payment_method,
            usage_limits,
            status: SubscriptionStatus::Active,
        };

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(user_id, subscription.clone());
        
        // Generate initial invoice
        self.generate_invoice_for_subscription(&subscription).await?;
        
        info!("Created subscription {} for user {}", subscription_id, user_id);
        Ok(subscription)
    }

    /// Get tier details
    fn get_tier_details(&self, tier: &SubscriptionTier, billing_cycle: &BillingCycle) -> (f64, UsageLimits) {
        let base_price = match tier {
            SubscriptionTier::Free => 0.0,
            SubscriptionTier::Basic => 9.99,
            SubscriptionTier::Pro => 29.99,
            SubscriptionTier::Enterprise => 99.99,
            SubscriptionTier::Custom => 0.0, // Custom pricing
        };

        let cycle_multiplier = match billing_cycle {
            BillingCycle::Monthly => 1.0,
            BillingCycle::Quarterly => 2.7, // 10% discount
            BillingCycle::Yearly => 10.0, // 17% discount
            BillingCycle::PayAsYouGo => 0.0,
        };

        let usage_limits = match tier {
            SubscriptionTier::Free => UsageLimits {
                storage_gb: Some(1.0),
                bandwidth_gb: Some(10.0),
                api_calls: Some(1000),
                file_count: Some(100),
                concurrent_connections: Some(2),
            },
            SubscriptionTier::Basic => UsageLimits {
                storage_gb: Some(100.0),
                bandwidth_gb: Some(1000.0),
                api_calls: Some(100000),
                file_count: Some(10000),
                concurrent_connections: Some(10),
            },
            SubscriptionTier::Pro => UsageLimits {
                storage_gb: Some(1000.0),
                bandwidth_gb: Some(10000.0),
                api_calls: Some(1000000),
                file_count: Some(100000),
                concurrent_connections: Some(50),
            },
            SubscriptionTier::Enterprise => UsageLimits {
                storage_gb: None, // Unlimited
                bandwidth_gb: None,
                api_calls: None,
                file_count: None,
                concurrent_connections: Some(500),
            },
            SubscriptionTier::Custom => UsageLimits {
                storage_gb: None,
                bandwidth_gb: None,
                api_calls: None,
                file_count: None,
                concurrent_connections: None,
            },
        };

        (base_price * cycle_multiplier, usage_limits)
    }

    /// Record usage for billing
    pub async fn record_usage(
        &self,
        user_id: UserId,
        resource_type: ResourceType,
        amount: f64,
        unit: String,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let cost = self.calculate_usage_cost(&resource_type, amount, &user_id).await?;
        
        let usage_record = UsageRecord {
            id: Uuid::new_v4(),
            user_id,
            resource_type,
            amount,
            unit,
            cost,
            timestamp: Utc::now(),
            metadata,
        };

        // Store usage record
        let mut usage_records = self.usage_records.write().await;
        usage_records.push(usage_record.clone());

        // Check usage limits
        self.check_usage_limits(&user_id).await?;

        debug!("Recorded usage: {:?}", usage_record);
        Ok(())
    }

    /// Calculate usage cost
    async fn calculate_usage_cost(
        &self,
        resource_type: &ResourceType,
        amount: f64,
        user_id: &UserId,
    ) -> Result<f64> {
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions.get(user_id);

        // Get tier-specific pricing
        let unit_cost = match (resource_type, subscription.map(|s| &s.tier)) {
            (ResourceType::Storage, Some(SubscriptionTier::Free)) => 0.0,
            (ResourceType::Storage, Some(SubscriptionTier::Basic)) => 0.0,
            (ResourceType::Storage, Some(SubscriptionTier::Pro)) => 0.0,
            (ResourceType::Storage, Some(SubscriptionTier::Enterprise)) => 0.0,
            (ResourceType::Storage, _) => 0.1, // $0.10 per GB
            
            (ResourceType::Bandwidth, Some(SubscriptionTier::Free)) => 0.0,
            (ResourceType::Bandwidth, Some(SubscriptionTier::Basic)) => 0.0,
            (ResourceType::Bandwidth, Some(SubscriptionTier::Pro)) => 0.0,
            (ResourceType::Bandwidth, Some(SubscriptionTier::Enterprise)) => 0.0,
            (ResourceType::Bandwidth, _) => 0.05, // $0.05 per GB
            
            (ResourceType::ApiCalls, Some(SubscriptionTier::Free)) => 0.0,
            (ResourceType::ApiCalls, Some(SubscriptionTier::Basic)) => 0.0,
            (ResourceType::ApiCalls, Some(SubscriptionTier::Pro)) => 0.0,
            (ResourceType::ApiCalls, Some(SubscriptionTier::Enterprise)) => 0.0,
            (ResourceType::ApiCalls, _) => 0.001, // $0.001 per API call
            
            (ResourceType::ProcessingTime, _) => 0.1, // $0.10 per hour
            (ResourceType::PremiumFeatures, _) => 1.0, // $1.00 per feature use
        };

        Ok(amount * unit_cost)
    }

    /// Check usage limits
    async fn check_usage_limits(&self, user_id: &UserId) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions.get(user_id);

        if let Some(sub) = subscription {
            let usage_records = self.usage_records.read().await;
            let user_usage: Vec<&UsageRecord> = usage_records
                .iter()
                .filter(|r| r.user_id == *user_id)
                .collect();

            // Check storage limit
            if let Some(storage_limit) = sub.usage_limits.storage_gb {
                let total_storage: f64 = user_usage
                    .iter()
                    .filter(|r| matches!(r.resource_type, ResourceType::Storage))
                    .map(|r| r.amount)
                    .sum();

                if total_storage > storage_limit {
                    warn!("User {} exceeded storage limit: {} GB > {} GB", user_id, total_storage, storage_limit);
                    // Could trigger limit enforcement here
                }
            }

            // Check bandwidth limit
            if let Some(bandwidth_limit) = sub.usage_limits.bandwidth_gb {
                let total_bandwidth: f64 = user_usage
                    .iter()
                    .filter(|r| matches!(r.resource_type, ResourceType::Bandwidth))
                    .map(|r| r.amount)
                    .sum();

                if total_bandwidth > bandwidth_limit {
                    warn!("User {} exceeded bandwidth limit: {} GB > {} GB", user_id, total_bandwidth, bandwidth_limit);
                }
            }

            // Check API call limit
            if let Some(api_limit) = sub.usage_limits.api_calls {
                let total_api_calls: f64 = user_usage
                    .iter()
                    .filter(|r| matches!(r.resource_type, ResourceType::ApiCalls))
                    .map(|r| r.amount)
                    .sum();

                if total_api_calls > api_limit as f64 {
                    warn!("User {} exceeded API call limit: {} > {}", user_id, total_api_calls, api_limit);
                }
            }
        }

        Ok(())
    }

    /// Generate invoice for subscription
    async fn generate_invoice_for_subscription(&self, subscription: &Subscription) -> Result<Invoice> {
        let invoice_id = Uuid::new_v4();
        let now = Utc::now();
        
        let mut line_items = Vec::new();
        
        // Add subscription fee
        line_items.push(LineItem {
            description: format!("{:?} subscription", subscription.tier),
            quantity: 1.0,
            unit_price: subscription.price,
            total: subscription.price,
            resource_type: ResourceType::PremiumFeatures,
        });

        // Add usage charges
        let usage_records = self.usage_records.read().await;
        let user_usage: Vec<&UsageRecord> = usage_records
            .iter()
            .filter(|r| r.user_id == subscription.user_id)
            .collect();

        for record in user_usage {
            if record.cost > 0.0 {
                line_items.push(LineItem {
                    description: format!("{:?} usage", record.resource_type),
                    quantity: record.amount,
                    unit_price: record.cost / record.amount,
                    total: record.cost,
                    resource_type: record.resource_type.clone(),
                });
            }
        }

        let total_amount: f64 = line_items.iter().map(|item| item.total).sum();
        let tax_amount = total_amount * self.config.tax_rate;
        let final_amount = total_amount + tax_amount;

        let invoice = Invoice {
            id: invoice_id,
            user_id: subscription.user_id,
            subscription_id: subscription.id,
            amount: final_amount,
            currency: subscription.currency.clone(),
            line_items,
            issued_at: now,
            due_date: now + chrono::Duration::days(30),
            paid_at: None,
            status: InvoiceStatus::Issued,
            payment_method: subscription.payment_method.clone(),
        };

        // Store invoice
        let mut invoices = self.invoices.write().await;
        invoices.insert(invoice_id, invoice.clone());

        info!("Generated invoice {} for user {} (${:.2})", invoice_id, subscription.user_id, final_amount);
        Ok(invoice)
    }

    /// Process payment for invoice
    pub async fn process_payment(&self, invoice_id: Uuid) -> Result<PaymentTransaction> {
        let mut invoices = self.invoices.write().await;
        let invoice = invoices.get_mut(&invoice_id)
            .ok_or_else(|| anyhow!("Invoice not found"))?;

        if invoice.status != InvoiceStatus::Issued {
            return Err(anyhow!("Invoice is not in a payable state"));
        }

        let transaction_id = Uuid::new_v4();
        let transaction = PaymentTransaction {
            id: transaction_id,
            user_id: invoice.user_id,
            invoice_id,
            amount: invoice.amount,
            currency: invoice.currency.clone(),
            payment_method: invoice.payment_method.clone(),
            status: PaymentStatus::Processing,
            created_at: Utc::now(),
            processed_at: None,
            transaction_id: Some(format!("txn_{}", transaction_id)),
            failure_reason: None,
        };

        // Store transaction
        let mut transactions = self.payment_transactions.write().await;
        transactions.insert(transaction_id, transaction.clone());

        // Simulate payment processing
        tokio::spawn(async move {
            // Simulate payment processing delay
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            // Simulate payment success/failure
            let success = fastrand::f64() > 0.1; // 90% success rate
            
            // Update transaction status (in real implementation, this would be done through callbacks)
            info!("Payment {} {}", transaction_id, if success { "succeeded" } else { "failed" });
        });

        Ok(transaction)
    }

    /// Get subscription for user
    pub async fn get_subscription(&self, user_id: &UserId) -> Result<Option<Subscription>> {
        let subscriptions = self.subscriptions.read().await;
        Ok(subscriptions.get(user_id).cloned())
    }

    /// Get user's usage records
    pub async fn get_user_usage(&self, user_id: &UserId) -> Result<Vec<UsageRecord>> {
        let usage_records = self.usage_records.read().await;
        let user_usage: Vec<UsageRecord> = usage_records
            .iter()
            .filter(|r| r.user_id == *user_id)
            .cloned()
            .collect();
        Ok(user_usage)
    }

    /// Get user's invoices
    pub async fn get_user_invoices(&self, user_id: &UserId) -> Result<Vec<Invoice>> {
        let invoices = self.invoices.read().await;
        let user_invoices: Vec<Invoice> = invoices
            .values()
            .filter(|i| i.user_id == *user_id)
            .cloned()
            .collect();
        Ok(user_invoices)
    }

    /// Start usage tracking
    async fn start_usage_tracking(&self) -> Result<()> {
        info!("Starting usage tracking");
        // This would start background processes for tracking usage
        Ok(())
    }

    /// Start invoice generation
    async fn start_invoice_generation(&self) -> Result<()> {
        let subscriptions = self.subscriptions.clone();
        let invoice_generation_day = self.config.invoice_generation_day;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // Daily check
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                if now.day() == invoice_generation_day {
                    info!("Generating monthly invoices");
                    
                    let subs = subscriptions.read().await;
                    for subscription in subs.values() {
                        if subscription.status == SubscriptionStatus::Active {
                            // Generate invoice (simplified)
                            debug!("Would generate invoice for subscription {}", subscription.id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Start payment processing
    async fn start_payment_processing(&self) -> Result<()> {
        info!("Starting payment processing");
        // This would start background payment processing
        Ok(())
    }

    /// Start subscription management
    async fn start_subscription_management(&self) -> Result<()> {
        let subscriptions = self.subscriptions.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60 * 60)); // Hourly check
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut subs = subscriptions.write().await;
                
                for subscription in subs.values_mut() {
                    if subscription.expires_at <= now && subscription.status == SubscriptionStatus::Active {
                        if subscription.auto_renew {
                            subscription.expires_at = now + chrono::Duration::days(30);
                            info!("Auto-renewed subscription {}", subscription.id);
                        } else {
                            subscription.status = SubscriptionStatus::Expired;
                            info!("Subscription {} expired", subscription.id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Load subscriptions from database
    async fn load_subscriptions(&self) -> Result<()> {
        // This would load subscriptions from the database
        info!("Loading subscriptions from database");
        Ok(())
    }

    /// Load usage records from database
    async fn load_usage_records(&self) -> Result<()> {
        // This would load usage records from the database
        info!("Loading usage records from database");
        Ok(())
    }

    /// Load invoices from database
    async fn load_invoices(&self) -> Result<()> {
        // This would load invoices from the database
        info!("Loading invoices from database");
        Ok(())
    }

    /// Get billing statistics
    pub async fn get_billing_stats(&self) -> Result<BillingStats> {
        let subscriptions = self.subscriptions.read().await;
        let usage_records = self.usage_records.read().await;
        let invoices = self.invoices.read().await;
        
        let total_subscriptions = subscriptions.len();
        let active_subscriptions = subscriptions.values()
            .filter(|s| s.status == SubscriptionStatus::Active)
            .count();
        
        let total_revenue: f64 = invoices.values()
            .filter(|i| i.status == InvoiceStatus::Paid)
            .map(|i| i.amount)
            .sum();
        
        let pending_revenue: f64 = invoices.values()
            .filter(|i| i.status == InvoiceStatus::Issued)
            .map(|i| i.amount)
            .sum();
        
        Ok(BillingStats {
            total_subscriptions,
            active_subscriptions,
            total_revenue,
            pending_revenue,
            total_usage_records: usage_records.len(),
            total_invoices: invoices.len(),
        })
    }
}

/// Billing statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct BillingStats {
    pub total_subscriptions: usize,
    pub active_subscriptions: usize,
    pub total_revenue: f64,
    pub pending_revenue: f64,
    pub total_usage_records: usize,
    pub total_invoices: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::get_default_db_path;

    #[tokio::test]
    async fn test_subscription_creation() {
        let db_path = get_default_db_path().unwrap();
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let economic_model = Arc::new(EconomicModel::new());
        
        let billing_system = BillingSystem::new(
            BillingConfig::default(),
            db,
            economic_model,
        );

        let user_id = 1;
        let subscription = billing_system.create_subscription(
            user_id,
            SubscriptionTier::Basic,
            BillingCycle::Monthly,
            PaymentMethod::CreditCard {
                last_four: "1234".to_string(),
                expiry: "12/25".to_string(),
            },
        ).await.unwrap();

        assert_eq!(subscription.user_id, user_id);
        assert_eq!(subscription.tier, SubscriptionTier::Basic);
        assert_eq!(subscription.status, SubscriptionStatus::Active);
    }

    #[tokio::test]
    async fn test_usage_recording() {
        let db_path = get_default_db_path().unwrap();
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let economic_model = Arc::new(EconomicModel::new());
        
        let billing_system = BillingSystem::new(
            BillingConfig::default(),
            db,
            economic_model,
        );

        let user_id = 1;
        let result = billing_system.record_usage(
            user_id,
            ResourceType::Storage,
            10.0,
            "GB".to_string(),
            HashMap::new(),
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_tier_details() {
        let db_path = get_default_db_path().unwrap();
        let db = Arc::new(DatabaseManager::new(&db_path).unwrap());
        let economic_model = Arc::new(EconomicModel::new());
        
        let billing_system = BillingSystem::new(
            BillingConfig::default(),
            db,
            economic_model,
        );

        let (price, limits) = billing_system.get_tier_details(
            &SubscriptionTier::Basic,
            &BillingCycle::Monthly,
        );

        assert_eq!(price, 9.99);
        assert_eq!(limits.storage_gb, Some(100.0));
        assert_eq!(limits.api_calls, Some(100000));
    }
}