<template>
  <div class="main-layout">
    <!-- Sidebar -->
    <el-aside
      :width="sidebarCollapsed ? '64px' : '240px'"
      class="sidebar"
      :class="{ 'sidebar-collapsed': sidebarCollapsed }"
    >
      <div class="sidebar-content">
        <!-- Logo -->
        <div class="sidebar-header">
          <div class="logo" @click="$router.push('/')">
            <el-icon size="28" color="#409EFF">
              <DataBoard />
            </el-icon>
            <span v-if="!sidebarCollapsed" class="logo-text">DataMesh</span>
          </div>
        </div>
        
        <!-- Navigation Menu -->
        <el-menu
          :default-active="activeMenu"
          :collapse="sidebarCollapsed"
          :unique-opened="true"
          class="sidebar-menu"
          router
        >
          <el-menu-item index="/" route="/">
            <el-icon><House /></el-icon>
            <span>Dashboard</span>
          </el-menu-item>
          
          <el-menu-item index="/files" route="/files">
            <el-icon><FolderOpened /></el-icon>
            <span>File Manager</span>
          </el-menu-item>
          
          <el-sub-menu index="/governance">
            <template #title>
              <el-icon><Flag /></el-icon>
              <span>Governance</span>
            </template>
            <el-menu-item index="/governance" route="/governance">
              <el-icon><TrendCharts /></el-icon>
              <span>Overview</span>
            </el-menu-item>
            <el-menu-item index="/governance/operators" route="/governance/operators">
              <el-icon><UserFilled /></el-icon>
              <span>Operators</span>
            </el-menu-item>
            <el-menu-item index="/governance/proposals" route="/governance/proposals">
              <el-icon><DocumentCopy /></el-icon>
              <span>Proposals</span>
            </el-menu-item>
            <el-menu-item index="/governance/voting" route="/governance/voting">
              <el-icon><Select /></el-icon>
              <span>Voting</span>
            </el-menu-item>
            <el-menu-item index="/governance/network-health" route="/governance/network-health">
              <el-icon><Monitor /></el-icon>
              <span>Network Health</span>
            </el-menu-item>
          </el-sub-menu>
          
          <el-sub-menu v-if="authStore.isAdmin" index="/administration">
            <template #title>
              <el-icon><Setting /></el-icon>
              <span>Administration</span>
            </template>
            <el-menu-item index="/administration" route="/administration">
              <el-icon><DataAnalysis /></el-icon>
              <span>Overview</span>
            </el-menu-item>
            <el-menu-item index="/administration/users" route="/administration/users">
              <el-icon><User /></el-icon>
              <span>Users</span>
            </el-menu-item>
            <el-menu-item index="/administration/operators" route="/administration/operators">
              <el-icon><Connection /></el-icon>
              <span>Operators</span>
            </el-menu-item>
            <el-menu-item index="/administration/system" route="/administration/system">
              <el-icon><Tools /></el-icon>
              <span>System</span>
            </el-menu-item>
            <el-menu-item index="/administration/audit" route="/administration/audit">
              <el-icon><Document /></el-icon>
              <span>Audit Logs</span>
            </el-menu-item>
          </el-sub-menu>
          
          <el-menu-item index="/analytics" route="/analytics">
            <el-icon><TrendCharts /></el-icon>
            <span>Analytics</span>
          </el-menu-item>
          
          <el-menu-item index="/settings" route="/settings">
            <el-icon><Tools /></el-icon>
            <span>Settings</span>
          </el-menu-item>
        </el-menu>
        
        <!-- User info (collapsed sidebar) -->
        <div v-if="sidebarCollapsed" class="sidebar-user-mini">
          <el-dropdown trigger="click" placement="right">
            <div class="user-avatar-mini">
              <el-avatar :size="32" :src="authStore.currentUser?.avatar">
                <el-icon><User /></el-icon>
              </el-avatar>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="$router.push('/profile')">
                  <el-icon><User /></el-icon>
                  Profile
                </el-dropdown-item>
                <el-dropdown-item @click="$router.push('/settings')">
                  <el-icon><Tools /></el-icon>
                  Settings
                </el-dropdown-item>
                <el-dropdown-item divided @click="logout">
                  <el-icon><SwitchButton /></el-icon>
                  Logout
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </el-aside>
    
    <!-- Main Content -->
    <el-container class="main-container">
      <!-- Header -->
      <el-header class="main-header">
        <div class="header-left">
          <el-button
            :icon="sidebarCollapsed ? 'Expand' : 'Fold'"
            circle
            @click="toggleSidebar"
            class="sidebar-toggle"
          />
          
          <el-breadcrumb separator="/" class="breadcrumb">
            <el-breadcrumb-item
              v-for="item in breadcrumbItems"
              :key="item.path"
              :to="item.path"
            >
              {{ item.name }}
            </el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        
        <div class="header-right">
          <!-- Search -->
          <el-input
            v-model="searchQuery"
            placeholder="Search files..."
            class="search-input"
            clearable
            @keyup.enter="performSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          
          <!-- Notifications -->
          <el-dropdown trigger="click" placement="bottom-end">
            <el-button circle>
              <el-badge :value="notificationCount" :hidden="notificationCount === 0">
                <el-icon><Bell /></el-icon>
              </el-badge>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <div class="notification-header">
                  <span>Notifications</span>
                  <el-button text size="small" @click="clearNotifications">
                    Clear All
                  </el-button>
                </div>
                <el-dropdown-item
                  v-for="notification in notifications"
                  :key="notification.id"
                  @click="handleNotificationClick(notification)"
                >
                  <div class="notification-item">
                    <el-icon :color="getNotificationColor(notification.type)">
                      <component :is="getNotificationIcon(notification.type)" />
                    </el-icon>
                    <div class="notification-content">
                      <div class="notification-title">{{ notification.title }}</div>
                      <div class="notification-time">{{ formatTime(notification.timestamp) }}</div>
                    </div>
                  </div>
                </el-dropdown-item>
                <el-dropdown-item v-if="notifications.length === 0" disabled>
                  No new notifications
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          
          <!-- Theme Toggle -->
          <el-button
            circle
            @click="themeStore.toggleTheme()"
            :icon="themeStore.isDark ? 'Sunny' : 'Moon'"
          />
          
          <!-- Connection Status -->
          <el-tooltip
            :content="connectionStatusText"
            placement="bottom"
          >
            <el-icon
              :color="connectionStatusColor"
              size="20"
              class="connection-status"
            >
              <component :is="connectionStatusIcon" />
            </el-icon>
          </el-tooltip>
          
          <!-- User Menu -->
          <el-dropdown v-if="!sidebarCollapsed" trigger="click" placement="bottom-end">
            <div class="user-info">
              <el-avatar :size="32" :src="authStore.currentUser?.avatar">
                <el-icon><User /></el-icon>
              </el-avatar>
              <div class="user-details">
                <div class="user-name">{{ authStore.currentUser?.name || 'User' }}</div>
                <div class="user-role">{{ authStore.currentUser?.role || 'Member' }}</div>
              </div>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="$router.push('/profile')">
                  <el-icon><User /></el-icon>
                  Profile
                </el-dropdown-item>
                <el-dropdown-item @click="$router.push('/settings')">
                  <el-icon><Tools /></el-icon>
                  Settings
                </el-dropdown-item>
                <el-dropdown-item divided @click="logout">
                  <el-icon><SwitchButton /></el-icon>
                  Logout
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>
      
      <!-- Main Content Area -->
      <el-main class="main-content">
        <router-view />
      </el-main>
    </el-container>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../../store/auth'
