/**
 * WebSocket composable with proper cleanup and memory leak prevention
 */

import { ref, onUnmounted, getCurrentInstance } from 'vue'
import { useWebSocketStore } from '@/store/websocket'

export function useWebSocket() {
  const webSocketStore = useWebSocketStore()
  const instance = getCurrentInstance()
  
  // Track registered event handlers for cleanup
  const eventHandlers = new Map()
  
  /**
   * Register an event handler with automatic cleanup
   * @param {string} event - Event name
   * @param {Function} handler - Event handler function
   */
  const on = (event, handler) => {
    // Store the handler for cleanup
    if (!eventHandlers.has(event)) {
      eventHandlers.set(event, new Set())
    }
    eventHandlers.get(event).add(handler)
    
    // Register with WebSocket store
    webSocketStore.on(event, handler)
  }
  
  /**
   * Remove an event handler
   * @param {string} event - Event name
   * @param {Function} handler - Event handler function
   */
  const off = (event, handler) => {
    // Remove from tracking
    if (eventHandlers.has(event)) {
      eventHandlers.get(event).delete(handler)
      if (eventHandlers.get(event).size === 0) {
        eventHandlers.delete(event)
      }
    }
    
    // Remove from WebSocket store
    webSocketStore.off(event, handler)
  }
  
  /**
   * Emit an event
   * @param {string} event - Event name
   * @param {any} data - Event data
   */
  const emit = (event, data) => {
    webSocketStore.emit(event, data)
  }
  
  /**
   * Clean up all registered event handlers
   */
  const cleanup = () => {
    eventHandlers.forEach((handlers, event) => {
      handlers.forEach(handler => {
        webSocketStore.off(event, handler)
      })
    })
    eventHandlers.clear()
  }
  
  /**
   * Subscribe to system updates
   */
  const subscribeToSystemUpdates = () => {
    webSocketStore.subscribeToSystemUpdates()
  }
  
  /**
   * Unsubscribe from system updates
   */
  const unsubscribeFromSystemUpdates = () => {
    webSocketStore.unsubscribeFromSystemUpdates()
  }
  
  /**
   * Subscribe to governance updates
   */
  const subscribeToGovernanceUpdates = () => {
    webSocketStore.subscribeToGovernanceUpdates()
  }
  
  /**
   * Unsubscribe from governance updates
   */
  const unsubscribeFromGovernanceUpdates = () => {
    webSocketStore.unsubscribeFromGovernanceUpdates()
  }
  
  // Connection status and methods
  const connectionStatus = ref(webSocketStore.connectionStatus)
  const isConnected = ref(webSocketStore.isConnected)
  
  // Automatic cleanup on component unmount
  onUnmounted(() => {
    cleanup()
  })
  
  return {
    // Event methods
    on,
    off,
    emit,
    cleanup,
    
    // Subscription methods
    subscribeToSystemUpdates,
    unsubscribeFromSystemUpdates,
    subscribeToGovernanceUpdates,
    unsubscribeFromGovernanceUpdates,
    
    // Connection status
    connectionStatus,
    isConnected,
    
    // Store methods
    connect: webSocketStore.connect,
    disconnect: webSocketStore.disconnect,
    pauseConnection: webSocketStore.pauseConnection,
    resumeConnection: webSocketStore.resumeConnection
  }
}

/**
 * Composable for managing intervals with automatic cleanup
 */
export function useInterval() {
  const intervals = new Set()
  
  /**
   * Create an interval with automatic cleanup
   * @param {Function} callback - Callback function
   * @param {number} delay - Delay in milliseconds
   * @returns {number} Interval ID
   */
  const setInterval = (callback, delay) => {
    const id = window.setInterval(callback, delay)
    intervals.add(id)
    return id
  }
  
  /**
   * Clear a specific interval
   * @param {number} id - Interval ID
   */
  const clearInterval = (id) => {
    window.clearInterval(id)
    intervals.delete(id)
  }
  
  /**
   * Clear all intervals
   */
  const clearAll = () => {
    intervals.forEach(id => window.clearInterval(id))
    intervals.clear()
  }
  
  // Automatic cleanup on component unmount
  onUnmounted(() => {
    clearAll()
  })
  
  return {
    setInterval,
    clearInterval,
    clearAll
  }
}

