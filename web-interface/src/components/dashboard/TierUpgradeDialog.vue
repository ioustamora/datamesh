<template>
  <el-dialog
    v-model="visible"
    title="Upgrade Storage Tier"
    width="500px"
    :before-close="handleClose"
    append-to-body
  >
    <div class="tier-upgrade">
      <div v-if="selectedTier" class="upgrade-info">
        <h3>Upgrade to {{ selectedTier.name }} Tier</h3>
        <p>{{ selectedTier.description }}</p>
        
        <div class="tier-comparison">
          <div class="tier-specs">
            <h4>{{ selectedTier.name }} Benefits:</h4>
            <ul>
              <li>{{ formatSize(selectedTier.max_storage) }} storage space</li>
              <li>{{ formatSize(selectedTier.upload_quota) }} monthly upload</li>
              <li>{{ formatSize(selectedTier.download_quota) }} monthly download</li>
            </ul>
          </div>
          
          <div v-if="selectedTier.monthly_cost" class="pricing">
            <div class="price">${{ selectedTier.monthly_cost }}/month</div>
          </div>
        </div>
        
        <el-form v-if="selectedTier.monthly_cost" :model="upgradeForm" label-width="120px">
          <el-form-item label="Payment Method">
            <el-select v-model="upgradeForm.paymentMethod" placeholder="Select payment method">
              <el-option label="Credit Card" value="card" />
              <el-option label="PayPal" value="paypal" />
              <el-option label="Bank Transfer" value="bank" />
            </el-select>
          </el-form-item>
        </el-form>
      </div>
    </div>
    
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">Cancel</el-button>
        <el-button type="primary" @click="startUpgrade" :loading="upgrading">
          {{ upgrading ? 'Processing...' : 'Upgrade Now' }}
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch } from 'vue'

const emit = defineEmits(['update:modelValue', 'success'])
const props = defineProps({
  modelValue: Boolean,
  selectedTier: Object
})

const visible = ref(false)
const upgrading = ref(false)
const upgradeForm = ref({
  paymentMethod: ''
})

watch(() => props.modelValue, (val) => {
  visible.value = val
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

const formatSize = (bytes) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const startUpgrade = async () => {
  upgrading.value = true
  
  // Simulate upgrade process
  setTimeout(() => {
    upgrading.value = false
    emit('success')
    visible.value = false
  }, 2000)
}

const handleClose = () => {
  visible.value = false
}
</script>

<style scoped>
.tier-upgrade {
  padding: 20px 0;
}

.upgrade-info h3 {
  margin: 0 0 12px 0;
  color: #303133;
}

.tier-comparison {
  margin: 20px 0;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 8px;
}

.tier-specs h4 {
  margin: 0 0 12px 0;
  color: #409eff;
}

.tier-specs ul {
  margin: 0;
  padding-left: 20px;
}

.tier-specs li {
  margin-bottom: 8px;
  color: #606266;
}

.pricing {
  margin-top: 16px;
  text-align: center;
}

.price {
  font-size: 24px;
  font-weight: 600;
  color: #409eff;
}
</style>