<template>
  <div class="mobile-enhanced-interface">
    <!-- Adaptive Header -->
    <div class="mobile-header" :class="{ 'collapsed': headerCollapsed }">
      <div class="header-content">
        <div class="user-info">
          <div class="avatar">
            <img :src="userAvatar" :alt="userName" />
          </div>
          <div class="user-details" v-if="!headerCollapsed">
            <span class="user-name">{{ userName }}</span>
            <span class="user-tier">{{ userTier }}</span>
          </div>
        </div>
        
        <div class="quick-stats">
          <div class="stat-item" v-for="stat in quickStats" :key="stat.id">
            <div class="stat-icon">{{ stat.icon }}</div>
            <div class="stat-value" v-if="!headerCollapsed">{{ stat.value }}</div>
          </div>
        </div>
        
        <div class="header-actions">
          <el-button 
            :icon="Search" 
            circle 
            @click="showGlobalSearch = true"
            class="action-btn"
          />
          <el-button 
            :icon="Notification" 
            circle 
            @click="showNotifications = true"
            class="action-btn"
            :badge="notificationCount"
          />
        </div>
      </div>
    </div>

    <!-- Smart Navigation -->
    <div class="mobile-navigation" :class="navigationLayout">
      <div class="nav-container">
        <div 
          v-for="item in adaptiveNavItems"
          :key="item.id"
          class="nav-item"
          :class="{ 'active': currentRoute === item.route }"
          @click="navigateTo(item.route)"
        >
          <div class="nav-icon">
            <component :is="item.icon" />
          </div>
          <span class="nav-label" v-if="showLabels">{{ item.label }}</span>
          <div class="nav-indicator" v-if="item.hasUpdate"></div>
        </div>
      </div>
    </div>

    <!-- Main Content with Gesture Support -->
    <div class="mobile-content" @touchstart="handleTouchStart" @touchmove="handleTouchMove" @touchend="handleTouchEnd">
      <div class="content-wrapper" :style="contentTransform">
        <router-view v-slot="{ Component }">
          <component :is="Component" :mobile-optimized="true" />
        </router-view>
      </div>
    </div>

    <!-- Contextual Floating Actions -->
    <div class="floating-actions" :class="fabPosition">
      <el-button
        v-for="action in contextualActions"
        :key="action.id"
        :type="action.type"
        :icon="action.icon"
        circle
        size="large"
        @click="executeAction(action)"
        class="fab-button"
        :class="{ 'primary': action.primary }"
      />
    </div>

    <!-- Pull-to-Refresh -->
    <div class="pull-refresh" :class="{ 'active': pullRefreshActive, 'triggered': pullRefreshTriggered }">
      <div class="refresh-icon">
        <el-icon><RefreshRight /></el-icon>
      </div>
      <span class="refresh-text">{{ pullRefreshText }}</span>
    </div>

    <!-- Global Search Overlay -->
    <el-drawer
      v-model="showGlobalSearch"
      title="Search"
      direction="ttb"
      size="100%"
      class="search-drawer"
    >
      <MobileGlobalSearch @close="showGlobalSearch = false" />
    </el-drawer>

    <!-- Notification Panel -->
    <el-drawer
      v-model="showNotifications"
      title="Notifications"
      direction="rtl"
      size="300px"
      class="notification-drawer"
    >
      <MobileNotifications @close="showNotifications = false" />
    </el-drawer>

    <!-- One-Handed Mode Toggle -->
    <div class="one-handed-toggle" v-if="deviceSupportsOneHanded">
      <el-button
        :icon="oneHandedMode ? Expand : Compress"
        circle
        size="small"
        @click="toggleOneHandedMode"
        class="one-handed-btn"
      />
    </div>

    <!-- Haptic Feedback Controller -->
    <HapticFeedback ref="hapticController" />
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { useDeviceStore } from '@/stores/device'
import { useGestureRecognition } from '@/composables/useGestureRecognition'
import { useHapticFeedback } from '@/composables/useHapticFeedback'
import { useAdaptiveNavigation } from '@/composables/useAdaptiveNavigation'
import { 
  Search, 
  Notification, 
  RefreshRight, 
  Expand, 
  Compress,
  House,
  Folder,
  Setting,
  User,
  PieChart
} from '@element-plus/icons-vue'

// Mobile-specific components
import MobileGlobalSearch from './MobileGlobalSearch.vue'
import MobileNotifications from './MobileNotifications.vue'
import HapticFeedback from './HapticFeedback.vue'

