<template>
  <div class="intelligent-dashboard">
    <!-- Header with dynamic greeting -->
    <div class="dashboard-header">
      <div class="user-greeting">
        <h1>{{ dynamicGreeting }}</h1>
        <p class="user-status">{{ userStatus }}</p>
      </div>
      <div class="quick-stats">
        <div class="stat-card" v-for="stat in quickStats" :key="stat.id">
          <div class="stat-icon">{{ stat.icon }}</div>
          <div class="stat-content">
            <span class="stat-value">{{ stat.value }}</span>
            <span class="stat-label">{{ stat.label }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Predictive Insights Panel -->
    <el-card class="insights-panel" shadow="hover">
      <template #header>
        <div class="insights-header">
          <h3>
            <el-icon><TrendCharts /></el-icon>
            Intelligent Insights
          </h3>
          <el-tag :type="insightReliability.type" size="small">
            {{ insightReliability.label }}
          </el-tag>
        </div>
      </template>
      
      <div class="insights-content">
        <div v-if="prioritizedInsights.length === 0" class="no-insights">
          <el-icon class="no-insights-icon"><InfoFilled /></el-icon>
          <p>Analyzing your usage patterns...</p>
        </div>
        
        <PredictiveInsight 
          v-for="insight in prioritizedInsights"
          :key="insight.id"
          :insight="insight"
          @action="handleInsightAction"
          @dismiss="dismissInsight"
        />
      </div>
    </el-card>

    <!-- Real-time usage visualization -->
    <el-row :gutter="20">
      <el-col :span="12">
        <el-card class="usage-card" shadow="hover">
          <template #header>
            <h3>
              <el-icon><PieChart /></el-icon>
              Storage Usage
            </h3>
          </template>
          
          <div class="usage-ring-chart">
            <div class="usage-progress">
              <el-progress 
                type="circle" 
                :percentage="storageUsagePercentage"
                :width="120"
                :stroke-width="8"
                :color="storageProgressColor"
              >
                <template #default="{ percentage }">
                  <span class="percentage-text">{{ percentage }}%</span>
                </template>
              </el-progress>
            </div>
            
            <div class="usage-breakdown">
              <div class="usage-item">
                <span class="usage-dot" :style="{ backgroundColor: '#409EFF' }"></span>
                <span class="label">Used</span>
                <span class="value">{{ formatBytes(storageUsed) }}</span>
              </div>
              <div class="usage-item">
                <span class="usage-dot" :style="{ backgroundColor: '#67C23A' }"></span>
                <span class="label">Available</span>
                <span class="value">{{ formatBytes(storageAvailable) }}</span>
              </div>
              <div class="usage-item">
                <span class="usage-dot" :style="{ backgroundColor: '#E6A23C' }"></span>
                <span class="label">Contributed</span>
                <span class="value">{{ formatBytes(contributedStorage) }}</span>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="12">
        <el-card class="economy-card" shadow="hover">
          <template #header>
            <h3>
              <el-icon><Money /></el-icon>
              Economy Overview
            </h3>
          </template>
          
          <div class="economy-stats">
            <div class="economy-item">
              <div class="economy-icon">💰</div>
              <div class="economy-content">
                <span class="economy-value">${{ currentBill.toFixed(2) }}</span>
                <span class="economy-label">Current Bill</span>
              </div>
            </div>
            
            <div class="economy-item">
              <div class="economy-icon">🏆</div>
              <div class="economy-content">
                <span class="economy-value">{{ earnedTokens }}</span>
                <span class="economy-label">Earned Tokens</span>
              </div>
            </div>
            
            <div class="economy-item">
              <div class="economy-icon">📊</div>
              <div class="economy-content">
                <span class="economy-value">{{ reputationScore }}/100</span>
                <span class="economy-label">Reputation</span>
              </div>
            </div>
          </div>
          
          <div class="tier-info">
            <div class="current-tier">
              <span class="tier-badge" :class="currentTier.toLowerCase()">
                {{ currentTier }}
              </span>
              <span class="tier-benefits">{{ tierBenefits }}</span>
            </div>
            
            <el-button 
              type="primary" 
              size="small" 
              @click="showTierOptimization"
              :icon="Upgrade"
            >
              Optimize Tier
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Smart Alerts with predictive analysis -->
    <div class="smart-alerts" v-if="smartAlerts.length > 0">
      <TransitionGroup name="alert" tag="div">
        <SmartAlert 
          v-for="alert in smartAlerts"
          :key="alert.id"
          :alert="alert"
          @action="handleAlertAction"
          @dismiss="dismissAlert"
        />
      </TransitionGroup>
    </div>

    <!-- Adaptive Quick Actions -->
    <el-card class="adaptive-actions" shadow="hover">
      <template #header>
        <h3>
          <el-icon><Lightning /></el-icon>
          Recommended Actions
        </h3>
      </template>
      
      <SmartActionGrid 
        :actions="adaptiveActions"
        :user-context="userContext"
        @execute="executeAction"
      />
    </el-card>

    <!-- Contribution rewards tracker -->
    <el-card class="contribution-rewards" shadow="hover">
      <template #header>
        <h3>
          <el-icon><Trophy /></el-icon>
          Contribution Progress
        </h3>
      </template>
      
      <div class="reward-progress">
        <div class="progress-header">
          <span class="progress-label">Next Reward</span>
          <span class="progress-tokens">{{ nextRewardAmount }} tokens</span>
        </div>
        
        <el-progress 
          :percentage="contributionProgress"
          :stroke-width="8"
          :color="progressColor"
        >
          <template #default="{ percentage }">
            <span class="progress-text">{{ percentage }}%</span>
          </template>
        </el-progress>
        
        <div class="progress-milestones">
          <div 
            v-for="milestone in contributionMilestones"
            :key="milestone.id"
            class="milestone"
            :class="{ 'achieved': milestone.achieved }"
          >
            <div class="milestone-icon">{{ milestone.icon }}</div>
            <div class="milestone-content">
              <span class="milestone-name">{{ milestone.name }}</span>
              <span class="milestone-reward">{{ milestone.reward }}</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Gamification Elements -->
    <el-card class="gamification-card" shadow="hover">
      <template #header>
        <h3>
          <el-icon><Medal /></el-icon>
          Your Achievements
        </h3>
      </template>
      
      <div class="achievement-grid">
        <div 
          v-for="achievement in recentAchievements"
          :key="achievement.id"
          class="achievement-card"
          :class="{ 'unlocked': achievement.unlocked }"
        >
          <div class="achievement-icon">{{ achievement.icon }}</div>
          <div class="achievement-content">
            <h4>{{ achievement.name }}</h4>
            <p>{{ achievement.description }}</p>
            <div class="achievement-progress">
              <el-progress 
                :percentage="achievement.progress"
                :stroke-width="4"
                :show-text="false"
                :color="achievement.unlocked ? '#67C23A' : '#409EFF'"
              />
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Proactive Notifications -->
    <ProactiveNotificationCenter 
      :notifications="intelligentNotifications"
      @dismiss="dismissNotification"
      @act="actOnNotification"
    />

    <!-- Dialogs -->
    <TierOptimizationDialog 
      v-model="showTierDialog"
      :current-tier="currentTier"
      :usage-data="usageData"
      :recommendations="tierRecommendations"
      @upgrade="handleTierUpgrade"
      @downgrade="handleTierDowngrade"
    />
  </div>
</template>

<script>
import { ref, computed, onMounted, watch } from 'vue'
import { useUserStore } from '@/stores/user'
import { useEconomyStore } from '@/stores/economy'
import { useGameStore } from '@/stores/gamification'
import { useIntelligentDashboard } from '@/composables/useIntelligentDashboard'
import { useSmartAlerts } from '@/composables/useSmartAlerts'
import { formatBytes, formatCurrency } from '@/utils/formatters'
import { 
  TrendCharts, 
  InfoFilled, 
  PieChart, 
  Money, 
  Lightning, 
  Trophy, 
  Medal, 
  Upgrade 
} from '@element-plus/icons-vue'

// Components
import PredictiveInsight from '@/components/dashboard/PredictiveInsight.vue'
import SmartActionGrid from '@/components/dashboard/SmartActionGrid.vue'
import SmartAlert from '@/components/dashboard/SmartAlert.vue'
import ProactiveNotificationCenter from '@/components/dashboard/ProactiveNotificationCenter.vue'
import TierOptimizationDialog from '@/components/dashboard/TierOptimizationDialog.vue'

export default {
  name: 'IntelligentDashboard',
  components: {
    PredictiveInsight,
    SmartActionGrid,
    SmartAlert,
    ProactiveNotificationCenter,
    TierOptimizationDialog,
    TrendCharts,
    InfoFilled,
    PieChart,
    Money,
    Lightning,
    Trophy,
    Medal,
    Upgrade
  },
  setup() {
    const userStore = useUserStore()
    const economyStore = useEconomyStore()
    const gameStore = useGameStore()
    
    const {
      prioritizedInsights,
      adaptiveActions,
      intelligentNotifications,
      userContext,
      insightReliability,
      analyzeUserBehavior,
      generatePredictions,
      personalizeInterface
    } = useIntelligentDashboard()
    
    const {
      smartAlerts,
      addAlert,
      dismissAlert,
      handleAlertAction
    } = useSmartAlerts()

    // Reactive state
    const showTierDialog = ref(false)
    const usageData = ref({})
    const tierRecommendations = ref([])

    // Computed properties
    const dynamicGreeting = computed(() => {
      const hour = new Date().getHours()
      const name = userStore.user?.name || 'User'
      
      if (hour < 12) {
        return `Good morning, ${name}! 🌅`
      } else if (hour < 18) {
        return `Good afternoon, ${name}! ☀️`
      } else {
        return `Good evening, ${name}! 🌙`
      }
    })

    const userStatus = computed(() => {
      const tier = economyStore.currentTier
      const reputation = gameStore.reputation
      
      return `${tier} tier • ${reputation}/100 reputation`
    })

    const quickStats = computed(() => [
      {
        id: 'files',
        icon: '📁',
        value: userStore.fileCount,
        label: 'Files'
      },
      {
        id: 'storage',
        icon: '💾',
        value: formatBytes(userStore.storageUsed),
        label: 'Storage Used'
      },
      {
        id: 'tokens',
        icon: '🪙',
        value: gameStore.tokens,
        label: 'Tokens'
      },
      {
        id: 'level',
        icon: '🏆',
        value: gameStore.level,
        label: 'Level'
      }
    ])

    const storageUsagePercentage = computed(() => {
      if (userStore.storageLimit === 0) return 0
      return Math.round((userStore.storageUsed / userStore.storageLimit) * 100)
    })

    const storageProgressColor = computed(() => {
      const percentage = storageUsagePercentage.value
      if (percentage < 60) return '#67C23A'
      if (percentage < 80) return '#E6A23C'
      return '#F56C6C'
    })

    const storageUsed = computed(() => userStore.storageUsed)
    const storageAvailable = computed(() => userStore.storageLimit - userStore.storageUsed)
    const contributedStorage = computed(() => userStore.contributedStorage)

    const currentBill = computed(() => economyStore.currentBill)
    const earnedTokens = computed(() => gameStore.tokens)
    const reputationScore = computed(() => gameStore.reputation)
    const currentTier = computed(() => economyStore.currentTier)

    const tierBenefits = computed(() => {
      const benefits = economyStore.tierBenefits
      return benefits.slice(0, 2).join(', ')
    })

    const contributionProgress = computed(() => {
      return Math.min(100, (userStore.contributedStorage / userStore.nextRewardThreshold) * 100)
    })

    const nextRewardAmount = computed(() => {
      return Math.floor(userStore.nextRewardThreshold / 1000000) // Convert to tokens
    })

    const progressColor = computed(() => {
      const progress = contributionProgress.value
      if (progress < 50) return '#409EFF'
      if (progress < 80) return '#E6A23C'
      return '#67C23A'
    })

    const contributionMilestones = computed(() => [
      {
        id: 'first_gb',
        name: 'First GB',
        icon: '🌟',
        reward: '100 tokens',
        achieved: userStore.contributedStorage >= 1000000000
      },
      {
        id: 'reliable_node',
        name: 'Reliable Node',
        icon: '🛡️',
        reward: '500 tokens',
        achieved: userStore.uptime >= 95
      },
      {
        id: 'community_helper',
        name: 'Community Helper',
        icon: '🤝',
        reward: '1000 tokens',
        achieved: gameStore.communityScore >= 80
      }
    ])

    const recentAchievements = computed(() => gameStore.achievements.slice(0, 3))

    // Methods
    const handleInsightAction = async (insight, action) => {
      try {
        await executeIntelligentAction(insight, action)
        await analyzeUserBehavior({ insight, action, timestamp: Date.now() })
        
        // Show success message
        ElMessage.success(`Action completed: ${action}`)
      } catch (error) {
        ElMessage.error(`Failed to execute action: ${error.message}`)
      }
    }

    const dismissInsight = (insightId) => {
      const index = prioritizedInsights.value.findIndex(i => i.id === insightId)
      if (index > -1) {
        prioritizedInsights.value.splice(index, 1)
      }
    }

    const showTierOptimization = async () => {
      try {
        usageData.value = await economyStore.getUsageAnalysis()
        tierRecommendations.value = await economyStore.getTierRecommendations()
        showTierDialog.value = true
      } catch (error) {
        ElMessage.error('Failed to load tier optimization data')
      }
    }

    const handleTierUpgrade = async (tierName) => {
      try {
        await economyStore.upgradeTier(tierName)
        ElMessage.success(`Successfully upgraded to ${tierName}`)
        showTierDialog.value = false
      } catch (error) {
        ElMessage.error(`Failed to upgrade tier: ${error.message}`)
      }
    }

    const handleTierDowngrade = async (tierName) => {
      try {
        await economyStore.downgradeTier(tierName)
        ElMessage.success(`Successfully downgraded to ${tierName}`)
        showTierDialog.value = false
      } catch (error) {
        ElMessage.error(`Failed to downgrade tier: ${error.message}`)
      }
    }

    const executeAction = async (action) => {
      try {
        await action.execute()
        ElMessage.success(`Action completed: ${action.name}`)
      } catch (error) {
        ElMessage.error(`Failed to execute action: ${error.message}`)
      }
    }

    const dismissNotification = (notificationId) => {
      const index = intelligentNotifications.value.findIndex(n => n.id === notificationId)
      if (index > -1) {
        intelligentNotifications.value.splice(index, 1)
      }
    }

    const actOnNotification = async (notification, action) => {
      try {
        await action.execute()
        dismissNotification(notification.id)
        ElMessage.success(`Action completed: ${action.name}`)
      } catch (error) {
        ElMessage.error(`Failed to execute action: ${error.message}`)
      }
    }

    const executeIntelligentAction = async (insight, action) => {
      // Implement intelligent action execution
      console.log('Executing intelligent action:', { insight, action })
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Update stores based on action
      switch (action) {
        case 'upgrade_tier':
          await economyStore.upgradeTier(insight.recommendedTier)
          break
        case 'optimize_storage':
          await userStore.optimizeStorage()
          break
        case 'increase_contribution':
          await userStore.increaseContribution(insight.recommendedAmount)
          break
        default:
          console.warn('Unknown action:', action)
      }
    }

    // Lifecycle
    onMounted(async () => {
      try {
        await Promise.all([
          userStore.fetchUserData(),
          economyStore.fetchEconomyData(),
          gameStore.fetchGameData(),
          generatePredictions(),
          personalizeInterface()
        ])
        
        // Check for predictive alerts
        await checkPredictiveAlerts()
      } catch (error) {
        console.error('Failed to load dashboard data:', error)
        ElMessage.error('Failed to load dashboard data')
      }
    })

    // Watch for changes that might trigger new insights
    watch([storageUsagePercentage, currentBill, reputationScore], async () => {
      await generatePredictions()
    })

    const checkPredictiveAlerts = async () => {
      // Check for quota alerts
      if (storageUsagePercentage.value > 80) {
        const daysUntilQuotaExceeded = calculateDaysUntilQuotaExceeded()
        
        addAlert({
          id: 'quota_warning',
          type: 'warning',
          title: 'Quota Alert',
          message: `You'll exceed your monthly quota in ${daysUntilQuotaExceeded} days`,
          actions: [
            {
              name: 'Upgrade Now',
              type: 'primary',
              execute: showTierOptimization
            },
            {
              name: 'Optimize Storage',
              type: 'default',
              execute: () => userStore.optimizeStorage()
            }
          ]
        })
      }
      
      // Check for tier optimization opportunities
      if (economyStore.tierEfficiency < 0.7) {
        addAlert({
          id: 'tier_optimization',
          type: 'info',
          title: 'Tier Optimization',
          message: 'You could save money by adjusting your tier',
          actions: [
            {
              name: 'View Recommendations',
              type: 'primary',
              execute: showTierOptimization
            }
          ]
        })
      }
    }

    const calculateDaysUntilQuotaExceeded = () => {
      const usage = userStore.storageUsed
      const limit = userStore.storageLimit
      const growthRate = userStore.storageGrowthRate || 0.05 // 5% per day
      
      if (growthRate <= 0) return Infinity
      
      const remaining = limit - usage
      const dailyGrowth = usage * growthRate
      
      return Math.ceil(remaining / dailyGrowth)
    }

    return {
      // Reactive state
      showTierDialog,
      usageData,
      tierRecommendations,
      
      // Computed properties
      dynamicGreeting,
      userStatus,
      quickStats,
      storageUsagePercentage,
      storageProgressColor,
      storageUsed,
      storageAvailable,
      contributedStorage,
      currentBill,
      earnedTokens,
      reputationScore,
      currentTier,
      tierBenefits,
      contributionProgress,
      nextRewardAmount,
      progressColor,
      contributionMilestones,
      recentAchievements,
      
      // Intelligence system
      prioritizedInsights,
      adaptiveActions,
      intelligentNotifications,
      userContext,
      insightReliability,
      smartAlerts,
      
      // Methods
      handleInsightAction,
      dismissInsight,
      showTierOptimization,
      handleTierUpgrade,
      handleTierDowngrade,
      executeAction,
      dismissNotification,
      actOnNotification,
      dismissAlert,
      handleAlertAction,
      
      // Utilities
      formatBytes,
      formatCurrency
    }
  }
}
</script>

<style scoped>
.intelligent-dashboard {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  color: white;
}

.user-greeting h1 {
  font-size: 2.5rem;
  margin: 0;
  font-weight: 700;
}

.user-status {
  font-size: 1.1rem;
  opacity: 0.9;
  margin: 5px 0 0;
}

.quick-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
}

