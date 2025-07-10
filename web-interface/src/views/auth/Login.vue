<template>
  <div class="login-form">
    <h2>Sign In</h2>
    <p>Access your DataMesh account</p>
    
    <el-form
      ref="loginFormRef"
      :model="loginForm"
      :rules="loginRules"
      @submit.prevent="handleLogin"
    >
      <el-form-item prop="email">
        <el-input
          v-model="loginForm.email"
          type="email"
          placeholder="Email address"
          size="large"
          :prefix-icon="Message"
          :disabled="loading"
        />
      </el-form-item>
      
      <el-form-item prop="password">
        <el-input
          v-model="loginForm.password"
          type="password"
          placeholder="Password"
          size="large"
          :prefix-icon="Lock"
          :disabled="loading"
          show-password
          @keyup.enter="handleLogin"
        />
      </el-form-item>
      
      <el-form-item>
        <div class="login-options">
          <el-checkbox
            v-model="loginForm.remember"
            :disabled="loading"
          >
            Remember me
          </el-checkbox>
          <router-link
            to="/auth/forgot-password"
            class="forgot-link"
          >
            Forgot password?
          </router-link>
        </div>
      </el-form-item>
      
      <el-form-item>
        <el-button
          type="primary"
          size="large"
          :loading="loading"
          class="login-button"
          @click="handleLogin"
        >
          Sign In
        </el-button>
      </el-form-item>
    </el-form>
    
    <div class="login-footer">
      <p>
        Don't have an account? <router-link to="/auth/register">
          Sign up
        </router-link>
      </p>
    </div>
    
    <!-- Demo credentials -->
    <div
      v-if="isDev"
      class="demo-credentials"
    >
      <el-divider>Demo Credentials</el-divider>
      <div class="demo-buttons">
        <el-button
          size="small"
          @click="setDemoCredentials('admin')"
        >
          Admin Demo
        </el-button>
        <el-button
          size="small"
          @click="setDemoCredentials('user')"
        >
          User Demo
        </el-button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../store/auth'
import { ElMessage } from 'element-plus'
import { Message, Lock } from '@element-plus/icons-vue'

export default {
  name: 'Login',
  setup() {
    const router = useRouter()
    const authStore = useAuthStore()
    
    const loginFormRef = ref()
    const loading = ref(false)
    const isDev = computed(() => import.meta.env.DEV)
    
    const loginForm = reactive({
      email: '',
      password: '',
      remember: false
    })
    
    const loginRules = {
      email: [
        { required: true, message: 'Please enter your email address', trigger: 'blur' },
        { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
      ],
      password: [
        { required: true, message: 'Please enter your password', trigger: 'blur' },
        { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
      ]
    }
    
    const handleLogin = async () => {
      if (!loginFormRef.value) return
      
      try {
        await loginFormRef.value.validate()
        loading.value = true
        
        await authStore.login({
          email: loginForm.email,
          password: loginForm.password,
          remember: loginForm.remember
        })
        
        ElMessage.success('Login successful!')
        router.push('/')
      } catch (error) {
        console.error('Login error:', error)
        ElMessage.error(error.message || 'Login failed. Please try again.')
      } finally {
        loading.value = false
      }
    }
    
    const setDemoCredentials = (type) => {
      if (type === 'admin') {
        loginForm.email = 'admin@datamesh.io'
        loginForm.password = 'admin123'
      } else {
        loginForm.email = 'user@datamesh.io'
        loginForm.password = 'user123'
      }
    }
    
    return {
      loginFormRef,
      loginForm,
      loginRules,
      loading,
      isDev,
      handleLogin,
      setDemoCredentials,
      Message,
      Lock
    }
  }
}
</script>

<style scoped>
.login-form {
  width: 100%;
}

.login-form h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  text-align: center;
}

.login-form p {
  margin: 0 0 32px 0;
  color: var(--el-text-color-secondary);
  text-align: center;
}

.login-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.forgot-link {
  color: var(--el-color-primary);
  text-decoration: none;
  font-size: 14px;
}

.forgot-link:hover {
  color: var(--el-color-primary-light-3);
}

.login-button {
  width: 100%;
  height: 48px;
  font-size: 16px;
  font-weight: 500;
}

.login-footer {
  text-align: center;
  margin-top: 24px;
}

.login-footer p {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.login-footer a {
  color: var(--el-color-primary);
  text-decoration: none;
  font-weight: 500;
}

.login-footer a:hover {
  color: var(--el-color-primary-light-3);
}

.demo-credentials {
  margin-top: 24px;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.demo-buttons {
  display: flex;
  gap: 8px;
  justify-content: center;
}

/* Focus styles */
.el-input:focus-within {
  box-shadow: 0 0 0 2px var(--el-color-primary-light-8);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .login-form h2 {
    font-size: 20px;
  }
  
  .login-options {
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  
  .demo-buttons {
    flex-direction: column;
  }
}
</style>