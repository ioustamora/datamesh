# DataMesh Storage Economy - Enhanced System

## Overview

DataMesh has implemented a comprehensive storage economy system that allows users to participate in the network through two primary methods:

### 1. Storage Contribution Model (4:1 Ratio)
- **Contribute 4x Storage**: Provide 4GB of storage space to earn 1GB of network usage
- **Continuous Verification**: Regular proof-of-space challenges ensure storage availability
- **Reputation System**: Build reputation through successful verifications
- **Bonus Rewards**: Earn additional credits for consistent performance

### 2. Premium Subscription Model
- **Pay for Storage**: Monthly subscription for guaranteed storage allocation
- **No Verification Required**: Immediate access without contribution requirements
- **Premium Features**: Higher bandwidth, priority support, enhanced features
- **Flexible Plans**: Scale storage based on needs

## Storage Tiers

### Free Tier
- **Storage**: 100MB
- **Upload Quota**: 1GB/month
- **Download Quota**: 2GB/month
- **Features**: Basic storage access
- **Cost**: Free
- **Limitations**: Limited bandwidth and features

### Contributor Tier
- **Storage**: Based on contribution (contributed_space / 4)
- **Upload Quota**: 2x earned storage
- **Download Quota**: 4x earned storage
- **Features**: 
  - Network storage access
  - Contribution rewards
  - Verification challenges
  - Reputation building
- **Requirements**: 
  - Minimum reputation score (75.0)
  - Regular verification challenges
  - Maintained storage contribution
- **Verification Types**:
  - Basic file tests
  - Random data generation
  - Merkle tree proofs
  - Time-lock puzzles
  - Sustained performance tests

### Premium Tier
- **Storage**: Based on subscription plan
- **Upload Quota**: 3x storage
- **Download Quota**: 5x storage
- **Features**:
  - Guaranteed storage allocation
  - Priority support
  - Enhanced backup redundancy
  - Premium features access
- **Cost**: $0.10/GB/month (configurable)
- **Benefits**: No verification required, immediate access

### Enterprise Tier
- **Storage**: Unlimited or very high limits
- **Upload/Download**: Unlimited
- **Features**:
  - Dedicated nodes
  - Custom replication factors
  - SLA guarantees
  - Compliance features
  - Priority support
  - Custom integrations
- **Cost**: Custom pricing

## Continuous Verification System

### Proof-of-Space Challenges
The system implements continuous verification to ensure contributors maintain their promised storage:

1. **Challenge Types**:
   - **BasicFileTest**: Simple file write/read verification
   - **RandomDataTest**: Generate and verify random data storage
   - **MerkleProof**: Cryptographic proof using Merkle trees
   - **TimeLockPuzzle**: Time-based verification challenges
   - **SustainedPerformance**: Performance consistency tests

2. **Verification Schedule**:
   - Default interval: 24 hours
   - Configurable timeout: 60 minutes
   - Maximum failures before penalization: 3
   - Consecutive challenge bonuses available

3. **Response Requirements**:
   - Minimum response time: 30 seconds
   - Maximum response time: 300 seconds
   - Cryptographic proof required
   - Storage accessibility verification

### Reputation System
- **Starting Reputation**: 75.0/100
- **Successful Verification**: +1-5 points (based on difficulty)
- **Failed Verification**: -5 points
- **Excessive Failures**: -20 points + penalties
- **Daily Decay**: -0.1 points (configurable)
- **Network Contribution Bonus**: Based on data served

### Bonus and Rewards System
- **Verification Streak Bonus**: 1MB per consecutive successful verification
- **Consistency Rewards**: Multiplier for sustained performance
- **Referral Credits**: Bonus for bringing new contributors
- **Network Participation**: Rewards for serving data to peers

## Commands Reference

### Basic Quota Management
```bash
# Show comprehensive storage overview
datamesh quota

# Show detailed usage information
datamesh quota --usage

# Show storage economy status
datamesh quota --economy

# Show tier information and upgrade options
datamesh quota --tier
```

