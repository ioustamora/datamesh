<template>
  <div class="error-page">
    <div class="error-container">
      <div class="error-illustration">
        <el-icon class="error-icon">
          <CircleCloseFilled />
        </el-icon>
      </div>
      
      <div class="error-content">
        <h1 class="error-title">500</h1>
        <h2 class="error-subtitle">Server Error</h2>
        <p class="error-message">
          Something went wrong on our end. Our team has been notified and is working on a fix.
        </p>
        
        <div class="error-actions">
          <el-button type="primary" @click="retry">
            <el-icon><Refresh /></el-icon>
            Try Again
          </el-button>
          <el-button @click="goHome">
            <el-icon><House /></el-icon>
            Go to Dashboard
          </el-button>
          <el-button @click="reportIssue">
            <el-icon><Warning /></el-icon>
            Report Issue
          </el-button>
        </div>
        
        <div class="error-details" v-if="showDetails">
          <h3>Error Details:</h3>
          <div class="error-code">
            <strong>Error Code:</strong> {{ errorCode }}
          </div>
          <div class="error-timestamp">
            <strong>Timestamp:</strong> {{ errorTimestamp }}
          </div>
          <div class="error-id" v-if="errorId">
            <strong>Error ID:</strong> {{ errorId }}
          </div>
        </div>
        
        <div class="error-suggestions">
          <h3>What you can do:</h3>
          <ul>
            <li>Wait a few minutes and try again</li>
            <li>Check your internet connection</li>
            <li>Clear your browser cache and cookies</li>
            <li>Try a different browser</li>
            <li>Contact our support team if the problem persists</li>
          </ul>
        </div>
        
        <div class="error-toggle">
          <el-button 
            text 
            type="info" 
            @click="showDetails = !showDetails"
          >
            {{ showDetails ? 'Hide' : 'Show' }} Technical Details
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { 
  CircleCloseFilled, 
  Refresh, 
  House, 
  Warning 
} from '@element-plus/icons-vue'
import dayjs from 'dayjs'

export default {
  name: 'ServerError',
  components: {
    CircleCloseFilled,
    Refresh,
    House,
    Warning
  },
  setup() {
    const router = useRouter()
    const showDetails = ref(false)
    const errorCode = ref('INTERNAL_SERVER_ERROR')
    const errorTimestamp = ref('')
    const errorId = ref('')
    
    const retry = () => {
      // Reload the current page
      window.location.reload()
    }
    
    const goHome = () => {
      router.push('/')
    }
    
    const reportIssue = () => {
      // Create error report
      const errorReport = {
        code: errorCode.value,
        timestamp: errorTimestamp.value,
        id: errorId.value,
        url: window.location.href,
        userAgent: navigator.userAgent
      }
      
      // In a real app, this would send to an error reporting service
      console.log('Error Report:', errorReport)
      
      // Copy error details to clipboard
      navigator.clipboard.writeText(JSON.stringify(errorReport, null, 2))
      ElMessage.success('Error details copied to clipboard. Please send this to support.')
    }
    
    onMounted(() => {
      // Generate error details
      errorTimestamp.value = dayjs().format('YYYY-MM-DD HH:mm:ss')
      errorId.value = Math.random().toString(36).substr(2, 9).toUpperCase()
      
      // Report error to analytics (in a real app)
      console.log('Server error occurred:', {
        code: errorCode.value,
        timestamp: errorTimestamp.value,
        id: errorId.value
      })
    })
    
    return {
      showDetails,
      errorCode,
      errorTimestamp,
      errorId,
      retry,
      goHome,
      reportIssue
    }
  }
}
</script>

<style scoped>
.error-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
  padding: 20px;
}

.error-container {
  background: var(--el-bg-color);
  border-radius: 12px;
  padding: 40px;
  text-align: center;
  max-width: 600px;
  width: 100%;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
}

.error-illustration {
  margin-bottom: 30px;
}

.error-icon {
  font-size: 120px;
  color: var(--el-color-danger);
  opacity: 0.8;
}

.error-title {
  font-size: 72px;
  font-weight: 700;
  color: var(--el-color-danger);
  margin: 0 0 10px 0;
  line-height: 1;
}

.error-subtitle {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 20px 0;
}

.error-message {
  font-size: 16px;
  color: var(--el-text-color-regular);
  margin: 0 0 30px 0;
  line-height: 1.6;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.error-details {
  background: var(--el-fill-color-light);
  padding: 20px;
  border-radius: 8px;
  margin: 20px 0;
  text-align: left;
  font-family: monospace;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.error-details > div {
  margin-bottom: 8px;
}

.error-suggestions {
  text-align: left;
  background: var(--el-fill-color-lighter);
  padding: 20px;
  border-radius: 8px;
  margin: 20px 0;
}

.error-suggestions h3 {
  color: var(--el-text-color-primary);
  margin: 0 0 15px 0;
  font-size: 16px;
}

.error-suggestions ul {
  margin: 0;
  padding-left: 20px;
  color: var(--el-text-color-regular);
}

.error-suggestions li {
  margin-bottom: 8px;
}

.error-toggle {
  margin-top: 20px;
}

@media (max-width: 768px) {
  .error-container {
    padding: 30px 20px;
  }
  
  .error-title {
    font-size: 56px;
  }
  
  .error-subtitle {
    font-size: 24px;
  }
  
  .error-actions {
    flex-direction: column;
    align-items: center;
  }
  
  .error-icon {
    font-size: 80px;
  }
}
</style>