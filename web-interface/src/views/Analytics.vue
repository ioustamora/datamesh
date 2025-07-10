<template>
  <div class="analytics-container">
    <div class="analytics-header">
      <h1 class="analytics-title">
        Analytics Dashboard
      </h1>
      <p class="analytics-subtitle">
        Monitor system performance, usage metrics, and network health
      </p>
      
      <div class="analytics-controls">
        <el-date-picker
          v-model="dateRange"
          type="datetimerange"
          range-separator="to"
          start-placeholder="Start date"
          end-placeholder="End date"
          format="YYYY-MM-DD HH:mm"
          value-format="YYYY-MM-DD HH:mm:ss"
          @change="handleDateRangeChange"
        />
        <el-select
          v-model="selectedMetric"
          placeholder="Select metric"
          @change="handleMetricChange"
        >
          <el-option
            v-for="metric in availableMetrics"
            :key="metric.value"
            :label="metric.label"
            :value="metric.value"
          />
        </el-select>
        <el-button
          :loading="loading"
          @click="refreshData"
        >
          <el-icon><Refresh /></el-icon>
          Refresh
        </el-button>
        <el-button @click="exportData">
          <el-icon><Download /></el-icon>
          Export
        </el-button>
      </div>
    </div>
    
    <div class="analytics-content">
      <div class="metrics-overview">
        <div class="overview-cards">
          <el-card class="metric-card">
            <div class="metric-header">
              <h3>System Health</h3>
              <el-icon
                class="metric-icon"
                :class="getHealthIconClass(systemHealth.status)"
              >
                <component :is="getHealthIcon(systemHealth.status)" />
              </el-icon>
            </div>
            <div class="metric-value">
              {{ systemHealth.score }}%
            </div>
            <div
              class="metric-change"
              :class="getChangeClass(systemHealth.change)"
            >
              {{ systemHealth.change > 0 ? '+' : '' }}{{ systemHealth.change }}%
            </div>
          </el-card>
          
          <el-card class="metric-card">
            <div class="metric-header">
              <h3>Active Users</h3>
              <el-icon class="metric-icon user-icon">
                <User />
              </el-icon>
            </div>
            <div class="metric-value">
              {{ formatNumber(activeUsers.current) }}
            </div>
            <div
              class="metric-change"
              :class="getChangeClass(activeUsers.change)"
            >
              {{ activeUsers.change > 0 ? '+' : '' }}{{ formatNumber(activeUsers.change) }}
            </div>
          </el-card>
          
          <el-card class="metric-card">
            <div class="metric-header">
              <h3>Storage Usage</h3>
              <el-icon class="metric-icon storage-icon">
                <FolderOpened />
              </el-icon>
            </div>
            <div class="metric-value">
              {{ storageUsage.used }}
            </div>
            <div
              class="metric-change"
              :class="getChangeClass(storageUsage.changePercent)"
            >
              {{ storageUsage.changePercent > 0 ? '+' : '' }}{{ storageUsage.changePercent }}%
            </div>
          </el-card>
          
          <el-card class="metric-card">
            <div class="metric-header">
              <h3>Network Throughput</h3>
              <el-icon class="metric-icon throughput-icon">
                <TrendCharts />
              </el-icon>
            </div>
            <div class="metric-value">
              {{ networkThroughput.current }}
            </div>
            <div
              class="metric-change"
              :class="getChangeClass(networkThroughput.change)"
            >
              {{ networkThroughput.change > 0 ? '+' : '' }}{{ networkThroughput.change }}%
            </div>
          </el-card>
        </div>
      </div>
      
      <div class="charts-section">
        <div class="charts-grid">
          <el-card class="chart-card">
            <div class="chart-header">
              <h3>Performance Metrics</h3>
              <div class="chart-controls">
                <el-radio-group
                  v-model="performanceMetric"
                  size="small"
                >
                  <el-radio-button label="response_time">
                    Response Time
                  </el-radio-button>
                  <el-radio-button label="throughput">
                    Throughput
                  </el-radio-button>
                  <el-radio-button label="error_rate">
                    Error Rate
                  </el-radio-button>
                </el-radio-group>
              </div>
            </div>
            <div class="chart-content">
              <canvas
                ref="performanceChart"
                height="300"
              />
            </div>
          </el-card>
          
          <el-card class="chart-card">
            <div class="chart-header">
              <h3>Usage Statistics</h3>
              <div class="chart-controls">
                <el-radio-group
                  v-model="usageMetric"
                  size="small"
                >
                  <el-radio-button label="users">
                    Users
                  </el-radio-button>
                  <el-radio-button label="files">
                    Files
                  </el-radio-button>
                  <el-radio-button label="bandwidth">
                    Bandwidth
                  </el-radio-button>
                </el-radio-group>
              </div>
            </div>
            <div class="chart-content">
              <canvas
                ref="usageChart"
                height="300"
              />
            </div>
          </el-card>
        </div>
        
        <div class="charts-grid">
          <el-card class="chart-card">
            <div class="chart-header">
              <h3>Network Health</h3>
            </div>
            <div class="chart-content">
              <div class="health-grid">
                <div class="health-item">
                  <div class="health-label">
                    Bootstrap Nodes
                  </div>
                  <div class="health-status">
                    <el-progress
                      :percentage="networkHealth.bootstrapNodes"
                      :stroke-width="8"
                      :color="getHealthColor(networkHealth.bootstrapNodes)"
                    />
                    <span class="health-value">{{ networkHealth.bootstrapNodes }}%</span>
                  </div>
                </div>
                
                <div class="health-item">
                  <div class="health-label">
                    Peer Connectivity
                  </div>
                  <div class="health-status">
                    <el-progress
                      :percentage="networkHealth.peerConnectivity"
                      :stroke-width="8"
                      :color="getHealthColor(networkHealth.peerConnectivity)"
                    />
                    <span class="health-value">{{ networkHealth.peerConnectivity }}%</span>
                  </div>
                </div>
                
                <div class="health-item">
                  <div class="health-label">
                    Data Availability
                  </div>
                  <div class="health-status">
                    <el-progress
                      :percentage="networkHealth.dataAvailability"
                      :stroke-width="8"
                      :color="getHealthColor(networkHealth.dataAvailability)"
                    />
                    <span class="health-value">{{ networkHealth.dataAvailability }}%</span>
                  </div>
                </div>
                
                <div class="health-item">
                  <div class="health-label">
                    Consensus Health
                  </div>
                  <div class="health-status">
                    <el-progress
                      :percentage="networkHealth.consensusHealth"
                      :stroke-width="8"
                      :color="getHealthColor(networkHealth.consensusHealth)"
                    />
                    <span class="health-value">{{ networkHealth.consensusHealth }}%</span>
                  </div>
                </div>
              </div>
            </div>
          </el-card>
          
          <el-card class="chart-card">
            <div class="chart-header">
              <h3>Resource Distribution</h3>
            </div>
            <div class="chart-content">
              <canvas
                ref="distributionChart"
                height="300"
              />
            </div>
          </el-card>
        </div>
      </div>
      
      <div class="detailed-metrics">
        <el-card class="metrics-table-card">
          <div class="table-header">
            <h3>Detailed Metrics</h3>
            <div class="table-controls">
              <el-input
                v-model="searchQuery"
                placeholder="Search metrics..."
                clearable
                @input="filterMetrics"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </div>
          </div>
          
          <el-table
            :data="filteredDetailedMetrics"
            style="width: 100%"
            @sort-change="handleSortChange"
          >
            <el-table-column
              prop="metric"
              label="Metric"
              min-width="200"
              sortable
            />
            <el-table-column
              prop="current"
              label="Current Value"
              width="150"
              sortable
            />
            <el-table-column
              prop="average"
              label="Average"
              width="120"
              sortable
            />
            <el-table-column
              prop="peak"
              label="Peak"
              width="120"
              sortable
            />
            <el-table-column
              prop="change"
              label="Change"
              width="100"
              sortable
            >
              <template #default="{ row }">
                <span :class="getChangeClass(row.change)">
                  {{ row.change > 0 ? '+' : '' }}{{ row.change }}%
                </span>
              </template>
            </el-table-column>
            <el-table-column
              prop="status"
              label="Status"
              width="120"
            >
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="lastUpdated"
              label="Last Updated"
              width="180"
            >
              <template #default="{ row }">
                {{ formatDateTime(row.lastUpdated) }}
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </div>
      
      <div class="alerts-section">
        <el-card class="alerts-card">
          <div class="alerts-header">
            <h3>System Alerts</h3>
            <div class="alerts-controls">
              <el-select
                v-model="alertFilter"
                placeholder="Filter alerts"
                size="small"
              >
                <el-option
                  label="All"
                  value="all"
                />
                <el-option
                  label="Critical"
                  value="critical"
                />
                <el-option
                  label="Warning"
                  value="warning"
                />
                <el-option
                  label="Info"
                  value="info"
                />
              </el-select>
              <el-button
                size="small"
                @click="clearAlerts"
              >
                Clear All
              </el-button>
            </div>
          </div>
          
          <div class="alerts-list">
            <div
              v-for="alert in filteredAlerts"
              :key="alert.id"
              class="alert-item"
              :class="alert.severity"
            >
              <div class="alert-icon">
                <el-icon :class="getAlertIconClass(alert.severity)">
                  <component :is="getAlertIcon(alert.severity)" />
                </el-icon>
              </div>
              <div class="alert-content">
                <div class="alert-title">
                  {{ alert.title }}
                </div>
                <div class="alert-description">
                  {{ alert.description }}
                </div>
                <div class="alert-time">
                  {{ formatTimeAgo(alert.timestamp) }}
                </div>
              </div>
              <div class="alert-actions">
                <el-button
                  size="small"
                  @click="acknowledgeAlert(alert.id)"
                >
                  Acknowledge
                </el-button>
                <el-button
                  size="small"
                  @click="dismissAlert(alert.id)"
                >
                  Dismiss
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  Refresh,
  Download,
  User,
  FolderOpened,
  TrendCharts,
  Search,
  CircleCheckFilled,
  CircleCloseFilled,
  Warning,
  InfoFilled
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