export default {
  name: 'MobileEnhancedInterface',
  components: {
    MobileGlobalSearch,
    MobileNotifications,
    HapticFeedback,
    Search,
    Notification,
    RefreshRight,
    Expand,
    Compress,
    House,
    Folder,
    Setting,
    User,
    PieChart
  },
  setup() {
    const router = useRouter()
    const route = useRoute()
    const userStore = useUserStore()
    const deviceStore = useDeviceStore()
    
    const { 
      recognizeGesture, 
      setupGestureListeners,
      cleanupGestureListeners 
    } = useGestureRecognition()
    
    const { 
      triggerHaptic, 
      isHapticSupported 
    } = useHapticFeedback()
    
    const {
      getAdaptiveNavigation,
      getContextualActions,
      updateNavigationContext
    } = useAdaptiveNavigation()

    // Reactive state
    const headerCollapsed = ref(false)
    const showGlobalSearch = ref(false)
    const showNotifications = ref(false)
    const oneHandedMode = ref(false)
    const pullRefreshActive = ref(false)
    const pullRefreshTriggered = ref(false)
    const contentTransform = ref('')
    const fabPosition = ref('bottom-right')
    const navigationLayout = ref('bottom-tabs')
    const showLabels = ref(true)
    
    // Touch handling
    const touchStartY = ref(0)
    const touchStartX = ref(0)
    const touchCurrentY = ref(0)
    const touchCurrentX = ref(0)
    const isScrolling = ref(false)

    // Computed properties
    const currentRoute = computed(() => route.name)
    const userName = computed(() => userStore.user?.name || 'User')
    const userTier = computed(() => userStore.user?.tier || 'Free')
    const userAvatar = computed(() => userStore.user?.avatar || '/default-avatar.jpg')
    const notificationCount = computed(() => userStore.notifications?.length || 0)
    const deviceSupportsOneHanded = computed(() => deviceStore.screenHeight > 700)

    const quickStats = computed(() => [
      {
        id: 'storage',
        icon: '💾',
        value: `${Math.round(userStore.storageUsed / 1024**3)}GB`
      },
      {
        id: 'files',
        icon: '📁',
        value: userStore.fileCount
      },
      {
        id: 'tokens',
        icon: '🪙',
        value: userStore.tokens
      }
    ])

    const adaptiveNavItems = computed(() => {
      const baseItems = [
        {
          id: 'dashboard',
          label: 'Dashboard',
          icon: House,
          route: 'dashboard',
          hasUpdate: false
        },
        {
          id: 'files',
          label: 'Files',
          icon: Folder,
          route: 'files',
          hasUpdate: userStore.hasNewFiles
        },
        {
          id: 'analytics',
          label: 'Analytics',
          icon: PieChart,
          route: 'analytics',
          hasUpdate: false
        },
        {
          id: 'settings',
          label: 'Settings',
          icon: Setting,
          route: 'settings',
          hasUpdate: userStore.hasSettingsUpdate
        },
        {
          id: 'profile',
          label: 'Profile',
          icon: User,
          route: 'profile',
          hasUpdate: false
        }
      ]

      // Filter items based on user context and device
      return getAdaptiveNavigation(baseItems, {
        userLevel: userStore.userLevel,
        deviceType: deviceStore.deviceType,
        oneHandedMode: oneHandedMode.value,
        currentRoute: currentRoute.value
      })
    })

    const contextualActions = computed(() => {
      return getContextualActions({
        currentRoute: currentRoute.value,
        userBehavior: userStore.recentBehavior,
        deviceContext: {
          oneHandedMode: oneHandedMode.value,
          orientation: deviceStore.orientation
        }
      })
    })

    const pullRefreshText = computed(() => {
      if (pullRefreshTriggered.value) {
        return 'Release to refresh'
      }
      return pullRefreshActive.value ? 'Pull to refresh' : ''
    })

    // Methods
    const handleTouchStart = (event) => {
      touchStartY.value = event.touches[0].clientY
      touchStartX.value = event.touches[0].clientX
      isScrolling.value = false
    }

    const handleTouchMove = (event) => {
      if (!event.touches[0]) return
      
      touchCurrentY.value = event.touches[0].clientY
      touchCurrentX.value = event.touches[0].clientX
      
      const deltaY = touchCurrentY.value - touchStartY.value
      const deltaX = touchCurrentX.value - touchStartX.value
      
      // Detect scroll direction
      if (Math.abs(deltaY) > Math.abs(deltaX)) {
        isScrolling.value = true
        
        // Pull to refresh logic
        if (deltaY > 0 && window.scrollY === 0) {
          handlePullToRefresh(deltaY)
        }
        
        // Auto-hide header on scroll
        if (deltaY > 50 && !headerCollapsed.value) {
          headerCollapsed.value = true
        } else if (deltaY < -50 && headerCollapsed.value) {
          headerCollapsed.value = false
        }
      } else {
        // Horizontal swipe gestures
        recognizeGesture({ deltaX, deltaY, touchStartX: touchStartX.value })
      }
    }

    const handleTouchEnd = () => {
      if (pullRefreshActive.value) {
        if (pullRefreshTriggered.value) {
          performRefresh()
        } else {
          pullRefreshActive.value = false
        }
      }
      
      touchStartY.value = 0
      touchStartX.value = 0
      isScrolling.value = false
    }

    const handlePullToRefresh = (deltaY) => {
      const threshold = 80
      const maxPull = 120
      
      if (deltaY > threshold) {
        pullRefreshActive.value = true
        pullRefreshTriggered.value = deltaY > maxPull
        
        if (pullRefreshTriggered.value && isHapticSupported.value) {
          triggerHaptic('medium')
        }
      }
    }

    const performRefresh = async () => {
      pullRefreshActive.value = false
      pullRefreshTriggered.value = false
      
      try {
        await userStore.refreshData()
        triggerHaptic('success')
      } catch (error) {
        console.error('Refresh failed:', error)
        triggerHaptic('error')
      }
    }

    const navigateTo = (routeName) => {
      if (routeName !== currentRoute.value) {
        router.push({ name: routeName })
        triggerHaptic('light')
      }
    }

    const executeAction = (action) => {
      triggerHaptic('medium')
      action.execute()
    }

    const toggleOneHandedMode = () => {
      oneHandedMode.value = !oneHandedMode.value
      triggerHaptic('light')
      
      // Adjust interface for one-handed mode
      if (oneHandedMode.value) {
        navigationLayout.value = 'bottom-tabs'
        fabPosition.value = 'bottom-center'
        showLabels.value = false
        
        // Move content higher
        contentTransform.value = 'translateY(-100px)'
      } else {
        navigationLayout.value = deviceStore.screenWidth < 768 ? 'bottom-tabs' : 'side-drawer'
        fabPosition.value = 'bottom-right'
        showLabels.value = true
        contentTransform.value = ''
      }
      
      // Save preference
      localStorage.setItem('oneHandedMode', oneHandedMode.value.toString())
    }

    const optimizeForDevice = () => {
      const { screenWidth, screenHeight, deviceType } = deviceStore
      
      // Adjust navigation layout based on device
      if (screenWidth < 600) {
        navigationLayout.value = 'bottom-tabs'
        showLabels.value = false
      } else if (screenWidth < 900) {
        navigationLayout.value = 'bottom-tabs'
        showLabels.value = true
      } else {
        navigationLayout.value = 'side-drawer'
        showLabels.value = true
      }
      
      // Adjust FAB position based on device and orientation
      if (deviceType === 'phone') {
        fabPosition.value = oneHandedMode.value ? 'bottom-center' : 'bottom-right'
      } else {
        fabPosition.value = 'bottom-right'
      }
      
      // Load saved preferences
      const savedOneHandedMode = localStorage.getItem('oneHandedMode')
      if (savedOneHandedMode === 'true' && deviceSupportsOneHanded.value) {
        oneHandedMode.value = true
        toggleOneHandedMode()
      }
    }

    // Lifecycle
    onMounted(() => {
      optimizeForDevice()
      setupGestureListeners()
      updateNavigationContext(currentRoute.value)
    })

    onUnmounted(() => {
      cleanupGestureListeners()
    })

    // Watch for route changes
    watch(currentRoute, (newRoute) => {
      updateNavigationContext(newRoute)
    })

    // Watch for device changes
    watch(() => deviceStore.orientation, optimizeForDevice)
    watch(() => deviceStore.screenWidth, optimizeForDevice)

    return {
      // Reactive state
      headerCollapsed,
      showGlobalSearch,
      showNotifications,
      oneHandedMode,
      pullRefreshActive,
      pullRefreshTriggered,
      contentTransform,
      fabPosition,
      navigationLayout,
      showLabels,
      
      // Computed properties
      currentRoute,
      userName,
      userTier,
      userAvatar,
      notificationCount,
      deviceSupportsOneHanded,
      quickStats,
      adaptiveNavItems,
      contextualActions,
      pullRefreshText,
      
      // Methods
      handleTouchStart,
      handleTouchMove,
      handleTouchEnd,
      navigateTo,
      executeAction,
      toggleOneHandedMode
    }
  }
}
</script>

