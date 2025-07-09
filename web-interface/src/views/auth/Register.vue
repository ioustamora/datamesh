<template>
  <div class="auth-container">
    <div class="auth-card">
      <div class="auth-header">
        <h1 class="auth-title">Create Account</h1>
        <p class="auth-subtitle">Join the DataMesh distributed storage network</p>
      </div>
      
      <el-form
        ref="registerForm"
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
          />
        </el-form-item>
        
        <el-form-item label="Password" prop="password">
          <el-input
            v-model="form.password"
            type="password"
            placeholder="Create a password"
            size="large"
            :prefix-icon="Lock"
            show-password
          />
          <div class="password-strength" v-if="form.password">
            <div class="strength-bar">
              <div 
                class="strength-fill" 
                :style="{ width: passwordStrength.percentage + '%' }"
                :class="passwordStrength.class"
              ></div>
            </div>
            <div class="strength-info">
              <div class="strength-text">{{ passwordStrength.text }}</div>
              <div class="strength-details">
                <div class="entropy">Entropy: {{ passwordStrength.entropy }} bits</div>
                <div class="crack-time">Crack time: {{ passwordStrength.crackTime }}</div>
              </div>
            </div>
            
            <!-- Password Requirements -->
            <div class="password-requirements" v-if="passwordStrength.suggestions.length > 0">
              <div class="requirements-title">Suggestions:</div>
              <ul class="requirements-list">
                <li v-for="suggestion in passwordStrength.suggestions" :key="suggestion">
                  {{ suggestion }}
                </li>
              </ul>
            </div>
            
            <!-- Validation Errors -->
            <div class="password-errors" v-if="passwordStrength.errors.length > 0">
              <div class="errors-title">Issues:</div>
              <ul class="errors-list">
                <li v-for="error in passwordStrength.errors" :key="error">
                  {{ error }}
                </li>
              </ul>
            </div>
          </div>
        </el-form-item>
        
        <el-form-item label="Confirm Password" prop="confirmPassword">
          <el-input
            v-model="form.confirmPassword"
            type="password"
            placeholder="Confirm your password"
            size="large"
            :prefix-icon="Lock"
            show-password
          />
        </el-form-item>
        
        <el-form-item label="Account Type" prop="accountType">
          <el-select v-model="form.accountType" placeholder="Select account type" size="large">
            <el-option
              v-for="type in accountTypes"
              :key="type.value"
              :label="type.label"
              :value="type.value"
            >
              <div class="account-type-option">
                <div class="option-header">
                  <span class="option-name">{{ type.label }}</span>
                  <span class="option-price">{{ type.price }}</span>
                </div>
                <div class="option-description">{{ type.description }}</div>
              </div>
            </el-option>
          </el-select>
        </el-form-item>
        
        <div class="account-type-details" v-if="selectedAccountType">
          <h4>{{ selectedAccountType.label }} Features:</h4>
          <ul>
            <li v-for="feature in selectedAccountType.features" :key="feature">
              {{ feature }}
            </li>
          </ul>
        </div>
        
        <el-form-item prop="agreeToTerms">
          <el-checkbox v-model="form.agreeToTerms" size="large">
            I agree to the 
            <el-link type="primary" @click="showTerms">Terms of Service</el-link> 
            and 
            <el-link type="primary" @click="showPrivacy">Privacy Policy</el-link>
          </el-checkbox>
        </el-form-item>
        
        <el-form-item prop="marketingConsent">
          <el-checkbox v-model="form.marketingConsent" size="large">
            I would like to receive updates and marketing communications
          </el-checkbox>
        </el-form-item>
        
        <el-form-item>
          <el-button
            type="primary"
            size="large"
            @click="handleSubmit"
            :loading="loading"
            style="width: 100%"
          >
            Create Account
          </el-button>
        </el-form-item>
      </el-form>
      
      <div class="auth-footer">
        <p>Already have an account? 
          <router-link to="/login" class="auth-link">Sign in</router-link>
        </p>
      </div>
    </div>
    
    <!-- Terms of Service Dialog -->
    <el-dialog v-model="showTermsDialog" title="Terms of Service" width="70%">
      <div class="terms-content">
        <h3>1. Acceptance of Terms</h3>
        <p>By creating an account with DataMesh, you agree to be bound by these Terms of Service and our Privacy Policy.</p>
        
        <h3>2. Service Description</h3>
        <p>DataMesh provides distributed storage services through a network of peer-to-peer nodes. We offer various service tiers with different storage quotas and bandwidth limits.</p>
        
        <h3>3. User Responsibilities</h3>
        <ul>
          <li>You are responsible for maintaining the security of your account credentials</li>
          <li>You must not upload illegal, harmful, or inappropriate content</li>
          <li>You must comply with all applicable laws and regulations</li>
          <li>You must respect the storage quotas and bandwidth limits of your account tier</li>
        </ul>
        
        <h3>4. Service Availability</h3>
        <p>While we strive for high availability, DataMesh is provided "as is" without guarantees of uptime or data availability, except as specified in Enterprise SLA agreements.</p>
        
        <h3>5. Data Privacy</h3>
        <p>Your data is encrypted end-to-end and stored across multiple nodes in the network. We cannot access your encrypted data without your keys.</p>
        
        <h3>6. Termination</h3>
        <p>You may terminate your account at any time. We may terminate accounts that violate these terms or applicable laws.</p>
      </div>
      
      <template #footer>
        <el-button @click="showTermsDialog = false">Close</el-button>
      </template>
    </el-dialog>
    
    <!-- Privacy Policy Dialog -->
    <el-dialog v-model="showPrivacyDialog" title="Privacy Policy" width="70%">
      <div class="privacy-content">
        <h3>1. Information We Collect</h3>
        <p>We collect minimal information necessary to provide our services:</p>
        <ul>
          <li>Email address for account identification</li>
          <li>Usage statistics (storage and bandwidth consumption)</li>
          <li>Technical logs for system operation and security</li>
        </ul>
        
        <h3>2. How We Use Your Information</h3>
        <ul>
          <li>To provide and maintain our storage services</li>
          <li>To communicate with you about your account</li>
          <li>To improve our services and user experience</li>
          <li>To comply with legal obligations</li>
        </ul>
        
        <h3>3. Data Security</h3>
        <p>All user data is encrypted end-to-end using industry-standard encryption. We cannot access the content of your files.</p>
        
        <h3>4. Data Sharing</h3>
        <p>We do not sell, trade, or rent your personal information to third parties. We may share information only when required by law.</p>
        
        <h3>5. Your Rights</h3>
        <p>You have the right to:</p>
        <ul>
          <li>Access your personal data</li>
          <li>Correct inaccurate data</li>
          <li>Delete your account and data</li>
          <li>Object to processing of your data</li>
        </ul>
        
        <h3>6. Contact Information</h3>
        <p>For privacy-related questions, contact us at privacy@datamesh.io</p>
      </div>
      
      <template #footer>
        <el-button @click="showPrivacyDialog = false">Close</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, reactive, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { ElMessage } from 'element-plus'