export default {
  name: 'Analytics',
  components: {
    Refresh,
    Download,
    User,
    FolderOpened,
    TrendCharts,
    Search,
    CircleCheckFilled,
    CircleCloseFilled,
    Warning,
    InfoFilled
  },
  setup() {
    const loading = ref(false)
    const dateRange = ref([
      dayjs().subtract(7, 'days').format('YYYY-MM-DD HH:mm:ss'),
      dayjs().format('YYYY-MM-DD HH:mm:ss')
    ])
    const selectedMetric = ref('all')
    const performanceMetric = ref('response_time')
    const usageMetric = ref('users')
    const searchQuery = ref('')
    const alertFilter = ref('all')
    
    const performanceChart = ref()
    const usageChart = ref()
    const distributionChart = ref()
    
    const availableMetrics = [
      { label: 'All Metrics', value: 'all' },
      { label: 'Performance', value: 'performance' },
      { label: 'Usage', value: 'usage' },
      { label: 'Network', value: 'network' },
      { label: 'Storage', value: 'storage' }
    ]
    
    const systemHealth = reactive({
      status: 'Healthy',
      score: 95,
      change: 2
    })
    
    const activeUsers = reactive({
      current: 12547,
      change: 1284
    })
    
    const storageUsage = reactive({
      used: '2.4TB',
      total: '10TB',
      changePercent: 8
    })
    
    const networkThroughput = reactive({
      current: '1.2 Gbps',
      change: 15
    })
    
    const networkHealth = reactive({
      bootstrapNodes: 98,
      peerConnectivity: 94,
      dataAvailability: 97,
      consensusHealth: 92
    })
    
    const detailedMetrics = reactive([
      {
        metric: 'Average Response Time',
        current: '245ms',
        average: '280ms',
        peak: '1.2s',
        change: -12,
        status: 'Good',
        lastUpdated: new Date()
      },
      {
        metric: 'Request Success Rate',
        current: '99.2%',
        average: '98.8%',
        peak: '99.9%',
        change: 0.4,
        status: 'Excellent',
        lastUpdated: new Date()
      },
      {
        metric: 'Bandwidth Utilization',
        current: '78%',
        average: '65%',
        peak: '95%',
        change: 13,
        status: 'Warning',
        lastUpdated: new Date()
      },
      {
        metric: 'File Upload Rate',
        current: '1,247/hr',
        average: '1,156/hr',
        peak: '2,341/hr',
        change: 8,
        status: 'Good',
        lastUpdated: new Date()
      },
      {
        metric: 'Error Rate',
        current: '0.3%',
        average: '0.5%',
        peak: '2.1%',
        change: -0.2,
        status: 'Excellent',
        lastUpdated: new Date()
      }
    ])
    
    const alerts = reactive([
      {
        id: 1,
        severity: 'critical',
        title: 'High Memory Usage',
        description: 'Bootstrap node memory usage exceeds 85% threshold',
        timestamp: new Date(Date.now() - 15 * 60 * 1000)
      },
      {
        id: 2,
        severity: 'warning',
        title: 'Slow Response Time',
        description: 'Average response time increased by 20% in the last hour',
        timestamp: new Date(Date.now() - 45 * 60 * 1000)
      },
      {
        id: 3,
        severity: 'info',
        title: 'New Peer Connected',
        description: 'A new peer has successfully joined the network',
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000)
      },
      {
        id: 4,
        severity: 'warning',
        title: 'Storage Quota Warning',
        description: 'Free tier storage usage approaching 80% limit',
        timestamp: new Date(Date.now() - 3 * 60 * 60 * 1000)
      }
    ])
    
    const filteredDetailedMetrics = computed(() => {
      if (!searchQuery.value) return detailedMetrics
      return detailedMetrics.filter(metric =>
        metric.metric.toLowerCase().includes(searchQuery.value.toLowerCase())
      )
    })
    
    const filteredAlerts = computed(() => {
      if (alertFilter.value === 'all') return alerts
      return alerts.filter(alert => alert.severity === alertFilter.value)
    })
    
    const handleDateRangeChange = (value) => {
      console.log('Date range changed:', value)
      refreshData()
    }
    
    const handleMetricChange = (value) => {
      console.log('Metric changed:', value)
      refreshData()
    }
    
    const refreshData = async () => {
      loading.value = true
      try {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 1000))
        
        // Update metrics with new data
        systemHealth.score = Math.floor(Math.random() * 10) + 90
        systemHealth.change = Math.floor(Math.random() * 10) - 5
        
        activeUsers.current = Math.floor(Math.random() * 5000) + 10000
        activeUsers.change = Math.floor(Math.random() * 2000) - 1000
        
        ElMessage.success('Data refreshed successfully')
      } catch (error) {
        ElMessage.error('Failed to refresh data')
      } finally {
        loading.value = false
      }
    }
    
    const exportData = () => {
      // Create CSV data
      const csvData = [
        ['Metric', 'Current', 'Average', 'Peak', 'Change', 'Status', 'Last Updated'],
        ...detailedMetrics.map(m => [
          m.metric,
          m.current,
          m.average,
          m.peak,
          m.change + '%',
          m.status,
          formatDateTime(m.lastUpdated)
        ])
      ]
      
      const csvContent = csvData.map(row => row.join(',')).join('\n')
      const blob = new Blob([csvContent], { type: 'text/csv' })
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `analytics-${dayjs().format('YYYY-MM-DD')}.csv`
      link.click()
      window.URL.revokeObjectURL(url)
      
      ElMessage.success('Data exported successfully')
    }
    
    const filterMetrics = () => {
      // Filtering is handled by computed property
    }
    
    const handleSortChange = ({ column, prop, order }) => {
      console.log('Sort changed:', column, prop, order)
    }
    
    const acknowledgeAlert = (alertId) => {
      const alert = alerts.find(a => a.id === alertId)
      if (alert) {
        alert.acknowledged = true
        ElMessage.success('Alert acknowledged')
      }
    }
    
    const dismissAlert = (alertId) => {
      const index = alerts.findIndex(a => a.id === alertId)
      if (index > -1) {
        alerts.splice(index, 1)
        ElMessage.success('Alert dismissed')
      }
    }
    
    const clearAlerts = () => {
      alerts.splice(0, alerts.length)
      ElMessage.success('All alerts cleared')
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
    
    const getHealthColor = (percentage) => {
      if (percentage >= 90) return '#67C23A'
      if (percentage >= 70) return '#E6A23C'
      return '#F56C6C'
    }
    
    const getChangeClass = (change) => {
      if (change > 0) return 'positive-change'
      if (change < 0) return 'negative-change'
      return 'neutral-change'
    }
    
    const getStatusType = (status) => {
      const typeMap = {
        'Excellent': 'success',
        'Good': 'success',
        'Warning': 'warning',
        'Critical': 'danger'
      }
      return typeMap[status] || 'info'
    }
    
    const getAlertIcon = (severity) => {
      const iconMap = {
        'critical': CircleCloseFilled,
        'warning': Warning,
        'info': InfoFilled
      }
      return iconMap[severity] || InfoFilled
    }
    
    const getAlertIconClass = (severity) => {
      const classMap = {
        'critical': 'alert-icon-critical',
        'warning': 'alert-icon-warning',
        'info': 'alert-icon-info'
      }
      return classMap[severity] || 'alert-icon-info'
    }
    
    const formatNumber = (num) => {
      return num.toLocaleString()
    }
    
    const formatDateTime = (date) => {
      return dayjs(date).format('YYYY-MM-DD HH:mm:ss')
    }
    
    const formatTimeAgo = (timestamp) => {
      return dayjs(timestamp).fromNow()
    }
    
    let refreshInterval = null
    
    onMounted(() => {
      refreshData()
      
      // Set up auto-refresh every 30 seconds
      refreshInterval = setInterval(refreshData, 30000)
    })
    
    onUnmounted(() => {
      if (refreshInterval) {
        clearInterval(refreshInterval)
      }
    })
    
    return {
      loading,
      dateRange,
      selectedMetric,
      performanceMetric,
      usageMetric,
      searchQuery,
      alertFilter,
      performanceChart,
      usageChart,
      distributionChart,
      availableMetrics,
      systemHealth,
      activeUsers,
      storageUsage,
      networkThroughput,
      networkHealth,
      detailedMetrics,
      alerts,
      filteredDetailedMetrics,
      filteredAlerts,
      handleDateRangeChange,
      handleMetricChange,
      refreshData,
      exportData,
      filterMetrics,
      handleSortChange,
      acknowledgeAlert,
      dismissAlert,
      clearAlerts,
      getHealthIcon,
      getHealthIconClass,
      getHealthColor,
      getChangeClass,
      getStatusType,
      getAlertIcon,
      getAlertIconClass,
      formatNumber,
      formatDateTime,
      formatTimeAgo
    }
  }
}
</script>

