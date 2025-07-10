<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <div class="header-content">
        <div class="welcome-section">
          <h1>Dashboard</h1>
          <p>Welcome back, {{ authStore.currentUser?.name || 'User' }}!</p>
        </div>
        <div class="header-actions">
          <el-tooltip content="Refresh all data (Ctrl+R)">
            <el-button
              :loading="isRefreshing"
              @click="refreshDashboard"
            >
              <el-icon><Refresh /></el-icon>
            </el-button>
          </el-tooltip>
          <el-tooltip content="Settings">
            <el-button @click="$router.push('/settings')">
              <el-icon><Setting /></el-icon>
            </el-button>
          </el-tooltip>
        </div>
      </div>
      
      <!-- Breadcrumb navigation -->
      <el-breadcrumb
        separator="/"
        class="breadcrumb"
      >
        <el-breadcrumb-item :to="{ path: '/' }">
          Home
        </el-breadcrumb-item>
        <el-breadcrumb-item>Dashboard</el-breadcrumb-item>
      </el-breadcrumb>
    </div>
    
    <!-- Quick Stats -->
    <div class="stats-grid">
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon
              size="32"
              color="#409EFF"
            >
              <Files />
            </el-icon>
          </div>
          <div class="stat-details">
            <h3>{{ formatNumber(filesStore.stats.total_files) }}</h3>
            <p>Total Files</p>
          </div>
        </div>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon
              size="32"
              color="#67C23A"
            >
              <Coin />
            </el-icon>
          </div>
          <div class="stat-details">
            <h3>{{ formatBytes(filesStore.stats.total_storage_bytes) }}</h3>
            <p>Storage Used</p>
          </div>
        </div>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon
              size="32"
              color="#E6A23C"
            >
              <Connection />
            </el-icon>
          </div>
          <div class="stat-details">
            <h3>{{ governanceStore.networkHealth.online_operators }}</h3>
            <p>Active Operators</p>
          </div>
        </div>
      </el-card>
      
      <el-card class="stat-card">
        <div class="stat-content">
          <div class="stat-icon">
            <el-icon
              size="32"
              color="#F56C6C"
            >
              <TrendCharts />
            </el-icon>
          </div>
          <div class="stat-details">
            <h3>{{ Math.round(filesStore.stats.cache_hit_ratio * 100) }}%</h3>
            <p>Cache Hit Ratio</p>
          </div>
        </div>
      </el-card>
    </div>
    
    <!-- Main Content Grid -->
    <div class="dashboard-grid">
      <!-- Recent Files -->
      <el-card class="dashboard-card">
        <template #header>
          <div class="card-header">
            <h3>Recent Files</h3>
            <el-button
              type="primary"
              text
              @click="$router.push('/files')"
            >
              View All
            </el-button>
          </div>
        </template>
        
        <div
          v-if="filesStore.isLoading"
          class="loading-container"
        >
          <el-skeleton
            :rows="3"
            animated
          />
        </div>
        
        <div
          v-else-if="recentFiles.length === 0"
          class="empty-state"
        >
          <el-empty description="No files uploaded yet">
            <el-button
              type="primary"
              @click="$router.push('/files')"
            >
              Upload Files
            </el-button>
          </el-empty>
        </div>
        
        <div
          v-else
          class="file-list"
        >
          <div
            v-for="file in recentFiles.slice(0, 5)"
            :key="file.file_key"
            class="file-item"
            @click="downloadFile(file)"
          >
            <div class="file-icon">
              <el-icon :class="getFileIconClass(file.file_name)">
                <component :is="getFileIcon(file.file_name)" />
              </el-icon>
            </div>
            <div class="file-info">
              <div class="file-name">
                {{ file.file_name }}
              </div>
              <div class="file-meta">
                {{ formatBytes(file.file_size) }} • {{ formatTime(file.uploaded_at) }}
              </div>
            </div>
            <div class="file-actions">
              <el-button
                size="small"
                circle
                @click.stop="downloadFile(file)"
              >
                <el-icon><Download /></el-icon>
              </el-button>
            </div>
          </div>
        </div>
      </el-card>
      
      <!-- System Health -->
      <el-card class="dashboard-card">
        <template #header>
          <div class="card-header">
            <h3>System Health</h3>
            <el-button
              type="primary"
              text
              @click="$router.push('/governance/network-health')"
            >
              View Details
            </el-button>
          </div>
        </template>
        
        <div class="health-metrics">
          <div class="health-item">
            <div class="health-label">
              Network Status
            </div>
            <el-tag
              :type="governanceStore.isNetworkHealthy ? 'success' : 'danger'"
              size="large"
            >
              {{ governanceStore.isNetworkHealthy ? 'Healthy' : 'Unhealthy' }}
            </el-tag>
          </div>
          
          <div class="health-item">
            <div class="health-label">
              Consensus
            </div>
            <el-tag
              :type="governanceStore.canReachConsensus ? 'success' : 'warning'"
              size="large"
            >
              {{ governanceStore.canReachConsensus ? 'Available' : 'Limited' }}
            </el-tag>
          </div>
          
          <div class="health-item">
            <div class="health-label">
              Operators Online
            </div>
            <div class="health-value">
              {{ governanceStore.networkHealth.online_operators }}/{{ governanceStore.networkHealth.total_operators }}
            </div>
          </div>
          
          <div class="health-item">
            <div class="health-label">
              Governance Weight
            </div>
            <el-progress
              :percentage="Math.round((governanceStore.onlineGovernanceWeight / governanceStore.totalGovernanceWeight) * 100)"
              :stroke-width="8"
              :show-text="false"
            />
          </div>
        </div>
      </el-card>
      
      <!-- Quick Actions -->
      <el-card class="dashboard-card">
        <template #header>
          <h3>Quick Actions</h3>
        </template>
        
        <div class="quick-actions">
          <el-button
            type="primary"
            size="large"
            class="action-button"
            @click="$router.push('/files')"
          >
            <el-icon><Upload /></el-icon>
            Upload Files
          </el-button>
          
          <el-button
            type="success"
            size="large"
            class="action-button"
            @click="$router.push('/governance')"
          >
            <el-icon><Flag /></el-icon>
            Governance
          </el-button>
          
          <el-button
            type="info"
            size="large"
            class="action-button"
            @click="$router.push('/analytics')"
          >
            <el-icon><TrendCharts /></el-icon>
            Analytics
          </el-button>
          
          <el-button
            v-if="authStore.isAdmin"
            type="warning"
            size="large"
            class="action-button"
            @click="$router.push('/administration')"
          >
            <el-icon><Setting /></el-icon>
            Administration
          </el-button>
        </div>
      </el-card>
      
      <!-- Activity Feed -->
      <el-card class="dashboard-card activity-feed">
        <template #header>
          <h3>Recent Activity</h3>
        </template>
        
        <el-timeline class="activity-timeline">
          <el-timeline-item
            v-for="activity in recentActivity"
            :key="activity.id"
            :timestamp="formatTime(activity.timestamp)"
            :type="activity.type"
            :icon="activity.icon"
          >
            <div class="activity-content">
              <div class="activity-title">
                {{ activity.title }}
              </div>
              <div class="activity-description">
                {{ activity.description }}
              </div>
            </div>
          </el-timeline-item>
          
          <el-timeline-item
            v-if="recentActivity.length === 0"
            timestamp="No recent activity"
            type="info"
          >
            <div class="activity-content">
              <div class="activity-title">
                Welcome to DataMesh
              </div>
              <div class="activity-description">
                Start by uploading some files or exploring the governance system.
              </div>
            </div>
          </el-timeline-item>
        </el-timeline>
      </el-card>
    </div>
    
    <!-- Onboarding Dialog for New Users -->
    <el-dialog
      v-model="showOnboarding"
      title="Welcome to DataMesh!"
      width="600px"
      :close-on-click-modal="false"
    >
      <div class="onboarding-content">
        <div class="onboarding-step">
          <el-icon
            class="step-icon"
            size="48"
            color="#409EFF"
          >
            <Upload />
          </el-icon>
          <h3>Upload Your First File</h3>
          <p>Start by uploading files to the distributed storage network. Your files are automatically encrypted and distributed across multiple nodes for maximum security and availability.</p>
          <el-button
            type="primary"
            @click="$router.push('/files'); showOnboarding = false"
          >
            Go to File Manager
          </el-button>
        </div>
        
        <div class="onboarding-step">
          <el-icon
            class="step-icon"
            size="48"
            color="#67C23A"
          >
            <Flag />
          </el-icon>
          <h3>Explore Governance</h3>
          <p>Participate in network governance by reviewing operator proposals and contributing to the decentralized decision-making process.</p>
          <el-button @click="$router.push('/governance'); showOnboarding = false">
            View Governance
          </el-button>
        </div>
        
        <div class="onboarding-step">
          <el-icon
            class="step-icon"
            size="48"
            color="#E6A23C"
          >
            <TrendCharts />
          </el-icon>
          <h3>Monitor Analytics</h3>
          <p>Track your storage usage, bandwidth consumption, and network participation through detailed analytics and reports.</p>
          <el-button @click="$router.push('/analytics'); showOnboarding = false">
            View Analytics
          </el-button>
        </div>
      </div>
      
      <template #footer>
        <div class="onboarding-footer">
          <el-checkbox v-model="dontShowAgain">
            Don't show this again
          </el-checkbox>
          <el-button @click="closeOnboarding">
            Skip Tour
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useAuthStore } from '../store/auth'
import { useFilesStore } from '../store/files'
import { useGovernanceStore } from '../store/governance'
import { useWebSocketStore } from '../store/websocket'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'
import { Refresh, Setting, Upload, Flag, TrendCharts, Files, Coin, Connection, Download } from '@element-plus/icons-vue'

