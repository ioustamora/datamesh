<template>
  <div class="governance-container">
    <div class="governance-header">
      <h1 class="governance-title">Network Governance</h1>
      <p class="governance-subtitle">Manage the DataMesh network through democratic decision-making and operator oversight</p>
    </div>
    
    <div class="governance-content">
      <div class="governance-tabs">
        <el-tabs v-model="activeTab" @tab-click="handleTabClick">
          <el-tab-pane label="Overview" name="overview">
            <div class="overview-section">
              <div class="overview-cards">
                <el-card class="overview-card">
                  <div class="card-header">
                    <h3>Network Health</h3>
                    <el-icon class="card-icon" :class="getHealthIconClass(networkHealth.status)">
                      <component :is="getHealthIcon(networkHealth.status)" />
                    </el-icon>
                  </div>
                  <div class="card-content">
                    <div class="health-status">
                      <span class="health-label">Status:</span>
                      <el-tag :type="getHealthTagType(networkHealth.status)">
                        {{ networkHealth.status }}
                      </el-tag>
                    </div>
                    <div class="health-metrics">
                      <div class="metric">
                        <span class="metric-label">Uptime:</span>
                        <span class="metric-value">{{ networkHealth.uptime }}</span>
                      </div>
                      <div class="metric">
                        <span class="metric-label">Response Time:</span>
                        <span class="metric-value">{{ networkHealth.responseTime }}</span>
                      </div>
                    </div>
                  </div>
                </el-card>
                
                <el-card class="overview-card">
                  <div class="card-header">
                    <h3>Bootstrap Operators</h3>
                    <el-icon class="card-icon operator-icon">
                      <User />
                    </el-icon>
                  </div>
                  <div class="card-content">
                    <div class="operator-count">
                      <span class="count-number">{{ operators.length }}</span>
                      <span class="count-label">Active Operators</span>
                    </div>
                    <div class="operator-regions">
                      <div class="region-item" v-for="region in operatorRegions" :key="region.name">
                        <span class="region-name">{{ region.name }}</span>
                        <span class="region-count">{{ region.count }}</span>
                      </div>
                    </div>
                  </div>
                </el-card>
                
                <el-card class="overview-card">
                  <div class="card-header">
                    <h3>Active Proposals</h3>
                    <el-icon class="card-icon proposal-icon">
                      <Document />
                    </el-icon>
                  </div>
                  <div class="card-content">
                    <div class="proposal-count">
                      <span class="count-number">{{ activeProposals.length }}</span>
                      <span class="count-label">Pending Votes</span>
                    </div>
                    <div class="proposal-actions">
                      <el-button size="small" @click="goToProposals">View All</el-button>
                      <el-button size="small" type="primary" @click="createProposal">Create New</el-button>
                    </div>
                  </div>
                </el-card>
                
                <el-card class="overview-card">
                  <div class="card-header">
                    <h3>Governance Tokens</h3>
                    <el-icon class="card-icon token-icon">
                      <Coin />
                    </el-icon>
                  </div>
                  <div class="card-content">
                    <div class="token-balance">
                      <span class="balance-amount">{{ tokenData.balance }}</span>
                      <span class="balance-label">DMT Balance</span>
                    </div>
                    <div class="token-stats">
                      <div class="stat-item">
                        <span class="stat-label">Voting Power:</span>
                        <span class="stat-value">{{ tokenData.votingPower }}%</span>
                      </div>
                      <div class="stat-item">
                        <span class="stat-label">Staked:</span>
                        <span class="stat-value">{{ tokenData.staked }}</span>
                      </div>
                    </div>
                  </div>
                </el-card>
              </div>
              
              <div class="overview-charts">
                <el-card class="chart-card">
                  <div class="chart-header">
                    <h3>Network Performance</h3>
                    <div class="chart-controls">
                      <el-select v-model="chartPeriod" size="small">
                        <el-option label="Last 24 Hours" value="24h" />
                        <el-option label="Last 7 Days" value="7d" />
                        <el-option label="Last 30 Days" value="30d" />
                      </el-select>
                    </div>
                  </div>
                  <div class="chart-content">
                    <canvas ref="performanceChart" height="200"></canvas>
                  </div>
                </el-card>
                
                <el-card class="chart-card">
                  <div class="chart-header">
                    <h3>Governance Activity</h3>
                  </div>
                  <div class="chart-content">
                    <div class="activity-list">
                      <div
                        v-for="activity in recentActivities"
                        :key="activity.id"
                        class="activity-item"
                      >
                        <div class="activity-icon">
                          <el-icon :class="getActivityIconClass(activity.type)">
                            <component :is="getActivityIcon(activity.type)" />
                          </el-icon>
                        </div>
                        <div class="activity-content">
                          <div class="activity-title">{{ activity.title }}</div>
                          <div class="activity-description">{{ activity.description }}</div>
                          <div class="activity-time">{{ formatTimeAgo(activity.timestamp) }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </el-card>
              </div>
            </div>
          </el-tab-pane>
          
          <el-tab-pane label="Operators" name="operators">
            <div class="operators-section">
              <div class="operators-header">
                <h3>Bootstrap Operators</h3>
                <div class="operators-actions">
                  <el-button @click="refreshOperators">
                    <el-icon><Refresh /></el-icon>
                    Refresh
                  </el-button>
                  <el-button type="primary" @click="proposeNewOperator">
                    <el-icon><Plus /></el-icon>
                    Propose New Operator
                  </el-button>
                </div>
              </div>
              
              <div class="operators-table">
                <el-table :data="operators" style="width: 100%">
                  <el-table-column prop="name" label="Operator" min-width="200">
                    <template #default="{ row }">
                      <div class="operator-info">
                        <el-avatar :size="32" :src="row.avatar">
                          <el-icon><User /></el-icon>
                        </el-avatar>
                        <div class="operator-details">
                          <div class="operator-name">{{ row.name }}</div>
                          <div class="operator-id">{{ row.id }}</div>
                        </div>
                      </div>
                    </template>
                  </el-table-column>
                  
                  <el-table-column prop="jurisdiction" label="Jurisdiction" width="150" />
                  
                  <el-table-column prop="stake" label="Stake" width="120">
                    <template #default="{ row }">
                      {{ formatTokenAmount(row.stake) }} DMT
                    </template>
                  </el-table-column>
                  
                  <el-table-column prop="votingWeight" label="Voting Weight" width="120">
                    <template #default="{ row }">
                      {{ (row.votingWeight * 100).toFixed(1) }}%
                    </template>
                  </el-table-column>
                  
                  <el-table-column prop="reputation" label="Reputation" width="120">
                    <template #default="{ row }">
                      <el-progress
                        :percentage="row.reputation * 100"
                        :stroke-width="6"
                        :show-text="false"
                        :color="getReputationColor(row.reputation)"
                      />
                      <span class="reputation-score">{{ (row.reputation * 100).toFixed(0) }}%</span>
                    </template>
                  </el-table-column>
                  
                  <el-table-column prop="services" label="Services" width="200">
                    <template #default="{ row }">
                      <div class="services-list">
                        <el-tag
                          v-for="service in row.services"
                          :key="service"
                          size="small"
                          :type="getServiceTagType(service)"
                        >
                          {{ service }}
                        </el-tag>
                      </div>
                    </template>
                  </el-table-column>
                  
                  <el-table-column prop="status" label="Status" width="100">
                    <template #default="{ row }">
                      <el-tag :type="getStatusTagType(row.status)">
                        {{ row.status }}
                      </el-tag>
                    </template>
                  </el-table-column>
                  
                  <el-table-column label="Actions" width="150">
                    <template #default="{ row }">
                      <el-button size="small" @click="viewOperator(row)">
                        <el-icon><View /></el-icon>
                      </el-button>
                      <el-button size="small" @click="contactOperator(row)">
                        <el-icon><Message /></el-icon>
                      </el-button>
                      <el-dropdown @command="handleOperatorAction">
                        <el-button size="small">
                          <el-icon><More /></el-icon>
                        </el-button>
                        <template #dropdown>
                          <el-dropdown-menu>
                            <el-dropdown-item :command="{ action: 'report', operator: row }">
                              Report Issue
                            </el-dropdown-item>
                            <el-dropdown-item :command="{ action: 'challenge', operator: row }" divided>
                              Challenge Status
                            </el-dropdown-item>
                          </el-dropdown-menu>
                        </template>
                      </el-dropdown>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </div>
          </el-tab-pane>
          
          <el-tab-pane label="Proposals" name="proposals">
            <div class="proposals-section">
              <div class="proposals-header">
                <h3>Governance Proposals</h3>
                <div class="proposals-actions">
                  <el-select v-model="proposalFilter" placeholder="Filter by status" size="small">
                    <el-option label="All" value="all" />
                    <el-option label="Active" value="active" />
                    <el-option label="Passed" value="passed" />
                    <el-option label="Failed" value="failed" />
                  </el-select>
                  <el-button type="primary" @click="createProposal">
                    <el-icon><Plus /></el-icon>
                    Create Proposal
                  </el-button>
                </div>
              </div>
              
              <div class="proposals-list">
                <div
                  v-for="proposal in filteredProposals"
                  :key="proposal.id"
                  class="proposal-card"
                >
                  <el-card>
                    <div class="proposal-header">
                      <div class="proposal-info">
                        <h4 class="proposal-title">{{ proposal.title }}</h4>
                        <div class="proposal-meta">
                          <span class="proposal-type">{{ proposal.type }}</span>
                          <span class="proposal-author">by {{ proposal.author }}</span>
                          <span class="proposal-date">{{ formatDate(proposal.createdAt) }}</span>
                        </div>
                      </div>
                      <div class="proposal-status">
                        <el-tag :type="getProposalStatusType(proposal.status)">
                          {{ proposal.status }}
                        </el-tag>
                      </div>
                    </div>
                    
                    <div class="proposal-content">
                      <p class="proposal-description">{{ proposal.description }}</p>
                      
                      <div class="proposal-voting" v-if="proposal.status === 'Active'">
                        <div class="voting-progress">
                          <div class="vote-stats">
                            <span class="vote-count">{{ proposal.votesFor }} For</span>
                            <span class="vote-count">{{ proposal.votesAgainst }} Against</span>
                            <span class="vote-count">{{ proposal.quorum }}% Quorum</span>
                          </div>
                          <el-progress
                            :percentage="proposal.quorum"
                            :stroke-width="8"
                            :show-text="false"
                            :color="getQuorumColor(proposal.quorum)"
                          />
                        </div>
                        
                        <div class="voting-actions">
                          <el-button type="success" @click="vote(proposal.id, 'for')">
                            <el-icon><Check /></el-icon>
                            Vote For
                          </el-button>
                          <el-button type="danger" @click="vote(proposal.id, 'against')">
                            <el-icon><Close /></el-icon>
                            Vote Against
                          </el-button>
                          <el-button @click="viewProposal(proposal)">
                            <el-icon><View /></el-icon>
                            View Details
                          </el-button>
                        </div>
                      </div>
                      
                      <div class="proposal-timeline" v-if="proposal.status !== 'Active'">
                        <div class="timeline-item">
                          <strong>Result:</strong> {{ proposal.result }}
                        </div>
                        <div class="timeline-item">
                          <strong>Execution:</strong> {{ proposal.executionStatus }}
                        </div>
                      </div>
                    </div>
                  </el-card>
                </div>
              </div>
            </div>
          </el-tab-pane>
          
          <el-tab-pane label="Voting" name="voting">
            <div class="voting-section">
              <div class="voting-header">
                <h3>My Voting Power</h3>
                <div class="voting-stats">
                  <div class="stat-card">
                    <div class="stat-value">{{ tokenData.balance }}</div>
                    <div class="stat-label">DMT Balance</div>
                  </div>
                  <div class="stat-card">
                    <div class="stat-value">{{ tokenData.votingPower }}%</div>
                    <div class="stat-label">Voting Power</div>
                  </div>
                  <div class="stat-card">
                    <div class="stat-value">{{ votingHistory.length }}</div>
                    <div class="stat-label">Votes Cast</div>
                  </div>
                </div>
              </div>
              
              <div class="voting-history">
                <h4>Voting History</h4>
                <div class="history-list">
                  <div
                    v-for="vote in votingHistory"
                    :key="vote.id"
                    class="history-item"
                  >
                    <div class="vote-info">
                      <div class="vote-proposal">{{ vote.proposalTitle }}</div>
                      <div class="vote-details">
                        <span class="vote-choice" :class="vote.choice">{{ vote.choice }}</span>
                        <span class="vote-power">{{ vote.power }} DMT</span>
                        <span class="vote-date">{{ formatDate(vote.timestamp) }}</span>
                      </div>
                    </div>
                    <div class="vote-result">
                      <el-tag :type="getVoteResultType(vote.result)">
                        {{ vote.result }}
                      </el-tag>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useGovernanceStore } from '@/store/governance'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  User,
  Document,
  Coin,
  Refresh,
  Plus,
  View,
  Message,
  More,
  Check,
  Close,
  CircleCheckFilled,
  CircleCloseFilled,
  Warning
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

