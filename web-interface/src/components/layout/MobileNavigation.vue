<template>
  <div class="mobile-navigation">
    <!-- Mobile Header -->
    <div
      class="mobile-header"
      role="banner"
    >
      <div class="mobile-header-content">
        <!-- Menu Toggle -->
        <el-button
          class="mobile-menu-toggle"
          :aria-label="mobileMenuOpen ? 'Close navigation menu' : 'Open navigation menu'"
          :aria-expanded="mobileMenuOpen"
          :aria-controls="'mobile-menu'"
          @click="toggleMobileMenu"
        >
          <el-icon v-if="!mobileMenuOpen">
            <Menu />
          </el-icon>
          <el-icon v-else>
            <Close />
          </el-icon>
        </el-button>

        <!-- Logo -->
        <router-link 
          to="/" 
          class="mobile-logo"
          aria-label="DataMesh home"
          @click="closeMobileMenu"
        >
          <el-icon
            size="24"
            color="#409EFF"
          >
            <DataBoard />
          </el-icon>
          <span class="logo-text">DataMesh</span>
        </router-link>

        <!-- Header Actions -->
        <div class="mobile-header-actions">
          <!-- Search Toggle -->
          <el-button
            circle
            :aria-label="mobileSearchOpen ? 'Close search' : 'Open search'"
            :aria-expanded="mobileSearchOpen"
            class="mobile-search-toggle"
            @click="toggleMobileSearch"
          >
            <el-icon><Search /></el-icon>
          </el-button>

          <!-- Notifications -->
          <el-dropdown 
            trigger="click" 
            placement="bottom-end"
            @visible-change="handleNotificationVisibility"
          >
            <el-button 
              circle
              :aria-label="`Notifications. ${notificationCount} unread`"
              class="mobile-notifications"
            >
              <el-badge
                :value="notificationCount"
                :hidden="notificationCount === 0"
              >
                <el-icon><Bell /></el-icon>
              </el-badge>
            </el-button>
            <template #dropdown>
              <div class="mobile-notifications-panel">
                <div class="notifications-header">
                  <h3>Notifications</h3>
                  <el-button 
                    text 
                    size="small" 
                    :disabled="notificationCount === 0"
                    @click="clearAllNotifications"
                  >
                    Clear All
                  </el-button>
                </div>
                <div class="notifications-content">
                  <div
                    v-for="notification in notifications"
                    :key="notification.id"
                    class="notification-item"
                    @click="handleNotificationClick(notification)"
                  >
                    <el-icon :color="getNotificationColor(notification.type)">
                      <component :is="getNotificationIcon(notification.type)" />
                    </el-icon>
                    <div class="notification-content">
                      <div class="notification-title">
                        {{ notification.title }}
                      </div>
                      <div class="notification-time">
                        {{ formatTime(notification.timestamp) }}
                      </div>
                    </div>
                  </div>
                  <div
                    v-if="notifications.length === 0"
                    class="no-notifications"
                  >
                    No new notifications
                  </div>
                </div>
              </div>
            </template>
          </el-dropdown>

          <!-- User Menu -->
          <el-dropdown 
            trigger="click" 
            placement="bottom-end"
            @visible-change="handleUserMenuVisibility"
          >
            <div class="mobile-user-menu">
              <el-avatar
                :size="32"
                :src="currentUser?.avatar"
              >
                <el-icon><User /></el-icon>
              </el-avatar>
            </div>
            <template #dropdown>
              <div class="mobile-user-panel">
                <div class="user-info">
                  <el-avatar
                    :size="48"
                    :src="currentUser?.avatar"
                  >
                    <el-icon><User /></el-icon>
                  </el-avatar>
                  <div class="user-details">
                    <div class="user-name">
                      {{ currentUser?.name || 'User' }}
                    </div>
                    <div class="user-role">
                      {{ currentUser?.role || 'Member' }}
                    </div>
                  </div>
                </div>
                <div class="user-actions">
                  <el-button
                    class="user-action-btn"
                    @click="goToProfile"
                  >
                    <el-icon><User /></el-icon>
                    Profile
                  </el-button>
                  <el-button
                    class="user-action-btn"
                    @click="goToSettings"
                  >
                    <el-icon><Setting /></el-icon>
                    Settings
                  </el-button>
                  <el-button
                    class="user-action-btn"
                    @click="toggleTheme"
                  >
                    <el-icon><component :is="themeIcon" /></el-icon>
                    {{ themeText }}
                  </el-button>
                  <el-button
                    type="danger"
                    class="user-action-btn"
                    @click="handleLogout"
                  >
                    <el-icon><SwitchButton /></el-icon>
                    Logout
                  </el-button>
                </div>
              </div>
            </template>
          </el-dropdown>
        </div>
      </div>

      <!-- Mobile Search Bar -->
      <transition name="search-slide">
        <div
          v-if="mobileSearchOpen"
          class="mobile-search-bar"
        >
          <el-input
            ref="mobileSearchInput"
            v-model="searchQuery"
            placeholder="Search files..."
            class="mobile-search-input"
            size="large"
            @keyup.enter="performSearch"
            @blur="handleSearchBlur"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
            <template #suffix>
              <el-button 
                text 
                aria-label="Close search"
                @click="closeMobileSearch"
              >
                <el-icon><Close /></el-icon>
              </el-button>
            </template>
          </el-input>
        </div>
      </transition>
    </div>

    <!-- Mobile Menu Overlay -->
    <transition name="overlay-fade">
      <div
        v-if="mobileMenuOpen"
        class="mobile-menu-overlay"
        aria-hidden="true"
        @click="closeMobileMenu"
      />
    </transition>

    <!-- Mobile Navigation Menu -->
    <transition name="menu-slide">
      <nav
        v-if="mobileMenuOpen"
        id="mobile-menu"
        class="mobile-menu"
        role="navigation"
        aria-label="Mobile navigation menu"
      >
        <div class="mobile-menu-content">
          <!-- Main Navigation -->
          <div class="mobile-menu-section">
            <h3 class="mobile-menu-title">
              Navigation
            </h3>
            <div class="mobile-menu-items">
              <router-link
                to="/"
                class="mobile-menu-item"
                :class="{ active: $route.path === '/' }"
                @click="closeMobileMenu"
              >
                <el-icon><House /></el-icon>
                <span>Dashboard</span>
              </router-link>

              <router-link
                to="/files"
                class="mobile-menu-item"
                :class="{ active: $route.path === '/files' }"
                @click="closeMobileMenu"
              >
                <el-icon><FolderOpened /></el-icon>
                <span>File Manager</span>
              </router-link>

              <div class="mobile-menu-group">
                <div 
                  class="mobile-menu-group-title"
                  :aria-expanded="governanceSubmenuOpen"
                  @click="toggleGovernanceSubmenu"
                >
                  <el-icon><Flag /></el-icon>
                  <span>Governance</span>
                  <el-icon
                    class="submenu-arrow"
                    :class="{ open: governanceSubmenuOpen }"
                  >
                    <ArrowDown />
                  </el-icon>
                </div>
                <transition name="submenu-slide">
                  <div
                    v-if="governanceSubmenuOpen"
                    class="mobile-submenu"
                  >
                    <router-link
                      to="/governance"
                      class="mobile-submenu-item"
                      @click="closeMobileMenu"
                    >
                      <el-icon><TrendCharts /></el-icon>
                      <span>Overview</span>
                    </router-link>
                    <router-link
                      to="/governance/operators"
                      class="mobile-submenu-item"
                      @click="closeMobileMenu"
                    >
                      <el-icon><UserFilled /></el-icon>
                      <span>Operators</span>
                    </router-link>
                    <router-link
                      to="/governance/proposals"
                      class="mobile-submenu-item"
                      @click="closeMobileMenu"
                    >
                      <el-icon><DocumentCopy /></el-icon>
                      <span>Proposals</span>
                    </router-link>
                  </div>
                </transition>
              </div>

              <router-link
                v-if="isAdmin"
                to="/administration"
                class="mobile-menu-item"
                :class="{ active: $route.path.startsWith('/administration') }"
                @click="closeMobileMenu"
              >
                <el-icon><Setting /></el-icon>
                <span>Administration</span>
              </router-link>

              <router-link
                to="/analytics"
                class="mobile-menu-item"
                :class="{ active: $route.path === '/analytics' }"
                @click="closeMobileMenu"
              >
                <el-icon><TrendCharts /></el-icon>
                <span>Analytics</span>
              </router-link>
            </div>
          </div>

          <!-- Quick Actions -->
          <div class="mobile-menu-section">
            <h3 class="mobile-menu-title">
              Quick Actions
            </h3>
            <div class="mobile-quick-actions">
              <el-button
                type="primary"
                class="mobile-action-btn"
                @click="handleUpload"
              >
                <el-icon><Upload /></el-icon>
                Upload Files
              </el-button>
              <el-button
                class="mobile-action-btn"
                @click="handleRefresh"
              >
                <el-icon><Refresh /></el-icon>
                Refresh
              </el-button>
            </div>
          </div>

          <!-- Connection Status -->
          <div class="mobile-menu-section">
            <div class="mobile-connection-status">
              <el-icon :color="connectionStatusColor">
                <component :is="connectionStatusIcon" />
              </el-icon>
              <span class="connection-text">{{ connectionStatusText }}</span>
            </div>
          </div>
        </div>
      </nav>
    </transition>

    <!-- Mobile Bottom Navigation -->
    <div
      class="mobile-bottom-nav"
      role="navigation"
      aria-label="Bottom navigation"
    >
      <router-link
        to="/"
        class="bottom-nav-item"
        :class="{ active: $route.path === '/' }"
      >
        <el-icon><House /></el-icon>
        <span>Dashboard</span>
      </router-link>
      
      <router-link
        to="/files"
        class="bottom-nav-item"
        :class="{ active: $route.path === '/files' }"
      >
        <el-icon><FolderOpened /></el-icon>
        <span>Files</span>
      </router-link>
      
      <div class="bottom-nav-item fab-container">
        <el-button 
          type="primary" 
          circle 
          size="large" 
          class="mobile-fab"
          aria-label="Upload files"
          @click="handleUpload"
        >
          <el-icon><Plus /></el-icon>
        </el-button>
      </div>
      
      <router-link
        to="/governance"
        class="bottom-nav-item"
        :class="{ active: $route.path.startsWith('/governance') }"
      >
        <el-icon><Flag /></el-icon>
        <span>Governance</span>
      </router-link>
      
      <router-link
        to="/analytics"
        class="bottom-nav-item"
        :class="{ active: $route.path === '/analytics' }"
      >
        <el-icon><TrendCharts /></el-icon>
        <span>Analytics</span>
      </router-link>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useThemeStore } from '@/store/theme'
