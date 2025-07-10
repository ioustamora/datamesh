<template>
  <div id="app" :class="{ 'dark': isDark }">
    <el-config-provider :locale="locale" :size="componentSize">
      <router-view />
    </el-config-provider>
    
    <!-- Global notification container -->
    <GlobalNotifications />
    
    <!-- WebSocket connection status -->
    <ConnectionStatus />
    
    <!-- Loading overlay for global operations -->
    <LoadingOverlay v-if="isLoading" />
  </div>
</template>

<script>
import { computed, onMounted, onUnmounted } from 'vue'
import { useThemeStore } from './store/theme'
import { useWebSocketStore } from './store/websocket'
import { useLoadingStore } from './store/loading'
import GlobalNotifications from './components/common/GlobalNotifications.vue'
import ConnectionStatus from './components/common/ConnectionStatus.vue'
import LoadingOverlay from './components/common/LoadingOverlay.vue'

// Import Element Plus locale
import en from 'element-plus/es/locale/lang/en'

export default {
  name: 'App',
  components: {
    GlobalNotifications,
    ConnectionStatus,
    LoadingOverlay
  },
  setup() {
    const themeStore = useThemeStore()
    const webSocketStore = useWebSocketStore()
    const loadingStore = useLoadingStore()
    
    // Computed properties
    const isDark = computed(() => themeStore.isDark)
    const isLoading = computed(() => loadingStore.isLoading)
    
    // Component configuration
    const locale = en
    const componentSize = 'default'
    
    // Lifecycle hooks
    onMounted(() => {
      // Initialize WebSocket connection
      webSocketStore.connect()
      
      // Listen for theme changes
      themeStore.watchSystemTheme()
      
      // Handle window resize
      window.addEventListener('resize', handleResize)
      
      // Handle visibility change
      document.addEventListener('visibilitychange', handleVisibilityChange)
    })
    
    onUnmounted(() => {
      // Cleanup
      webSocketStore.disconnect()
      window.removeEventListener('resize', handleResize)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    })
    
    // Event handlers
    const handleResize = () => {
      // Update responsive breakpoints if needed
      themeStore.updateBreakpoint()
    }
    
    const handleVisibilityChange = () => {
      if (document.hidden) {
        // Page is hidden, reduce WebSocket activity
        webSocketStore.pauseConnection()
      } else {
        // Page is visible, resume WebSocket activity
        webSocketStore.resumeConnection()
      }
    }
    
    return {
      isDark,
      isLoading,
      locale,
      componentSize
    }
  }
}
</script>

<style>
/* Global styles */
* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#app {
  height: 100vh;
  width: 100%;
  overflow: hidden;
}

/* Dark theme overrides */
.dark {
  color-scheme: dark;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: var(--el-border-color-darker);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--el-border-color-dark);
}

/* Dark theme scrollbar */
.dark ::-webkit-scrollbar-track {
  background: var(--el-fill-color-dark);
}

.dark ::-webkit-scrollbar-thumb {
  background: var(--el-border-color-light);
}

.dark ::-webkit-scrollbar-thumb:hover {
  background: var(--el-border-color);
}

/* Responsive utilities */
@media (max-width: 768px) {
  .hide-mobile {
    display: none !important;
  }
}

@media (min-width: 769px) {
  .hide-desktop {
    display: none !important;
  }
}

/* Animation utilities */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.slide-enter-active, .slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from {
  transform: translateX(-100%);
}

.slide-leave-to {
  transform: translateX(100%);
}

/* Loading states */
.loading-shimmer {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.dark .loading-shimmer {
  background: linear-gradient(90deg, #2a2a2a 25%, #3a3a3a 50%, #2a2a2a 75%);
  background-size: 200% 100%;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

/* Custom Element Plus overrides */
.el-drawer__header {
  margin-bottom: 0;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.el-card {
  border: 1px solid var(--el-border-color-lighter);
}

.el-table {
  --el-table-border-color: var(--el-border-color-lighter);
}

/* Status indicators */
.status-online {
  color: var(--el-color-success);
}

.status-offline {
  color: var(--el-color-danger);
}

.status-pending {
  color: var(--el-color-warning);
}

/* File type icons */
.file-icon {
  font-size: 24px;
  margin-right: 8px;
}

.file-icon-image {
  color: var(--el-color-primary);
}

.file-icon-document {
  color: var(--el-color-info);
}

.file-icon-archive {
  color: var(--el-color-warning);
}

.file-icon-video {
  color: var(--el-color-danger);
}

.file-icon-audio {
  color: var(--el-color-success);
}

.file-icon-code {
  color: var(--el-color-primary);
}

.file-icon-default {
  color: var(--el-text-color-secondary);
}
</style>