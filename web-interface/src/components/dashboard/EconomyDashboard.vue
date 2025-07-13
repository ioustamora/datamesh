<template>
  <div class="economy-dashboard">
    <!-- Header -->
    <div class="dashboard-header">
      <h2 class="dashboard-title">
        <el-icon><Wallet /></el-icon>
        Storage Economy
      </h2>
      <div class="header-actions">
        <el-button @click="refreshData" :loading="isRefreshing" size="small" type="primary">
          <el-icon><Refresh /></el-icon>
          Refresh
        </el-button>
      </div>
    </div>

    <!-- Quick Stats -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon storage-icon">
          <el-icon><FolderOpened /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatStorageSize(userProfile.current_usage) }}</div>
          <div class="stat-label">Storage Used</div>
          <div class="stat-progress">
            <el-progress 
              :percentage="storageUsagePercentage" 
              :status="storageUsagePercentage > 85 ? 'exception' : 'success'"
              :stroke-width="6"
              :show-text="false"
            />
          </div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon tier-icon" :style="{ backgroundColor: getTierColor(userProfile.tier) }">
          <el-icon><Medal /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ userProfile.tier }}</div>
          <div class="stat-label">Current Tier</div>
          <div class="stat-meta">{{ formatStorageSize(userProfile.max_storage) }} limit</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon reputation-icon">
          <el-icon><Star /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ userProfile.reputation_score.toFixed(1) }}</div>
          <div class="stat-label">Reputation</div>
          <div class="stat-meta" :class="{ 'text-success': userProfile.can_contribute }">
            {{ userProfile.can_contribute ? 'Can Contribute' : 'Cannot Contribute' }}
          </div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon contribution-icon">
          <el-icon><Share /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatStorageSize(contributionStatus.contributed_amount) }}</div>
          <div class="stat-label">Contributing</div>
          <div class="stat-meta" :class="contributionStatus.active ? 'text-success' : 'text-muted'">
            {{ contributionStatus.active ? 'Active' : 'Inactive' }}
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="content-grid">
      <!-- Storage Quota Panel -->
      <el-card class="quota-panel">
        <template #header>
          <div class="card-header">
            <h3>Storage Quotas</h3>
            <el-button size="small" @click="showQuotaDetails = !showQuotaDetails">
              {{ showQuotaDetails ? 'Hide Details' : 'Show Details' }}
            </el-button>
          </div>
        </template>

        <div class="quota-overview">
          <!-- Storage Usage -->
          <div class="quota-item">
            <div class="quota-header">
              <span class="quota-label">Storage Usage</span>
              <span class="quota-value">
                {{ formatStorageSize(userProfile.current_usage) }} / {{ formatStorageSize(userProfile.max_storage) }}
              </span>
            </div>
            <el-progress 
              :percentage="storageUsagePercentage"
              :status="storageUsagePercentage > 85 ? 'exception' : 'success'"
            >
              <template #default="{ percentage }">
                <span :class="{ 'text-warning': percentage > 85 }">{{ percentage }}%</span>
              </template>
            </el-progress>
          </div>

          <!-- Upload Quota -->
          <div class="quota-item">
            <div class="quota-header">
              <span class="quota-label">Monthly Upload</span>
              <span class="quota-value">
                {{ formatStorageSize(userProfile.upload_quota_used) }} / {{ formatStorageSize(userProfile.upload_quota_limit) }}
              </span>
            </div>
            <el-progress 
              :percentage="uploadQuotaPercentage"
              :status="uploadQuotaPercentage > 85 ? 'exception' : 'success'"
            >
              <template #default="{ percentage }">
                <span :class="{ 'text-warning': percentage > 85 }">{{ percentage }}%</span>
              </template>
            </el-progress>
          </div>

          <!-- Download Quota -->
          <div class="quota-item">
            <div class="quota-header">
              <span class="quota-label">Monthly Download</span>
              <span class="quota-value">
                {{ formatStorageSize(userProfile.download_quota_used) }} / {{ formatStorageSize(userProfile.download_quota_limit) }}
              </span>
            </div>
            <el-progress 
              :percentage="downloadQuotaPercentage"
              :status="downloadQuotaPercentage > 85 ? 'exception' : 'success'"
            >
              <template #default="{ percentage }">
                <span :class="{ 'text-warning': percentage > 85 }">{{ percentage }}%</span>
              </template>
            </el-progress>
          </div>
        </div>

        <!-- Quota Details -->
        <div v-if="showQuotaDetails" class="quota-details">
          <el-divider />
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">Quota Reset:</span>
              <span class="detail-value">{{ formatDate(quotaStatus.next_reset) }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Violations:</span>
              <span class="detail-value" :class="{ 'text-warning': userProfile.violations_count > 0 }">
                {{ userProfile.violations_count }}
              </span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Last Activity:</span>
              <span class="detail-value">{{ formatDate(userProfile.last_activity) }}</span>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Tier Management Panel -->
      <el-card class="tier-panel">
        <template #header>
          <h3>Storage Tier</h3>
        </template>

        <div class="current-tier">
          <div class="tier-badge" :style="{ backgroundColor: getTierColor(userProfile.tier) }">
            <el-icon><Medal /></el-icon>
            {{ userProfile.tier }}
          </div>
          <div class="tier-benefits">
            <div class="benefit-item">
              <el-icon><FolderOpened /></el-icon>
              {{ formatStorageSize(userProfile.max_storage) }} Storage
            </div>
            <div class="benefit-item">
              <el-icon><Upload /></el-icon>
              {{ formatStorageSize(userProfile.upload_quota_limit) }} Monthly Upload
            </div>
            <div class="benefit-item">
              <el-icon><Download /></el-icon>
              {{ formatStorageSize(userProfile.download_quota_limit) }} Monthly Download
            </div>
          </div>
        </div>

        <!-- Upgrade Options -->
        <div v-if="canUpgradeTier" class="upgrade-section">
          <el-divider />
          <h4>Available Upgrades</h4>
          <div class="tier-options">
            <div 
              v-for="tier in availableUpgrades" 
              :key="tier.name"
              class="tier-option"
              @click="selectTierForUpgrade(tier)"
            >
              <div class="tier-option-header">
                <span class="tier-name" :style="{ color: getTierColor(tier.name) }">{{ tier.name }}</span>
                <span v-if="tier.monthly_cost" class="tier-cost">${{ tier.monthly_cost }}/month</span>
                <span v-else class="tier-cost free">Free</span>
              </div>
              <div class="tier-description">{{ tier.description }}</div>
              <div class="tier-specs">
                <span>{{ formatStorageSize(tier.max_storage) }}</span> • 
                <span>{{ formatStorageSize(tier.upload_quota) }} upload</span> • 
                <span>{{ formatStorageSize(tier.download_quota) }} download</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- Contribution Panel -->
      <el-card class="contribution-panel">
        <template #header>
          <div class="card-header">
            <h3>Storage Contribution</h3>
            <el-switch
              v-model="contributionStatus.active"
              @change="toggleContribution"
              :loading="loading.contributing"
              active-text="Active"
              inactive-text="Inactive"
            />
          </div>
        </template>

        <div v-if="contributionStatus.active" class="contribution-active">
          <div class="contribution-stats">
            <div class="contrib-stat">
              <span class="contrib-label">Contributing:</span>
              <span class="contrib-value">{{ formatStorageSize(contributionStatus.contributed_amount) }}</span>
            </div>
            <div class="contrib-stat">
              <span class="contrib-label">Verified:</span>
              <span class="contrib-value">{{ formatStorageSize(contributionStatus.verified_amount) }}</span>
            </div>
            <div class="contrib-stat">
              <span class="contrib-label">Last Verification:</span>
              <span class="contrib-value">{{ formatDate(contributionStatus.last_verification) }}</span>
            </div>
          </div>
          
          <el-progress 
            :percentage="verificationPercentage"
            :status="verificationPercentage === 100 ? 'success' : 'active'"
            class="verification-progress"
          >
            <template #default="{ percentage }">
              {{ percentage }}% Verified
            </template>
          </el-progress>
        </div>

        <div v-else class="contribution-inactive">
          <div class="inactive-message">
            <el-icon><Warning /></el-icon>
            <p>Storage contribution is currently inactive.</p>
            <p class="help-text">
              Enable contribution to earn additional storage space and improve your reputation score.
            </p>
          </div>
          
          <el-button 
            @click="showContributionSetup = true" 
            type="primary"
            :disabled="!userProfile.can_contribute"
            class="setup-button"
          >
            <el-icon><Plus /></el-icon>
            Set Up Contribution
          </el-button>
          
          <div v-if="!userProfile.can_contribute" class="cannot-contribute">
            <el-alert
              title="Cannot Contribute"
              type="warning"
              :closable="false"
              show-icon
            >
              <template #default>
                Your reputation score is too low to contribute storage. 
                Improve your reputation by using the service responsibly.
              </template>
            </el-alert>
          </div>
        </div>
      </el-card>

      <!-- Verification Status Panel -->
      <el-card class="verification-panel">
        <template #header>
          <h3>Verification Status</h3>
        </template>

        <div class="verification-overview">
          <div class="verification-stat">
            <span class="verification-label">Success Rate:</span>
            <span class="verification-value" :class="getSuccessRateClass(verificationStatus.success_rate)">
              {{ (verificationStatus.success_rate * 100).toFixed(1) }}%
            </span>
          </div>
          
          <div class="verification-stat">
            <span class="verification-label">Pending Challenges:</span>
            <span class="verification-value">{{ verificationStatus.pending_challenges }}</span>
          </div>
          
          <div class="verification-stat">
            <span class="verification-label">Last Challenge:</span>
            <span class="verification-value">{{ formatDate(verificationStatus.last_challenge) }}</span>
          </div>
        </div>

        <div v-if="verificationStatus.pending_challenges > 0" class="pending-challenges">
          <el-alert
            title="Pending Challenges"
            type="info"
            show-icon
            :closable="false"
          >
            <template #default>
              You have {{ verificationStatus.pending_challenges }} pending verification challenge(s).
              Please respond to maintain your contribution status.
            </template>
          </el-alert>
          
          <el-button @click="showChallengeDialog = true" type="primary" class="challenge-button">
            <el-icon><Document /></el-icon>
            View Challenges
          </el-button>
        </div>
      </el-card>

      <!-- Recent Transactions Panel -->
      <el-card class="transactions-panel">
        <template #header>
          <div class="card-header">
            <h3>Recent Transactions</h3>
            <el-button size="small" @click="showAllTransactions = true">
              View All
            </el-button>
          </div>
        </template>

        <div v-if="transactions.length > 0" class="transactions-list">
          <div 
            v-for="transaction in recentTransactions" 
            :key="transaction.transaction_id"
            class="transaction-item"
          >
            <div class="transaction-icon" :class="getTransactionIconClass(transaction.transaction_type)">
              <el-icon v-if="transaction.transaction_type === 'contribution'"><Share /></el-icon>
              <el-icon v-else-if="transaction.transaction_type === 'upgrade'"><TrendCharts /></el-icon>
              <el-icon v-else-if="transaction.transaction_type === 'verification'"><Document /></el-icon>
              <el-icon v-else><Coin /></el-icon>
            </div>
            <div class="transaction-content">
              <div class="transaction-description">{{ transaction.description }}</div>
              <div class="transaction-meta">
                <span class="transaction-date">{{ formatDate(transaction.timestamp) }}</span>
                <span class="transaction-status" :class="getStatusClass(transaction.status)">
                  {{ transaction.status }}
                </span>
              </div>
            </div>
            <div class="transaction-amount">
              {{ formatStorageSize(transaction.amount) }}
            </div>
          </div>
        </div>

        <div v-else class="no-transactions">
          <el-icon><Document /></el-icon>
          <p>No recent transactions</p>
        </div>
      </el-card>
    </div>

    <!-- Contribution Setup Dialog -->
    <ContributionSetupDialog 
      v-model="showContributionSetup"
      @success="onContributionSetupSuccess"
    />

    <!-- Tier Upgrade Dialog -->
    <TierUpgradeDialog
      v-model="showTierUpgrade"
      :selected-tier="selectedTier"
      @success="onTierUpgradeSuccess"
    />

    <!-- Challenge Response Dialog -->
    <ChallengeResponseDialog
      v-model="showChallengeDialog"
      @success="onChallengeResponseSuccess"
    />

    <!-- All Transactions Dialog -->
    <AllTransactionsDialog
      v-model="showAllTransactions"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useEconomyStore } from '../../store/economy'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Wallet, Refresh, FolderOpened, Medal, Star, Share, Upload, Download,
  Warning, Plus, Document, TrendCharts, Coin
} from '@element-plus/icons-vue'