.stat-card {
  display: flex;
  align-items: center;
  padding: 15px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  backdrop-filter: blur(10px);
  min-width: 140px;
}

.stat-icon {
  font-size: 2rem;
  margin-right: 12px;
}

.stat-content {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.2;
}

.stat-label {
  font-size: 0.9rem;
  opacity: 0.8;
}

.insights-panel {
  margin-bottom: 20px;
}

.insights-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.insights-header h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.insights-content {
  min-height: 120px;
}

.no-insights {
  text-align: center;
  color: #909399;
  padding: 40px 20px;
}

.no-insights-icon {
  font-size: 3rem;
  margin-bottom: 10px;
}

.usage-card,
.economy-card {
  height: 100%;
}

.usage-card h3,
.economy-card h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.usage-ring-chart {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 0;
}

.usage-progress {
  flex-shrink: 0;
}

.percentage-text {
  font-size: 1.2rem;
  font-weight: 600;
  color: #303133;
}

.usage-breakdown {
  flex: 1;
  margin-left: 30px;
}

.usage-item {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.usage-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  margin-right: 8px;
}

.usage-item .label {
  flex: 1;
  font-size: 0.9rem;
  color: #606266;
}

.usage-item .value {
  font-weight: 600;
  color: #303133;
}

.economy-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-bottom: 20px;
}

