import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { io } from 'socket.io-client'

export const useWebSocketStore = defineStore('websocket', () => {
  // State
  const socket = ref(null)
  const connected = ref(false)
  const connecting = ref(false)
  const error = ref(null)
  const reconnectAttempts = ref(0)
  const maxReconnectAttempts = ref(5)
  const reconnectDelay = ref(1000)
  const paused = ref(false)
  
  // Event listeners
  const listeners = ref(new Map())
  
  // Real-time data
  const systemStatus = ref({
    status: 'unknown',
    message: 'Not connected',
    timestamp: new Date()
  })
  
  const fileUploadProgress = ref(new Map())
  const fileDownloadProgress = ref(new Map())
  const cacheStats = ref({
    hit_ratio: 0,
    cache_size: 0,
    timestamp: new Date()
  })
  
  const networkHealth = ref({
    total_operators: 0,
    online_operators: 0,
    consensus_status: false,
    timestamp: new Date()
  })
  
  // Getters
  const isConnected = computed(() => connected.value)
  const isConnecting = computed(() => connecting.value)
  const hasError = computed(() => !!error.value)
  const connectionStatus = computed(() => {
    if (connected.value) return 'connected'
    if (connecting.value) return 'connecting'
    if (error.value) return 'error'
    return 'disconnected'
  })
  
  // Actions
  const connect = () => {
    if (socket.value || connecting.value) return
    
    connecting.value = true
    error.value = null
    
    try {
      socket.value = io({
        path: '/ws',
        transports: ['websocket'],
        upgrade: true,
        rememberUpgrade: true,
        timeout: 10000,
        forceNew: true
      })
      
      setupEventListeners()
    } catch (err) {
      console.error('WebSocket connection failed:', err)
      error.value = err.message
      connecting.value = false
    }
  }
  
  const disconnect = () => {
    if (socket.value) {
      socket.value.disconnect()
      socket.value = null
    }
    
    connected.value = false
    connecting.value = false
    error.value = null
    reconnectAttempts.value = 0
  }
  
  const reconnect = () => {
    if (reconnectAttempts.value >= maxReconnectAttempts.value) {
      console.error('Max reconnection attempts reached')
      error.value = 'Max reconnection attempts reached'
      return
    }
    
    reconnectAttempts.value++
    const delay = reconnectDelay.value * Math.pow(2, reconnectAttempts.value - 1)
    
    setTimeout(() => {
      console.log(`Reconnection attempt ${reconnectAttempts.value}/${maxReconnectAttempts.value}`)
      connect()
    }, delay)
  }
  
  const pauseConnection = () => {
    paused.value = true
    if (socket.value) {
      socket.value.disconnect()
    }
  }
  
  const resumeConnection = () => {
    paused.value = false
    if (!connected.value && !connecting.value) {
      connect()
    }
  }
  
  const setupEventListeners = () => {
    if (!socket.value) return
    
    // Connection events
    socket.value.on('connect', () => {
      console.log('WebSocket connected')
      connected.value = true
      connecting.value = false
      error.value = null
      reconnectAttempts.value = 0
      
      // Update system status
      systemStatus.value = {
        status: 'connected',
        message: 'WebSocket connected',
        timestamp: new Date()
      }
    })
    
    socket.value.on('disconnect', (reason) => {
      console.log('WebSocket disconnected:', reason)
      connected.value = false
      connecting.value = false
      
      systemStatus.value = {
        status: 'disconnected',
        message: `Disconnected: ${reason}`,
        timestamp: new Date()
      }
      
      // Auto-reconnect if not paused and not manually disconnected
      if (!paused.value && reason !== 'io client disconnect') {
        reconnect()
      }
    })
    
    socket.value.on('connect_error', (err) => {
      console.error('WebSocket connection error:', err)
      error.value = err.message
      connecting.value = false
      
      systemStatus.value = {
        status: 'error',
        message: `Connection error: ${err.message}`,
        timestamp: new Date()
      }
      
      if (!paused.value) {
        reconnect()
      }
    })
    
    // Data events
    socket.value.on('system_status', (data) => {
      systemStatus.value = {
        ...data,
        timestamp: new Date()
      }
    })
    
    socket.value.on('file_upload_progress', (data) => {
      fileUploadProgress.value.set(data.file_key, {
        ...data,
        timestamp: new Date()
      })
    })
    
    socket.value.on('file_download_progress', (data) => {
      fileDownloadProgress.value.set(data.file_key, {
        ...data,
        timestamp: new Date()
      })
    })
    
    socket.value.on('cache_stats', (data) => {
      cacheStats.value = {
        ...data,
        timestamp: new Date()
      }
    })
    
    socket.value.on('network_health', (data) => {
      networkHealth.value = {
        ...data,
        timestamp: new Date()
      }
    })
    
    // Governance events
    socket.value.on('governance_update', (data) => {
      console.log('Governance update:', data)
      // Emit to governance store
      emitToListeners('governance_update', data)
    })
    
    socket.value.on('operator_status_change', (data) => {
      console.log('Operator status change:', data)
      emitToListeners('operator_status_change', data)
    })
    
    socket.value.on('admin_action_executed', (data) => {
      console.log('Admin action executed:', data)
      emitToListeners('admin_action_executed', data)
    })
    
    // File events
    socket.value.on('file_uploaded', (data) => {
      console.log('File uploaded:', data)
      emitToListeners('file_uploaded', data)
    })
    
    socket.value.on('file_deleted', (data) => {
      console.log('File deleted:', data)
      emitToListeners('file_deleted', data)
    })
    
    socket.value.on('storage_stats_update', (data) => {
      console.log('Storage stats update:', data)
      emitToListeners('storage_stats_update', data)
    })
  }
  
  const emit = (event, data) => {
    if (socket.value && connected.value) {
      socket.value.emit(event, data)
    } else {
      console.warn('WebSocket not connected, cannot emit event:', event)
    }
  }
  
  const on = (event, callback) => {
    if (!listeners.value.has(event)) {
      listeners.value.set(event, new Set())
    }
    listeners.value.get(event).add(callback)
    
    // Return unsubscribe function
    return () => {
      const eventListeners = listeners.value.get(event)
      if (eventListeners) {
        eventListeners.delete(callback)
        if (eventListeners.size === 0) {
          listeners.value.delete(event)
        }
      }
    }
  }
  
  const off = (event, callback) => {
    const eventListeners = listeners.value.get(event)
    if (eventListeners) {
      eventListeners.delete(callback)
      if (eventListeners.size === 0) {
        listeners.value.delete(event)
      }
    }
  }
  
  const emitToListeners = (event, data) => {
    const eventListeners = listeners.value.get(event)
    if (eventListeners) {
      eventListeners.forEach(callback => {
        try {
          callback(data)
        } catch (err) {
          console.error('Error in WebSocket event listener:', err)
        }
      })
    }
  }
  
  // Subscription management
  const subscribeToFileProgress = (fileKey) => {
    emit('subscribe_file_progress', { file_key: fileKey })
  }
  
  const unsubscribeFromFileProgress = (fileKey) => {
    emit('unsubscribe_file_progress', { file_key: fileKey })
  }
  
  const subscribeToGovernanceUpdates = () => {
    emit('subscribe_governance_updates')
  }
  
  const unsubscribeFromGovernanceUpdates = () => {
    emit('unsubscribe_governance_updates')
  }
  
  const subscribeToSystemUpdates = () => {
    emit('subscribe_system_updates')
  }
  
  const unsubscribeFromSystemUpdates = () => {
    emit('unsubscribe_system_updates')
  }
  
  // Helper methods
  const getUploadProgress = (fileKey) => {
    return fileUploadProgress.value.get(fileKey) || null
  }
  
  const getDownloadProgress = (fileKey) => {
    return fileDownloadProgress.value.get(fileKey) || null
  }
  
  const clearFileProgress = (fileKey) => {
    fileUploadProgress.value.delete(fileKey)
    fileDownloadProgress.value.delete(fileKey)
  }
  
  const ping = () => {
    if (socket.value && connected.value) {
      const startTime = Date.now()
      socket.value.emit('ping', { timestamp: startTime })
      
      const handlePong = (data) => {
        const latency = Date.now() - startTime
        console.log(`WebSocket latency: ${latency}ms`)
        socket.value.off('pong', handlePong)
      }
      
      socket.value.on('pong', handlePong)
    }
  }
  
  // Cleanup
  const cleanup = () => {
    disconnect()
    listeners.value.clear()
    fileUploadProgress.value.clear()
    fileDownloadProgress.value.clear()
  }
  
  return {
    // State
    socket,
    connected,
    connecting,
    error,
    reconnectAttempts,
    maxReconnectAttempts,
    reconnectDelay,
    paused,
    systemStatus,
    fileUploadProgress,
    fileDownloadProgress,
    cacheStats,
    networkHealth,
    
    // Getters
    isConnected,
    isConnecting,
    hasError,
    connectionStatus,
    
    // Actions
    connect,
    disconnect,
    reconnect,
    pauseConnection,
    resumeConnection,
    emit,
    on,
    off,
    subscribeToFileProgress,
    unsubscribeFromFileProgress,
    subscribeToGovernanceUpdates,
    unsubscribeFromGovernanceUpdates,
    subscribeToSystemUpdates,
    unsubscribeFromSystemUpdates,
    getUploadProgress,
    getDownloadProgress,
    clearFileProgress,
    ping,
    cleanup
  }
})