// Import child components
import ContributionSetupDialog from './ContributionSetupDialog.vue'
import TierUpgradeDialog from './TierUpgradeDialog.vue'
import ChallengeResponseDialog from './ChallengeResponseDialog.vue'
import AllTransactionsDialog from './AllTransactionsDialog.vue'

// Store
const economyStore = useEconomyStore()
const {
  economyStatus,
  userProfile,
  storageTiers,
  contributionStatus,
  verificationStatus,
  transactions,
  quotaStatus,
  loading,
  errors
} = storeToRefs(economyStore)

// Computed properties from store
const {
  storageUsagePercentage,
  uploadQuotaPercentage,
  downloadQuotaPercentage,
  canUpgradeTier,
  nextTier,
  formatStorageSize,
  getTierColor,
  isStorageLow,
  isQuotaLow
} = economyStore

// Local reactive state
const showQuotaDetails = ref(false)
const showContributionSetup = ref(false)
const showTierUpgrade = ref(false)
const showChallengeDialog = ref(false)
const showAllTransactions = ref(false)
const selectedTier = ref(null)
const isRefreshing = ref(false)

// Auto-refresh interval
let refreshInterval = null

// Computed properties
const verificationPercentage = computed(() => {
  if (contributionStatus.value.contributed_amount === 0) return 0
  return Math.round((contributionStatus.value.verified_amount / contributionStatus.value.contributed_amount) * 100)
})

