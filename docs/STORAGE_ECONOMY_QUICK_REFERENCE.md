# DataMesh Storage Economy - Quick Reference

## 🏁 Getting Started

### Check Your Current Status
```bash
datamesh quota                    # Complete overview
datamesh economy                  # Economy status
```

## 💾 Storage Contribution (4:1 Ratio)

### Become a Contributor
```bash
# Contribute 4GB to earn 1GB of network storage
datamesh economy --contribute --path /path/to/storage --amount 4GB
```

### Monitor Your Contribution
```bash
datamesh economy --tier-info              # Detailed tier information
datamesh economy --verification-history   # Your verification record
datamesh economy --test-challenge         # Test your setup
```

### Manage Verification
```bash
datamesh economy --enable-monitoring      # Auto verification
datamesh economy --disable-monitoring     # Manual verification
datamesh economy --verify                 # Check current status
```

## ⭐ Premium Upgrade

### Upgrade to Premium
```bash
# Get guaranteed storage without verification
datamesh economy --upgrade --premium-size 100GB --payment-method card --duration 12
```

### Check Upgrade Options
```bash
datamesh economy --upgrade-options        # See available upgrades
```

## 📊 Monitoring & Information

### Usage Tracking
```bash
datamesh quota --usage                    # Detailed usage stats
datamesh quota --economy                  # Economy overview
datamesh quota --tier                     # Tier information
```

### Performance & Rewards
```bash
datamesh economy --contribution-stats     # Network contribution
datamesh economy --rewards                # Credits and bonuses
datamesh economy --reputation             # Reputation score
datamesh economy --proof-details          # Proof-of-space info
```

## 🎯 Storage Tiers

| Tier | Storage | Upload Quota | Download Quota | Cost | Requirements |
|------|---------|--------------|----------------|------|--------------|
| **Free** | 100MB | 1GB/month | 2GB/month | Free | None |
| **Contributor** | Earned (4:1) | 2x earned | 4x earned | Storage space | Verification |
| **Premium** | Paid amount | 3x storage | 5x storage | $0.10/GB/month | Payment |
| **Enterprise** | Unlimited | Unlimited | Unlimited | Custom | Contract |

## 🔄 Verification System

### Challenge Types
- **BasicFileTest**: Simple read/write verification
- **RandomDataTest**: Random data generation and storage
- **MerkleProof**: Cryptographic proof construction
- **TimeLockPuzzle**: Time-based verification challenges
- **SustainedPerformance**: Performance consistency tests

### Success Tips
1. **Reliable Storage**: Use always-accessible storage paths
2. **Adequate Space**: Maintain 10% buffer above contribution
3. **Quick Response**: Respond to challenges within timeout
4. **Consistent Performance**: Build verification streak for bonuses
5. **Monitor Regularly**: Check status and resolve issues promptly

## 🎁 Rewards System

### Earn Bonus Credits
- **Verification Streak**: +1MB per consecutive success
- **Performance Bonus**: +reputation for consistent verification
- **Network Participation**: +credits for serving data
- **Referral Rewards**: +credits for new contributor referrals

### Reputation Building
- Start with 75/100 reputation
- +1-5 points per successful verification
- -5 points per failed verification
- -20 points for excessive failures
- Daily decay: -0.1 points

## ⚠️ Troubleshooting

### Common Issues
| Problem | Solution |
|---------|----------|
| Verification fails | Check storage path accessibility |
| Low reputation | Improve verification success rate |
| Storage full | Upgrade tier or contribute more |
| Challenge timeout | Ensure adequate system resources |

### Recovery Steps
1. **Fix Storage Issues**: Ensure path is accessible with enough space
2. **Wait for Next Challenge**: System automatically retries
3. **Build Streak**: Consistent success restores reputation
4. **Contact Support**: For persistent issues

## 🚀 Quick Actions

### Daily Commands
```bash
datamesh quota                            # Check your status
datamesh economy --verify                 # Verification status
```

### Weekly Commands
```bash
datamesh economy --verification-history   # Review performance
datamesh economy --rewards                # Check accumulated rewards
```

### Monthly Commands
```bash
datamesh economy --contribution-stats     # Network impact
datamesh economy --upgrade-options        # Consider upgrades
```

## 📱 Mobile-Friendly Commands

### Short Status Check
```bash
datamesh quota | head -20                 # Quick overview
```

### Key Metrics Only
```bash
datamesh economy --reputation             # Just reputation
datamesh economy --rewards | grep Credits # Just credits
```

## 🔗 Related Commands

### File Operations
```bash
datamesh put <file>                       # Store files
datamesh get <file>                       # Retrieve files
datamesh list                             # List stored files
```

### Network Operations
```bash
datamesh peers                            # Show network peers
datamesh health                           # Network health
```

### Maintenance
```bash
datamesh cleanup                          # Clean up storage
datamesh repair                           # Repair damaged files
```

---

💡 **Pro Tip**: Set up aliases for frequently used commands:
```bash
alias dq="datamesh quota"
alias de="datamesh economy"
alias dev="datamesh economy --verify"
alias der="datamesh economy --rewards"
```
