<template>
  <div class="governance-overview">
    <div class="page-header">
      <h1>Governance Overview</h1>
      <p>Monitor and manage DataMesh governance activities</p>
    </div>

    <div class="overview-grid">
      <!-- Statistics Cards -->
      <div class="stats-section">
        <el-row :gutter="20">
          <el-col :span="6">
            <el-card class="stat-card">
              <div class="stat-content">
                <div class="stat-number">{{ totalOperators }}</div>
                <div class="stat-label">Active Operators</div>
              </div>
              <el-icon class="stat-icon"><UserFilled /></el-icon>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card class="stat-card">
              <div class="stat-content">
                <div class="stat-number">{{ activeProposals }}</div>
                <div class="stat-label">Active Proposals</div>
              </div>
              <el-icon class="stat-icon"><DocumentCopy /></el-icon>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card class="stat-card">
              <div class="stat-content">
                <div class="stat-number">{{ networkHealth }}%</div>
                <div class="stat-label">Network Health</div>
              </div>
              <el-icon class="stat-icon"><Monitor /></el-icon>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card class="stat-card">
              <div class="stat-content">
                <div class="stat-number">{{ votingPower }}</div>
                <div class="stat-label">Total Voting Power</div>
              </div>
              <el-icon class="stat-icon"><Select /></el-icon>
            </el-card>
          </el-col>
        </el-row>
      </div>

      <!-- Recent Activity -->
      <div class="activity-section">
        <el-card>
          <template #header>
            <div class="card-header">
              <h3>Recent Activity</h3>
              <el-button text @click="viewAllActivity">View All</el-button>
            </div>
          </template>
          <div class="activity-list">
            <div v-for="activity in recentActivity" :key="activity.id" class="activity-item">
              <div class="activity-icon">
                <el-icon :color="getActivityColor(activity.type)">
                  <component :is="getActivityIcon(activity.type)" />
                </el-icon>
              </div>
              <div class="activity-content">
                <div class="activity-title">{{ activity.title }}</div>
                <div class="activity-time">{{ formatTime(activity.timestamp) }}</div>
              </div>
            </div>
          </div>
        </el-card>
      </div>

      <!-- Quick Actions -->
      <div class="actions-section">
        <el-card>
          <template #header>
            <h3>Quick Actions</h3>
          </template>
          <div class="action-buttons">
            <el-button type="primary" @click="$router.push('/governance/proposals')">
              <el-icon><Plus /></el-icon>
              Create Proposal
            </el-button>
            <el-button @click="$router.push('/governance/operators')">
              <el-icon><UserFilled /></el-icon>
              Manage Operators
            </el-button>
            <el-button @click="$router.push('/governance/voting')">
              <el-icon><Select /></el-icon>
              View Voting
            </el-button>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { UserFilled, DocumentCopy, Monitor, Select, Plus } from '@element-plus/icons-vue'

export default {
  name: 'GovernanceOverview',
  components: {
    UserFilled,
    DocumentCopy,
    Monitor,
    Select,
    Plus
  },
  setup() {
    const totalOperators = ref(12)
    const activeProposals = ref(3)
    const networkHealth = ref(98)
    const votingPower = ref('2.4M')
    const recentActivity = ref([
      {
        id: 1,
        type: 'proposal',
        title: 'New storage allocation proposal submitted',
        timestamp: new Date(Date.now() - 1000 * 60 * 30)
      },
      {
        id: 2,
        type: 'vote',
        title: 'Voting completed for network upgrade',
        timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2)
      },
      {
        id: 3,
        type: 'operator',
        title: 'New operator node registered',
        timestamp: new Date(Date.now() - 1000 * 60 * 60 * 4)
      }
    ])

    const getActivityIcon = (type) => {
      switch (type) {
        case 'proposal': return 'DocumentCopy'
        case 'vote': return 'Select'
        case 'operator': return 'UserFilled'
        default: return 'InfoFilled'
      }
    }

    const getActivityColor = (type) => {
      switch (type) {
        case 'proposal': return 'var(--el-color-primary)'
        case 'vote': return 'var(--el-color-success)'
        case 'operator': return 'var(--el-color-warning)'
        default: return 'var(--el-color-info)'
      }
    }

    const formatTime = (timestamp) => {
      return new Date(timestamp).toLocaleString()
    }

    const viewAllActivity = () => {
      // Navigate to activity page
      console.log('View all activity')
    }

    onMounted(() => {
      // Load governance data
    })

    return {
      totalOperators,
      activeProposals,
      networkHealth,
      votingPower,
      recentActivity,
      getActivityIcon,
      getActivityColor,
      formatTime,
      viewAllActivity
    }
  }
}
</script>

<style scoped>
.governance-overview {
  padding: 24px;
}

.page-header {
  margin-bottom: 24px;
}

.page-header h1 {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.page-header p {
  margin: 0;
  color: var(--el-text-color-secondary);
}

.overview-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stat-card {
  position: relative;
  overflow: hidden;
}

.stat-content {
  position: relative;
  z-index: 2;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.stat-icon {
  position: absolute;
  top: 16px;
  right: 16px;
  font-size: 24px;
  color: var(--el-color-primary);
  opacity: 0.3;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.activity-item:last-child {
  border-bottom: none;
}

.activity-icon {
  flex-shrink: 0;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.activity-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.action-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

@media (max-width: 768px) {
  .governance-overview {
    padding: 16px;
  }
  
  .action-buttons {
    flex-direction: column;
  }
  
  .action-buttons .el-button {
    width: 100%;
  }
}
</style>