import {
  Menu, Close, Search, Bell, User, DataBoard, House, FolderOpened,
  Flag, Setting, TrendCharts, UserFilled, DocumentCopy, Upload,
  Refresh, Plus, ArrowDown, SwitchButton, Moon, Sunny
} from '@element-plus/icons-vue'
import { a11y } from '@/utils/accessibility'

export default {
  name: 'MobileNavigation',
  components: {
    Menu, Close, Search, Bell, User, DataBoard, House, FolderOpened,
    Flag, Setting, TrendCharts, UserFilled, DocumentCopy, Upload,
    Refresh, Plus, ArrowDown, SwitchButton, Moon, Sunny
  },
  props: {
    notifications: {
      type: Array,
      default: () => []
    },
    connectionStatus: {
      type: String,
      default: 'disconnected'
    }
  },
  emits: [
    'upload-click',
    'refresh-click',
    'search',
    'notification-click',
    'clear-notifications'
  ],
  setup(props, { emit }) {
    const router = useRouter()
    const route = useRoute()
    const authStore = useAuthStore()
    const themeStore = useThemeStore()

    // Reactive state
    const mobileMenuOpen = ref(false)
    const mobileSearchOpen = ref(false)
    const governanceSubmenuOpen = ref(false)
    const searchQuery = ref('')
    const mobileSearchInput = ref(null)

    // Computed properties
    const currentUser = computed(() => authStore.currentUser)
    const isAdmin = computed(() => authStore.isAdmin)
    const notificationCount = computed(() => 
      props.notifications.filter(n => !n.read).length
    )

    const connectionStatusText = computed(() => {
      switch (props.connectionStatus) {
        case 'connected': return 'Connected'
        case 'connecting': return 'Connecting...'
        case 'error': return 'Connection error'
        default: return 'Disconnected'
      }
    })

    const connectionStatusColor = computed(() => {
      switch (props.connectionStatus) {
        case 'connected': return 'var(--el-color-success)'
        case 'connecting': return 'var(--el-color-warning)'
        case 'error': return 'var(--el-color-danger)'
        default: return 'var(--el-color-info)'
      }
    })

    const connectionStatusIcon = computed(() => {
      switch (props.connectionStatus) {
        case 'connected': return 'CircleCheck'
        case 'connecting': return 'Loading'
        case 'error': return 'Warning'
        default: return 'Close'
      }
    })

    const themeIcon = computed(() => themeStore.isDark ? 'Sunny' : 'Moon')
    const themeText = computed(() => themeStore.isDark ? 'Light Mode' : 'Dark Mode')

    // Methods
    const toggleMobileMenu = () => {
      mobileMenuOpen.value = !mobileMenuOpen.value
      
      if (mobileMenuOpen.value) {
        // Trap focus within mobile menu
        nextTick(() => {
          const menu = document.getElementById('mobile-menu')
          if (menu) {
            a11y.focusManager.trapFocus(menu)
          }
        })
        a11y.screenReader.announce('Mobile menu opened')
      } else {
        a11y.focusManager.restoreFocus()
        a11y.screenReader.announce('Mobile menu closed')
      }
    }

    const closeMobileMenu = () => {
      if (mobileMenuOpen.value) {
        mobileMenuOpen.value = false
        governanceSubmenuOpen.value = false
        a11y.focusManager.restoreFocus()
        a11y.screenReader.announce('Mobile menu closed')
      }
    }

    const toggleMobileSearch = () => {
      mobileSearchOpen.value = !mobileSearchOpen.value
      
      if (mobileSearchOpen.value) {
        nextTick(() => {
          if (mobileSearchInput.value) {
            mobileSearchInput.value.focus()
          }
        })
        a11y.screenReader.announce('Search opened')
      } else {
        searchQuery.value = ''
        a11y.screenReader.announce('Search closed')
      }
    }

    const closeMobileSearch = () => {
      mobileSearchOpen.value = false
      searchQuery.value = ''
    }

    const handleSearchBlur = () => {
      // Small delay to allow click events on search results
      setTimeout(() => {
        if (!searchQuery.value) {
          closeMobileSearch()
        }
      }, 200)
    }

    const toggleGovernanceSubmenu = () => {
      governanceSubmenuOpen.value = !governanceSubmenuOpen.value
      const text = governanceSubmenuOpen.value ? 'Governance menu opened' : 'Governance menu closed'
      a11y.screenReader.announce(text)
    }

    const performSearch = () => {
      if (searchQuery.value.trim()) {
        emit('search', searchQuery.value)
        closeMobileSearch()
        a11y.screenReader.announce(`Searching for ${searchQuery.value}`)
      }
    }

    const handleUpload = () => {
      emit('upload-click')
      closeMobileMenu()
    }

    const handleRefresh = () => {
      emit('refresh-click')
      closeMobileMenu()
    }

    const handleNotificationClick = (notification) => {
      emit('notification-click', notification)
    }

    const clearAllNotifications = () => {
      emit('clear-notifications')
      a11y.screenReader.announce('All notifications cleared')
    }

    const handleNotificationVisibility = (visible) => {
      if (visible) {
        a11y.screenReader.announce(`Notifications panel opened. ${notificationCount.value} unread notifications`)
      }
    }

    const handleUserMenuVisibility = (visible) => {
      if (visible) {
        a11y.screenReader.announce('User menu opened')
      }
    }

    const goToProfile = () => {
      router.push('/profile')
    }

    const goToSettings = () => {
      router.push('/settings')
    }

    const toggleTheme = () => {
      themeStore.toggleTheme()
      const mode = themeStore.isDark ? 'dark' : 'light'
      a11y.screenReader.announce(`Switched to ${mode} theme`)
    }

    const handleLogout = async () => {
      try {
        await authStore.logout()
        router.push('/auth/login')
        a11y.screenReader.announce('Logged out successfully')
      } catch (error) {
        a11y.screenReader.announceError('Logout failed')
      }
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
      return new Date(timestamp).toLocaleTimeString()
    }

    // Close mobile menu on route change
    watch(() => route.path, () => {
      closeMobileMenu()
      closeMobileSearch()
    })

    // Close mobile menu on window resize to desktop
    const handleResize = () => {
      if (window.innerWidth >= 768) {
        closeMobileMenu()
        closeMobileSearch()
      }
    }

    // Set up event listeners
    window.addEventListener('resize', handleResize)

    return {
      // State
      mobileMenuOpen,
      mobileSearchOpen,
      governanceSubmenuOpen,
      searchQuery,
      mobileSearchInput,

      // Computed
      currentUser,
      isAdmin,
      notificationCount,
      connectionStatusText,
      connectionStatusColor,
      connectionStatusIcon,
      themeIcon,
      themeText,

      // Methods
      toggleMobileMenu,
      closeMobileMenu,
      toggleMobileSearch,
      closeMobileSearch,
      handleSearchBlur,
      toggleGovernanceSubmenu,
      performSearch,
      handleUpload,
      handleRefresh,
      handleNotificationClick,
      clearAllNotifications,
      handleNotificationVisibility,
      handleUserMenuVisibility,
      goToProfile,
      goToSettings,
      toggleTheme,
      handleLogout,
      getNotificationIcon,
      getNotificationColor,
      formatTime
    }
  }
}
</script>

