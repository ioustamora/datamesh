<template>
  <el-dialog
    v-model="visible"
    title="Set Up Storage Contribution"
    width="600px"
    :before-close="handleClose"
    append-to-body
  >
    <div class="contribution-setup">
      <!-- Step Progress -->
      <el-steps :active="currentStep" align-center class="setup-steps">
        <el-step title="Configure" description="Set up contribution parameters" />
        <el-step title="Verify" description="Confirm storage path" />
        <el-step title="Complete" description="Start contributing" />
      </el-steps>

      <!-- Step 1: Configuration -->
      <div v-if="currentStep === 0" class="setup-step">
        <h3>Configure Your Storage Contribution</h3>
        <p class="step-description">
          Choose the storage path and amount you want to contribute to the DataMesh network.
          Contributing storage helps the network and earns you additional storage quota.
        </p>

        <el-form ref="configForm" :model="contributionForm" :rules="configRules" label-width="140px">
          <el-form-item label="Storage Path" prop="storagePath">
            <el-input
              v-model="contributionForm.storagePath"
              placeholder="Enter the path to storage directory"
              :suffix-icon="FolderOpened"
            />
            <div class="form-hint">
              Choose a directory with sufficient free space. The path should be accessible and persistent.
            </div>
          </el-form-item>

          <el-form-item label="Contribution Size" prop="amount">
            <div class="amount-input">
              <el-input-number
                v-model="contributionAmount"
                :min="1"
                :max="maxContribution"
                :step="1"
                controls-position="right"
                style="width: 150px"
              />
              <el-select v-model="sizeUnit" style="width: 80px; margin-left: 8px">
                <el-option label="GB" value="GB" />
                <el-option label="TB" value="TB" />
              </el-select>
            </div>
            <div class="form-hint">
              Minimum: 1 GB, Maximum: {{ formatSize(getMaxContributionBytes()) }}
            </div>
          </el-form-item>

          <el-form-item label="Contribution Ratio">
            <div class="ratio-display">
              <div class="ratio-item">
                <span class="ratio-label">You contribute:</span>
                <span class="ratio-value contribute">{{ formatSize(contributionForm.amount) }}</span>
              </div>
              <el-icon class="ratio-arrow"><ArrowRight /></el-icon>
              <div class="ratio-item">
                <span class="ratio-label">You earn:</span>
                <span class="ratio-value earn">{{ formatSize(Math.floor(contributionForm.amount / 4)) }}</span>
              </div>
            </div>
            <div class="form-hint">
              DataMesh uses a 4:1 contribution ratio. For every 4 GB you contribute, you earn 1 GB of storage quota.
            </div>
          </el-form-item>
        </el-form>

        <div class="step-actions">
          <el-button @click="handleClose">Cancel</el-button>
          <el-button type="primary" @click="nextStep" :disabled="!isConfigValid">
            Next: Verify Path
          </el-button>
        </div>
      </div>

      <!-- Step 2: Verification -->
      <div v-if="currentStep === 1" class="setup-step">
        <h3>Verify Storage Path</h3>
        <p class="step-description">
          We'll verify that the specified path is accessible and has sufficient free space.
        </p>

        <div class="verification-status">
          <div class="verification-item" :class="pathChecks.exists ? 'success' : 'pending'">
            <el-icon>
              <Check v-if="pathChecks.exists" />
              <Loading v-else-if="verifying" />
              <Close v-else />
            </el-icon>
            <span>Path exists and is accessible</span>
          </div>

          <div class="verification-item" :class="pathChecks.writable ? 'success' : 'pending'">
            <el-icon>
              <Check v-if="pathChecks.writable" />
              <Loading v-else-if="verifying" />
              <Close v-else />
            </el-icon>
            <span>Path is writable</span>
          </div>

          <div class="verification-item" :class="pathChecks.space ? 'success' : 'pending'">
            <el-icon>
              <Check v-if="pathChecks.space" />
              <Loading v-else-if="verifying" />
              <Close v-else />
            </el-icon>
            <span>Sufficient free space available ({{ formatSize(availableSpace) }})</span>
          </div>

          <div class="verification-item" :class="pathChecks.permissions ? 'success' : 'pending'">
            <el-icon>
              <Check v-if="pathChecks.permissions" />
              <Loading v-else-if="verifying" />
              <Close v-else />
            </el-icon>
            <span>Proper permissions configured</span>
          </div>
        </div>

        <div v-if="verificationError" class="verification-error">
          <el-alert
            :title="verificationError"
            type="error"
            show-icon
            :closable="false"
          />
        </div>

        <div class="step-actions">
          <el-button @click="prevStep">Back</el-button>
          <el-button @click="verifyPath" :loading="verifying" type="primary">
            {{ verifying ? 'Verifying...' : 'Verify Path' }}
          </el-button>
          <el-button 
            type="primary" 
            @click="nextStep" 
            :disabled="!isVerificationComplete"
            v-if="isVerificationComplete"
          >
            Next: Complete Setup
          </el-button>
        </div>
      </div>

      <!-- Step 3: Complete -->
      <div v-if="currentStep === 2" class="setup-step">
        <h3>Complete Setup</h3>
        <p class="step-description">
          Review your contribution configuration and start contributing to the DataMesh network.
        </p>

        <div class="contribution-summary">
          <div class="summary-item">
            <span class="summary-label">Storage Path:</span>
            <span class="summary-value">{{ contributionForm.storagePath }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Contribution Amount:</span>
            <span class="summary-value">{{ formatSize(contributionForm.amount) }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Storage Quota Earned:</span>
            <span class="summary-value earned">{{ formatSize(Math.floor(contributionForm.amount / 4)) }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Verification Schedule:</span>
            <span class="summary-value">Daily random challenges</span>
          </div>
        </div>

        <div class="contribution-agreement">
          <el-checkbox v-model="agreedToTerms">
            I agree to the 
            <el-link type="primary" @click="showTerms = true">Storage Contribution Terms</el-link>
            and understand the verification requirements
          </el-checkbox>
        </div>

        <div class="step-actions">
          <el-button @click="prevStep">Back</el-button>
          <el-button 
            type="primary" 
            @click="startContribution" 
            :loading="starting"
            :disabled="!agreedToTerms"
          >
            {{ starting ? 'Starting...' : 'Start Contributing' }}
          </el-button>
        </div>
      </div>
    </div>

    <!-- Terms Dialog -->
    <el-dialog
      v-model="showTerms"
      title="Storage Contribution Terms"
      width="500px"
      append-to-body
    >
      <div class="terms-content">
        <h4>DataMesh Storage Contribution Agreement</h4>
        <ol>
          <li>You voluntarily contribute storage space to the DataMesh network</li>
          <li>Contributed storage must remain available and accessible</li>
          <li>The system will periodically verify your contribution through challenges</li>
          <li>Failing verification challenges may result in reputation penalties</li>
          <li>You can stop contributing at any time</li>
          <li>Storage quota earned is proportional to verified contribution (4:1 ratio)</li>
          <li>DataMesh does not guarantee uptime or availability</li>
        </ol>
      </div>
      <template #footer>
        <el-button @click="showTerms = false">Close</el-button>
      </template>
    </el-dialog>
  </el-dialog>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useEconomyStore } from '../../store/economy'
import { ElMessage } from 'element-plus'
import {
  FolderOpened, ArrowRight, Check, Loading, Close
} from '@element-plus/icons-vue'

const emit = defineEmits(['update:modelValue', 'success'])
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  }
})