.economy-item {
  display: flex;
  align-items: center;
  text-align: center;
}

.economy-icon {
  font-size: 2rem;
  margin-right: 12px;
}

.economy-content {
  display: flex;
  flex-direction: column;
}

.economy-value {
  font-size: 1.3rem;
  font-weight: 600;
  color: #303133;
}

.economy-label {
  font-size: 0.8rem;
  color: #909399;
}

.tier-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 8px;
}

.tier-badge {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  margin-right: 10px;
}

.tier-badge.free {
  background: #e3f2fd;
  color: #1976d2;
}

.tier-badge.basic {
  background: #e8f5e8;
  color: #4caf50;
}

.tier-badge.pro {
  background: #fff3e0;
  color: #ff9800;
}

.tier-badge.enterprise {
  background: #f3e5f5;
  color: #9c27b0;
}

.tier-benefits {
  font-size: 0.9rem;
  color: #606266;
}

.smart-alerts {
  margin-bottom: 20px;
}

.alert-enter-active,
.alert-leave-active {
  transition: all 0.3s ease;
}

.alert-enter-from,
.alert-leave-to {
  opacity: 0;
  transform: translateX(30px);
}

.adaptive-actions {
  margin-bottom: 20px;
}

.adaptive-actions h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.contribution-rewards {
  margin-bottom: 20px;
}