export default {
  name: 'Governance',
  components: {
    User,
    Document,
    Coin,
    Refresh,
    Plus,
    View,
    Message,
    More,
    Check,
    Close,
    CircleCheckFilled,
    CircleCloseFilled,
    Warning
  },
  setup() {
    const router = useRouter()
    const governanceStore = useGovernanceStore()
    
    const activeTab = ref('overview')
    const chartPeriod = ref('24h')
    const proposalFilter = ref('all')
    const performanceChart = ref()
    
    const networkHealth = reactive({
      status: 'Healthy',
      uptime: '99.95%',
      responseTime: '245ms'
    })
    
    const operators = reactive([
      {
        id: 'bootstrap-node-1',
        name: 'DataMesh Foundation',
        jurisdiction: 'Estonia',
        stake: 1000000,
        votingWeight: 0.15,
        reputation: 0.95,
        services: ['Storage', 'Bandwidth', 'Bootstrap'],
        status: 'Active',
        avatar: ''
      },
      {
        id: 'bootstrap-node-2',
        name: 'CryptoCloud Inc.',
        jurisdiction: 'Singapore',
        stake: 800000,
        votingWeight: 0.12,
        reputation: 0.89,
        services: ['Storage', 'CDN'],
        status: 'Active',
        avatar: ''
      },
      {
        id: 'bootstrap-node-3',
        name: 'DecentStorage Ltd.',
        jurisdiction: 'Switzerland',
        stake: 600000,
        votingWeight: 0.09,
        reputation: 0.92,
        services: ['Storage', 'Monitoring'],
        status: 'Active',
        avatar: ''
      }
    ])
    
    const proposals = reactive([
      {
        id: 1,
        title: 'Increase Free Tier Storage to 10GB',
        type: 'QuotaModification',
        author: 'community-user-1',
        description: 'Proposal to increase the free tier storage limit from 5GB to 10GB to attract more users.',
        status: 'Active',
        votesFor: 1250000,
        votesAgainst: 340000,
        quorum: 32,
        createdAt: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000),
        endDate: new Date(Date.now() + 9 * 24 * 60 * 60 * 1000)
      },
      {
        id: 2,
        title: 'Add New Bootstrap Node in Asia',
        type: 'BootstrapNodeAddition',
        author: 'asia-operator-1',
        description: 'Proposal to add a new bootstrap node in Tokyo, Japan to improve network coverage in Asia.',
        status: 'Passed',
        result: 'Approved with 78% support',
        executionStatus: 'Implemented',
        createdAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
      },
      {
        id: 3,
        title: 'Reduce API Rate Limits',
        type: 'FeeAdjustment',
        author: 'developer-advocate',
        description: 'Proposal to reduce API rate limits for premium users to improve developer experience.',
        status: 'Failed',
        result: 'Rejected - insufficient quorum',
        executionStatus: 'Not executed',
        createdAt: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000)
      }
    ])
    
    const tokenData = reactive({
      balance: '25,000',
      votingPower: '0.38',
      staked: '15,000'
    })
    
    const votingHistory = reactive([
      {
        id: 1,
        proposalTitle: 'Increase Free Tier Storage to 10GB',
        choice: 'For',
        power: 25000,
        timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
        result: 'Pending'
      },
      {
        id: 2,
        proposalTitle: 'Add New Bootstrap Node in Asia',
        choice: 'For',
        power: 20000,
        timestamp: new Date(Date.now() - 25 * 24 * 60 * 60 * 1000),
        result: 'Passed'
      }
    ])
    
    const recentActivities = reactive([
      {
        id: 1,
        type: 'proposal',
        title: 'New Proposal Created',
        description: 'Increase Free Tier Storage to 10GB',
        timestamp: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000)
      },
      {
        id: 2,
        type: 'vote',
        title: 'Proposal Approved',
        description: 'Add New Bootstrap Node in Asia',
        timestamp: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
      },
      {
        id: 3,
        type: 'operator',
        title: 'Operator Status Updated',
        description: 'CryptoCloud Inc. - Status changed to Active',
        timestamp: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000)
      }
    ])
    
    const operatorRegions = computed(() => {
      const regions = {}
      operators.forEach(op => {
        regions[op.jurisdiction] = (regions[op.jurisdiction] || 0) + 1
      })
      return Object.entries(regions).map(([name, count]) => ({ name, count }))
    })
    
    const activeProposals = computed(() => {
      return proposals.filter(p => p.status === 'Active')
    })
    
    const filteredProposals = computed(() => {
      if (proposalFilter.value === 'all') {
        return proposals
      }
      return proposals.filter(p => p.status.toLowerCase() === proposalFilter.value)
    })
    
    const handleTabClick = (tab) => {
      activeTab.value = tab.name
    }
    
    const getHealthIcon = (status) => {
      const iconMap = {
        'Healthy': CircleCheckFilled,
        'Warning': Warning,
        'Critical': CircleCloseFilled
      }
      return iconMap[status] || CircleCheckFilled
    }
    
    const getHealthIconClass = (status) => {
      const classMap = {
        'Healthy': 'health-icon-healthy',
        'Warning': 'health-icon-warning',
        'Critical': 'health-icon-critical'
      }
      return classMap[status] || 'health-icon-healthy'
    }
    
    const getHealthTagType = (status) => {
      const typeMap = {
        'Healthy': 'success',
        'Warning': 'warning',
        'Critical': 'danger'
      }
      return typeMap[status] || 'success'
    }
    
    const getActivityIcon = (type) => {
      const iconMap = {
        'proposal': Document,
        'vote': Check,
        'operator': User
      }
      return iconMap[type] || Document
    }
    
    const getActivityIconClass = (type) => {
      const classMap = {
        'proposal': 'activity-icon-proposal',
        'vote': 'activity-icon-vote',
        'operator': 'activity-icon-operator'
      }
      return classMap[type] || 'activity-icon-default'
    }
    
    const formatTokenAmount = (amount) => {
      return amount.toLocaleString()
    }
    
    const getReputationColor = (reputation) => {
      if (reputation >= 0.9) return '#67C23A'
      if (reputation >= 0.7) return '#E6A23C'
      return '#F56C6C'
    }
    
    const getServiceTagType = (service) => {
      const typeMap = {
        'Storage': 'primary',
        'Bandwidth': 'success',
        'Bootstrap': 'warning',
        'CDN': 'info',
        'Monitoring': 'danger'
      }
      return typeMap[service] || 'default'
    }
    
    const getStatusTagType = (status) => {
      const typeMap = {
        'Active': 'success',
        'Inactive': 'warning',
        'Suspended': 'danger'
      }
      return typeMap[status] || 'info'
    }
    
    const getProposalStatusType = (status) => {
      const typeMap = {
        'Active': 'primary',
        'Passed': 'success',
        'Failed': 'danger',
        'Pending': 'warning'
      }
      return typeMap[status] || 'info'
    }
    
    const getQuorumColor = (quorum) => {
      if (quorum >= 50) return '#67C23A'
      if (quorum >= 20) return '#E6A23C'
      return '#F56C6C'
    }
    
    const getVoteResultType = (result) => {
      const typeMap = {
        'Passed': 'success',
        'Failed': 'danger',
        'Pending': 'warning'
      }
      return typeMap[result] || 'info'
    }
    
    const formatTimeAgo = (timestamp) => {
      return dayjs(timestamp).fromNow()
    }
    
    const formatDate = (date) => {
      return dayjs(date).format('YYYY-MM-DD')
    }
    
    const goToProposals = () => {
      activeTab.value = 'proposals'
    }
    
    const createProposal = () => {
      router.push('/governance/create-proposal')
    }
    
    const refreshOperators = async () => {
      try {
        await governanceStore.fetchOperators()
        ElMessage.success('Operators refreshed')
      } catch (error) {
        ElMessage.error('Failed to refresh operators')
      }
    }
    
    const proposeNewOperator = () => {
      router.push('/governance/propose-operator')
    }
    
    const viewOperator = (operator) => {
      router.push(`/governance/operator/${operator.id}`)
    }
    
    const contactOperator = (operator) => {
      ElMessage.info(`Contact functionality for ${operator.name} to be implemented`)
    }
    
    const handleOperatorAction = ({ action, operator }) => {
      switch (action) {
        case 'report':
          ElMessage.info(`Report functionality for ${operator.name} to be implemented`)
          break
        case 'challenge':
          ElMessage.info(`Challenge functionality for ${operator.name} to be implemented`)
          break
      }
    }
    
    const vote = async (proposalId, choice) => {
      try {
        await ElMessageBox.confirm(
          `Are you sure you want to vote "${choice}" on this proposal?`,
          'Confirm Vote',
          {
            confirmButtonText: 'Vote',
            cancelButtonText: 'Cancel',
            type: 'info'
          }
        )
        
        await governanceStore.vote(proposalId, choice)
        ElMessage.success('Vote cast successfully')
        
        // Update voting history
        const proposal = proposals.find(p => p.id === proposalId)
        if (proposal) {
          votingHistory.unshift({
            id: Date.now(),
            proposalTitle: proposal.title,
            choice: choice === 'for' ? 'For' : 'Against',
            power: parseInt(tokenData.balance.replace(',', '')),
            timestamp: new Date(),
            result: 'Pending'
          })
        }
      } catch (error) {
        if (error !== 'cancel') {
          ElMessage.error('Failed to cast vote')
        }
      }
    }
    
    const viewProposal = (proposal) => {
      router.push(`/governance/proposal/${proposal.id}`)
    }
    
    onMounted(() => {
      // Initialize charts and data
    })
    
    return {
      activeTab,
      chartPeriod,
      proposalFilter,
      performanceChart,
      networkHealth,
      operators,
      proposals,
      tokenData,
      votingHistory,
      recentActivities,
      operatorRegions,
      activeProposals,
      filteredProposals,
      handleTabClick,
      getHealthIcon,
      getHealthIconClass,
      getHealthTagType,
      getActivityIcon,
      getActivityIconClass,
      formatTokenAmount,
      getReputationColor,
      getServiceTagType,
      getStatusTagType,
      getProposalStatusType,
      getQuorumColor,
      getVoteResultType,
      formatTimeAgo,
      formatDate,
      goToProposals,
      createProposal,
      refreshOperators,
      proposeNewOperator,
      viewOperator,
      contactOperator,
      handleOperatorAction,
      vote,
      viewProposal
    }
  }
}
</script>