<style scoped>
.mobile-navigation {
  display: none;
}

/* Show mobile navigation on mobile devices */
@media (max-width: 768px) {
  .mobile-navigation {
    display: block;
  }

  /* Mobile Header */
  .mobile-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 1000;
    background: var(--el-bg-color);
    border-bottom: 1px solid var(--el-border-color-lighter);
    backdrop-filter: blur(8px);
  }

  .mobile-header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    height: 56px;
  }

  .mobile-menu-toggle {
    min-width: 44px;
    min-height: 44px;
    padding: 0;
  }

  .mobile-logo {
    display: flex;
    align-items: center;
    gap: 8px;
    text-decoration: none;
    color: var(--el-text-color-primary);
    font-weight: 600;
    font-size: 18px;
  }

  .mobile-header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mobile-search-toggle,
  .mobile-notifications {
    min-width: 44px;
    min-height: 44px;
    padding: 0;
  }

  .mobile-user-menu {
    cursor: pointer;
    padding: 4px;
    border-radius: 50%;
    transition: background-color 0.3s ease;
  }

  .mobile-user-menu:hover {
    background: var(--el-fill-color-light);
  }

  /* Mobile Search Bar */
  .mobile-search-bar {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--el-bg-color);
    border-bottom: 1px solid var(--el-border-color-lighter);
    padding: 12px 16px;
    z-index: 999;
  }

  .mobile-search-input {
    width: 100%;
  }

  /* Search Transitions */
  .search-slide-enter-active,
  .search-slide-leave-active {
    transition: all 0.3s ease;
  }

  .search-slide-enter-from,
  .search-slide-leave-to {
    transform: translateY(-100%);
    opacity: 0;
  }

  /* Mobile Menu Overlay */
  .mobile-menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1001;
  }

  .overlay-fade-enter-active,
  .overlay-fade-leave-active {
    transition: opacity 0.3s ease;
  }

  .overlay-fade-enter-from,
  .overlay-fade-leave-to {
    opacity: 0;
  }

  /* Mobile Menu */
  .mobile-menu {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 280px;
    background: var(--el-bg-color);
    border-right: 1px solid var(--el-border-color-lighter);
    z-index: 1002;
    overflow-y: auto;
  }

  .mobile-menu-content {
    padding: 72px 0 20px;
  }

  .mobile-menu-section {
    margin-bottom: 24px;
  }

  .mobile-menu-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 12px 0;
    padding: 0 20px;
  }

  .mobile-menu-items {
    padding: 0 12px;
  }

  .mobile-menu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    text-decoration: none;
    color: var(--el-text-color-primary);
    font-weight: 500;
    margin-bottom: 4px;
    min-height: 48px;
    transition: all 0.3s ease;
  }

  .mobile-menu-item:hover,
  .mobile-menu-item.active {
    background: var(--el-color-primary-light-9);
    color: var(--el-color-primary);
  }

  .mobile-menu-group {
    margin-bottom: 4px;
  }

  .mobile-menu-group-title {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
    min-height: 48px;
    transition: all 0.3s ease;
  }

  .mobile-menu-group-title:hover {
    background: var(--el-fill-color-light);
  }

  .submenu-arrow {
    margin-left: auto;
    transition: transform 0.3s ease;
  }

  .submenu-arrow.open {
    transform: rotate(180deg);
  }

  .mobile-submenu {
    padding-left: 20px;
    margin-top: 4px;
  }

  .mobile-submenu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-radius: 6px;
    text-decoration: none;
    color: var(--el-text-color-regular);
    font-size: 14px;
    margin-bottom: 2px;
    min-height: 44px;
    transition: all 0.3s ease;
  }

  .mobile-submenu-item:hover {
    background: var(--el-fill-color-light);
    color: var(--el-text-color-primary);
  }

  /* Submenu Transitions */
  .submenu-slide-enter-active,
  .submenu-slide-leave-active {
    transition: all 0.3s ease;
    overflow: hidden;
  }

  .submenu-slide-enter-from,
  .submenu-slide-leave-to {
    height: 0;
    opacity: 0;
  }

  /* Menu Transitions */
  .menu-slide-enter-active,
  .menu-slide-leave-active {
    transition: transform 0.3s ease;
  }

  .menu-slide-enter-from,
  .menu-slide-leave-to {
    transform: translateX(-100%);
  }

  /* Quick Actions */
  .mobile-quick-actions {
    padding: 0 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mobile-action-btn {
    width: 100%;
    justify-content: flex-start;
    height: 44px;
  }

  /* Connection Status */
  .mobile-connection-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 20px;
    color: var(--el-text-color-secondary);
    font-size: 14px;
  }

  /* Mobile Bottom Navigation */
  .mobile-bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    background: var(--el-bg-color);
    border-top: 1px solid var(--el-border-color-lighter);
    padding: 8px 0;
    z-index: 100;
    backdrop-filter: blur(8px);
  }

  .bottom-nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    text-decoration: none;
    color: var(--el-text-color-secondary);
    font-size: 12px;
    font-weight: 500;
    transition: color 0.3s ease;
  }

  .bottom-nav-item.active {
    color: var(--el-color-primary);
  }

  .bottom-nav-item:hover {
    color: var(--el-text-color-primary);
  }

  .fab-container {
    position: relative;
  }

  .mobile-fab {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    width: 56px;
    height: 56px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  /* Notification Panel */
  .mobile-notifications-panel {
    width: 280px;
    max-height: 400px;
    background: var(--el-bg-color);
  }

  .notifications-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--el-border-color-lighter);
  }

  .notifications-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .notifications-content {
    max-height: 300px;
    overflow-y: auto;
  }

  .notification-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    cursor: pointer;
    transition: background-color 0.3s ease;
  }

  .notification-item:hover {
    background: var(--el-fill-color-light);
  }

  .notification-content {
    flex: 1;
  }

  .notification-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--el-text-color-primary);
    margin-bottom: 4px;
  }

  .notification-time {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }

  .no-notifications {
    text-align: center;
    padding: 40px 20px;
    color: var(--el-text-color-secondary);
    font-size: 14px;
  }

  /* User Panel */
  .mobile-user-panel {
    width: 240px;
    background: var(--el-bg-color);
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 20px;
    border-bottom: 1px solid var(--el-border-color-lighter);
  }

  .user-details {
    flex: 1;
  }

  .user-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    margin-bottom: 4px;
  }

  .user-role {
    font-size: 14px;
    color: var(--el-text-color-secondary);
  }

  .user-actions {
    padding: 12px;
  }

  .user-action-btn {
    width: 100%;
    justify-content: flex-start;
    margin-bottom: 8px;
    height: 44px;
  }

  .user-action-btn:last-child {
    margin-bottom: 0;
  }

  /* Dark Mode */
  .dark .mobile-header {
    background: var(--el-bg-color-overlay);
  }

  .dark .mobile-menu {
    background: var(--el-bg-color-overlay);
  }

  .dark .mobile-bottom-nav {
    background: var(--el-bg-color-overlay);
  }

  /* High Contrast */
  @media (prefers-contrast: high) {
    .mobile-menu-item,
    .mobile-submenu-item {
      border: 1px solid transparent;
    }

    .mobile-menu-item:focus,
    .mobile-submenu-item:focus {
      border-color: var(--el-color-primary);
    }
  }

  /* Landscape orientation adjustments */
  @media (orientation: landscape) and (max-height: 500px) {
    .mobile-header-content {
      padding: 8px 16px;
      height: 48px;
    }

    .mobile-menu-content {
      padding-top: 60px;
    }

    .mobile-fab {
      bottom: 12px;
      width: 48px;
      height: 48px;
    }
  }
}

/* Ensure mobile navigation is hidden on desktop */
@media (min-width: 769px) {
  .mobile-navigation {
    display: none !important;
  }
}
</style>