.contribution-rewards h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.progress-label {
  font-size: 1rem;
  font-weight: 500;
  color: #303133;
}

.progress-tokens {
  font-size: 1.1rem;
  font-weight: 600;
  color: #409EFF;
}

.progress-text {
  font-size: 0.9rem;
  color: #606266;
}

.progress-milestones {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
  margin-top: 20px;
}

.milestone {
  display: flex;
  align-items: center;
  padding: 12px;
  border-radius: 8px;
  background: #f8f9fa;
  transition: all 0.2s ease;
}

.milestone.achieved {
  background: #e8f5e8;
  border: 1px solid #4caf50;
}

.milestone-icon {
  font-size: 1.5rem;
  margin-right: 12px;
}

.milestone-content {
  display: flex;
  flex-direction: column;
}

.milestone-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: #303133;
}

.milestone-reward {
  font-size: 0.8rem;
  color: #606266;
}

.gamification-card h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.achievement-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.achievement-card {
  display: flex;
  align-items: center;
  padding: 15px;
  border-radius: 8px;
  background: #f8f9fa;
  border: 1px solid #e4e7ed;
  transition: all 0.2s ease;
}

.achievement-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.achievement-card.unlocked {
  background: #e8f5e8;
  border-color: #4caf50;
}

.achievement-icon {
  font-size: 2rem;
  margin-right: 15px;
}

.achievement-content {
  flex: 1;
}

.achievement-content h4 {
  margin: 0 0 5px;
  font-size: 1rem;
  color: #303133;
}

.achievement-content p {
  margin: 0 0 10px;
  font-size: 0.85rem;
  color: #606266;
}

.achievement-progress {
  margin-top: 8px;
}

/* Responsive Design */
@media (max-width: 768px) {
  .dashboard-header {
    flex-direction: column;
    text-align: center;
  }

  .user-greeting h1 {
    font-size: 2rem;
  }

  .quick-stats {
    grid-template-columns: repeat(2, 1fr);
    margin-top: 20px;
  }

  .usage-ring-chart {
    flex-direction: column;
    text-align: center;
  }

  .usage-breakdown {
    margin-left: 0;
    margin-top: 20px;
  }

  .economy-stats {
    grid-template-columns: 1fr;
  }

  .tier-info {
    flex-direction: column;
    gap: 10px;
  }

  .progress-milestones {
    grid-template-columns: 1fr;
  }

  .achievement-grid {
    grid-template-columns: 1fr;
  }
}
</style>