const availableUpgrades = computed(() => {
  const currentTierIndex = storageTiers.value.findIndex(tier => tier.name === userProfile.value.tier)
  return storageTiers.value.slice(currentTierIndex + 1)
})

const recentTransactions = computed(() => {
  return transactions.value.slice(0, 5)
})

// Methods
const refreshData = async () => {
  isRefreshing.value = true
  try {
    await economyStore.refreshData()
    ElMessage.success('Data refreshed successfully')
  } catch (error) {
    ElMessage.error('Failed to refresh data')
  } finally {
    isRefreshing.value = false
  }
}

const toggleContribution = async (active) => {
  try {
    if (active) {
      showContributionSetup.value = true
    } else {
      await economyStore.stopContribution()
      ElMessage.success('Storage contribution stopped')
    }
  } catch (error) {
    ElMessage.error('Failed to toggle contribution')
    // Revert the switch state
    contributionStatus.value.active = !active
  }
}

const selectTierForUpgrade = (tier) => {
  selectedTier.value = tier
  showTierUpgrade.value = true
}

const onContributionSetupSuccess = () => {
  showContributionSetup.value = false
  ElMessage.success('Storage contribution started successfully')
}

const onTierUpgradeSuccess = () => {
  showTierUpgrade.value = false
  selectedTier.value = null
  ElMessage.success('Tier upgrade initiated successfully')
}