<style scoped>
.governance-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.governance-header {
  padding: 24px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.governance-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px 0;
}

.governance-subtitle {
  color: var(--el-text-color-regular);
  margin: 0;
}

.governance-content {
  flex: 1;
  overflow: hidden;
}

.governance-tabs {
  height: 100%;
  padding: 24px;
}

.governance-tabs :deep(.el-tabs__content) {
  height: calc(100% - 60px);
  overflow: auto;
}

.overview-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.overview-card {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.card-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
  font-size: 16px;
}

.card-icon {
  font-size: 24px;
}

.health-icon-healthy {
  color: var(--el-color-success);
}

.health-icon-warning {
  color: var(--el-color-warning);
}

.health-icon-critical {
  color: var(--el-color-danger);
}

.operator-icon {
  color: var(--el-color-primary);
}

.proposal-icon {
  color: var(--el-color-info);
}

.token-icon {
  color: var(--el-color-warning);
}

.health-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.health-label {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.health-metrics {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
}

.metric-label {
  color: var(--el-text-color-secondary);
}

.metric-value {
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.operator-count {
  text-align: center;
  margin-bottom: 16px;
}

.count-number {
  display: block;
  font-size: 32px;
  font-weight: 600;
  color: var(--el-color-primary);
}

.count-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.operator-regions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.region-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
}

.region-name {
  color: var(--el-text-color-primary);
}

.region-count {
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.proposal-count {
  text-align: center;
  margin-bottom: 16px;
}

.proposal-actions {
  display: flex;
  gap: 8px;
}

.token-balance {
  text-align: center;
  margin-bottom: 16px;
}

.balance-amount {
  display: block;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-color-warning);
}

.balance-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.token-stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
}

