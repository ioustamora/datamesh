/**
 * Global error handler plugin for Vue.js
 * Provides centralized error handling and reporting
 */

import { ElMessage, ElNotification } from 'element-plus'

// Error types for categorization
const ERROR_TYPES = {
  NETWORK: 'network',
  AUTHENTICATION: 'authentication',
  AUTHORIZATION: 'authorization',
  VALIDATION: 'validation',
  RUNTIME: 'runtime',
  CHUNK_LOAD: 'chunk_load',
  UNKNOWN: 'unknown'
}

// Error severity levels
const ERROR_SEVERITY = {
  LOW: 'low',
  MEDIUM: 'medium',
  HIGH: 'high',
  CRITICAL: 'critical'
}

class ErrorHandler {
  constructor() {
    this.errorQueue = []
    this.maxQueueSize = 100
    this.reportingEnabled = true
    this.userNotificationThreshold = ERROR_SEVERITY.MEDIUM
    this.retryAttempts = new Map()
    this.maxRetryAttempts = 3
    
    // Initialize error tracking
    this.initializeErrorTracking()
  }

  /**
   * Initialize error tracking systems
   */
  initializeErrorTracking() {
    // Global error handlers
    window.addEventListener('error', this.handleGlobalError.bind(this))
    window.addEventListener('unhandledrejection', this.handleUnhandledRejection.bind(this))
    
    // Network error monitoring
    this.setupNetworkMonitoring()
    
    // Performance monitoring
    this.setupPerformanceMonitoring()
  }

  /**
   * Handle global JavaScript errors
   * @param {ErrorEvent} event - Error event
   */
  handleGlobalError(event) {
    const error = {
      type: ERROR_TYPES.RUNTIME,
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      stack: event.error?.stack,
      timestamp: new Date().toISOString(),
      severity: this.determineSeverity(event.error)
    }

    this.processError(error)
  }

  /**
   * Handle unhandled promise rejections
   * @param {PromiseRejectionEvent} event - Promise rejection event
   */
  handleUnhandledRejection(event) {
    const error = {
      type: this.determineErrorType(event.reason),
      message: event.reason?.message || 'Unhandled promise rejection',
      reason: event.reason,
      stack: event.reason?.stack,
      timestamp: new Date().toISOString(),
      severity: this.determineSeverity(event.reason)
    }

    this.processError(error)
    
    // Prevent default browser behavior
    event.preventDefault()
  }

  /**
   * Handle Vue.js errors
   * @param {Error} error - Vue error
   * @param {Object} instance - Vue instance
   * @param {string} info - Error info
   */
  handleVueError(error, instance, info) {
    const errorData = {
      type: ERROR_TYPES.RUNTIME,
      message: error.message,
      stack: error.stack,
      component: instance?.$options.name || 'Unknown',
      lifecycle: info,
      timestamp: new Date().toISOString(),
      severity: this.determineSeverity(error)
    }

    this.processError(errorData)
  }

  /**
   * Handle API errors
   * @param {Object} error - API error object
   * @param {string} endpoint - API endpoint
   * @param {string} method - HTTP method
   */
  handleAPIError(error, endpoint, method) {
    const errorData = {
      type: this.determineAPIErrorType(error),
      message: error.message || 'API request failed',
      endpoint,
      method,
      status: error.response?.status,
      statusText: error.response?.statusText,
      data: error.response?.data,
      timestamp: new Date().toISOString(),
      severity: this.determineAPIErrorSeverity(error)
    }

    this.processError(errorData)
  }

  /**
   * Process and handle errors
   * @param {Object} error - Error data
   */
  processError(error) {
    // Add to error queue
    this.addToQueue(error)
    
    // Log to console in development
    if (import.meta.env.DEV) {
      console.error('Error processed:', error)
    }
    
    // Send to monitoring service
    this.reportError(error)
    
    // Show user notification if needed
    this.showUserNotification(error)
    
    // Attempt automatic recovery
    this.attemptRecovery(error)
  }

  /**
   * Add error to processing queue
   * @param {Object} error - Error data
   */
  addToQueue(error) {
    this.errorQueue.push(error)
    
    // Maintain queue size
    if (this.errorQueue.length > this.maxQueueSize) {
      this.errorQueue.shift()
    }
  }

