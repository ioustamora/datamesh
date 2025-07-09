<template>
  <div class="error-boundary">
    <div v-if="!hasError" class="error-boundary-content">
      <slot />
    </div>
    
    <div v-else class="error-boundary-fallback">
      <div class="error-container">
        <div class="error-icon">
          <el-icon size="48" color="#F56C6C">
            <Warning />
          </el-icon>
        </div>
        
        <div class="error-content">
          <h2 class="error-title">Something went wrong</h2>
          <p class="error-message">{{ errorMessage }}</p>
          
          <div class="error-details" v-if="showDetails">
            <el-collapse v-model="detailsOpen">
              <el-collapse-item name="error-details">
                <template #title>
                  <span class="details-title">Error Details</span>
                </template>
                
                <div class="error-info">
                  <div class="info-item">
                    <strong>Error Type:</strong> {{ errorInfo.type }}
                  </div>
                  <div class="info-item">
                    <strong>Component:</strong> {{ errorInfo.componentName }}
                  </div>
                  <div class="info-item">
                    <strong>Time:</strong> {{ errorInfo.timestamp }}
                  </div>
                  <div class="info-item" v-if="errorInfo.stack">
                    <strong>Stack Trace:</strong>
                    <pre class="stack-trace">{{ errorInfo.stack }}</pre>
                  </div>
                </div>
              </el-collapse-item>
            </el-collapse>
          </div>
          
          <div class="error-actions">
            <el-button type="primary" @click="retry">
              <el-icon><RefreshRight /></el-icon>
              Try Again
            </el-button>
            
            <el-button @click="goHome">
              <el-icon><House /></el-icon>
              Go to Dashboard
            </el-button>
            
            <el-button @click="reportError" v-if="canReport">
              <el-icon><Warning /></el-icon>
              Report Issue
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onErrorCaptured, getCurrentInstance, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Warning, RefreshRight, House } from '@element-plus/icons-vue'

export default {
  name: 'ErrorBoundary',
  components: {
    Warning,
    RefreshRight,
    House
  },
  props: {
    // Custom error message
    fallbackMessage: {
      type: String,
      default: 'An unexpected error occurred. Please try again.'
    },
    // Whether to show detailed error information
    showDetails: {
      type: Boolean,
      default: import.meta.env.DEV
    },
    // Whether to allow error reporting
    canReport: {
      type: Boolean,
      default: true
    },
    // Custom retry handler
    onRetry: {
      type: Function,
      default: null
    },
    // Component name for error tracking
    componentName: {
      type: String,
      default: 'Unknown'
    }
  },
  emits: ['error', 'retry', 'report'],
  setup(props, { emit }) {
    const router = useRouter()
    const instance = getCurrentInstance()
    
    const hasError = ref(false)
    const errorInfo = ref({})
    const detailsOpen = ref([])
    const retryCount = ref(0)
    const maxRetries = 3
    
    const errorMessage = computed(() => {
      if (errorInfo.value.userMessage) {
        return errorInfo.value.userMessage
      }
      return props.fallbackMessage
    })
    
    // Error capture handler
    onErrorCaptured((error, instance, info) => {
      console.error('Error captured by ErrorBoundary:', error)
      
      // Store error information
      errorInfo.value = {
        type: error.name || 'Error',
        message: error.message || 'Unknown error',
        stack: error.stack,
        componentName: props.componentName,
        timestamp: new Date().toLocaleString(),
        userMessage: getUserFriendlyMessage(error),
        retryCount: retryCount.value
      }
      
      hasError.value = true
      
      // Emit error event
      emit('error', {
        error,
        errorInfo: errorInfo.value,
        instance,
        info
      })
      
      // Log error to monitoring service
      logErrorToService(error, errorInfo.value)
      
      // Return false to prevent the error from propagating further
      return false
    })
    
    // Get user-friendly error message
    const getUserFriendlyMessage = (error) => {
      if (error.name === 'ChunkLoadError') {
        return 'Failed to load application resources. Please refresh the page.'
      }
      
      if (error.name === 'NetworkError') {
        return 'Network connection error. Please check your internet connection.'
      }
      
      if (error.message?.includes('timeout')) {
        return 'The request timed out. Please try again.'
      }
      
      if (error.message?.includes('permission')) {
        return 'You don\'t have permission to perform this action.'
      }
      
      if (error.message?.includes('not found')) {
        return 'The requested resource was not found.'
      }
      
      return props.fallbackMessage
    }
    
    // Log error to monitoring service
    const logErrorToService = (error, info) => {
      if (window.errorTracker) {
        window.errorTracker.logError(error, info)
      }
      
      // In production, you would send this to your error tracking service
      if (import.meta.env.PROD) {
        // Example: Sentry, LogRocket, or custom service
        console.log('Logging error to service:', { error, info })
      }
    }
    
    // Retry the component
    const retry = () => {
      if (retryCount.value >= maxRetries) {
        ElMessage.warning('Maximum retry attempts reached. Please refresh the page.')
        return
      }
      
      retryCount.value++
      
      if (props.onRetry) {
        props.onRetry()
      }
      
      // Reset error state
      hasError.value = false
      errorInfo.value = {}
      
      emit('retry', {
        attempt: retryCount.value,
        maxRetries
      })
      
      // Force component re-render
      instance?.proxy?.$forceUpdate()
    }
    
    // Navigate to home page
    const goHome = () => {
      router.push('/')
    }
    
    // Report error to support
    const reportError = async () => {
      try {
        await ElMessageBox.prompt(
          'Please describe what you were doing when this error occurred:',
          'Report Error',
          {
            confirmButtonText: 'Send Report',
            cancelButtonText: 'Cancel',
            inputType: 'textarea',
            inputPlaceholder: 'Describe the steps that led to this error...'
          }
        )
        
        // In production, send report to support system
        ElMessage.success('Error report sent successfully. We\'ll look into it.')
        
        emit('report', {
          errorInfo: errorInfo.value,
          userDescription: 'User description would be here'
        })
      } catch (error) {
        // User cancelled or error occurred
        console.log('Error reporting cancelled')
      }
    }
    
    return {
      hasError,
      errorInfo,
      errorMessage,
      detailsOpen,
      retry,
      goHome,
      reportError
    }
  }
}
</script>

