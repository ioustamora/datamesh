import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

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
      // Use native WebSocket instead of Socket.IO
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const wsUrl = `${protocol}//${window.location.hostname}:8080/api/v1/ws`
      
      socket.value = new WebSocket(wsUrl)
      
      setupEventListeners()
    } catch (err) {
      console.error('WebSocket connection failed:', err)
      error.value = err.message
      connecting.value = false
    }
  }
  
  const disconnect = () => {
    if (socket.value) {
      socket.value.close()
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
      socket.value.close()
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
    
    // Connection events - use native WebSocket events
    socket.value.onopen = () => {
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
    }
    
    socket.value.onclose = (event) => {
      console.log('WebSocket disconnected:', event.reason)
      connected.value = false
      connecting.value = false
      
      systemStatus.value = {
        status: 'disconnected',
        message: `Disconnected: ${event.reason || 'Connection closed'}`,
        timestamp: new Date()
      }
      
      // Auto-reconnect if not paused and not manually disconnected
      if (!paused.value && event.code !== 1000) {
        reconnect()
      }
    }
    
    socket.value.onerror = (err) => {
      console.error('WebSocket connection error:', err)
      error.value = 'WebSocket connection error'
      connecting.value = false
      
      systemStatus.value = {
        status: 'error',
        message: 'WebSocket connection error',
        timestamp: new Date()
      }
      
      if (!paused.value) {
        reconnect()
      }
    }
    
    // Message events - parse JSON messages from backend
    socket.value.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data)
        handleWebSocketMessage(message)
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err)
      }
    }
  }
  
  const handleWebSocketMessage = (message) => {
    if (!message.type) return
    
    switch (message.type) {
      case 'SystemStatus':
        systemStatus.value = {
          status: message.status,
          message: message.message,
          timestamp: new Date()
        }
        break
        
      case 'FileUploadProgress':
        fileUploadProgress.value.set(message.file_key, {
          ...message,
          timestamp: new Date()
        })
        break
        
      case 'FileDownloadProgress':
        fileDownloadProgress.value.set(message.file_key, {
          ...message,
          timestamp: new Date()
        })
        break
        
      case 'CacheStats':
        cacheStats.value = {
          hit_ratio: message.hit_ratio,
          cache_size: message.cache_size,
          timestamp: new Date()
        }
        break
        
      case 'NetworkHealth':
        networkHealth.value = {
          total_operators: message.total_operators,
          online_operators: message.online_operators,
          online_percentage: message.online_percentage,
          can_reach_consensus: message.can_reach_consensus,
          timestamp: new Date()
        }
        break
        
      case 'GovernanceUpdate':
        console.log('Governance update:', message)
        emitToListeners('governance_update', message.data)
        break
        
      case 'OperatorStatusChange':
        console.log('Operator status change:', message)
        emitToListeners('operator_status_change', message)
        break
        
      case 'AdminActionExecuted':
        console.log('Admin action executed:', message)
        emitToListeners('admin_action_executed', message)
        break
        
      case 'Heartbeat':
        // Handle heartbeat - update connection status
        console.log('Heartbeat received')
        break
        
      default:
        console.log('Unknown WebSocket message type:', message.type)
    }
  }
  
  const emit = (event, data) => {
    if (socket.value && connected.value) {
      const message = {
        type: event,
        data: data,
        timestamp: new Date().toISOString()
      }
      socket.value.send(JSON.stringify(message))
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
      const pingMessage = {
        type: 'ping',
        timestamp: startTime
      }
      socket.value.send(JSON.stringify(pingMessage))
      
      // Store ping time for latency calculation
      const pingTimeout = setTimeout(() => {
        console.warn('Ping timeout - no pong received')
      }, 5000)
      
      // Listen for pong in message handler
      const originalOnMessage = socket.value.onmessage
      socket.value.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data)
          if (message.type === 'pong') {
            clearTimeout(pingTimeout)
            const latency = Date.now() - startTime
            console.log(`WebSocket latency: ${latency}ms`)
            socket.value.onmessage = originalOnMessage
          } else {
            originalOnMessage(event)
          }
        } catch (err) {
          originalOnMessage(event)
        }
      }
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