// Store
const economyStore = useEconomyStore()

// Local state
const visible = ref(false)
const currentStep = ref(0)
const contributionAmount = ref(4)
const sizeUnit = ref('GB')
const agreedToTerms = ref(false)
const verifying = ref(false)
const starting = ref(false)
const verificationError = ref('')
const availableSpace = ref(0)
const showTerms = ref(false)

// Form data
const contributionForm = ref({
  storagePath: '',
  amount: 0
})

// Validation rules
const configRules = {
  storagePath: [
    { required: true, message: 'Please enter a storage path', trigger: 'blur' },
    { min: 3, message: 'Path must be at least 3 characters', trigger: 'blur' }
  ],
  amount: [
    { required: true, message: 'Please specify contribution amount', trigger: 'blur' },
    { type: 'number', min: 1073741824, message: 'Minimum contribution is 1 GB', trigger: 'blur' }
  ]
}

// Path verification status
const pathChecks = ref({
  exists: false,
  writable: false,
  space: false,
  permissions: false
})

// Computed properties
const maxContribution = computed(() => {
  return sizeUnit.value === 'TB' ? 10 : 1000 // 10 TB or 1000 GB max
})

const isConfigValid = computed(() => {
  return contributionForm.value.storagePath.length >= 3 && 
         contributionForm.value.amount >= 1073741824 // 1 GB minimum
})

const isVerificationComplete = computed(() => {
  return Object.values(pathChecks.value).every(check => check === true)
})

// Watch for changes
watch(() => props.modelValue, (newVal) => {
  visible.value = newVal
  if (newVal) {
    resetForm()
  }
})

watch(visible, (newVal) => {
  emit('update:modelValue', newVal)
})

watch([contributionAmount, sizeUnit], () => {
  updateContributionAmount()
})

// Methods
const updateContributionAmount = () => {
  const multiplier = sizeUnit.value === 'TB' ? 1099511627776 : 1073741824 // TB or GB in bytes
  contributionForm.value.amount = contributionAmount.value * multiplier
}

const getMaxContributionBytes = () => {
  return maxContribution.value * (sizeUnit.value === 'TB' ? 1099511627776 : 1073741824)
}