<style scoped>
.error-boundary {
  height: 100%;
  width: 100%;
}

.error-boundary-content {
  height: 100%;
  width: 100%;
}

.error-boundary-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 24px;
  background: var(--el-bg-color-page);
}

.error-container {
  max-width: 600px;
  text-align: center;
  background: var(--el-bg-color);
  padding: 40px;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.error-icon {
  margin-bottom: 24px;
}

.error-content {
  width: 100%;
}

.error-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 12px 0;
}

.error-message {
  font-size: 16px;
  color: var(--el-text-color-regular);
  margin: 0 0 24px 0;
  line-height: 1.5;
}

.error-details {
  margin: 24px 0;
  text-align: left;
}

.details-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.error-info {
  padding: 16px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  font-size: 14px;
}

.info-item {
  margin-bottom: 12px;
  color: var(--el-text-color-regular);
}

.info-item:last-child {
  margin-bottom: 0;
}

.info-item strong {
  color: var(--el-text-color-primary);
  margin-right: 8px;
}

.stack-trace {
  background: var(--el-fill-color-dark);
  padding: 12px;
  border-radius: 4px;
  font-size: 11px;
  color: var(--el-text-color-regular);
  overflow-x: auto;
  max-height: 200px;
  overflow-y: auto;
  margin-top: 8px;
  font-family: 'Courier New', monospace;
  white-space: pre-wrap;
  word-break: break-all;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.error-actions .el-button {
  min-width: 120px;
}

/* Dark mode adjustments */
.dark .error-container {
  background: var(--el-bg-color-overlay);
}

.dark .stack-trace {
  background: var(--el-fill-color);
  color: var(--el-text-color-primary);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .error-boundary-fallback {
    padding: 16px;
    min-height: 300px;
  }
  
  .error-container {
    padding: 24px 16px;
  }
  
  .error-title {
    font-size: 20px;
  }
  
  .error-message {
    font-size: 14px;
  }
  
  .error-actions {
    flex-direction: column;
    align-items: center;
  }
  
  .error-actions .el-button {
    width: 100%;
    min-width: unset;
  }
  
  .stack-trace {
    font-size: 10px;
    max-height: 150px;
  }
}
</style>