/**
 * Advanced Error Boundary System with context-aware error handling
 */

import { ElMessage, ElNotification } from 'element-plus'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

// Error types and classification
export const ERROR_TYPES = {
  NETWORK: 'network',
  AUTHENTICATION: 'authentication',
  AUTHORIZATION: 'authorization',
  VALIDATION: 'validation',
  SYSTEM: 'system',
  UNKNOWN: 'unknown'
}

export const ERROR_SEVERITY = {
  LOW: 'low',
  MEDIUM: 'medium',
  HIGH: 'high',
  CRITICAL: 'critical'
}

// Error classification rules
const ERROR_CLASSIFICATION = {
  [ERROR_TYPES.NETWORK]: {
    patterns: [/network/i, /timeout/i, /connection/i, /fetch/i],
    severity: ERROR_SEVERITY.MEDIUM,
    recoverable: true,
    userMessage: 'Network connection issue. Please check your internet connection.'
  },
  [ERROR_TYPES.AUTHENTICATION]: {
    patterns: [/401/, /unauthorized/i, /authentication/i],
    severity: ERROR_SEVERITY.HIGH,
    recoverable: true,
    userMessage: 'Authentication failed. Please log in again.'
  },
  [ERROR_TYPES.AUTHORIZATION]: {
    patterns: [/403/, /forbidden/i, /authorization/i],
    severity: ERROR_SEVERITY.HIGH,
    recoverable: false,
    userMessage: 'You don\'t have permission to access this resource.'
  },
  [ERROR_TYPES.VALIDATION]: {
    patterns: [/400/, /validation/i, /invalid/i],
    severity: ERROR_SEVERITY.LOW,
    recoverable: true,
    userMessage: 'Please check your input and try again.'
  },
  [ERROR_TYPES.SYSTEM]: {
    patterns: [/500/, /internal/i, /server/i],
    severity: ERROR_SEVERITY.CRITICAL,
    recoverable: false,
    userMessage: 'System error occurred. Our team has been notified.'
  }
}

// Error store for tracking and analytics
class ErrorStore {
  constructor() {
    this.errors = ref([])
    this.errorCount = ref(0)
    this.lastError = ref(null)
    this.errorHistory = ref([])
    this.maxHistorySize = 50
  }

  addError(error) {
    const errorEntry = {
      id: Date.now() + Math.random(),
      timestamp: new Date().toISOString(),
      error: this.serializeError(error),
      type: this.classifyError(error),
      severity: this.getSeverity(error),
      context: this.getContext(),
      recovered: false,
      attempts: 0
    }

    this.errors.value.unshift(errorEntry)
    this.errorCount.value++
    this.lastError.value = errorEntry
    
    // Maintain history size
    if (this.errorHistory.value.length >= this.maxHistorySize) {
      this.errorHistory.value.pop()
    }
    this.errorHistory.value.unshift(errorEntry)

    return errorEntry
  }

  markRecovered(errorId) {
    const error = this.errors.value.find(e => e.id === errorId)
    if (error) {
      error.recovered = true
      error.recoveredAt = new Date().toISOString()
    }
  }

  incrementAttempts(errorId) {
    const error = this.errors.value.find(e => e.id === errorId)
    if (error) {
      error.attempts++
    }
  }

  clearErrors() {
    this.errors.value = []
    this.errorCount.value = 0
    this.lastError.value = null
  }

  serializeError(error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack,
      cause: error.cause,
      code: error.code,
      status: error.status
    }
  }

  classifyError(error) {
    const message = error.message || ''
    const status = error.status || error.code || ''
    const fullText = `${message} ${status}`.toLowerCase()

    for (const [type, config] of Object.entries(ERROR_CLASSIFICATION)) {
      if (config.patterns.some(pattern => pattern.test(fullText))) {
        return type
      }
    }

    return ERROR_TYPES.UNKNOWN
  }

  getSeverity(error) {
    const type = this.classifyError(error)
    return ERROR_CLASSIFICATION[type]?.severity || ERROR_SEVERITY.MEDIUM
  }

  getContext() {
    return {
      url: window.location.href,
      userAgent: navigator.userAgent,
      timestamp: Date.now(),
      viewport: {
        width: window.innerWidth,
        height: window.innerHeight
      },
      memory: performance.memory ? {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize
      } : null
    }
  }

  getErrorStats() {
    const stats = {
      total: this.errorCount.value,
      byType: {},
      bySeverity: {},
      recoveryRate: 0
    }

    this.errorHistory.value.forEach(error => {
      stats.byType[error.type] = (stats.byType[error.type] || 0) + 1
      stats.bySeverity[error.severity] = (stats.bySeverity[error.severity] || 0) + 1
    })

    const recoveredCount = this.errorHistory.value.filter(e => e.recovered).length
    stats.recoveryRate = this.errorHistory.value.length > 0 
      ? (recoveredCount / this.errorHistory.value.length) * 100 
      : 0

    return stats
  }
}

