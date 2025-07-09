<template>
  <div class="notifications-container">
    <transition-group name="notification" tag="div">
      <div
        v-for="notification in notifications"
        :key="notification.id"
        class="notification"
        :class="notificationClass(notification)"
      >
        <div class="notification-icon">
          <el-icon>
            <component :is="getIcon(notification.type)" />
          </el-icon>
        </div>
        <div class="notification-content">
          <div class="notification-title">{{ notification.title }}</div>
          <div v-if="notification.message" class="notification-message">
            {{ notification.message }}
          </div>
        </div>
        <div class="notification-actions">
          <el-button
            v-if="notification.action"
            size="small"
            type="primary"
            text
            @click="handleAction(notification)"
          >
            {{ notification.actionText || 'View' }}
          </el-button>
          <el-button
            size="small"
            text
            @click="removeNotification(notification.id)"
          >
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

export default {
  name: 'GlobalNotifications',
  setup() {
    const router = useRouter()
    const notifications = ref([])
    const maxNotifications = 5
    const defaultDuration = 5000
    
    // Track timeouts to prevent memory leaks
    const timeouts = new Map()
    
    const addNotification = (notification) => {
      const id = Date.now()
      const newNotification = {
        id,
        type: 'info',
        title: 'Notification',
        message: '',
        duration: defaultDuration,
        persistent: false,
        ...notification
      }
      
      notifications.value.unshift(newNotification)
      
      // Keep only max notifications
      if (notifications.value.length > maxNotifications) {
        const removed = notifications.value.splice(maxNotifications)
        // Clear timeouts for removed notifications
        removed.forEach(n => {
          if (timeouts.has(n.id)) {
            clearTimeout(timeouts.get(n.id))
            timeouts.delete(n.id)
          }
        })
      }
      
      // Auto remove after duration
      if (!newNotification.persistent) {
        const timeoutId = setTimeout(() => {
          removeNotification(id)
        }, newNotification.duration)
        timeouts.set(id, timeoutId)
      }
    }
    
    const removeNotification = (id) => {
      // Clear timeout if exists
      if (timeouts.has(id)) {
        clearTimeout(timeouts.get(id))
        timeouts.delete(id)
      }
      
      notifications.value = notifications.value.filter(n => n.id !== id)
    }
    
    const clearAllNotifications = () => {
      // Clear all timeouts
      timeouts.forEach(timeoutId => clearTimeout(timeoutId))
      timeouts.clear()
      
      notifications.value = []
    }
    
    const notificationClass = (notification) => {
      return `notification-${notification.type}`
    }
    
    const getIcon = (type) => {
      switch (type) {
        case 'success':
          return 'CircleCheck'
        case 'warning':
          return 'Warning'
        case 'error':
          return 'CircleClose'
        case 'info':
          return 'InfoFilled'
        default:
          return 'Bell'
      }
    }
    
    const handleAction = (notification) => {
      if (notification.action) {
        if (typeof notification.action === 'function') {
          notification.action()
        } else if (typeof notification.action === 'string') {
          router.push(notification.action)
        }
      }
      removeNotification(notification.id)
    }
    
    // Global event listeners
    const handleGlobalNotification = (event) => {
      addNotification(event.detail)
    }
    
    onMounted(() => {
      window.addEventListener('global-notification', handleGlobalNotification)
      
      // Make addNotification globally available
      window.addNotification = addNotification
      
      // Example notifications for testing
      if (import.meta.env.DEV) {
        setTimeout(() => {
          addNotification({
            type: 'success',
            title: 'Welcome to DataMesh!',
            message: 'Your distributed storage system is ready to use.',
            duration: 8000
          })
        }, 1000)
      }
    })
    
    onUnmounted(() => {
      window.removeEventListener('global-notification', handleGlobalNotification)
      delete window.addNotification
      
      // Clear all timeouts to prevent memory leaks
      timeouts.forEach(timeoutId => clearTimeout(timeoutId))
      timeouts.clear()
      
      // Clear all notifications
      notifications.value = []
    })
    
    return {
      notifications,
      removeNotification,
      clearAllNotifications,
      notificationClass,
      getIcon,
      handleAction
    }
  }
}
</script>

<style scoped>
.notifications-container {
  position: fixed;
  top: 80px;
  right: 16px;
  z-index: 2000;
  max-width: 400px;
  width: 100%;
}

.notification {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  margin-bottom: 12px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(8px);
  border: 1px solid var(--el-border-color-lighter);
}

.notification-success {
  background: rgba(103, 194, 58, 0.1);
  border-color: var(--el-color-success);
}

.notification-warning {
  background: rgba(230, 162, 60, 0.1);
  border-color: var(--el-color-warning);
}

.notification-error {
  background: rgba(245, 108, 108, 0.1);
  border-color: var(--el-color-danger);
}

.notification-info {
  background: rgba(64, 158, 255, 0.1);
  border-color: var(--el-color-primary);
}

.notification-icon {
  flex-shrink: 0;
  font-size: 20px;
  margin-top: 2px;
}

.notification-success .notification-icon {
  color: var(--el-color-success);
}

.notification-warning .notification-icon {
  color: var(--el-color-warning);
}

.notification-error .notification-icon {
  color: var(--el-color-danger);
}

.notification-info .notification-icon {
  color: var(--el-color-primary);
}

.notification-content {
  flex: 1;
}

.notification-title {
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.notification-message {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  line-height: 1.4;
}

.notification-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* Animations */
.notification-enter-active {
  transition: all 0.3s ease;
}

.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.notification-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.notification-move {
  transition: transform 0.3s ease;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .notifications-container {
    left: 16px;
    right: 16px;
    max-width: none;
  }
  
  .notification {
    padding: 12px;
  }
  
  .notification-title {
    font-size: 14px;
  }
  
  .notification-message {
    font-size: 12px;
  }
}
</style>