const onChallengeResponseSuccess = () => {
  showChallengeDialog.value = false
  ElMessage.success('Challenge response submitted successfully')
}

const formatDate = (dateString) => {
  if (!dateString) return 'Never'
  const date = new Date(dateString)
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString()
}

const getSuccessRateClass = (rate) => {
  if (rate >= 0.9) return 'text-success'
  if (rate >= 0.7) return 'text-warning'
  return 'text-danger'
}

const getTransactionIconClass = (type) => {
  const classes = {
    'contribution': 'contribution-icon',
    'upgrade': 'upgrade-icon',
    'verification': 'verification-icon',
    'default': 'default-icon'
  }
  return classes[type] || classes.default
}

const getStatusClass = (status) => {
  const classes = {
    'completed': 'text-success',
    'pending': 'text-warning',
    'failed': 'text-danger',
    'cancelled': 'text-muted'
  }
  return classes[status] || 'text-muted'
}

// Lifecycle hooks
onMounted(async () => {
  try {
    await economyStore.initializeEconomyData()
    
    // Set up auto-refresh every 30 seconds
    refreshInterval = setInterval(() => {
      economyStore.refreshData()
    }, 30000)
  } catch (error) {
    console.error('Failed to initialize economy dashboard:', error)
    ElMessage.error('Failed to load economy data')
  }
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<style scoped>
.economy-dashboard {
  padding: 24px;
  background-color: #f5f7fa;
  min-height: calc(100vh - 60px);
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.dashboard-title {
  margin: 0;
  color: #303133;
  font-size: 28px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.stat-card {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  display: flex;
  align-items: center;
  gap: 16px;
  transition: transform 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 24px;
}

.storage-icon { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }
.tier-icon { background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); }
.reputation-icon { background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%); }
.contribution-icon { background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%); }

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  line-height: 1.2;
}