export const errorStore = new ErrorStore()

// Error recovery strategies
export class ErrorRecoveryManager {
  constructor() {
    this.strategies = new Map()
    this.setupDefaultStrategies()
  }

  setupDefaultStrategies() {
    // Network error recovery
    this.strategies.set(ERROR_TYPES.NETWORK, {
      maxAttempts: 3,
      backoffMultiplier: 2,
      baseDelay: 1000,
      async recover(error, attempt) {
        const delay = this.baseDelay * Math.pow(this.backoffMultiplier, attempt - 1)
        await new Promise(resolve => setTimeout(resolve, delay))
        
        // Try to refetch or reconnect
        if (error.context?.originalRequest) {
          return fetch(error.context.originalRequest)
        }
        
        return Promise.resolve()
      }
    })

    // Authentication error recovery
    this.strategies.set(ERROR_TYPES.AUTHENTICATION, {
      maxAttempts: 1,
      async recover(error) {
        // Redirect to login or refresh token
        const router = useRouter()
        await router.push('/login')
        return Promise.resolve()
      }
    })

    // Validation error recovery
    this.strategies.set(ERROR_TYPES.VALIDATION, {
      maxAttempts: 1,
      async recover(error) {
        // Show validation errors to user
        ElMessage({
          type: 'warning',
          message: error.message || 'Please check your input',
          duration: 5000
        })
        return Promise.resolve()
      }
    })
  }

  async attemptRecovery(errorEntry) {
    const strategy = this.strategies.get(errorEntry.type)
    if (!strategy) return false

    if (errorEntry.attempts >= strategy.maxAttempts) {
      return false
    }

    try {
      errorStore.incrementAttempts(errorEntry.id)
      await strategy.recover(errorEntry.error, errorEntry.attempts)
      errorStore.markRecovered(errorEntry.id)
      return true
    } catch (recoveryError) {
      console.error('Recovery failed:', recoveryError)
      return false
    }
  }

  registerStrategy(errorType, strategy) {
    this.strategies.set(errorType, strategy)
  }
}

export const recoveryManager = new ErrorRecoveryManager()

// Error reporting service
export class ErrorReporter {
  constructor() {
    this.endpoint = '/api/v1/errors'
    this.batchSize = 10
    this.batchTimeout = 5000
    this.errorBatch = []
    this.batchTimer = null
    this.enabled = true
  }

  async reportError(errorEntry) {
    if (!this.enabled) return

    // Add to batch
    this.errorBatch.push({
      ...errorEntry,
      userId: this.getUserId(),
      sessionId: this.getSessionId(),
      buildVersion: this.getBuildVersion()
    })

    // Send batch if full
    if (this.errorBatch.length >= this.batchSize) {
      await this.sendBatch()
    } else {
      this.scheduleBatchSend()
    }
  }

  scheduleBatchSend() {
    if (this.batchTimer) {
      clearTimeout(this.batchTimer)
    }

    this.batchTimer = setTimeout(() => {
      this.sendBatch()
    }, this.batchTimeout)
  }

  async sendBatch() {
    if (this.errorBatch.length === 0) return

    try {
      const response = await fetch(this.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          errors: this.errorBatch,
          timestamp: new Date().toISOString()
        })
      })

      if (response.ok) {
        console.log(`Sent ${this.errorBatch.length} error reports`)
      }
    } catch (error) {
      console.error('Failed to send error reports:', error)
    }

    this.errorBatch = []
    if (this.batchTimer) {
      clearTimeout(this.batchTimer)
      this.batchTimer = null
    }
  }

  getUserId() {
    // Get from auth store or localStorage
    return localStorage.getItem('userId') || 'anonymous'
  }

  getSessionId() {
    // Get or create session ID
    let sessionId = sessionStorage.getItem('sessionId')
    if (!sessionId) {
      sessionId = Date.now().toString(36) + Math.random().toString(36).substr(2)
      sessionStorage.setItem('sessionId', sessionId)
    }
    return sessionId
  }

  getBuildVersion() {
    return process.env.VUE_APP_VERSION || 'unknown'
  }

  disable() {
    this.enabled = false
  }

  enable() {
    this.enabled = true
  }
}

export const errorReporter = new ErrorReporter()