  /**
   * Determine error type
   * @param {Error} error - Error object
   * @returns {string} Error type
   */
  determineErrorType(error) {
    if (!error) return ERROR_TYPES.UNKNOWN
    
    if (error.name === 'ChunkLoadError') {
      return ERROR_TYPES.CHUNK_LOAD
    }
    
    if (error.name === 'NetworkError' || error.code === 'NETWORK_ERROR') {
      return ERROR_TYPES.NETWORK
    }
    
    if (error.response?.status === 401) {
      return ERROR_TYPES.AUTHENTICATION
    }
    
    if (error.response?.status === 403) {
      return ERROR_TYPES.AUTHORIZATION
    }
    
    if (error.response?.status >= 400 && error.response?.status < 500) {
      return ERROR_TYPES.VALIDATION
    }
    
    return ERROR_TYPES.RUNTIME
  }

  /**
   * Determine API error type
   * @param {Object} error - API error
   * @returns {string} Error type
   */
  determineAPIErrorType(error) {
    if (error.response?.status === 401) {
      return ERROR_TYPES.AUTHENTICATION
    }
    
    if (error.response?.status === 403) {
      return ERROR_TYPES.AUTHORIZATION
    }
    
    if (error.response?.status >= 400 && error.response?.status < 500) {
      return ERROR_TYPES.VALIDATION
    }
    
    if (error.code === 'ECONNABORTED' || error.code === 'NETWORK_ERROR') {
      return ERROR_TYPES.NETWORK
    }
    
    return ERROR_TYPES.RUNTIME
  }

  /**
   * Determine error severity
   * @param {Error} error - Error object
   * @returns {string} Severity level
   */
  determineSeverity(error) {
    if (!error) return ERROR_SEVERITY.LOW
    
    if (error.name === 'ChunkLoadError') {
      return ERROR_SEVERITY.HIGH
    }
    
    if (error.name === 'TypeError' || error.name === 'ReferenceError') {
      return ERROR_SEVERITY.HIGH
    }
    
    if (error.message?.includes('Cannot read property')) {
      return ERROR_SEVERITY.MEDIUM
    }
    
    return ERROR_SEVERITY.LOW
  }

  /**
   * Determine API error severity
   * @param {Object} error - API error
   * @returns {string} Severity level
   */
  determineAPIErrorSeverity(error) {
    if (error.response?.status >= 500) {
      return ERROR_SEVERITY.CRITICAL
    }
    
    if (error.response?.status === 401 || error.response?.status === 403) {
      return ERROR_SEVERITY.HIGH
    }
    
    if (error.response?.status >= 400) {
      return ERROR_SEVERITY.MEDIUM
    }
    
    return ERROR_SEVERITY.LOW
  }

  /**
   * Show user notification based on error severity
   * @param {Object} error - Error data
   */
  showUserNotification(error) {
    if (error.severity === ERROR_SEVERITY.LOW) {
      return // Don't show notifications for low severity errors
    }
    
    const userMessage = this.getUserFriendlyMessage(error)
    
    if (error.severity === ERROR_SEVERITY.CRITICAL) {
      ElNotification({
        title: 'Critical Error',
        message: userMessage,
        type: 'error',
        duration: 0, // Don't auto-close
        showClose: true
      })
    } else if (error.severity === ERROR_SEVERITY.HIGH) {
      ElMessage({
        message: userMessage,
        type: 'error',
        duration: 5000,
        showClose: true
      })
    } else if (error.severity === ERROR_SEVERITY.MEDIUM) {
      ElMessage({
        message: userMessage,
        type: 'warning',
        duration: 3000
      })
    }
  }

  /**
   * Get user-friendly error message
   * @param {Object} error - Error data
   * @returns {string} User-friendly message
   */
  getUserFriendlyMessage(error) {
    switch (error.type) {
      case ERROR_TYPES.NETWORK:
        return 'Connection error. Please check your internet connection and try again.'
      
      case ERROR_TYPES.AUTHENTICATION:
        return 'Authentication failed. Please log in again.'
      
      case ERROR_TYPES.AUTHORIZATION:
        return 'You don\'t have permission to perform this action.'
      
      case ERROR_TYPES.VALIDATION:
        return 'Invalid input. Please check your data and try again.'
      
      case ERROR_TYPES.CHUNK_LOAD:
        return 'Failed to load application resources. Please refresh the page.'
      
      case ERROR_TYPES.RUNTIME:
        return 'An unexpected error occurred. Please try again.'
      
      default:
        return 'Something went wrong. Please try again or contact support.'
    }
  }

  /**
   * Attempt automatic error recovery
   * @param {Object} error - Error data
   */
  attemptRecovery(error) {
    const errorKey = this.getErrorKey(error)
    const attempts = this.retryAttempts.get(errorKey) || 0
    
    if (attempts >= this.maxRetryAttempts) {
      return
    }
    
    switch (error.type) {
      case ERROR_TYPES.NETWORK:
        this.retryNetworkOperation(error, attempts)
        break
      
      case ERROR_TYPES.CHUNK_LOAD:
        this.retryChunkLoad(error, attempts)
        break
      
      case ERROR_TYPES.AUTHENTICATION:
        this.handleAuthenticationError(error)
        break
      
      default:
        // No automatic recovery for other error types
        break
    }
  }