### Storage Economy Management
```bash
# Show comprehensive economy status
datamesh economy

# Contribute storage (4:1 ratio)
datamesh economy --contribute --path /path/to/storage --amount 4GB

# Upgrade to premium
datamesh economy --upgrade --premium-size 100GB --payment-method card --duration 12

# Show detailed tier information
datamesh economy --tier-info

# Show network contribution statistics
datamesh economy --contribution-stats

# Show rewards and credits
datamesh economy --rewards

# Show available upgrade options
datamesh economy --upgrade-options
```

### Verification Management
```bash
# Show verification history
datamesh economy --verification-history

# Show current verification status
datamesh economy --verify

# Enable continuous monitoring
datamesh economy --enable-monitoring

# Disable continuous monitoring
datamesh economy --disable-monitoring

# Test verification challenge
datamesh economy --test-challenge

# Show proof-of-space details
datamesh economy --proof-details
```

### Challenge Response
```bash
# Respond to verification challenge
datamesh economy --challenge-response <response_data> --challenge-id <challenge_id>

# Show reputation score
datamesh economy --reputation
```

## Configuration

### Storage Economy Config (datamesh.toml)
```toml
[storage_economy]
# Contribution ratio (4:1 default)
contribution_ratio = 4.0

# Free tier limits
free_tier_storage = 104857600  # 100MB
free_tier_upload_quota = 1073741824  # 1GB/month
free_tier_download_quota = 2147483648  # 2GB/month

# Verification settings
verification_interval_hours = 24
verification_timeout_minutes = 60
max_failed_verifications = 3

# Enhanced verification
proof_of_space_enabled = true
continuous_verification = true
challenge_difficulty_levels = 5
min_verification_response_time = 30
max_verification_response_time = 300
verification_reward_multiplier = 1.1
verification_streak_bonus = 1048576  # 1MB per streak

# Pricing
premium_price_per_gb_month = 0.10
enterprise_multiplier = 2.0

# Reputation system
min_reputation_for_contributor = 75.0
reputation_decay_rate = 0.1
contribution_score_weight = 0.3
uptime_requirement = 90.0
data_serving_reward = 0.01
```

## Best Practices

### For Contributors
1. **Choose Reliable Storage**: Use stable, always-accessible storage paths
2. **Maintain Free Space**: Keep at least 10% buffer above contributed amount
3. **Monitor Verification**: Check status regularly and respond to challenges promptly
4. **Build Reputation**: Consistent verification success improves rewards
5. **Enable Monitoring**: Continuous verification provides better rewards

### For Premium Users
1. **Right-size Subscription**: Choose appropriate storage amounts for your needs
2. **Monitor Usage**: Track usage patterns to optimize subscription size
3. **Renew Timely**: Set reminders for subscription renewal
4. **Consider Hybrid**: Combine contribution with premium for optimal value

### Network Health
1. **Participate Actively**: Regular network participation improves overall health
2. **Serve Data**: Contributing nodes help distribute data efficiently
3. **Maintain Uptime**: High uptime improves network reliability
4. **Report Issues**: Help identify and resolve network problems

## Troubleshooting

### Common Issues
1. **Verification Failures**: Check storage path accessibility and free space
2. **Low Reputation**: Improve verification success rate and participate more
3. **Storage Limits**: Consider upgrading tier or contributing more storage
4. **Challenge Timeouts**: Ensure system has adequate resources for responses

### Recovery Procedures
1. **Failed Verifications**: Address underlying storage issues and wait for next challenge
2. **Reputation Recovery**: Consistent successful verifications restore reputation
3. **Storage Migration**: Update contribution path if storage location changes
4. **Subscription Issues**: Contact support for payment or billing problems

## Future Enhancements

- **Multi-tier Contributions**: Support different contribution levels
- **Geographic Distribution**: Location-based storage optimization
- **Smart Contracts**: Blockchain-based verification and payments
- **Advanced Analytics**: Detailed performance and contribution tracking
- **Mobile Verification**: Smartphone-based verification responses
- **Automated Management**: AI-driven optimization of storage contributions