// Error boundary composable
export function useErrorBoundary() {
  const router = useRouter()
  const isRecovering = ref(false)
  const currentError = ref(null)

  const handleError = async (error, instance, info) => {
    console.error('Error boundary caught:', error, info)

    // Store error
    const errorEntry = errorStore.addError(error)
    currentError.value = errorEntry

    // Report error
    await errorReporter.reportError(errorEntry)

    // Show user notification
    showErrorNotification(errorEntry)

    // Attempt recovery
    if (ERROR_CLASSIFICATION[errorEntry.type]?.recoverable) {
      isRecovering.value = true
      
      try {
        const recovered = await recoveryManager.attemptRecovery(errorEntry)
        if (recovered) {
          ElMessage({
            type: 'success',
            message: 'Issue resolved automatically'
          })
          currentError.value = null
        }
      } catch (recoveryError) {
        console.error('Recovery failed:', recoveryError)
      } finally {
        isRecovering.value = false
      }
    }

    // Handle critical errors
    if (errorEntry.severity === ERROR_SEVERITY.CRITICAL) {
      handleCriticalError(errorEntry)
    }
  }

  const showErrorNotification = (errorEntry) => {
    const config = ERROR_CLASSIFICATION[errorEntry.type]
    const message = config?.userMessage || 'An unexpected error occurred'

    ElNotification({
      title: 'Error',
      message,
      type: 'error',
      duration: config?.recoverable ? 5000 : 0,
      showClose: true
    })
  }

  const handleCriticalError = (errorEntry) => {
    // Log critical error
    console.error('CRITICAL ERROR:', errorEntry)

    // Show modal or redirect to error page
    ElNotification({
      title: 'Critical Error',
      message: 'A critical error occurred. Please refresh the page or contact support.',
      type: 'error',
      duration: 0,
      showClose: true
    })

    // Optionally redirect to error page
    setTimeout(() => {
      router.push('/error')
    }, 3000)
  }

  const retry = async () => {
    if (currentError.value) {
      isRecovering.value = true
      
      try {
        const recovered = await recoveryManager.attemptRecovery(currentError.value)
        if (recovered) {
          currentError.value = null
          ElMessage({
            type: 'success',
            message: 'Retry successful'
          })
        } else {
          ElMessage({
            type: 'error',
            message: 'Retry failed. Please try again later.'
          })
        }
      } finally {
        isRecovering.value = false
      }
    }
  }

  const clearError = () => {
    currentError.value = null
  }

  return {
    handleError,
    retry,
    clearError,
    isRecovering: computed(() => isRecovering.value),
    currentError: computed(() => currentError.value),
    errorStats: computed(() => errorStore.getErrorStats())
  }
}

// Global error handler setup
export function setupGlobalErrorHandling() {
  // Vue error handler
  const app = getCurrentInstance()?.appContext.app
  if (app) {
    app.config.errorHandler = (error, instance, info) => {
      const { handleError } = useErrorBoundary()
      handleError(error, instance, info)
    }
  }

  // Global promise rejection handler
  window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason)
    const { handleError } = useErrorBoundary()
    handleError(event.reason, null, 'unhandledrejection')
  })

  // Global error handler
  window.addEventListener('error', (event) => {
    console.error('Global error:', event.error)
    const { handleError } = useErrorBoundary()
    handleError(event.error, null, 'global')
  })
}

// Resource loading error handler
export function setupResourceErrorHandling() {
  // Handle image loading errors
  document.addEventListener('error', (event) => {
    if (event.target.tagName === 'IMG') {
      const img = event.target
      if (!img.dataset.fallbackAttempted) {
        img.dataset.fallbackAttempted = 'true'
        img.src = '/icons/image-error.svg'
      }
    }
  }, true)

  // Handle script loading errors
  document.addEventListener('error', (event) => {
    if (event.target.tagName === 'SCRIPT') {
      console.error('Script loading failed:', event.target.src)
      // Could implement script retry logic here
    }
  }, true)
}

// Performance monitoring integration
export function setupPerformanceErrorMonitoring() {
  // Monitor performance issues
  if ('PerformanceObserver' in window) {
    const observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (entry.name === 'long-task') {
          console.warn('Long task detected:', entry.duration)
          // Could report as performance error
        }
      }
    })
    
    observer.observe({ entryTypes: ['longtask'] })
  }

  // Monitor memory usage
  setInterval(() => {
    if (performance.memory) {
      const memoryUsage = performance.memory.usedJSHeapSize / performance.memory.totalJSHeapSize
      if (memoryUsage > 0.9) {
        console.warn('High memory usage detected:', memoryUsage)
        // Could trigger cleanup or report as error
      }
    }
  }, 30000) // Check every 30 seconds
}

export default {
  useErrorBoundary,
  setupGlobalErrorHandling,
  setupResourceErrorHandling,
  setupPerformanceErrorMonitoring,
  errorStore,
  recoveryManager,
  errorReporter,
  ERROR_TYPES,
  ERROR_SEVERITY
}