.stat-label {
  color: #909399;
  font-size: 14px;
  margin-bottom: 8px;
}

.stat-meta {
  color: #606266;
  font-size: 12px;
  margin-top: 4px;
}

.stat-progress {
  margin-top: 8px;
}

.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
}

.el-card {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  border: none;
  border-radius: 12px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: #303133;
  font-size: 18px;
  font-weight: 600;
}

.quota-item {
  margin-bottom: 20px;
}

.quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.quota-label {
  color: #606266;
  font-weight: 500;
}

.quota-value {
  color: #303133;
  font-weight: 600;
  font-size: 14px;
}

.quota-details {
  margin-top: 16px;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.detail-label {
  color: #909399;
  font-size: 14px;
}

.detail-value {
  color: #303133;
  font-weight: 500;
  font-size: 14px;
}

.current-tier {
  text-align: center;
  margin-bottom: 20px;
}

.tier-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border-radius: 20px;
  color: white;
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 16px;
}

.tier-benefits {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.benefit-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #606266;
  font-size: 14px;
}

.tier-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tier-option {
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tier-option:hover {
  border-color: #409eff;
  background-color: #f0f9ff;
}

.tier-option-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.tier-name {
  font-weight: 600;
  font-size: 16px;
}

.tier-cost {
  font-weight: 600;
  color: #409eff;
}

.tier-cost.free {
  color: #67c23a;
}

.tier-description {
  color: #606266;
  font-size: 14px;
  margin-bottom: 8px;
}

.tier-specs {
  color: #909399;
  font-size: 12px;
}

.contribution-active {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.contribution-stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.contrib-stat {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.contrib-label {
  color: #606266;
  font-size: 14px;
}

.contrib-value {
  color: #303133;
  font-weight: 500;
}

.contribution-inactive {
  text-align: center;
}

.inactive-message {
  margin-bottom: 20px;
}

.inactive-message .el-icon {
  font-size: 48px;
  color: #e6a23c;
  margin-bottom: 12px;
}

.inactive-message p {
  margin: 8px 0;
  color: #606266;
}

.help-text {
  font-size: 14px;
  color: #909399;
}

.setup-button {
  margin-bottom: 16px;
}

.verification-overview {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.verification-stat {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.verification-label {
  color: #606266;
  font-size: 14px;
}

.verification-value {
  font-weight: 600;
}

.pending-challenges {
  margin-top: 16px;
}

.challenge-button {
  margin-top: 12px;
  width: 100%;
}

.transactions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.transaction-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  transition: background-color 0.2s ease;
}

.transaction-item:hover {
  background-color: #f9f9f9;
}

.transaction-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.transaction-icon.contribution-icon { background-color: #67c23a; }
.transaction-icon.upgrade-icon { background-color: #409eff; }
.transaction-icon.verification-icon { background-color: #e6a23c; }
.transaction-icon.default-icon { background-color: #909399; }

.transaction-content {
  flex: 1;
}

.transaction-description {
  font-weight: 500;
  color: #303133;
  font-size: 14px;
}

.transaction-meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
}

.transaction-date {
  color: #909399;
  font-size: 12px;
}

.transaction-status {
  font-size: 12px;
  font-weight: 500;
}

.transaction-amount {
  font-weight: 600;
  color: #303133;
}

.no-transactions {
  text-align: center;
  color: #909399;
  padding: 32px;
}

.no-transactions .el-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

/* Utility classes */
.text-success { color: #67c23a; }
.text-warning { color: #e6a23c; }
.text-danger { color: #f56c6c; }
.text-muted { color: #909399; }

/* Responsive design */
@media (max-width: 768px) {
  .economy-dashboard {
    padding: 16px;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
  
  .content-grid {
    grid-template-columns: 1fr;
  }
  
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>