  /**
   * Retry network operation
   * @param {Object} error - Error data
   * @param {number} attempts - Previous attempts
   */
  retryNetworkOperation(error, attempts) {
    const errorKey = this.getErrorKey(error)
    const delay = Math.pow(2, attempts) * 1000 // Exponential backoff
    
    setTimeout(() => {
      this.retryAttempts.set(errorKey, attempts + 1)
      // Network retry logic would go here
      console.log(`Retrying network operation (attempt ${attempts + 1})`)
    }, delay)
  }

  /**
   * Retry chunk loading
   * @param {Object} error - Error data
   * @param {number} attempts - Previous attempts
   */
  retryChunkLoad(error, attempts) {
    const errorKey = this.getErrorKey(error)
    
    setTimeout(() => {
      this.retryAttempts.set(errorKey, attempts + 1)
      // Chunk retry logic would go here
      console.log(`Retrying chunk load (attempt ${attempts + 1})`)
      window.location.reload()
    }, 1000)
  }

  /**
   * Handle authentication errors
   * @param {Object} error - Error data
   */
  handleAuthenticationError(error) {
    // Clear auth tokens
    localStorage.removeItem('datamesh_token')
    sessionStorage.clear()
    
    // Redirect to login
    window.location.href = '/auth/login'
  }

  /**
   * Get unique error key for tracking
   * @param {Object} error - Error data
   * @returns {string} Error key
   */
  getErrorKey(error) {
    return `${error.type}_${error.message}_${error.endpoint || ''}`
  }

  /**
   * Report error to monitoring service
   * @param {Object} error - Error data
   */
  reportError(error) {
    if (!this.reportingEnabled) return
    
    // In production, send to error tracking service
    if (import.meta.env.PROD) {
      // Example: Send to Sentry, LogRocket, or custom service
      this.sendToMonitoringService(error)
    }
  }

  /**
   * Send error to monitoring service
   * @param {Object} error - Error data
   */
  sendToMonitoringService(error) {
    // Implementation would depend on your monitoring service
    // Example for a generic service:
    
    fetch('/api/v1/errors', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        error,
        userAgent: navigator.userAgent,
        url: window.location.href,
        timestamp: new Date().toISOString()
      })
    }).catch(err => {
      console.error('Failed to report error:', err)
    })
  }

  /**
   * Setup network monitoring
   */
  setupNetworkMonitoring() {
    // Monitor online/offline status
    window.addEventListener('online', () => {
      ElMessage.success('Connection restored')
    })
    
    window.addEventListener('offline', () => {
      ElMessage.warning('Connection lost. Working offline.')
    })
  }

  /**
   * Setup performance monitoring
   */
  setupPerformanceMonitoring() {
    // Monitor performance issues
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.duration > 5000) { // 5 second threshold
            this.processError({
              type: 'performance',
              message: `Slow operation detected: ${entry.name}`,
              duration: entry.duration,
              timestamp: new Date().toISOString(),
              severity: ERROR_SEVERITY.LOW
            })
          }
        }
      })
      
      observer.observe({ entryTypes: ['measure', 'navigation'] })
    }
  }

  /**
   * Get error statistics
   * @returns {Object} Error statistics
   */
  getErrorStats() {
    const stats = {
      total: this.errorQueue.length,
      byType: {},
      bySeverity: {},
      recent: this.errorQueue.slice(-10)
    }
    
    this.errorQueue.forEach(error => {
      stats.byType[error.type] = (stats.byType[error.type] || 0) + 1
      stats.bySeverity[error.severity] = (stats.bySeverity[error.severity] || 0) + 1
    })
    
    return stats
  }

  /**
   * Clear error queue
   */
  clearErrorQueue() {
    this.errorQueue = []
    this.retryAttempts.clear()
  }

  /**
   * Enable/disable error reporting
   * @param {boolean} enabled - Whether to enable reporting
   */
  setReportingEnabled(enabled) {
    this.reportingEnabled = enabled
  }
}

// Create singleton instance
const errorHandler = new ErrorHandler()

// Vue plugin
export default {
  install(app) {
    // Set up Vue error handler
    app.config.errorHandler = errorHandler.handleVueError.bind(errorHandler)
    
    // Make error handler available globally
    app.config.globalProperties.$errorHandler = errorHandler
    app.provide('errorHandler', errorHandler)
  }
}

// Export error handler instance
export { errorHandler, ERROR_TYPES, ERROR_SEVERITY }