import { useThemeStore } from '../../store/theme'
import { useWebSocketStore } from '../../store/websocket'
import { ElMessage, ElMessageBox } from 'element-plus'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

export default {
  name: 'MainLayout',
  setup() {
    const route = useRoute()
    const router = useRouter()
    const authStore = useAuthStore()
    const themeStore = useThemeStore()
    const webSocketStore = useWebSocketStore()
    
    // State
    const sidebarCollapsed = ref(localStorage.getItem('datamesh_sidebar_collapsed') === 'true')
    const searchQuery = ref('')
    const notifications = ref([])
    
    // Computed
    const activeMenu = computed(() => route.path)
    const notificationCount = computed(() => notifications.value.filter(n => !n.read).length)
    
    const breadcrumbItems = computed(() => {
      const items = []
      const pathParts = route.path.split('/').filter(Boolean)
      let currentPath = ''
      
      items.push({ name: 'Dashboard', path: '/' })
      
      pathParts.forEach(part => {
        currentPath += `/${part}`
        const routeRecord = router.resolve(currentPath)
        if (routeRecord.meta?.title) {
          items.push({
            name: routeRecord.meta.title,
            path: currentPath
          })
        }
      })
      
      return items
    })
    
    const connectionStatusText = computed(() => {
      const status = webSocketStore.connectionStatus
      switch (status) {
        case 'connected': return 'Connected to DataMesh'
        case 'connecting': return 'Connecting...'
        case 'error': return 'Connection error'
        default: return 'Disconnected'
      }
    })
    
    const connectionStatusColor = computed(() => {
      const status = webSocketStore.connectionStatus
      switch (status) {
        case 'connected': return 'var(--el-color-success)'
        case 'connecting': return 'var(--el-color-warning)'
        case 'error': return 'var(--el-color-danger)'
        default: return 'var(--el-color-info)'
      }
    })
    
    const connectionStatusIcon = computed(() => {
      const status = webSocketStore.connectionStatus
      switch (status) {
        case 'connected': return 'Connection'
        case 'connecting': return 'Loading'
        case 'error': return 'Warning'
        default: return 'Close'
      }
    })
    
    // Methods
    const toggleSidebar = () => {
      sidebarCollapsed.value = !sidebarCollapsed.value
      localStorage.setItem('datamesh_sidebar_collapsed', sidebarCollapsed.value.toString())
    }
    
    const logout = async () => {
      try {
        await ElMessageBox.confirm(
          'Are you sure you want to log out?',
          'Confirm Logout',
          {
            confirmButtonText: 'Logout',
            cancelButtonText: 'Cancel',
            type: 'warning'
          }
        )
        
        await authStore.logout()
        router.push('/auth/login')
        ElMessage.success('Logged out successfully')
      } catch (error) {
        if (error !== 'cancel') {
          ElMessage.error('Logout failed')
        }
      }
    }
    
    const performSearch = () => {
      if (searchQuery.value.trim()) {
        router.push({
          path: '/files',
          query: { search: searchQuery.value }
        })
      }
    }
    
    const handleNotificationClick = (notification) => {
      // Mark as read
      notification.read = true
      
      // Handle notification action
      if (notification.action) {
        router.push(notification.action)
      }
    }
    
    const clearNotifications = () => {
      notifications.value = []
    }
    
    const getNotificationIcon = (type) => {
      switch (type) {
        case 'success': return 'CircleCheck'
        case 'warning': return 'Warning'
        case 'error': return 'CircleClose'
        case 'info': return 'InfoFilled'
        default: return 'Bell'
      }
    }
    
    const getNotificationColor = (type) => {
      switch (type) {
        case 'success': return 'var(--el-color-success)'
        case 'warning': return 'var(--el-color-warning)'
        case 'error': return 'var(--el-color-danger)'
        case 'info': return 'var(--el-color-info)'
        default: return 'var(--el-color-primary)'
      }
    }
    
    const formatTime = (timestamp) => {
      return dayjs(timestamp).fromNow()
    }
    
    const addNotification = (notification) => {
      notifications.value.unshift({
        id: Date.now(),
        timestamp: new Date(),
        read: false,
        ...notification
      })
      
      // Keep only last 50 notifications
      if (notifications.value.length > 50) {
        notifications.value = notifications.value.slice(0, 50)
      }
    }
    
    // WebSocket event handlers
    const handleWebSocketEvents = () => {
      webSocketStore.on('file_uploaded', (data) => {
        addNotification({
          type: 'success',
          title: `File "${data.file_name}" uploaded successfully`,
          action: '/files'
        })
      })
      
      webSocketStore.on('file_deleted', (data) => {
        addNotification({
          type: 'info',
          title: `File "${data.file_name}" deleted`,
          action: '/files'
        })
      })
      
      webSocketStore.on('governance_update', (data) => {
        addNotification({
          type: 'info',
          title: 'Governance update',
          action: '/governance'
        })
      })
      
      webSocketStore.on('operator_status_change', (data) => {
        addNotification({
          type: 'warning',
          title: `Operator status changed: ${data.status}`,
          action: '/governance/operators'
        })
      })
      
      webSocketStore.on('admin_action_executed', (data) => {
        addNotification({
          type: 'info',
          title: `Admin action executed: ${data.action_type}`,
          action: '/administration'
        })
      })
    }
    
    // Lifecycle
    onMounted(() => {
      handleWebSocketEvents()
      
      // Subscribe to system updates
      webSocketStore.subscribeToSystemUpdates()
      webSocketStore.subscribeToGovernanceUpdates()
    })
    
    onUnmounted(() => {
      webSocketStore.unsubscribeFromSystemUpdates()
      webSocketStore.unsubscribeFromGovernanceUpdates()
    })
    
    // Watch for route changes to update breadcrumbs
    watch(() => route.path, () => {
      // Clear search on route change
      searchQuery.value = ''
    })
    
    return {
      // Stores
      authStore,
      themeStore,
      webSocketStore,
      
      // State
      sidebarCollapsed,
      searchQuery,
      notifications,
      
      // Computed
      activeMenu,
      notificationCount,
      breadcrumbItems,
      connectionStatusText,
      connectionStatusColor,
      connectionStatusIcon,
      
      // Methods
      toggleSidebar,
      logout,
      performSearch,
      handleNotificationClick,
      clearNotifications,
      getNotificationIcon,
      getNotificationColor,
      formatTime
    }
  }
}
</script>