<style scoped>
.analytics-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.analytics-header {
  padding: 24px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.analytics-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px 0;
}

.analytics-subtitle {
  color: var(--el-text-color-regular);
  margin: 0 0 20px 0;
}

.analytics-controls {
  display: flex;
  gap: 12px;
  align-items: center;
}

.analytics-content {
  flex: 1;
  overflow: auto;
  padding: 24px;
}

.metrics-overview {
  margin-bottom: 32px;
}

.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.metric-card {
  padding: 20px;
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.metric-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
  font-size: 16px;
}

.metric-icon {
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

.user-icon {
  color: var(--el-color-primary);
}

.storage-icon {
  color: var(--el-color-info);
}

.throughput-icon {
  color: var(--el-color-warning);
}

.metric-value {
  font-size: 32px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
}

.metric-change {
  font-size: 14px;
  font-weight: 500;
}

.positive-change {
  color: var(--el-color-success);
}

.negative-change {
  color: var(--el-color-danger);
}

.neutral-change {
  color: var(--el-text-color-secondary);
}

.charts-section {
  margin-bottom: 32px;
}

.charts-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
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

.health-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.health-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.health-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.health-status {
  display: flex;
  align-items: center;
  gap: 12px;
}

.health-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.detailed-metrics {
  margin-bottom: 32px;
}

.metrics-table-card {
  padding: 20px;
}

.table-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.table-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.table-controls {
  width: 300px;
}

.alerts-section {
  margin-bottom: 32px;
}

.alerts-card {
  padding: 20px;
}

.alerts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.alerts-header h3 {
  margin: 0;
  color: var(--el-text-color-primary);
}

.alerts-controls {
  display: flex;
  gap: 8px;
}

.alerts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.alert-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  border-radius: 8px;
  border-left: 4px solid;
}

