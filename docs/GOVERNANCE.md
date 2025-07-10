# DataMesh Governance & Administration System

This document describes the governance and administration system implemented for DataMesh, based on the roadmap's vision of a managed public network with bootstrap operator administration and fair usage policies.

## Overview

The governance system transforms DataMesh from a basic distributed storage system into a **governable, enterprise-scale public platform** that balances decentralization benefits with practical resource management and legal compliance.

## Architecture

The governance system consists of several integrated components:

```
┌─────────────────────────────────────────────────────────────────┐
│                     NetworkGovernanceService                    │
│                    (Main Orchestration Layer)                   │
├─────────────────────────────────────────────────────────────────┤
│  UserRegistry  │  QuotaService  │  BootstrapAdmin  │  Economics │
│  - User mgmt   │  - Quotas      │  - Operators     │  - Tokens  │
│  - Auth        │  - Rate limits │  - Services      │  - Rewards │
│  - Permissions │  - Usage       │  - Admin actions │  - Costs   │
├─────────────────────────────────────────────────────────────────┤
│                        Core DataMesh                            │
│                   (File Storage & Network)                      │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. User Authentication & Management (`governance.rs`)

**Purpose**: Manages user accounts, authentication, and authorization within the network.

**Key Features**:
- User registration with email and cryptographic keys
- Account types (Free, Premium, Enterprise) with different privileges
- Reputation scoring system
- Multi-level verification (Email, Phone, KYC)
- Abuse flag tracking and content moderation

**Usage Example**:
```rust
use datamesh::governance_service::NetworkGovernanceService;

let service = NetworkGovernanceService::new();

// Register a new user
let user = service.register_user(
    "alice@example.com".to_string(),
    "04a1b2c3d4...".to_string() // Public key
).await?;

// Authenticate user
let session = service.authenticate_user(&user.user_id, "04a1b2c3d4...").await?;
```

### 2. Resource Quotas & Fair Usage (`quota_service.rs`)

**Purpose**: Enforces resource limits and fair usage policies to prevent abuse and ensure quality of service.

**Key Features**:
- Storage quotas (GB per user, file count, file size limits)
- Bandwidth quotas (upload/download speeds, monthly transfer limits)
- API rate limiting (requests per hour, concurrent operations)
- Real-time quota enforcement
- Usage tracking and analytics

**Usage Example**:
```rust
use datamesh::quota_service::{OperationContext, OperationType};

let context = OperationContext {
    user_id: user.user_id,
    operation_type: OperationType::Upload,
    file_size: Some(1024 * 1024), // 1MB
    expected_bandwidth: None,
};

// Check if operation is allowed
service.check_operation_permission(&session.session_id, &context).await?;

// Start operation with quota tracking
let guard = quota_service.start_operation(&context).await?;
// ... perform file upload ...
guard.complete(Some(1024 * 1024)).await; // Update usage stats
```

### 3. Bootstrap Node Administration (`bootstrap_admin.rs`)

**Purpose**: Manages bootstrap operators who provide network infrastructure and have administrative privileges.

**Key Features**:
- Operator registration and approval process
- Service registration (Storage, Bandwidth, CDN, Monitoring)
- Performance metrics tracking
- Administrative action execution
- Reputation scoring for operators
- Network health monitoring

**Usage Example**:
```rust
use datamesh::bootstrap_admin::{OperatorRegistrationRequest, ServiceConfig};

let request = OperatorRegistrationRequest {
    legal_name: "DataMesh Foundation".to_string(),
    jurisdiction: "Estonia".to_string(),
    stake_amount: 1000000, // 1M DMT tokens
    proposed_services: vec![NetworkService::Storage, NetworkService::Bandwidth],
    // ... other fields
};

let operator = bootstrap_admin.register_operator(request, "peer_id").await?;

// Register a storage service
let storage_config = ServiceConfig::Storage {
    capacity_gb: 1000,
    redundancy_factor: 3,
    data_retention_days: 365,
};

bootstrap_admin.register_service(
    &operator.operator_id,
    NetworkService::Storage,
    storage_config,
).await?;
```

### 4. Network Governance Framework (`governance_service.rs`)

**Purpose**: Implements democratic governance through proposals and voting mechanisms.

**Key Features**:
- Proposal submission system
- Voting mechanisms with stake-weighted influence
- Proposal types (NetworkUpgrade, FeeAdjustment, QuotaModification, etc.)
- Governance configuration management
- Execution of approved proposals

**Usage Example**:
```rust
// Submit a governance proposal
let proposal = service.submit_proposal(
    &session.session_id,
    "Increase Free Tier Storage".to_string(),
    "Proposal to increase free storage from 5GB to 10GB".to_string(),
    ProposalType::QuotaModification,
).await?;

// Vote on the proposal
let vote = service.vote_on_proposal(
    &session.session_id,
    proposal.proposal_id,
    VoteType::For,
).await?;
```

### 5. Economic Model & Token System (`economics.rs`)

**Purpose**: Manages the token economy, rewards, and cost calculations for network sustainability.

**Key Features**:
- DataMesh Token (DMT) management
- Token transfers and staking
- Cost calculations for operations
- Reward distribution to node operators
- Economic statistics and analytics
- Inflation and deflation mechanisms

**Usage Example**:
```rust
use datamesh::economics::{EconomicService, ResourceUsage};