const formatSize = (bytes) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const nextStep = () => {
  if (currentStep.value < 2) {
    currentStep.value++
    
    if (currentStep.value === 1) {
      // Auto-verify when entering verification step
      setTimeout(() => {
        verifyPath()
      }, 500)
    }
  }
}

const prevStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

const verifyPath = async () => {
  verifying.value = true
  verificationError.value = ''
  
  // Reset checks
  Object.keys(pathChecks.value).forEach(key => {
    pathChecks.value[key] = false
  })

  try {
    // Simulate path verification (in real implementation, this would call an API)
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // Simulate verification results
    const isValidPath = contributionForm.value.storagePath.length > 0
    const hasSpace = contributionForm.value.amount <= 500 * 1073741824 // Assume 500GB available
    
    if (isValidPath) {
      pathChecks.value.exists = true
      
      setTimeout(() => {
        pathChecks.value.writable = true
        
        setTimeout(() => {
          if (hasSpace) {
            pathChecks.value.space = true
            availableSpace.value = 500 * 1073741824 // 500 GB available
            
            setTimeout(() => {
              pathChecks.value.permissions = true
            }, 500)
          } else {
            verificationError.value = 'Insufficient space available at the specified path'
          }
        }, 500)
      }, 500)
    } else {
      verificationError.value = 'Path does not exist or is not accessible'
    }
  } catch (error) {
    verificationError.value = 'Failed to verify path: ' + error.message
  } finally {
    verifying.value = false
  }
}

const startContribution = async () => {
  starting.value = true
  
  try {
    await economyStore.startContribution({
      storage_path: contributionForm.value.storagePath,
      amount: contributionForm.value.amount
    })
    
    ElMessage.success('Storage contribution started successfully!')
    emit('success')
    handleClose()
  } catch (error) {
    ElMessage.error('Failed to start contribution: ' + error.message)
  } finally {
    starting.value = false
  }
}

const resetForm = () => {
  currentStep.value = 0
  contributionAmount.value = 4
  sizeUnit.value = 'GB'
  agreedToTerms.value = false
  verifying.value = false
  starting.value = false
  verificationError.value = ''
  availableSpace.value = 0
  
  contributionForm.value = {
    storagePath: '',
    amount: 0
  }
  
  Object.keys(pathChecks.value).forEach(key => {
    pathChecks.value[key] = false
  })
  
  updateContributionAmount()
}

const handleClose = () => {
  if (starting.value || verifying.value) {
    return false
  }
  visible.value = false
  return true
}

// Initialize
updateContributionAmount()
</script>

<style scoped>
.contribution-setup {
  padding: 20px 0;
}

.setup-steps {
  margin-bottom: 40px;
}

.setup-step {
  min-height: 400px;
}

.setup-step h3 {
  margin: 0 0 16px 0;
  color: #303133;
  font-size: 20px;
  font-weight: 600;
}

.step-description {
  color: #606266;
  margin-bottom: 32px;
  line-height: 1.6;
}

.form-hint {
  color: #909399;
  font-size: 12px;
  margin-top: 4px;
  line-height: 1.4;
}

.amount-input {
  display: flex;
  align-items: center;
}

.ratio-display {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.ratio-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.ratio-label {
  font-size: 12px;
  color: #909399;
}

.ratio-value {
  font-size: 16px;
  font-weight: 600;
}

.ratio-value.contribute {
  color: #e6a23c;
}

.ratio-value.earn {
  color: #67c23a;
}

.ratio-arrow {
  font-size: 20px;
  color: #409eff;
}

.verification-status {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin: 24px 0;
}

.verification-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e4e7ed;
  transition: all 0.3s ease;
}

.verification-item.success {
  border-color: #67c23a;
  background-color: #f0f9ff;
  color: #67c23a;
}

.verification-item.pending {
  border-color: #e6a23c;
  background-color: #fdf6ec;
  color: #e6a23c;
}

.verification-item .el-icon {
  font-size: 18px;
}

.verification-error {
  margin: 16px 0;
}

.contribution-summary {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 8px;
  margin-bottom: 24px;
}

.summary-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.summary-label {
  color: #606266;
  font-weight: 500;
}

.summary-value {
  color: #303133;
  font-weight: 600;
}

.summary-value.earned {
  color: #67c23a;
}

.contribution-agreement {
  margin-bottom: 24px;
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  background-color: #fafbfc;
}

.step-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 32px;
  padding-top: 20px;
  border-top: 1px solid #e4e7ed;
}

.terms-content {
  line-height: 1.6;
}

.terms-content h4 {
  margin: 0 0 16px 0;
  color: #303133;
}

.terms-content ol {
  padding-left: 20px;
}

.terms-content li {
  margin-bottom: 8px;
  color: #606266;
}

/* Animation for verification items */
.verification-item {
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>