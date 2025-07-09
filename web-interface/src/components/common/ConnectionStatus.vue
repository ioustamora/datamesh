<template>
  <div
    v-if="showStatus"
    class="connection-status"
    :class="statusClass"
  >
    <div class="status-content">
      <el-icon class="status-icon">
        <component :is="statusIcon" />
      </el-icon>
      <span class="status-text">{{ statusText }}</span>
      <el-button
        v-if="showReconnect"
        size="small"
        type="primary"
        text
        @click="reconnect"
      >
        Reconnect
      </el-button>
    </div>
  </div>
</template>

<script>
import { computed } from 'vue'
import { useWebSocketStore } from '../../store/websocket'

export default {
  name: 'ConnectionStatus',
  setup() {
    const webSocketStore = useWebSocketStore()
    
    const showStatus = computed(() => {
      const status = webSocketStore.connectionStatus
      return status === 'connecting' || status === 'error'
    })
    
    const statusClass = computed(() => {
      const status = webSocketStore.connectionStatus
      return `status-${status}`
    })
    
    const statusIcon = computed(() => {
      const status = webSocketStore.connectionStatus
      switch (status) {
        case 'connecting':
          return 'Loading'
        case 'error':
          return 'Warning'
        default:
          return 'Connection'
      }
    })
    
    const statusText = computed(() => {
      const status = webSocketStore.connectionStatus
      switch (status) {
        case 'connecting':
          return `Connecting... (${webSocketStore.reconnectAttempts}/${webSocketStore.maxReconnectAttempts})`
        case 'error':
          return 'Connection failed'
        default:
          return 'Connected'
      }
    })
    
    const showReconnect = computed(() => {
      return webSocketStore.connectionStatus === 'error'
    })
    
    const reconnect = () => {
      webSocketStore.connect()
    }
    
    return {
      showStatus,
      statusClass,
      statusIcon,
      statusText,
      showReconnect,
      reconnect
    }
  }
}
</script>

<style scoped>
.connection-status {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 1000;
  border-radius: 8px;
  padding: 12px 16px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  min-width: 200px;
  backdrop-filter: blur(8px);
}

.status-connecting {
  background: rgba(230, 162, 60, 0.1);
  border: 1px solid var(--el-color-warning);
  color: var(--el-color-warning);
}

.status-error {
  background: rgba(245, 108, 108, 0.1);
  border: 1px solid var(--el-color-danger);
  color: var(--el-color-danger);
}

.status-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
}

.status-icon {
  flex-shrink: 0;
}

.status-text {
  flex: 1;
}

.status-connecting .status-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>