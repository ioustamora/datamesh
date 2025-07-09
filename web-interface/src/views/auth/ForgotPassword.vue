<template>
  <div class="auth-container">
    <div class="auth-card">
      <div class="auth-header">
        <h1 class="auth-title">Reset Password</h1>
        <p class="auth-subtitle">Enter your email address and we'll send you a link to reset your password</p>
      </div>
      
      <div v-if="!emailSent" class="reset-form">
        <el-form
          ref="resetForm"
          :model="form"
          :rules="rules"
          label-position="top"
          @submit.prevent="handleSubmit"
        >
          <el-form-item label="Email Address" prop="email">
            <el-input
              v-model="form.email"
              type="email"
              placeholder="Enter your email address"
              size="large"
              :prefix-icon="Message"
              autofocus
            />
          </el-form-item>
          
          <el-form-item>
            <el-button
              type="primary"
              size="large"
              @click="handleSubmit"
              :loading="loading"
              style="width: 100%"
            >
              Send Reset Link
            </el-button>
          </el-form-item>
        </el-form>
      </div>
      
      <div v-else class="reset-success">
        <div class="success-icon">
          <el-icon class="icon">
            <CircleCheckFilled />
          </el-icon>
        </div>
        
        <h2 class="success-title">Check Your Email</h2>
        <p class="success-message">
          We've sent a password reset link to <strong>{{ form.email }}</strong>
        </p>
        
        <div class="success-instructions">
          <h3>What to do next:</h3>
          <ol>
            <li>Check your email inbox</li>
            <li>Click the reset link in the email</li>
            <li>Create a new password</li>
            <li>Sign in with your new password</li>
          </ol>
        </div>
        
        <div class="success-note">
          <p>
            <strong>Didn't receive the email?</strong><br>
            Check your spam folder or 
            <el-button type="text" @click="resendEmail" :loading="resendLoading">
              click here to resend
            </el-button>
          </p>
        </div>
        
        <div class="success-actions">
          <el-button type="primary" @click="goToLogin">
            Return to Login
          </el-button>
          <el-button @click="resetForm">
            Try Different Email
          </el-button>
        </div>
      </div>
      
      <div class="auth-footer">
        <p>Remember your password? 
          <router-link to="/login" class="auth-link">Sign in</router-link>
        </p>
        <p>Don't have an account? 
          <router-link to="/register" class="auth-link">Sign up</router-link>
        </p>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { ElMessage } from 'element-plus'
import { Message, CircleCheckFilled } from '@element-plus/icons-vue'

export default {
  name: 'ForgotPassword',
  components: {
    Message,
    CircleCheckFilled
  },
  setup() {
    const router = useRouter()
    const authStore = useAuthStore()
    
    const resetForm = ref()
    const loading = ref(false)
    const resendLoading = ref(false)
    const emailSent = ref(false)
    
    const form = reactive({
      email: ''
    })
    
    const rules = {
      email: [
        { required: true, message: 'Please enter your email address', trigger: 'blur' },
        { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
      ]
    }
    
    const handleSubmit = async () => {
      try {
        const valid = await resetForm.value.validate()
        if (!valid) return
        
        loading.value = true
        
        await authStore.requestPasswordReset(form.email)
        
        emailSent.value = true
        ElMessage.success('Password reset link sent successfully!')
        
      } catch (error) {
        ElMessage.error(error.message || 'Failed to send reset link')
      } finally {
        loading.value = false
      }
    }
    
    const resendEmail = async () => {
      try {
        resendLoading.value = true
        
        await authStore.requestPasswordReset(form.email)
        
        ElMessage.success('Reset link sent again!')
        
      } catch (error) {
        ElMessage.error(error.message || 'Failed to resend reset link')
      } finally {
        resendLoading.value = false
      }
    }
    
    const goToLogin = () => {
      router.push('/login')
    }
    
    const resetForm = () => {
      emailSent.value = false
      form.email = ''
    }
    
    return {
      resetForm,
      form,
      rules,
      loading,
      resendLoading,
      emailSent,
      handleSubmit,
      resendEmail,
      goToLogin,
      resetForm
    }
  }
}
</script>

<style scoped>
.auth-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.auth-card {
  background: var(--el-bg-color);
  border-radius: 12px;
  padding: 40px;
  width: 100%;
  max-width: 500px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
}

.auth-header {
  text-align: center;
  margin-bottom: 30px;
}

.auth-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px 0;
}

.auth-subtitle {
  color: var(--el-text-color-regular);
  margin: 0;
  font-size: 16px;
  line-height: 1.5;
}

.reset-success {
  text-align: center;
}

.success-icon {
  margin-bottom: 20px;
}

.success-icon .icon {
  font-size: 64px;
  color: var(--el-color-success);
}

.success-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 16px 0;
}

.success-message {
  color: var(--el-text-color-regular);
  margin: 0 0 24px 0;
  font-size: 16px;
  line-height: 1.5;
}

.success-instructions {
  background: var(--el-fill-color-light);
  padding: 20px;
  border-radius: 8px;
  margin: 24px 0;
  text-align: left;
}

.success-instructions h3 {
  color: var(--el-text-color-primary);
  margin: 0 0 12px 0;
  font-size: 16px;
}

.success-instructions ol {
  margin: 0;
  padding-left: 20px;
  color: var(--el-text-color-regular);
}

.success-instructions li {
  margin-bottom: 8px;
  line-height: 1.4;
}

.success-note {
  background: var(--el-fill-color-lighter);
  padding: 16px;
  border-radius: 8px;
  margin: 24px 0;
  text-align: left;
}

.success-note p {
  margin: 0;
  color: var(--el-text-color-regular);
  font-size: 14px;
  line-height: 1.5;
}

.success-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.auth-footer {
  text-align: center;
  margin-top: 24px;
  color: var(--el-text-color-regular);
}

.auth-footer p {
  margin: 8px 0;
}

.auth-link {
  color: var(--el-color-primary);
  text-decoration: none;
  font-weight: 500;
}

.auth-link:hover {
  text-decoration: underline;
}

@media (max-width: 768px) {
  .auth-card {
    padding: 30px 20px;
  }
  
  .auth-title {
    font-size: 24px;
  }
  
  .auth-subtitle {
    font-size: 14px;
  }
  
  .success-title {
    font-size: 20px;
  }
  
  .success-message {
    font-size: 14px;
  }
  
  .success-actions {
    flex-direction: column;
  }
  
  .success-icon .icon {
    font-size: 48px;
  }
}
</style>