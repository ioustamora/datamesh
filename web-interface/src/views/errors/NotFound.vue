<template>
  <div class="error-page">
    <div class="error-container">
      <div class="error-illustration">
        <el-icon class="error-icon">
          <WarnTriangleFilled />
        </el-icon>
      </div>
      
      <div class="error-content">
        <h1 class="error-title">
          404
        </h1>
        <h2 class="error-subtitle">
          Page Not Found
        </h2>
        <p class="error-message">
          The page you're looking for doesn't exist or has been moved.
        </p>
        
        <div class="error-actions">
          <el-button
            type="primary"
            @click="goHome"
          >
            <el-icon><House /></el-icon>
            Go to Dashboard
          </el-button>
          <el-button @click="goBack">
            <el-icon><ArrowLeft /></el-icon>
            Go Back
          </el-button>
        </div>
        
        <div class="error-suggestions">
          <h3>Suggestions:</h3>
          <ul>
            <li>Check the URL for typos</li>
            <li>Try using the search function</li>
            <li>Navigate from the main menu</li>
            <li>Contact support if the problem persists</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { useRouter } from 'vue-router'
import { WarnTriangleFilled, House, ArrowLeft } from '@element-plus/icons-vue'

export default {
  name: 'NotFound',
  components: {
    WarnTriangleFilled,
    House,
    ArrowLeft
  },
  setup() {
    const router = useRouter()
    
    const goHome = () => {
      router.push('/')
    }
    
    const goBack = () => {
      router.go(-1)
    }
    
    return {
      goHome,
      goBack
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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
  color: var(--el-color-warning);
  opacity: 0.8;
}

.error-title {
  font-size: 72px;
  font-weight: 700;
  color: var(--el-color-primary);
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
}

.error-suggestions {
  text-align: left;
  background: var(--el-fill-color-light);
  padding: 20px;
  border-radius: 8px;
  margin-top: 20px;
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