import { Message, Lock } from '@element-plus/icons-vue'
import { passwordValidator } from '@/utils/passwordValidator'

export default {
  name: 'Register',
  components: {
    Message,
    Lock
  },
  setup() {
    const router = useRouter()
    const authStore = useAuthStore()
    
    const registerForm = ref()
    const loading = ref(false)
    const showTermsDialog = ref(false)
    const showPrivacyDialog = ref(false)
    
    const form = reactive({
      email: '',
      password: '',
      confirmPassword: '',
      accountType: 'free',
      agreeToTerms: false,
      marketingConsent: false
    })
    
    const accountTypes = [
      {
        value: 'free',
        label: 'Free',
        price: 'Free',
        description: 'Basic storage with 5GB space and 50GB/month bandwidth',
        features: [
          '5GB storage space',
          '50GB/month bandwidth',
          '100 API calls/hour',
          'Community support',
          'Basic encryption'
        ]
      },
      {
        value: 'premium',
        label: 'Premium',
        price: '$9.99/month',
        description: 'Enhanced storage with 100GB space and 1TB/month bandwidth',
        features: [
          '100GB storage space',
          '1TB/month bandwidth',
          '10,000 API calls/hour',
          'Priority support',
          'Advanced encryption',
          'File versioning'
        ]
      },
      {
        value: 'enterprise',
        label: 'Enterprise',
        price: 'Custom pricing',
        description: 'Unlimited storage and bandwidth with SLA guarantees',
        features: [
          'Unlimited storage',
          'Unlimited bandwidth',
          'Unlimited API calls',
          '24/7 dedicated support',
          'Custom encryption',
          'SLA guarantees',
          'Admin dashboard',
          'Audit logs'
        ]
      }
    ]
    
    const selectedAccountType = computed(() => {
      return accountTypes.find(type => type.value === form.accountType)
    })
    
    const passwordValidation = ref(null)
    
    const passwordStrength = computed(() => {
      const password = form.password
      if (!password) return { percentage: 0, class: '', text: '', entropy: 0, crackTime: '', suggestions: [], errors: [] }
      
      if (!passwordValidation.value) {
        return { percentage: 0, class: '', text: '', entropy: 0, crackTime: '', suggestions: [], errors: [] }
      }
      
      const validation = passwordValidation.value
      const strengthNames = ['Very Weak', 'Weak', 'Fair', 'Good', 'Strong', 'Very Strong']
      const strengthClasses = ['very-weak', 'weak', 'fair', 'good', 'strong', 'very-strong']
      
      const percentage = Math.min(100, (validation.strength / 5) * 100)
      
      return {
        percentage,
        class: strengthClasses[validation.strength] || 'very-weak',
        text: strengthNames[validation.strength] || 'Very Weak',
        entropy: validation.entropy,
        crackTime: passwordValidator.getCrackTimeDescription(validation.estimatedCrackTime),
        suggestions: validation.suggestions || [],
        errors: validation.errors || []
      }
    })
    
    // Watch password changes for validation
    watch(() => form.password, (newPassword) => {
      if (newPassword) {
        const userInfo = {
          email: form.email,
          username: form.email?.split('@')[0]
        }
        passwordValidation.value = passwordValidator.validate(newPassword, userInfo)
      } else {
        passwordValidation.value = null
      }
    })
    
    const rules = {
      email: [
        { required: true, message: 'Please enter your email address', trigger: 'blur' },
        { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
      ],
      password: [
        { required: true, message: 'Please create a password', trigger: 'blur' },
        {
          validator: (rule, value, callback) => {
            if (!value) {
              callback(new Error('Please create a password'))
              return
            }
            
            const userInfo = {
              email: form.email,
              username: form.email?.split('@')[0]
            }
            
            const validation = passwordValidator.validate(value, userInfo)
            
            if (!validation.isValid) {
              callback(new Error(validation.errors[0] || 'Password does not meet requirements'))
              return
            }
            
            if (validation.strength < 2) { // Require at least 'Fair' strength
              callback(new Error('Password is too weak. Please create a stronger password.'))
              return
            }
            
            callback()
          },
          trigger: 'blur'
        }
      ],
      confirmPassword: [
        { required: true, message: 'Please confirm your password', trigger: 'blur' },
        {
          validator: (rule, value, callback) => {
            if (value !== form.password) {
              callback(new Error('Passwords do not match'))
            } else {
              callback()
            }
          },
          trigger: 'blur'
        }
      ],
      accountType: [
        { required: true, message: 'Please select an account type', trigger: 'change' }
      ],
      agreeToTerms: [
        {
          validator: (rule, value, callback) => {
            if (!value) {
              callback(new Error('You must agree to the Terms of Service'))
            } else {
              callback()
            }
          },
          trigger: 'change'
        }
      ]
    }
    
    const handleSubmit = async () => {
      try {
        const valid = await registerForm.value.validate()
        if (!valid) return
        
        loading.value = true
        
        await authStore.register({
          email: form.email,
          password: form.password,
          accountType: form.accountType,
          agreeToTerms: form.agreeToTerms,
          marketingConsent: form.marketingConsent
        })
        
        ElMessage.success('Account created successfully! Please check your email for verification.')
        router.push('/login')
        
      } catch (error) {
        ElMessage.error(error.message || 'Registration failed')
      } finally {
        loading.value = false
      }
    }
    
    const showTerms = () => {
      showTermsDialog.value = true
    }
    
    const showPrivacy = () => {
      showPrivacyDialog.value = true
    }
    
    return {
      registerForm,
      form,
      rules,
      loading,
      showTermsDialog,
      showPrivacyDialog,
      accountTypes,
      selectedAccountType,
      passwordStrength,
      passwordValidation,
      handleSubmit,
      showTerms,
      showPrivacy
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
}

.password-strength {
  margin-top: 8px;
}

.strength-bar {
  height: 4px;
  background: var(--el-fill-color);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 4px;
}

.strength-fill {
  height: 100%;
  transition: all 0.3s ease;
}

.strength-fill.very-weak {
  background: #ff4757;
}

.strength-fill.weak {
  background: var(--el-color-danger);
}

.strength-fill.fair {
  background: var(--el-color-warning);
}

.strength-fill.good {
  background: var(--el-color-success);
}

.strength-fill.strong {
  background: var(--el-color-primary);
}

.strength-fill.very-strong {
  background: #5352ed;
}

.strength-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.strength-text {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-weight: 500;
}

.strength-details {
  display: flex;
  gap: 16px;
  font-size: 10px;
  color: var(--el-text-color-placeholder);
}

.password-requirements {
  margin-top: 8px;
  padding: 8px;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
  border-left: 3px solid var(--el-color-primary);
}

.requirements-title {
  font-size: 11px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.requirements-list {
  margin: 0;
  padding-left: 16px;
  font-size: 10px;
  color: var(--el-text-color-regular);
}

.requirements-list li {
  margin-bottom: 2px;
}

.password-errors {
  margin-top: 8px;
  padding: 8px;
  background: var(--el-color-danger-light-9);
  border-radius: 4px;
  border-left: 3px solid var(--el-color-danger);
}

.errors-title {
  font-size: 11px;
  font-weight: 500;
  color: var(--el-color-danger);
  margin-bottom: 4px;
}

.errors-list {
  margin: 0;
  padding-left: 16px;
  font-size: 10px;
  color: var(--el-color-danger-dark-2);
}

.errors-list li {
  margin-bottom: 2px;
}

.account-type-option {
  padding: 8px 0;
}

.option-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.option-name {
  font-weight: 500;
}

.option-price {
  font-size: 12px;
  color: var(--el-color-primary);
  font-weight: 500;
}

.option-description {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.4;
}

.account-type-details {
  background: var(--el-fill-color-light);
  padding: 16px;
  border-radius: 8px;
  margin: 16px 0;
}

.account-type-details h4 {
  margin: 0 0 12px 0;
  color: var(--el-text-color-primary);
}

.account-type-details ul {
  margin: 0;
  padding-left: 20px;
}

.account-type-details li {
  margin-bottom: 4px;
  color: var(--el-text-color-regular);
  font-size: 14px;
}

.auth-footer {
  text-align: center;
  margin-top: 24px;
  color: var(--el-text-color-regular);
}

.auth-link {
  color: var(--el-color-primary);
  text-decoration: none;
  font-weight: 500;
}

.auth-link:hover {
  text-decoration: underline;
}

.terms-content,
.privacy-content {
  max-height: 400px;
  overflow-y: auto;
  padding: 16px;
  line-height: 1.6;
}

.terms-content h3,
.privacy-content h3 {
  color: var(--el-text-color-primary);
  margin: 20px 0 12px 0;
}

.terms-content h3:first-child,
.privacy-content h3:first-child {
  margin-top: 0;
}

.terms-content p,
.privacy-content p {
  margin: 12px 0;
  color: var(--el-text-color-regular);
}

.terms-content ul,
.privacy-content ul {
  margin: 12px 0;
  padding-left: 20px;
}

.terms-content li,
.privacy-content li {
  margin-bottom: 8px;
  color: var(--el-text-color-regular);
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
  
  .strength-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .strength-details {
    gap: 8px;
  }
  
  .password-requirements,
  .password-errors {
    padding: 6px;
  }
  
  .requirements-list,
  .errors-list {
    padding-left: 12px;
  }
}
</style>