.alert-item.critical {
  background: var(--el-color-danger-light-9);
  border-left-color: var(--el-color-danger);
}

.alert-item.warning {
  background: var(--el-color-warning-light-9);
  border-left-color: var(--el-color-warning);
}

.alert-item.info {
  background: var(--el-color-info-light-9);
  border-left-color: var(--el-color-info);
}

.alert-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.alert-icon-critical {
  color: var(--el-color-danger);
}

.alert-icon-warning {
  color: var(--el-color-warning);
}

.alert-icon-info {
  color: var(--el-color-info);
}

.alert-content {
  flex: 1;
}

.alert-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.alert-description {
  color: var(--el-text-color-regular);
  margin-bottom: 4px;
}

.alert-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.alert-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .analytics-controls {
    flex-direction: column;
    align-items: stretch;
  }
  
  .overview-cards {
    grid-template-columns: 1fr;
  }
  
  .charts-grid {
    grid-template-columns: 1fr;
  }
  
  .health-grid {
    grid-template-columns: 1fr;
  }
  
  .table-header {
    flex-direction: column;
    gap: 12px;
  }
  
  .table-controls {
    width: 100%;
  }
  
  .alerts-header {
    flex-direction: column;
    gap: 12px;
  }
  
  .alert-item {
    flex-direction: column;
    gap: 12px;
  }
  
  .alert-actions {
    align-self: flex-end;
  }
}
</style>