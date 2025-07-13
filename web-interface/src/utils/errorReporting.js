/**
 * Enhanced Error Boundary System
 */

import { ref, computed } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'

export class ErrorReporting {
  constructor() {
    this.errorQueue = []
    this.isReporting = false
    this.breadcrumbs = []
    this.maxBreadcrumbs = 50
    this.userId = null
    this.sessionId = this.generateSessionId()
  }

  /**
   * Generate unique session ID
   */
  generateSessionId() {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  /**
   * Set user ID for error tracking
   */
  setUserId(userId) {
    this.userId = userId
  }

  /**
   * Add breadcrumb for error context
   */
  addBreadcrumb(message, category = 'user', level = 'info', data = {}) {
    const breadcrumb = {
      timestamp: Date.now(),
      message,
      category,
      level,
      data
    }

    this.breadcrumbs.push(breadcrumb)

    // Keep only recent breadcrumbs
    if (this.breadcrumbs.length > this.maxBreadcrumbs) {
      this.breadcrumbs.shift()
    }
  }

  /**
   * Capture error with enhanced context
   */
  captureError(error, errorInfo = {}) {
    const enhancedError = {
      id: this.generateErrorId(),
      timestamp: Date.now(),
      message: error.message,
      stack: error.stack,
      type: error.name || 'Error',
      url: window.location.href,
      userAgent: navigator.userAgent,
      userId: this.userId,
      sessionId: this.sessionId,
      buildVersion: import.meta.env.VITE_BUILD_VERSION || 'unknown',
      breadcrumbs: [...this.breadcrumbs],
      deviceInfo: this.getDeviceInfo(),
      networkInfo: this.getNetworkInfo(),
      memoryInfo: this.getMemoryInfo(),
      ...errorInfo
    }

    this.errorQueue.push(enhancedError)
    this.processErrorQueue()

    return enhancedError
  }

  /**
   * Generate unique error ID
   */
  generateErrorId() {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  /**
   * Get device information
   */
  getDeviceInfo() {
    return {
      platform: navigator.platform,
      language: navigator.language,
      screenResolution: `${screen.width}x${screen.height}`,
      viewportSize: `${window.innerWidth}x${window.innerHeight}`,
      colorDepth: screen.colorDepth,
      pixelRatio: window.devicePixelRatio,
      touchSupport: 'ontouchstart' in window,
      cookieEnabled: navigator.cookieEnabled
    }
  }

  /**
   * Get network information
   */
  getNetworkInfo() {
    const connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection

    if (connection) {
      return {
        effectiveType: connection.effectiveType,
        downlink: connection.downlink,
        rtt: connection.rtt,
        saveData: connection.saveData
      }
    }

    return {
      online: navigator.onLine
    }
  }

  /**
   * Get memory information
   */
  getMemoryInfo() {
    if (performance.memory) {
      return {
        usedJSHeapSize: performance.memory.usedJSHeapSize,
        totalJSHeapSize: performance.memory.totalJSHeapSize,
        jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
      }
    }

    return null
  }

  /**
   * Process error queue
   */
  async processErrorQueue() {
    if (this.isReporting || this.errorQueue.length === 0) return

    this.isReporting = true

    while (this.errorQueue.length > 0) {
      const error = this.errorQueue.shift()
      
      try {
        await this.reportError(error)
      } catch (reportError) {
        console.error('Failed to report error:', reportError)
        
        // Store failed reports locally
        this.storeErrorLocally(error)
      }
    }

    this.isReporting = false
  }

  /**
   * Report error to server
   */
  async reportError(errorData) {
    const response = await fetch('/api/v1/errors/report', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
      },
      body: JSON.stringify(errorData)
    })

    if (!response.ok) {
      throw new Error(`Error reporting failed: ${response.status}`)
    }

    return response.json()
  }

  /**
   * Store error locally for later reporting
   */
  storeErrorLocally(errorData) {
    try {
      const storedErrors = JSON.parse(localStorage.getItem('stored_errors') || '[]')
      storedErrors.push(errorData)
      
      // Keep only recent errors
      const maxStoredErrors = 10
      if (storedErrors.length > maxStoredErrors) {
        storedErrors.splice(0, storedErrors.length - maxStoredErrors)
      }
      
      localStorage.setItem('stored_errors', JSON.stringify(storedErrors))
    } catch (error) {
      console.error('Failed to store error locally:', error)
    }
  }