/**
 * Composable for managing timeouts with automatic cleanup
 */
export function useTimeout() {
  const timeouts = new Set()
  
  /**
   * Create a timeout with automatic cleanup
   * @param {Function} callback - Callback function
   * @param {number} delay - Delay in milliseconds
   * @returns {number} Timeout ID
   */
  const setTimeout = (callback, delay) => {
    const id = window.setTimeout(() => {
      callback()
      timeouts.delete(id)
    }, delay)
    timeouts.add(id)
    return id
  }
  
  /**
   * Clear a specific timeout
   * @param {number} id - Timeout ID
   */
  const clearTimeout = (id) => {
    window.clearTimeout(id)
    timeouts.delete(id)
  }
  
  /**
   * Clear all timeouts
   */
  const clearAll = () => {
    timeouts.forEach(id => window.clearTimeout(id))
    timeouts.clear()
  }
  
  // Automatic cleanup on component unmount
  onUnmounted(() => {
    clearAll()
  })
  
  return {
    setTimeout,
    clearTimeout,
    clearAll
  }
}

/**
 * Composable for managing event listeners with automatic cleanup
 */
export function useEventListener() {
  const listeners = new Set()
  
  /**
   * Add event listener with automatic cleanup
   * @param {EventTarget} target - Event target
   * @param {string} event - Event name
   * @param {Function} handler - Event handler
   * @param {Object} options - Event options
   */
  const addEventListener = (target, event, handler, options = {}) => {
    target.addEventListener(event, handler, options)
    
    const listener = { target, event, handler, options }
    listeners.add(listener)
    
    return () => removeEventListener(target, event, handler)
  }
  
  /**
   * Remove event listener
   * @param {EventTarget} target - Event target
   * @param {string} event - Event name
   * @param {Function} handler - Event handler
   */
  const removeEventListener = (target, event, handler) => {
    target.removeEventListener(event, handler)
    
    // Remove from tracking
    listeners.forEach(listener => {
      if (listener.target === target && listener.event === event && listener.handler === handler) {
        listeners.delete(listener)
      }
    })
  }
  
  /**
   * Clean up all event listeners
   */
  const cleanup = () => {
    listeners.forEach(({ target, event, handler }) => {
      target.removeEventListener(event, handler)
    })
    listeners.clear()
  }
  
  // Automatic cleanup on component unmount
  onUnmounted(() => {
    cleanup()
  })
  
  return {
    addEventListener,
    removeEventListener,
    cleanup
  }
}

/**
 * Composable for managing async operations with proper cleanup
 */
export function useAsyncOperation() {
  const operations = new Set()
  
  /**
   * Execute an async operation with cleanup tracking
   * @param {Function} operation - Async operation function
   * @returns {Promise} Promise with cancellation support
   */
  const execute = (operation) => {
    const controller = new AbortController()
    
    const promise = new Promise(async (resolve, reject) => {
      try {
        const result = await operation(controller.signal)
        if (!controller.signal.aborted) {
          resolve(result)
        }
      } catch (error) {
        if (!controller.signal.aborted) {
          reject(error)
        }
      }
    })
    
    const operationData = { promise, controller }
    operations.add(operationData)
    
    // Clean up when operation completes
    promise.finally(() => {
      operations.delete(operationData)
    })
    
    return promise
  }
  
  /**
   * Cancel all pending operations
   */
  const cancelAll = () => {
    operations.forEach(({ controller }) => {
      controller.abort()
    })
    operations.clear()
  }
  
  // Automatic cleanup on component unmount
  onUnmounted(() => {
    cancelAll()
  })
  
  return {
    execute,
    cancelAll
  }
}

export default {
  useWebSocket,
  useInterval,
  useTimeout,
  useEventListener,
  useAsyncOperation
}