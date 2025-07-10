<template>
  <div class="error-page">
    <div class="error-container">
      <div class="error-illustration">
        <div class="lock-icon">
          <el-icon
            size="80"
            color="#F56C6C"
          >
            <Lock />
          </el-icon>
        </div>
        <div class="error-code">
          403
        </div>
      </div>
      
      <div class="error-content">
        <h1 class="error-title">
          Access Forbidden
        </h1>
        <p class="error-message">
          You don't have permission to access this resource. This could be because:
        </p>
        
        <ul class="error-reasons">
          <li>Your account doesn't have the required permissions</li>
          <li>You need to log in with a different account</li>
          <li>The resource requires elevated privileges</li>
          <li>Your session may have expired</li>
        </ul>
        
        <div class="error-actions">
          <el-button
            type="primary"
            size="large"
            @click="goBack"
          >
            <el-icon><ArrowLeft /></el-icon>
            Go Back
          </el-button>
          
          <el-button
            size="large"
            @click="goHome"
          >
            <el-icon><House /></el-icon>
            Dashboard
          </el-button>
          
          <el-button
            size="large"
            @click="reLogin"
          >
            <el-icon><User /></el-icon>
            Re-login
          </el-button>
        </div>
        
        <div class="error-help">
          <p>
            If you believe this is an error, please 
            <el-link
              type="primary"
              @click="contactSupport"
            >
              contact support
            </el-link> 
            or check your account permissions.
          </p>
        </div>
      </div>
    </div>
    
    <!-- Support Contact Dialog -->
    <el-dialog
      v-model="showSupportDialog"
      title="Contact Support"
      width="500px"
      :show-close="true"
    >
      <el-form
        :model="supportForm"
        label-position="top"
      >
        <el-form-item label="Email">
          <el-input
            v-model="supportForm.email"
            type="email"
            placeholder="Your email address"
            :readonly="!!currentUser?.email"
          />
        </el-form-item>
        
        <el-form-item label="Subject">
          <el-input
            v-model="supportForm.subject"
            placeholder="403 Access Forbidden - Need Help"
          />
        </el-form-item>
        
        <el-form-item label="Description">
          <el-input
            v-model="supportForm.description"
            type="textarea"
            rows="4"
            placeholder="Please describe what you were trying to do when you encountered this error..."
          />
        </el-form-item>
        
        <el-form-item label="Current Page">
          <el-input
            v-model="supportForm.currentPage"
            readonly
            placeholder="Current page URL"
          />
        </el-form-item>
      </el-form>
      
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showSupportDialog = false">
            Cancel
          </el-button>
          <el-button
            type="primary"
            :loading="submittingSupport"
            @click="submitSupportRequest"
          >
            Send Request
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Lock, ArrowLeft, House, User } from '@element-plus/icons-vue'

export default {
  name: 'Forbidden',
  components: {
    Lock,
    ArrowLeft,
    House,
    User
  },
  setup() {
    const router = useRouter()
    const route = useRoute()
    const authStore = useAuthStore()
    
    const showSupportDialog = ref(false)
    const submittingSupport = ref(false)
    
    const supportForm = reactive({
      email: '',
      subject: '403 Access Forbidden - Need Help',
      description: '',
      currentPage: ''
    })
    
    const currentUser = computed(() => authStore.currentUser)
    
    // Initialize form data
    onMounted(() => {
      if (currentUser.value?.email) {
        supportForm.email = currentUser.value.email
      }
      supportForm.currentPage = window.location.href
    })
    
    const goBack = () => {
      // Use browser history if available
      if (window.history.length > 1) {
        router.go(-1)
      } else {
        router.push('/')
      }
    }
    
    const goHome = () => {
      router.push('/')
    }
    
    const reLogin = async () => {
      try {
        await ElMessageBox.confirm(
          'You will be logged out and redirected to the login page.',
          'Re-login Required',
          {
            confirmButtonText: 'Continue',
            cancelButtonText: 'Cancel',
            type: 'warning'
          }
        )
        
        await authStore.logout()
        router.push('/auth/login')
      } catch (error) {
        // User cancelled
      }
    }
    
    const contactSupport = () => {
      showSupportDialog.value = true
    }
    
    const submitSupportRequest = async () => {
      if (!supportForm.email || !supportForm.description) {
        ElMessage.error('Please fill in all required fields')
        return
      }
      
      submittingSupport.value = true
      
      try {
        // In a real application, this would send to your support system
        await new Promise(resolve => setTimeout(resolve, 1000)) // Simulate API call
        
        ElMessage.success('Support request sent successfully. We\'ll get back to you soon.')
        showSupportDialog.value = false
        
        // Reset form
        supportForm.description = ''
        supportForm.subject = '403 Access Forbidden - Need Help'
        
      } catch (error) {
        ElMessage.error('Failed to send support request. Please try again.')
      } finally {
        submittingSupport.value = false
      }
    }
    
    return {
      showSupportDialog,
      submittingSupport,
      supportForm,
      currentUser,
      goBack,
      goHome,
      reLogin,
      contactSupport,
      submitSupportRequest
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
  max-width: 600px;
  width: 100%;
  text-align: center;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
}

.error-illustration {
  margin-bottom: 30px;
  position: relative;
}

.lock-icon {
  margin-bottom: 20px;
}

.error-code {
  font-size: 72px;
  font-weight: bold;
  color: var(--el-color-danger);
  margin: 0;
  line-height: 1;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
}

.error-content {
  text-align: left;
}

.error-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 16px 0;
  text-align: center;
}

.error-message {
  font-size: 16px;
  color: var(--el-text-color-regular);
  margin: 0 0 20px 0;
  line-height: 1.6;
}

.error-reasons {
  margin: 0 0 30px 0;
  padding-left: 20px;
  color: var(--el-text-color-regular);
}

.error-reasons li {
  margin-bottom: 8px;
  line-height: 1.5;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.error-help {
  text-align: center;
  padding-top: 20px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.error-help p {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* Dark mode adjustments */
.dark .error-container {
  background: var(--el-bg-color-overlay);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .error-page {
    padding: 16px;
  }
  
  .error-container {
    padding: 30px 20px;
  }
  
  .error-code {
    font-size: 56px;
  }
  
  .error-title {
    font-size: 24px;
  }
  
  .error-message {
    font-size: 14px;
  }
  
  .error-actions {
    flex-direction: column;
  }
  
  .error-actions .el-button {
    width: 100%;
  }
  
  .lock-icon {
    margin-bottom: 16px;
  }
}

@media (max-width: 480px) {
  .error-container {
    padding: 20px 16px;
  }
  
  .error-code {
    font-size: 48px;
  }
  
  .error-title {
    font-size: 20px;
  }
  
  .error-reasons {
    padding-left: 16px;
  }
}
</style>