  /**
   * Retry reporting stored errors
   */
  async retryStoredErrors() {
    try {
      const storedErrors = JSON.parse(localStorage.getItem('stored_errors') || '[]')
      
      for (const error of storedErrors) {
        try {
          await this.reportError(error)
        } catch (reportError) {
          console.warn('Retry reporting failed for error:', error.id)
        }
      }
      
      // Clear successfully reported errors
      localStorage.removeItem('stored_errors')
    } catch (error) {
      console.error('Failed to retry stored errors:', error)
    }
  }
}

/**
 * Error Recovery System
 */
export class ErrorRecovery {
  constructor() {
    this.recoveryStrategies = new Map()
    this.setupDefaultStrategies()
  }

  /**
   * Setup default recovery strategies
   */
  setupDefaultStrategies() {
    // Network error recovery
    this.recoveryStrategies.set('NetworkError', {
      strategy: 'retry',
      maxRetries: 3,
      retryDelay: 1000,
      backoffMultiplier: 2
    })

    // Component error recovery
    this.recoveryStrategies.set('ComponentError', {
      strategy: 'fallback',
      fallbackComponent: 'ErrorFallback'
    })

    // Chunk loading error recovery
    this.recoveryStrategies.set('ChunkLoadError', {
      strategy: 'reload',
      maxRetries: 2
    })

    // API error recovery
    this.recoveryStrategies.set('APIError', {
      strategy: 'cache',
      fallbackToCache: true
    })
  }

  /**
   * Attempt error recovery
   */
  async attemptRecovery(error, context = {}) {
    const errorType = this.categorizeError(error)
    const strategy = this.recoveryStrategies.get(errorType)

    if (!strategy) {
      console.warn(`No recovery strategy for error type: ${errorType}`)
      return false
    }

    try {
      return await this.executeStrategy(strategy, error, context)
    } catch (recoveryError) {
      console.error('Error recovery failed:', recoveryError)
      return false
    }
  }

  /**
   * Categorize error type
   */
  categorizeError(error) {
    if (error.message?.includes('Loading chunk')) {
      return 'ChunkLoadError'
    }
    
    if (error.message?.includes('fetch') || error.message?.includes('network')) {
      return 'NetworkError'
    }
    
    if (error.message?.includes('API') || error.status) {
      return 'APIError'
    }
    
    return 'ComponentError'
  }

  /**
   * Execute recovery strategy
   */
  async executeStrategy(strategy, error, context) {
    switch (strategy.strategy) {
      case 'retry':
        return await this.retryOperation(strategy, error, context)
      
      case 'fallback':
        return this.fallbackComponent(strategy, error, context)
      
      case 'reload':
        return this.reloadChunk(strategy, error, context)
      
      case 'cache':
        return this.useCachedData(strategy, error, context)
      
      default:
        return false
    }
  }

  /**
   * Retry operation with backoff
   */
  async retryOperation(strategy, error, context) {
    const { maxRetries, retryDelay, backoffMultiplier } = strategy
    let delay = retryDelay

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      await new Promise(resolve => setTimeout(resolve, delay))
      
      try {
        if (context.retryFunction) {
          await context.retryFunction()
          return true
        }
      } catch (retryError) {
        console.warn(`Retry attempt ${attempt} failed:`, retryError)
        delay *= backoffMultiplier
      }
    }

    return false
  }

  /**
   * Use fallback component
   */
  fallbackComponent(strategy, error, context) {
    // This would be handled by the error boundary component
    console.log('Using fallback component:', strategy.fallbackComponent)
    return true
  }

  /**
   * Reload failed chunk
   */
  async reloadChunk(strategy, error, context) {
    if (error.message?.includes('Loading chunk')) {
      window.location.reload()
      return true
    }
    return false
  }

  /**
   * Use cached data as fallback
   */
  async useCachedData(strategy, error, context) {
    // Implementation would check cache for fallback data
    console.log('Attempting to use cached data for error:', error.message)
    return false
  }
}

export const errorReporting = new ErrorReporting()
export const errorRecovery = new ErrorRecovery()

// Global error handlers
window.addEventListener('error', (event) => {
  errorReporting.addBreadcrumb(`Global error: ${event.error?.message}`, 'error')
  errorReporting.captureError(event.error)
})

window.addEventListener('unhandledrejection', (event) => {
  errorReporting.addBreadcrumb(`Unhandled promise rejection: ${event.reason}`, 'error')
  errorReporting.captureError(new Error(event.reason))
})

export default {
  ErrorReporting,
  ErrorRecovery,
  errorReporting,
  errorRecovery
}