<style scoped>
.main-layout {
  height: 100vh;
  display: flex;
}

.sidebar {
  background: var(--el-bg-color);
  border-right: 1px solid var(--el-border-color-lighter);
  transition: width 0.3s ease;
  overflow: hidden;
}

.sidebar-collapsed {
  width: 64px !important;
}

.sidebar-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  height: 60px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.logo {
  display: flex;
  align-items: center;
  cursor: pointer;
  transition: all 0.3s ease;
}

.logo:hover {
  transform: scale(1.05);
}

.logo-text {
  margin-left: 12px;
  font-size: 20px;
  font-weight: bold;
  color: var(--el-text-color-primary);
}

.sidebar-menu {
  flex: 1;
  border: none;
  background: transparent;
}

.sidebar-user-mini {
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.user-avatar-mini {
  display: flex;
  justify-content: center;
  cursor: pointer;
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.main-header {
  height: 60px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.sidebar-toggle {
  margin-right: 8px;
}

.breadcrumb {
  color: var(--el-text-color-secondary);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.search-input {
  width: 300px;
}

.notification-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  font-weight: 500;
}

.notification-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  width: 280px;
}

.notification-content {
  flex: 1;
}

.notification-title {
  font-size: 14px;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.notification-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.connection-status {
  margin-left: 8px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.user-details {
  display: flex;
  flex-direction: column;
}

.user-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.user-role {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.main-content {
  flex: 1;
  padding: 24px;
  background: var(--el-bg-color-page);
  overflow-y: auto;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .main-layout {
    flex-direction: column;
  }
  
  .sidebar {
    width: 100% !important;
    height: auto;
    order: 2;
  }
  
  .main-container {
    order: 1;
  }
  
  .search-input {
    width: 200px;
  }
  
  .main-content {
    padding: 16px;
  }
  
  .header-right {
    gap: 8px;
  }
  
  .user-details {
    display: none;
  }
}
</style>