<style scoped>
.mobile-enhanced-interface {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f8f9fa;
  position: relative;
  overflow: hidden;
}

.mobile-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: env(safe-area-inset-top) 16px 16px;
  transition: all 0.3s ease;
  z-index: 100;
}

.mobile-header.collapsed {
  padding: env(safe-area-inset-top) 16px 8px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.avatar img {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.3);
}

.user-details {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-name {
  font-weight: 600;
  font-size: 0.9rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-tier {
  font-size: 0.7rem;
  opacity: 0.8;
}

.quick-stats {
  display: flex;
  gap: 8px;
  flex: 1;
  justify-content: center;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  backdrop-filter: blur(10px);
}

.stat-icon {
  font-size: 0.8rem;
}

.stat-value {
  font-size: 0.7rem;
  font-weight: 600;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  backdrop-filter: blur(10px);
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.mobile-navigation {
  background: white;
  border-top: 1px solid #e4e7ed;
  z-index: 99;
}

.mobile-navigation.bottom-tabs {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding-bottom: env(safe-area-inset-bottom);
}

.mobile-navigation.side-drawer {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 240px;
  border-right: 1px solid #e4e7ed;
  border-top: none;
}

.nav-container {
  display: flex;
  padding: 8px;
}

.bottom-tabs .nav-container {
  justify-content: space-around;
}

.side-drawer .nav-container {
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  min-width: 60px;
}

.side-drawer .nav-item {
  flex-direction: row;
  justify-content: flex-start;
  gap: 12px;
  min-width: auto;
  width: 100%;
}

.nav-item:hover {
  background: #f0f0f0;
}

.nav-item.active {
  background: #409EFF;
  color: white;
}

.nav-icon {
  font-size: 1.2rem;
  margin-bottom: 4px;
}

.side-drawer .nav-icon {
  margin-bottom: 0;
}

.nav-label {
  font-size: 0.7rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.nav-indicator {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 8px;
  height: 8px;
  background: #F56C6C;
  border-radius: 50%;
  border: 2px solid white;
}

.mobile-content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 80px; /* Space for bottom navigation */
  -webkit-overflow-scrolling: touch;
}

.side-drawer ~ .mobile-content {
  margin-left: 240px;
  padding-bottom: 20px;
}

.content-wrapper {
  transition: transform 0.3s ease;
}

.floating-actions {
  position: fixed;
  z-index: 98;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.floating-actions.bottom-right {
  bottom: 100px;
  right: 16px;
}

.floating-actions.bottom-center {
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  flex-direction: row;
}

.fab-button {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(10px);
}

.fab-button.primary {
  background: #409EFF;
  color: white;
}

.pull-refresh {
  position: fixed;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.9);
  border-radius: 0 0 16px 16px;
  backdrop-filter: blur(10px);
  transition: all 0.3s ease;
  opacity: 0;
  z-index: 101;
}

.pull-refresh.active {
  opacity: 1;
}

.pull-refresh.triggered {
  background: rgba(103, 194, 58, 0.9);
  color: white;
}

.refresh-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.refresh-text {
  font-size: 0.8rem;
  font-weight: 500;
}

.one-handed-toggle {
  position: fixed;
  top: 50%;
  right: 4px;
  transform: translateY(-50%);
  z-index: 97;
}

.one-handed-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #e4e7ed;
  backdrop-filter: blur(10px);
}

.search-drawer :deep(.el-drawer__body) {
  padding: 0;
}

.notification-drawer :deep(.el-drawer__body) {
  padding: 16px;
}

/* Responsive adjustments */
@media (max-width: 480px) {
  .user-details {
    display: none;
  }
  
  .quick-stats {
    gap: 4px;
  }
  
  .stat-item {
    padding: 4px 6px;
  }
  
  .nav-label {
    font-size: 0.6rem;
  }
  
  .floating-actions.bottom-right {
    bottom: 90px;
    right: 12px;
  }
  
  .fab-button {
    width: 48px;
    height: 48px;
  }
}

@media (orientation: landscape) and (max-height: 500px) {
  .mobile-header {
    padding: 8px 16px;
  }
  
  .mobile-navigation.bottom-tabs {
    padding: 4px 8px;
  }
  
  .nav-item {
    padding: 4px 8px;
  }
  
  .nav-icon {
    font-size: 1rem;
    margin-bottom: 2px;
  }
  
  .nav-label {
    font-size: 0.6rem;
  }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .mobile-enhanced-interface {
    background: #1a1a1a;
  }
  
  .mobile-navigation {
    background: #2d2d2d;
    border-top-color: #404040;
  }
  
  .nav-item:hover {
    background: #404040;
  }
  
  .pull-refresh {
    background: rgba(45, 45, 45, 0.9);
    color: white;
  }
  
  .one-handed-btn {
    background: rgba(45, 45, 45, 0.9);
    border-color: #404040;
    color: white;
  }
}

/* Accessibility improvements */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .mobile-header {
    background: #000;
    color: #fff;
  }
  
  .nav-item.active {
    background: #000;
    color: #fff;
  }
  
  .fab-button.primary {
    background: #000;
    color: #fff;
  }
}
</style>