.stat-label {
  color: var(--el-text-color-secondary);
}

.stat-value {
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.overview-charts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  flex: 1;
}

.chart-card {
  padding: 20px;
  display: flex;
  flex-direction: column;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.chart-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
  font-size: 16px;
}

.chart-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.activity-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.activity-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--el-fill-color-light);
  flex-shrink: 0;
}

.activity-icon-proposal {
  color: var(--el-color-info);
}

.activity-icon-vote {
  color: var(--el-color-success);
}

.activity-icon-operator {
  color: var(--el-color-primary);
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.activity-description {
  color: var(--el-text-color-regular);
  margin-bottom: 4px;
}

.activity-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.operators-section {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.operators-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.operators-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.operators-actions {
  display: flex;
  gap: 8px;
}

.operators-table {
  flex: 1;
}

.operator-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.operator-details {
  display: flex;
  flex-direction: column;
}

.operator-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.operator-id {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
}

.reputation-score {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-left: 8px;
}

.services-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.proposals-section {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.proposals-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.proposals-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.proposals-actions {
  display: flex;
  gap: 8px;
}

.proposals-list {
  flex: 1;
  overflow: auto;
}

.proposal-card {
  margin-bottom: 20px;
}

.proposal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.proposal-title {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
  font-size: 18px;
}

.proposal-meta {
  display: flex;
  gap: 16px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.proposal-description {
  color: var(--el-text-color-regular);
  margin-bottom: 16px;
  line-height: 1.5;
}

.proposal-voting {
  background: var(--el-fill-color-light);
  padding: 16px;
  border-radius: 8px;
}

.voting-progress {
  margin-bottom: 16px;
}

.vote-stats {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.vote-count {
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.voting-actions {
  display: flex;
  gap: 8px;
}

.proposal-timeline {
  background: var(--el-fill-color-light);
  padding: 16px;
  border-radius: 8px;
}

.timeline-item {
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.voting-section {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.voting-header {
  margin-bottom: 32px;
}

.voting-header h3 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.voting-stats {
  display: flex;
  gap: 24px;
}

.stat-card {
  background: var(--el-fill-color-light);
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-color-primary);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.voting-history {
  flex: 1;
}

.voting-history h4 {
  margin: 0 0 16px 0;
  color: var(--el-text-color-primary);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.vote-info {
  flex: 1;
}

.vote-proposal {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.vote-details {
  display: flex;
  gap: 16px;
  font-size: 14px;
}

.vote-choice {
  font-weight: 500;
}

.vote-choice.For {
  color: var(--el-color-success);
}

.vote-choice.Against {
  color: var(--el-color-danger);
}

.vote-power {
  color: var(--el-text-color-secondary);
}

.vote-date {
  color: var(--el-text-color-secondary);
}

.vote-result {
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .overview-cards {
    grid-template-columns: 1fr;
  }
  
  .overview-charts {
    grid-template-columns: 1fr;
  }
  
  .voting-stats {
    flex-direction: column;
  }
  
  .proposal-header {
    flex-direction: column;
    gap: 12px;
  }
  
  .voting-actions {
    flex-direction: column;
  }
  
  .vote-stats {
    flex-direction: column;
    gap: 4px;
  }
}
</style>