export default {
  name: 'Dashboard',
  components: {
    Refresh,
    Setting,
    Upload,
    Flag,
    TrendCharts,
    Files,
    Coin,
    Connection,
    Download
  },
  setup() {
    const authStore = useAuthStore()
    const filesStore = useFilesStore()
    const governanceStore = useGovernanceStore()
    const webSocketStore = useWebSocketStore()
    
    // State
    const recentActivity = ref([])
    const refreshInterval = ref(null)
    const isRefreshing = ref(false)
    const showOnboarding = ref(false)
    const dontShowAgain = ref(false)
    
    // Computed
    const recentFiles = computed(() => filesStore.getRecentFiles.slice(0, 5))
    
    // Methods
    const formatNumber = (num) => {
      if (num >= 1000000) {
        return (num / 1000000).toFixed(1) + 'M'
      } else if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'K'
      }
      return num.toString()
    }
    
    const formatBytes = (bytes) => {
      if (bytes === 0) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }
    
    const formatTime = (timestamp) => {
      return dayjs(timestamp).fromNow()
    }
    
    const getFileIcon = (filename) => {
      const ext = filename.split('.').pop()?.toLowerCase()
      switch (ext) {
        case 'jpg':
        case 'jpeg':
        case 'png':
        case 'gif':
        case 'svg':
          return 'Picture'
        case 'pdf':
          return 'Document'
        case 'doc':
        case 'docx':
          return 'Document'
        case 'zip':
        case 'rar':
        case '7z':
          return 'FolderOpened'
        case 'mp4':
        case 'avi':
        case 'mov':
          return 'VideoPlay'
        case 'mp3':
        case 'wav':
        case 'flac':
          return 'Headphone'
        case 'js':
        case 'ts':
        case 'py':
        case 'java':
        case 'cpp':
          return 'Document'
        default:
          return 'Document'
      }
    }
    
    const getFileIconClass = (filename) => {
      const ext = filename.split('.').pop()?.toLowerCase()
      switch (ext) {
        case 'jpg':
        case 'jpeg':
        case 'png':
        case 'gif':
        case 'svg':
          return 'file-icon-image'
        case 'pdf':
        case 'doc':
        case 'docx':
          return 'file-icon-document'
        case 'zip':
        case 'rar':
        case '7z':
          return 'file-icon-archive'
        case 'mp4':
        case 'avi':
        case 'mov':
          return 'file-icon-video'
        case 'mp3':
        case 'wav':
        case 'flac':
          return 'file-icon-audio'
        case 'js':
        case 'ts':
        case 'py':
        case 'java':
        case 'cpp':
          return 'file-icon-code'
        default:
          return 'file-icon-default'
      }
    }
    
    const downloadFile = async (file) => {
      try {
        await filesStore.downloadFile(file.file_key, file.file_name)
        ElMessage.success('File download started')
      } catch (error) {
        ElMessage.error('Failed to download file')
      }
    }
    
    const loadDashboardData = async () => {
      try {
        await Promise.all([
          filesStore.fetchStats(),
          filesStore.fetchFiles({ page: 1, page_size: 10 }),
          governanceStore.fetchNetworkHealth(),
          governanceStore.fetchGovernanceStatus()
        ])
      } catch (error) {
        console.error('Failed to load dashboard data:', error)
        ElMessage.error('Failed to load dashboard data')
      }
    }
    
    const refreshDashboard = async () => {
      isRefreshing.value = true
      try {
        await loadDashboardData()
        ElMessage.success('Dashboard refreshed')
      } finally {
        isRefreshing.value = false
      }
    }
    
    const closeOnboarding = () => {
      showOnboarding.value = false
      if (dontShowAgain.value) {
        localStorage.setItem('datamesh_skip_onboarding', 'true')
      }
    }
    
    const addActivity = (activity) => {
      recentActivity.value.unshift({
        id: Date.now(),
        timestamp: new Date(),
        ...activity
      })
      
      // Keep only last 20 activities
      if (recentActivity.value.length > 20) {
        recentActivity.value = recentActivity.value.slice(0, 20)
      }
    }
    
    // WebSocket event handlers for proper cleanup
    const webSocketHandlers = {
      file_uploaded: (data) => {
        addActivity({
          type: 'success',
          icon: 'Upload',
          title: 'File Uploaded',
          description: `${data.file_name} was uploaded successfully`
        })
      },
      file_deleted: (data) => {
        addActivity({
          type: 'warning',
          icon: 'Delete',
          title: 'File Deleted',
          description: `${data.file_name} was deleted`
        })
      },
      governance_update: (data) => {
        addActivity({
          type: 'info',
          icon: 'Flag',
          title: 'Governance Update',
          description: data.message || 'Governance system updated'
        })
      },
      operator_status_change: (data) => {
        addActivity({
          type: 'primary',
          icon: 'Connection',
          title: 'Operator Status Change',
          description: `Operator ${data.operator_id} is now ${data.status}`
        })
      },
      admin_action_executed: (data) => {
        addActivity({
          type: 'warning',
          icon: 'Setting',
          title: 'Admin Action',
          description: `${data.action_type} action executed`
        })
      }
    }
    
    const setupWebSocketListeners = () => {
      // Register all WebSocket event handlers
      Object.entries(webSocketHandlers).forEach(([event, handler]) => {
        webSocketStore.on(event, handler)
      })
    }
    
    const cleanupWebSocketListeners = () => {
      // Remove all WebSocket event handlers
      Object.entries(webSocketHandlers).forEach(([event, handler]) => {
        webSocketStore.off(event, handler)
      })
    }
    
    const startAutoRefresh = () => {
      refreshInterval.value = setInterval(() => {
        loadDashboardData()
      }, 30000) // Refresh every 30 seconds
    }
    
    const stopAutoRefresh = () => {
      if (refreshInterval.value) {
        clearInterval(refreshInterval.value)
        refreshInterval.value = null
      }
    }
    
    // Check if user is new (no files uploaded)
    const isNewUser = computed(() => {
      return filesStore.stats.total_files === 0
    })
    
    // Keyboard shortcuts
    const handleKeydown = (event) => {
      if (event.ctrlKey || event.metaKey) {
        switch (event.key) {
          case 'r':
            event.preventDefault()
            refreshDashboard()
            break
        }
      }
    }
    
    // Lifecycle
    onMounted(async () => {
      await loadDashboardData()
      setupWebSocketListeners()
      startAutoRefresh()
      document.addEventListener('keydown', handleKeydown)
      
      // Show onboarding for new users
      if (isNewUser.value && !localStorage.getItem('datamesh_skip_onboarding')) {
        showOnboarding.value = true
      }
    })
    
    onUnmounted(() => {
      // Stop auto refresh
      stopAutoRefresh()
      
      // Clean up WebSocket listeners
      cleanupWebSocketListeners()
      
      // Clear activity data
      recentActivity.value = []
      
      // Remove keyboard listener
      document.removeEventListener('keydown', handleKeydown)
    })
    
    return {
      // Stores
      authStore,
      filesStore,
      governanceStore,
      
      // State
      recentActivity,
      
      // Computed
      recentFiles,
      
      // Methods
      formatNumber,
      formatBytes,
      formatTime,
      getFileIcon,
      getFileIconClass,
      downloadFile,
      
      // Dashboard improvements
      isRefreshing,
      refreshDashboard,
      showOnboarding,
      dontShowAgain,
      closeOnboarding,
      isNewUser
    }
  }
}
</script>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  margin-bottom: 24px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.welcome-section h1 {
  margin: 0 0 8px 0;
  color: var(--el-text-color-primary);
}