let economic_service = EconomicService::new();

// Initialize user balance
economic_service.initialize_user_balance(user_id, 10000)?;

// Transfer tokens
let tx = economic_service.transfer_tokens(sender_id, recipient_id, 1000)?;

// Stake tokens for rewards
let stake_tx = economic_service.stake_tokens(user_id, 5000)?;

// Calculate operation cost
let usage = ResourceUsage {
    storage_gb: 5.0,
    bandwidth_gb: 10.0,
    api_calls: 500,
    duration_hours: 24.0,
};

let cost = economic_service.calculate_operation_cost(user_id, usage);
```

## Account Types and Quotas

### Free Tier
- **Storage**: 5GB
- **Bandwidth**: 50GB/month
- **API Calls**: 100/hour
- **File Size Limit**: 100MB
- **Max Files**: 1,000

### Premium Tier
- **Storage**: 100GB
- **Bandwidth**: 1TB/month
- **API Calls**: 10,000/hour
- **File Size Limit**: 1GB
- **Max Files**: 10,000

### Enterprise Tier
- **Storage**: Unlimited
- **Bandwidth**: Unlimited
- **API Calls**: Unlimited
- **File Size Limit**: 10GB
- **SLA**: 99.9% uptime guarantee

## Integration with Existing System

The governance system is designed to integrate seamlessly with the existing DataMesh file storage system:

```rust
// Example of governance-integrated file upload
async fn upload_file_with_governance(
    governance_service: &NetworkGovernanceService,
    session_id: &str,
    file_data: Vec<u8>,
    file_name: &str,
) -> DfsResult<String> {
    // 1. Validate user session
    let session = governance_service.validate_session(session_id).await?;
    
    // 2. Check operation permissions and quotas
    let context = OperationContext {
        user_id: session.user_id,
        operation_type: OperationType::Upload,
        file_size: Some(file_data.len() as u64),
        expected_bandwidth: None,
    };
    
    governance_service.check_operation_permission(session_id, &context).await?;
    
    // 3. Start quota tracking
    let quota_service = governance_service.quota_service();
    let guard = quota_service.start_operation(&context).await?;
    
    // 4. Perform actual file upload (existing DataMesh functionality)
    // let file_key = file_storage::store_file(file_data, file_name).await?;
    
    // 5. Complete operation and update usage
    guard.complete(Some(file_data.len() as u64)).await;
    
    Ok("file_key".to_string())
}
```

## Running the Example

To see the governance system in action:

```bash
# Run the integration example
cargo run --example governance_integration

# Run tests
cargo test governance
cargo test quota_service
cargo test bootstrap_admin
cargo test economics
```

## Configuration

The governance system can be configured through the `GovernanceConfig` and `EconomicConfig` structures:

```rust
// Governance configuration
let config = GovernanceConfig {
    min_stake_for_proposal: 10000,
    voting_period_days: 14,
    quorum_percentage: 20.0,
    min_reputation_for_vote: 0.3,
    // ... other parameters
};

// Economic configuration
let economic_config = EconomicConfig {
    storage_cost_per_gb_month: 0.10,
    bandwidth_cost_per_gb: 0.05,
    staking_reward_rate_annual: 0.05,
    // ... other parameters
};
```

## Security Considerations

1. **Authentication**: All operations require valid session tokens
2. **Authorization**: Role-based permissions prevent unauthorized actions
3. **Quota Enforcement**: Prevents resource abuse and ensures fair usage
4. **Reputation System**: Establishes trust through historical behavior
5. **Audit Logging**: All administrative actions are logged for transparency

## Deployment Scenarios

### Community Network
- Bootstrap operators: Academic institutions, foundations
- Funding: Grants, donations, minimal fees
- Governance: Community-driven decisions

### Commercial Network
- Bootstrap operators: Cloud providers, CDN companies
- Funding: Competitive pricing, SLA guarantees
- Governance: Stakeholder voting with economic incentives

### Hybrid Network (Recommended)
- Bootstrap operators: Mix of commercial and non-profit
- Funding: Freemium model with multiple tiers
- Governance: Token-based democratic participation

## Future Enhancements

1. **Advanced Analytics**: Machine learning for usage prediction and optimization
2. **Cross-Chain Integration**: Support for multiple blockchain networks
3. **Advanced Staking**: Liquid staking and delegation mechanisms
4. **Content Delivery Network**: Intelligent caching and geographic distribution
5. **Enterprise Features**: Advanced SLA monitoring and priority support

## Contributing

The governance system is designed to be modular and extensible. To contribute:

1. Follow the existing code patterns and documentation standards
2. Add comprehensive tests for new features
3. Update this documentation for any new components
4. Ensure backward compatibility with existing DataMesh functionality

## License

This governance system is part of the DataMesh project and follows the same licensing terms.