.welcome-section p {
  margin: 0;
  color: var(--el-text-color-secondary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.breadcrumb {
  border-top: 1px solid var(--el-border-color-lighter);
  padding-top: 12px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  border: 1px solid var(--el-border-color-lighter);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  flex-shrink: 0;
}

.stat-details h3 {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.stat-details p {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
}

.dashboard-card {
  border: 1px solid var(--el-border-color-lighter);
  height: fit-content;
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

.loading-container {
  padding: 16px 0;
}

.empty-state {
  text-align: center;
  padding: 24px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.file-item:hover {
  background: var(--el-fill-color-light);
}

.file-icon {
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.file-actions {
  flex-shrink: 0;
}

.health-metrics {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.health-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.health-label {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.health-value {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
}

.action-button {
  height: 60px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-weight: 500;
}

.activity-feed {
  grid-column: 1 / -1;
}

.activity-timeline {
  max-height: 400px;
  overflow-y: auto;
}

.activity-content {
  padding-left: 8px;
}

.activity-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.activity-description {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

/* Onboarding styles */
.onboarding-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-height: 60vh;
  overflow-y: auto;
}

.onboarding-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  transition: all 0.3s ease;
}

.onboarding-step:hover {
  border-color: var(--el-color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.step-icon {
  margin-bottom: 16px;
}

.onboarding-step h3 {
  margin: 0 0 12px 0;
  color: var(--el-text-color-primary);
}

.onboarding-step p {
  margin: 0 0 16px 0;
  color: var(--el-text-color-secondary);
  line-height: 1.6;
}

.onboarding-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
  
  .dashboard-grid {
    grid-template-columns: 1fr;
  }
  
  .quick-actions {
    grid-template-columns: 1fr;
  }
  
  .action-button {
    height: 50px;
  }
}
</style>